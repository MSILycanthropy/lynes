use thiserror::Error;

pub type MapperResult<T> = Result<T, MapperError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MapperError {
    #[error("Unsupported mapper ID: {0}")]
    UnsupportedMapper(u16),
    #[error("Unsupported NROM PRG ROM size: {0} bytes (expected 16384 or 32768)")]
    UnsupportedNromPrgRomSize(usize),
}

#[derive(Debug)]
pub enum Mapper {
    Nrom { prg_rom_size: usize },
}

impl Default for Mapper {
    fn default() -> Self {
        Mapper::Nrom {
            prg_rom_size: 0x4000,
        }
    }
}

impl Mapper {
    pub fn new(mapper_id: u16, prg_rom_size: usize) -> MapperResult<Self> {
        match (mapper_id, prg_rom_size) {
            (0, 0x4000 | 0x8000) => Ok(Self::Nrom { prg_rom_size }),
            (0, _) => Err(MapperError::UnsupportedNromPrgRomSize(prg_rom_size)),
            _ => Err(MapperError::UnsupportedMapper(mapper_id)),
        }
    }

    pub fn map_chr(&self, address: u16) -> usize {
        match self {
            Self::Nrom { .. } => address as usize,
        }
    }

    pub fn map_prg_rom(&self, address: u16) -> usize {
        match self {
            Self::Nrom { prg_rom_size } => nrom_map_prg_rom(*prg_rom_size, address),
        }
    }

    pub fn write_register(&mut self, _address: u16, _value: u8) {
        match self {
            Self::Nrom { .. } => {}
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
