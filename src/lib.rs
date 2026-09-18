pub mod cartridge;
pub mod cpu;
pub mod frame;
pub mod input;
pub mod living_room;
pub mod logger;
pub mod ppu;
pub mod tv;

use cpu::{AddrMode, CPU};
use ppu::PPU;

use crate::{
    cartridge::ScreenMirroring,
    frame::Frame,
    input::{ButtonState, Controller},
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

#[derive(Default, Copy, Clone)]
struct InterruptState {
    nmi_pending: bool,
    irq_asserted: bool,
}

impl InterruptState {
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
    // cpu
    cpu_ram: [u8; 2048],
    prg_rom: Vec<u8>,
    cpu_cycles: usize,
    total_cpu_cycles: usize,
    pub cpu_registers: cpu::registers::CpuRegisters,

    // ppu
    chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    ppu_vram: [u8; 2048],
    prg_ram: Vec<u8>,

    oam_data: [u8; 256],
    mirroring: ScreenMirroring,
    ppu_dot: usize,
    ppu_scanline: usize,
    ppu_read_buffer: u8,
    pub ppu_registers: ppu::registers::PpuRegisters,
    ppu_odd_frame: bool,

    // misc
    interrupt_state: InterruptState,

    current_frame: Frame,
    controller: Controller,
}

impl Default for NES {
    fn default() -> Self {
        Self {
            cpu_ram: [0; 2048],
            prg_rom: vec![],
            prg_ram: vec![0; 8192],
            cpu_cycles: 0,
            total_cpu_cycles: 0,
            cpu_registers: cpu::registers::CpuRegisters::default(),

            chr_rom: vec![],
            palette_table: [0; 32],
            ppu_vram: [0; 2048],

            oam_data: [0; 256],
            mirroring: ScreenMirroring::Horizontal,
            ppu_dot: 0,
            ppu_scanline: 0,
            ppu_read_buffer: 0,
            ppu_registers: ppu::registers::PpuRegisters::default(),
            ppu_odd_frame: false,

            interrupt_state: InterruptState::default(),

            current_frame: Frame::new(),
            controller: Controller::new(),
        }
    }
}

impl NES {
    /// The most recently rendered RGB frame.
    pub fn frame(&self) -> &Frame {
        &self.current_frame
    }

    pub fn step(&mut self) -> StepResult {
        let (kind, cpu_cycles) = self.execute_cpu_action();
        let frame_ready = self.advance_cpu_cycles(cpu_cycles);

        if frame_ready {
            self.render();
        }

        StepResult {
            kind,
            cpu_cycles,
            frame_ready,
        }
    }

    fn advance_cpu_cycles(&mut self, cpu_cycles: usize) -> bool {
        let mut frame_ready = false;

        for _ in 0..cpu_cycles {
            self.total_cpu_cycles += 1;

            for _ in 0..3 {
                frame_ready |= self.tick_ppu();
            }
        }

        frame_ready
    }

    fn execute_cpu_action(&mut self) -> (StepKind, usize) {
        let interrupt_disable = self.cpu_registers.status.interrupt_disable();

        if let Some(interrupt) = self.interrupt_state.take(interrupt_disable) {
            let cycles = self.enter_interrupt(&interrupt);

            return (StepKind::Interrupt(interrupt), cycles);
        }

        let (pc, opcode, cycles) = self.execute_next_instruction();

        (StepKind::Instructrion { pc, opcode }, cycles)
    }

    pub fn reset(&mut self) {
        self.cpu_registers.accumulator = 0;
        self.cpu_registers.x = 0;
        self.cpu_registers.y = 0;
        self.cpu_registers.stack_pointer = 0xFD;

        self.cpu_registers.status.set_bits(0b0010_0100);
        self.cpu_registers.program_counter = self.cpu_read_u16(0xFFFC);

        self.cpu_cycles = 7;
        self.total_cpu_cycles = 7;
        self.ppu_dot = 21;
    }

    pub fn insert_cart(&mut self, cart: cartridge::Cartridge) {
        assert!(
            cart.screen_mirroring != ScreenMirroring::FourScreen,
            "No four screen mirroring yet cuz it hard."
        );

        self.mirroring = cart.screen_mirroring;

        self.prg_rom = cart.prg_rom;
        self.chr_rom = cart.chr_rom;
    }

    pub fn set_buttons(&mut self, buttons: ButtonState) {
        self.controller.button_state = buttons;
    }

    // Returns the address and if a page boundary was crossed
    pub fn get_operating_address(&mut self, mode: &AddrMode) -> (u16, bool) {
        match mode {
            AddrMode::Implied => {
                panic!("Implied addressing mode has no operating address as it is implied")
            }
            AddrMode::Accumulator => panic!(
                "Accumulator addressing mode has no operating address as it operates on the accumulator"
            ),
            AddrMode::Immediate => {
                let addr = self.cpu_registers.program_counter;
                (addr, false)
            }
            _ => self.get_absolute_address(self.cpu_registers.program_counter, mode),
        }
    }

    pub fn get_absolute_address(&mut self, addr: u16, mode: &AddrMode) -> (u16, bool) {
        match mode {
            AddrMode::ZeroPage => {
                let addr = self.cpu_read(addr) as u16;
                (addr, false)
            }
            AddrMode::ZeroPageX => {
                let addr = self.cpu_read(addr).wrapping_add(self.cpu_registers.x) as u16;
                (addr, false)
            }
            AddrMode::ZeroPageY => {
                let addr = self.cpu_read(addr).wrapping_add(self.cpu_registers.y) as u16;
                (addr, false)
            }
            AddrMode::Relative => {
                let offset = self.cpu_read(addr) as u16;
                let old_addr = addr;
                let addr = old_addr.wrapping_add(1).wrapping_add(offset);

                (addr, page_crossed(old_addr, addr))
            }
            AddrMode::Absolute => {
                let addr = self.cpu_read_u16(addr);
                (addr, false)
            }
            AddrMode::AbsoluteX => {
                let old_addr = self.cpu_read_u16(addr);
                let addr = old_addr.wrapping_add(self.cpu_registers.x as u16);

                (addr, page_crossed(old_addr, addr))
            }
            AddrMode::AbsoluteY => {
                let old_addr = self.cpu_read_u16(addr);
                let addr = old_addr.wrapping_add(self.cpu_registers.y as u16);

                (addr, page_crossed(old_addr, addr))
            }
            AddrMode::Indirect => {
                let old_addr = self.cpu_read_u16(addr);

                let addr = if old_addr & 0x00FF == 0x00FF {
                    let low = self.cpu_read(old_addr);
                    let high = self.cpu_read(old_addr & 0xFF00);

                    u16::from_le_bytes([low, high])
                } else {
                    self.cpu_read_u16(old_addr)
                };

                (addr, false)
            }
            AddrMode::IndirectX => {
                let zero_page_addr = self.cpu_read(addr);
                let pointer = zero_page_addr.wrapping_add(self.cpu_registers.x);
                let low = self.cpu_read(pointer as u16);
                let high = self.cpu_read(pointer.wrapping_add(1) as u16);

                let addr = u16::from_le_bytes([low, high]);

                (addr, false)
            }
            AddrMode::IndirectY => {
                let zero_page_addr = self.cpu_read(addr);
                let low = self.cpu_read(zero_page_addr as u16);
                let high = self.cpu_read(zero_page_addr.wrapping_add(1) as u16);

                let old_addr = u16::from_le_bytes([low, high]);
                let addr = old_addr.wrapping_add(self.cpu_registers.y as u16);

                (addr, page_crossed(old_addr, addr))
            }
            _ => panic!("Invalid absolute addressing mode"),
        }
    }
}

fn page_crossed(old_addr: u16, new_addr: u16) -> bool {
    old_addr & 0xFF00 != new_addr & 0xFF00
}
