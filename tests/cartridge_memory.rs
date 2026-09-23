use lynes::{NES, cartridge::Cartridge, mapper::Mapper};

#[test]
fn cpu_reads_preserve_16_and_32_kib_prg_mapping() {
    for size in [16_384, 32_768] {
        let mut prg_rom = vec![0; size];
        prg_rom[0] = 0x12;
        prg_rom[0x3FFF] = 0x34;
        if size == 32_768 {
            prg_rom[0x4000] = 0x56;
            prg_rom[0x7FFF] = 0x78;
        }
        let mut nes = NES::default();
        nes.insert_cart(Cartridge {
            mapper: Mapper::new(0, size).unwrap(),
            prg_rom,
            ..Cartridge::default()
        });

        assert_eq!(nes.cpu_read(0x8000), 0x12);
        assert_eq!(nes.cpu_read(0xBFFF), 0x34);
        assert_eq!(
            nes.cpu_read(0xC000),
            if size == 16_384 { 0x12 } else { 0x56 }
        );
        assert_eq!(
            nes.cpu_read(0xFFFF),
            if size == 16_384 { 0x34 } else { 0x78 }
        );
    }
}

#[test]
fn nrom_register_writes_leave_prg_rom_and_mapping_unchanged() {
    for size in [16_384, 32_768] {
        let mut prg_rom = vec![0x12; size];
        prg_rom[size - 1] = 0x34;
        let mut nes = NES::default();
        nes.insert_cart(Cartridge {
            mapper: Mapper::new(0, size).unwrap(),
            prg_rom,
            ..Cartridge::default()
        });

        for address in [0x8000, 0xBFFF, 0xC000, 0xFFFF] {
            let before = nes.cpu_read(address);
            nes.cpu_write(address, 0xFF);
            assert_eq!(nes.cpu_read(address), before);
        }
        assert_eq!(nes.cpu_read(0x8000), 0x12);
        assert_eq!(nes.cpu_read(0xFFFF), 0x34);
    }
}

#[test]
fn prg_ram_survives_cartridge_insertion_and_reset() {
    let mut nes = NES::default();
    nes.cpu_write(0x6000, 0x12);
    nes.cpu_write(0x7FFF, 0x34);

    for _ in 0..2 {
        nes.insert_cart(Cartridge {
            mapper: Mapper::new(0, 32_768).unwrap(),
            prg_rom: vec![0; 32_768],
            ..Cartridge::default()
        });
        nes.reset();
        assert_eq!(nes.cpu_read(0x6000), 0x12);
        assert_eq!(nes.cpu_read(0x7FFF), 0x34);
    }
}
