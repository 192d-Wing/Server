# Quality Gates

Run from `rust/`. Scope to touched crates (`-p <crate>`) while iterating; run
the workspace form once before the final gate. When `rust/Makefile` (or
`justfile`) exists, its targets are authoritative and this table should be
updated to point at them.

| Gate | Iterating | Final | Required |
|------|-----------|-------|----------|
| Format | `cargo fmt --all` | `cargo fmt --all --check` | Yes |
| Lint | `cargo clippy -p <crate> --all-targets -- -D warnings` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Yes |
| Lint (WASM) | `cargo clippy -p <web-crate> --target wasm32-unknown-unknown -- -D warnings` | same, all `rust/web/*` crates | When `rust/web/**` changed |
| Unit tests | `cargo test -p <crate> --lib` | `cargo test --workspace --lib` | Yes |
| Doc tests | `cargo test -p <crate> --doc` | `cargo test --workspace --doc` | When public API changed |
| WASM tests | `wasm-pack test --headless --firefox rust/web/<crate>` | all web crates | When `rust/web/**` changed |
| Frontend build | `trunk build` in the app dir | `trunk build --release` (record bundle size) | When `rust/web/**` changed |
| Integration | `cargo test -p tak-integration-tests --test <name>` | `cargo test -p tak-integration-tests` | When service behaviour changed (needs Postgres+PostGIS) |
| Parity | `cargo test -p tak-parity --test <name>` | full parity suite against a reference Java TAK Server | `port` mode; any wire/API change |
| Supply chain | — | `cargo deny check` + `cargo audit` | When `Cargo.toml`/`Cargo.lock` changed |
| Fuzz smoke | `make fuzz-smoke` (`cargo fuzz run <target> -- -max_total_time=60` in `rust/fuzz/`, pinned nightly) | all targets, 5 min each | When a parser changed (CoT XML, protobuf framing, multipart, JWT, X.509 input) |
| Golden corpus | `make test` (runs `tak-cot/tests/golden.rs` against `rust/parity/corpus/golden/`); CI: `harness.sh capture-corpus verify-corpus` in `rust-parity.yml` | 0 mismatches outside approved deviations | When tak-cot, tak-proto or the corpus changed |
| No `unsafe` | `grep -rn "unsafe" rust/crates rust/services rust/web` | every hit has `// SAFETY:`; `#![forbid(unsafe_code)]` on crates that need none | Yes |
| Compliance annotations | — | count of `COMPLIANCE MAPPING:` should not drop | Yes for code changes |
| Markdown | — | `make markdownlint` | When docs changed |
| Policy | `make policy` | `make policy` + `make policy-selftest` | Yes (unsafe, manifests, annotations/SPDX, POAM IDs, advisory sync, native deps, toolchain, workflows) |
| Provenance | — | provenance checklist (`rust/CONTRIBUTING.md` §Provenance) answered for every port | `port` mode; any ported behaviour |

Once `rust/Makefile` exists, prefer its targets (`make fmt`, `make clippy`,
`make test`, `make deny`, `make audit`, `make ci`) over the raw commands above.

## Parity testing

The parity harness drives the same input at a reference Java TAK Server
(container built from `src/`, version recorded in the report) and at the Rust
server, then compares outputs after normalising fields that legitimately
differ (timestamps, UIDs it generates, ordering where the protocol allows).

- Record the Java server version/commit and the normalisation rules used.
- A difference that is not normalised is a FAIL unless `parity-map.md` lists
  it as an approved deviation.

## Rules

- Record every command with exit status and the last relevant output lines in
  `qa-report.md`. "Tests pass" without the command is not evidence.
- Missing environment (Postgres/PostGIS, Java reference server, browser for
  WASM tests) → gate is **NOT RUN** with the reason, never PASS.
- No new `#[allow(clippy::...)]`, `#[ignore]`, or `unwrap()`/`expect()` on
  untrusted input to get green without a justification comment and a finding.
- Final workspace-wide gates run in the background.
