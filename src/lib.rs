mod cartridge;
mod cpu;
mod logger;
mod ppu;

use cpu::{AddrMode, CPU};
use ppu::PPU;

use crate::cartridge::ScreenMirroring;

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
    pub ppu_registers: ppu::registers::PpuRegisters,
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
            ppu_registers: ppu::registers::PpuRegisters::default(),
        }
    }
}

impl NES {
    pub fn start(&mut self, rom_file: &str) {
        let cart = cartridge::Cartridge::load(rom_file);
        self.insert_cart(cart);
        self.reset();
        self.cpu_registers.program_counter = 0xC000;

        loop {
            if self.cpu_cycles == 0 {
                logger::log(self);
            }

            self.cpu_clock();
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
}

fn page_crossed(old_addr: u16, new_addr: u16) -> bool {
    old_addr & 0xFF00 != new_addr & 0xFF00
}
