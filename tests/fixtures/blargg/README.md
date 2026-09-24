# Blargg test ROMs

Pinned from [christopherpow/nes-test-roms](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b), commit `95d8f621ae55cee0d09b91519a8989ae0e64753b`. The ROMs and upstream readmes are unmodified. Local paths flatten `rom_singles/` and rename `instr_test-v3`/`instr_test-v5` to `instr_test_v3`/`instr_test_v5`.

The fixtures cover CPU, PPU, APU/DMC, reset, MMC3, and PAL tests, including older revisions. Related suites (`cpu_dummy_writes`, `cpu_exec_space`, and `ppu_read_buffer`) include work by Joel Yliluoma/Bisqwit; see upstream readmes for attribution.

## Running

```sh
cargo test --test blargg
cargo test --test blargg -- --list
```

Known failures have explicit ignore reasons in `tests/support/blargg_cases.rs`. Run them while working on a suite:

```sh
cargo test --test blargg ppu_vbl -- --include-ignored --nocapture
cargo test --test blargg instr_test_v5 -- --include-ignored --nocapture
cargo test --test blargg ppu_vbl_nmi_06_suppression -- --ignored --exact --nocapture
```

The original names `cpu_basics` (v5's `01-basics`) and `ppu_vbl_basics` are preserved. Both run by default. The `instr_test_v5` filter therefore excludes `cpu_basics`.

Run an external ROM using the supported result protocol:

```sh
BLARGG_ROM=/absolute/path/to/test.nes cargo test --test blargg external_rom -- --ignored --exact --nocapture
```

Add or edit Rust test declarations directly in `tests/support/blargg_cases.rs`. Remove a test's ignore reason when it passes. There is no generation step.

## Limits

The runner starts at the reset vector through `NES::step()`. It requires the `DE B0 61` signature at `$6001–$6003`, reads status from `$6000`, and bounds diagnostic text reads from `$6004` at the end of PRG RAM. Reset requests wait at least 100 ms of emulated NTSC time and do not renew the cycle budget.

Currently supported cartridges use mapper 0, 16/32 KiB PRG ROM, 8 KiB CHR ROM, and horizontal/vertical mirroring. Other layouts are rejected. Most tests get 30 million CPU cycles; longer instruction/OAM suites get 120 million. The external-ROM helper uses 30 million.

Legacy screen/beep tests need verified result adapters; audio/visual tests need human or dedicated output validation. Their `deferred_test!` declarations are ignored placeholders: explicitly running one reports the prerequisite without executing the ROM. Completion is not treated as a pass for those fixtures.

A pass establishes only that ROM's assertions. In particular, passing an APU flag-clear test does not establish working audio while APU reads are stubbed to zero. Fine NMI timing, APU functionality, additional mappers, and PAL timing remain emulator work.
