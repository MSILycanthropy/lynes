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
    accepted: Option<Interrupt>,
}

impl InterruptState {
    fn sample_nmi(&mut self, asserted: bool) {
        if asserted && !self.previous_nmi {
            self.nmi_pending = true;
        }

        self.previous_nmi = asserted;
    }

    fn poll(&mut self, interrupt_disable: bool) {
        let accepted = if self.nmi_pending {
            Some(Interrupt::NMI)
        } else if self.irq_asserted && !interrupt_disable {
            Some(Interrupt::IRQ)
        } else {
            None
        };

        if accepted.is_none() {
            return;
        }

        self.accepted = accepted;
    }

    fn take(&mut self) -> Option<Interrupt> {
        self.accepted.take()
    }
}

#[derive(Default)]
pub struct Cpu {
    pub registers: CpuRegisters,
    interrupt_state: InterruptState,
}

impl Cpu {
    pub(crate) fn clock_cycle(&mut self, bus: &mut CpuBus) {
        bus.total_cpu_cycles += 1;

        for _ in 0..3 {
            bus.frame_pending |= bus.ppu.tick();
            // Preserve the current per-dot sampling until cycle phases are modeled.
            self.sample_nmi_input(bus.nmi_asserted());
        }
    }

    pub(crate) fn reset(&mut self, bus: &mut CpuBus) {
        bus.oam_dma_request = None;

        let pc = self.registers.program_counter;

        self.read(bus, pc);
        self.read(bus, pc);

        for _ in 0..3 {
            let address = 0x0100 | u16::from(self.registers.stack_pointer);

            self.read(bus, address);

            self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
        }

        self.registers.status.set_interrupt_disable(true);
        self.registers.program_counter = self.read_u16(bus, 0xFFFC);
    }

    pub(crate) fn fetch_instruction_byte(&mut self, bus: &mut CpuBus) -> u8 {
        let value = self.read(bus, self.registers.program_counter);

        let pc = self.registers.program_counter.wrapping_add(1);
        self.registers.program_counter = pc;

        value
    }

    pub(crate) fn read(&mut self, bus: &mut CpuBus, address: u16) -> u8 {
        if let Some(page) = bus.oam_dma_request.take() {
            self.run_oam_dma(bus, page, address);
        }

        self.clock_cycle(bus);
        let value = bus.read(address);

        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 2 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        value
    }

