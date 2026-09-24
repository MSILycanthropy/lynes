use crate::{
    Interrupt, NES,
    cartridge::{Cartridge, ScreenMirroring},
    mapper::Mapper,
};

fn cartridge(mirroring: ScreenMirroring) -> Cartridge {
    Cartridge {
        prg_rom: vec![0; 32_768],
        prg_ram: vec![0; 8192],
        chr_rom: vec![0; 8_192],
        mapper: Mapper::new(0, 32_768).unwrap(),
        submapper: 0,
        screen_mirroring: mirroring,
    }
}

// These helpers exercise PPU register behavior without advancing CPU/PPU time.
fn set_vram_address(nes: &mut NES, address: u16) {
    nes.bus.read(0x2002); // Start with the first PPUADDR write.
    nes.bus.write(0x2006, (address >> 8) as u8);
    nes.bus.write(0x2006, address as u8);
}

fn write_vram(nes: &mut NES, address: u16, value: u8) {
    set_vram_address(nes, address);
    nes.bus.write(0x2007, value);
}

fn read_nametable(nes: &mut NES, address: u16) -> u8 {
    set_vram_address(nes, address);
    nes.bus.read(0x2007); // Discard the previous buffered value.
    // Stay in nametable space even when testing the last byte at $3EFF.
    set_vram_address(nes, address);
    nes.bus.read(0x2007)
}

fn read_palette_color(nes: &mut NES, address: u16) -> u8 {
    set_vram_address(nes, address);
    // Storage/mirroring checks compare color bits; upper bits come from the I/O latch.
    nes.bus.read(0x2007) & 0x3F
}

#[test]
fn status_writes_latch_data_without_changing_flags_or_scroll_toggle() {
    let mut nes = NES::default();
    nes.bus.ppu.registers.status.set_vblank_started(true);
    nes.bus.ppu.registers.status.set_sprite_zero_hit(true);
    nes.bus.write(0x2005, 0x12);

    nes.bus.write(0x3FFA, 0x1B); // Mirrored PPUSTATUS write.
    assert_eq!(nes.bus.peek(0x2000), 0x1B);
    assert_eq!(nes.bus.peek(0x2002), 0xDB);
    assert_eq!(nes.bus.peek(0x2000), 0x1B); // Status peek doesn't latch its result.

    nes.bus.write(0x2005, 0x34);
    assert_eq!(nes.bus.ppu.registers.scroll.scroll_x(), 0x12);
    assert_eq!(nes.bus.ppu.registers.scroll.scroll_y(), 0x34);
    assert_eq!(nes.bus.read(0x2002), 0xD4);
    assert_eq!(nes.bus.peek(0x2000), 0xD4); // Latches status before clearing vblank.
    assert_eq!(nes.bus.peek(0x2002), 0x54);
}

#[test]
fn data_and_oam_reads_latch_returned_bytes_but_peeks_do_not() {
    let mut nes = NES::default();
    set_vram_address(&mut nes, 0x2000);
    nes.bus.ppu.read_buffer = 0xA6;
    nes.bus.ciram[0] = 0x39;

    assert_eq!(nes.bus.peek(0x2007), 0xA6);
    assert_eq!(nes.bus.peek(0x2000), 0);
    assert_eq!(nes.bus.read(0x2007), 0xA6);
    assert_eq!(nes.bus.peek(0x2000), 0xA6);
    assert_eq!(nes.bus.ppu.read_buffer, 0x39); // Refill is separate from the I/O latch.

    nes.bus.ppu.oam_data[0] = 0xCA;
    nes.bus.write(0x2003, 0);
    nes.bus.write(0x2002, 0x5B);
    assert_eq!(nes.bus.peek(0x2004), 0xCA);
    assert_eq!(nes.bus.peek(0x2000), 0x5B);
    assert_eq!(nes.bus.read(0x2004), 0xCA);
    assert_eq!(nes.bus.peek(0x2000), 0xCA);
    assert_eq!(nes.bus.ppu.registers.oam_addr, 0);
}

