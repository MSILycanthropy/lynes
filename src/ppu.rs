use crate::{NES, cartridge::ScreenMirroring, frame::Frame, ppu::registers::PpuRegisters};

mod palette;
pub(crate) mod registers;
mod render;

#[cfg(test)]
mod tests;

pub struct Ppu {
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
    pub(crate) fn tick(&mut self) -> bool {
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
                self.registers.status.set_vblank_started(true);
            }
            (261, 1) => {
                self.registers.status.set_vblank_started(false);
                self.registers.status.set_sprite_zero_hit(false);
                self.registers.status.set_sprite_overflow(false);
            }
            _ => {}
        }

        return frame_ready;
    }

    pub(crate) fn write_address(&mut self, data: u8) {
        self.registers.address.update(data);
    }

    pub(crate) fn write_control(&mut self, data: u8) {
        self.registers.control.update(data);
    }

    pub(crate) fn write_mask(&mut self, data: u8) {
        self.registers.mask.update(data);
    }

    pub(crate) fn write_scroll(&mut self, data: u8) {
        self.registers.scroll.update(data)
    }

    pub(crate) fn read_status(&mut self) -> u8 {
        let status = self.registers.status.clone();
        let data = *status.into_bytes().first().unwrap();

        self.registers.status.set_vblank_started(false);
        self.registers.address.reset_latch();
        self.registers.scroll.reset_latch();

        data
    }

    pub(crate) fn write_oam_address(&mut self, data: u8) {
        self.registers.oam_addr = data;
    }

    pub(crate) fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.registers.oam_addr as usize] = data;
        self.registers.oam_addr = self.registers.oam_addr.wrapping_add(1);
    }

    pub(crate) fn read_oam_data(&self) -> u8 {
        self.oam_data[self.registers.oam_addr as usize]
    }

    pub(crate) fn nmi_asserted(&self) -> bool {
        self.registers.control.generate_nmi() && self.registers.status.vblank_started()
    }
}

pub trait PPU {
    fn ppu_read(&mut self) -> u8;
    fn ppu_write(&mut self, value: u8);

    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]);

    fn background_palette(&self, attribute_table: &[u8], tile_x: usize, tile_y: usize) -> [u8; 4];
    fn sprite_palette(&self, index: usize) -> [u8; 4];
    fn mirror_vram_address(&self, address: u16) -> u16;

    fn is_sprite_0_hit(&self, cycle: usize) -> bool;
}

impl PPU for NES {
    fn ppu_read(&mut self) -> u8 {
        let address = self.bus.ppu.registers.address.as_u16();

        self.bus.ppu.registers.increment_vram_address();

        match address {
            0..=0x1FFF => {
                let result = self.bus.ppu.read_buffer;
                self.bus.ppu.read_buffer = self.bus.cartridge.ppu_read(address);
                result
            }
            0x2000..=0x3EFF => {
                let result = self.bus.ppu.read_buffer;
                self.bus.ppu.read_buffer =
                    self.bus.ciram[self.mirror_vram_address(address) as usize];
                result
            }
            0x3F00..=0x3FFF => self.bus.ppu.palette_table[palette_index(address)],
            _ => unreachable!("attempted to access mirrored address space {}", address),
        }
    }

    fn ppu_write(&mut self, value: u8) {
        let address = self.bus.ppu.registers.address.as_u16();
        match address {
            0..=0x1FFF => self.bus.cartridge.ppu_write(address, value),
            0x2000..=0x3EFF => {
                self.bus.ciram[self.mirror_vram_address(address) as usize] = value;
            }
            0x3F00..=0x3FFF => {
                self.bus.ppu.palette_table[palette_index(address)] = value & 0x3F;
            }
            _ => panic!("unexpected access to mirrored space {}", address),
        }

        self.bus.ppu.registers.increment_vram_address();
    }

    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]) {
        for &data in buffer {
            self.bus.ppu.write_oam_data(data);
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
            self.bus.ppu.palette_table[0],
            self.bus.ppu.palette_table[palette_start],
            self.bus.ppu.palette_table[palette_start + 1],
            self.bus.ppu.palette_table[palette_start + 2],
        ]
    }

    fn sprite_palette(&self, index: usize) -> [u8; 4] {
        let palette_index = self.bus.ppu.oam_data[index + 2] & 0b11;
        let palette_start = 0x11 + (palette_index * 4) as usize;

        [
            0,
            self.bus.ppu.palette_table[palette_start],
            self.bus.ppu.palette_table[palette_start + 1],
            self.bus.ppu.palette_table[palette_start + 2],
        ]
    }

    fn mirror_vram_address(&self, address: u16) -> u16 {
        let mirrored_vram = address & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        match (&self.bus.cartridge.screen_mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
            (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.bus.ppu.oam_data[0] as usize;
        let x = self.bus.ppu.oam_data[3] as usize;

        (y == self.bus.ppu.scanline as usize)
            && x <= cycle
            && self.bus.ppu.registers.mask.show_sprite()
    }
}

fn palette_index(address: u16) -> usize {
    let index = (address & 0x1F) as usize;

    match index {
        0x10 | 0x14 | 0x18 | 0x1C => index - 0x10,
        _ => index,
    }
}
