# Mapper 0–5 test suite

Run all planned mapper families with one command:

```sh
cargo test --test mappers
cargo test --test mappers -- --list
cargo test --test mappers mmc1 -- --ignored --nocapture
cargo test --test mappers mmc3 -- --ignored --nocapture
cargo test --test mappers mmc5 -- --ignored --nocapture
```

Use `--include-ignored` to run enabled and ignored cases together. Known failures
have explicit prerequisites in `tests/support/mapper_cases.rs` and `tests/mappers.rs`.
Remove ignores after implementing the prerequisite and observing a pass.

## Coverage

| Mapper | Vendored ROMs and checks |
| --- | --- |
| 0, NROM | Holy Mapperel: 32 KiB PRG with 8 KiB CHR ROM or CHR RAM; fixed mapping, memory detection and RAM tests. Both full checks enabled. |
| 1, MMC1 | Nine Holy Mapperel configurations: 32/128 KiB CHR ROM, absent/volatile/battery PRG RAM, CHR RAM, and 512 KiB SUROM/SXROM with 8/32 KiB battery RAM. Seven 128 KiB PRG configurations and the SEROM submapper-5 fixed-PRG test pass and are enabled. SUROM/SXROM remain ignored. |
| 2, UxROM | Rainwarrior submappers 0/1/2 (256 KiB PRG, bus conflicts) plus Holy Mapperel (128 KiB PRG, CHR RAM). All four enabled. |
| 3, CNROM | Rainwarrior submappers 0/1/2 (four CHR banks, bus conflicts, absent PRG RAM) plus Holy Mapperel (32 KiB CHR). All full checks enabled. |
| 4, MMC3/MMC6 | Three Holy Mapperel configurations: 256 KiB CHR ROM or 8/32 KiB CHR RAM. Twelve existing blargg ROMs cover IRQ clocking, details, A12, scanline timing and MMC3/MMC6 variants. |
| 5, MMC5 | Rainwarrior RAM-capacity ROMs for 0/8/16/32/64/128 KiB, volatile and battery-backed declarations, plus mixed RAM. iNES 1 defaults and 256–1024 KiB characterization ROMs are staged with explicit policy/oracle prerequisites. |

There are **56 distinct ROM fixtures**: 44 here and 12 shared from
`../blargg/mmc3_test` and `../blargg/mmc3_test_2`. The latter are reused directly
by this suite, so they do not need another download or a separate command.
The additional mapper 0 tests in `tests/cartridge_memory.rs` remain available.

The twenty-three enabled ROM checks cover NROM/MMC1/UxROM/CNROM banking, CHR RAM,
PRG RAM absence, volatile/battery RAM access and UxROM/CNROM bus conflicts
(including overlapping partial checks). Runner regression tests are also enabled.
All other cases remain ignored for
stated prerequisites. This registers all six planned mapper families; it does
not imply those mappers are implemented or every aspect of their hardware is
covered. The ignored run after basic MMC1 support enabled seven more passing
cases; SEROM was subsequently verified and enabled after adding submapper-5
fixed PRG mapping. The remaining 38 cases retain their prerequisites: the two
512 KiB MMC1 ROMs are rejected by the current PRG-size limit, and mapper 4/5
and deferred policy cases remain unsupported.

## Fixture provenance

Upstream files are unmodified. Sources and original documentation are retained;
no assembler, archive extractor, or network access is needed to run the suite.
`SHA256SUMS` records all vendored files and the twelve shared MMC3 ROMs/readmes.
Verify from this directory:

```sh
sha256sum --check SHA256SUMS
```

