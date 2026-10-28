# TAK-RS fuzz workspace (`rust/fuzz/`)

Coverage-guided fuzzing of the `tak-cot` and `tak-proto` parsers and
conversions with [cargo-fuzz] and LLVM libFuzzer. This is development tooling.
None of it ships.

## Why a separate workspace

`libfuzzer-sys` compiles libFuzzer (C++) with `cc`. The product's pure-Rust
policy does not allow that. The Phase 1 plan (§1.8, question Q2) approved it as
a **development-only native-code exception**, limited to this directory:

| Control | Where |
|---------|-------|
| Not a member of the product workspace (`exclude = ["fuzz"]`) | `rust/Cargo.toml` |
| Own lockfile; `libfuzzer-sys` never in `rust/Cargo.lock` | `fuzz/Cargo.lock` |
| Own cargo-deny policy: `rust/deny.toml` plus marked `FUZZ-DELTA` blocks only (`cc` under `libfuzzer-sys`, NCSA licence for `libfuzzer-sys`, its build script) | `fuzz/deny.toml` |
| Own native-code allow-list, naming only `libfuzzer-sys` | `fuzz/native-allowlist.txt` |
| Pinned dated nightly (cargo-fuzz needs `-Zsanitizer`) | `fuzz/rust-toolchain.toml` |
| Drift and isolation check, run in `make policy` on every PR | `scripts/check-fuzz-policy.sh` |
| Non-required CI workflow | `.github/workflows/rust-fuzz.yml` |

The product gates (`make deny`, `make check-native-deps`, `make clippy`,
`make test`, the SBOM) never see this workspace.

## Targets

| Target | Input |
|--------|-------|
| `cot_parse` | CoT XML bytes |
| `cot_to_proto` | CoT XML bytes, then conversion to TAK Protocol v1 |
| `proto_decode` | TAK Protocol v1 `TakMessage` bytes |
| `proto_to_cot` | `TakMessage` bytes, then conversion to CoT XML |
| `proto_roundtrip` | `TakMessage` bytes, encoded and decoded again |
| `stream_frame` | Stream framing (`0xbf`, varint length, payload) |
| `javafmt` | Java-compatible number and time formatting strings |

Properties (plan WU9): no panic, no timeout (more than 1 s per input), no OOM
(`-rss_limit_mb=2048`). For inputs that parse, the targets also check the §1.8
idempotence and re-parse properties.

When you add a target, add it in three places: a `[[bin]]` entry here, the
`target` matrix in `rust-fuzz.yml`, and `targets=` in `scripts/fuzz-seeds.sh`.
`check-fuzz-policy` (rule F8) fails if these three lists differ.

## Running (from `rust/`)

You need cargo-fuzz 0.13.2 (`cargo install cargo-fuzz --version 0.13.2
--locked`) and the nightly pinned in `rust-toolchain.toml`. rustup installs
the nightly automatically when cargo runs in this directory. Do not use
`cargo +<toolchain>`.

| Command | What it does |
|---------|--------------|
| `make fuzz-build` | Policy check, `cargo fetch --locked`, then `cargo fuzz build --debug-assertions` (AddressSanitizer is on by default) |
| `make fuzz-seeds` | Writes `fuzz/seeds/<target>/` from `parity/corpus/cases/` |
| `make fuzz-run FUZZ_TARGET=cot_parse FUZZ_MAX_TOTAL_TIME=300` | Runs one target |
| `make fuzz-smoke` | Runs every target for `FUZZ_MAX_TOTAL_TIME` seconds (default 60) |
| `make fuzz-deny` | Fuzz policy check, `cargo deny` with `fuzz/deny.toml`, `cargo audit` on `fuzz/Cargo.lock`, and `check-native-deps.sh` with `fuzz/native-allowlist.txt` |

Gate evidence is 300 s per target with 0 crashes, 0 timeouts and 0 OOMs.
`rust-fuzz.yml` runs each target for 60 s on pull requests, 600 s nightly,
and for the `max_total_time` input on dispatch (default 300, allowed 30..3600).

Seeds larger than `FUZZ_SEED_MAX_BYTES` (default 65536) are skipped. Today
that drops three limit cases: the 8 MiB, 1 MiB and 400 KB inputs. libFuzzer's
`-max_len` is set to the same value. The size limits themselves are tested by
the `tak-cot` and `tak-proto` unit tests and proptest properties.

These directories are generated and never committed (`.gitignore`):

- `target/`
- `seeds/`
- `corpus/` (the working corpus)
- `artifacts/` (crash, timeout and OOM inputs)
- `coverage/`

To reproduce a crash, run
`cd fuzz && cargo fuzz run --debug-assertions <target> artifacts/<target>/<file>`.

## Updating

- **Nightly:** change `channel` in `rust-toolchain.toml` to a dated
  `nightly-YYYY-MM-DD`, then record the new pin in the CM-2 pin inventory
  (`docs/compliance/ATO_EVIDENCE.md`).
- **Dependencies:** Dependabot covers `/rust/fuzz`. Keep the crates shared with
  the product (the `tak-cot`/`tak-proto` graph) at the versions in
  `rust/Cargo.lock`. The easiest way is to copy `rust/Cargo.lock` here and run
  `cargo metadata`, which adds only the fuzz-only packages.
- **Policy:** edit `rust/deny.toml` first, then copy the change into
  `fuzz/deny.toml` outside the `FUZZ-DELTA` blocks. Widening the exception
  needs a plan entry, and `scripts/check-fuzz-policy.sh` must change with it.

[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz
