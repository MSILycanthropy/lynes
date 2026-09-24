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
    data_bus: u8,
    pub(crate) total_cpu_cycles: usize,
    pub(crate) frame_pending: bool,
    pub(crate) ram: [u8; 2048],
    pub(crate) ppu: Ppu,
    pub(crate) ciram: [u8; 2048],
    pub(crate) controller: Controller,
    pub(crate) cartridge: Cartridge,
    pub(crate) oam_dma_request: Option<u8>,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            data_bus: 0,
            total_cpu_cycles: 0,
            frame_pending: false,
            ram: [0; 2048],
            ppu: Ppu::default(),
            ciram: [0; 2048],
            controller: Controller::new(),
            cartridge: Cartridge::default(),
            oam_dma_request: None,
        }
    }
}

impl CpuBus {
    pub fn read(&mut self, address: u16) -> u8 {
        let value = match address {
            0x0000..=0x1FFF => {
                let mirrored_address = address & 0b00000111_11111111;

                self.ram[mirrored_address as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.ppu.peek_io_latch(),
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => {
                let mut bus = PpuBus {
                    cartridge: &mut self.cartridge,
                    ciram: &mut self.ciram,
                };

                self.ppu.read_data(&mut bus)
            }
            0x4000..=0x4014 => self.data_bus,
            0x4015 => self.data_bus & 0x20,
            0x4016 => (self.data_bus & 0xE0) | self.controller.read(),
            0x4017 => self.data_bus & 0xE0,
            0x4018..=0x401F => self.data_bus,
            0x2008..=0x3FFF => {
                let mirrored_address = address & 0b00100000_00000111;
                self.read(mirrored_address)
            }
            0x4020..=0xFFFF => self.cartridge.cpu_read(address).unwrap_or(self.data_bus),
        };

        if address != 0x4015 {
            self.data_bus = value;
        }

        value
    }

    /// Returns the current read value without changing device state or advancing time.
    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.ppu.peek_io_latch(),
            0x2002 => self.ppu.peek_status(),
            0x2004 => self.ppu.peek_oam_data(),
            0x2007 => self.ppu.peek_data(),
            0x2008..=0x3FFF => self.peek(address & 0x2007),
            0x4000..=0x4014 => self.data_bus,
            0x4015 => self.data_bus & 0x20,
            0x4016 => (self.data_bus & 0xE0) | self.controller.peek(),
            0x4017 => self.data_bus & 0xE0,
            0x4018..=0x401F => self.data_bus,
            0x4020..=0xFFFF => self.cartridge.cpu_read(address).unwrap_or(self.data_bus),
        }
    }

    pub fn peek_u16(&self, address: u16) -> u16 {
        let low = self.peek(address);
        let high = self.peek(address.wrapping_add(1));

        u16::from_le_bytes([low, high])
    }

