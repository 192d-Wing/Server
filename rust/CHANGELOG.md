# Changelog

All notable changes to the TAK-RS workspace (`rust/`) are recorded here.
The format follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and the workspace uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(`[workspace.package] version` in `Cargo.toml`). Before 1.0.0, a minor bump
may contain breaking changes. Nothing has been released yet.

## [Unreleased]

### Added

- Phase 0 Cargo workspace under `rust/`: edition 2024, `resolver = "3"`,
  `[workspace.package]` version 0.0.1 with `license = "Apache-2.0"` and
  `publish = false`, and `[workspace.lints]` with `unsafe_code = "forbid"` and
  strict clippy (`-D warnings` in CI).
- Toolchain pin `rust-toolchain.toml` (channel 1.99.1, MSRV `rust-version =
  "1.99"`, rustfmt, clippy and the `wasm32-unknown-unknown` target);
  `rustfmt.toml`, `clippy.toml` and `.cargo/config.toml` (AWS-LC FIPS build
  settings forced: `GOPROXY=off`, `GOTOOLCHAIN=local`).
- Stub crates `tak-api-types` (native and wasm32) and `tak-web-components`
  (Yew 0.23 `csr`, text-node rendering only), so CI builds both the native
  and the `wasm32-unknown-unknown` targets.
- Build entry point `rust/Makefile` (`fmt-check`, `clippy`, `clippy-wasm`,
  `build-wasm`, `test`, `test-doc`, `doc`, `deny`, `audit`, `supply-chain`,
  `policy`, `policy-selftest`, `markdownlint`, `sbom`, `ci`, parity targets).
- Supply-chain policy: `deny.toml` (crates.io only, no git sources; licence
  allow-list without copyleft; native-code crates banned except through
  their approved wrappers) and `.cargo/audit.toml`, kept in sync by
  `scripts/check-advisory-sync.sh`.
- Policy scripts under `rust/scripts/` (`check-unsafe`, `check-manifests`,
  `check-annotations`, `check-poam-ids`, `check-advisory-sync`,
  `check-native-deps` with `native-allowlist.txt`, `check-toolchain`,
  `check-workflows`, `ci-changes`, `ci-gate`) and their self-tests
  (`scripts/selftest.sh`, `make policy-selftest`).
- CI (`.github/workflows/rust-ci.yml`) on every pull request, push to
  `main`, merge queue and manual dispatch: a change detector, always-run
  workflow lint (actionlint, zizmor, `check-workflows`), and fmt, clippy
  (native and wasm32), wasm32 build, tests, doc tests, rustdoc, policy and
  shellcheck, cargo-deny, cargo-audit, native-dependency and markdownlint
  jobs. The always-run **`rust-ci-gate`** job is the single required check;
  a PR that touches only `src/**` passes it without running the Rust jobs.
- Nightly and manual supply-chain scan (`rust-supply-chain.yml`: audit,
  deny, native-deps) and Dependabot for `/rust` (Cargo) and GitHub Actions,
  with a 7-day cooldown on version updates.
- Best-effort CycloneDX SBOM job (`rust-sbom` artifact, 90-day retention);
  not part of the gate yet (POAM-2026-10-26-01).
- Compliance document skeletons in `docs/compliance/` against NIST SP 800-53
  Rev 5 (Release 5.2.0) and SP 800-53B, covering all 20 control families:
  `NIST_800-53_MAPPING`, `ASD_STIG_CHECKLIST`, `FIPS_COMPLIANCE`,
  `PROTOCOL_PARITY` (feature inventory, all "Not started"), `ATO_EVIDENCE`,
  `TEST_EVIDENCE` and `POAM`.
- Parity harness v0 (`rust/parity/`): builds the Java reference server from
  `src/` in a throwaway clone with pinned images and Gradle checksums, runs
  TAKCL suites against it with PostgreSQL/PostGIS, and verifies the JUnit
  results; `rust-parity.yml` runs it nightly, on demand and on
  `rust/parity/**` changes (never a required check).
- `LICENSE` (Apache-2.0), `NOTICE` (including the AWS-LC/OpenSSL notice),
  `ARCHITECTURE.md` and `CONTRIBUTING.md` with the provenance rules for code
  derived from the GPLv3 Java reference (behaviour, schemas and black-box
  tests only; no line-by-line translation; SPDX headers;
  POAM-2026-10-26-05).
- Phase 1a crate `tak-proto`: Rust types for TAK Protocol v1 and the related
  messages, generated at build time from the 15 `.proto` schemas read in
  place from `src/takserver-protobuf` by `protox` and `prost-build` (pure
  Rust; no `protoc` binary, nothing copied from `src/`). Bounded
  `decode_tak_message`/`encode_tak_message` (8 MiB default), TAK stream
  framing (`0xbf`, varint length, prefix capped at 5 bytes) and mesh
  framing, with typed, content-free `DecodeError` and `FrameError`. Builds
  for native and `wasm32-unknown-unknown`.
