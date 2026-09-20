use std::path::Path;

use crate::mapper::Mapper;

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq, Clone)]
pub enum ScreenMirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: Mapper,
    pub submapper: u8,
    pub screen_mirroring: ScreenMirroring,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            prg_rom: vec![],
            prg_ram: vec![0; 8192],
            chr_rom: vec![],
            mapper: Mapper::default(),
            submapper: 0,
            screen_mirroring: ScreenMirroring::Horizontal,
        }
    }
}

impl Cartridge {
    pub fn cpu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x6000..=0x7FFF => Some(self.prg_ram[(address - 0x6000) as usize]),
            0x8000..=0xFFFF => {
                let offset = self.mapper.map_prg_rom(address);
                Some(self.prg_rom[offset])
            }
            _ => None,
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0x4020..=0x5FFF => {}
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize] = value,
            0x8000..=0xFFFF => self.mapper.write_register(address, value),
            _ => panic!("Invalid cartridge CPU write address: {:#06X}", address),
        }
    }

    pub fn ppu_read(&self, address: u16) -> u8 {
        match address {
            0..=0x1FFF => {
                let offset = self.mapper.map_chr(address);
                self.chr_rom[offset]
            }
            _ => panic!("Invalid cartridge PPU read address: {:#06X}", address),
        }
    }

    pub fn ppu_write(&mut self, address: u16, _value: u8) {
        match address {
            0..=0x1FFF => {}
            _ => panic!("Invalid cartridge PPU write address: {:#06X}", address),
        }
    }

    pub fn load(file: impl AsRef<Path>) -> Self {
        let bytes = std::fs::read(file).expect("Unable to read Cartridge file");
        Self::load_bytes(&bytes)
    }

    fn load_bytes(bytes: &[u8]) -> Self {
        let rom = RomImage::parse(bytes);
        let mapper = Mapper::new(rom.mapper_id, rom.prg_rom.len())
            .unwrap_or_else(|error| panic!("Unable to initialize cartridge mapper: {error}"));

        Self {
            prg_rom: rom.prg_rom,
            prg_ram: vec![0; 8192],
            chr_rom: rom.chr_rom,
            mapper,
            submapper: rom.submapper,
            screen_mirroring: rom.screen_mirroring,
        }
    }
}

// File-format parsing is independent of which cartridge hardware we can emulate.
struct RomImage {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper_id: u16,
    submapper: u8,
    screen_mirroring: ScreenMirroring,
}

impl RomImage {
    fn parse(bytes: &[u8]) -> Self {
        if bytes[0..4] != NES_TAG {
            panic!("Invalid NES file");
        }

        let mut mapper = ((bytes[7] & 0xF0) | (bytes[6] >> 4)) as u16;
        let ines_ver = (bytes[7] >> 2) & 0x03;

        if ines_ver != 0 && ines_ver != 2 {
            panic!("Unsupported iNES version");
        }

        let submapper = if ines_ver == 2 {
            mapper |= ((bytes[8] & 0x0F) as u16) << 8;
            bytes[8] >> 4
        } else {
            0
        };

        let four_screen = bytes[6] & 0x08 != 0;
        let vertical_mirroring = bytes[6] & 0x01 != 0;

        let screen_mirroring = if four_screen {
            ScreenMirroring::FourScreen
        } else if vertical_mirroring {
            ScreenMirroring::Vertical
        } else {
            ScreenMirroring::Horizontal
        };

        let size_msb = if ines_ver == 2 { bytes[9] } else { 0 };
        let prg_rom_size =
            rom_size(bytes[4], size_msb & 0x0F, PRG_ROM_PAGE_SIZE).expect("PRG ROM size overflow");
        let chr_rom_size =
            rom_size(bytes[5], size_msb >> 4, CHR_ROM_PAGE_SIZE).expect("CHR ROM size overflow");

        let skip_trainer = bytes[6] & 0x04 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Self {
            prg_rom: bytes[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: bytes[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper_id: mapper,
            submapper: submapper,
            screen_mirroring: screen_mirroring,
        }
    }
}

fn rom_size(lsb: u8, msb_nibble: u8, unit: usize) -> Option<usize> {
    if msb_nibble == 0xF {
        let exponent = (lsb >> 2) as u32;
        let multiplier = (lsb & 0b11) as usize * 2 + 1;

        1usize.checked_shl(exponent)?.checked_mul(multiplier)
    } else {
        let units = ((msb_nibble as usize) << 8) | lsb as usize;
        units.checked_mul(unit)
    }
}

pub mod test {
    use super::*;

    #[allow(dead_code)]
    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    #[allow(dead_code)]
    fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer.as_ref().map_or(0, |t| t.len())
                + rom.prg_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.prg_rom);
        result.extend(&rom.chr_rom);

        result
    }

