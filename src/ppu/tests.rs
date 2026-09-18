use super::PPU;
use crate::{
    Interrupt, NES,
    cartridge::{Cartridge, ScreenMirroring},
    cpu::CPU,
};

fn cartridge(mirroring: ScreenMirroring) -> Cartridge {
    Cartridge {
        prg_rom: vec![0; 32_768],
        chr_rom: vec![0; 8_192],
        mapper: 0,
        screen_mirroring: mirroring,
    }
}

fn set_vram_address(nes: &mut NES, address: u16) {
    nes.cpu_read(0x2002); // Start with the first PPUADDR write.
    nes.cpu_write(0x2006, (address >> 8) as u8);
    nes.cpu_write(0x2006, address as u8);
}

fn write_vram(nes: &mut NES, address: u16, value: u8) {
    set_vram_address(nes, address);
    nes.cpu_write(0x2007, value);
}

fn read_nametable(nes: &mut NES, address: u16) -> u8 {
    set_vram_address(nes, address);
    nes.cpu_read(0x2007); // Discard the previous buffered value.
    // Stay in nametable space even when testing the last byte at $3EFF.
    set_vram_address(nes, address);
    nes.cpu_read(0x2007)
}

fn read_palette(nes: &mut NES, address: u16) -> u8 {
    set_vram_address(nes, address);
    nes.cpu_read(0x2007)
}

#[test]
fn palette_reads_and_writes_mirror_every_32_bytes() {
    let mut nes = NES::default();

    for base in 0x3F00..=0x3F1F {
        for displacement in (0..0x100).step_by(0x20) {
            let alias = base + displacement;
            write_vram(&mut nes, base, 0x12);
            assert_eq!(
                read_palette(&mut nes, alias),
                0x12,
                "read ${alias:04X} after writing ${base:04X}"
            );

            write_vram(&mut nes, alias, 0x2B);
            assert_eq!(
                read_palette(&mut nes, base),
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
                            read_palette(&mut nes, alias),
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
            read_palette(&mut nes, 0x3F00 + entry),
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
            assert!(nes.palette_table.iter().all(|&value| value <= 0x3F));
            assert_eq!(
                read_palette(&mut nes, address),
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
    assert_eq!(nes.cpu_read(0x2007), 0x12);
    set_vram_address(&mut nes, 0x3F1F);
    assert_eq!(nes.cpu_read(0x2007), 0x23);
    assert_eq!(read_nametable(&mut nes, 0x3EFF), 0xA5);
}

#[test]
#[should_panic(expected = "No four screen mirroring")]
fn four_screen_cartridges_are_rejected() {
    NES::default().insert_cart(cartridge(ScreenMirroring::FourScreen));
}

fn assert_frame_length(nes: &mut NES, dots: usize) {
    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 0));
    let was_odd = nes.ppu_odd_frame;

    for elapsed in 1..=dots {
        assert_eq!(
            nes.tick_ppu(),
            elapsed == dots,
            "unexpected frame boundary at dot {elapsed} of {dots}"
        );
    }

    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 0));
    assert_eq!(nes.ppu_odd_frame, !was_odd);
}

#[test]
fn vblank_starts_at_scanline_241_dot_1() {
    for nmi_enabled in [false, true] {
        let mut nes = NES::default();
        nes.ppu_scanline = 240;
        nes.ppu_dot = 340;
        nes.ppu_write_control(if nmi_enabled { 0x80 } else { 0 });

        assert!(!nes.tick_ppu());
        assert_eq!((nes.ppu_scanline, nes.ppu_dot), (241, 0));
        assert!(!nes.ppu_registers.status.vblank_started());
        assert!(!nes.interrupt_state.nmi_pending);

        assert!(!nes.tick_ppu());
        assert!(nes.ppu_registers.status.vblank_started());
        assert_eq!(nes.interrupt_state.nmi_pending, nmi_enabled);
        assert!(!nes.ppu_registers.status.sprite_zero_hit());

        // Once acknowledged, the same vblank must not request NMI every dot.
        let interrupt = nes.interrupt_state.take(true);
        assert_eq!(matches!(interrupt, Some(Interrupt::NMI)), nmi_enabled);
        assert!(!nes.tick_ppu());
        assert!(!nes.interrupt_state.nmi_pending);
    }
}

