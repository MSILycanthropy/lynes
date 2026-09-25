use crate::{
    frame::Frame,
    ppu::{bus::PpuBus, registers::PpuRegisters},
};

pub(crate) mod bus;
mod palette;
pub(crate) mod registers;
mod render;

#[cfg(test)]
mod tests;

pub struct Ppu {
    io_latch: u8,
    supress_vblank: bool,

    pub(crate) registers: PpuRegisters,
    pub(crate) palette_table: [u8; 32],
    pub(crate) oam_data: [u8; 256],
    pub(crate) dot: usize,
    pub(crate) scanline: usize,
    pub(crate) read_buffer: u8,
    pub(crate) odd_frame: bool,
    pub(crate) frame: Frame,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            io_latch: 0,
            supress_vblank: false,

            registers: PpuRegisters::default(),
            palette_table: [0; 32],
            oam_data: [0; 256],
            dot: 0,
            scanline: 0,
            read_buffer: 0,
            odd_frame: false,
            frame: Frame::default(),
        }
    }
}

impl Ppu {
    pub(crate) fn tick(&mut self, bus: &PpuBus<'_>) -> bool {
        let rendering_enabled =
            self.registers.mask.show_background() || self.registers.mask.show_sprite();

        // TODO: Regions so that we dont hardcode NTSC
        let frame_ready = match (self.scanline, self.dot) {
            (261, 339) if self.odd_frame && rendering_enabled => {
                self.scanline = 0;
                self.dot = 0;
                true
            }
            (261, 340) => {
                self.scanline = 0;
                self.dot = 0;
                true
            }
            (_, 340) => {
                self.scanline += 1;
                self.dot = 0;
                false
            }

            _ => {
                self.dot += 1;

                false
            }
        };

        if frame_ready {
            self.odd_frame = !self.odd_frame;
        }

        match (self.scanline, self.dot) {
            (241, 1) => {
                if !self.supress_vblank {
                    self.registers.status.set_vblank_started(true);
                }

                self.supress_vblank = false
            }
            (261, 1) => {
                self.registers.status.set_vblank_started(false);
                self.registers.status.set_sprite_zero_hit(false);
                self.registers.status.set_sprite_overflow(false);
            }
            _ => {}
        }

        self.check_sprite_zero_hit(bus);

        return frame_ready;
    }

    pub(crate) fn write_address(&mut self, data: u8) {
        self.io_latch = data;
        self.registers.scroll.write_address(data);
    }

    pub(crate) fn write_control(&mut self, data: u8) {
        self.io_latch = data;

        self.registers.control.update(data);
        self.registers.scroll.write_control(data);
    }

    pub(crate) fn write_data(&mut self, bus: &mut PpuBus<'_>, value: u8) {
        self.io_latch = value;

        let address = self.registers.scroll.memory_address();

        match address {
            0..=0x3EFF => bus.write(address, value),
            0x3F00..=0x3FFF => {
                self.palette_table[palette_index(address)] = value & 0x3F;
            }
            _ => panic!("unexpected ppu write to {address:#06X}"),
        }

        self.registers.increment_vram_address();
    }

    pub(crate) fn write_mask(&mut self, data: u8) {
        self.io_latch = data;
        self.registers.mask.update(data);
    }

    pub(crate) fn write_scroll(&mut self, data: u8) {
        self.io_latch = data;
        self.registers.scroll.write_scroll(data);
    }

    pub(crate) fn write_oam_address(&mut self, data: u8) {
        self.io_latch = data;
        self.registers.oam_addr = data;
    }

    pub(crate) fn write_oam_data(&mut self, data: u8) {
        self.io_latch = data;
        self.oam_data[self.registers.oam_addr as usize] = data;
        self.registers.oam_addr = self.registers.oam_addr.wrapping_add(1);
    }

    pub(crate) fn write_status(&mut self, data: u8) {
        // Writing PPUSTATUS affects the I/O latch, but not the status flags.
        self.io_latch = data;
    }

    pub(crate) fn read_data(&mut self, bus: &mut PpuBus<'_>) -> u8 {
        let address = self.registers.scroll.memory_address();
        let result = self.peek_data();

        self.registers.increment_vram_address();

        match address {
            0..=0x3EFF => {
                self.read_buffer = bus.read(address);
            }
            0x3F00..=0x3FFF => {
                self.read_buffer = bus.read(address - 0x1000);
            }
            _ => panic!("unexpected ppu read at {address:#06X}"),
        }

        self.io_latch = result;

        result
    }

    pub(crate) fn peek_data(&self) -> u8 {
        let address = self.registers.scroll.memory_address();

        match address {
            0..=0x3EFF => self.read_buffer,
            0x3F00..=0x3FFF => {
                let mask = if self.registers.mask.greyscale() {
                    0x30
                } else {
                    0x3F
                };

                let color = self.palette_table[palette_index(address)] & mask;

                (self.io_latch & 0xC0) | color
            }
            _ => panic!("unexpected ppu peek at {address:#06X}"),
        }
    }

    pub(crate) fn read_status(&mut self) -> u8 {
        if self.scanline == 241 && self.dot == 0 {
            self.supress_vblank = true;
        }

        let data = self.peek_status();

        self.io_latch = data;
        self.registers.status.set_vblank_started(false);
        self.registers.scroll.reset_write_toggle();

        data
    }

