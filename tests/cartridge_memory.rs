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
            mapper: Mapper::new(0, size, 0, 0, 0, 0, 0).unwrap(),
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
            mapper: Mapper::new(0, size, 0, 0, 0, 0, 0).unwrap(),
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

fn ram_cartridge(size: usize, battery_backed: bool) -> Cartridge {
    let (ram_size, nvram_size) = if battery_backed { (0, size) } else { (size, 0) };
    Cartridge {
        mapper: Mapper::new(0, 32_768, 0, 8192, ram_size, nvram_size, 0).unwrap(),
        prg_rom: vec![0; 32_768],
        prg_ram: vec![0; ram_size],
        prg_nvram: vec![0; nvram_size],
        chr_ram: vec![0; 8192],
        ..Cartridge::default()
    }
}

#[test]
fn prg_ram_and_nvram_survive_reset() {
    for battery_backed in [false, true] {
        let mut nes = NES::default();
        nes.insert_cart(ram_cartridge(8192, battery_backed));
        nes.cpu_write(0x6000, 0x12);
        nes.cpu_write(0x7FFF, 0x34);
        nes.reset();
        assert_eq!(nes.cpu_read(0x6000), 0x12);
        assert_eq!(nes.cpu_read(0x7FFF), 0x34);
    }
}

#[test]
fn cartridge_insertion_preserves_incoming_ram_not_previous_ram() {
    for battery_backed in [false, true] {
        let mut nes = NES::default();
        nes.insert_cart(ram_cartridge(8192, battery_backed));
        nes.cpu_write(0x6000, 0x12);
        nes.cpu_write(0x7FFF, 0x34);

        let mut incoming = ram_cartridge(2048, battery_backed);
        incoming.cpu_write(0x6000, 0x56, 0);
        incoming.cpu_write(0x67FF, 0x78, 0);
        nes.insert_cart(incoming);

        assert_eq!(nes.cpu_read(0x6000), 0x56);
        assert_eq!(nes.cpu_read(0x7FFF), 0x78);
    }
}

#[test]
fn small_prg_ram_and_nvram_mirror_reads_and_writes() {
    for battery_backed in [false, true] {
        let mut cart = ram_cartridge(2048, battery_backed);
        cart.cpu_write(0x6800, 0x12, 0);
        cart.cpu_write(0x7FFF, 0x34, 0);
        for base in [0x6000, 0x6800, 0x7000, 0x7800] {
            assert_eq!(cart.cpu_read(base), Some(0x12));
            assert_eq!(cart.cpu_read(base + 0x7FF), Some(0x34));
        }
    }
}

#[test]
fn absent_prg_ram_ignores_writes_and_reads_open_bus() {
    let mut cart = ram_cartridge(0, false);
    for address in [0x6000, 0x7FFF] {
        cart.cpu_write(address, 0x12, 0);
        assert_eq!(cart.cpu_read(address), None);
    }
    let mut nes = NES::default();
    nes.insert_cart(cart);
    for address in [0x6000, 0x7FFF] {
        nes.cpu_write(address, 0x12);
        nes.cpu_write(0x0000, 0xA5);
        assert_eq!(nes.cpu_read(address), 0xA5);
    }
}
