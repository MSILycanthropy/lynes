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
    pub(crate) fn clock_cycle(&mut self, bus: &mut CpuBus) {
        bus.total_cpu_cycles += 1;

        for _ in 0..3 {
            bus.frame_pending |= bus.ppu.tick();
            // Preserve the current per-dot sampling until cycle phases are modeled.
            self.sample_nmi_input(bus.nmi_asserted());
        }
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

        let value = bus.read(address);

        // Temporary: preserve existing sampling until clocked accesses land.
        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 2 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        self.clock_cycle(bus);

        value
    }

    pub(crate) fn write(&mut self, bus: &mut CpuBus, address: u16, value: u8) {
        let effect = bus.write(address, value);

        // Preserve the current sampling workaround, including PPUCTRL mirrors,
        // until interrupt sampling moves into the CPU cycle clocking path.
        if (0x2000..=0x3FFF).contains(&address) && address & 0b111 == 0 {
            self.sample_nmi_input(bus.nmi_asserted());
        }

        self.clock_cycle(bus);

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
        self.stack_push_u16(bus, self.registers.program_counter);

        let mut status = self.registers.status.clone();
        status.set_b(0b10);

        self.stack_push(bus, status.bits());

        self.registers.status.set_interrupt_disable(true);
        self.registers.program_counter = self.read_u16(bus, interrupt.address());
    }

    pub(crate) fn take_interrupt(&mut self) -> Option<Interrupt> {
        self.interrupt_state
            .take(self.registers.status.interrupt_disable())
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
    fn reset_discards_pending_dma_before_fetching_the_vector() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 16384];
        nes.bus.cartridge.prg_rom[0x3FFC..0x3FFE].copy_from_slice(&0x8000u16.to_le_bytes());
        nes.bus.ppu.oam_data.fill(0x55);
        nes.cpu_write(0x4014, 0x02);

        nes.reset();
        assert_eq!(nes.bus.oam_dma_request, None);
        assert!(nes.bus.ppu.oam_data.iter().all(|&value| value == 0x55));
        assert_eq!(nes.cpu.registers.program_counter, 0x8000);
    }
}