    pub(crate) fn peek_status(&self) -> u8 {
        (self.registers.status.into_bits() & 0xE0) | (self.io_latch & 0x1F)
    }

    pub(crate) fn read_oam_data(&mut self) -> u8 {
        let value = self.peek_oam_data();
        self.io_latch = value;
        value
    }

    pub(crate) fn peek_oam_data(&self) -> u8 {
        self.oam_data[self.registers.oam_addr as usize]
    }

    pub(crate) fn peek_io_latch(&self) -> u8 {
        self.io_latch
    }

    pub(crate) fn nmi_asserted(&self) -> bool {
        self.registers.control.generate_nmi() && self.registers.status.vblank_started()
    }

    fn check_sprite_zero_hit(&mut self, bus: &PpuBus<'_>) {
        if self.registers.status.sprite_zero_hit()
            || self.scanline >= 240
            || !(1..=255).contains(&self.dot)
        {
            return;
        }

        let mask = &self.registers.mask;
        if !mask.show_background() || !mask.show_sprite() {
            return;
        }

        let x = self.dot - 1;
        let y = self.scanline;

        if x < 8 && (!mask.leftmost_8px_background() || !mask.leftmost_8px_sprite()) {
            return;
        }

        if self.sprite_zero_opaque_at(bus, x, y) && self.background_opaque_at(bus, x, y) {
            self.registers.status.set_sprite_zero_hit(true);
        }
    }

    fn background_palette(&self, attribute_table: &[u8], tile_x: usize, tile_y: usize) -> [u8; 4] {
        let attribute_table_index = tile_y / 4 * 8 + tile_x / 4;
        let attibute_table_value = attribute_table[attribute_table_index];

        let palette_table_index = match (tile_x % 4 / 2, tile_y % 4 / 2) {
            (0, 0) => (attibute_table_value >> 0) & 0b11,
            (1, 0) => (attibute_table_value >> 2) & 0b11,
            (0, 1) => (attibute_table_value >> 4) & 0b11,
            (1, 1) => (attibute_table_value >> 6) & 0b11,
            _ => unreachable!(),
        } as usize;
        let palette_start = palette_table_index * 4 + 1;

        [
            self.palette_table[0],
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    fn background_opaque_at(&self, bus: &PpuBus<'_>, x: usize, y: usize) -> bool {
        let scroll = &self.registers.scroll;
        let mut nametable = scroll.name_table_address();

        let scrolled_x = x + usize::from(scroll.scroll_x());
        if scrolled_x >= 256 {
            nametable ^= 0x0400;
        }
        let tile_x = (scrolled_x / 8) & 31;

        let scroll_y = usize::from(scroll.scroll_y());
        let pixel_y = (scroll_y & 7) + y;
        let mut tile_y = scroll_y / 8;

        for _ in 0..pixel_y / 8 {
            match tile_y {
                29 => {
                    tile_y = 0;
                    nametable ^= 0x0800;
                }
                31 => tile_y = 0,
                _ => tile_y += 1,
            }
        }

        let tile_address = nametable + (tile_y * 32 + tile_x) as u16;
        let tile = u16::from(bus.read(tile_address));

        let address = self.registers.control.background_pattern_address_value()
            + tile * 16
            + (pixel_y & 7) as u16;

        let low = bus.read(address);
        let high = bus.read(address + 8);
        let bit = 7 - (scrolled_x & 7);

        ((low | high) & (1u8 << bit)) != 0
    }

    fn sprite_palette(&self, index: usize) -> [u8; 4] {
        let palette_index = self.oam_data[index + 2] & 0b11;
        let palette_start = 0x11 + (palette_index * 4) as usize;

        [
            0,
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    fn sprite_zero_opaque_at(&self, bus: &PpuBus<'_>, x: usize, y: usize) -> bool {
        let top = usize::from(self.oam_data[0]) + 1;
        let tile = u16::from(self.oam_data[1]);
        let attributes = self.oam_data[2];
        let left = usize::from(self.oam_data[3]);
        let height = if self.registers.control.sprite_size() {
            16
        } else {
            8
        };

        if x < left || x >= left + 8 || y < top || y >= top + height {
            return false;
        }

        let mut column = x - left;
        let mut row = y - top;

        if attributes & 0x40 != 0 {
            column = 7 - column;
        }
        if attributes & 0x80 != 0 {
            row = height - 1 - row;
        }

        let address = if height == 16 {
            let bank = (tile & 1) * 0x1000;
            let tile = (tile & !1) + (row / 8) as u16;
            bank + tile * 16 + (row % 8) as u16
        } else {
            self.registers.control.sprite_pattern_address_value() + tile * 16 + row as u16
        };

        let low = bus.read(address);
        let high = bus.read(address + 8);
        let bit = 7 - column;

        ((low | high) & (1u8 << bit)) != 0
    }
}

fn palette_index(address: u16) -> usize {
    let index = (address & 0x1F) as usize;

    match index {
        0x10 | 0x14 | 0x18 | 0x1C => index - 0x10,
        _ => index,
    }
}
