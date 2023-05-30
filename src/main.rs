use lynes::{*, renderer::Renderer, color::ColorPalette};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", 256 * 3, 240 * 3)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
                .create_texture_target(
                    sdl2::pixels::PixelFormatEnum::RGB24,
                    256,
                    240,
                )
                .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut renderer = renderer::SDLRenderer::new(canvas, texture);
    let mut nes = NESBuilder::new()
                    .rom("roms/pacman.nes")
                    .color_palette("palettes/ntsc.pal")
                    .build();
    nes.reset();

    let mut frame = renderer::Frame::new();

    loop {
        if nes.should_render {
            let bg_bank = if nes.ppu_registers.control.background_pattern_table_address() {
                0x1000
            } else {
                0x0000
            };

            let spr_bank = if nes.ppu_registers.control.sprite_pattern_table_address() {
                0x1000
            } else {
                0x0000
            };
            show_nametable(&mut frame, &nes, bg_bank, spr_bank);

            renderer.render(&frame);
            // renderer.render(&show_tiles(&nes.chr_rom, &nes.palette_table, &nes.color_palette, palette_index, right_bank));
            for event in event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. } => std::process::exit(0),
                    _ => {}
                }
            }
        }


        nes.clock();
    }
}

pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];


fn show_nametable(frame: &mut renderer::Frame, nes: &NES, bg_bank: u16, spr_bank: u16) {
    // println!("oam data: {:?}", nes.oam_data);

    for i in 0..0x3c0 {
        let tile_index = nes.ppu_vram[i] as u16;
        let tile_column  = i % 32;
        let tile_row = i / 32;
        let tile_addr  = tile_index * 16;
        let tile = &nes.chr_rom[(bg_bank + tile_addr) as usize..(bg_bank + tile_addr + 16) as usize];
        let palette = bg_palette(&nes.ppu_vram, tile_column as u8, tile_row as u8, &nes.palette_table);

        for y in 0..8 {
            let mut msb = tile[y];
            let mut lsb = tile[y + 8];

            for x in (0..8).rev() {
                let pixel = (lsb & 0x01) << 1 | (msb & 0x01);

                lsb >>= 1;
                msb >>= 1;

                let color_index = palette[pixel as usize];
                let color = SYSTEM_PALLETE[color_index as usize];

                frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, color);
            }
        }
    }

    for i in (0..nes.oam_data.len()).step_by(4).rev() {
        let tile_index = nes.oam_data[i + 1] as u16;
        let tile_x  = nes.oam_data[i + 3] as usize;
        let tile_y = nes.oam_data[i] as usize;

        // println!("tile_index: {:x}", tile_index);

        let flip_vertically = nes.oam_data[i + 2] >> 7 & 0x01 == 1;
        let flip_horizontally = nes.oam_data[i + 2] >> 6 & 0x01 == 1;

        let palette_index = nes.oam_data[i + 2] & 0x03;
        let sprite_palette = sprite_palette(&nes.palette_table, palette_index);

        let tile_addr = tile_index * 16;
        let tile = &nes.chr_rom[(spr_bank + tile_addr) as usize..(spr_bank + tile_addr + 16) as usize];

        for y in 0..8 {
            let mut msb = tile[y];
            let mut lsb = tile[y + 8];

            for x in (0..8).rev() {
                let pixel = (lsb & 0x01) << 1 | (msb & 0x01);

                lsb >>= 1;
                msb >>= 1;

                if pixel == 0 {
                    continue;
                }

                let color_index = sprite_palette[pixel as usize];
                let color = SYSTEM_PALLETE[color_index as usize];

                let x = if flip_horizontally { 7 - x } else { x };
                let y = if flip_vertically { 7 - y } else { y };

                frame.set_pixel(tile_x + x, tile_y + y, color);
            }
        }
    }
}

fn sprite_palette(palette_table: &[u8; 32], palette_index: u8) -> [u8; 4] {
    let palette_start = 0x11 + (palette_index as usize * 4);

    [0, palette_table[palette_start], palette_table[palette_start + 1], palette_table[palette_start + 2]]
}

fn bg_palette(vram: &[u8; 2048], tile_column: u8, tile_row: u8, palette_table: &[u8; 32]) -> [u8; 4] {
    let attr_table_index = (tile_column / 4) + ((tile_row / 4) * 8);
    let attr_table_addr = 0x3c0 + attr_table_index as usize;
    let attr_byte = vram[attr_table_addr];

    let pallete_index = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0x03,
        (1, 0) => (attr_byte >> 2) & 0x03,
        (0, 1) => (attr_byte >> 4) & 0x03,
        (1, 1) => (attr_byte >> 6) & 0x03,
        _ => panic!("Invalid tile position")
    };

    let palette_start = (pallete_index * 4 + 1) as usize;

    [palette_table[0], palette_table[palette_start], palette_table[palette_start + 1], palette_table[palette_start + 2]]
}