#[test]
fn palette_reads_combine_latch_bits_and_greyscale_while_refilling_buffer() {
    for greyscale in [false, true] {
        for upper_bits in [0x00, 0x40, 0x80, 0xC0] {
            let mut nes = NES::default();
            write_vram(&mut nes, 0x2F00, 0x9A);
            write_vram(&mut nes, 0x3F00, 0x2F);
            set_vram_address(&mut nes, 0x3F00);
            nes.bus.write(0x2001, u8::from(greyscale));
            nes.bus.write(0x2002, upper_bits | 0x15);
            nes.bus.ppu.read_buffer = 0x55;
            let expected = upper_bits | if greyscale { 0x20 } else { 0x2F };

            assert_eq!(nes.bus.peek(0x2007), expected);
            assert_eq!(nes.bus.peek(0x2000), upper_bits | 0x15);
            assert_eq!(nes.bus.ppu.read_buffer, 0x55);
            assert_eq!(nes.bus.ppu.registers.scroll.memory_address(), 0x3F00);

            assert_eq!(nes.bus.read(0x2007), expected);
            assert_eq!(nes.bus.peek(0x2000), expected);
            assert_eq!(nes.bus.ppu.palette_table[0], 0x2F);
            assert_eq!(nes.bus.ppu.registers.scroll.memory_address(), 0x3F01);
            assert_eq!(nes.bus.ppu.read_buffer, 0x9A);

            set_vram_address(&mut nes, 0x2000);
            assert_eq!(nes.bus.read(0x2007), 0x9A);
        }
    }
}

#[test]
fn cartridge_chr_reads_stay_buffered_and_rom_writes_are_ignored() {
    let mut cart = cartridge(ScreenMirroring::Horizontal);
    cart.chr_rom[0] = 0x12;
    cart.chr_rom[0x1FFF] = 0x34;
    let mut nes = NES::default();
    nes.insert_cart(cart);

    set_vram_address(&mut nes, 0);
    assert_eq!(nes.bus.read(0x2007), 0);
    assert_eq!(nes.bus.read(0x2007), 0x12);

    write_vram(&mut nes, 0x1FFF, 0xFF);
    set_vram_address(&mut nes, 0x1FFF);
    nes.bus.read(0x2007);
    assert_eq!(nes.bus.read(0x2007), 0x34);
}

#[test]
fn renderer_reads_background_and_sprite_patterns_from_cartridge() {
    let mut cart = cartridge(ScreenMirroring::Horizontal);
    // Background tile 0 in bank 0 uses color 1; sprite tile 1 in bank 1 uses color 2.
    cart.chr_rom[0..8].fill(0xFF);
    cart.chr_rom[0x1018..0x1020].fill(0xFF);
    let mut nes = NES::default();
    nes.insert_cart(cart);
    nes.bus.write(0x2000, 0x08);
    nes.bus.ppu.palette_table[1] = 1;
    nes.bus.ppu.palette_table[0x12] = 2;
    nes.bus.ppu.oam_data[..4].copy_from_slice(&[16, 1, 0, 16]);

    nes.render();

    for (x, y, palette_index) in [(32, 32, 1), (16, 16, 2)] {
        let offset = y * crate::frame::FRAME_STRIDE + x * 3;
        let color = super::palette::SYSTEM_PALLETE[palette_index];
        assert_eq!(
            &nes.frame().data()[offset..offset + 3],
            &[color.0, color.1, color.2]
        );
    }
}

#[test]
fn palette_reads_and_writes_mirror_every_32_bytes() {
    let mut nes = NES::default();

    for base in 0x3F00..=0x3F1F {
        for displacement in (0..0x100).step_by(0x20) {
            let alias = base + displacement;
            write_vram(&mut nes, base, 0x12);
            assert_eq!(
                read_palette_color(&mut nes, alias),
                0x12,
                "read ${alias:04X} after writing ${base:04X}"
            );

            write_vram(&mut nes, alias, 0x2B);
            assert_eq!(
                read_palette_color(&mut nes, base),
                0x2B,
                "read ${base:04X} after writing ${alias:04X}"
            );
        }
    }
}

