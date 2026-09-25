#[path = "support/blargg.rs"]
mod blargg_runner;
#[path = "support/legacy.rs"]
mod legacy_runner;

use blargg_runner::{DEFAULT_CYCLE_BUDGET, Outcome, run_rom};

fn assert_rom_passes(relative_path: &str, cycle_budget: usize) {
    let path = format!(
        "{}/tests/fixtures/blargg/{relative_path}",
        env!("CARGO_MANIFEST_DIR")
    );
    let report = run_rom(&path, cycle_budget);
    assert_eq!(report.outcome, Outcome::Passed, "{relative_path}\n{report}");
}

macro_rules! blargg_test {
    ($name:ident, $path:literal, $budget:literal $(, $reason:literal)?) => {
        #[test]
        $(#[ignore = $reason])?
        fn $name() {
            assert_rom_passes($path, $budget);
        }
    };
}

macro_rules! legacy_test {
    ($name:ident, $path:literal, $budget:literal $(, $reason:literal)?) => {
        #[test]
        $(#[ignore = $reason])?
        fn $name() {
            let report = legacy_runner::run($path, $budget);
            assert_eq!(report.outcome, Outcome::Passed, "{}\n{}", $path, report);
        }
    };
}

// Fixtures whose result protocol needs an adapter or human validation.
// Running one explicitly fails with its prerequisite, never a fabricated pass.
macro_rules! deferred_test {
    ($name:ident, $path:literal, $reason:literal) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            panic!("{}: {}", $path, $reason);
        }
    };
}

include!("support/blargg_cases.rs");

#[test]
#[ignore = "manual runner: set BLARGG_ROM to a compatible ROM path"]
fn external_rom() {
    let path = std::env::var("BLARGG_ROM").expect("set BLARGG_ROM to the ROM path");
    let report = run_rom(&path, DEFAULT_CYCLE_BUDGET);
    assert_eq!(report.outcome, Outcome::Passed, "{path}\n{report}");
}
