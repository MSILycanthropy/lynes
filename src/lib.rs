pub mod cartridge;
pub mod cpu;
pub mod input;
pub mod logger;
pub mod ppu;
pub mod renderer;

use cpu::{AddrMode, CPU};
use ppu::PPU;

use crate::{
    cartridge::ScreenMirroring,
    input::Controller,
    renderer::{Frame, ViewPortRect, palette},
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
    oam_data: [u8; 256],
    mirroring: ScreenMirroring,
    ppu_cycles: usize,
    ppu_scanline: usize,
    ppu_read_buffer: u8,
    pub ppu_registers: ppu::registers::PpuRegisters,

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
            cpu_cycles: 0,
            total_cpu_cycles: 0,
            cpu_registers: cpu::registers::CpuRegisters::default(),

            chr_rom: vec![],
            palette_table: [0; 32],
            ppu_vram: [0; 2048],
            oam_data: [0; 256],
            mirroring: ScreenMirroring::Horizontal,
            ppu_cycles: 0,
            ppu_scanline: 0,
            ppu_read_buffer: 0,
            ppu_registers: ppu::registers::PpuRegisters::default(),

            interrupt_state: InterruptState::default(),

            current_frame: Frame::new(),
            controller: Controller::new(),
        }
    }
}

impl NES {
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
            frame_ready |= self.ppu_clock(1);
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
        self.ppu_cycles = 21;
    }

    pub fn insert_cart(&mut self, cart: cartridge::Cartridge) {
        self.prg_rom = cart.prg_rom;
        self.chr_rom = cart.chr_rom;
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

    fn render(&mut self) {
        self.render_background();
        self.render_sprites();
    }

    fn render_background(&mut self) {
        let scroll_x = self.ppu_registers.scroll.scroll_x as usize;
        let scroll_y = self.ppu_registers.scroll.scroll_y as usize;

        let (first_nametable, second_nametable) = match (
            self.mirroring.clone(),
            self.ppu_registers.control.name_table_address(),
        ) {
            (ScreenMirroring::Vertical, 0x2000)
            | (ScreenMirroring::Vertical, 0x2800)
            | (ScreenMirroring::Horizontal, 0x2000)
            | (ScreenMirroring::Horizontal, 0x2400) => (
                &self.ppu_vram.clone()[0..0x400],
                &self.ppu_vram.clone()[0x400..0x800],
            ),
            (ScreenMirroring::Vertical, 0x2400)
            | (ScreenMirroring::Vertical, 0x2C00)
            | (ScreenMirroring::Horizontal, 0x2800)
            | (ScreenMirroring::Horizontal, 0x2C00) => (
                &self.ppu_vram.clone()[0x400..0x800],
                &self.ppu_vram.clone()[0..0x400],
            ),
            (_, _) => {
                panic!("Not supported mirroring type {:?}", self.mirroring);
            }
        };

        self.render_name_table(
            first_nametable,
            ViewPortRect::new(scroll_x, scroll_y, 256, 240),
            -(scroll_x as isize),
            -(scroll_y as isize),
        );

        if scroll_x > 0 {
            self.render_name_table(
                second_nametable,
                ViewPortRect::new(0, 0, scroll_x, 240),
                (256 - scroll_x) as isize,
                0,
            );
        } else if scroll_y > 0 {
            self.render_name_table(
                second_nametable,
                ViewPortRect::new(0, 0, 256, scroll_y),
                0,
                (240 - scroll_y) as isize,
            );
        }
    }

    fn render_name_table(
        &mut self,
        name_table: &[u8],
        view_port: ViewPortRect,
        shift_x: isize,
        shift_y: isize,
    ) {
        let bank = self
            .ppu_registers
            .control
            .background_pattern_address_value();
        let attribute_table = &name_table[0x3C0..0x400];

        for i in 0..0x3C0 {
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile = name_table[i] as u16;
            let tile =
                &self.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
            let palette = self.background_palette(attribute_table, tile_x, tile_y);

            for y in 0..=7 {
                let mut high = tile[y];
                let mut low = tile[y + 8];

                for x in (0..=7).rev() {
                    let value = (1 & low) << 1 | 1 & high;

                    high = high >> 1;
                    low = low >> 1;

                    let color = palette::SYSTEM_PALLETE[palette[value as usize] as usize];

                    let pixel_x = tile_x * 8 + x;
                    let pixel_y = tile_y * 8 + y;

                    if view_port.point_is_bounded(pixel_x, pixel_y) {
                        self.current_frame.set_pixel(
                            (shift_x + pixel_x as isize) as usize,
                            (shift_y + pixel_y as isize) as usize,
                            color,
                        );
                    }
                }
            }
        }
    }

    fn render_sprites(&mut self) {
        for i in (0..self.oam_data.len()).step_by(4).rev() {
            let tile = self.oam_data[i + 1] as u16;
            let tile_x = self.oam_data[i + 3] as usize;
            let tile_y = self.oam_data[i] as usize;

            let flip_vertical = self.oam_data[i + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam_data[i + 2] >> 6 & 1 == 1;

            let palette = self.sprite_palette(i);

            let bank = self.ppu_registers.control.sprite_pattern_address_value();

            let tile =
                &self.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];

            for y in 0..=7 {
                let mut high = tile[y];
                let mut low = tile[y + 8];

                'inner: for x in (0..=7).rev() {
                    let value = (1 & low) << 1 | 1 & high;

                    high = high >> 1;
                    low = low >> 1;

                    if value == 0 {
                        continue 'inner;
                    }

                    let color = palette::SYSTEM_PALLETE[palette[value as usize] as usize];

                    match (flip_horizontal, flip_vertical) {
                        (false, false) => {
                            self.current_frame.set_pixel(tile_x + x, tile_y + y, color)
                        }
                        (true, false) => {
                            self.current_frame
                                .set_pixel(tile_x + 7 - x, tile_y + y, color)
                        }
                        (false, true) => {
                            self.current_frame
                                .set_pixel(tile_x + x, tile_y + 7 - y, color)
                        }
                        (true, true) => {
                            self.current_frame
                                .set_pixel(tile_x + 7 - x, tile_y + 7 - y, color)
                        }
                    }
                }
            }
        }
    }
}

fn page_crossed(old_addr: u16, new_addr: u16) -> bool {
    old_addr & 0xFF00 != new_addr & 0xFF00
}
