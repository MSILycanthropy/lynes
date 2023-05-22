use colored::Colorize;

use crate::{
    cpu::{self, AddrMode, CPU},
    NES,
};

const ILLEGAL_NOPS: [&'static str; 2] = ["DOP", "TOP"];

pub fn log(nes: &NES) {
    println!(
        "{: <6}{: <10}{: <32}{}",
        program_counter_log(nes.cpu_registers.program_counter).blue(),
        instruction_log(nes).cyan(),
        assembly_log(nes).yellow(),
        cpu_registers_log(nes).magenta()
    );
}

fn program_counter_log(program_counter: u16) -> String {
    format!("{:04X}", program_counter)
}

fn instruction_log(nes: &NES) -> String {
    let opcode = nes.cpu_read(nes.cpu_registers.program_counter);
    let instruction = match &cpu::instructions::INSTRUCTIONS_TABLE[opcode as usize] {
        Some(instruction) => instruction,
        None => return format!("{:02X}", opcode)
    };

    let log = match instruction.len {
        1 => format!("{:02X}", opcode),
        2 => {
            let operand = nes.cpu_read(nes.cpu_registers.program_counter + 1);
            format!("{:02X} {:02X}", opcode, operand)
        }
        3 => {
            let operand1 = nes.cpu_read(nes.cpu_registers.program_counter + 1);
            let operand2 = nes.cpu_read(nes.cpu_registers.program_counter + 2);
            format!("{:02X} {:02X} {:02X}", opcode, operand1, operand2)
        }
        _ => unreachable!(),
    };

    if instruction.legal {
        log
    } else {
        format!("{: <9}*", log)
    }
}

fn assembly_log(nes: &NES) -> String {
    let opcode = nes.cpu_read(nes.cpu_registers.program_counter);
    let instruction = match &cpu::instructions::INSTRUCTIONS_TABLE[opcode as usize] {
        Some(instruction) => instruction,
        None => return "???".to_string(),
    };

    let instruction_name = if ILLEGAL_NOPS.contains(&instruction.name) {
        "NOP"
    } else {
        instruction.name
    };

    let program_counter = nes.cpu_registers.program_counter;
    let (mem_addr, stored) = match instruction.mode {
        AddrMode::Immediate | AddrMode::Accumulator | AddrMode::Implied => (0, 0),
        _ => {
            let (addr, _) = nes.get_absolute_address(program_counter + 1, &instruction.mode);
            let stored = nes.cpu_read(addr);

            (addr, stored)
        }
    };

    let addr = nes.cpu_read(program_counter + 1);
    let addr_16 = nes.cpu_read_u16(program_counter + 1);

    match instruction.mode {
        AddrMode::Accumulator => format!("{} A", instruction_name),
        AddrMode::Implied => format!("{}", instruction_name),
        AddrMode::Relative => {
            let jump_addr = (program_counter + 2).wrapping_add(addr_16 as i8 as u16);

            format!("{} ${:02X}", instruction_name, jump_addr)
        }
        AddrMode::Immediate => {
            format!("{} #${:02X}", instruction_name, addr)
        }
        AddrMode::ZeroPage => {
            format!("{} ${:02X} = {:02X}", instruction_name, mem_addr, stored)
        }
        AddrMode::Absolute => {
            if instruction_name == "JMP" || instruction_name == "JSR" {
                return format!("{} ${:04X}", instruction_name, mem_addr);
            }

            format!("{} ${:04X} = {:02X}", instruction_name, mem_addr, stored)
        }
        AddrMode::Indirect => {
            let jump_addr = if addr_16 & 0x00FF == 0x00FF {
                let low = nes.cpu_read(addr_16);
                let high = nes.cpu_read(addr_16 & 0xFF00);

                u16::from_le_bytes([low, high])
            } else {
                nes.cpu_read_u16(addr_16)
            };

            format!(
                "{} (${:04X}) = {:04X}",
                instruction_name, addr_16, jump_addr
            )
        }
        AddrMode::ZeroPageX => {
            format!(
                "{} ${:02X},X @ {:02X} = {:02X}",
                instruction_name, addr, mem_addr, stored
            )
        }
        AddrMode::ZeroPageY => {
            format!(
                "{} ${:02X},Y @ {:02X} = {:02X}",
                instruction_name, addr, mem_addr, stored
            )
        }
        AddrMode::AbsoluteX => {
            format!(
                "{} ${:04X},X @ {:04X} = {:02X}",
                instruction_name, addr_16, mem_addr, stored
            )
        }
        AddrMode::AbsoluteY => {
            format!(
                "{} ${:04X},Y @ {:04X} = {:02X}",
                instruction_name, addr_16, mem_addr, stored
            )
        }
        AddrMode::IndirectX => {
            format!(
                "{} (${:02X},X) @ {:02X} = {:04X} = {:02X}",
                instruction_name,
                addr,
                addr.wrapping_add(nes.cpu_registers.x),
                mem_addr,
                stored
            )
        }
        AddrMode::IndirectY => {
            format!(
                "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                instruction_name,
                addr,
                mem_addr.wrapping_sub(nes.cpu_registers.y as u16),
                mem_addr,
                stored
            )
        }
    }
}

fn cpu_registers_log(nes: &NES) -> String {
    format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
        nes.cpu_registers.accumulator,
        nes.cpu_registers.x,
        nes.cpu_registers.y,
        nes.cpu_registers.status.bits(),
        nes.cpu_registers.stack_pointer,
        nes.clock_count,
    )
}