    pub fn write(&mut self, address: u16, value: u8) -> WriteEffect {
        self.data_bus = value;

        match address {
            0x0000..=0x1FFF => {
                let mirrored_address = address & 0b00000111_11111111;

                self.ram[mirrored_address as usize] = value;
            }
            0x2000 => self.ppu.write_control(value),
            0x2001 => self.ppu.write_mask(value),
            0x2002 => self.ppu.write_status(value),
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

#[cfg(test)]
mod tests {
    use super::CpuBus;

    #[test]
    fn ppu_peeks_preserve_status_scroll_and_data_buffer() {
        let mut bus = CpuBus::default();
        bus.ppu.registers.status.set_vblank_started(true);
        bus.write(0x2005, 0x12);
        assert_eq!(bus.peek(0x2002), 0x92);
        assert_eq!(bus.peek(0x3FFA), 0x92);
        bus.write(0x2005, 0x34);
        assert_eq!(bus.ppu.registers.scroll.scroll_x(), 0x12);
        assert_eq!(bus.ppu.registers.scroll.scroll_y(), 0x34);
        assert_eq!(bus.read(0x2002), 0x94);
        assert_eq!(bus.peek(0x2002), 0x14);

        bus.write(0x2006, 0x20);
        bus.write(0x2006, 0x00);
        bus.ppu.read_buffer = 0xAB;
        bus.ciram[0] = 0xCD;
        assert_eq!(bus.peek(0x2007), 0xAB);
        assert_eq!(bus.peek(0x3FFF), 0xAB);
        assert_eq!(bus.ppu.registers.scroll.memory_address(), 0x2000);
        assert_eq!(bus.read(0x2007), 0xAB);
        assert_eq!(bus.peek(0x2007), 0xCD);

        bus.write(0x2006, 0x3F);
        bus.write(0x2006, 0x10);
        bus.ppu.palette_table[0] = 0x23;
        assert_eq!(bus.peek(0x2007), 0x23);
        assert_eq!(bus.ppu.read_buffer, 0xCD);
        assert_eq!(bus.ppu.registers.scroll.memory_address(), 0x3F10);
    }

    #[test]
    fn peeks_use_memory_mapping_and_wrap_u16_reads() {
        let mut bus = CpuBus::default();
        bus.cartridge.prg_rom = vec![0; 0x4000];
        bus.cartridge.prg_rom[0x3FFF] = 0x34;
        bus.write(0x0000, 0x12);
        assert_eq!(bus.peek(0x1800), 0x12);
        assert_eq!(bus.peek(0xBFFF), 0x34);
        assert_eq!(bus.peek_u16(0xFFFF), 0x1234);
    }

    #[test]
    fn cpu_open_bus_tracks_accesses_but_not_peeks_or_apu_status_reads() {
        let mut bus = CpuBus::default();
        bus.write(0, 0x56);
        bus.write(0x4000, 0xAB); // Even an ignored write drives the bus.
        assert_eq!(bus.peek(0), 0x56);

        for address in [0x4000, 0x4014, 0x4018, 0x401F, 0x4020, 0x5FFF] {
            assert_eq!(bus.peek(address), 0xAB);
            assert_eq!(bus.read(address), 0xAB);
        }

        assert_eq!(bus.peek(0x4015), 0x20);
        assert_eq!(bus.read(0x4015), 0x20);
        assert_eq!(bus.read(0x4014), 0xAB);
        assert_eq!(bus.read(0), 0x56);
        assert_eq!(bus.read(0x4014), 0x56);

        bus.controller.button_state.set_a(true);
        bus.write(0x4000, 0xA5);
        assert_eq!(bus.peek(0x4016), 0xA1);
        assert_eq!(bus.peek(0x4017), 0xA0);
        assert_eq!(bus.peek(0x4014), 0xA5);
        assert_eq!(bus.read(0x4016), 0xA1);
        assert_eq!(bus.read(0x4014), 0xA1);
        assert_eq!(bus.read(0x4016), 0xA0); // B is not pressed.
        assert_eq!(bus.read(0x4017), 0xA0); // Second port is disconnected.
    }

    #[test]
    fn cpu_open_bus_and_ppu_io_latch_are_independent() {
        let mut bus = CpuBus::default();
        bus.write(0x2001, 0xAB);
        bus.write(0, 0x56);

        for address in [0x2000, 0x2001, 0x2003, 0x2005, 0x2006, 0x3FFD] {
            assert_eq!(bus.peek(address), 0xAB);
        }
        assert_eq!(bus.read(0x4014), 0x56);
        assert_eq!(bus.read(0x2000), 0xAB);
        assert_eq!(bus.read(0x4014), 0xAB); // PPU read drives the CPU bus.

        assert_eq!(bus.read(0), 0x56);
        assert_eq!(bus.peek(0x2000), 0xAB); // RAM read leaves the PPU latch alone.
        bus.write(0x3FF9, 0xC2); // Mirrored PPUMASK write updates both.
        assert_eq!(bus.peek(0x2000), 0xC2);
        assert_eq!(bus.peek(0x4014), 0xC2);
    }
}