#[test]
fn pre_render_dot_1_clears_flags_but_preserves_pending_interrupts() {
    let mut nes = NES::default();
    nes.ppu_scanline = 260;
    nes.ppu_dot = 340;
    nes.ppu_registers.status.set_vblank_started(true);
    nes.ppu_registers.status.set_sprite_zero_hit(true);
    nes.ppu_registers.status.set_sprite_overflow(true);
    nes.interrupt_state.nmi_pending = true;
    nes.interrupt_state.irq_asserted = true;

    assert!(!nes.tick_ppu());
    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (261, 0));
    assert!(nes.ppu_registers.status.vblank_started());
    assert!(nes.ppu_registers.status.sprite_zero_hit());
    assert!(nes.ppu_registers.status.sprite_overflow());

    assert!(!nes.tick_ppu());
    assert!(!nes.ppu_registers.status.vblank_started());
    assert!(!nes.ppu_registers.status.sprite_zero_hit());
    assert!(!nes.ppu_registers.status.sprite_overflow());

    // Neither pre-render flag clearing nor frame wrap consumes the CPU latch.
    for _ in 0..340 {
        nes.tick_ppu();
    }
    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 0));
    assert!(matches!(
        nes.interrupt_state.take(true),
        Some(Interrupt::NMI)
    ));
    assert!(nes.interrupt_state.irq_asserted);
}

#[test]
fn enabling_nmi_during_vblank_requests_only_on_enable_edges() {
    let mut nes = NES::default();
    nes.ppu_scanline = 241;
    nes.tick_ppu();
    assert!(!nes.interrupt_state.nmi_pending);

    nes.ppu_write_control(0x80);
    assert!(matches!(
        nes.interrupt_state.take(true),
        Some(Interrupt::NMI)
    ));
    nes.ppu_write_control(0x80);
    assert!(!nes.interrupt_state.nmi_pending);

    nes.ppu_write_control(0);
    nes.ppu_write_control(0x80);
    assert!(matches!(
        nes.interrupt_state.take(true),
        Some(Interrupt::NMI)
    ));
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
        nes.ppu_write_mask(mask);
        for dots in [89_342, 89_341, 89_342, 89_341] {
            assert_frame_length(&mut nes, dots);
        }
    }
}

#[test]
fn dot_skip_uses_rendering_state_at_the_end_of_pre_render() {
    for rendering_at_skip in [false, true] {
        let mut nes = NES::default();
        nes.ppu_odd_frame = true;
        nes.ppu_write_mask(if rendering_at_skip { 0 } else { 0x08 });
        nes.ppu_scanline = 261;
        nes.ppu_dot = 338;
        assert!(!nes.tick_ppu());

        nes.ppu_write_mask(if rendering_at_skip { 0x08 } else { 0 });
        assert_eq!(nes.tick_ppu(), rendering_at_skip);
        if !rendering_at_skip {
            assert_eq!((nes.ppu_scanline, nes.ppu_dot), (261, 340));
            assert!(nes.tick_ppu());
        }
        assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 0));
        assert!(!nes.ppu_odd_frame);
    }
}

#[test]
fn cpu_cycle_advancement_keeps_ticking_after_frame_completion() {
    let mut nes = NES::default();
    nes.ppu_scanline = 261;
    nes.ppu_dot = 339;

    assert!(nes.advance_cpu_cycles(2));
    assert_eq!(nes.total_cpu_cycles, 2);
    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 4));
    assert!(!nes.advance_cpu_cycles(1));
    assert_eq!(nes.total_cpu_cycles, 3);
    assert_eq!((nes.ppu_scanline, nes.ppu_dot), (0, 7));
}