#[test]
fn special_palette_aliases_work_in_both_directions_in_every_mirror() {
    let mut nes = NES::default();

    for (background, sprite) in [
        (0x3F00, 0x3F10),
        (0x3F04, 0x3F14),
        (0x3F08, 0x3F18),
        (0x3F0C, 0x3F1C),
    ] {
        for displacement in (0..0x100).step_by(0x20) {
            for address in [background + displacement, sprite + displacement] {
                let value = 1
                    + (displacement / 0x20) as u8
                    + if address == sprite + displacement {
                        16
                    } else {
                        0
                    };
                write_vram(&mut nes, address, value);
                for other_displacement in (0..0x100).step_by(0x20) {
                    for alias in [background + other_displacement, sprite + other_displacement] {
                        assert_eq!(
                            read_palette_color(&mut nes, alias),
                            value,
                            "read ${alias:04X} after writing ${address:04X}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn distinct_palette_entries_do_not_alias() {
    let mut nes = NES::default();
    // The 28 independent bytes, including background entries $04/$08/$0C.
    let entries = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x11, 0x12, 0x13, 0x15, 0x16, 0x17, 0x19, 0x1A, 0x1B, 0x1D, 0x1E, 0x1F,
    ];
    for entry in entries {
        write_vram(&mut nes, 0x3F00 + entry, 0x20 + entry as u8);
    }
    for entry in entries {
        assert_eq!(
            read_palette_color(&mut nes, 0x3F00 + entry),
            0x20 + entry as u8,
            "palette entry ${entry:02X} was overwritten through another entry"
        );
    }
}

#[test]
fn palette_writes_store_only_six_color_bits() {
    let mut nes = NES::default();
    for (written, expected) in [
        (0x00, 0x00),
        (0x3F, 0x3F),
        (0x40, 0x00),
        (0x7F, 0x3F),
        (0x80, 0x00),
        (0xC5, 0x05),
        (0xFF, 0x3F),
    ] {
        for address in 0x3F00..=0x3FFF {
            write_vram(&mut nes, address, written);
            // Inspect storage as well as readback: the renderer reads it directly.
            assert!(nes.bus.ppu.palette_table.iter().all(|&value| value <= 0x3F));
            assert_eq!(
                read_palette_color(&mut nes, address),
                expected,
                "write ${written:02X} at ${address:04X}"
            );
        }
    }
}

#[test]
fn nametable_mirroring_uses_cartridge_configuration() {
    for (mirroring, banks) in [
        (ScreenMirroring::Vertical, [0, 1, 0, 1]),
        (ScreenMirroring::Horizontal, [0, 0, 1, 1]),
    ] {
        let mut nes = NES::default();
        nes.insert_cart(cartridge(mirroring));

        // Tile and attribute memory, including both ends of each nametable.
        for offset in [0, 0x3BF, 0x3C0, 0x3FF] {
            let mut expected = [0; 2];
            for (table, &bank) in banks.iter().enumerate() {
                let value = 0x10 + table as u8;
                write_vram(&mut nes, 0x2000 + table as u16 * 0x400 + offset, value);
                expected[bank] = value;

                // Writing any alias changes only its own physical nametable.
                for (other, &other_bank) in banks.iter().enumerate() {
                    let address = 0x2000 + other as u16 * 0x400 + offset;
                    assert_eq!(
                        read_nametable(&mut nes, address),
                        expected[other_bank],
                        "read ${address:04X} after writing table {table}"
                    );
                }
            }
        }
    }
}

#[test]
fn upper_nametable_range_mirrors_reads_and_writes() {
    for mirroring in [ScreenMirroring::Vertical, ScreenMirroring::Horizontal] {
        let mut nes = NES::default();
        nes.insert_cart(cartridge(mirroring));

        for offset in 0..0xF00u16 {
            let value = (offset as u8).wrapping_add((offset >> 8) as u8);
            write_vram(&mut nes, 0x2000 + offset, value);
            assert_eq!(
                read_nametable(&mut nes, 0x3000 + offset),
                value,
                "upper alias at offset ${offset:03X}"
            );

            write_vram(&mut nes, 0x3000 + offset, !value);
            assert_eq!(
                read_nametable(&mut nes, 0x2000 + offset),
                !value,
                "lower alias at offset ${offset:03X}"
            );
        }
    }
}

#[test]
fn palette_boundary_does_not_use_nametable_mapping_or_buffering() {
    let mut nes = NES::default();
    nes.insert_cart(cartridge(ScreenMirroring::Vertical));
    write_vram(&mut nes, 0x3EFF, 0xA5);
    write_vram(&mut nes, 0x3F00, 0x12);
    write_vram(&mut nes, 0x3F1F, 0x23);

    assert_eq!(read_nametable(&mut nes, 0x2EFF), 0xA5);
    set_vram_address(&mut nes, 0x3F00);
    assert_eq!(nes.bus.read(0x2007), 0x12);
    set_vram_address(&mut nes, 0x3F1F);
    assert_eq!(nes.bus.read(0x2007), 0x23);
    assert_eq!(read_nametable(&mut nes, 0x3EFF), 0xA5);
}

#[test]
#[should_panic(expected = "No four screen mirroring")]
fn four_screen_cartridges_are_rejected() {
    NES::default().insert_cart(cartridge(ScreenMirroring::FourScreen));
}

fn assert_frame_length(nes: &mut NES, dots: usize) {
    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 0));
    let was_odd = nes.bus.ppu.odd_frame;

    for elapsed in 1..=dots {
        assert_eq!(
            nes.bus.ppu.tick(),
            elapsed == dots,
            "unexpected frame boundary at dot {elapsed} of {dots}"
        );
    }

    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 0));
    assert_eq!(nes.bus.ppu.odd_frame, !was_odd);
}

#[test]
fn vblank_starts_at_scanline_241_dot_1() {
    for nmi_enabled in [false, true] {
        let mut nes = NES::default();
        nes.bus.ppu.scanline = 240;
        nes.bus.ppu.dot = 340;
        nes.bus.write(0x2000, if nmi_enabled { 0x80 } else { 0 });

        assert!(!nes.bus.ppu.tick());
        nes.sample_interrupt_line();
        assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (241, 0));
        assert!(!nes.bus.ppu.registers.status.vblank_started());
        assert!(!nes.bus.ppu.nmi_asserted());
        assert!(nes.cpu.take_interrupt().is_none());

        assert!(!nes.bus.ppu.tick());
        nes.sample_interrupt_line();
        assert!(nes.bus.ppu.registers.status.vblank_started());
        assert_eq!(nes.bus.ppu.nmi_asserted(), nmi_enabled);
        assert!(!nes.bus.ppu.registers.status.sprite_zero_hit());

        // Once acknowledged, the same vblank must not request NMI every dot.
        let interrupt = nes.cpu.take_interrupt();
        assert_eq!(matches!(interrupt, Some(Interrupt::NMI)), nmi_enabled);
        assert!(!nes.bus.ppu.tick());
        nes.sample_interrupt_line();
        assert!(nes.cpu.take_interrupt().is_none());
    }
}

