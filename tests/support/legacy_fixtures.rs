// Addresses apply only to these exact pinned ROM bytes (FNV-1a fingerprint).
// Reporting routines and source references are documented in ../fixtures/blargg/LEGACY.md.
const FIXTURES: &[Fixture] = &[
    Fixture {
        path: "blargg_apu_2005.07.30/01.len_ctr.nes",
        fingerprint: 0xB681E7F3009E0C19,
        protocol: Protocol::Memory {
            pc: 0xE01B,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/02.len_table.nes",
        fingerprint: 0x27D11A19C160574C,
        protocol: Protocol::Memory {
            pc: 0xE034,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/03.irq_flag.nes",
        fingerprint: 0xFF589D20A3CFBE78,
        protocol: Protocol::Memory {
            pc: 0xE01B,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/04.clock_jitter.nes",
        fingerprint: 0xE490302D594F5842,
        protocol: Protocol::Memory {
            pc: 0xE01B,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/05.len_timing_mode0.nes",
        fingerprint: 0x4E2E1F5E0C91C9FA,
        protocol: Protocol::Memory {
            pc: 0xE03A,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/06.len_timing_mode1.nes",
        fingerprint: 0x86EA30DD9BD6F32E,
        protocol: Protocol::Memory {
            pc: 0xE03A,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/07.irq_flag_timing.nes",
        fingerprint: 0x721D609DF5DD6FF6,
        protocol: Protocol::Memory {
            pc: 0xE03A,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/08.irq_timing.nes",
        fingerprint: 0xC5DF316A981B237E,
        protocol: Protocol::Memory {
            pc: 0xE03A,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/09.reset_timing.nes",
        fingerprint: 0x6AA46A38A4A21916,
        protocol: Protocol::Memory {
            pc: 0xE01F,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/10.len_halt_timing.nes",
        fingerprint: 0x093D68A572B477C3,
        protocol: Protocol::Memory {
            pc: 0xE03A,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_apu_2005.07.30/11.len_reload_timing.nes",
        fingerprint: 0x6462C42550B0BB95,
        protocol: Protocol::Memory {
            pc: 0xE053,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_nes_cpu_test5/cpu.nes",
        fingerprint: 0xD99F7E745A40B497,
        protocol: Protocol::Accumulator { pc: 0x81FA },
    },
    Fixture {
        path: "blargg_nes_cpu_test5/official.nes",
        fingerprint: 0x87F375E648E3A6D7,
        protocol: Protocol::Accumulator { pc: 0x81FA },
    },
    Fixture {
        path: "blargg_ppu_tests_2005.09.15b/palette_ram.nes",
        fingerprint: 0x1C4B89814B111EA6,
        protocol: Protocol::Memory {
            pc: 0xE000,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
        fingerprint: 0x5A744EC15C5BB09B,
        protocol: Protocol::Memory {
            pc: 0xE000,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
        fingerprint: 0x0689AE3A5B23F3FE,
        protocol: Protocol::Memory {
            pc: 0xE000,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
        fingerprint: 0x44E18F77B75EF068,
        protocol: Protocol::Memory {
            pc: 0xE000,
            address: 0xF0,
        },
    },
    Fixture {
        path: "blargg_ppu_tests_2005.09.15b/vram_access.nes",
        fingerprint: 0x9FEBFF67CDC17A1B,
        protocol: Protocol::Memory {
            pc: 0xE000,
            address: 0xF0,
        },
    },
    Fixture {
        path: "branch_timing_tests/1.Branch_Basics.nes",
        fingerprint: 0xCBB132A62B6DE288,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "branch_timing_tests/2.Backward_Branch.nes",
        fingerprint: 0xA2C4635B385A9E96,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "branch_timing_tests/3.Forward_Branch.nes",
        fingerprint: 0xD0AA1D3A1D8A587D,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "cpu_dummy_reads/cpu_dummy_reads.nes",
        fingerprint: 0xAAB8F3CE27508C20,
        protocol: Protocol::Accumulator { pc: 0xE469 },
    },
    Fixture {
        path: "cpu_timing_test6/cpu_timing_test.nes",
        fingerprint: 0xBBE2A2C67A0551BA,
        protocol: Protocol::Timing {
            pass_pc: 0xE1B7,
            fail_pc: 0xE0B0,
        },
    },
    Fixture {
        path: "dmc_dma_during_read4/dma_2007_read.nes",
        fingerprint: 0xD07BE582F66D1890,
        protocol: Protocol::Crc {
            pc: 0xE569,
            expected: &[0x159A7A8F, 0x5E3DF9C4],
        },
    },
    Fixture {
        path: "dmc_dma_during_read4/dma_2007_write.nes",
        fingerprint: 0x404B52A10F49367C,
        protocol: Protocol::Accumulator { pc: 0xE569 },
    },
    Fixture {
        path: "dmc_dma_during_read4/dma_4016_read.nes",
        fingerprint: 0xA86F19FB2DDD7278,
        protocol: Protocol::Accumulator { pc: 0xE569 },
    },
    Fixture {
        path: "dmc_dma_during_read4/double_2007_read.nes",
        fingerprint: 0xF46947C0482E053B,
        protocol: Protocol::Crc {
            pc: 0xE369,
            expected: &[0x85CFD627, 0xF018C287, 0x440EF923, 0xE52F41A5],
        },
    },
    Fixture {
        path: "dmc_dma_during_read4/read_write_2007.nes",
        fingerprint: 0xD53D55A07DA542D8,
        protocol: Protocol::Accumulator { pc: 0xE369 },
    },
    Fixture {
        path: "dmc_tests/buffer_retained.nes",
        fingerprint: 0x725587B431981A2F,
        protocol: Protocol::Observation {
            pc: 0xE149,
            reason: "DMC audio diagnostic; no machine-readable pass/fail assertion",
        },
    },
    Fixture {
        path: "dmc_tests/latency.nes",
        fingerprint: 0x1DD442C3CB51D2CA,
        protocol: Protocol::Observation {
            pc: 0xE162,
            reason: "DMC audio diagnostic; no machine-readable pass/fail assertion",
        },
    },
    Fixture {
        path: "dmc_tests/status.nes",
        fingerprint: 0xDED9BBA08A57C9A8,
        protocol: Protocol::Observation {
            pc: 0xE14E,
            reason: "DMC audio diagnostic; no machine-readable pass/fail assertion",
        },
    },
    Fixture {
        path: "dmc_tests/status_irq.nes",
        fingerprint: 0x03DFA57AA9B7D92D,
        protocol: Protocol::Observation {
            pc: 0xE154,
            reason: "DMC audio diagnostic; no machine-readable pass/fail assertion",
        },
    },
    Fixture {
        path: "mmc3_irq_tests/1.Clocking.nes",
        fingerprint: 0x67F9379B1A3FC5C2,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "mmc3_irq_tests/2.Details.nes",
        fingerprint: 0x01D98D5EE41335D7,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "mmc3_irq_tests/3.A12_clocking.nes",
        fingerprint: 0xF5F3BE2E162451E1,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "mmc3_irq_tests/4.Scanline_timing.nes",
        fingerprint: 0x92EECEEDF0B7DBA5,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "mmc3_irq_tests/5.MMC3_rev_A.nes",
        fingerprint: 0x66B249067B8AB6E2,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "mmc3_irq_tests/6.MMC3_rev_B.nes",
        fingerprint: 0x57FF03DEB4F017C5,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/01.len_ctr.nes",
        fingerprint: 0x33D3B7406DBE05EB,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/02.len_table.nes",
        fingerprint: 0xFA3625C032AB05A3,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/03.irq_flag.nes",
        fingerprint: 0xB71D4D5313E15AF0,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/04.clock_jitter.nes",
        fingerprint: 0x77517796E944FF6E,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/05.len_timing_mode0.nes",
        fingerprint: 0x1ACF448504168C79,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/06.len_timing_mode1.nes",
        fingerprint: 0x18129271F885EC50,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/07.irq_flag_timing.nes",
        fingerprint: 0xCF40417635EAFCE5,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/08.irq_timing.nes",
        fingerprint: 0xC65B66F4C508BB7A,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/10.len_halt_timing.nes",
        fingerprint: 0x85FF0D27DBE36F9C,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "pal_apu_tests/11.len_reload_timing.nes",
        fingerprint: 0x7295C254C1986C39,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "read_joy3/count_errors.nes",
        fingerprint: 0x8A1E556D4714F7F0,
        protocol: Protocol::Measurement {
            pc: 0xE367,
            address: 0x1D,
        },
    },
    Fixture {
        path: "read_joy3/count_errors_fast.nes",
        fingerprint: 0xBAB06903A46FCA11,
        protocol: Protocol::Measurement {
            pc: 0xE367,
            address: 0x1D,
        },
    },
    Fixture {
        path: "read_joy3/test_buttons.nes",
        fingerprint: 0xDA0ED53D328980E2,
        protocol: Protocol::Accumulator { pc: 0xE367 },
    },
    Fixture {
        path: "read_joy3/thorough_test.nes",
        fingerprint: 0xFFBFCF6AC983FCAE,
        protocol: Protocol::Accumulator { pc: 0xE467 },
    },
    Fixture {
        path: "sprdma_and_dmc_dma/sprdma_and_dmc_dma.nes",
        fingerprint: 0x4E1242AF6A39F4E3,
        protocol: Protocol::Status,
    },
    Fixture {
        path: "sprdma_and_dmc_dma/sprdma_and_dmc_dma_512.nes",
        fingerprint: 0x0CCD1B37FE524BFE,
        protocol: Protocol::Status,
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/01.basics.nes",
        fingerprint: 0xB6FDBDB9FCA8FF95,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/02.alignment.nes",
        fingerprint: 0x2E79407E50585CC7,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/03.corners.nes",
        fingerprint: 0xB53829C50FE1AF9F,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/04.flip.nes",
        fingerprint: 0xC24EC3232E4669DE,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/05.left_clip.nes",
        fingerprint: 0xC00661E6AD88DE54,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/06.right_edge.nes",
        fingerprint: 0x47E63A6E7955AF6D,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/07.screen_bottom.nes",
        fingerprint: 0x2D34874B16BC7A4F,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/08.double_height.nes",
        fingerprint: 0x18D050708F5BF90E,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/09.timing_basics.nes",
        fingerprint: 0x8260605EEDF85546,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/10.timing_order.nes",
        fingerprint: 0x26D867B8EFF45EA2,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_hit_tests_2005.10.05/11.edge_timing.nes",
        fingerprint: 0x7D060BD9D28A2E11,
        protocol: Protocol::Memory {
            pc: 0xE00B,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_overflow_tests/1.Basics.nes",
        fingerprint: 0xFDFCA71332180AC1,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_overflow_tests/2.Details.nes",
        fingerprint: 0xF089EF368605F66A,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_overflow_tests/3.Timing.nes",
        fingerprint: 0x97FF16DACD000B82,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_overflow_tests/4.Obscure.nes",
        fingerprint: 0x9CA043FF9FD044DE,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "sprite_overflow_tests/5.Emulator.nes",
        fingerprint: 0x698654D213A12387,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/1.frame_basics.nes",
        fingerprint: 0xF38E55854ADF55FE,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/2.vbl_timing.nes",
        fingerprint: 0x05C061881F8CB61D,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/3.even_odd_frames.nes",
        fingerprint: 0xF6038A0AF6BEF96F,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/4.vbl_clear_timing.nes",
        fingerprint: 0xED4B74849D626A12,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/5.nmi_suppression.nes",
        fingerprint: 0xE3CE608860FF8D09,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/6.nmi_disable.nes",
        fingerprint: 0x1F18F4752854B5AF,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
    Fixture {
        path: "vbl_nmi_timing/7.nmi_timing.nes",
        fingerprint: 0x462BCAC7B7CCC413,
        protocol: Protocol::Memory {
            pc: 0xE01D,
            address: 0xF8,
        },
    },
];
