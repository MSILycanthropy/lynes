use crate::{ppu::PPU, NES};

pub(crate) mod instructions;
pub(crate) mod registers;

#[derive(Debug)]
pub enum AddrMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

pub trait CPU {
    fn cpu_clock(&mut self) -> usize;
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn cpu_read_u16(&mut self, addr: u16) -> u16 {
        let low = self.cpu_read(addr);
        let high = self.cpu_read(addr + 1);

        u16::from_le_bytes([low, high])
    }
    fn cpu_write_u16(&mut self, addr: u16, data: u16) {
        let [low, high] = data.to_le_bytes();

        self.cpu_write(addr, low);
        self.cpu_write(addr + 1, high);
    }
    fn stack_push(&mut self, data: u8);
    fn stack_pop(&mut self) -> u8;
    fn stack_push_u16(&mut self, data: u16) {
        let [low, high] = data.to_le_bytes();

        self.stack_push(high);
        self.stack_push(low);
    }
    fn stack_pop_u16(&mut self) -> u16 {
        let low = self.stack_pop();
        let high = self.stack_pop();

        u16::from_le_bytes([low, high])
    }
    fn execute_instruction(&mut self, opcode: u8) -> (u16, usize);
}

impl CPU for NES {
    fn cpu_clock(&mut self) -> usize {
        if self.cpu_cycles == 0 {
            let opcode = self.cpu_read(self.cpu_registers.program_counter);

            self.cpu_registers.program_counter += 1;

            let old_program_counter = self.cpu_registers.program_counter;

            let (length, cycles) = self.execute_instruction(opcode);

            if old_program_counter == self.cpu_registers.program_counter {
                self.cpu_registers.program_counter += length;
            }

            self.cpu_cycles += cycles;
            self.clock_count += cycles;

            return cycles;
        }

        self.cpu_cycles -= 1;
        return 0;
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0b00000111_11111111;

                self.cpu_ram[mirrored_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("attempted to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu_read_status(),
            0x2004 => self.ppu_read_oam_data(),
            0x2007 => self.ppu_read(),
            0x4000..=0x4017 => {
                // panic!("APU and I/O registers are not implemented yet!")
                0
            }
            0x2008..=0x3FFF => {
                let mirrored_down_address = addr & 0b00100000_00000111;
                self.cpu_read(mirrored_down_address)
            }
            0x8000..=0xFFFF => {
                let mut addr = addr - 0x8000;

                if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    addr %= 0x4000;
                }

                self.prg_rom[addr as usize]
            }
            _ => {
                panic!("Invalid CPU read address: {:#06X}", addr);
            }
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0b00000111_11111111;

                self.cpu_ram[mirrored_addr as usize] = data;
            }
            0x2000 => self.ppu_write_control(data),
            0x2001 => self.ppu_write_mask(data),
            0x2002 => panic!("attempt to write to ppu status register.. dont do that"),
            0x2003 => self.ppu_write_oam_address(data),
            0x2004 => self.ppu_write_oam_data(data),
            0x2005 => self.ppu_write_scroll(data),
            0x2006 => self.ppu_write_address(data),
            0x2007 => self.ppu_write(data),
            0x2008..=0x3FFF => {
                let mirrored_down_address = addr & 0b00100000_00000111;
                self.cpu_write(mirrored_down_address, data);
            }
            0x4000..=0x4017 => {
                // panic!("APU and I/O registers are not implemented yet!")
            }
            0x4018..=0x401F => {
                // panic!("APU and I/O functionality that is normally disabled")
            }
            0x4020..=0x7999 => {
                // panic!("PRG RAM and mapper registers")
            }
            0x8000..=0xFFFF => {
                // panic!("Cannot write to PRG ROM!")
            }
            _ => {
                panic!("Invalid CPU write address: {:#06X}", addr);
            }
        }
    }

    fn stack_push(&mut self, data: u8) {
        self.cpu_write(0x0100 + self.cpu_registers.stack_pointer as u16, data);
        self.cpu_registers.stack_pointer = self.cpu_registers.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.cpu_registers.stack_pointer = self.cpu_registers.stack_pointer.wrapping_add(1);
        self.cpu_read(0x0100 + self.cpu_registers.stack_pointer as u16)
    }

    fn execute_instruction(&mut self, opcode: u8) -> (u16, usize) {
        let instruction = &instructions::INSTRUCTIONS_TABLE[opcode as usize];

        let cycles = instruction.execute(self);

        (instruction.size(), cycles)
    }
}
