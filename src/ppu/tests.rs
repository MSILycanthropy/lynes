use super::PPU;
use crate::{Interrupt, NES};

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
