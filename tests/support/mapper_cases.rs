// Pinned fixtures and explicit prerequisites for our mapper 0–5 plan.
macro_rules! holy_test {
    ($name:ident, $rom:literal $(, $reason:literal)?) => {
        #[test]
        $(#[ignore = $reason])?
        fn $name() {
            holy_mapperel::assert_full($rom);
        }
    };
}

#[test]
fn nrom_holy_mapperel_banking() {
    let name = "M0_P32K_C8K_V.nes";
    holy_mapperel::assert_banking(name, &holy_mapperel::run(name));
}

#[test]
fn cnrom_holy_mapperel_banking() {
    let name = "M3_P32K_C32K_H.nes";
    holy_mapperel::assert_banking(name, &holy_mapperel::run(name));
}
holy_test!(nrom_holy_p32k_c8k_v, "M0_P32K_C8K_V.nes");
holy_test!(nrom_holy_p32k_cr8k_v, "M0_P32K_CR8K_V.nes");
holy_test!(mmc1_holy_p128k_c128k, "M1_P128K_C128K.nes");
holy_test!(mmc1_holy_p128k_c128k_s8k, "M1_P128K_C128K_S8K.nes");
holy_test!(mmc1_holy_p128k_c128k_w8k, "M1_P128K_C128K_W8K.nes");
holy_test!(mmc1_holy_p128k_c32k, "M1_P128K_C32K.nes");
holy_test!(mmc1_holy_p128k_c32k_s8k, "M1_P128K_C32K_S8K.nes");
holy_test!(mmc1_holy_p128k_c32k_w8k, "M1_P128K_C32K_W8K.nes");
holy_test!(mmc1_holy_p128k_cr8k, "M1_P128K_CR8K.nes");
holy_test!(
    mmc1_holy_p512k_cr8k_s32k,
    "M1_P512K_CR8K_S32K.nes",
    "requires SXROM 512 KiB outer PRG banking and 32 KiB PRG RAM banking"
);
holy_test!(
    mmc1_holy_p512k_cr8k_s8k,
    "M1_P512K_CR8K_S8K.nes",
    "requires SUROM 512 KiB outer PRG banking"
);
holy_test!(uxrom_holy_p128k_cr8k_v, "M2_P128K_CR8K_V.nes");
holy_test!(cnrom_holy_p32k_c32k_h, "M3_P32K_C32K_H.nes");
holy_test!(
    mmc3_holy_p128k_cr32k,
    "M4_P128K_CR32K.nes",
    "requires mapper 4, bank switching and cartridge RAM/mirroring support"
);
holy_test!(
    mmc3_holy_p128k_cr8k,
    "M4_P128K_CR8K.nes",
    "requires mapper 4, bank switching and cartridge RAM/mirroring support"
);
holy_test!(
    mmc3_holy_p256k_c256k,
    "M4_P256K_C256K.nes",
    "requires mapper 4, bank switching and cartridge RAM/mirroring support"
);

#[test]
fn mmc1_serom_fixed_prg() {
    let path = mapper_roms::fixture("serom/serom.nes");
    // serom.c: fail initialized at $C154, stored at $0330; final loop $C375.
    mapper_roms::check_bytes(&path, 16 + 0x4154, &[0xA9, 0, 0x8D, 0x30, 3]);
    mapper_roms::check_bytes(&path, 16 + 0x4375, &[0x4C, 0x75, 0xC3]);
    let nes = mapper_roms::run_until(&path, 0xC375, CYCLE_BUDGET).unwrap();
    assert_eq!(
        nes.bus.peek(0x0330),
        0,
        "SEROM failed fixed-bank assertions"
    );
}

macro_rules! mmc3_irq_test {
    ($name:ident, $rom:literal) => {
        #[test]
        #[ignore = "requires mapper 4 and MMC3 IRQ counter / PPU A12 clocking"]
        fn $name() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/blargg/", $rom);
            let report = blargg_runner::run_rom(path, blargg_runner::DEFAULT_CYCLE_BUDGET);
            assert_eq!(report.outcome, blargg_runner::Outcome::Passed, "{report}");
        }
    };
}
mmc3_irq_test!(mmc3_test_1_clocking, "mmc3_test/1-clocking.nes");
mmc3_irq_test!(mmc3_test_2_details, "mmc3_test/2-details.nes");
mmc3_irq_test!(mmc3_test_3_a12_clocking, "mmc3_test/3-A12_clocking.nes");
mmc3_irq_test!(
    mmc3_test_4_scanline_timing,
    "mmc3_test/4-scanline_timing.nes"
);
mmc3_irq_test!(mmc3_test_5_mmc3, "mmc3_test/5-MMC3.nes");
mmc3_irq_test!(mmc3_test_6_mmc6, "mmc3_test/6-MMC6.nes");
mmc3_irq_test!(mmc3_test_2_1_clocking, "mmc3_test_2/1-clocking.nes");
mmc3_irq_test!(mmc3_test_2_2_details, "mmc3_test_2/2-details.nes");
mmc3_irq_test!(mmc3_test_2_3_a12_clocking, "mmc3_test_2/3-A12_clocking.nes");
mmc3_irq_test!(
    mmc3_test_2_4_scanline_timing,
    "mmc3_test_2/4-scanline_timing.nes"
);
mmc3_irq_test!(mmc3_test_2_5_mmc3, "mmc3_test_2/5-MMC3.nes");
mmc3_irq_test!(mmc3_test_2_6_mmc3_alt, "mmc3_test_2/6-MMC3_alt.nes");

