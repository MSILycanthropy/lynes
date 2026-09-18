use crate::{NES, cartridge::ScreenMirroring};

mod palette;
pub(crate) mod registers;
mod render;

#[cfg(test)]
mod tests;

pub trait PPU {
    fn tick_ppu(&mut self) -> bool;
    fn ppu_read(&mut self) -> u8;
    fn ppu_write(&mut self, value: u8);

    fn ppu_write_address(&mut self, data: u8);
    fn ppu_write_control(&mut self, data: u8);
    fn ppu_write_oam_address(&mut self, data: u8);
    fn ppu_write_oam_data(&mut self, data: u8);
    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]);
    fn ppu_write_mask(&mut self, data: u8);
    fn ppu_write_scroll(&mut self, data: u8);

    fn ppu_read_status(&mut self) -> u8;
    fn ppu_read_oam_data(&mut self) -> u8;

    fn background_palette(&self, attribute_table: &[u8], tile_x: usize, tile_y: usize) -> [u8; 4];
    fn sprite_palette(&self, index: usize) -> [u8; 4];
    fn mirror_vram_address(&self, address: u16) -> u16;

    fn is_sprite_0_hit(&self, cycle: usize) -> bool;
}

impl PPU for NES {
    fn tick_ppu(&mut self) -> bool {
        let rendering_enabled =
            self.ppu_registers.mask.show_background() || self.ppu_registers.mask.show_sprite();

        // TODO: Regions so that we dont hardcode NTSC
        let frame_ready = match (self.ppu_scanline, self.ppu_dot) {
            (261, 339) if self.ppu_odd_frame && rendering_enabled => {
                self.ppu_scanline = 0;
                self.ppu_dot = 0;
                true
            }
            (261, 340) => {
                self.ppu_scanline = 0;
                self.ppu_dot = 0;
                true
            }
            (_, 340) => {
                self.ppu_scanline += 1;
                self.ppu_dot = 0;
                false
            }

            _ => {
                self.ppu_dot += 1;

                false
            }
        };

        if frame_ready {
            self.ppu_odd_frame = !self.ppu_odd_frame;
        }

        match (self.ppu_scanline, self.ppu_dot) {
            (241, 1) => {
                self.ppu_registers.status.set_vblank_started(true);

                if self.ppu_registers.control.generate_nmi() {
                    self.interrupt_state.nmi_pending = true;
                }
            }
            (261, 1) => {
                self.ppu_registers.status.set_vblank_started(false);
                self.ppu_registers.status.set_sprite_zero_hit(false);
                self.ppu_registers.status.set_sprite_overflow(false);
            }
            _ => {}
        }

        return frame_ready;
    }

    fn ppu_read(&mut self) -> u8 {
        let address = self.ppu_registers.address.as_u16();

        self.ppu_registers.increment_vram_address();

        match address {
            0..=0x1FFF => {
                let result = self.ppu_read_buffer;
                self.ppu_read_buffer = self.chr_rom[address as usize];
                result
            }
            0x2000..=0x3EFF => {
                let result = self.ppu_read_buffer;
                self.ppu_read_buffer = self.ppu_vram[self.mirror_vram_address(address) as usize];
                result
            }
            0x3F00..=0x3FFF => self.palette_table[palette_index(address)],
            _ => unreachable!("attempted to access mirrored address space {}", address),
        }
    }

    fn ppu_write(&mut self, value: u8) {
        let address = self.ppu_registers.address.as_u16();
        match address {
            0..=0x1FFF => println!("attempt to write to chr rom space {}", address),
            0x2000..=0x3EFF => {
                self.ppu_vram[self.mirror_vram_address(address) as usize] = value;
            }
            0x3F00..=0x3FFF => {
                self.palette_table[palette_index(address)] = value & 0x3F;
            }
            _ => panic!("unexpected access to mirrored space {}", address),
        }

        self.ppu_registers.increment_vram_address();
    }

    fn ppu_write_address(&mut self, data: u8) {
        self.ppu_registers.address.update(data);
    }

    fn ppu_write_control(&mut self, data: u8) {
        let nmi_status_before = self.ppu_registers.control.generate_nmi();

        self.ppu_registers.control.update(data);

        let nmi_status_after = self.ppu_registers.control.generate_nmi();

        if !nmi_status_before && nmi_status_after && self.ppu_registers.status.vblank_started() {
            self.interrupt_state.nmi_pending = true;
        }
    }

    fn ppu_write_mask(&mut self, data: u8) {
        self.ppu_registers.mask.update(data);
    }

    fn ppu_write_scroll(&mut self, data: u8) {
        self.ppu_registers.scroll.update(data)
    }

    fn ppu_write_oam_address(&mut self, data: u8) {
        self.ppu_registers.oam_addr = data;
    }

    fn ppu_write_oam_data(&mut self, data: u8) {
        self.oam_data[self.ppu_registers.oam_addr as usize] = data;
        self.ppu_registers.oam_addr = self.ppu_registers.oam_addr.wrapping_add(1);
    }

    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]) {
        for data in buffer.iter() {
            self.ppu_write_oam_data(*data);
        }
    }

    fn ppu_read_status(&mut self) -> u8 {
        let status = self.ppu_registers.status.clone();
        let data = *status.into_bytes().first().unwrap();

        self.ppu_registers.status.set_vblank_started(false);
        self.ppu_registers.address.reset_latch();
        self.ppu_registers.scroll.reset_latch();

        data
    }

    fn ppu_read_oam_data(&mut self) -> u8 {
        self.oam_data[self.ppu_registers.oam_addr as usize]
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

    fn mirror_vram_address(&self, address: u16) -> u16 {
        let mirrored_vram = address & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        match (&self.mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
            (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;

        (y == self.ppu_scanline as usize) && x <= cycle && self.ppu_registers.mask.show_sprite()
    }
}

fn palette_index(address: u16) -> usize {
    let index = (address & 0x1F) as usize;

    match index {
        0x10 | 0x14 | 0x18 | 0x1C => index - 0x10,
        _ => index,
    }
}
