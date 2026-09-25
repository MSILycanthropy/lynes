#[path = "support/mapper_roms.rs"]
mod mapper_roms;

#[path = "support/blargg.rs"]
mod blargg_runner;
#[path = "support/holy_mapperel.rs"]
mod holy_mapperel;
#[path = "support/mmc5.rs"]
mod mmc5;

use mapper_roms::{Board, CYCLE_BUDGET, assert_banking_and_bus, run};

include!("support/mapper_cases.rs");

macro_rules! cnrom_test {
    ($name:ident, $submapper:expr) => {
        #[test]
        fn $name() {
            let result = run(Board::Cnrom, $submapper, CYCLE_BUDGET).unwrap();
            assert_banking_and_bus(Board::Cnrom, $submapper, &result);
        }
    };
}

cnrom_test!(cnrom_submapper_0_banking_and_bus, 0);
cnrom_test!(cnrom_submapper_1_banking_without_conflicts, 1);
cnrom_test!(cnrom_submapper_2_banking_with_and_conflicts, 2);

macro_rules! cnrom_full_test {
    ($name:ident, $submapper:expr) => {
        #[test]
        fn $name() {
            let result = run(Board::Cnrom, $submapper, CYCLE_BUDGET).unwrap();
            assert_banking_and_bus(Board::Cnrom, $submapper, &result);
            assert_eq!(result.prg_ram_present, Some(0), "{result:#?}");
        }
    };
}

cnrom_full_test!(cnrom_submapper_0_full_rom, 0);
cnrom_full_test!(cnrom_submapper_1_full_rom, 1);
cnrom_full_test!(cnrom_submapper_2_full_rom, 2);

macro_rules! uxrom_test {
    ($name:ident, $submapper:expr) => {
        #[test]
        fn $name() {
            let result = run(Board::Uxrom, $submapper, CYCLE_BUDGET).unwrap();
            assert_banking_and_bus(Board::Uxrom, $submapper, &result);
        }
    };
}

uxrom_test!(uxrom_submapper_0_full_rom, 0);
uxrom_test!(uxrom_submapper_1_full_rom, 1);
uxrom_test!(uxrom_submapper_2_full_rom, 2);

#[test]
fn unfinished_rom_is_not_a_pass() {
    let error = run(Board::Cnrom, 1, 100).unwrap_err();
    assert!(error.contains("timed out"), "{error}");
}

#[test]
fn conflict_adapter_rejects_the_wrong_submapper_behavior() {
    let result = run(Board::Cnrom, 1, CYCLE_BUDGET).unwrap();
    assert!(
        std::panic::catch_unwind(|| {
            assert_banking_and_bus(Board::Cnrom, 2, &result);
        })
        .is_err()
    );
}

#[test]
fn mmc5_capacity_adapter_rejects_missing_ram_and_inconsistent_aliases() {
    // Two 8 KiB chips may have representative bank tags 0 and 4; don't assume
    // that physical bank identifiers are contiguous.
    let results = std::array::from_fn(|i| (i & 4) as u8);
    mmc5::assert_capacity(&results, 16);
    assert!(std::panic::catch_unwind(|| mmc5::assert_capacity(&results, 32)).is_err());
    assert!(std::panic::catch_unwind(|| mmc5::assert_capacity(&[0xFF; 128], 8)).is_err());
    let mut bad_alias = results;
    bad_alias[0] = 4;
    assert!(std::panic::catch_unwind(|| mmc5::assert_capacity(&bad_alias, 16)).is_err());
}