- `crates/tak-proto/schema.sha256`: `build.rs` fails the build when a schema
  is added, removed or changed, or when the compiled descriptor set holds a
  file the manifest does not cover (CRLF checkouts get a hint). A schema
  change from an upstream merge stays red until the manifest gets a reviewed
  update (tak-protocol-reviewer). The digest is a build-host change
  detector, outside the TOE; it is not an SI-7 or FIPS claim.
- Phase 1a crate `tak-cot`: CoT XML event model (`CotEvent`), bounded XML
  parser and serializer, Java-compatible number and time formatting
  (`javafmt`), and CoT XML ↔ TAK Protocol v1 conversion (`to_tak_message`,
  `from_tak_message`), including the `<detail>` split into typed
  sub-messages plus `xmlDetail`. Builds for native and
  `wasm32-unknown-unknown`. Unit, property (proptest), limit-boundary and
  hostile-input tests for both crates.
- Golden corpus inputs in `parity/corpus/`: synthetic Apache-2.0 cases
  (`MANIFEST.tsv`, one row per id and direction over `x2p`, `x2x`, `p2x`,
  `p2p` and five number tables; `cases/REUSE.toml`; `.gitattributes` with
  `* -text`), the Apache-2.0 Java capture driver `CorpusDriver.java`, and
  the harness commands `capture-corpus`, `verify-corpus` and
  `import-corpus` (`make parity-capture-corpus`). **The goldens are not
  captured yet**, so byte equivalence with the Java reference is not shown
  and the Phase 1a exit criterion is not met.
- Coverage-guided fuzzing in the excluded workspace `rust/fuzz/` (7 targets:
  `cot_parse`, `cot_to_proto`, `proto_decode`, `proto_to_cot`,
  `proto_roundtrip`, `stream_frame`, `javafmt`) with its own `Cargo.lock`,
  `deny.toml`, native allow-list and pinned `nightly-2026-10-23`;
  `cargo-fuzz` 0.13.2. Make targets `fuzz-build`, `fuzz-seeds`, `fuzz-run`,
  `fuzz-smoke` and `fuzz-deny`; `scripts/fuzz-seeds.sh` builds per-target
  seeds from the corpus inputs (inputs over 64 KiB skipped). Development
  only: libFuzzer is C++ (POAM-2026-10-27-01).
- Non-required workflow `rust-fuzz.yml`: a `fuzz-deny` job (fuzz policy,
  cargo-deny, cargo-audit, native allow-list) and one job per target under
  AddressSanitizer with debug assertions, `-timeout=1`,
  `-rss_limit_mb=2048` and `-max_len=65536`; 60 s per target on pull
  requests, 600 s nightly, 30–3600 s on dispatch. Crash inputs are
  uploaded. Dependabot also covers `/rust/fuzz`.
- `rust-parity.yml` captures the golden corpus after the Java build and
  compares it with `parity/corpus/golden/` (dispatch input `verify_corpus`,
  `false` only for the first capture), uploads the capture as
  `parity-corpus-<run>`, and also runs on changes to `tak-cot` and
  `tak-proto`.
- Policy checks in `make policy`: `check-fuzz-policy` (F1–F9: fuzz
  workspace excluded and complete, no `libfuzzer-sys` in the product graph,
  `fuzz/deny.toml` equal to `deny.toml` except three approved delta lines,
  a single allow-list row, a dated nightly, targets equal across manifest,
  CI matrix and seed script, and every `rust_inputs` prefix of
  `ci-changes.sh` in the `rust-fuzz.yml` pull-request paths) and
  `check-poam-markers` (every `POAM:` marker cites a
  `POAM-YYYY-MM-DD-NN` ID; NIST-010).
- Compliance documents for Phase 1a: mapping blocks SC-5, SI-10, SI-10(3),
  SI-11 (Partially Implemented) and SA-10(1) (Planned), SA-11(8) to
  Partially Implemented; six ASD STIG rows; `PROTOCOL_PARITY.md` Table C
  with the six deviations approved on 2026-10-27 (DEV-01a, DEV-01b, DEV-01c
  stream part, DEV-02, DEV-03, DEV-04) and the four approved on 2026-10-28
  (DEV-01c mesh part, DEV-01d, DEV-05, DEV-07); local QA and fuzz records in
  `TEST_EVIDENCE.md` (no CI evidence yet).

### Changed

- `harness.sh --version-tag <X.Y-RELEASE-N>|auto` replaces the workflow's local tag step.
- Compliance annotations must cite controls mapped in `NIST_800-53_MAPPING.md` (`check-annotations` rule (d)).
- `src/takserver-protobuf/` changes now run the Rust CI jobs (future
  `tak-proto` build input). New `make check-src-coupling` (in `make policy`)
  fails when code under `rust/` reads from `src/` or `utils/`, or links
  there, without the change detector knowing (SEC-008).
