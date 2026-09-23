use crate::{
    Interrupt, NES,
    cpu::{
        bus::{CpuBus, WriteEffect},
        registers::CpuRegisters,
    },
};

mod addressing;
pub mod bus;
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
    pub(crate) fn read(&mut self, bus: &mut CpuBus, address: u16) -> u8 {
        let value = bus.read(address);

        // Temporary: preserve existing sampling until clocked accesses land.
        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 2 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        value
    }

    pub(crate) fn write(&mut self, bus: &mut CpuBus, address: u16, value: u8) {
        let effect = bus.write(address, value);

        // Preserve the current sampling workaround, including PPUCTRL mirrors,
        // until interrupt sampling moves into the CPU cycle clocking path.
        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 0 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        if let WriteEffect::OamDma { page } = effect {
            let mut buffer = [0; 256];
            let start = u16::from(page) << 8;

            for offset in 0..256u16 {
                buffer[offset as usize] = self.read(bus, start + offset);
            }

            bus.ppu.write_oam_dma(&buffer);
        }
    }

    pub(crate) fn read_u16(&mut self, bus: &mut CpuBus, address: u16) -> u16 {
        let low = self.read(bus, address);
        let high = self.read(bus, address.wrapping_add(1));

        u16::from_le_bytes([low, high])
    }

    pub(crate) fn write_u16(&mut self, bus: &mut CpuBus, address: u16, value: u16) {
        let [low, high] = value.to_le_bytes();

        self.write(bus, address, low);
        self.write(bus, address.wrapping_add(1), high);
    }

    pub(crate) fn stack_push(&mut self, bus: &mut CpuBus, value: u8) {
        let address = 0x0100 | u16::from(self.registers.stack_pointer);
        self.write(bus, address, value);
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
    }

    pub(crate) fn stack_pop(&mut self, bus: &mut CpuBus) -> u8 {
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);

        let address = 0x0100 | u16::from(self.registers.stack_pointer);
        self.read(bus, address)
    }

    pub(crate) fn stack_push_u16(&mut self, bus: &mut CpuBus, value: u16) {
        let [low, high] = value.to_le_bytes();

        self.stack_push(bus, high);
        self.stack_push(bus, low);
    }

    pub(crate) fn stack_pop_u16(&mut self, bus: &mut CpuBus) -> u16 {
        let low = self.stack_pop(bus);
        let high = self.stack_pop(bus);

        u16::from_le_bytes([low, high])
    }

    pub(crate) fn sample_nmi_input(&mut self, nmi_asserted: bool) {
        self.interrupt_state.sample_nmi(nmi_asserted);
    }

    pub(crate) fn enter_interrupt(&mut self, bus: &mut CpuBus, interrupt: &Interrupt) -> usize {
        self.stack_push_u16(bus, self.registers.program_counter);

        let mut status = self.registers.status.clone();
        status.set_b(0b10);

        self.stack_push(bus, status.bits());

        self.registers.status.set_interrupt_disable(true);
        self.registers.program_counter = self.read_u16(bus, interrupt.address());

        7
    }

    pub(crate) fn take_interrupt(&mut self) -> Option<Interrupt> {
        self.interrupt_state
            .take(self.registers.status.interrupt_disable())
    }
}

pub trait CPU {
    fn execute_next_instruction(&mut self) -> (u16, u8, usize);

    fn execute_instruction(&mut self, opcode: u8) -> (u16, usize);
}

impl CPU for NES {
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

    fn execute_instruction(&mut self, opcode: u8) -> (u16, usize) {
        let instruction = &instructions::INSTRUCTIONS_TABLE[opcode as usize];

        let cycles = instruction.execute(self);

        (instruction.size(), cycles)
    }
}

impl NES {
    pub fn enter_interrupt(&mut self, interrupt: &Interrupt) -> usize {
        self.cpu.enter_interrupt(&mut self.bus, interrupt)
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        self.cpu.read(&mut self.bus, address)
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        self.cpu.write(&mut self.bus, address, value);
    }

    pub fn stack_push(&mut self, value: u8) {
        self.cpu.stack_push(&mut self.bus, value);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.cpu.stack_pop(&mut self.bus)
    }

    pub fn cpu_read_u16(&mut self, address: u16) -> u16 {
        self.cpu.read_u16(&mut self.bus, address)
    }

    pub fn cpu_write_u16(&mut self, address: u16, value: u16) {
        self.cpu.write_u16(&mut self.bus, address, value);
    }

    pub fn stack_push_u16(&mut self, value: u16) {
        self.cpu.stack_push_u16(&mut self.bus, value);
    }

    pub fn stack_pop_u16(&mut self) -> u16 {
        self.cpu.stack_pop_u16(&mut self.bus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_memory_accesses_cross_pages_and_wrap_at_end_of_address_space() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFF] = 0x34;
        nes.cpu_write(0x0000, 0x12);

        assert_eq!(nes.cpu_read_u16(0xFFFF), 0x1234);

        nes.cpu_write_u16(0xFFFF, 0xABCD);
        // The low-byte NROM write is ignored; the high byte wraps to RAM.
        assert_eq!(nes.cpu_read(0xFFFF), 0x34);
        assert_eq!(nes.cpu_read(0x0000), 0xAB);

        nes.cpu_write_u16(0x00FF, 0x5678);
        assert_eq!(nes.cpu_read(0x00FF), 0x78);
        assert_eq!(nes.cpu_read(0x0100), 0x56);
        assert_eq!(nes.cpu_read(0x0000), 0xAB);
        assert_eq!(nes.cpu_read_u16(0x00FF), 0x5678);
    }

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