#[test]
fn pre_render_dot_1_clears_flags_but_preserves_pending_interrupts() {
    let mut nes = NES::default();
    nes.bus.ppu.scanline = 260;
    nes.bus.ppu.dot = 340;
    nes.bus.ppu.registers.status.set_vblank_started(true);
    nes.bus.ppu.registers.status.set_sprite_zero_hit(true);
    nes.bus.ppu.registers.status.set_sprite_overflow(true);
    // Assert the PPU output without advancing past the boundary under test.
    nes.bus.write(0x2000, 0x80);
    nes.sample_interrupt_line();
    assert!(nes.bus.ppu.nmi_asserted());

    assert!(!nes.bus.ppu.tick());
    nes.sample_interrupt_line();
    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (261, 0));
    assert!(nes.bus.ppu.registers.status.vblank_started());
    assert!(nes.bus.ppu.registers.status.sprite_zero_hit());
    assert!(nes.bus.ppu.registers.status.sprite_overflow());

    assert!(!nes.bus.ppu.tick());
    nes.sample_interrupt_line();
    assert!(!nes.bus.ppu.registers.status.vblank_started());
    assert!(!nes.bus.ppu.registers.status.sprite_zero_hit());
    assert!(!nes.bus.ppu.registers.status.sprite_overflow());
    assert!(!nes.bus.ppu.nmi_asserted());

    // Neither pre-render flag clearing nor frame wrap consumes the CPU latch.
    for _ in 0..340 {
        nes.bus.ppu.tick();
        nes.sample_interrupt_line();
    }
    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 0));
    assert!(matches!(nes.cpu.take_interrupt(), Some(Interrupt::NMI)));
    assert!(nes.cpu.take_interrupt().is_none());
}

