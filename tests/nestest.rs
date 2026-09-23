use std::fmt;

use lynes::{NES, StepKind, cartridge::Cartridge};

const REFERENCE: &str = include_str!("fixtures/nestest/nestest.log");
const ROM_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/nestest/nestest.nes"
);
const EXPECTED_INSTRUCTIONS: usize = 8_991;
const RESET_CYCLES: usize = 7;
const MAX_CPU_CYCLES: usize = 100_000;

#[derive(Debug, PartialEq, Eq)]
struct CpuState {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    status: u8,
    sp: u8,
    cycles: usize,
}

impl CpuState {
    fn capture(nes: &NES, cycles: usize) -> Self {
        let registers = &nes.cpu.registers;
        Self {
            pc: registers.program_counter,
            a: registers.accumulator,
            x: registers.x,
            y: registers.y,
            status: registers.status.bits(),
            sp: registers.stack_pointer,
            cycles,
        }
    }

    fn parse(line: &str) -> Self {
        let field = |prefix: &str| {
            line.split_whitespace()
                .find_map(|token| token.strip_prefix(prefix))
                .unwrap_or_else(|| panic!("Missing {prefix} in nestest reference: {line}"))
        };
        let byte = |prefix| {
            u8::from_str_radix(field(prefix), 16)
                .unwrap_or_else(|_| panic!("Invalid {prefix} in nestest reference: {line}"))
        };

        Self {
            pc: u16::from_str_radix(line.split_whitespace().next().unwrap(), 16)
                .expect("Invalid PC in nestest reference"),
            a: byte("A:"),
            x: byte("X:"),
            y: byte("Y:"),
            status: byte("P:"),
            sp: byte("SP:"),
            cycles: field("CYC:")
                .parse()
                .expect("Invalid reference cycle count"),
        }
    }
}

impl fmt::Display for CpuState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc, self.a, self.x, self.y, self.status, self.sp, self.cycles
        )
    }
}

fn step_instruction(nes: &mut NES, reference_line: usize) -> usize {
    let result = nes.step();
    assert!(
        matches!(result.kind, StepKind::Instructrion { .. }),
        "Unexpected interrupt at nestest reference line {reference_line}"
    );
    result.cpu_cycles
}

#[test]
fn cpu_matches_nestest_reference() {
    let lines: Vec<_> = REFERENCE.lines().collect();
    assert_eq!(
        lines.len(),
        EXPECTED_INSTRUCTIONS,
        "nestest reference is incomplete or has changed"
    );
    let expected: Vec<_> = lines.iter().map(|line| CpuState::parse(line)).collect();

    let mut nes = NES::default();
    nes.insert_cart(Cartridge::load(ROM_PATH));
    nes.reset();

    nes.cpu.registers.program_counter = 0xC000;

    let mut cycles = RESET_CYCLES;
    for (index, expected_state) in expected.iter().enumerate() {
        let actual = CpuState::capture(&nes, cycles);
        if actual != *expected_state {
            let context = lines[index.saturating_sub(5)..index].join("\n");
            panic!(
                "nestest mismatch at instruction {} / reference line {}\n\
                 Expected: {expected_state}\n\
                 Actual:   {actual}\n\
                 Previous matched reference entries:\n{context}\n\
                 Current reference entry:\n{}",
                index + 1,
                index + 1,
                lines[index],
            );
        }

        let elapsed = step_instruction(&mut nes, index + 1);
        assert!(
            elapsed <= MAX_CPU_CYCLES - cycles,
            "nestest exceeded its CPU cycle budget at reference line {}",
            index + 1,
        );
        cycles += elapsed;
    }

    assert_eq!(
        CpuState::capture(&nes, cycles),
        CpuState {
            pc: 0x0001,
            a: 0x00,
            x: 0xFF,
            y: 0x15,
            status: 0x27,
            sp: 0xFF,
            cycles: 26_560,
        },
        "nestest did not return from its final instruction as expected"
    );

    assert_eq!(
        nes.cpu_read(0x0002),
        0,
        "nestest reported an error at $0002"
    );
    assert_eq!(
        nes.cpu_read(0x0003),
        0,
        "nestest reported an error at $0003"
    );
}
