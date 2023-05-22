const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq)]
pub enum ScreenMirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8, // TODO: implement mappers
    pub screen_mirroring: ScreenMirroring,
}

impl Cartridge {
    pub fn load(file: &str) -> Self {
        let bytes = std::fs::read(file).expect("Unable to read Cartridge file");
        Self::load_bytes(&bytes)
    }

    fn load_bytes(bytes: &Vec<u8>) -> Self {
        if bytes[0..4] != NES_TAG {
            panic!("Invalid NES file");
        }

        let mapper = (bytes[7] & 0xF0) | (bytes[6] >> 4);
        let ines_ver = (bytes[7] >> 2) & 0x03;

        if ines_ver != 0 {
            panic!("Unsupported iNES version");
        }

        let four_screen = bytes[6] & 0x08 != 0;
        let vertical_mirroring = bytes[6] & 0x01 != 0;

        let screen_mirroring = if four_screen {
            ScreenMirroring::FourScreen
        } else if vertical_mirroring {
            ScreenMirroring::Vertical
        } else {
            ScreenMirroring::Horizontal
        };

        let prg_rom_size = bytes[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = bytes[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = bytes[6] & 0x04 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Self {
            prg_rom: bytes[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: bytes[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        }
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
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
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

        let rom: Cartridge = Cartridge::load_bytes(&test_rom);

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, ScreenMirroring::Vertical);
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

        let rom: Cartridge = Cartridge::load_bytes(&test_rom);

        assert_eq!(rom.chr_rom, vec!(2; 1 * CHR_ROM_PAGE_SIZE));
        assert_eq!(rom.prg_rom, vec!(1; 2 * PRG_ROM_PAGE_SIZE));
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, ScreenMirroring::Vertical);
    }

    #[test]
    #[should_panic]
    fn test_nes2_is_not_supported() {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x31, 0x8, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            prg_rom: vec![1; 1 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let _ = Cartridge::load_bytes(&test_rom);
    }
}
