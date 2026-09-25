use super::mapper_roms::{CYCLE_BUDGET, check_bytes, fixture, run_until};
use std::collections::BTreeSet;

// This asserts capacity and successful RAM probes, not a particular PCB's
// alias/chip-select wiring. The ROM labels each physical bank with the lowest
// register value that aliases it, which need not be a contiguous bank number.
pub fn assert_ram_capacity(name: &str, expected_kib: usize) {
    let path = fixture(&format!("mmc5ramsize/{name}.nes"));
    check_bytes(&path, 16 + 0x35C, &[0x4C, 0x5C, 0xC3]);
    let nes = run_until(&path, 0xC35C, CYCLE_BUDGET).unwrap();
    // mmc5ramsize.cfg: RAM starts $0200; optional OAM is absent.
    let results: [u8; 128] = std::array::from_fn(|i| nes.bus.peek(0x0200 + i as u16));
    assert_capacity(&results, expected_kib);
}

pub fn assert_capacity(results: &[u8; 128], expected_kib: usize) {
    let mut banks = BTreeSet::new();
    for &bank in results {
        if bank == 0xFF {
            continue;
        } // failed RAM probe / unpopulated chip
        assert!(
            bank < 128,
            "unexpected saved-data bit on fresh boot: {results:02X?}"
        );
        assert_eq!(
            results[usize::from(bank)],
            bank,
            "inconsistent RAM bank tags: {results:02X?}"
        );
        banks.insert(bank);
    }
    assert_eq!(
        banks.len() * 8,
        expected_kib,
        "RAM capacity mismatch: {results:02X?}"
    );
}
