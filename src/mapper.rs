use thiserror::Error;

use crate::cartridge::ScreenMirroring;

pub type MapperResult<T> = Result<T, MapperError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MapperError {
    #[error("Unsupported mapper ID: {0}")]
    UnsupportedMapper(u16),
    #[error("Unsupported PRG ROM size: {0} bytes")]
    UnsupportedPrgRomSize(usize),
}

pub enum ChrMapping {
    Rom(usize),
    Ram(usize),
}

pub enum PrgMapping {
    Rom(usize),
    Ram(usize),
    NvRam(usize),
}

#[derive(Debug)]
pub struct Mapper {
    prg_rom_size: usize,
    chr_rom_size: usize,
    chr_ram_size: usize,
    prg_ram_size: usize,
    prg_nvram_size: usize,

    submapper: usize,

    pub(crate) kind: MapperKind,
}

#[derive(Debug)]
pub enum MapperKind {
    Nrom,
    Mmc1 {
        shift_register: u8,
        control: u8,
        chr_bank_0: u8,
        chr_bank_1: u8,
        prg_bank: u8,
        last_write_cycle: Option<usize>,
    },
    Uxrom {
        prg_bank: usize,
    },
    Cnrom {
        chr_bank: usize,
    },
}

impl Default for Mapper {
    fn default() -> Self {
        Self {
            prg_rom_size: 0x4000,
            chr_rom_size: 0,
            chr_ram_size: 0,
            prg_ram_size: 0,
            prg_nvram_size: 0,

            submapper: 0,

            kind: MapperKind::Nrom,
        }
    }
}

impl Mapper {
    pub fn new(
        mapper_id: u16,
        prg_rom_size: usize,
        chr_rom_size: usize,
        chr_ram_size: usize,
        prg_ram_size: usize,
        prg_nvram_size: usize,
        submapper: usize,
    ) -> MapperResult<Self> {
        if mapper_id == 1 && submapper == 5 && prg_rom_size != 0x8000 {
            return Err(MapperError::UnsupportedPrgRomSize(prg_rom_size));
        }

        let kind = match (mapper_id, prg_rom_size) {
            (0, 0x4000 | 0x8000) => MapperKind::Nrom,
            (1, size) if valid_mapper_size(size, 0x4000) => MapperKind::Mmc1 {
                shift_register: 0x10,
                control: 0x0C,
                chr_bank_0: 0,
                chr_bank_1: 0,
                prg_bank: 0,
                last_write_cycle: None,
            },
            (2, size) if valid_mapper_size(size, 0x8000) => MapperKind::Uxrom { prg_bank: 0 },
            (3, 0x4000 | 0x8000) => MapperKind::Cnrom { chr_bank: 0 },
            (0..=3, _) => return Err(MapperError::UnsupportedPrgRomSize(prg_rom_size)),
            _ => return Err(MapperError::UnsupportedMapper(mapper_id)),
        };

        Ok(Self {
            prg_rom_size,
            chr_rom_size,
            chr_ram_size,
            prg_ram_size,
            prg_nvram_size,
            submapper,
            kind,
        })
    }

    pub fn mirroring(&self) -> Option<ScreenMirroring> {
        match &self.kind {
            MapperKind::Mmc1 { control, .. } => Some(match *control & 0b11 {
                0 => ScreenMirroring::SingleScreenLower,
                1 => ScreenMirroring::SingleScreenUpper,
                2 => ScreenMirroring::Vertical,
                3 => ScreenMirroring::Horizontal,
                _ => unreachable!(),
            }),
            _ => None,
        }
    }

    pub fn map_chr(&self, address: u16) -> ChrMapping {
        let offset = match &self.kind {
            MapperKind::Nrom | MapperKind::Uxrom { .. } => usize::from(address),
            MapperKind::Mmc1 {
                control,
                chr_bank_0,
                chr_bank_1,
                ..
            } => mmc1_map_chr(*control, *chr_bank_0, *chr_bank_1, address),
            MapperKind::Cnrom { chr_bank } => *chr_bank * 0x2000 + usize::from(address),
        };

        match (self.chr_rom_size, self.chr_ram_size) {
            (rom_size, 0) if rom_size > 0 => ChrMapping::Rom(offset % rom_size),
            (0, ram_size) if ram_size > 0 => ChrMapping::Ram(offset % ram_size),
            (0, 0) => panic!("Cartridge has no CHR memory"),
            _ => panic!("Mixed CHR ROM/RAM mapping is not supported"),
        }
    }

    pub fn map_prg_rom(&self, address: u16) -> usize {
        match &self.kind {
            MapperKind::Nrom | MapperKind::Cnrom { .. } => {
                nrom_map_prg_rom(self.prg_rom_size, address)
            }
            MapperKind::Mmc1 { .. } if self.submapper == 5 => usize::from(address - 0x8000),
            MapperKind::Mmc1 {
                control, prg_bank, ..
            } => mmc1_map_prg_rom(self.prg_rom_size, *control, *prg_bank, address),
            MapperKind::Uxrom { prg_bank } => {
                uxrom_map_prg_rom(self.prg_rom_size, *prg_bank, address)
            }
        }
    }

