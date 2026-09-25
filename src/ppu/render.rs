use crate::frame::{FRAME_HEIGHT, FRAME_WIDTH};

use super::{Ppu, bus::PpuBus, palette};

#[derive(Clone, Copy)]
struct SpritePixel {
    color: u8,
    behind_background: bool,
}

impl Ppu {
    fn sprite_scanline(
        &self,
        bus: &PpuBus<'_>,
        scanline: usize,
    ) -> [Option<SpritePixel>; FRAME_WIDTH] {
        let mut pixels = [None; FRAME_WIDTH];

        if !self.registers.mask.show_sprite() {
            return pixels;
        }

        for i in (0..self.oam_data.len()).step_by(4) {
            let tile = self.oam_data[i + 1] as u16;
            let tile_x = self.oam_data[i + 3] as usize;
            let tile_y = self.oam_data[i] as usize;

            if scanline < tile_y || scanline >= tile_y + 8 {
                continue;
            }

            let flip_vertical = self.oam_data[i + 2] >> 7 & 1 == 1;
            let flip_horizontal = self.oam_data[i + 2] >> 6 & 1 == 1;

            let palette = self.sprite_palette(i);

            let bank = self.registers.control.sprite_pattern_address_value();

            let tile_address = bank + tile * 16;

            let row = scanline - tile_y;
            let row = if flip_vertical { 7 - row } else { row };
            let mut high = bus.read(tile_address + row as u16);
            let mut low = bus.read(tile_address + row as u16 + 8);

            'inner: for x in (0..=7).rev() {
                let value = (1 & low) << 1 | 1 & high;

                high = high >> 1;
                low = low >> 1;

                if value == 0 {
                    continue 'inner;
                }

                let screen_x = tile_x + if flip_horizontal { 7 - x } else { x };

                if screen_x < 8 && !self.registers.mask.leftmost_8px_sprite() {
                    continue;
                }

                if screen_x >= FRAME_WIDTH {
                    continue;
                }

                if pixels[screen_x].is_none() {
                    pixels[screen_x] = Some(SpritePixel {
                        color: palette[usize::from(value)],
                        behind_background: self.oam_data[i + 2] & 0x20 != 0,
                    })
                }
            }
        }

        pixels
    }

    pub(crate) fn render_scanline(&mut self, bus: &PpuBus<'_>, scanline: usize) {
        assert!(scanline < FRAME_HEIGHT);

        let sprites = self.sprite_scanline(bus, scanline);

        for (x, sprite) in sprites.into_iter().enumerate() {
            let background_visible = self.registers.mask.show_background()
                && (x >= 8 || self.registers.mask.leftmost_8px_background());

            let background = if background_visible {
                self.background_pixel(bus, x)
            } else {
                0
            };

            let color = match sprite {
                Some(sprite) if background == 0 || !sprite.behind_background => sprite.color,
                _ => self.palette_table[background as usize],
            };

            self.frame
                .set_pixel(x, scanline, palette::SYSTEM_PALLETE[color as usize]);
        }
    }
}
