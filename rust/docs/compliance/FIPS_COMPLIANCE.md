# FIPS Cryptographic Standards Compliance

**Document Version:** 0.4
**Date:** 2026-10-28
**TAK-RS Version:** 0.0.1 (Phase 1a, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 1a build run. No cryptography is implemented in the product; every standard is Planned. The Phase 1a build-host schema digest is recorded as outside the TOE and not a FIPS claim (CRY-001).
**Run:** `20261027-180458-phase1-protocol-config`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** (SC-13, SC-8, IA-7) |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed; [ROADMAP.md](../../ROADMAP.md) §6 Q5) |
| Overlay | CNSSI 1253 (TBD) |
| STIG | DISA ASD STIG, release TBD |
| Module standard | FIPS 140-3 |

### Scope

Phase 0: development environment only. Phase 0 contains **no crypto code and
no crypto dependency**. Phase 1a adds the TOE library crates `tak-proto` and
`tak-cot`, which call no cryptographic primitive, open no TLS session,
validate no certificate and depend on no crypto crate; the `check-native-deps`
wasm32 crypto-name rule passed on their closure in the Phase 1a QA run
(2026-10-28). This document records the decided provider, the build pins
already in place, and what Phase 2 (`tak-crypto`) must deliver.

### Vocabulary

Implementation status: **Implemented**, **Partially Implemented**,
**Planned**, **Alternative Implementation**, **Not Applicable**, plus the
overlay **Risk Accepted (POAM-…)**. See
[NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) §Vocabulary.

## Cryptographic Module

Decision D5 ([ROADMAP.md](../../ROADMAP.md) §2): rustls with an
**aws-lc-rs `fips`** `CryptoProvider` is the single crypto provider for all of
TAK-RS. There is no provider-agnostic seam.