    pub(crate) fn write(&mut self, bus: &mut CpuBus, address: u16, value: u8) {
        self.clock_cycle(bus);
        let effect = bus.write(address, value);

        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 0 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        if let WriteEffect::OamDma { page } = effect {
            bus.oam_dma_request = Some(page);
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

    fn run_oam_dma(&mut self, bus: &mut CpuBus, page: u8, halted_address: u16) {
        self.read(bus, halted_address);

        if bus.total_cpu_cycles & 1 != 0 {
            self.read(bus, halted_address);
        }

        let start = u16::from(page) << 8;

        for offset in 0..256u16 {
            let value = self.read(bus, start + offset);
            self.write(bus, 0x2004, value);
        }
    }

    pub(crate) fn sample_nmi_input(&mut self, nmi_asserted: bool) {
        self.interrupt_state.sample_nmi(nmi_asserted);
    }

    pub(crate) fn enter_interrupt(&mut self, bus: &mut CpuBus, interrupt: &Interrupt) {
        let pc = self.registers.program_counter;

        self.read(bus, pc);
        self.read(bus, pc);

        let mut status = self.registers.status.clone();
        status.set_b(0b10);

        self.finish_interrupt_entry(bus, interrupt, status.bits());
    }

    pub(crate) fn finish_interrupt_entry(
        &mut self,
        bus: &mut CpuBus,
        requested: &Interrupt,
        status: u8,
    ) {
        self.stack_push_u16(bus, self.registers.program_counter);

        let vector = self.select_interrupt_vector(requested);

        self.stack_push(bus, status);

        self.registers.status.set_interrupt_disable(true);
        self.registers.program_counter = self.read_u16(bus, vector);
    }

    fn select_interrupt_vector(&mut self, requested: &Interrupt) -> u16 {
        if matches!(requested, Interrupt::NMI) || self.interrupt_state.nmi_pending {
            self.interrupt_state.nmi_pending = false;
            Interrupt::NMI.address()
        } else {
            requested.address()
        }
    }

    pub(crate) fn poll_interrupts(&mut self) {
        self.interrupt_state
            .poll(self.registers.status.interrupt_disable())
    }

    pub(crate) fn take_interrupt(&mut self) -> Option<Interrupt> {
        self.interrupt_state.take()
    }
}

impl Cpu {
    pub(crate) fn execute_next_instruction(&mut self, bus: &mut CpuBus) -> (u16, u8) {
        let pc = self.registers.program_counter;
        let opcode = self.fetch_instruction_byte(bus);
        self.execute_instruction(bus, opcode);

        (pc, opcode)
    }

    fn execute_instruction(&mut self, bus: &mut CpuBus, opcode: u8) {
        let instruction = &instructions::INSTRUCTIONS_TABLE[opcode as usize];

        instruction.execute(self, bus);
    }
}

impl NES {
    pub fn enter_interrupt(&mut self, interrupt: &Interrupt) {
        self.cpu.enter_interrupt(&mut self.bus, interrupt);
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
    use super::addressing::AccessKind;
    use super::*;
    use crate::StepKind;

    #[test]
    fn status_read_observes_vblank_starting_during_its_bus_cycle() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        bus.ppu.scanline = 241;
        bus.ppu.dot = 0;
        assert!(!bus.ppu.registers.status.vblank_started());

        // The read completes at dot 3, after vblank starts at dot 1.
        assert_eq!(cpu.read(&mut bus, 0x2002) & 0x80, 0x80);
        assert_eq!(bus.total_cpu_cycles, 1);
        assert_eq!((bus.ppu.scanline, bus.ppu.dot), (241, 3));
        assert!(!bus.ppu.registers.status.vblank_started());
    }

    #[test]
    fn control_write_enables_nmi_using_vblank_at_the_bus_access() {
        for (scanline, initially_in_vblank, nmi_expected) in
            [(241, false, true), (261, true, false)]
        {
            let mut cpu = Cpu::default();
            let mut bus = CpuBus::default();
            bus.ppu.scanline = scanline;
            bus.ppu.dot = 0;
            bus.ppu
                .registers
                .status
                .set_vblank_started(initially_in_vblank);

            // Vblank starts/ends before this write enables NMI at dot 3.
            // Enabling at the beginning of the cycle would use the old flag.
            cpu.write(&mut bus, 0x2000, 0x80);

            assert_eq!(bus.total_cpu_cycles, 1);
            assert_eq!((bus.ppu.scanline, bus.ppu.dot), (scanline, 3));
            assert_eq!(bus.nmi_asserted(), nmi_expected);
            assert_eq!(cpu.interrupt_state.nmi_pending, nmi_expected);
        }
    }

    fn interrupt_hijack_machine(brk: bool, nmi_cycle: usize) -> NES {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        // Different low and high bytes expose any mixed-vector fetch.
        nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x10, 0x07]);
        nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x20, 0x06]);
        nes.bus.write(0x0200, 0x00); // BRK, or the discarded opcode during IRQ.
        nes.bus.write(0x0201, 0xEA); // BRK padding.
        nes.bus.write(0x0620, 0xEA); // First IRQ-handler instruction.
        nes.bus.write(0x0710, 0xEA); // First two NMI-handler instructions.
        nes.bus.write(0x0711, 0xEA);
        nes.cpu.registers.program_counter = 0x0200;
        nes.cpu.registers.stack_pointer = 0xFD;
        // BRK also verifies that I cannot prevent an NMI hijack.
        nes.cpu
            .registers
            .status
            .set_bits(if brk { 0xEF } else { 0xEB });
        if !brk {
            nes.cpu.interrupt_state.irq_asserted = true;
            nes.cpu.poll_interrupts();
        }
        nes.bus.write(0x2000, 0x80);
        nes.bus.ppu.scanline = 240;
        // Vblank begins on the last PPU dot of the specified CPU cycle.
        nes.bus.ppu.dot = 342 - 3 * nmi_cycle;
        nes
    }

    #[test]
    fn nmi_hijacks_brk_and_irq_through_cycle_four_without_changing_the_stack() {
        for brk in [false, true] {
            for nmi_cycle in 1..=4 {
                let mut nes = interrupt_hijack_machine(brk, nmi_cycle);
                let entry = nes.step();

                if brk {
                    assert!(matches!(
                        entry.kind,
                        StepKind::Instructrion { opcode: 0x00, .. }
                    ));
                } else {
                    // StepKind records what started entry, even when redirected.
                    assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::IRQ)));
                }
                assert_eq!(entry.cpu_cycles, 7);
                assert_eq!(
                    nes.cpu.registers.program_counter, 0x0710,
                    "BRK={brk}, NMI in cycle {nmi_cycle}"
                );
                assert_eq!(nes.cpu.registers.stack_pointer, 0xFA);
                assert_eq!(nes.bus.peek(0x01FD), 0x02);
                assert_eq!(nes.bus.peek(0x01FC), if brk { 0x02 } else { 0x00 });
                assert_eq!(nes.bus.peek(0x01FB), if brk { 0xFF } else { 0xEB });
                assert_eq!(nes.cpu.registers.status.bits(), 0xEF);
                assert!(nes.bus.nmi_asserted());
                assert!(!nes.cpu.interrupt_state.nmi_pending);
                assert!(nes.cpu.interrupt_state.accepted.is_none());
                assert_eq!(nes.cpu.interrupt_state.irq_asserted, !brk);

                // A held NMI line must not trigger another entry after the
                // handler's first instruction polls it.
                for expected_pc in [0x0710, 0x0711] {
                    let nop = nes.step();
                    assert!(matches!(nop.kind,
                        StepKind::Instructrion { pc, opcode: 0xEA } if pc == expected_pc));
                    assert_eq!(nop.cpu_cycles, 2);
                    assert_eq!(nes.cpu.registers.program_counter, expected_pc + 1);
                    assert_eq!(nes.cpu.registers.stack_pointer, 0xFA);
                    assert!(!nes.cpu.interrupt_state.nmi_pending);
                    assert!(nes.cpu.interrupt_state.accepted.is_none());
                }
            }
        }
    }

    #[test]
    fn nmi_after_hijack_cutoff_preserves_the_vector_and_waits_for_a_handler_poll() {
        for brk in [false, true] {
            for nmi_cycle in 5..=7 {
                let mut nes = interrupt_hijack_machine(brk, nmi_cycle);
                let entry = nes.step();

                if brk {
                    assert!(matches!(
                        entry.kind,
                        StepKind::Instructrion { opcode: 0x00, .. }
                    ));
                } else {
                    assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::IRQ)));
                }
                assert_eq!(entry.cpu_cycles, 7);
                assert_eq!(
                    nes.cpu.registers.program_counter, 0x0620,
                    "BRK={brk}, NMI in cycle {nmi_cycle}"
                );
                assert_eq!(nes.cpu.registers.stack_pointer, 0xFA);
                assert_eq!(nes.bus.peek(0x01FD), 0x02);
                assert_eq!(nes.bus.peek(0x01FC), if brk { 0x02 } else { 0x00 });
                assert_eq!(nes.bus.peek(0x01FB), if brk { 0xFF } else { 0xEB });
                assert_eq!(nes.cpu.registers.status.bits(), 0xEF);
                assert!(nes.cpu.interrupt_state.nmi_pending);
                assert!(nes.cpu.interrupt_state.accepted.is_none());

                let nop = nes.step();
                assert!(matches!(
                    nop.kind,
                    StepKind::Instructrion {
                        pc: 0x0620,
                        opcode: 0xEA
                    }
                ));
                assert_eq!(nop.cpu_cycles, 2);
                assert_eq!(nes.cpu.registers.program_counter, 0x0621);
                assert!(matches!(
                    nes.cpu.interrupt_state.accepted,
                    Some(Interrupt::NMI)
                ));

                let nmi = nes.step();
                assert!(matches!(nmi.kind, StepKind::Interrupt(Interrupt::NMI)));
                assert_eq!(nmi.cpu_cycles, 7);
                assert_eq!(nes.cpu.registers.program_counter, 0x0710);
                assert_eq!(nes.cpu.registers.stack_pointer, 0xF7);
                assert_eq!(nes.bus.peek(0x01FA), 0x06);
                assert_eq!(nes.bus.peek(0x01F9), 0x21);
                assert_eq!(nes.bus.peek(0x01F8), 0xEF);
                assert!(!nes.cpu.interrupt_state.nmi_pending);
                assert!(nes.cpu.interrupt_state.accepted.is_none());
            }
        }
    }

    #[test]
    fn interrupt_entry_saves_pc_and_old_status_and_rti_restores_them() {
        for (interrupt, status, handler) in [
            (Interrupt::IRQ, 0xEB, 0x0600),
            (Interrupt::NMI, 0xEB, 0x0700),
            (Interrupt::NMI, 0xEF, 0x0700), // NMI also enters with I set.
        ] {
            let mut nes = NES::default();
            nes.bus.cartridge.prg_rom = vec![0; 0x4000];
            nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x00, 0x07]);
            nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
            nes.bus.write(handler, 0x40); // RTI
            nes.cpu.registers.program_counter = 0x5234;
            nes.cpu.registers.stack_pointer = 0; // Exercise stack wrapping.
            nes.cpu.registers.status.set_bits(status);
            nes.cpu.registers.accumulator = 0xA5;
            nes.cpu.registers.x = 0x12;
            nes.cpu.registers.y = 0x34;
            match interrupt {
                Interrupt::IRQ => nes.cpu.interrupt_state.irq_asserted = true,
                Interrupt::NMI => nes.cpu.sample_nmi_input(true),
            }
            // This fixture starts after the preceding instruction's poll.
            nes.cpu.poll_interrupts();

            let entry = nes.step();

            assert!(matches!(entry.kind, StepKind::Interrupt(actual) if actual == interrupt));
            assert_eq!(entry.cpu_cycles, 7);
            assert_eq!(nes.bus.ppu.dot, 21);
            assert_eq!(nes.cpu.registers.program_counter, handler);
            assert_eq!(nes.cpu.registers.stack_pointer, 0xFD);
            assert_eq!(nes.bus.peek(0x0100), 0x52);
            assert_eq!(nes.bus.peek(0x01FF), 0x34);
            assert_eq!(nes.bus.peek(0x01FE), status); // B clear; original I.
            assert_eq!(nes.cpu.registers.status.bits(), status | 0x04);

            let resumed = nes.step();

            assert!(matches!(
                resumed.kind,
                StepKind::Instructrion { opcode: 0x40, .. }
            ));
            assert_eq!(resumed.cpu_cycles, 6);
            assert_eq!(nes.cpu.registers.program_counter, 0x5234);
            assert_eq!(nes.cpu.registers.stack_pointer, 0);
            assert_eq!(nes.cpu.registers.status.bits(), status);
            assert_eq!(nes.cpu.registers.accumulator, 0xA5);
            assert_eq!(nes.cpu.registers.x, 0x12);
            assert_eq!(nes.cpu.registers.y, 0x34);
        }
    }

    #[test]
    fn interrupt_entry_reads_the_same_pc_twice_with_bus_side_effects() {
        for interrupt in [Interrupt::IRQ, Interrupt::NMI] {
            let mut nes = NES::default();
            nes.bus.cartridge.prg_rom = vec![0; 0x4000];
            nes.cpu.registers.program_counter = 0x4016;
            nes.cpu.registers.stack_pointer = 0xFD;
            nes.cpu.registers.status.set_interrupt_disable(false);
            nes.bus.controller.button_state.set_select(true);
            match interrupt {
                Interrupt::IRQ => nes.cpu.interrupt_state.irq_asserted = true,
                Interrupt::NMI => nes.cpu.sample_nmi_input(true),
            }
            // This fixture starts after the preceding instruction's poll.
            nes.cpu.poll_interrupts();

            let entry = nes.step();

            assert!(matches!(entry.kind, StepKind::Interrupt(actual) if actual == interrupt));
            assert_eq!(entry.cpu_cycles, 7);
            // The two reads consumed A and B, leaving Select next.
            assert_eq!(nes.bus.controller.peek(), 1);
            assert_eq!(nes.bus.peek(0x01FD), 0x40);
            assert_eq!(nes.bus.peek(0x01FC), 0x16);
        }
    }

    #[test]
    fn indexed_addressing_clocks_dummy_reads_and_applies_io_side_effects() {
        for mode in [
            AddrMode::AbsoluteX,
            AddrMode::AbsoluteY,
            AddrMode::IndirectY,
        ] {
            // Both cases read $4016 when a dummy access is required. The
            // crossing case must use the uncorrected high byte, not $4116.
            for (base, index, expected_address, crossed) in [
                (0x4015_u16, 1, 0x4016, false),
                (0x40FF_u16, 0x17, 0x4116, true),
            ] {
                for (access, always_dummy) in [
                    (AccessKind::Read, false),
                    (AccessKind::Write, true),
                    (AccessKind::ReadModifyWrite, true),
                ] {
                    let mut cpu = Cpu::default();
                    let mut bus = CpuBus::default();
                    cpu.registers.program_counter = 0x0200;
                    cpu.registers.x = index;
                    cpu.registers.y = index;
                    bus.controller.button_state.set_a(true);
                    let [low, high] = base.to_le_bytes();
                    let operand_cycles = if matches!(mode, AddrMode::IndirectY) {
                        bus.write(0x0200, 0xFF);
                        bus.write(0x00FF, low);
                        bus.write(0x0000, high); // Pointer wraps within zero page.
                        3
                    } else {
                        bus.write(0x0200, low);
                        bus.write(0x0201, high);
                        2
                    };

                    let address = cpu.fetch_operand_address(&mut bus, &mode, access);
                    let dummy_read = crossed || always_dummy;

                    assert_eq!(address, expected_address);
                    assert_eq!(
                        bus.total_cpu_cycles,
                        operand_cycles + usize::from(dummy_read)
                    );
                    assert_eq!(bus.controller.peek(), u8::from(!dummy_read));
                    assert_eq!(
                        cpu.registers.program_counter,
                        if matches!(mode, AddrMode::IndirectY) {
                            0x0201
                        } else {
                            0x0202
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn zero_page_indexing_reads_the_base_and_wraps_the_address() {
        for mode in [AddrMode::ZeroPageX, AddrMode::ZeroPageY] {
            let mut cpu = Cpu::default();
            let mut bus = CpuBus::default();
            cpu.registers.program_counter = 0x0200;
            cpu.registers.x = 2;
            cpu.registers.y = 2;
            bus.write(0x0200, 0xFF);
            bus.write(0x00FF, 0x5A);
            bus.write(0x0001, 0xA5);

            let address = cpu.fetch_operand_address(&mut bus, &mode, AccessKind::Read);

            assert_eq!(address, 0x0001);
            assert_eq!(bus.total_cpu_cycles, 2);
            // CPU open bus retains the dummy read's byte, not the operand
            // byte or the final target's data.
            assert_eq!(bus.peek(0x4018), 0x5A);
        }
    }

    #[test]
    fn indexed_indirect_clocks_the_dummy_read_and_wraps_the_pointer() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        cpu.registers.program_counter = 0x0200;
        cpu.registers.x = 2;
        bus.write(0x0200, 0xFD);
        bus.write(0x00FF, 0x34);
        bus.write(0x0000, 0x12);

        let address = cpu.fetch_operand_address(&mut bus, &AddrMode::IndirectX, AccessKind::Read);

        assert_eq!(address, 0x1234);
        assert_eq!(bus.total_cpu_cycles, 4);
    }

    #[test]
    fn operand_address_fetch_wraps_pc_without_reading_target_data() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        bus.cartridge.prg_rom = vec![0; 0x4000];
        bus.cartridge.prg_rom[0x3FFF] = 0x16;
        bus.write(0x0000, 0x40);
        bus.controller.button_state.set_a(true);
        cpu.registers.program_counter = 0xFFFF;

        let address = cpu.fetch_operand_address(&mut bus, &AddrMode::Absolute, AccessKind::Read);

        assert_eq!(address, 0x4016);
        assert_eq!(cpu.registers.program_counter, 0x0001);
        assert_eq!(bus.total_cpu_cycles, 2);
        assert_eq!(bus.controller.peek(), 1);
    }

    #[test]
    fn read_operand_consumes_immediate_bytes_once_and_memory_data_separately() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        cpu.registers.program_counter = 0x4016;
        bus.controller.button_state.set_a(true);
        bus.controller.button_state.set_select(true);

        assert_eq!(cpu.read_operand(&mut bus, &AddrMode::Immediate) & 1, 1);
        assert_eq!(cpu.registers.program_counter, 0x4017);
        assert_eq!(bus.total_cpu_cycles, 1);
        assert_eq!(bus.controller.peek(), 0); // Exactly one controller read.

        cpu.registers.program_counter = 0x0200;
        bus.write(0x0200, 0x16);
        bus.write(0x0201, 0x40);
        assert_eq!(cpu.read_operand(&mut bus, &AddrMode::Absolute) & 1, 0);
        assert_eq!(cpu.registers.program_counter, 0x0202);
        assert_eq!(bus.total_cpu_cycles, 4);
        assert_eq!(bus.controller.peek(), 1);
    }

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
        let mut bus = CpuBus::default();
        bus.cartridge.prg_rom = vec![0; 0x4000];
        cpu.registers.status.set_interrupt_disable(true);

        cpu.sample_nmi_input(false);
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(true);
        cpu.sample_nmi_input(false);
        assert!(cpu.take_interrupt().is_none()); // An edge alone is not a poll.
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        assert!(cpu.interrupt_state.nmi_pending); // Taking is not acknowledgment.
        cpu.enter_interrupt(&mut bus, &Interrupt::NMI);
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());

        // Hold the actual PPU output high throughout interrupt entry.
        bus.ppu.scanline = 241;
        bus.ppu.registers.status.set_vblank_started(true);
        bus.write(0x2000, 0x80);
        cpu.sample_nmi_input(true);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        cpu.enter_interrupt(&mut bus, &Interrupt::NMI);
        cpu.sample_nmi_input(true);
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(false);
        cpu.sample_nmi_input(true);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
    }

    #[test]
    fn irq_respects_mask_and_remains_asserted_after_nmi_service() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        bus.cartridge.prg_rom = vec![0; 0x4000];
        cpu.interrupt_state.irq_asserted = true;
        cpu.registers.status.set_interrupt_disable(true);
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());

        cpu.sample_nmi_input(true);
        cpu.sample_nmi_input(false);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        cpu.enter_interrupt(&mut bus, &Interrupt::NMI);
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());

        cpu.registers.status.set_interrupt_disable(false);
        cpu.sample_nmi_input(true);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        cpu.enter_interrupt(&mut bus, &Interrupt::NMI);
        assert!(cpu.interrupt_state.irq_asserted);
        cpu.registers.status.set_interrupt_disable(false);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::IRQ)));
        cpu.enter_interrupt(&mut bus, &Interrupt::IRQ);
        cpu.registers.status.set_interrupt_disable(false);
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::IRQ)));

        cpu.interrupt_state.irq_asserted = false;
        cpu.poll_interrupts();
        assert!(cpu.take_interrupt().is_none());
    }

    #[test]
    fn cli_delays_an_asserted_irq_until_after_the_following_instruction() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
        nes.bus.write(0, 0x58); // CLI
        nes.bus.write(1, 0xEA); // NOP
        nes.cpu.registers.status.set_interrupt_disable(true);
        nes.cpu.interrupt_state.irq_asserted = true;

        let cli = nes.step();
        assert!(matches!(
            cli.kind,
            StepKind::Instructrion { opcode: 0x58, .. }
        ));
        assert_eq!(cli.cpu_cycles, 2);
        assert!(!nes.cpu.registers.status.interrupt_disable());
        assert!(nes.cpu.interrupt_state.accepted.is_none());

        let nop = nes.step();
        assert!(matches!(
            nop.kind,
            StepKind::Instructrion { opcode: 0xEA, .. }
        ));
        assert_eq!(nop.cpu_cycles, 2);
        assert_eq!(nes.cpu.registers.program_counter, 2);

        let entry = nes.step();
        assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::IRQ)));
        assert_eq!(entry.cpu_cycles, 7);
        assert_eq!(nes.cpu.registers.program_counter, 0x0600);
    }

    #[test]
    fn sei_does_not_cancel_an_irq_accepted_using_the_old_interrupt_mask() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
        nes.bus.write(0, 0x78); // SEI
        nes.cpu.registers.status.set_interrupt_disable(false);
        nes.cpu.interrupt_state.irq_asserted = true;

        let sei = nes.step();
        assert!(matches!(
            sei.kind,
            StepKind::Instructrion { opcode: 0x78, .. }
        ));
        assert_eq!(sei.cpu_cycles, 2);
        assert!(nes.cpu.registers.status.interrupt_disable());

        // Entry must use the accepted decision, without rechecking I or the line.
        nes.cpu.interrupt_state.irq_asserted = false;
        let entry = nes.step();
        assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::IRQ)));
        assert_eq!(entry.cpu_cycles, 7);
        assert_eq!(nes.cpu.registers.program_counter, 0x0600);
    }

    #[test]
    fn nmi_arriving_after_nop_poll_waits_for_the_next_instruction_poll() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x00, 0x06]);
        nes.bus.write(0, 0xEA); // NOP
        nes.bus.write(1, 0xEA); // NOP
        nes.bus.write(0x2000, 0x80);
        // Vblank begins during the dummy read, after this NOP's poll.
        nes.bus.ppu.scanline = 240;
        nes.bus.ppu.dot = 337;

        let first = nes.step();
        assert!(matches!(
            first.kind,
            StepKind::Instructrion { opcode: 0xEA, .. }
        ));
        assert_eq!(first.cpu_cycles, 2);
        assert!(nes.cpu.interrupt_state.nmi_pending);
        assert!(nes.cpu.interrupt_state.accepted.is_none());

        let second = nes.step();
        assert!(matches!(
            second.kind,
            StepKind::Instructrion { opcode: 0xEA, .. }
        ));
        assert_eq!(second.cpu_cycles, 2);
        assert_eq!(nes.cpu.registers.program_counter, 2);

        let entry = nes.step();
        assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::NMI)));
        assert_eq!(entry.cpu_cycles, 7);
        assert_eq!(nes.cpu.registers.program_counter, 0x0600);
        assert!(!nes.cpu.interrupt_state.nmi_pending);
    }

    #[test]
    fn branch_first_poll_accepts_irq_for_each_branch_path() {
        for (pc, taken, target, cycles) in [
            (0x0200, false, 0x0202, 2),
            (0x0200, true, 0x0204, 3),
            (0x02FC, true, 0x0300, 4),
        ] {
            let mut nes = NES::default();
            nes.bus.cartridge.prg_rom = vec![0; 0x4000];
            nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
            nes.bus.write(pc, 0x90); // BCC +2
            nes.bus.write(pc + 1, 2);
            nes.bus.write(target, 0xEA); // Must not execute before IRQ entry.
            nes.cpu.registers.program_counter = pc;
            nes.cpu.registers.stack_pointer = 0xFD;
            nes.cpu.registers.status.set_carry(!taken);
            nes.cpu.registers.status.set_interrupt_disable(false);
            nes.cpu.interrupt_state.irq_asserted = true;

            let branch = nes.step();
            assert!(matches!(
                branch.kind,
                StepKind::Instructrion { opcode: 0x90, .. }
            ));
            assert_eq!(branch.cpu_cycles, cycles);
            assert_eq!(nes.cpu.registers.program_counter, target);
            assert!(matches!(
                nes.cpu.interrupt_state.accepted,
                Some(Interrupt::IRQ)
            ));

            // The accepted decision survives the source going away.
            nes.cpu.interrupt_state.irq_asserted = false;
            let entry = nes.step();
            assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::IRQ)));
            assert_eq!(entry.cpu_cycles, 7);
            assert_eq!(nes.cpu.registers.program_counter, 0x0600);
            assert_eq!(nes.bus.peek(0x01FD), (target >> 8) as u8);
            assert_eq!(nes.bus.peek(0x01FC), target as u8);
        }
    }

    #[test]
    fn taken_same_page_branch_delays_nmi_arriving_after_its_first_poll() {
        // At three PPU dots per CPU cycle, these positions put the vblank edge
        // in either the offset fetch (cycle 2) or the dummy read (cycle 3).
        for dot in [337, 334] {
            let mut nes = NES::default();
            nes.bus.cartridge.prg_rom = vec![0; 0x4000];
            nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x00, 0x06]);
            nes.bus.write(0x0200, 0x90); // BCC +2
            nes.bus.write(0x0201, 2);
            nes.bus.write(0x0204, 0xEA); // NOP at the branch target.
            nes.cpu.registers.program_counter = 0x0200;
            nes.cpu.registers.stack_pointer = 0xFD;
            nes.cpu.registers.status.set_carry(false);
            nes.bus.write(0x2000, 0x80);
            nes.bus.ppu.scanline = 240;
            nes.bus.ppu.dot = dot;

            let branch = nes.step();
            assert!(matches!(
                branch.kind,
                StepKind::Instructrion { opcode: 0x90, .. }
            ));
            assert_eq!(branch.cpu_cycles, 3);
            assert_eq!(nes.cpu.registers.program_counter, 0x0204);
            assert!(nes.cpu.interrupt_state.nmi_pending);
            assert!(nes.cpu.interrupt_state.accepted.is_none());

            let nop = nes.step();
            assert!(matches!(
                nop.kind,
                StepKind::Instructrion {
                    pc: 0x0204,
                    opcode: 0xEA
                }
            ));
            assert_eq!(nop.cpu_cycles, 2);
            assert_eq!(nes.cpu.registers.program_counter, 0x0205);

            let entry = nes.step();
            assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::NMI)));
            assert_eq!(entry.cpu_cycles, 7);
            assert_eq!(nes.cpu.registers.program_counter, 0x0600);
            assert_eq!(nes.bus.peek(0x01FD), 0x02);
            assert_eq!(nes.bus.peek(0x01FC), 0x05);
        }
    }

    #[test]
    fn crossing_branch_second_poll_catches_nmi_before_but_not_during_final_read() {
        for (pc, offset, target) in [(0x02FC, 0x02, 0x0300), (0x0300, 0xFC, 0x02FE)] {
            // Vblank edges in cycles 2 and 3 are caught by the second poll.
            // An edge in cycle 4 arrives after that poll and must wait.
            for (dot, accepted) in [(337, true), (334, true), (331, false)] {
                let mut nes = NES::default();
                nes.bus.cartridge.prg_rom = vec![0; 0x4000];
                nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x00, 0x06]);
                nes.bus.write(pc, 0x90); // BCC across a page boundary.
                nes.bus.write(pc + 1, offset);
                nes.bus.write(target, 0xEA);
                nes.cpu.registers.program_counter = pc;
                nes.cpu.registers.stack_pointer = 0xFD;
                nes.cpu.registers.status.set_carry(false);
                nes.bus.write(0x2000, 0x80);
                nes.bus.ppu.scanline = 240;
                nes.bus.ppu.dot = dot;

                let branch = nes.step();
                assert!(matches!(
                    branch.kind,
                    StepKind::Instructrion { opcode: 0x90, .. }
                ));
                assert_eq!(branch.cpu_cycles, 4);
                assert_eq!(nes.cpu.registers.program_counter, target);
                assert!(nes.cpu.interrupt_state.nmi_pending);
                assert_eq!(
                    matches!(nes.cpu.interrupt_state.accepted, Some(Interrupt::NMI)),
                    accepted,
                    "branch at {pc:#06X}, starting PPU dot {dot}"
                );

                let return_address = if accepted {
                    target
                } else {
                    let nop = nes.step();
                    assert!(matches!(
                        nop.kind,
                        StepKind::Instructrion { opcode: 0xEA, .. }
                    ));
                    assert_eq!(nop.cpu_cycles, 2);
                    assert_eq!(nes.cpu.registers.program_counter, target + 1);
                    target + 1
                };

                let entry = nes.step();
                assert!(matches!(entry.kind, StepKind::Interrupt(Interrupt::NMI)));
                assert_eq!(entry.cpu_cycles, 7);
                assert_eq!(nes.cpu.registers.program_counter, 0x0600);
                assert_eq!(nes.bus.peek(0x01FD), (return_address >> 8) as u8);
                assert_eq!(nes.bus.peek(0x01FC), return_address as u8);
            }
        }
    }

    #[test]
    fn later_polls_preserve_an_accepted_interrupt_and_prioritize_nmi() {
        let mut state = InterruptState::default();
        state.irq_asserted = true;
        state.poll(false);
        state.irq_asserted = false;
        state.poll(false);
        assert!(matches!(state.accepted, Some(Interrupt::IRQ)));

        state.sample_nmi(true);
        state.poll(true);
        assert!(matches!(state.accepted, Some(Interrupt::NMI)));
        state.sample_nmi(false);
        state.irq_asserted = true;
        state.poll(false);
        assert!(matches!(state.take(), Some(Interrupt::NMI)));
    }

    #[test]
    fn dma_waits_for_a_read_and_copies_with_both_alignments_and_oam_wrap() {
        // Convention: even cycle indices are get cycles, odd indices are put cycles.
        for (initial_cycles, dma_cycles) in [(0, 513), (1, 514)] {
            let mut cpu = Cpu::default();
            let mut bus = CpuBus::default();
            for offset in 0..256u16 {
                bus.write(0x0200 + offset, (offset as u8) ^ 0xA5);
            }
            bus.write(0x0010, 0xE7);
            bus.write(0x2003, 0xFC);
            for _ in 0..initial_cycles {
                cpu.clock_cycle(&mut bus);
            }

            cpu.write(&mut bus, 0x4014, 0x02);
            assert_eq!(bus.total_cpu_cycles, initial_cycles + 1);
            assert_eq!(bus.oam_dma_request, Some(0x02));
            assert!(bus.ppu.oam_data.iter().all(|&value| value == 0));

            let before = bus.total_cpu_cycles;
            assert_eq!(cpu.read(&mut bus, 0x0010), 0xE7);
            assert_eq!(bus.total_cpu_cycles - before, dma_cycles + 1);
            assert_eq!(bus.oam_dma_request, None);
            assert_eq!(bus.ppu.registers.oam_addr, 0xFC);
            for offset in 0..256u16 {
                let destination = 0xFCu8.wrapping_add(offset as u8);
                assert_eq!(
                    bus.ppu.oam_data[usize::from(destination)],
                    (offset as u8) ^ 0xA5
                );
            }
            assert_eq!(bus.ppu.peek_io_latch(), 0xFF ^ 0xA5);
            let dots = bus.total_cpu_cycles * 3;
            assert_eq!((bus.ppu.scanline, bus.ppu.dot), (dots / 341, dots % 341));

            let before = bus.total_cpu_cycles;
            assert_eq!(cpu.read(&mut bus, 0x0010), 0xE7);
            assert_eq!(bus.total_cpu_cycles - before, 1); // Request is consumed once.
        }
    }

    #[test]
    fn dma_uses_the_latest_page_and_waits_through_consecutive_writes() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        for offset in 0..256u16 {
            bus.write(0x0200 + offset, 0x12);
            bus.write(0x0300 + offset, 0x34);
        }

        cpu.write(&mut bus, 0x4014, 0x02);
        cpu.write(&mut bus, 0x4014, 0x03);
        cpu.write(&mut bus, 0x0010, 0x56);
        assert_eq!(bus.total_cpu_cycles, 3);
        assert_eq!(bus.oam_dma_request, Some(0x03));
        assert!(bus.ppu.oam_data.iter().all(|&value| value == 0));

        assert_eq!(cpu.read(&mut bus, 0x0010), 0x56);
        assert!(bus.ppu.oam_data.iter().all(|&value| value == 0x34));
    }

    #[test]
    fn dma_halt_read_applies_register_side_effects_before_the_resumed_read() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        bus.ppu.registers.status.set_vblank_started(true);
        cpu.write(&mut bus, 0x4014, 0x02);

        // The halt read clears vblank, so the resumed status read must see it cleared.
        assert_eq!(cpu.read(&mut bus, 0x2002) & 0x80, 0);
        assert!(!bus.ppu.registers.status.vblank_started());
    }

    #[test]
    fn nmi_is_latched_during_dma_without_interrupting_the_transfer() {
        let mut cpu = Cpu::default();
        let mut bus = CpuBus::default();
        bus.write(0x2000, 0x80);
        bus.ppu.scanline = 240;
        bus.ppu.dot = 338;
        cpu.write(&mut bus, 0x4014, 0x02);
        assert_eq!((bus.ppu.scanline, bus.ppu.dot), (241, 0));
        assert!(cpu.take_interrupt().is_none());

        cpu.read(&mut bus, 0);
        assert_eq!(bus.total_cpu_cycles, 515);
        assert_eq!(bus.oam_dma_request, None);
        assert!(cpu.interrupt_state.nmi_pending);
        assert!(cpu.take_interrupt().is_none());
        cpu.poll_interrupts();
        assert!(matches!(cpu.take_interrupt(), Some(Interrupt::NMI)));
        assert!(cpu.take_interrupt().is_none());
    }

    #[test]
    fn step_reports_dma_cycles_and_preserves_a_frame_completed_during_transfer() {
        let mut nes = NES::default();
        nes.bus.cartridge.chr_rom = vec![0; 8192]; // Frame completion invokes the renderer.
        nes.bus.write(0, 0xA9); // LDA #$56: two real reads.
        nes.bus.write(1, 0x56);
        nes.cpu_write(0x4014, 0x02);
        nes.bus.ppu.scanline = 261;
        nes.bus.ppu.dot = 339;
        let before = nes.bus.total_cpu_cycles;

        let result = nes.step();
        assert!(matches!(
            result.kind,
            StepKind::Instructrion {
                pc: 0,
                opcode: 0xA9
            }
        ));
        assert_eq!(nes.cpu.registers.accumulator, 0x56);
        assert_eq!(result.cpu_cycles, 513 + 2);
        assert_eq!(nes.bus.total_cpu_cycles - before, result.cpu_cycles);
        assert!(result.frame_ready);
        assert!(!nes.bus.frame_pending);
        assert_eq!(
            nes.bus.ppu.scanline * 341 + nes.bus.ppu.dot,
            result.cpu_cycles * 3 - 2
        );
    }

    #[test]
    fn initial_reset_clocks_seven_cycles_and_initializes_the_stack_pointer() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFC..0x3FFE].copy_from_slice(&[0x23, 0x81]);
        nes.bus.ram[0x0100..0x0200].fill(0xA5);
        assert_eq!(nes.cpu.registers.stack_pointer, 0);

        nes.reset();

        assert_eq!(nes.cpu.registers.program_counter, 0x8123);
        assert_eq!(nes.cpu.registers.stack_pointer, 0xFD);
        assert_eq!(nes.cpu.registers.status.bits(), 0x24);
        assert_eq!(nes.bus.total_cpu_cycles, 7);
        assert_eq!(nes.bus.ppu.scanline, 0);
        assert_eq!(nes.bus.ppu.dot, 21);
        assert!(
            nes.bus.ram[0x0100..0x0200]
                .iter()
                .all(|&value| value == 0xA5)
        );

        nes.reset();

        assert_eq!(nes.cpu.registers.stack_pointer, 0xFA);
        assert_eq!(nes.bus.total_cpu_cycles, 14);
        assert_eq!(nes.bus.ppu.dot, 42);
    }

    #[test]
    fn reset_preserves_registers_and_ram_and_keeps_the_ppu_clock_running() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFC..0x3FFE].copy_from_slice(&[0x23, 0x81]);
        nes.bus.ram.fill(0xA5);
        nes.bus.write(0x6000, 0x5A);
        nes.cpu.registers.accumulator = 0x12;
        nes.cpu.registers.x = 0x34;
        nes.cpu.registers.y = 0x56;
        nes.cpu.registers.stack_pointer = 1;
        nes.cpu.registers.status.set_bits(0xEB); // All real flags except I set.
        nes.cpu.registers.program_counter = 0x4016;
        nes.bus.controller.button_state.set_select(true);
        for _ in 0..13 {
            nes.cpu.clock_cycle(&mut nes.bus);
        }
        // Reset must preserve a frame completed while its bus accesses run.
        nes.bus.ppu.scanline = 261;
        nes.bus.ppu.dot = 335;
        let before = nes.bus.total_cpu_cycles;
        let ram = nes.bus.ram;

        nes.reset();

        assert_eq!(nes.bus.total_cpu_cycles, before + 7);
        assert_eq!(nes.bus.ppu.scanline, 0);
        assert_eq!(nes.bus.ppu.dot, 15);
        assert!(nes.bus.frame_pending);
        assert_eq!(nes.cpu.registers.program_counter, 0x8123);
        assert_eq!(nes.cpu.registers.stack_pointer, 0xFE);
        assert_eq!(nes.cpu.registers.accumulator, 0x12);
        assert_eq!(nes.cpu.registers.x, 0x34);
        assert_eq!(nes.cpu.registers.y, 0x56);
        assert_eq!(nes.cpu.registers.status.bits(), 0xEF);
        assert_eq!(nes.bus.ram, ram);
        assert_eq!(nes.bus.peek(0x6000), 0x5A);
        assert_eq!(nes.bus.controller.peek(), 1); // Two dummy PC reads consumed A and B.
    }

    #[test]
    fn reset_discards_pending_dma_before_fetching_the_vector() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 16384];
        nes.bus.cartridge.prg_rom[0x3FFC..0x3FFE].copy_from_slice(&0x8000u16.to_le_bytes());
        nes.bus.ppu.oam_data.fill(0x55);
        nes.cpu_write(0x4014, 0x02);
        let before = nes.bus.total_cpu_cycles;

        nes.reset();
        assert_eq!(nes.bus.total_cpu_cycles, before + 7);
        assert_eq!(nes.bus.oam_dma_request, None);
        assert!(nes.bus.ppu.oam_data.iter().all(|&value| value == 0x55));
        assert_eq!(nes.cpu.registers.program_counter, 0x8000);
    }
}
