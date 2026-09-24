pub mod cartridge;
pub mod cpu;
pub mod frame;
pub mod input;
pub mod living_room;
pub mod logger;
pub mod mapper;
pub mod ppu;
pub mod tv;

use crate::{
    cartridge::{Cartridge, ScreenMirroring},
    cpu::Cpu,
    cpu::bus::CpuBus,
    frame::Frame,
    input::ButtonState,
};

#[derive(PartialEq)]
pub enum Interrupt {
    NMI,
    IRQ,
}

impl Interrupt {
    fn address(&self) -> u16 {
        match self {
            Interrupt::NMI => 0xFFFA,
            Interrupt::IRQ => 0xFFFE,
        }
    }
}

pub enum StepKind {
    Instructrion { pc: u16, opcode: u8 },
    Interrupt(Interrupt),
}

pub struct StepResult {
    pub kind: StepKind,
    pub cpu_cycles: usize,
    pub frame_ready: bool,
}

pub struct NES {
    pub cpu: Cpu,
    pub bus: CpuBus,
}

impl Default for NES {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            bus: CpuBus::default(),
        }
    }
}

impl NES {
    pub fn frame(&self) -> &Frame {
        &self.bus.ppu.frame
    }

    pub fn step(&mut self) -> StepResult {
        let before = self.bus.total_cpu_cycles;
        let kind = self.execute_cpu_action();

        let cpu_cycles = self.bus.total_cpu_cycles - before;
        let frame_ready = std::mem::take(&mut self.bus.frame_pending);

        if frame_ready {
            self.render();
        }

        StepResult {
            kind,
            cpu_cycles,
            frame_ready,
        }
    }

    fn execute_cpu_action(&mut self) -> StepKind {
        if let Some(interrupt) = self.cpu.take_interrupt() {
            self.enter_interrupt(&interrupt);

            return StepKind::Interrupt(interrupt);
        }

        let (pc, opcode) = self.cpu.execute_next_instruction(&mut self.bus);

        StepKind::Instructrion { pc, opcode }
    }

    pub fn reset(&mut self) {
        self.bus.oam_dma_request = None;
        self.cpu.registers.accumulator = 0;
        self.cpu.registers.x = 0;
        self.cpu.registers.y = 0;
        self.cpu.registers.stack_pointer = 0xFD;

        self.cpu.registers.status.set_bits(0b0010_0100);
        self.cpu.registers.program_counter = self.cpu_read_u16(0xFFFC);

        self.bus.total_cpu_cycles = 7;
        self.bus.frame_pending = false;
        self.bus.ppu.dot = 21;
    }

    pub fn insert_cart(&mut self, mut cart: Cartridge) {
        assert!(
            cart.screen_mirroring != ScreenMirroring::FourScreen,
            "No four screen mirroring yet cuz it hard."
        );

        std::mem::swap(&mut cart.prg_ram, &mut self.bus.cartridge.prg_ram);
        self.bus.cartridge = cart;
    }

    pub fn update_buttons(&mut self, update: impl FnOnce(&mut ButtonState)) {
        update(&mut self.bus.controller.button_state);
    }

    #[cfg(test)]
    fn sample_interrupt_line(&mut self) {
        let nmi_asserted = self.bus.ppu.nmi_asserted();
        self.cpu.sample_nmi_input(nmi_asserted);
    }
}