| Item | Value |
|------|-------|
| Provider | rustls `CryptoProvider` from `aws-lc-rs` with the `fips` feature |
| FIPS module crate | `aws-lc-fips-sys` (AWS-LC-FIPS), reachable only through `aws-lc-rs` |
| FIPS boundary | `aws-lc-fips-sys` **only** |
| Module version | TBD (Phase 2); see [Known issues](#known-issues-for-phase-2) |
| CMVP certificate | TBD (Phase 2) |
| Operating environments claimed | **Linux x86_64 and aarch64, static build only** |
| Not claimed | macOS and Windows (AWS-LC builds the FIPS module as a shared library there; development and tests only); `wasm32-unknown-unknown` (crypto is excluded from the wasm graph) |
| Status | Planned (Phase 2) |

### Non-FIPS code in the graph

From Phase 2, the rustls 0.23 `fips` feature also builds the non-FIPS AWS-LC
library (`aws-lc-sys`). It is linked but not selected when `fips` is on. It is
**outside the FIPS boundary and out of scope** for every FIPS claim in this
document. It is allowed only through `aws-lc-rs`, only on Linux, and never on
wasm32 (POAM-2026-10-26-08).

### Build pins (in place, inert, not exercised by any gate)

Set in [.cargo/config.toml](../../.cargo/config.toml) with `force = true`.
They are inert until a crate depends on `aws-lc-rs` (Phase 2), so no Phase 0
gate shows that they work; the first evidence is the Phase 2 FIPS build.
The PR 7 CI run (2026-10-27,
[rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255))
confirms this: its `check-native-deps` step reports `aws-lc-fips-sys`,
`aws-lc-sys` and `aws-lc-rs` as allow-listed but "not in graph" for x86_64
and aarch64 Linux, so that run is no evidence for any FIPS claim.

| Setting | Value | Purpose |
|---------|-------|---------|
| `AWS_LC_FIPS_SYS_USE_SYSTEM` | `0` | Always build the bundled, `Cargo.lock`-pinned module source; never a system libcrypto |
| `GOTOOLCHAIN` | `local` | The FIPS build's Go tooling may not download another Go toolchain |
| `GOPROXY` | `off` | The FIPS build may not download Go modules |

`AWS_LC_FIPS_SYS_SYSTEM_DIR` is deliberately not set.

### Build prerequisites (Phase 2)

| Tool | Requirement |
|------|-------------|
| CMake | 3.5 or later |
| Go | 1.17.13 or later; mandatory for the FIPS build |
| Perl | Perl 5; mandatory for the FIPS build |
| C compiler | clang or gcc |
| libclang | Only when bindings must be regenerated; pregenerated bindings exist for the Linux and macOS x86_64 and aarch64 targets |

From Phase 2, CI logs `cmake --version`, `go version`, `perl -v` and
`cc --version` as CM-2 evidence. The tools come from the signed Ubuntu archive
(SR-11).

### Planned module checks (Phase 2)

- Startup check that the module reports FIPS mode
  (`aws_lc_rs::try_fips_mode()`); the process fails closed otherwise.
- Startup assertion that every rustls `ServerConfig` and `ClientConfig`
  reports `fips()`; the process fails closed otherwise (SC-13).
- A test that the rustls provider in use is the FIPS provider
  (POAM-2026-10-26-08).

## Standards

| Standard | Use in TAK-RS | Provider | Status |
|----------|---------------|----------|--------|
| FIPS 140-3 | Cryptographic module validation | AWS-LC-FIPS (`aws-lc-fips-sys`) | Planned (Phase 2) |
| FIPS 180-4 | SHA-2 hashing (TLS, certificates) | AWS-LC-FIPS | Planned (Phase 2) |
| FIPS 186-5 | ECDSA and RSA signatures (TLS, certificates, CA operations) | AWS-LC-FIPS | Planned (Phase 2) |
| FIPS 197 | AES (TLS AES-GCM suites) | AWS-LC-FIPS | Planned (Phase 2) |
| FIPS 198-1 | HMAC (TLS PRF and HKDF) | AWS-LC-FIPS | Planned (Phase 2) |
| SP 800-52 Rev. 2 | TLS configuration | rustls with the FIPS provider | Planned (Phase 2) |
| SP 800-56A Rev. 3 | ECDH key establishment | AWS-LC-FIPS | Planned (Phase 2) |
| SP 800-56B Rev. 2 | RSA key establishment | AWS-LC-FIPS | Planned (Phase 2) |
| SP 800-56C Rev. 2 | Key derivation | AWS-LC-FIPS | Planned (Phase 2) |
| SP 800-90A Rev. 1 | Deterministic random bit generation | AWS-LC-FIPS | Planned (Phase 2) |

Integrity checks used by the developer tooling (SHA-256 of the Gradle
distribution, the wrapper jar and `rust/LICENSE`; OCI image digests; Cargo
checksums; git commit SHAs) are supply-chain controls (SR-11), not TOE
cryptography, and are not FIPS claims.

**Build-host schema digest (Phase 1a, CRY-001).** The `tak-proto` build
script ([crates/tak-proto/build.rs](../../crates/tak-proto/build.rs)) computes
SHA-256 over the 15 `.proto` schemas it reads in place and compares the
digests with the committed manifest
[crates/tak-proto/schema.sha256](../../crates/tak-proto/schema.sha256),
failing the build on any difference. The SHA-256 is written in the build
script to FIPS 180-4 and self-tested against the standard's test vectors
before use. It is **not** a validated module and **not** a FIPS claim:

- it runs on the build host only and is linked into no TAK-RS artifact;
- it is unkeyed and protects nothing: anyone who can change a schema can
  change the manifest, and review of that change is the control;
- it is a developer configuration-management change detector, mapped to
  CM-3, SA-10(1) and SR-3 (with SA-15), not to SC-13 and not to SI-7 or its
  enhancements; the single-provider rule (D5) covers product cryptography
  and does not apply to it.

The crypto-tls-reviewer approved this (findings CRY-001, 2026-10-28) on one
condition: if the digest is ever relied on against a malicious upstream, used
as an integrity mechanism of the TOE (SI-7(6)), or moved into a runtime
crate, it moves to the aws-lc-rs FIPS provider. The `sha256sum`/`shasum`
digests planned for the corpus `PROVENANCE.json` are likewise CI integrity
tooling outside the TOE, not a FIPS claim.

## TLS Policy

Placeholder; the Phase 2 `tak-crypto` plan owns the design.

| Item | Planned value | Status |
|------|---------------|--------|
| Protocol versions | TLS 1.2 and TLS 1.3 only (fielded ATAK needs TLS 1.2) | Planned (Phase 2) |
| Cipher suites | Only suites the FIPS provider exposes: ECDHE with AES-GCM; no ChaCha20-Poly1305 | Planned (Phase 2) |
| Key-exchange groups | P-256, P-384; post-quantum hybrids only if the provider marks them FIPS | Planned (Phase 2) |
| Certificate validation | rustls-webpki with the FIPS provider; path building to the configured TAK CA set; CRL checking (`tak-auth`); no system roots unless configured | Planned (Phase 2) |
| Fail-closed FIPS assertion | `fips()` checked at startup | Planned (Phase 2) |

The parity harness runs the **Java reference server** with the upstream test
PKI and TLS 1.2. Those are behaviours of the reference under test, not TAK-RS
settings.

## Known issues for Phase 2

1. **Non-FIPS `aws-lc-sys` in the rustls `fips` graph** (plan Open question 2,
   resolved by option (a) on 2026-10-26). Tracked as POAM-2026-10-26-08.
2. **Module version discrepancy.** The `aws-lc-fips-sys` 0.14.3 README says it
   binds AWS-LC-FIPS 3.0.x, while the `aws-lc-rs` 1.18.2 crate documentation
   says 4.x (`fips-2025-09-12-lts`). Phase 2 must decide from the crate's
   `package.metadata.aws-lc-fips-sys.commit-hash` and the CMVP listing, then
   pin the exact `aws-lc-fips-sys` version and record the module version and
   certificate here.
3. **Static build only on Linux.** FIPS claims are limited to Linux x86_64 and
   aarch64 static builds. macOS builds are for development and tests.
4. **The FIPS dependency graph fails `cargo deny check bans` today (QA-004).**
   `deny.toml` `[bans.build]` sets `executables = "deny"` (and, since fix
   round 1, `include-archives = true`). With `aws-lc-rs` `fips` in the graph,
   the build-time chain `bindgen` → `clang-sys` → `libloading` 0.8.9 brings
   two pre-built test DLLs, `tests/nagisa32.dll` and `tests/nagisa64.dll`,
   which that rule rejects. The Phase 2 `tak-crypto` plan must add a
   checksum-pinned bypass for exactly these two files (SHA-256
   `5c69b2bd9c8a6ad04165c221075fc9fade1dd66ca697399ace528a5a62328e36` for
   `nagisa32.dll` and
   `e20b95e3036f3289421abd100760874d4f455afd33c3b5b64fec56b191f7d477` for
   `nagisa64.dll`, from the release-engineer's WU3 scratch check; re-verify
   against the version that resolves), not turn the rule off.
   The WU3 scratch table (`impl-notes-wu3-wu4.md`, the `aws-lc-rs` `fips`
   probe row) records `bans ok` for this graph, while the same notes report
   that `[bans.build]` flags the two DLLs on it. The two statements conflict,
   and QA-004 (QA Round 0) reports the failure. The PASS row is therefore
   **not** evidence that the FIPS graph passes the current policy. QA did not
   re-run the probe in Round 1.
5. **`allow-build-scripts` must grow in Phase 2.** Since fix round 1,
   `[bans.build]` allows build scripts only for the crates in today's graph
   (15 in Phase 0, 20 since Phase 1a added `anyhow`, `getrandom`,
   `num-traits`, `rustix` and `tak-proto`, none of which compiles C). `aws-lc-fips-sys`, `aws-lc-sys`, `aws-lc-rs` (if it has one) and
   their build-time toolchain crates (for example `cmake`, `cc`, `bindgen`,
   `clang-sys`) must be added by name in the `tak-crypto` plan,
   each reviewed. The list names crates, not versions, so a later version
   with a new build script relies on `Cargo.lock` and Dependabot review
   (SEC-007 residual).

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-28 | 0.4 | Phase 1a (WU10). Scope: `tak-proto` and `tak-cot` contain no cryptography and no crypto dependency. Standards section: the `tak-proto` build-script SHA-256 schema digest recorded as a build-host change detector (FIPS 180-4, self-tested, not validated, not linked into any artifact, protects nothing; CM-3, SA-10(1), SR-3, SA-15; not SC-13, not SI-7), with the crypto-tls-reviewer's condition for moving it to aws-lc-rs FIPS (CRY-001). Known issue 5: 20 build-script crates since Phase 1a. No status change: every standard stays Planned (Phase 2) | `20261027-180458-phase1-protocol-config` (Phase 5, WU10) |
| 2026-10-27 | 0.3 | PR 7 CI: Build pins section notes that run 40539495255 shows no AWS-LC crate in the graph, so the pins stay unexercised. No status change: every standard stays Planned (Phase 2) | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-26 | 0.2 | Phase 5 (pre-gate). Known issues 4 (QA-004: FIPS graph fails `[bans.build]` on two `libloading` test DLLs; the WU3 scratch PASS is not evidence) and 5 (`allow-build-scripts` must list the AWS-LC crates in Phase 2; SEC-007 residual). Build pins described as inert and not exercised by any gate. No status change: every standard stays Planned (Phase 2) | `20261026-210255-phase0-foundations` (Phase 5) |
| 2026-10-26 | 0.1 | Initial skeleton: provider (aws-lc-rs `fips`), FIPS boundary limited to `aws-lc-fips-sys` with `aws-lc-sys` out of scope (POAM-2026-10-26-08), Linux-only operating environments, build pins and prerequisites, standards table (all Planned), TLS policy placeholder, known issues | `20261026-210255-phase0-foundations` (WU6) |
