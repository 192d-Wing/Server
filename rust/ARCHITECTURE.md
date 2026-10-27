# TAK-RS Architecture

Short map of the `rust/` workspace. Decisions and phases are in
[ROADMAP.md](ROADMAP.md) (§2 decisions D1–D12, §4 phases).

## Layout

```text
rust/
  Cargo.toml  Cargo.lock  rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml
  Makefile  .gitignore  .markdownlint-cli2.jsonc
  LICENSE  NOTICE  CONTRIBUTING.md
  .cargo/config.toml  .cargo/audit.toml
  ARCHITECTURE.md  CHANGELOG.md  ROADMAP.md
  crates/          backend library crates (tak-*)
  services/        binaries (takserver, CLIs, fed-hub)
  web/             Yew/WASM apps and shared components
  scripts/         policy and CI-helper scripts
  docs/compliance/ NIST 800-53, STIG, FIPS, parity, POA&M documents
  parity/          harness that runs the Java reference server and TAKCL
```

## Crate status

| Crate | Path | Status | Phase |
|-------|------|--------|-------|
| `tak-api-types` | `crates/tak-api-types` | Stub (native + wasm32) | 0 (DTOs from 4) |
| `tak-web-components` | `web/components` | Stub (Yew, native + wasm32) | 0 |
| `tak-proto` | `crates/tak-proto` | Planned | 1 |
| `tak-cot` | `crates/tak-cot` | Planned | 1 |
| `tak-config` | `crates/tak-config` | Planned | 1 |
| `tak-db` | `crates/tak-db` | Planned | 2 |
| `tak-crypto` | `crates/tak-crypto` | Planned | 2 |
| `tak-auth` | `crates/tak-auth` | Planned | 2 |
| `tak-audit` | `crates/tak-audit` | Planned | 2 |
| `tak-schema` | `services/tak-schema` | Planned | 2 |
| `tak-usermanager` | `services/tak-usermanager` | Planned | 2 |
| `tak-net` | `crates/tak-net` | Planned | 3 |
| `tak-router` | `crates/tak-router` | Planned | 3 |
| `takserver` | `services/takserver` | Planned | 3 |
| `tak-api` | `crates/tak-api` | Planned | 4 |
| `admin` | `web/admin` | Planned | 4 |
| `tak-federation` | `crates/tak-federation` | Planned | 6 |
| `tak-plugin-host` | `crates/tak-plugin-host` | Planned | 8 |
| `fed-hub` | `services/fed-hub` | Planned | 8 |
| `fedhub-ui` | `web/fedhub-ui` | Planned | 8 |

## Dependency rules

- Web crates (`rust/web/*`) depend only on `tak-api-types` and other
  `rust/web/*` crates. They must build for `wasm32-unknown-unknown`.
- All cryptography and TLS goes through `tak-crypto`: rustls with a single
  `CryptoProvider` from aws-lc-rs with the `fips` feature. No other crate calls
  a crypto primitive crate. No wasm crate may reach `tak-crypto` or any
  `aws-lc-*` crate.
- The only native code is AWS-LC: `aws-lc-fips-sys` (the FIPS boundary) and,
  because rustls 0.23 `fips` pulls it in, the non-FIPS `aws-lc-sys`, which is
  linked but unused and outside the FIPS boundary (POAM-2026-10-26-08). Both
  are reachable only through `aws-lc-rs` (`deny.toml` wrappers).
- No crate depends on `services/*`.

## Build prerequisites

Phase 0 needs only the pinned Rust toolchain (`rust-toolchain.toml`, with the
`wasm32-unknown-unknown` target).

From Phase 2, crates that reach `aws-lc-fips-sys` also need CMake ≥ 3.5,
Go ≥ 1.17.13, Perl 5 and a C compiler (libclang only when bindings are
regenerated). `.cargo/config.toml` forces the bundled module and forbids Go
downloads. The FIPS module builds statically only on Linux x86_64/aarch64, so
FIPS claims cover **Linux x86_64/aarch64 only**; macOS builds are for
development and tests.

## Licence boundary

- `rust/` is Apache-2.0 (`license.workspace = true` in every member).
- `src/` is the upstream Java TAK Server under GPLv3 and is a read-only
  behavioural reference. Behaviour is derived from schemas, wire behaviour and
  black-box tests; Java is not translated line by line, and no file is copied
  from `src/` (see `CONTRIBUTING.md` §Provenance, POAM-2026-10-26-05).

## Conventions

- Every `.rs` and `.sh` file starts with `SPDX-License-Identifier: Apache-2.0`.
- `// COMPLIANCE MAPPING:` lists NIST SP 800-53 Rev 5 controls (Release 5.2.0,
  e.g. `AC-3`, `CM-2(2)`) and ASD STIG V-IDs for control implementations.
- `// PARITY: <java file>#<method>` cites the Java behaviour reference, for
  traceability only.
- `// POAM: <ID> <description>` marks an open gap tracked in
  `docs/compliance/POAM.md`; acceptance, if any, is recorded there.
- `// SAFETY:` is required on every `unsafe` block. `unsafe_code` is
  forbidden workspace-wide; an exception needs its own `[lints]` table and
  plan approval.
- Lints: `[workspace.lints]` in `Cargo.toml` (clippy `all` deny, `pedantic`
  warn, no `unwrap`/`expect`/`panic` outside tests). CI runs with
  `-D warnings`.

## See also

- [ROADMAP.md](ROADMAP.md) §2 decisions
- `CONTRIBUTING.md` (provenance rules and review checklist)
- `parity/README.md` (parity harness)
