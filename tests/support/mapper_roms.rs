use lynes::{NES, cartridge::Cartridge};

pub const CYCLE_BUDGET: usize = 5_000_000;

pub fn fixture(relative: &str) -> String {
    format!(
        "{}/tests/fixtures/mappers/{relative}",
        env!("CARGO_MANIFEST_DIR")
    )
}

// Used only with source-verified completion addresses in pinned ROMs. Returning
// from this function means execution finished, not that any assertions passed.
pub fn run_until(path: &str, completion: u16, budget: usize) -> Result<NES, String> {
    let mut nes = NES::default();
    nes.insert_cart(Cartridge::load(path));
    nes.reset();
    let mut cycles = 7;
    while nes.cpu.registers.program_counter != completion {
        if cycles >= budget {
            return Err(format!(
                "{path}: timed out after {cycles} CPU cycles; PC=${:04X}",
                nes.cpu.registers.program_counter
            ));
        }
        let step = nes.step();
        if step.cpu_cycles == 0 {
            return Err(format!("{path}: CPU made no progress"));
        }
        cycles += step.cpu_cycles;
    }
    Ok(nes)
}

pub fn check_bytes(path: &str, offset: usize, expected: &[u8]) {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        bytes.get(offset..offset + expected.len()),
        Some(expected),
        "{path}: pinned result adapter no longer matches the ROM"
    );
}

#[derive(Clone, Copy)]
pub enum Board {
    Uxrom,
    Cnrom,
}

impl Board {
    fn mapper(self) -> u8 {
        match self {
            Self::Uxrom => 2,
            Self::Cnrom => 3,
        }
    }

    fn completion_pc(self) -> u16 {
        // The final `jmp :-` in each pinned source's `testing` routine.
        // UxROM switches bank $0F into $8000 before reaching this loop.
        match self {
            Self::Uxrom => 0x8352,
            Self::Cnrom => 0x8365,
        }
    }
}

#[derive(Debug)]
pub struct Results {
    pub banks: u8,
    pub bus_reads: [u8; 4],
    pub bus_kind: u8,
    pub detected_submapper: u8,
    pub prg_ram_present: Option<u8>,
}

pub fn run(board: Board, submapper: u8, budget: usize) -> Result<Results, String> {
    let mapper = board.mapper();
    let path = format!(
        "{}/tests/fixtures/mappers/{mapper}_test/{mapper}_test_{submapper}.nes",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(&path).map_err(|error| format!("{path}: {error}"))?;
    let completion = board.completion_pc();
    let loop_bytes = [0x4C, completion as u8, (completion >> 8) as u8];
    let offset = 16 + usize::from(completion - 0x8000);
    if bytes.get(offset..offset + 3) != Some(loop_bytes.as_slice()) {
        return Err(format!("{path}: pinned completion loop changed"));
    }

    let mut nes = NES::default();
    nes.insert_cart(Cartridge::load(&path));
    nes.reset();
    let mut cycles = 7;
    loop {
        // Never interpret cleared result RAM as a completed test. These ROMs
        // have no blargg signature: completion is specific to the pinned build.
        if nes.cpu.registers.program_counter == completion {
            let raw: [u8; 9] = std::array::from_fn(|i| nes.bus.peek(0x0300 + i as u16));
            let bank_index = match board {
                Board::Uxrom => 1,
                Board::Cnrom => 2,
            };
            return Ok(Results {
                banks: raw[bank_index],
                bus_reads: raw[bank_index + 1..bank_index + 5].try_into().unwrap(),
                bus_kind: raw[bank_index + 5],
                detected_submapper: raw[bank_index + 6],
                prg_ram_present: matches!(board, Board::Cnrom).then_some(raw[1]),
            });
        }
        if cycles >= budget {
            return Err(format!(
                "{path}: timed out after {cycles} CPU cycles; PC=${:04X}; results={:02X?}",
                nes.cpu.registers.program_counter,
                (0x0300..=0x0308)
                    .map(|addr| nes.bus.peek(addr))
                    .collect::<Vec<_>>()
            ));
        }
        let step = nes.step();
        if step.cpu_cycles == 0 {
            return Err(format!("{path}: CPU made no progress"));
        }
        cycles += step.cpu_cycles;
    }
}

pub fn assert_banking_and_bus(board: Board, submapper: u8, results: &Results) {
    let banks = match board {
        Board::Uxrom => 16,
        Board::Cnrom => 4,
    };
    assert_eq!(results.banks, banks, "{results:#?}");
    // Submapper zero leaves conflict behavior unspecified. Accept either known
    // behavior, but require the raw observations and reported detection to agree.
    let expected_kind = match submapper {
        0 => results.bus_kind,
        1 => 0,
        2 => 1,
        _ => panic!("unsupported test submapper"),
    };
    let expected_reads = match expected_kind {
        0 => [banks - 1, 0, banks - 1, banks - 1],
        1 => [0, 0, 1, 2],
        _ => panic!("unrecognized bus-conflict behavior: {results:#?}"),
    };
    assert_eq!(results.bus_reads, expected_reads, "{results:#?}");
    assert_eq!(results.bus_kind, expected_kind, "{results:#?}");
    assert_eq!(
        results.detected_submapper,
        expected_kind + 1,
        "{results:#?}"
    );
}