- `2_test`, `3_test`, `mmc5ramsize`: Brad Smith / rainwarrior. `serom`: lidnariq.
  Source mirror: [perilsensitive/nes_test_roms](https://github.com/perilsensitive/nes_test_roms/tree/3eab7a917dda3638ee851021917f6395a0fb949c),
  pinned revision `3eab7a917dda3638ee851021917f6395a0fb949c`.
- `holy_mapperel`: Damian Yerrick, zlib license retained at
  `holy_mapperel/source/LICENSE`. Binaries from the upstream
  [v0.02 release](https://github.com/pinobatch/holy-mapperel/releases/tag/v0.02),
  archive `holy-mapperel-bin-0.02.7z`, SHA-256
  `70f85671e21f293599baebb662faeb06a4c04e9c9ceb283d96d4197f09e4ce7a`.
  Source tree pinned to `c022622274ca8b83d214dea97e4388a6b0e92d8a` from that tag.
  Only the selected 0–4 configurations are extracted; old duplicate filenames
  and the NROM 32 KiB CHR-RAM configuration are omitted.
- The shared MMC3 tests (Shay Green / blargg) come from
  [christopherpow/nes-test-roms](https://github.com/christopherpow/nes-test-roms/tree/95d8f621ae55cee0d09b91519a8989ae0e64753b),
  pinned revision `95d8f621ae55cee0d09b91519a8989ae0e64753b`.

The [NESdev emulator-test index](https://www.nesdev.org/wiki/Emulator_tests)
provides the upstream test-family descriptions and author links.

## Rainwarrior UxROM/CNROM expectations

| Header submapper | Conflict behavior | CNROM observations | UxROM observations |
| --- | --- | --- | --- |
| 0 | Unspecified; accept either known behavior | Either row below | Either row below |
| 1 | No conflicts | `03 00 03 03` | `0F 00 0F 0F` |
| 2 | AND conflicts | `00 00 01 02` | `00 00 01 02` |

Bank counts must be four (CNROM) or sixteen (UxROM); raw observations, detected
conflict kind and detected submapper must agree. Full CNROM assertions also
require `PRG RAM: NO`. Startup bank is unspecified by upstream and not asserted.

## Result adapter

These ROMs do **not** use blargg's `$6000` status protocol. The adapter in
`tests/support/mapper_roms.rs` uses the pinned assembly and linker layouts:

| ROM | Completion PC | Results in CPU RAM |
| --- | --- | --- |
| `2_test_*` | `$8352` | `$0300`: startup bank; `$0301`: bank count; `$0302–$0305`: conflict reads; `$0306`: conflict kind; `$0307`: detected submapper |
| `3_test_*` | `$8365` | `$0300`: startup bank; `$0301`: PRG RAM present; `$0302`: bank count; `$0303–$0306`: conflict reads; `$0307`: conflict kind; `$0308`: detected submapper |

Both completion addresses are the final self-jump in `testing`, after printing
all results. UxROM selects bank `$0F` before that jump. Both linker layouts
reserve `$0200–$02FF` for OAM and place result RAM immediately afterward.
Conflict kind is `0` for none, `1` for AND, and `2` for unrecognized behavior.

The runner checks the completion-loop bytes before loading, runs the normal CPU
and PPU, and reads result RAM without advancing emulation only after completion.
Timeouts (five million CPU cycles), stalled execution, and emulator panics fail
the test. Reaching completion alone is insufficient: assertions must also pass.
The addresses are specific to these builds; update the adapter and checksums
together if replacing the upstream ROMs. No assembler or network is needed to
run the tests.

## Additional result adapters

### Holy Mapperel

`tests/support/holy_mapperel.rs` stops at `$C01B`, immediately after
`JSR driver_mapper_test` returns in `main.s::reset2`, where the source states
that all tests are complete. This precedes screen formatting and beep codes.
The adapter checks the sentinel stores and call bytes in each ROM first.

Zero-page locations follow the `makefile` link order and `nrom256.x` allocation:

| Address | Result |
| --- | --- |
| `$15` | Detected mapper |
| `$17` | PRG size in 4 KiB banks minus one |
| `$18–$1A` | CHR ROM flag, CHR size in 8 KiB banks minus one, CHR test result |
| `$1B–$1E` | Saved-data flag, PRG RAM present, RAM size in 8 KiB banks minus one, RAM test result |
| `$1F–$20` | Detailed PRG/WRAM and CHR/IRQ error flags |

Expected sizes and mapper come from the pinned NES 2.0 headers. CHR tests and
both detailed error bytes must be zero. Full assertions also check RAM presence,
size and RAM test result. Battery declarations test allocation and access on a
fresh boot; this suite does not test persistence to disk or power cycling.
The budget is 120 million CPU cycles for RAM sweeps.

### MMC1 SEROM

The pinned `serom.c` initializes `fail`, performs three fixed-mapping checks and
then loops forever. In the compiled ROM, `fail` is at `$0330` (the initialization
is `A9 00 8D 30 03` at `$C154`), and the final self-jump is at `$C375`.
The adapter validates those instruction bytes, waits for completion, and requires
`fail == 0`. This is a submapper-5 test, not a generic MMC1 banking test.

### MMC3/MMC6

The shared blargg runner requires signature `DE B0 61` at `$6001–$6003` and a
passing final status at `$6000`. Its old NROM-only PRG/CHR size restriction has
been removed, so mapper support can be added without an artificial runner
barrier. See `../blargg/README.md` for resets and diagnostics. MMC6 and alternative
MMC3 revisions remain separate tests; they are not expected to all describe the
same chip variant.

### MMC5 RAM capacity

`mmc5ramsize.s` writes descending bank tags through `$5113`, probes address lines
and RAM retention, then displays the results. `mmc5ramsize.cfg` places the 128
result bytes at `$0200–$027F`; the final self-jump is `$C35C`. The adapter validates
that loop and waits for it before reading RAM. `$FF` means the selected bank did
not behave as RAM. Other values identify the lowest register value aliasing a
physical bank; bit 7 indicates data saved before startup.

For fresh boots, non-`$FF` values must have no saved-data flag, their representative
bank must refer to itself, and the number of distinct working banks times 8 KiB
must equal the declared capacity. This checks capacity and the ROM's RAM probes,
not exact PCB-specific chip-select/alias placement. Noncontiguous tags such as
`0` and `4` are valid for two separate chips. Volatile, battery and mixed RAM
headers are included; persistent save-file behavior is not asserted.

The 256–1024 KiB variants exceed the standard MMC5's four RAM bank address bits.
They are retained as characterization fixtures, with a failing deferred case
until an explicit extension policy and expected result are chosen. Likewise,
iNES 1 fixtures need a default-capacity policy. These are not silently assigned
a guessed pass condition. See [MMC5 PRG banking](https://www.nesdev.org/wiki/MMC5#PRG_Bankswitching_.28.245113-.245117.29).
