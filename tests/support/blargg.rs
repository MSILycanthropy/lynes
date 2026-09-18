use std::{
    any::Any,
    fmt,
    panic::{AssertUnwindSafe, catch_unwind},
};

use lynes::{
    NES,
    cartridge::{Cartridge, ScreenMirroring},
    cpu::CPU,
};

pub const DEFAULT_CYCLE_BUDGET: usize = 30_000_000;
// ceil(NTSC CPU frequency * 100 ms). This is emulated time, not a host sleep.
const RESET_DELAY_CYCLES: usize = 178_978;
const RESET_CYCLES: usize = 7;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed(u8),
    TimedOut,
    UnsupportedCartridge(String),
    LoadError(String),
    EmulatorPanicked(String),
    InvalidStatus(u8),
    NoProgress,
}

#[derive(Debug)]
pub struct Report {
    pub outcome: Outcome,
    pub cpu_cycles: usize,
    pub resets: usize,
    pub pc: u16,
    pub status: Option<u8>,
    pub text: String,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{:?}; CPU cycles: {}; resets: {}; PC: ${:04X}; status: {:?}",
            self.outcome, self.cpu_cycles, self.resets, self.pc, self.status
        )?;
        write!(f, "{}", self.text)
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        "non-string panic".to_owned()
    }
}

fn status(nes: &mut NES) -> Option<u8> {
    let signature = [
        nes.cpu_read(0x6001),
        nes.cpu_read(0x6002),
        nes.cpu_read(0x6003),
    ];
    (signature == [0xDE, 0xB0, 0x61]).then(|| nes.cpu_read(0x6000))
}

fn diagnostic_text(nes: &mut NES) -> String {
    let mut bytes = Vec::new();
    for address in 0x6004..=0x7FFF {
        let byte = nes.cpu_read(address);
        if byte == 0 {
            return String::from_utf8_lossy(&bytes).into_owned();
        }
        bytes.push(byte);
    }
    format!(
        "{}\n[unterminated diagnostic text at end of PRG RAM]",
        String::from_utf8_lossy(&bytes)
    )
}

fn report(nes: &mut NES, outcome: Outcome, cpu_cycles: usize, resets: usize) -> Report {
    let status = status(nes);
    let text = if status.is_some() {
        diagnostic_text(nes)
    } else {
        String::new()
    };
    Report {
        outcome,
        cpu_cycles,
        resets,
        pc: nes.cpu_registers.program_counter,
        status,
        text,
    }
}

pub fn run_rom(path: &str, cycle_budget: usize) -> Report {
    match catch_unwind(|| Cartridge::load(path)) {
        Ok(cart) => run_cartridge(cart, cycle_budget),
        Err(error) => report(
            &mut NES::default(),
            Outcome::LoadError(panic_message(error)),
            0,
            0,
        ),
    }
}