    #[allow(dead_code)]
    pub fn test_rom(prg_rom: Option<Vec<u8>>) -> Cartridge {
        let prg_rom = match prg_rom {
            Some(p) => {
                let mut result = vec![0; 2 * PRG_ROM_PAGE_SIZE];
                result[..p.len()].copy_from_slice(&p);
                result[0x7FFD] = 0x80;
                result
            }
            None => vec![1; 2 * PRG_ROM_PAGE_SIZE],
        };
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x01, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            prg_rom: prg_rom,
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        Cartridge::load_bytes(&test_rom)
    }

    #[test]
    fn test() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper_id, 3);
        assert_eq!(rom.screen_mirroring, ScreenMirroring::Vertical);
    }

    #[test]
    fn test_load_nrom_from_ines_and_nes2() {
        for flags_7 in [0, 0x08] {
            let mut prg_rom = vec![0xA5; 2 * PRG_ROM_PAGE_SIZE];
            prg_rom[PRG_ROM_PAGE_SIZE..].fill(0x5A);
            let bytes = create_rom(TestRom {
                header: vec![
                    0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x04, flags_7, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                trainer: Some(vec![0xFF; 512]),
                prg_rom,
                chr_rom: vec![0x12; CHR_ROM_PAGE_SIZE],
            });

            let cart = Cartridge::load_bytes(&bytes);

            assert_eq!(cart.cpu_read(0x8000), Some(0xA5));
            assert_eq!(cart.cpu_read(0xC000), Some(0x5A));
            assert_eq!(cart.ppu_read(0), 0x12);
            assert_eq!(cart.ppu_read(0x1FFF), 0x12);
        }
    }

    #[test]
    fn test_nes2_mapper_and_submapper() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x30, 0xA8, 0xB5, 0, 0, 0, 0, 0, 0, 0,
            ],
            trainer: None,
            prg_rom: vec![1; PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.mapper_id, 0x5A3);
        assert_eq!(rom.submapper, 0xB);
        assert_eq!(rom.prg_rom, vec![1; PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![2; CHR_ROM_PAGE_SIZE]);
        assert_eq!(rom.screen_mirroring, ScreenMirroring::Horizontal);
    }

    #[test]
    fn test_ines_byte_8_does_not_extend_mapper() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x30, 0xA0, 0xB5, 0, 0, 0, 0, 0, 0, 0,
            ],
            trainer: None,
            prg_rom: vec![1; PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.mapper_id, 0xA3);
        assert_eq!(rom.submapper, 0);
    }

    #[test]
    fn test_nes2_extended_prg_rom_size() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0, 0x08, 0, 0x01, 0, 0, 0, 0, 0, 0,
            ],
            trainer: None,
            prg_rom: vec![0xA5; 258 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![0x5A; CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.prg_rom, vec![0xA5; 258 * PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![0x5A; CHR_ROM_PAGE_SIZE]);
    }

    #[test]
    fn test_nes2_exponent_multiplier_prg_rom_size() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x35, 0x01, 0, 0x08, 0, 0x0F, 0, 0, 0, 0, 0, 0,
            ],
            trainer: None,
            // 0x35 encodes exponent 13 and multiplier 3: 24 KiB.
            prg_rom: vec![0xA5; 24 * 1024],
            chr_rom: vec![0x5A; CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.prg_rom, vec![0xA5; 24 * 1024]);
        assert_eq!(rom.chr_rom, vec![0x5A; CHR_ROM_PAGE_SIZE]);
    }

    #[test]
    fn test_ines_tv_system_does_not_extend_rom_sizes() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0,
            ],
            trainer: None,
            prg_rom: vec![0xA5; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![0x5A; CHR_ROM_PAGE_SIZE],
        });

        let rom = Cartridge::load_bytes(&test_rom);

        assert_eq!(rom.prg_rom, vec![0xA5; 2 * PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![0x5A; CHR_ROM_PAGE_SIZE]);
    }

    #[test]
    fn test_with_trainer() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E,
                0x45,
                0x53,
                0x1A,
                0x02,
                0x01,
                0x31 | 0b100,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
                00,
            ],
            trainer: Some(vec![0; 512]),
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });

        let rom = RomImage::parse(&test_rom);

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper_id, 3);
        assert_eq!(rom.screen_mirroring, ScreenMirroring::Vertical);
    }
}