- New non-required CI job `annotations-strict` shows a pending
  compliance-mapping ID on every pull request; it is not part of
  `rust-ci-gate`. Rule (d) of `check-annotations` no longer counts a mapping
  block that disclaims its control, and a pending ID that the mapping
  disclaims always fails (NIST-013, NIST-014).
- Workspace members `tak-cot` and `tak-proto`, with `exclude = ["fuzz"]`.
  New workspace dependencies, all with default features off: `prost`,
  `quick-xml`, `ryu`, `thiserror` (linked); `prost-build`, `protox`
  (build only); `proptest` (tests only).
- `deny.toml` `allow-build-scripts` grows from 15 to 20 crates: `anyhow`,
  `getrandom`, `num-traits`, `rustix` (rustc and cfg probes only; build or
  dev graph) and `tak-proto` (first party, compiles the schemas).
- `tak-proto` and `tak-cot` are wasm32 roots: `make build-wasm`,
  `clippy-wasm` and `check-native-deps` cover them. Each has a
  `clippy.toml` copy of the web bans, guarded by a drift test.
- `check-unsafe` and the SPDX scan of `check-annotations` also cover
  `fuzz/fuzz_targets`; `check-annotations` and `check-src-coupling` skip
  fuzz build output, working corpus, seeds and crash inputs.
- `make check-annotations-strict` also runs `check-poam-markers`; both
  always run, and the target fails if either fails.