    pub fn map_prg(&self, address: u16) -> Option<PrgMapping> {
        match address {
            0x6000..=0x7FFF => {
                if matches!(&self.kind, MapperKind::Mmc1 { prg_bank, .. } if *prg_bank & 0x10 != 0)
                {
                    return None;
                }

                let offset = usize::from(address - 0x6000);

                match (self.prg_ram_size, self.prg_nvram_size) {
                    (0, 0) => None,
                    (ram_size, 0) => Some(PrgMapping::Ram(offset % ram_size)),
                    (0, nvram_size) => Some(PrgMapping::NvRam(offset % nvram_size)),
                    _ => panic!("Mixed PRG RAM/NVRAM mapping is not supported"),
                }
            }
            0x8000..=0xFFFF => Some(PrgMapping::Rom(self.map_prg_rom(address))),
            _ => None,
        }
    }

    pub fn write_register(&mut self, address: u16, value: u8, cpu_cycle: usize) {
        match &mut self.kind {
            MapperKind::Nrom => {}
            MapperKind::Mmc1 {
                shift_register,
                control,
                chr_bank_0,
                chr_bank_1,
                prg_bank,
                last_write_cycle,
            } => mmc1_write_register(
                shift_register,
                control,
                chr_bank_0,
                chr_bank_1,
                prg_bank,
                last_write_cycle,
                address,
                value,
                cpu_cycle,
            ),
            MapperKind::Cnrom { chr_bank } => {
                *chr_bank = usize::from(value & 0b11);
            }
            MapperKind::Uxrom { prg_bank } => *prg_bank = usize::from(value),
        }
    }
}

fn nrom_map_prg_rom(prg_rom_size: usize, address: u16) -> usize {
    let index = (address - 0x8000) as usize;

    match prg_rom_size {
        0x4000 => index & 0x3FFF,
        0x8000 => index,
        _ => panic!("Unsupported NROM PRG ROM size"),
    }
}

fn uxrom_map_prg_rom(prg_rom_size: usize, prg_bank: usize, address: u16) -> usize {
    let bank_count = prg_rom_size / 0x4000;

    let bank = if address < 0xC000 {
        prg_bank % bank_count
    } else {
        bank_count - 1
    };

    let offset = usize::from(address) & 0x3FFF;

    bank * 0x4000 + offset
}

fn mmc1_write_register(
    shift_register: &mut u8,
    control: &mut u8,
    chr_bank_0: &mut u8,
    chr_bank_1: &mut u8,
    prg_bank: &mut u8,
    last_write_cycle: &mut Option<usize>,
    address: u16,
    value: u8,
    cpu_cycle: usize,
) {
    let consecutive = last_write_cycle.is_some_and(|last| cpu_cycle == last.wrapping_add(1));

    *last_write_cycle = Some(cpu_cycle);

    if value & 0x80 != 0 {
        *shift_register = 0x10;
        *control |= 0x0C;

        return;
    }

    if consecutive {
        return;
    }

    let complete = *shift_register & 1 != 0;
    *shift_register = (*shift_register >> 1) | ((value & 1) << 4);

    if complete {
        match (address >> 13) & 0b11 {
            0 => *control = *shift_register,
            1 => *chr_bank_0 = *shift_register,
            2 => *chr_bank_1 = *shift_register,
            3 => *prg_bank = *shift_register,
            _ => unreachable!(),
        }

        *shift_register = 0x10;
    }
}

fn mmc1_map_prg_rom(prg_rom_size: usize, control: u8, prg_bank: u8, address: u16) -> usize {
    let bank_count = prg_rom_size / 0x4000;
    let selected = usize::from(prg_bank & 0x0F);
    let upper_half = address >= 0xC000;
    let mode = (control >> 2) & 0b11;

    let bank = match mode {
        0 | 1 => (selected & !1) + usize::from(upper_half),
        2 => {
            if upper_half {
                selected
            } else {
                0
            }
        }
        3 => {
            if upper_half {
                bank_count - 1
            } else {
                selected
            }
        }
        _ => unreachable!(),
    };

    let offset = usize::from(address) & 0x3FFF;

    (bank % bank_count) * 0x4000 + offset
}

fn mmc1_map_chr(control: u8, chr_bank_0: u8, chr_bank_1: u8, address: u16) -> usize {
    if control & 0x10 == 0 {
        let bank = usize::from(chr_bank_0 & !1);
        return bank * 0x1000 + usize::from(address);
    }

    let bank = if address < 0x1000 {
        chr_bank_0
    } else {
        chr_bank_1
    };

    usize::from(bank) * 0x1000 + (usize::from(address) & 0x0FFF)
}

fn valid_mapper_size(size: usize, min_size: usize) -> bool {
    size.is_power_of_two() && (min_size..=0x40000).contains(&size)
}
