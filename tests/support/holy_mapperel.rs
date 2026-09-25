use super::mapper_roms::{check_bytes, fixture, run_until};

#[derive(Debug)]
pub struct Results {
    pub mapper: u8,
    pub prg_banks: usize,
    pub chr_rom: bool,
    pub chr_banks: usize,
    pub chr_test: u8,
    pub ram_present: bool,
    pub ram_banks: usize,
    pub ram_test: u8,
    pub prg_detail: u8,
    pub chr_detail: u8,
}

// v0.02 source: main.s, loadchr.s, wram.s, drivers.s, linked in makefile
// object order with ZEROPAGE starting at $10 (nrom256.x).
pub fn run(name: &str) -> Results {
    let path = fixture(&format!("holy_mapperel/testroms/{name}"));
    let bytes = std::fs::read(&path).unwrap();
    let prg_size = usize::from(bytes[4]) * 16384;
    // reset2: sentinel stores followed by JSR driver_mapper_test ($0406).
    // PC $C01B is reached on return, at "All tests are complete" in main.s.
    check_bytes(
        &path,
        16 + prg_size - 16384 + 0x10,
        &[
            0xA9, 0xC0, 0x85, 0x1F, 0xA9, 0xDE, 0x85, 0x20, 0x20, 0x06, 0x04,
        ],
    );
    let nes = run_until(&path, 0xC01B, 120_000_000).unwrap();
    let read = |address| nes.bus.peek(address);
    Results {
        mapper: read(0x15),
        prg_banks: usize::from(read(0x17)) + 1,
        chr_rom: read(0x18) != 0,
        chr_banks: usize::from(read(0x19)) + 1,
        chr_test: read(0x1A),
        ram_present: read(0x1C) != 0,
        ram_banks: usize::from(read(0x1D)) + 1,
        ram_test: read(0x1E),
        prg_detail: read(0x1F),
        chr_detail: read(0x20),
    }
}

pub fn assert_banking(name: &str, r: &Results) {
    let bytes = std::fs::read(fixture(&format!("holy_mapperel/testroms/{name}"))).unwrap();
    let mapper = (bytes[6] >> 4) | (bytes[7] & 0xF0);
    let chr_rom = bytes[5] != 0;
    let chr_size = if chr_rom {
        usize::from(bytes[5]) * 8192
    } else {
        64usize << (bytes[11] & 15)
    };
    assert_eq!(r.mapper, mapper, "{name}: {r:#?}");
    assert_eq!(
        r.prg_banks * 4096,
        usize::from(bytes[4]) * 16384,
        "{name}: {r:#?}"
    );
    assert_eq!(r.chr_rom, chr_rom, "{name}: {r:#?}");
    assert_eq!(r.chr_banks * 8192, chr_size, "{name}: {r:#?}");
    assert_eq!(r.chr_test, 0, "{name}: {r:#?}");
    assert_eq!(r.prg_detail, 0, "{name}: {r:#?}");
    assert_eq!(r.chr_detail, 0, "{name}: {r:#?}");
}

pub fn assert_full(name: &str) {
    let r = run(name);
    assert_banking(name, &r);
    let bytes = std::fs::read(fixture(&format!("holy_mapperel/testroms/{name}"))).unwrap();
    let size = |shift| if shift == 0 { 0 } else { 64usize << shift };
    let ram_size = size(bytes[10] & 15) + size(bytes[10] >> 4);
    assert_eq!(r.ram_present, ram_size != 0, "{name}: {r:#?}");
    if ram_size != 0 {
        assert_eq!(r.ram_banks * 8192, ram_size, "{name}: {r:#?}");
        assert_eq!(r.ram_test, 0, "{name}: {r:#?}");
    }
}
