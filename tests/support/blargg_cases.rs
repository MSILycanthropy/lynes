// Add ROM tests here. Remove an ignore reason once the test passes.

deferred_test!(
    apu_mixer_dmc,
    "apu_mixer/dmc.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    apu_mixer_noise,
    "apu_mixer/noise.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    apu_mixer_square,
    "apu_mixer/square.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    apu_mixer_triangle,
    "apu_mixer/triangle.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
blargg_test!(
    apu_reset_4015_cleared,
    "apu_reset/4015_cleared.nes",
    30000000
);
blargg_test!(
    apu_reset_4017_timing,
    "apu_reset/4017_timing.nes",
    30000000,
    "baseline: Failed(3); Delay after effective $4017 write: 0"
);
blargg_test!(
    apu_reset_4017_written,
    "apu_reset/4017_written.nes",
    30000000,
    "baseline: Failed(2); At power, $4017 should be written with $00"
);
blargg_test!(
    apu_reset_irq_flag_cleared,
    "apu_reset/irq_flag_cleared.nes",
    30000000
);
blargg_test!(
    apu_reset_len_ctrs_enabled,
    "apu_reset/len_ctrs_enabled.nes",
    30000000,
    "baseline: Failed(3); At reset, length counters should be enabled, triangle unaffected"
);
blargg_test!(
    apu_reset_works_immediately,
    "apu_reset/works_immediately.nes",
    30000000,
    "baseline: Failed(2); At power, writes should work immediately"
);
blargg_test!(
    apu_test_apu_test,
    "apu_test/apu_test.nes",
    30000000,
    "baseline: Failed(1); 1-len_ctr failed #2: length counter load or $4015"
);
blargg_test!(
    apu_test_1_len_ctr,
    "apu_test/1-len_ctr.nes",
    30000000,
    "baseline: Failed(2); Channel: 0"
);
blargg_test!(
    apu_test_2_len_table,
    "apu_test/2-len_table.nes",
    30000000,
    "baseline: Failed(1); Channel: 0"
);
blargg_test!(
    apu_test_3_irq_flag,
    "apu_test/3-irq_flag.nes",
    30000000,
    "baseline: Failed(4); Flag should be set in $4017 mode $00"
);
blargg_test!(
    apu_test_4_jitter,
    "apu_test/4-jitter.nes",
    30000000,
    "baseline: Failed(3); Frame irq is set too late"
);
blargg_test!(
    apu_test_5_len_timing,
    "apu_test/5-len_timing.nes",
    30000000,
    "baseline: Failed(2); Channel: 0"
);
blargg_test!(
    apu_test_6_irq_flag_timing,
    "apu_test/6-irq_flag_timing.nes",
    30000000,
    "baseline: Failed(3); Flag first set too late"
);
blargg_test!(
    apu_test_7_dmc_basics,
    "apu_test/7-dmc_basics.nes",
    30000000,
    "baseline: Failed(2); Channel: 0"
);
blargg_test!(
    apu_test_8_dmc_rates,
    "apu_test/8-dmc_rates.nes",
    30000000,
    "baseline: Failed(2); Rate 0's period is too short"
);
deferred_test!(
    blargg_apu_2005_07_30_01_len_ctr,
    "blargg_apu_2005.07.30/01.len_ctr.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_02_len_table,
    "blargg_apu_2005.07.30/02.len_table.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_03_irq_flag,
    "blargg_apu_2005.07.30/03.irq_flag.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_04_clock_jitter,
    "blargg_apu_2005.07.30/04.clock_jitter.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_05_len_timing_mode0,
    "blargg_apu_2005.07.30/05.len_timing_mode0.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_06_len_timing_mode1,
    "blargg_apu_2005.07.30/06.len_timing_mode1.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_07_irq_flag_timing,
    "blargg_apu_2005.07.30/07.irq_flag_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_08_irq_timing,
    "blargg_apu_2005.07.30/08.irq_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_09_reset_timing,
    "blargg_apu_2005.07.30/09.reset_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_10_len_halt_timing,
    "blargg_apu_2005.07.30/10.len_halt_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_apu_2005_07_30_11_len_reload_timing,
    "blargg_apu_2005.07.30/11.len_reload_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_nes_cpu_test5_cpu,
    "blargg_nes_cpu_test5/cpu.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_nes_cpu_test5_official,
    "blargg_nes_cpu_test5/official.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_ppu_tests_2005_09_15b_palette_ram,
    "blargg_ppu_tests_2005.09.15b/palette_ram.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_ppu_tests_2005_09_15b_power_up_palette,
    "blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_ppu_tests_2005_09_15b_sprite_ram,
    "blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_ppu_tests_2005_09_15b_vbl_clear_time,
    "blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    blargg_ppu_tests_2005_09_15b_vram_access,
    "blargg_ppu_tests_2005.09.15b/vram_access.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    branch_timing_tests_1_branch_basics,
    "branch_timing_tests/1.Branch_Basics.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    branch_timing_tests_2_backward_branch,
    "branch_timing_tests/2.Backward_Branch.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    branch_timing_tests_3_forward_branch,
    "branch_timing_tests/3.Forward_Branch.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    cpu_dummy_reads_cpu_dummy_reads,
    "cpu_dummy_reads/cpu_dummy_reads.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
blargg_test!(
    cpu_dummy_writes_cpu_dummy_writes_oam,
    "cpu_dummy_writes/cpu_dummy_writes_oam.nes",
    30000000
);
blargg_test!(
    cpu_dummy_writes_cpu_dummy_writes_ppumem,
    "cpu_dummy_writes/cpu_dummy_writes_ppumem.nes",
    30000000
);
blargg_test!(
    cpu_exec_space_test_cpu_exec_space_apu,
    "cpu_exec_space/test_cpu_exec_space_apu.nes",
    30000000
);
blargg_test!(
    cpu_exec_space_test_cpu_exec_space_ppuio,
    "cpu_exec_space/test_cpu_exec_space_ppuio.nes",
    30000000
);
blargg_test!(
    cpu_interrupts_v2_cpu_interrupts,
    "cpu_interrupts_v2/cpu_interrupts.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported PRG ROM size: 81920 bytes\")"
);
blargg_test!(
    cpu_interrupts_v2_1_cli_latency,
    "cpu_interrupts_v2/1-cli_latency.nes",
    30000000,
    "baseline: Failed(3); APU should generate IRQ when $4017 = $00"
);
blargg_test!(
    cpu_interrupts_v2_2_nmi_and_brk,
    "cpu_interrupts_v2/2-nmi_and_brk.nes",
    30000000
);
blargg_test!(
    cpu_interrupts_v2_3_nmi_and_irq,
    "cpu_interrupts_v2/3-nmi_and_irq.nes",
    30000000,
    "baseline: Failed(1); NMI BRK"
);
blargg_test!(
    cpu_interrupts_v2_4_irq_and_dma,
    "cpu_interrupts_v2/4-irq_and_dma.nes",
    30000000,
    "baseline: Failed(1); 53 +0"
);
blargg_test!(
    cpu_interrupts_v2_5_branch_delays_irq,
    "cpu_interrupts_v2/5-branch_delays_irq.nes",
    30000000,
    "baseline: TimedOut; test_jmp"
);
blargg_test!(
    cpu_reset_ram_after_reset,
    "cpu_reset/ram_after_reset.nes",
    30000000
);
blargg_test!(
    cpu_reset_registers,
    "cpu_reset/registers.nes",
    30000000
);
deferred_test!(
    cpu_timing_test6_cpu_timing_test,
    "cpu_timing_test6/cpu_timing_test.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_dma_during_read4_dma_2007_read,
    "dmc_dma_during_read4/dma_2007_read.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_dma_during_read4_dma_2007_write,
    "dmc_dma_during_read4/dma_2007_write.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_dma_during_read4_dma_4016_read,
    "dmc_dma_during_read4/dma_4016_read.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_dma_during_read4_double_2007_read,
    "dmc_dma_during_read4/double_2007_read.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_dma_during_read4_read_write_2007,
    "dmc_dma_during_read4/read_write_2007.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_tests_buffer_retained,
    "dmc_tests/buffer_retained.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_tests_latency,
    "dmc_tests/latency.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_tests_status,
    "dmc_tests/status.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    dmc_tests_status_irq,
    "dmc_tests/status_irq.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    full_palette_flowing_palette,
    "full_palette/flowing_palette.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    full_palette_full_palette,
    "full_palette/full_palette.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    full_palette_full_palette_smooth,
    "full_palette/full_palette_smooth.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
blargg_test!(
    instr_misc_instr_misc,
    "instr_misc/instr_misc.nes",
    30000000,
    "baseline: Failed(1); 04-dummy_reads_apu failed #2: Official opcodes failed"
);
blargg_test!(
    instr_misc_01_abs_x_wrap,
    "instr_misc/01-abs_x_wrap.nes",
    30000000
);
blargg_test!(
    instr_misc_02_branch_wrap,
    "instr_misc/02-branch_wrap.nes",
    30000000
);
blargg_test!(
    instr_misc_03_dummy_reads,
    "instr_misc/03-dummy_reads.nes",
    30000000
);
blargg_test!(
    instr_misc_04_dummy_reads_apu,
    "instr_misc/04-dummy_reads_apu.nes",
    30000000,
    "baseline: Failed(2); Official opcodes failed"
);
blargg_test!(
    instr_test_v3_all_instrs,
    "instr_test_v3/all_instrs.nes",
    120000000
);
blargg_test!(
    instr_test_v3_official_only,
    "instr_test_v3/official_only.nes",
    120000000
);
blargg_test!(
    instr_test_v3_01_implied,
    "instr_test_v3/01-implied.nes",
    120000000
);
blargg_test!(
    instr_test_v3_02_immediate,
    "instr_test_v3/02-immediate.nes",
    120000000
);
blargg_test!(
    instr_test_v3_03_zero_page,
    "instr_test_v3/03-zero_page.nes",
    120000000
);
blargg_test!(
    instr_test_v3_04_zp_xy,
    "instr_test_v3/04-zp_xy.nes",
    120000000
);
blargg_test!(
    instr_test_v3_05_absolute,
    "instr_test_v3/05-absolute.nes",
    120000000
);
blargg_test!(
    instr_test_v3_06_abs_xy,
    "instr_test_v3/06-abs_xy.nes",
    120000000
);
blargg_test!(
    instr_test_v3_07_ind_x,
    "instr_test_v3/07-ind_x.nes",
    120000000
);
blargg_test!(
    instr_test_v3_08_ind_y,
    "instr_test_v3/08-ind_y.nes",
    120000000
);
blargg_test!(
    instr_test_v3_09_branches,
    "instr_test_v3/09-branches.nes",
    120000000
);
blargg_test!(
    instr_test_v3_10_stack,
    "instr_test_v3/10-stack.nes",
    120000000
);
blargg_test!(
    instr_test_v3_11_jmp_jsr,
    "instr_test_v3/11-jmp_jsr.nes",
    120000000
);
blargg_test!(instr_test_v3_12_rts, "instr_test_v3/12-rts.nes", 120000000);
blargg_test!(instr_test_v3_13_rti, "instr_test_v3/13-rti.nes", 120000000);
blargg_test!(
    instr_test_v3_14_brk,
    "instr_test_v3/14-brk.nes",
    120000000
);
blargg_test!(
    instr_test_v3_15_special,
    "instr_test_v3/15-special.nes",
    120000000
);
blargg_test!(
    instr_test_v5_all_instrs,
    "instr_test_v5/all_instrs.nes",
    120000000
);
blargg_test!(
    instr_test_v5_official_only,
    "instr_test_v5/official_only.nes",
    120000000
);
blargg_test!(cpu_basics, "instr_test_v5/01-basics.nes", 120000000);
blargg_test!(
    instr_test_v5_02_implied,
    "instr_test_v5/02-implied.nes",
    120000000
);
blargg_test!(
    instr_test_v5_03_immediate,
    "instr_test_v5/03-immediate.nes",
    120000000
);
blargg_test!(
    instr_test_v5_04_zero_page,
    "instr_test_v5/04-zero_page.nes",
    120000000
);
blargg_test!(
    instr_test_v5_05_zp_xy,
    "instr_test_v5/05-zp_xy.nes",
    120000000
);
blargg_test!(
    instr_test_v5_06_absolute,
    "instr_test_v5/06-absolute.nes",
    120000000
);
blargg_test!(
    instr_test_v5_07_abs_xy,
    "instr_test_v5/07-abs_xy.nes",
    120000000
);
blargg_test!(
    instr_test_v5_08_ind_x,
    "instr_test_v5/08-ind_x.nes",
    120000000
);
blargg_test!(
    instr_test_v5_09_ind_y,
    "instr_test_v5/09-ind_y.nes",
    120000000
);
blargg_test!(
    instr_test_v5_10_branches,
    "instr_test_v5/10-branches.nes",
    120000000
);
blargg_test!(
    instr_test_v5_11_stack,
    "instr_test_v5/11-stack.nes",
    120000000
);
blargg_test!(
    instr_test_v5_12_jmp_jsr,
    "instr_test_v5/12-jmp_jsr.nes",
    120000000
);
blargg_test!(instr_test_v5_13_rts, "instr_test_v5/13-rts.nes", 120000000);
blargg_test!(instr_test_v5_14_rti, "instr_test_v5/14-rti.nes", 120000000);
blargg_test!(
    instr_test_v5_15_brk,
    "instr_test_v5/15-brk.nes",
    120000000
);
blargg_test!(
    instr_test_v5_16_special,
    "instr_test_v5/16-special.nes",
    120000000
);
blargg_test!(
    instr_timing_instr_timing,
    "instr_timing/instr_timing.nes",
    30000000,
    "baseline: Failed(1); 1-instr_timing failed #5: APU length-period or instruction timing"
);
blargg_test!(
    instr_timing_1_instr_timing,
    "instr_timing/1-instr_timing.nes",
    30000000,
    "baseline: Failed(5); Timing of APU length period, INC zp, LDA abs, AND #imm, or BNE (taken) is wrong"
);
blargg_test!(
    instr_timing_2_branch_timing,
    "instr_timing/2-branch_timing.nes",
    30000000,
    "baseline: Failed(1); 10 0 0 0 0 0 0 0 0"
);
deferred_test!(
    mmc3_irq_tests_1_clocking,
    "mmc3_irq_tests/1.Clocking.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    mmc3_irq_tests_2_details,
    "mmc3_irq_tests/2.Details.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    mmc3_irq_tests_3_a12_clocking,
    "mmc3_irq_tests/3.A12_clocking.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    mmc3_irq_tests_4_scanline_timing,
    "mmc3_irq_tests/4.Scanline_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    mmc3_irq_tests_5_mmc3_rev_a,
    "mmc3_irq_tests/5.MMC3_rev_A.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    mmc3_irq_tests_6_mmc3_rev_b,
    "mmc3_irq_tests/6.MMC3_rev_B.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
blargg_test!(
    mmc3_test_1_clocking,
    "mmc3_test/1-clocking.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_details,
    "mmc3_test/2-details.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_3_a12_clocking,
    "mmc3_test/3-A12_clocking.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_4_scanline_timing,
    "mmc3_test/4-scanline_timing.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_5_mmc3,
    "mmc3_test/5-MMC3.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_6_mmc6,
    "mmc3_test/6-MMC6.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_1_clocking,
    "mmc3_test_2/1-clocking.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_2_details,
    "mmc3_test_2/2-details.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_3_a12_clocking,
    "mmc3_test_2/3-A12_clocking.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_4_scanline_timing,
    "mmc3_test_2/4-scanline_timing.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_5_mmc3,
    "mmc3_test_2/5-MMC3.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    mmc3_test_2_6_mmc3_alt,
    "mmc3_test_2/6-MMC3_alt.nes",
    30000000,
    "baseline: LoadError(\"Unable to initialize cartridge mapper: Unsupported mapper ID: 4\")"
);
blargg_test!(
    nes_instr_test_01_implied,
    "nes_instr_test/01-implied.nes",
    120000000
);
blargg_test!(
    nes_instr_test_02_immediate,
    "nes_instr_test/02-immediate.nes",
    120000000
);
blargg_test!(
    nes_instr_test_03_zero_page,
    "nes_instr_test/03-zero_page.nes",
    120000000
);
blargg_test!(
    nes_instr_test_04_zp_xy,
    "nes_instr_test/04-zp_xy.nes",
    120000000
);
blargg_test!(
    nes_instr_test_05_absolute,
    "nes_instr_test/05-absolute.nes",
    120000000
);
blargg_test!(
    nes_instr_test_06_abs_xy,
    "nes_instr_test/06-abs_xy.nes",
    120000000
);
blargg_test!(
    nes_instr_test_07_ind_x,
    "nes_instr_test/07-ind_x.nes",
    120000000
);
blargg_test!(
    nes_instr_test_08_ind_y,
    "nes_instr_test/08-ind_y.nes",
    120000000
);
blargg_test!(
    nes_instr_test_09_branches,
    "nes_instr_test/09-branches.nes",
    120000000
);
blargg_test!(
    nes_instr_test_10_stack,
    "nes_instr_test/10-stack.nes",
    120000000
);
blargg_test!(
    nes_instr_test_11_special,
    "nes_instr_test/11-special.nes",
    120000000
);
deferred_test!(
    nmi_sync_demo_ntsc,
    "nmi_sync/demo_ntsc.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
deferred_test!(
    nmi_sync_demo_pal,
    "nmi_sync/demo_pal.nes",
    "requires audio/visual validation; completion alone is not a pass"
);
blargg_test!(oam_read_oam_read, "oam_read/oam_read.nes", 30000000);
blargg_test!(
    oam_stress_oam_stress,
    "oam_stress/oam_stress.nes",
    120000000,
    "baseline: Failed(1); ------*---*---*-"
);
deferred_test!(
    pal_apu_tests_01_len_ctr,
    "pal_apu_tests/01.len_ctr.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_02_len_table,
    "pal_apu_tests/02.len_table.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_03_irq_flag,
    "pal_apu_tests/03.irq_flag.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_04_clock_jitter,
    "pal_apu_tests/04.clock_jitter.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_05_len_timing_mode0,
    "pal_apu_tests/05.len_timing_mode0.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_06_len_timing_mode1,
    "pal_apu_tests/06.len_timing_mode1.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_07_irq_flag_timing,
    "pal_apu_tests/07.irq_flag_timing.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_08_irq_timing,
    "pal_apu_tests/08.irq_timing.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_10_len_halt_timing,
    "pal_apu_tests/10.len_halt_timing.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
deferred_test!(
    pal_apu_tests_11_len_reload_timing,
    "pal_apu_tests/11.len_reload_timing.nes",
    "PAL timing and APU unavailable; legacy result protocol also needs an adapter"
);
blargg_test!(
    ppu_open_bus_ppu_open_bus,
    "ppu_open_bus/ppu_open_bus.nes",
    30000000,
    "baseline: Failed(3); Decay value should become zero by one second"
);
blargg_test!(
    ppu_read_buffer_test_ppu_read_buffer,
    "ppu_read_buffer/test_ppu_read_buffer.nes",
    120000000,
    "baseline: Failed(63); sprite 0 hit checks fail (reported tests: 69 67 65 63)"
);
blargg_test!(
    ppu_vbl_nmi_ppu_vbl_nmi,
    "ppu_vbl_nmi/ppu_vbl_nmi.nes",
    120000000,
    "baseline: Failed(1); test 10 of 10: Clock is skipped too late, relative to enabling BG; 08 07"
);
blargg_test!(ppu_vbl_basics, "ppu_vbl_nmi/01-vbl_basics.nes", 30000000);
blargg_test!(
    ppu_vbl_nmi_02_vbl_set_time,
    "ppu_vbl_nmi/02-vbl_set_time.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_03_vbl_clear_time,
    "ppu_vbl_nmi/03-vbl_clear_time.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_04_nmi_control,
    "ppu_vbl_nmi/04-nmi_control.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_05_nmi_timing,
    "ppu_vbl_nmi/05-nmi_timing.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_06_suppression,
    "ppu_vbl_nmi/06-suppression.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_07_nmi_on_timing,
    "ppu_vbl_nmi/07-nmi_on_timing.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_08_nmi_off_timing,
    "ppu_vbl_nmi/08-nmi_off_timing.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_09_even_odd_frames,
    "ppu_vbl_nmi/09-even_odd_frames.nes",
    30000000
);
blargg_test!(
    ppu_vbl_nmi_10_even_odd_timing,
    "ppu_vbl_nmi/10-even_odd_timing.nes",
    30000000,
    "baseline: Failed(3); Clock is skipped too late, relative to enabling BG; 08 07"
);
deferred_test!(
    read_joy3_count_errors,
    "read_joy3/count_errors.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    read_joy3_count_errors_fast,
    "read_joy3/count_errors_fast.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    read_joy3_test_buttons,
    "read_joy3/test_buttons.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    read_joy3_thorough_test,
    "read_joy3/thorough_test.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprdma_and_dmc_dma_sprdma_and_dmc_dma,
    "sprdma_and_dmc_dma/sprdma_and_dmc_dma.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprdma_and_dmc_dma_sprdma_and_dmc_dma_512,
    "sprdma_and_dmc_dma/sprdma_and_dmc_dma_512.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_01_basics,
    "sprite_hit_tests_2005.10.05/01.basics.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_02_alignment,
    "sprite_hit_tests_2005.10.05/02.alignment.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_03_corners,
    "sprite_hit_tests_2005.10.05/03.corners.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_04_flip,
    "sprite_hit_tests_2005.10.05/04.flip.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_05_left_clip,
    "sprite_hit_tests_2005.10.05/05.left_clip.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_06_right_edge,
    "sprite_hit_tests_2005.10.05/06.right_edge.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_07_screen_bottom,
    "sprite_hit_tests_2005.10.05/07.screen_bottom.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_08_double_height,
    "sprite_hit_tests_2005.10.05/08.double_height.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_09_timing_basics,
    "sprite_hit_tests_2005.10.05/09.timing_basics.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_10_timing_order,
    "sprite_hit_tests_2005.10.05/10.timing_order.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_hit_tests_2005_10_05_11_edge_timing,
    "sprite_hit_tests_2005.10.05/11.edge_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_overflow_tests_1_basics,
    "sprite_overflow_tests/1.Basics.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_overflow_tests_2_details,
    "sprite_overflow_tests/2.Details.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_overflow_tests_3_timing,
    "sprite_overflow_tests/3.Timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_overflow_tests_4_obscure,
    "sprite_overflow_tests/4.Obscure.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    sprite_overflow_tests_5_emulator,
    "sprite_overflow_tests/5.Emulator.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_1_frame_basics,
    "vbl_nmi_timing/1.frame_basics.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_2_vbl_timing,
    "vbl_nmi_timing/2.vbl_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_3_even_odd_frames,
    "vbl_nmi_timing/3.even_odd_frames.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_4_vbl_clear_timing,
    "vbl_nmi_timing/4.vbl_clear_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_5_nmi_suppression,
    "vbl_nmi_timing/5.nmi_suppression.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_6_nmi_disable,
    "vbl_nmi_timing/6.nmi_disable.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
deferred_test!(
    vbl_nmi_timing_7_nmi_timing,
    "vbl_nmi_timing/7.nmi_timing.nes",
    "legacy result protocol needs an adapter verified against upstream source"
);
