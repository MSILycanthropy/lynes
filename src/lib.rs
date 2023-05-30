mod cartridge;
mod cpu;
mod interrupt;
pub mod logger;
mod ppu;
pub mod renderer;
pub mod color;

use cpu::{AddrMode, CPU};
use interrupt::Interrupt;
use ppu::PPU;
use color::{ColorPalette};
use renderer::Frame;

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenMirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

#[derive(Debug, Clone)]
pub enum NmiStatus {
    None,
    Triggered,
}

#[derive(Clone)]
pub struct NES {
    cpu_ram: [u8; 2048],
    pub ppu_vram: [u8; 2048],

    prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,

    pub palette_table: [u8; 32],
    pub color_palette: ColorPalette,

    pub oam_data: [u8; 256],

    ppu_buffer_data: u8,

    // mapper: cartridge::mapper::Mapper, // TODO: Implement mappers
    pub cpu_registers: cpu::registers::CpuRegisters,
    pub ppu_registers: ppu::registers::PpuRegisters,

    pub cpu_cycle: usize,
    cpu_total_cycles: usize,

    ppu_cycle: usize,
    ppu_scanline: usize,

    pub total_cycles: usize,

    screen_mirroring: ScreenMirroring,

    nmi_status: NmiStatus,

    pub current_frame: Frame,

    pub should_render: bool,
}

impl NES {
    fn new(color_palette: ColorPalette) -> Self {
        Self {
            cpu_ram: [0; 2048],
            ppu_vram: [0; 2048],

            prg_rom: vec![],
            chr_rom: vec![],

            color_palette,
            palette_table: [0; 32],
            oam_data: [0; 256],

            ppu_buffer_data: 0,

            cpu_registers: cpu::registers::CpuRegisters::default(),
            ppu_registers: ppu::registers::PpuRegisters::default(),

            cpu_cycle: 0,
            cpu_total_cycles: 0,

            ppu_cycle: 0,
            ppu_scanline: 0,

            total_cycles: 0,

            screen_mirroring: ScreenMirroring::Horizontal,

            nmi_status: NmiStatus::None,

            current_frame: Frame::new(),

            should_render: false,
        }
    }
}

impl NES {
    pub fn clock(&mut self) {
        self.try_nmi();

        let cycles = self.cpu_clock();

        self.should_render = self.ppu_clock(cycles * 3);

        self.total_cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu_registers.accumulator = 0;
        self.cpu_registers.x = 0;
        self.cpu_registers.y = 0;
        self.cpu_registers.stack_pointer = 0xFD;

        self.cpu_registers.status.set_bits(0b0010_0100);

        self.cpu_registers.program_counter = self.cpu_read_u16(0xFFFC);

        self.cpu_cycle = 7;
        self.cpu_total_cycles = 7;
    }

    pub fn insert_cart(&mut self, cart: cartridge::Cartridge) {
        self.prg_rom = cart.prg_rom;
        self.chr_rom = cart.chr_rom;
        self.screen_mirroring = cart.screen_mirroring;
    }

    fn try_nmi(&mut self) {
        match self.nmi_status {
            NmiStatus::Triggered => {
                self.nmi_status = NmiStatus::None;
                self.nmi();
            }
            _ => {}
        }
    }

    // Returns the address and if a page boundary was crossed
    pub fn get_operating_address(&mut self, mode: &AddrMode) -> (u16, bool) {
        match mode {
            AddrMode::Implied => panic!("Implied addressing mode has no operating address as it is implied"),
            AddrMode::Accumulator => panic!("Accumulator addressing mode has no operating address as it operates on the accumulator"),
            AddrMode::Immediate => {
                let addr = self.cpu_registers.program_counter;
                (addr, false)
            },
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

pub struct NESBuilder {
    rom: String,
    color_palette: String,
}

impl NESBuilder {
    pub fn new() -> NESBuilder {
        NESBuilder {
            rom: String::new(),
            color_palette: String::new(),
        }
    }

    pub fn rom(mut self, rom: &str) -> NESBuilder {
        self.rom = rom.to_string();
        self
    }

    pub fn color_palette(mut self, color_palette: &str) -> NESBuilder {
        self.color_palette = color_palette.to_string();
        self
    }

    pub fn build(self) -> NES {
        let palette = ColorPalette::load(&self.color_palette);
        let mut nes = NES::new(palette);
        nes.insert_cart(cartridge::Cartridge::load(&self.rom));

        nes
    }
}

fn page_crossed(old_addr: u16, new_addr: u16) -> bool {
    old_addr & 0xFF00 != new_addr & 0xFF00
}