- `markdownlint` covers `fuzz/*.md`.
- `check-native-deps` judges the wasm32 crypto-crate rule on normal edges
  only, which is what ships (orchestrator decision, Phase 1a WU2): crypto or
  RNG crates that a wasm32 root reaches only through dev or build edges
  (such as `proptest`'s `rand`) no longer fail it. The `links` rule still
  checks the full graph.

### Fixed

- `check-annotations` rule (d) recognises `Not Applicable (component)`, case
  and emphasis variants, `None claimed` and `Not implemented` as disclaimers
  again (QA-007, NIST-016).
- `check-src-coupling` resolves `..` segments before judging a reference
  (QA-008).
- `check-poam-ids` also scans `.github/dependabot.yml`; the paused
  security-updates setting is marked with POAM-2026-10-26-10 (NIST-020).
- `check-native-deps.sh` no longer claims CM-7 (NIST-019).

### Security

- Every GitHub Action is pinned to a full commit SHA (annotated tags
  dereferenced) with a version comment; tools are installed at exact versions
  with checksum verification and no `cargo-binstall` fallback; actionlint is
  downloaded by checksum-pinned release.
- Least-privilege CI: top-level `permissions: contents: read` with no job
  escalation, `persist-credentials: false` on every checkout, no secrets, no
  `pull_request_target` or `workflow_run` triggers, and no event, input or
  `needs` expressions inside `run:` (enforced by `make check-workflows` and
  zizmor). Build caches are saved only from `main`.
- Fail-closed gate: the change detector falls back to running everything on
  any doubt, and `rust-ci-gate` accepts skipped jobs only when the detector
  succeeded and found no Rust-relevant change. `check-workflows` now fails
  when `rust-ci.yml` is missing.
- Pure-Rust native-dependency gate (`cargo deny` bans plus
  `check-native-deps.sh` per target): the only native code allowed is
  AWS-LC through `aws-lc-rs`, Linux only, never on wasm32. `aws-lc-fips-sys`
  is the FIPS boundary. `aws-lc-sys` (non-FIPS AWS-LC, pulled in by the
  rustls 0.23 `fips` feature) is a scoped second exception: linked but
  unused, outside the FIPS boundary, reachable only through `aws-lc-rs`
  (POAM-2026-10-26-08).
- Nightly supply-chain scan of `main` (cargo-audit with `--deny warnings`,
  cargo-deny, native-dependency allow-list) catches new advisories against
  an unchanged lockfile; failures are notified only by GitHub email for now
  (POAM-2026-10-26-07).
- Two advisory ignores for unmaintained transitive dependencies of `yew`
  0.23: `proc-macro-error` (RUSTSEC-2024-0370) and `bincode` 1.x
  (RUSTSEC-2025-0141), identical in `deny.toml` and `.cargo/audit.toml`.
  POAM-2026-10-26-09: accepted by the user on 2026-10-27 as an open POA&M for
  development, pending AO review.
- CI change detector is an allow-list (`src/`, `utils/`, top-level `*.md`/`*.txt`); every other path, including a repo-root `.cargo/` and any `.github/` change, runs the Rust jobs. Paths are read NUL-separated without git quoting.
- Parity harness rejects symlinks and special files in container output before the host reads it; `verify-junit` rejects DTDs in any encoding and never follows links. The parity Gradle cache holds downloaded modules only.
- `check-workflows` rejects job-level permission escalation, `continue-on-error` outside `sbom`, `github.head_ref`/`steps.*` in `run:`, and spoofed `rust-ci` job names in other workflows.
- `cargo-deny` rejects pre-built archives and build scripts outside an allow-list; `check-native-deps` fails on crypto crates in the wasm32 graph and on `web/` crates missing from the wasm32 roots.
- Clippy `disallowed-methods` and `disallowed-types` ban raw-HTML insertion,
  `eval`-style JavaScript, browser storage and browser dialogs in every
  crate, and the `std` time, thread and sleep calls that panic on wasm32 in
  the wasm32 crates (`web/clippy.toml`, `crates/tak-api-types/clippy.toml`),
  with drift-guard tests (WEB-002, WEB-003).
- CI workflow policy (`make check-workflows`) now checks the parsed YAML
  (Ruby stdlib psych) instead of single lines: quoted keys, a space before
  the colon, escapes, block scalars and flow style can no longer hide
  `continue-on-error`, job `permissions`, unpinned actions, template
  injection in `run:` or a job that impersonates `rust-ci-gate`. Duplicate
  keys and explicit tags are rejected; a missing Ruby fails the check
  (QA-006, SEC-004).
- The parity harness and the parity workflow make container output
  owner-readable before scanning it and treat an unreadable directory as a
  failure; the workflow uploads results only after the whole tree was
  listed and cleaned (SEC-009).
- `check-workflows`: `${{ }}` inside `run:` is now an allow-list (`matrix.*`,
  `runner.*`, `github.workspace|sha|run_id|run_attempt|repository`); index
  syntax, function calls, case variants and `env.*` fail (SEC-011).
- `check-workflows`: every workflow file (dotfiles and `.YML`/`.YAML`
  included) must declare top-level `permissions:` and must not grant
  `write-all`, `checks: write` or `statuses: write`; matrix values in rust
  workflows may not contain expressions, closing two `rust-ci-gate` spoofing
  paths (SEC-010).
- `check-workflows`: case variants of known workflow keys (`Name:`, `Uses:`,
  `Permissions:`) fail (SEC-012).
- `tak-cot` parser limits (CM-6, SC-5, SI-10; library defaults, no operator
  setting until Phase 3): 8 MiB input, depth 64, 100 000 nodes, 256
  attributes and 32 namespace declarations per element, 1024-byte names,
  1 MiB attribute values, 8 MiB text and an exact 8 MiB `xmlDetail` cap.
  DTDs, external entities and XML 1.1 are refused. Errors are typed and
  content-free, and the `Debug` output of events, elements and attributes
  holds no input data. Hostile input fails with an error, not a panic
  (tested; 7 fuzz targets run locally for 300 s each with 0 crashes; no CI
  fuzz run yet). Merging `xmlDetail` into the event is linear in the
  element count.
- `tak-proto` decoding is bounded: a message over the caller's limit, a
  length prefix over 5 bytes or above `u32`, and a known field with the
  wrong wire type are refused. The last two and the XML limits deviate from
  the Java reference on purpose; the user approved them on 2026-10-27
  (DEV-01a, DEV-01b, DEV-01c stream part, DEV-04) and on 2026-10-28 (DEV-01c
  mesh part, DEV-01d).
- `tak-cot` conversion rejects, where the Java reference relays or wraps: a
  proto string that XML 1.0 cannot carry (DEV-07) and a CoT time outside the
  64-bit millisecond range (DEV-05). Both approved by the user on 2026-10-28.
- `CotEvent::uid`, `type_` and `how` document that identity attributes
  match by local name, first match wins (`p:uid` and `uid` both present);
  Phase 3 must route authorization, filtering and persistence through these
  accessors or reject duplicates (D-18).
- No new native code in the product graph: the schemas compile without
  `protoc`, and `libfuzzer-sys` and `cc` exist only in `rust/fuzz/`, which
  is excluded from the workspace, the product lockfile and the SBOM.
  `check-fuzz-policy` fails if either reaches the product graph. The fuzz
  workflow has a read-only token, no caches and no secrets
  (POAM-2026-10-27-01: development-only libFuzzer exception, approved by
  the user 2026-10-27, Open).
- New POA&M entries for TOE code, both for Phase 3: POAM-2026-10-27-02 (one
  conversion can hold about 23 times its input in memory, and nothing caps
  conversions in flight) and POAM-2026-10-27-03 (serializer output is
  unbounded). POAM-2026-10-26-05 now also covers the build-time schema read,
  the Java-generated goldens and the Java driver (decision Q9);
  POAM-2026-10-26-02 also covers the corpus capture's unverified Java build
  downloads.
