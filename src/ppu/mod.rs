use crate::NES;

pub trait PPU {
    fn ppu_clock(&mut self);
    fn ppu_read(&mut self, addr: u16) -> u8;
    fn ppu_write(&mut self, addr: u16, data: u8);
}

impl PPU for NES {
    fn ppu_clock(&mut self) {
        println!("PPU clocked!");
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        println!("PPU read from address: {:X}", addr);
        0
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        println!("PPU write to address: {:X} with {:X}", addr, data);
    }
}