#[test]
fn enabling_nmi_during_vblank_requests_only_on_enable_edges() {
    let mut nes = NES::default();
    nes.bus.ppu.scanline = 241;
    nes.bus.ppu.tick();
    nes.sample_interrupt_line();
    assert!(!nes.bus.ppu.nmi_asserted());
    assert!(nes.cpu.take_interrupt().is_none());

    nes.cpu_write(0x2000, 0x80);
    assert!(nes.bus.ppu.nmi_asserted());
    assert!(matches!(nes.cpu.take_interrupt(), Some(Interrupt::NMI)));
    nes.cpu_write(0x2000, 0x80);
    assert!(nes.cpu.take_interrupt().is_none());

    nes.cpu_write(0x2000, 0);
    assert!(!nes.bus.ppu.nmi_asserted());
    nes.cpu_write(0x2000, 0x80);
    assert!(matches!(nes.cpu.take_interrupt(), Some(Interrupt::NMI)));
}

#[test]
fn status_read_preserves_latched_nmi_and_allows_next_vblank_edge() {
    let mut nes = NES::default();
    nes.cpu_write(0x2000, 0x80);
    nes.bus.ppu.scanline = 241;
    nes.bus.ppu.dot = 0;

    // Exercise the machine's sampling loop, not just the test's manual wiring.
    nes.cpu.clock_cycle(&mut nes.bus);
    assert!(nes.bus.ppu.nmi_asserted());
    assert_eq!(nes.cpu_read(0x2002) & 0x80, 0x80);
    assert!(!nes.bus.ppu.nmi_asserted());
    assert!(matches!(nes.cpu.take_interrupt(), Some(Interrupt::NMI)));
    assert!(nes.cpu.take_interrupt().is_none());

    // Position at the next vblank edge without another sample in between:
    // the status read itself must have delivered the deasserted line.
    nes.bus.ppu.scanline = 241;
    nes.bus.ppu.dot = 0;
    nes.cpu.clock_cycle(&mut nes.bus);
    assert!(matches!(nes.cpu.take_interrupt(), Some(Interrupt::NMI)));
    nes.cpu.clock_cycle(&mut nes.bus);
    assert!(nes.cpu.take_interrupt().is_none());
}

#[test]
fn rendering_disabled_keeps_both_frame_parities_full_length() {
    let mut nes = NES::default();
    for _ in 0..4 {
        assert_frame_length(&mut nes, 89_342);
    }
}

#[test]
fn either_rendering_layer_shortens_only_odd_frames() {
    for mask in [0x08, 0x10, 0x18] {
        let mut nes = NES::default();
        nes.bus.ppu.write_mask(mask);
        for dots in [89_342, 89_341, 89_342, 89_341] {
            assert_frame_length(&mut nes, dots);
        }
    }
}

#[test]
fn dot_skip_uses_rendering_state_at_the_end_of_pre_render() {
    for rendering_at_skip in [false, true] {
        let mut nes = NES::default();
        nes.bus.ppu.odd_frame = true;
        nes.bus
            .ppu
            .write_mask(if rendering_at_skip { 0 } else { 0x08 });
        nes.bus.ppu.scanline = 261;
        nes.bus.ppu.dot = 338;
        assert!(!nes.bus.ppu.tick());

        nes.bus
            .ppu
            .write_mask(if rendering_at_skip { 0x08 } else { 0 });
        assert_eq!(nes.bus.ppu.tick(), rendering_at_skip);
        if !rendering_at_skip {
            assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (261, 340));
            assert!(nes.bus.ppu.tick());
        }
        assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 0));
        assert!(!nes.bus.ppu.odd_frame);
    }
}

#[test]
fn cpu_cycle_advancement_keeps_ticking_after_frame_completion() {
    let mut nes = NES::default();
    nes.bus.ppu.scanline = 261;
    nes.bus.ppu.dot = 339;

    for _ in 0..2 {
        nes.cpu.clock_cycle(&mut nes.bus);
    }
    assert!(std::mem::take(&mut nes.bus.frame_pending));
    assert!(!nes.bus.frame_pending);
    assert_eq!(nes.bus.total_cpu_cycles, 2);
    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 4));
    nes.cpu.clock_cycle(&mut nes.bus);
    assert!(!std::mem::take(&mut nes.bus.frame_pending));
    assert_eq!(nes.bus.total_cpu_cycles, 3);
    assert_eq!((nes.bus.ppu.scanline, nes.bus.ppu.dot), (0, 7));
}