macro_rules! mmc5_ram_test {
    ($name:ident, $rom:literal, $kib:literal) => {
        #[test]
        #[ignore = "requires MMC5 PRG RAM banking and NES 2.0 RAM sizing"]
        fn $name() {
            mmc5::assert_ram_capacity($rom, $kib);
        }
    };
}
mmc5_ram_test!(mmc5_u_ram_0k, "mmc5ramsize_u_ines2_0", 0);
mmc5_ram_test!(mmc5_u_ram_8k, "mmc5ramsize_u_ines2_1", 8);
mmc5_ram_test!(mmc5_u_ram_16k, "mmc5ramsize_u_ines2_2", 16);
mmc5_ram_test!(mmc5_u_ram_32k, "mmc5ramsize_u_ines2_4", 32);
mmc5_ram_test!(mmc5_u_ram_64k, "mmc5ramsize_u_ines2_8", 64);
mmc5_ram_test!(mmc5_u_ram_128k, "mmc5ramsize_u_ines2_16", 128);
mmc5_ram_test!(mmc5_s_ram_0k, "mmc5ramsize_s_ines2_0", 0);
mmc5_ram_test!(mmc5_s_ram_8k, "mmc5ramsize_s_ines2_1", 8);
mmc5_ram_test!(mmc5_s_ram_16k, "mmc5ramsize_s_ines2_2", 16);
mmc5_ram_test!(mmc5_s_ram_32k, "mmc5ramsize_s_ines2_4", 32);
mmc5_ram_test!(mmc5_s_ram_64k, "mmc5ramsize_s_ines2_8", 64);
mmc5_ram_test!(mmc5_s_ram_128k, "mmc5ramsize_s_ines2_16", 128);
mmc5_ram_test!(
    mmc5_mixed_volatile_and_battery_ram,
    "mmc5ramsize_s_ines2_2u1",
    16
);

macro_rules! deferred_test {
    ($name:ident, $rom:literal, $reason:literal) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            assert!(std::path::Path::new(&mapper_roms::fixture($rom)).is_file());
            panic!("{}: {}", $rom, $reason);
        }
    };
}
deferred_test!(
    mmc5_u_ines1_ram_policy,
    "mmc5ramsize/mmc5ramsize_u_ines1.nes",
    "requires mapper 5 and an explicit default RAM capacity policy for iNES 1"
);
deferred_test!(
    mmc5_u_extended_ram_256k,
    "mmc5ramsize/mmc5ramsize_u_ines2_32.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
deferred_test!(
    mmc5_u_extended_ram_512k,
    "mmc5ramsize/mmc5ramsize_u_ines2_64.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
deferred_test!(
    mmc5_u_extended_ram_1024k,
    "mmc5ramsize/mmc5ramsize_u_ines2_128.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
deferred_test!(
    mmc5_s_ines1_ram_policy,
    "mmc5ramsize/mmc5ramsize_s_ines1.nes",
    "requires mapper 5 and an explicit default RAM capacity policy for iNES 1"
);
deferred_test!(
    mmc5_s_extended_ram_256k,
    "mmc5ramsize/mmc5ramsize_s_ines2_32.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
deferred_test!(
    mmc5_s_extended_ram_512k,
    "mmc5ramsize/mmc5ramsize_s_ines2_64.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
deferred_test!(
    mmc5_s_extended_ram_1024k,
    "mmc5ramsize/mmc5ramsize_s_ines2_128.nes",
    "characterization: exceeds standard MMC5 128 KiB RAM addressing; needs an explicit extension policy and result oracle"
);