fn run_cartridge(cart: Cartridge, cycle_budget: usize) -> Report {
    // The emulator does not dispatch cartridge accesses through mappers yet.
    // Reject unsupported layouts rather than silently testing the wrong mapping.
    let unsupported = if cart.mapper != 0 {
        Some(format!(
            "mapper {} (only mapper 0 is supported by this runner)",
            cart.mapper
        ))
    } else if cart.screen_mirroring == ScreenMirroring::FourScreen {
        Some("four-screen mirroring".to_owned())
    } else if !matches!(cart.prg_rom.len(), 16_384 | 32_768) || cart.chr_rom.len() != 8_192 {
        Some("requires 16/32 KiB PRG ROM and 8 KiB CHR ROM; CHR RAM is not implemented".to_owned())
    } else {
        None
    };
    let mut nes = NES::default();
    if let Some(reason) = unsupported {
        return report(&mut nes, Outcome::UnsupportedCartridge(reason), 0, 0);
    }
    nes.insert_cart(cart);
    nes.reset();

    let mut cycles = RESET_CYCLES;
    let mut resets = 0;
    let mut reset_requested_at = None;
    let mut waiting_for_reset_ack = false;

    loop {
        match status(&mut nes) {
            Some(0) => return report(&mut nes, Outcome::Passed, cycles, resets),
            Some(code @ 1..=0x7F) => {
                return report(&mut nes, Outcome::Failed(code), cycles, resets);
            }
            Some(0x80) => {
                reset_requested_at = None;
                waiting_for_reset_ack = false;
            }
            Some(0x81) => {
                if !waiting_for_reset_ack {
                    let requested_at = *reset_requested_at.get_or_insert(cycles);
                    if cycles - requested_at >= RESET_DELAY_CYCLES
                        && cycles + RESET_CYCLES <= cycle_budget
                    {
                        nes.reset();
                        cycles += RESET_CYCLES;
                        resets += 1;
                        waiting_for_reset_ack = true;
                        reset_requested_at = None;
                    }
                }
            }
            Some(code) => return report(&mut nes, Outcome::InvalidStatus(code), cycles, resets),
            None => {}
        }

        // A CPU action may cross the budget by its own cycle cost; never start
        // another action after reaching the limit. Resets do not renew the budget.
        if cycles >= cycle_budget {
            return report(&mut nes, Outcome::TimedOut, cycles, resets);
        }
        match catch_unwind(AssertUnwindSafe(|| nes.step())) {
            Ok(step) if step.cpu_cycles == 0 => {
                return report(&mut nes, Outcome::NoProgress, cycles, resets);
            }
            Ok(step) => cycles += step.cpu_cycles,
            Err(error) => {
                return report(
                    &mut nes,
                    Outcome::EmulatorPanicked(panic_message(error)),
                    cycles,
                    resets,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cart(program: &[u8]) -> Cartridge {
        let mut prg_rom = vec![0; 32_768];
        prg_rom[..program.len()].copy_from_slice(program);
        prg_rom[0x7FFC..0x7FFE].copy_from_slice(&0x8000u16.to_le_bytes());
        Cartridge {
            prg_rom,
            chr_rom: vec![0; 8192],
            mapper: 0,
            screen_mirroring: ScreenMirroring::Horizontal,
        }
    }

    fn write(program: &mut Vec<u8>, address: u16, value: u8) {
        // LDA #value; STA address
        program.extend([0xA9, value, 0x8D, address as u8, (address >> 8) as u8]);
    }

    fn initialize(program: &mut Vec<u8>) {
        for (address, value) in [
            (0x6000, 0x80),
            (0x6001, 0xDE),
            (0x6002, 0xB0),
            (0x6003, 0x61),
        ] {
            write(program, address, value);
        }
    }

    fn loop_forever(program: &mut Vec<u8>) {
        let pc = 0x8000 + program.len() as u16;
        program.extend([0x4C, pc as u8, (pc >> 8) as u8]);
    }

    #[test]
    fn uninitialized_zero_ram_is_not_a_pass() {
        let result = run_cartridge(cart(&[0x4C, 0x00, 0x80]), 100);
        assert_eq!(result.outcome, Outcome::TimedOut);
        assert_eq!(result.status, None);
    }

    #[test]
    fn incomplete_signature_is_not_a_result() {
        let mut program = Vec::new();
        write(&mut program, 0x6001, 0xDE);
        write(&mut program, 0x6002, 0xB0);
        loop_forever(&mut program);
        assert_eq!(
            run_cartridge(cart(&program), 100).outcome,
            Outcome::TimedOut
        );
    }

    #[test]
    fn returns_pass_failure_and_diagnostics() {
        for code in [0, 3] {
            let mut program = Vec::new();
            initialize(&mut program);
            write(&mut program, 0x6004, b'O');
            write(&mut program, 0x6005, b'K');
            write(&mut program, 0x6000, code);
            loop_forever(&mut program);
            let result = run_cartridge(cart(&program), 1000);
            assert_eq!(
                result.outcome,
                if code == 0 {
                    Outcome::Passed
                } else {
                    Outcome::Failed(code)
                }
            );
            assert_eq!(result.text, "OK");
        }
    }

    #[test]
    fn running_rom_times_out_and_reserved_status_is_rejected() {
        for code in [0x80, 0x82] {
            let mut program = Vec::new();
            initialize(&mut program);
            write(&mut program, 0x6000, code);
            loop_forever(&mut program);
            let result = run_cartridge(cart(&program), 1000);
            assert_eq!(
                result.outcome,
                if code == 0x80 {
                    Outcome::TimedOut
                } else {
                    Outcome::InvalidStatus(code)
                }
            );
        }
    }

    #[test]
    fn unsupported_mapper_is_reported_before_execution() {
        let mut cart = cart(&[]);
        cart.mapper = 1;
        assert!(matches!(
            run_cartridge(cart, 100).outcome,
            Outcome::UnsupportedCartridge(_)
        ));
    }

    #[test]
    fn emulator_panic_is_distinct_from_rom_failure() {
        let result = run_cartridge(cart(&[0x8D, 0x00, 0x80]), 100);
        assert!(matches!(result.outcome, Outcome::EmulatorPanicked(_)));
    }

    #[test]
    fn diagnostic_read_stops_at_end_of_prg_ram() {
        let mut nes = NES::default();
        for address in 0x6004..=0x7FFF {
            nes.cpu_write(address, b'x');
        }
        let text = diagnostic_text(&mut nes);
        assert!(text.starts_with(&"x".repeat(8192 - 4)));
        assert!(text.contains("unterminated diagnostic"));
    }

    #[test]
    fn prg_ram_addresses_are_distinct_and_survive_reset() {
        let mut nes = NES::default();
        nes.insert_cart(cart(&[]));
        let values = [(0x6000, 0x12), (0x6123, 0x34), (0x7FFF, 0x56)];
        for (address, value) in values {
            nes.cpu_write(address, value);
        }
        nes.reset();
        for (address, value) in values {
            assert_eq!(nes.cpu_read(address), value);
        }
    }

    #[test]
    fn reset_request_waits_and_preserves_ram() {
        // LDA $6010; BNE finished. The RAM marker selects the post-reset path.
        let mut program = vec![0xAD, 0x10, 0x60, 0xD0, 0];
        initialize(&mut program);
        write(&mut program, 0x6010, 1);
        write(&mut program, 0x6000, 0x81);
        loop_forever(&mut program);
        program[4] = (program.len() - 5) as u8;
        write(&mut program, 0x6000, 0);
        loop_forever(&mut program);
        let result = run_cartridge(cart(&program), 200_000);
        assert_eq!(result.outcome, Outcome::Passed, "{result}");
        assert_eq!(result.resets, 1);
        assert!(result.cpu_cycles >= RESET_DELAY_CYCLES + 2 * RESET_CYCLES);
    }

    #[test]
    fn unchanged_reset_request_does_not_reset_forever() {
        let mut program = vec![0xAD, 0x10, 0x60, 0xD0, 0];
        initialize(&mut program);
        write(&mut program, 0x6010, 1);
        write(&mut program, 0x6000, 0x81);
        program[4] = (program.len() - 5) as u8;
        loop_forever(&mut program);
        let result = run_cartridge(cart(&program), 400_000);
        assert_eq!(result.outcome, Outcome::TimedOut, "{result}");
        assert_eq!(result.resets, 1);
    }
}
