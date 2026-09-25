# Legacy result adapters

All 77 former adapter placeholders have a bounded execution path. Passing ROMs
run by default; failures, unsupported hardware, and diagnostics remain ignored
with specific reasons in `tests/support/blargg_cases.rs`.

These adapters apply to the fixtures pinned at upstream commit
[`95d8f621ae55cee0d09b91519a8989ae0e64753b`](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b).
The ROM files are unmodified. `tests/support/legacy_fixtures.rs` records each
file's FNV-1a fingerprint and reporting addresses. A changed file is rejected
until its adapter is re-verified. This fingerprint is an identity check, not a
security signature.

The observer runs between CPU actions. It accepts a result only at the verified
reporting entry point, never merely because RAM contains zero, one, or a test
number. It does not patch the ROM, alter emulated RAM, or depend on rendered
pixels. The controller fixture receives input through the normal button API.
Budgets, load errors, emulator panics, and no-progress handling use the existing
Blargg runner. PAL fixtures are reported as unsupported instead of being judged
using NTSC timing.

## Protocol verification

| Fixtures | Result contract and evidence |
| --- | --- |
| Sprite hit (11), sprite overflow (5), branch timing (3), legacy VBL/NMI (7), legacy MMC3 (6), PAL APU (10) | `validation.a`/`.asm` sets `$F8 = 1` on success and enters `report_final_result`; other nonzero values are failure codes. Zero is invalid. The entry is `$E00B` for sprite hit and `$E01D` for the other pinned binaries. |
| Legacy NTSC APU (11), legacy PPU (5) | Test sources assign `result`, using 1 for success, then jump to `report_final_result`. The supplied prefix files describe the reporting interface but omit parts of its implementation. Inspection of each binary confirms the variable is `$F0`, its success branch, and its reporting target; targets vary by ROM. |
| Legacy combined CPU tests (2) | `common/shell.a` receives the result in A at `exit`, with zero meaning success. The pinned multi-test shell enters `$81FA` after its overall error count check. Individual banked subtest exits are not completion of the combined suite. |
| CPU dummy reads (1), three DMC/PPU tests, controller button and thorough tests (2) | `common/shell.inc` receives A at `exit`; `common/testing.s` uses zero for success and nonzero for failure. These cases have explicit assertions. |
| `dma_2007_read`, `double_2007_read` | Their source prints a CRC and returns without checking it. At exit the adapter reads the complement of the little-endian CRC state at `$10..$13` and compares against the alternatives documented in each source file. A zero exit code alone is insufficient. |
| `cpu_timing_test6` | Source `done` at `$E1B7` reports success; `error_beep_exit` at `$E0B0` reports failure. Failures include the opcode and test mode from `$13/$14`. The default run selects official instructions; branches are covered by the separate branch suite. |
| `read_joy3/test_buttons` | Source `main` asks for A, B, Select, Start, Up, Down, Left, Right. The adapter supplies the requested `$1D` button at the press-loop call `$E094`, then releases it at the release-loop call `$E0A0`. The ROM still checks the returned button and determines the result. |
| `read_joy3/count_errors*` | Sources print a count without an expected value. At exit, nonzero A is a failure; zero produces `NeedsValidation` with the measured `$1D` count, never a pass. |
| `dmc_tests/*` (4) | Upstream provides binaries without source or a documented numeric result protocol. Binary inspection identifies their terminal loops, but completion is only `NeedsValidation`: these are audio diagnostics, not verified pass/fail assertions. |
| `sprdma_and_dmc_dma/*` (2) | Binary inspection confirms the existing `$6000` status and `$6001..$6003` signature protocol. The adapter retains diagnostics and refuses to count a status-zero `Done`/measurement-only exit as a pass. Both currently time out while waiting with DMC unimplemented. |

Relevant pinned sources:

- [Sprite-hit validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/sprite_hit_tests_2005.10.05/source/runtime/validation.a)
- [Sprite-overflow validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/sprite_overflow_tests/source/validation.a)
- [Branch validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/branch_timing_tests/source/validation.a)
- [VBL/NMI validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/vbl_nmi_timing/source/support/validation.a)
- [MMC3 validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/mmc3_irq_tests/source/validation.asm)
- [PAL validation](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/pal_apu_tests/source/validation.a)
- [Legacy APU sources](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b/blargg_apu_2005.07.30/source)
- [Legacy PPU sources](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b/blargg_ppu_tests_2005.09.15b/source)
- [Combined CPU shell](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/blargg_nes_cpu_test5/source/common/shell.a)
- [CPU timing source](https://github.com/christopherpow/nes-test-roms/blob/95d8f621ae55cee0d09b91519a8989ae0e64753b/cpu_timing_test6/source/cpu_timing_test.asm)
- [DMC/PPU sources and expected CRCs](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b/dmc_dma_during_read4/source)
- [Controller sources](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b/read_joy3/source)

## Recheck results

Of the 77 former placeholders, **33 pass**, **20 report failed assertions**,
**16 require unsupported hardware**, **6 need output validation**, and **2 time
out**. No emulator behavior was changed for this audit.

| Newly passing group | ROMs |
| --- | ---: |
| Sprite hit, including all three timing tests | 11 |
| Legacy VBL/NMI | 7 |
| Legacy PPU: palette RAM, sprite RAM, VBL clear, VRAM access | 4 |
| Branch timing | 3 |
| Combined legacy CPU instruction tests | 2 |
| CPU dummy reads and CPU timing | 2 |
| DMC/PPU: `dma_2007_write`, `read_write_2007` | 2 |
| Controller: scripted buttons, thorough routine test | 2 |

The remaining 44 break down as follows:

- 11 legacy APU failures: length counters and frame IRQ behavior are absent.
- 5 sprite-overflow failures: sprite evaluation/overflow is absent.
- 3 PPU/DMC read failures: `dma_2007_read` yields CRC `498C5C5F`,
  `double_2007_read` yields `D84F6815`, and `dma_4016_read` fails its CRC assertion.
  The double-read test does not itself require DMC; its failure concerns
  consecutive `$2007` reads.
- 1 power-up-palette mismatch. The source explicitly compares against one
  author's console-specific table; this is not a universal power-up requirement.
- 6 MMC3 tests cannot load mapper 4.
- 10 PAL APU tests require PAL CPU/APU timing.
- 4 DMC audio diagnostics and 2 controller conflict-count measurements need
  output validation; none is counted as passing.
- 2 combined sprite/DMC DMA ROMs time out at `$E29D`, status `$80`, with DMC absent.

Passing the controller routine or a test that expects writes to be unaffected
does not establish working DMC DMA. The emulator still lacks DMC. Likewise,
sprite-hit tests exercise the CPU-visible hit flag, not RGB sprite compositing.

Run enabled tests normally:

```sh
cargo test --test blargg
```

Recheck an ignored legacy suite with its adapter:

```sh
cargo test --test blargg sprite_overflow_tests -- --include-ignored --nocapture
cargo test --test blargg blargg_apu_2005 -- --include-ignored --nocapture
```

Recheck all automated/diagnostic ignored entries, omitting the original nine
audio/visual placeholders and the external-ROM helper (failures are expected):

```sh
cargo test --test blargg -- --ignored --skip external_rom --skip apu_mixer --skip full_palette --skip nmi_sync
```
