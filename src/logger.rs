use colored::Colorize;

use crate::{
    NES,
    cpu::{self, AddrMode},
};

const ILLEGAL_NOPS: [&'static str; 2] = ["DOP", "TOP"];

pub fn log(nes: &NES) {
    println!(
        "{: <6}{: <10}{: <32}{}",
        program_counter_log(nes.cpu.registers.program_counter).blue(),
        instruction_log(nes).cyan(),
        assembly_log(nes).yellow(),
        cpu_registers_log(nes).magenta()
    );
}

fn program_counter_log(program_counter: u16) -> String {
    format!("{:04X}", program_counter)
}

fn instruction_log(nes: &NES) -> String {
    let opcode = nes.bus.peek(nes.cpu.registers.program_counter);
    let instruction = &cpu::instructions::INSTRUCTIONS_TABLE[opcode as usize];

    let log = match instruction.len {
        1 => format!("{:02X}", opcode),
        2 => {
            let operand = nes
                .bus
                .peek(nes.cpu.registers.program_counter.wrapping_add(1));
            format!("{:02X} {:02X}", opcode, operand)
        }
        3 => {
            let operand1 = nes
                .bus
                .peek(nes.cpu.registers.program_counter.wrapping_add(1));
            let operand2 = nes
                .bus
                .peek(nes.cpu.registers.program_counter.wrapping_add(2));
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
    let opcode = nes.bus.peek(nes.cpu.registers.program_counter);
    let instruction = &cpu::instructions::INSTRUCTIONS_TABLE[opcode as usize];

    let instruction_name = if ILLEGAL_NOPS.contains(&instruction.name) {
        "NOP"
    } else {
        instruction.name
    };

    let program_counter = nes.cpu.registers.program_counter;
    let (mem_addr, stored) = match instruction.mode {
        AddrMode::Immediate | AddrMode::Accumulator | AddrMode::Implied | AddrMode::Relative => {
            (0, 0)
        }
        _ => {
            let addr =
                peek_operand_address(nes, program_counter.wrapping_add(1), &instruction.mode);
            let stored = nes.bus.peek(addr);

            (addr, stored)
        }
    };

    let addr = nes.bus.peek(program_counter.wrapping_add(1));
    let addr_16 = nes.bus.peek_u16(program_counter.wrapping_add(1));

    match instruction.mode {
        AddrMode::Accumulator => format!("{} A", instruction_name),
        AddrMode::Implied => format!("{}", instruction_name),
        AddrMode::Relative => {
            let jump_addr = program_counter
                .wrapping_add(2)
                .wrapping_add(addr_16 as i8 as u16);

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
                let low = nes.bus.peek(addr_16);
                let high = nes.bus.peek(addr_16 & 0xFF00);

                u16::from_le_bytes([low, high])
            } else {
                nes.bus.peek_u16(addr_16)
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
                addr.wrapping_add(nes.cpu.registers.x),
                mem_addr,
                stored
            )
        }
        AddrMode::IndirectY => {
            format!(
                "{} (${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                instruction_name,
                addr,
                mem_addr.wrapping_sub(nes.cpu.registers.y as u16),
                mem_addr,
                stored
            )
        }
    }
}

// Debug decoding uses only peeks; executing CPU addressing helpers performs real reads.
fn peek_operand_address(nes: &NES, address: u16, mode: &AddrMode) -> u16 {
    let bus = &nes.bus;
    let registers = &nes.cpu.registers;

    match mode {
        AddrMode::ZeroPage => u16::from(bus.peek(address)),
        AddrMode::ZeroPageX => u16::from(bus.peek(address).wrapping_add(registers.x)),
        AddrMode::ZeroPageY => u16::from(bus.peek(address).wrapping_add(registers.y)),
        AddrMode::Absolute => bus.peek_u16(address),
        AddrMode::AbsoluteX => bus.peek_u16(address).wrapping_add(u16::from(registers.x)),
        AddrMode::AbsoluteY => bus.peek_u16(address).wrapping_add(u16::from(registers.y)),
        AddrMode::Indirect => {
            let pointer = bus.peek_u16(address);
            // JMP's high-byte fetch wraps within the pointer's page.
            let high_address = (pointer & 0xFF00) | (pointer.wrapping_add(1) & 0x00FF);
            u16::from_le_bytes([bus.peek(pointer), bus.peek(high_address)])
        }
        AddrMode::IndirectX => {
            let pointer = bus.peek(address).wrapping_add(registers.x);
            u16::from_le_bytes([
                bus.peek(u16::from(pointer)),
                bus.peek(u16::from(pointer.wrapping_add(1))),
            ])
        }
        AddrMode::IndirectY => {
            let pointer = bus.peek(address);
            let base = u16::from_le_bytes([
                bus.peek(u16::from(pointer)),
                bus.peek(u16::from(pointer.wrapping_add(1))),
            ]);
            base.wrapping_add(u16::from(registers.y))
        }
        _ => unreachable!("addressing mode has no memory operand to display"),
    }
}

fn cpu_registers_log(nes: &NES) -> String {
    format!(
        "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
        nes.cpu.registers.accumulator,
        nes.cpu.registers.x,
        nes.cpu.registers.y,
        nes.cpu.registers.status.bits(),
        nes.cpu.registers.stack_pointer,
        nes.bus.ppu.scanline,
        nes.bus.ppu.dot,
        nes.total_cpu_cycles,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_io_operands_preserves_the_next_real_read() {
        let mut nes = NES::default();
        nes.cpu.registers.program_counter = 0;
        nes.bus.write(0, 0xAD); // LDA absolute
        nes.bus.ppu.registers.status.set_vblank_started(true);
        nes.bus.ppu.read_buffer = 0x45;
        nes.bus.ppu.write_address(0x20);
        nes.bus.ppu.write_address(0x00);
        nes.bus.controller.button_state.set_a(true);

        // The $40 operand byte written below supplies the controller's open-bus bits.
        for (address, expected) in [(0x2002u16, 0x80), (0x2007, 0x45), (0x4016, 0x41)] {
            let [low, high] = address.to_le_bytes();
            nes.bus.write(1, low);
            nes.bus.write(2, high);
            assert_eq!(instruction_log(&nes), format!("AD {low:02X} {high:02X}"));
            let assembly = format!("LDA ${address:04X} = {expected:02X}");
            assert_eq!(assembly_log(&nes), assembly);
            assert_eq!(assembly_log(&nes), assembly);
            assert_eq!(nes.bus.read(address), expected);
        }
    }

    #[test]
    fn debug_address_decoding_wraps_indirect_pointers() {
        let mut nes = NES::default();
        nes.bus.write(1, 0xFF);
        nes.bus.write(2, 0x02);
        nes.bus.write(0x02FF, 0x34);
        nes.bus.write(0x0200, 0x12);
        assert_eq!(peek_operand_address(&nes, 1, &AddrMode::Indirect), 0x1234);

        nes.bus.write(0x00FF, 0x78);
        nes.bus.write(0x0000, 0x56);
        nes.cpu.registers.x = 0;
        nes.cpu.registers.y = 1;
        assert_eq!(peek_operand_address(&nes, 1, &AddrMode::IndirectX), 0x5678);
        assert_eq!(peek_operand_address(&nes, 1, &AddrMode::IndirectY), 0x5679);
    }
}
