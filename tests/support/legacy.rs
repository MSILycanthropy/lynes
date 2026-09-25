//! Adapters for the pinned legacy fixtures. See fixtures/blargg/LEGACY.md.
//! A result byte is meaningful only when its ROM enters the reporting routine.

use crate::blargg_runner::{Outcome, Report, run_rom, run_rom_observed};
use lynes::{NES, input::ButtonState};

#[derive(Clone, Copy)]
enum Protocol {
    Memory { pc: u16, address: u16 },
    Accumulator { pc: u16 },
    Crc { pc: u16, expected: &'static [u32] },
    Timing { pass_pc: u16, fail_pc: u16 },
    Observation { pc: u16, reason: &'static str },
    Measurement { pc: u16, address: u16 },
    Status,
}

struct Fixture {
    path: &'static str,
    fingerprint: u64,
    protocol: Protocol,
}

include!("legacy_fixtures.rs");

fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn unavailable(outcome: Outcome) -> Report {
    Report {
        outcome,
        cpu_cycles: 0,
        resets: 0,
        pc: 0,
        status: None,
        text: String::new(),
    }
}

fn observe(protocol: Protocol, nes: &NES) -> Option<(Outcome, Option<u8>, String)> {
    let current_pc = nes.cpu.registers.program_counter;
    let (outcome, code, text) = match protocol {
        Protocol::Memory { pc, address } if current_pc == pc => {
            let code = nes.bus.peek(address);
            let outcome = match code {
                0 => Outcome::InvalidStatus(0),
                1 => Outcome::Passed,
                _ => Outcome::Failed(code),
            };
            (
                outcome,
                Some(code),
                format!("legacy result at ${address:04X}"),
            )
        }
        Protocol::Accumulator { pc } if current_pc == pc => {
            let code = nes.cpu.registers.accumulator;
            let outcome = if code == 0 {
                Outcome::Passed
            } else {
                Outcome::Failed(code)
            };
            (outcome, Some(code), "legacy exit result in A".to_owned())
        }
        Protocol::Crc { pc, expected } if current_pc == pc => {
            let code = nes.cpu.registers.accumulator;
            let actual =
                !u32::from_le_bytes(std::array::from_fn(|i| nes.bus.peek(0x10 + i as u16)));
            let outcome = if code != 0 {
                Outcome::Failed(code)
            } else if expected.contains(&actual) {
                Outcome::Passed
            } else {
                Outcome::Failed(1)
            };
            (
                outcome,
                Some(code),
                format!("output CRC: {actual:08X}; expected one of {expected:08X?}"),
            )
        }
        Protocol::Timing { pass_pc, fail_pc } if current_pc == pass_pc || current_pc == fail_pc => {
            let passed = current_pc == pass_pc;
            (
                if passed {
                    Outcome::Passed
                } else {
                    Outcome::Failed(1)
                },
                None,
                format!(
                    "CPU timing: opcode ${:02X}, mode ${:02X}",
                    nes.bus.peek(0x13),
                    nes.bus.peek(0x14)
                ),
            )
        }
        Protocol::Observation { pc, reason } if current_pc == pc => (
            Outcome::NeedsValidation(reason.to_owned()),
            None,
            reason.to_owned(),
        ),
        Protocol::Measurement { pc, address } if current_pc == pc => {
            let code = nes.cpu.registers.accumulator;
            let reason = "DMC/controller conflict measurement; no asserted expected count";
            let outcome = if code == 0 {
                Outcome::NeedsValidation(reason.to_owned())
            } else {
                Outcome::Failed(code)
            };
            (
                outcome,
                Some(code),
                format!(
                    "Measured conflicts/errors: {} / 1000. {reason}",
                    nes.bus.peek(address)
                ),
            )
        }
        _ => return None,
    };
    Some((outcome, code, text))
}

pub fn run(relative_path: &str, cycle_budget: usize) -> Report {
    let Some(fixture) = FIXTURES
        .iter()
        .find(|fixture| fixture.path == relative_path)
    else {
        return unavailable(Outcome::LoadError(format!(
            "No legacy adapter for {relative_path}"
        )));
    };
    let path = format!(
        "{}/tests/fixtures/blargg/{relative_path}",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => return unavailable(Outcome::LoadError(error.to_string())),
    };
    if fingerprint(&bytes) != fixture.fingerprint {
        return unavailable(Outcome::LoadError(
            "Legacy fixture changed; re-verify its adapter before running".to_owned(),
        ));
    }
    if relative_path.starts_with("pal_apu_tests/") {
        return unavailable(Outcome::UnsupportedRegion(
            "PAL CPU/APU timing is not implemented".to_owned(),
        ));
    }

    if let Protocol::Status = fixture.protocol {
        let mut report = run_rom(&path, cycle_budget);
        // These diagnostic ROMs may print Done with status zero, rather than
        // assert a result. Preserve their output without claiming it passed.
        if report.outcome == Outcome::Passed
            && !report.text.lines().any(|line| line.trim() == "Passed")
        {
            report.outcome = Outcome::NeedsValidation(
                "Diagnostic output needs an expected-output comparison".to_owned(),
            );
        }
        return report;
    }

    run_rom_observed(&path, cycle_budget, |nes| {
        if relative_path == "read_joy3/test_buttons.nes" {
            // Source main's two read_joy calls: press the requested button,
            // then release it. Only controller input changes, never test RAM.
            match nes.cpu.registers.program_counter {
                0xE094 => {
                    let button = nes.bus.peek(0x1D);
                    nes.update_buttons(|state| *state = ButtonState::from_bits(button));
                }
                0xE0A0 => nes.update_buttons(|state| state.clear()),
                _ => {}
            }
        }
        observe(fixture.protocol, nes)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_legacy_fixtures_are_pinned_and_unique() {
        assert_eq!(FIXTURES.len(), 77);
        let mut names = std::collections::HashSet::new();
        for fixture in FIXTURES {
            assert!(names.insert(fixture.path));
            let path = format!(
                "{}/tests/fixtures/blargg/{}",
                env!("CARGO_MANIFEST_DIR"),
                fixture.path
            );
            let mut bytes = std::fs::read(path).unwrap();
            assert_eq!(fingerprint(&bytes), fixture.fingerprint, "{}", fixture.path);
            bytes[16] ^= 1;
            assert_ne!(fingerprint(&bytes), fixture.fingerprint);
        }
    }

    #[test]
    fn result_byte_requires_reporting_pc_and_nonzero_result() {
        let mut nes = NES::default();
        let protocol = Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        };
        for code in [0, 1, 7] {
            nes.bus.write(0xF8, code);
            nes.cpu.registers.program_counter = 0xE100;
            assert!(observe(protocol, &nes).is_none());
            nes.cpu.registers.program_counter = 0xE00B;
            let expected = match code {
                0 => Outcome::InvalidStatus(0),
                1 => Outcome::Passed,
                _ => Outcome::Failed(code),
            };
            assert_eq!(observe(protocol, &nes).unwrap().0, expected);
        }
    }

    #[test]
    fn observation_completion_is_never_a_pass() {
        let mut nes = NES::default();
        nes.cpu.registers.program_counter = 0xE100;
        let result = observe(
            Protocol::Observation {
                pc: 0xE100,
                reason: "audio measurement",
            },
            &nes,
        )
        .unwrap();
        assert!(matches!(result.0, Outcome::NeedsValidation(_)));
    }

    #[test]
    fn exit_codes_and_timing_branches_distinguish_pass_from_failure() {
        let mut nes = NES::default();
        nes.cpu.registers.program_counter = 0xE100;
        for code in [0, 1, 17] {
            nes.cpu.registers.accumulator = code;
            let result = observe(Protocol::Accumulator { pc: 0xE100 }, &nes).unwrap();
            assert_eq!(
                result.0,
                if code == 0 {
                    Outcome::Passed
                } else {
                    Outcome::Failed(code)
                }
            );
            let result = observe(
                Protocol::Measurement {
                    pc: 0xE100,
                    address: 0x1D,
                },
                &nes,
            )
            .unwrap();
            if code == 0 {
                assert!(matches!(result.0, Outcome::NeedsValidation(_)));
            } else {
                assert_eq!(result.0, Outcome::Failed(code));
            }
        }
        let protocol = Protocol::Timing {
            pass_pc: 0xE100,
            fail_pc: 0xE200,
        };
        assert_eq!(observe(protocol, &nes).unwrap().0, Outcome::Passed);
        nes.cpu.registers.program_counter = 0xE200;
        assert_eq!(observe(protocol, &nes).unwrap().0, Outcome::Failed(1));
        nes.cpu.registers.program_counter = 0xE300;
        assert!(observe(protocol, &nes).is_none());
    }

    #[test]
    fn crc_checks_output_instead_of_accepting_a_silent_exit() {
        let mut nes = NES::default();
        nes.cpu.registers.program_counter = 0xE100;
        nes.cpu.registers.accumulator = 0;
        let protocol = Protocol::Crc {
            pc: 0xE100,
            expected: &[0x159A7A8F, 0x5E3DF9C4],
        };
        for actual in [0x159A7A8Fu32, 0x5E3DF9C4, 0] {
            for (i, byte) in (!actual).to_le_bytes().into_iter().enumerate() {
                nes.bus.write(0x10 + i as u16, byte);
            }
            let expected = if actual == 0 {
                Outcome::Failed(1)
            } else {
                Outcome::Passed
            };
            assert_eq!(observe(protocol, &nes).unwrap().0, expected);
        }
    }
}
