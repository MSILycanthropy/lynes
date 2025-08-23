pub mod cartridge;
pub mod cpu;
pub mod logger;
pub mod ppu;
pub mod renderer;

use cpu::{AddrMode, CPU};
use ppu::PPU;

use crate::{
    cartridge::ScreenMirroring,
    renderer::{palette, Frame},
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
            Interrupt::IRQ => 0xFFFB,
        }
    }
}

pub struct NES {
    // cpu
    cpu_ram: [u8; 2048],
    prg_rom: Vec<u8>,
    cpu_cycles: usize,
    clock_count: usize,
    pub cpu_registers: cpu::registers::CpuRegisters,

    // ppu
    chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    ppu_vram: [u8; 2048],
    oam_data: [u8; 256],
    mirroring: ScreenMirroring,
    ppu_cycles: usize,
    ppu_scanline: usize,
    pub ppu_registers: ppu::registers::PpuRegisters,

    // misc
    next_interrupt: Option<Interrupt>,

    current_frame: Frame,
}

impl Default for NES {
    fn default() -> Self {
        Self {
            cpu_ram: [0; 2048],
            prg_rom: vec![],
            cpu_cycles: 0,
            clock_count: 0,
            cpu_registers: cpu::registers::CpuRegisters::default(),

            chr_rom: vec![],
            palette_table: [0; 32],
            ppu_vram: [0; 2048],
            oam_data: [0; 256],
            mirroring: ScreenMirroring::Horizontal,
            ppu_cycles: 0,
            ppu_scanline: 0,
            ppu_registers: ppu::registers::PpuRegisters::default(),

            next_interrupt: None,

            current_frame: Frame::new(),
        }
    }
}

impl NES {
    pub fn start<F>(&mut self, rom_file: &str, mut render_callback: F)
    where
        F: FnMut(&Frame),
    {
        let cart = cartridge::Cartridge::load(rom_file);
        self.insert_cart(cart);
        self.reset();
        self.cpu_registers.program_counter = 0xC000;

        loop {
            if self.cpu_cycles == 0 {
                // logger::log(self);
            }

            let cycles = self.cpu_clock();

            if cycles > 0 {
                let new_frame = self.ppu_clock(cycles);

                self.try_interrupt();

                if new_frame {
                    self.render();
                    render_callback(&self.current_frame)
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.cpu_registers.accumulator = 0;
        self.cpu_registers.x = 0;
        self.cpu_registers.y = 0;
        self.cpu_registers.stack_pointer = 0xFD;

        self.cpu_registers.status.set_bits(0b0010_0100);
        self.cpu_registers.program_counter = self.cpu_read_u16(0xFFFC);

        self.cpu_cycles = 7;
        self.clock_count = 7;
        self.ppu_cycles = 21;
    }

    pub fn insert_cart(&mut self, cart: cartridge::Cartridge) {
        self.prg_rom = cart.prg_rom;
        self.chr_rom = cart.chr_rom;
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

    fn try_interrupt(&mut self) {
        if let None = self.next_interrupt {
            return;
        }

        self.perform_interrupt();

        self.next_interrupt = None;
    }

    fn perform_interrupt(&mut self) {
        self.stack_push_u16(self.cpu_registers.program_counter);
        let mut flag = self.cpu_registers.status.clone();
        flag.set_b(0b01);

        self.stack_push(*flag.into_bytes().first().unwrap());
        self.cpu_registers.status.set_interrupt_disable(true);

        self.cpu_cycles += 2;
        self.clock_count += 2;
        self.ppu_clock(2);

        let interrupt = self.next_interrupt.as_ref().unwrap();

        self.cpu_registers.program_counter = self.cpu_read_u16(interrupt.address());
    }

    fn render(&mut self) {
        let bank = self
            .ppu_registers
            .control
            .background_pattern_address_value();

        for i in 0..0x3C0 {
            let tile = self.ppu_vram[i] as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;

            let tile =
                &self.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];

            for y in 0..=7 {
                let mut high = tile[y];
                let mut low = tile[y + 8];

                for x in (0..=7).rev() {
                    let value = (1 & high) << 1 | 1 & low;

                    high = high >> 1;
                    low = low >> 1;

                    let color = match value {
                        0 => palette::SYSTEM_PALLETE[0x01],
                        1 => palette::SYSTEM_PALLETE[0x23],
                        2 => palette::SYSTEM_PALLETE[0x27],
                        3 => palette::SYSTEM_PALLETE[0x30],
                        _ => panic!("can't be"),
                    };

                    self.current_frame
                        .set_pixel(tile_x * 8 + x, tile_y * 8 + y, color);
                }
            }
        }
    }
}

fn page_crossed(old_addr: u16, new_addr: u16) -> bool {
    old_addr & 0xFF00 != new_addr & 0xFF00
}
