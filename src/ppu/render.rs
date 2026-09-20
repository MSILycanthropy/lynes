use crate::{NES, cartridge::ScreenMirroring, ppu::PPU};

use super::palette;

impl NES {
    pub(crate) fn render(&mut self) {
        self.render_background();
        self.render_sprites();
    }

    fn render_background(&mut self) {
        let scroll_x = self.ppu_registers.scroll.scroll_x as usize;
        let scroll_y = self.ppu_registers.scroll.scroll_y as usize;

        let (first_nametable, second_nametable) = match (
            self.cartridge.screen_mirroring.clone(),
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
                panic!(
                    "Not supported mirroring type {:?}",
                    self.cartridge.screen_mirroring
                );
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
            let tile_address = bank + tile * 16;
            let palette = self.background_palette(attribute_table, tile_x, tile_y);

            for y in 0..=7 {
                let mut high = self.cartridge.ppu_read(tile_address + y as u16);
                let mut low = self.cartridge.ppu_read(tile_address + y as u16 + 8);

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

            let tile_address = bank + tile * 16;

            for y in 0..=7 {
                let mut high = self.cartridge.ppu_read(tile_address + y as u16);
                let mut low = self.cartridge.ppu_read(tile_address + y as u16 + 8);

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

struct ViewPortRect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl ViewPortRect {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Self {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }

    fn point_is_bounded(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }
}
