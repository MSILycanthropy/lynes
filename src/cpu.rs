use crate::{
    Interrupt, NES,
    cpu::registers::CpuRegisters,
    ppu::{PPU, bus::PpuBus},
};

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

#[derive(Default)]
struct InterruptState {
    previous_nmi: bool,
    nmi_pending: bool,
    irq_asserted: bool,
}

impl InterruptState {
    fn sample_nmi(&mut self, asserted: bool) {
        if asserted && !self.previous_nmi {
            self.nmi_pending = true;
        }

        self.previous_nmi = asserted;
    }

    fn take(&mut self, interrupt_disable: bool) -> Option<Interrupt> {
        if self.nmi_pending {
            self.nmi_pending = false;
            return Some(Interrupt::NMI);
        }

        if self.irq_asserted && !interrupt_disable {
            return Some(Interrupt::IRQ);
        }

        None
    }
}

#[derive(Default)]
pub struct Cpu {
    pub registers: CpuRegisters,
    interrupt_state: InterruptState,
}

impl Cpu {
    pub(crate) fn sample_nmi_input(&mut self, nmi_asserted: bool) {
        self.interrupt_state.sample_nmi(nmi_asserted);
    }

    pub(crate) fn take_interrupt(&mut self) -> Option<Interrupt> {
        self.interrupt_state
            .take(self.registers.status.interrupt_disable())
    }
}

pub trait CPU {
    fn enter_interrupt(&mut self, interrupt: &Interrupt) -> usize;
    fn execute_next_instruction(&mut self) -> (u16, u8, usize);

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
    fn enter_interrupt(&mut self, interrupt: &Interrupt) -> usize {
        self.stack_push_u16(self.cpu.registers.program_counter);

        let mut status = self.cpu.registers.status.clone();
        status.set_b(0b10);

        self.stack_push(status.bits());

        self.cpu.registers.status.set_interrupt_disable(true);
        self.cpu.registers.program_counter = self.cpu_read_u16(interrupt.address());

        7
    }

    fn execute_next_instruction(&mut self) -> (u16, u8, usize) {
        let pc = self.cpu.registers.program_counter;
        let opcode = self.cpu_read(pc);
        self.cpu.registers.program_counter = pc.wrapping_add(1);

        let old_program_counter = self.cpu.registers.program_counter;

        let (length, cycles) = self.execute_instruction(opcode);

        if old_program_counter == self.cpu.registers.program_counter {
            self.cpu.registers.program_counter =
                self.cpu.registers.program_counter.wrapping_add(length);
        }

        (pc, opcode, cycles)
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0b00000111_11111111;

                self.bus.ram[mirrored_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("attempted to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => {
                let status = self.bus.ppu.read_status();
                self.sample_interrupt_line();
                status
            }
            0x2004 => self.bus.ppu.read_oam_data(),
            0x2007 => {
                let mut bus = PpuBus {
                    cartridge: &mut self.bus.cartridge,
                    ciram: &mut self.bus.ciram,
                };

                self.bus.ppu.read_data(&mut bus)
            }
            0x4000..=0x4015 => {
                // panic!("APU and I/O registers are not implemented yet!")
                0
            }
            0x4016 => self.bus.controller.read(),
            0x4017 => 0,
            0x2008..=0x3FFF => {
                let mirrored_down_address = addr & 0b00100000_00000111;
                self.cpu_read(mirrored_down_address)
            }
            0x4020..=0xFFFF => self
                .bus
                .cartridge
                .cpu_read(addr)
                .unwrap_or_else(|| panic!("Invalid CPU read address: {:#06X}", addr)),
            _ => {
                panic!("Invalid CPU read address: {:#06X}", addr);
            }
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mirrored_addr = addr & 0b00000111_11111111;

                self.bus.ram[mirrored_addr as usize] = data;
            }
            0x2000 => {
                self.bus.ppu.write_control(data);
                self.sample_interrupt_line();
            }
            0x2001 => self.bus.ppu.write_mask(data),
            0x2002 => {} // Writes dont change PPUSTATUS, but we do have tests that.. well test that.
            0x2003 => self.bus.ppu.write_oam_address(data),
            0x2004 => self.bus.ppu.write_oam_data(data),
            0x2005 => self.bus.ppu.write_scroll(data),
            0x2006 => self.bus.ppu.write_address(data),
            0x2007 => {
                let mut bus = PpuBus {
                    cartridge: &mut self.bus.cartridge,
                    ciram: &mut self.bus.ciram,
                };

                self.bus.ppu.write_data(&mut bus, data)
            }
            0x2008..=0x3FFF => {
                let mirrored_down_address = addr & 0b00100000_00000111;
                self.cpu_write(mirrored_down_address, data);
            }
            0x4014 => {
                let mut buffer = [0u8; 256];
                let high = (data as u16) << 8;

                for i in 0..256u16 {
                    buffer[i as usize] = self.cpu_read(high + i);
                }

                self.ppu_write_oam_dma(&buffer)
            }
            0x4000..=0x4015 => {
                // panic!("APU and I/O registers are not implemented yet!")
            }
            0x4016 => self.bus.controller.write(data),
            0x4017 => {
                // ignore controller 2
            }
            0x4018..=0x401F => {
                // panic!("APU and I/O functionality that is normally disabled")
            }
            0x4020..=0xFFFF => self.bus.cartridge.cpu_write(addr, data),
        }
    }

    fn stack_push(&mut self, data: u8) {
        self.cpu_write(0x0100 + self.cpu.registers.stack_pointer as u16, data);
        self.cpu.registers.stack_pointer = self.cpu.registers.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.cpu.registers.stack_pointer = self.cpu.registers.stack_pointer.wrapping_add(1);
        self.cpu_read(0x0100 + self.cpu.registers.stack_pointer as u16)
    }

    fn execute_instruction(&mut self, opcode: u8) -> (u16, usize) {
        let instruction = &instructions::INSTRUCTIONS_TABLE[opcode as usize];

        let cycles = instruction.execute(self);

        (instruction.size(), cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nmi_latches_edges_until_consumed_even_when_interrupts_are_disabled() {
        let mut cpu = Cpu::default();
        cpu.registers.status.set_interrupt_disable(true);

        cpu.sample_nmi_input(false);
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(true);
        cpu.sample_nmi_input(false);
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(true);
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        cpu.sample_nmi_input(true);
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(false);
        cpu.sample_nmi_input(true);
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
    }

    #[test]
    fn irq_respects_mask_and_remains_asserted_after_nmi_service() {
        let mut cpu = Cpu::default();
        cpu.interrupt_state.irq_asserted = true;
        cpu.registers.status.set_interrupt_disable(true);
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(true);
        cpu.sample_nmi_input(false);
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        assert!(cpu.take_interrupt().is_none());

        cpu.registers.status.set_interrupt_disable(false);
        cpu.sample_nmi_input(true);
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::IRQ)));
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::IRQ)));

        cpu.interrupt_state.irq_asserted = false;
        assert!(cpu.take_interrupt().is_none());
    }
}
