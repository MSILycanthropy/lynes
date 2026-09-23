use crate::{
    cartridge::Cartridge,
    input::Controller,
    ppu::{Ppu, bus::PpuBus},
};

pub enum WriteEffect {
    None,
    OamDma { page: u8 },
}

pub struct CpuBus {
    pub(crate) ram: [u8; 2048],
    pub(crate) ppu: Ppu,
    pub(crate) ciram: [u8; 2048],
    pub(crate) controller: Controller,
    pub(crate) cartridge: Cartridge,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 2048],
            ppu: Ppu::default(),
            ciram: [0; 2048],
            controller: Controller::new(),
            cartridge: Cartridge::default(),
        }
    }
}

impl CpuBus {
    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => {
                let mirrored_address = address & 0b00000111_11111111;

                self.ram[mirrored_address as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("attempted to read from write-only PPU address {address:#06X}");
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => {
                let mut bus = PpuBus {
                    cartridge: &mut self.cartridge,
                    ciram: &mut self.ciram,
                };

                self.ppu.read_data(&mut bus)
            }
            0x4000..=0x4015 => {
                // panic!("APU and I/O registers are not implemented yet!")
                0
            }
            0x4016 => self.controller.read(),
            0x4017 => 0,
            0x2008..=0x3FFF => {
                let mirrored_address = address & 0b00100000_00000111;
                self.read(mirrored_address)
            }
            0x4020..=0xFFFF => self
                .cartridge
                .cpu_read(address)
                .unwrap_or_else(|| panic!("Invalid CPU read address: {address:#06X}")),
            _ => {
                panic!("Invalid CPU read address: {address:#06X}");
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) -> WriteEffect {
        match address {
            0x0000..=0x1FFF => {
                let mirrored_address = address & 0b00000111_11111111;

                self.ram[mirrored_address as usize] = value;
            }
            0x2000 => self.ppu.write_control(value),
            0x2001 => self.ppu.write_mask(value),
            0x2002 => {} // Writes dont change PPUSTATUS, but we do have tests that.. well test that.
            0x2003 => self.ppu.write_oam_address(value),
            0x2004 => self.ppu.write_oam_data(value),
            0x2005 => self.ppu.write_scroll(value),
            0x2006 => self.ppu.write_address(value),
            0x2007 => {
                let mut bus = PpuBus {
                    cartridge: &mut self.cartridge,
                    ciram: &mut self.ciram,
                };

                self.ppu.write_data(&mut bus, value)
            }
            0x2008..=0x3FFF => {
                let mirrored_address = address & 0b00100000_00000111;
                return self.write(mirrored_address, value);
            }
            0x4014 => return WriteEffect::OamDma { page: value },
            0x4000..=0x4015 => {
                // panic!("APU and I/O registers are not implemented yet!")
            }
            0x4016 => self.controller.write(value),
            0x4017 => {
                // ignore controller 2
            }
            0x4018..=0x401F => {
                // panic!("APU and I/O functionality that is normally disabled")
            }
            0x4020..=0xFFFF => self.cartridge.cpu_write(address, value),
        }

        WriteEffect::None
    }

    pub(crate) fn nmi_asserted(&self) -> bool {
        self.ppu.nmi_asserted()
    }
}
