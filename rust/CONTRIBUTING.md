# Contributing to TAK-RS

TAK-RS is the pure-Rust and Yew rewrite of the Java TAK Server, in this
`rust/` workspace. The Java server in `../src` is the behavioural reference.
Read [ARCHITECTURE.md](ARCHITECTURE.md) and [ROADMAP.md](ROADMAP.md) first.

Changes to `rust/` normally go through the `/tak-rs-workflow` skill
(plan → build → test → review → fix loop → docs → gate). The rules below apply
to every change, whether written by a person or an agent.

## Licence

- **TAK-RS is Apache-2.0.** The workspace sets `license = "Apache-2.0"` in
  `[workspace.package]`, and every member uses `license.workspace = true`
  (enforced by `scripts/check-manifests.sh`). The licence text is in
  [LICENSE](LICENSE) and attributions are in [NOTICE](NOTICE).
- **Inbound = outbound.** Under section 5 of the Apache License 2.0, any
  contribution you submit for inclusion is licensed under Apache-2.0, with no
  additional terms, unless you state otherwise in writing. Do not submit code
  you cannot license this way.
- **SPDX header.** Every `.rs` and `.sh` file starts with
  `SPDX-License-Identifier: Apache-2.0` in its first three lines
  (`// SPDX-License-Identifier: Apache-2.0` in Rust,
  `# SPDX-License-Identifier: Apache-2.0` in shell, after the shebang). TOML,
  YAML and Makefiles carry the same header by convention.
  `scripts/check-annotations.sh` enforces it for `.rs` and `.sh`.
- **`src/` is not part of this work.** The Java TAK Server in `../src` is
  licensed separately (GPLv3, see the repository root `LICENSE.md` and
  `INTENT.md`). Nothing in `rust/` is licensed under the GPL, and no GPL code
  may enter `rust/` (see [Provenance](#provenance)).
- **Dependencies.** Copyleft and weak-copyleft licences (GPL, LGPL, AGPL, MPL,
  EPL, CDDL, EUPL, OSL, CC-BY-SA) are denied because they are absent from the
  `deny.toml` allow-list. Adding one needs a plan entry and legal sign-off
  (NIST SP 800-53 CM-10).

## Provenance

`rust/` is Apache-2.0 and the Java reference in `src/` is GPLv3. TAK-RS must
reproduce Java's behaviour on the wire without becoming a translation of the
Java code. This rule is the control for that; no tool can prove clean
derivation, so reviewers check it.

**Allowed sources** for ported behaviour:

- schemas (`.proto`, XSD, OpenAPI, SQL DDL), consumed **in place** from `src/`
  by build scripts or tests;
- observed wire behaviour (captures from the parity harness);
- black-box tests (the TAKCL integration suites);
- public specifications;
- Java source **read to understand what it does**.

**Not allowed:**

- line-by-line translation of Java (or of JS, JSP or CSS from `src/`);
- copying Java comments, method decomposition or local structure;
- pasting Java into `rust/`;
- copying **any file** from `src/` into `rust/` while POAM-2026-10-26-05 (the
  derivative-work legal review) is open. This includes schemas and SQL
  migrations: read them from `src/` at build or test time instead.

**`// PARITY:` annotations** cite the Java location
(`// PARITY: src/<path>#<method>`) for traceability only. They mean "this
reproduces that behaviour", not "this was translated from that code".

**`Derived from` is required.** For every ported item, the impl notes and the
run's `parity-map.md` record what it was derived from: `schema`,
`wire capture`, `black-box test` or `Java read`. The project-wide record is the
`Derived from` column of
[docs/compliance/PROTOCOL_PARITY.md](docs/compliance/PROTOCOL_PARITY.md).

The open legal questions (derivative work, reuse of schema files, the effect
of `INTENT.md`, the NOTICE copyright holder, OpenSSL attribution) are tracked
in [docs/compliance/POAM.md](docs/compliance/POAM.md) as POAM-2026-10-26-05.

## Review checklist

Every change must pass these before merge. The commands and when each gate is
required are in
[quality-gates.md](../.claude/skills/tak-rs-workflow/references/quality-gates.md).

- [ ] **Provenance:** no file copied from `src/`. No Java comment text, and no
  Java method structure mirrored statement by statement. Each `PARITY:` item
  lists its derivation source in `parity-map.md` or the impl notes. A
  violation is a MAJOR finding, or a BLOCKER for copied files.
- [ ] **Format:** `make fmt-check`.
- [ ] **Lint:** `make clippy`, and `make clippy-wasm build-wasm` when
  `web/**` or a crate built for wasm32 changed. No new `#[allow(...)]`
  without a justification comment.
- [ ] **Tests:** `make test`; `make test-doc` when a public API changed. No
  new `#[ignore]` without a justification.
- [ ] **Docs:** `make doc`.
- [ ] **Supply chain:** `make supply-chain` when `Cargo.toml` or `Cargo.lock`
  changed.
- [ ] **Policy:** `make policy` and `make policy-selftest` (unsafe, manifests,
  annotations and SPDX headers, POA&M IDs, toolchain, workflows, `src/`
  coupling). `make check-workflows` needs `ruby` locally and fails without
  it.
- [ ] **No `unsafe`** unless the crate's plan approved it; every `unsafe`
  block has a `// SAFETY:` comment.
- [ ] **Compliance annotations:** the number of `COMPLIANCE MAPPING:` blocks
  does not drop; new control implementations are annotated.
- [ ] **Markdown:** `make markdownlint` when docs changed.
- [ ] **Parity:** for `port` runs and wire or API changes, the parity tests
  pass, or the difference is an approved deviation in `parity-map.md`.
- [ ] **Compliance docs** in `docs/compliance/` updated for any control,
  STIG, FIPS, parity or POA&M change.

A gate that cannot run (for example, no container engine for the parity
harness) is recorded as **NOT RUN** with the reason, never as passed.

## Supply chain

- **No new native code.** TAK-RS is pure Rust. The only native code is AWS-LC
  through `aws-lc-rs`: `aws-lc-fips-sys` (the FIPS 140-3 module) and
  `aws-lc-sys` (pulled in by the rustls `fips` feature, never called directly;
  POAM-2026-10-26-08), plus their build-time toolchain crates. Any other crate
  with a Cargo `links` key fails `make check-native-deps` and needs a plan
  entry and user approval. No crypto crate may reach `wasm32-unknown-unknown`.
- **One crypto provider.** All crypto and TLS go through the project's rustls
  `CryptoProvider` backed by aws-lc-rs FIPS (from Phase 2, `tak-crypto`).
  `openssl`, `native-tls`, `ring` and `boring` are banned.
- **Reading from `src/` or `utils/`.** Code under `rust/` that reads a file
  from `../src` or `../utils` (for example a `tak-proto` `build.rs` reading
  `.proto` files) needs a matching `rust_inputs` entry in
  `scripts/ci-changes.sh`, so that upstream changes to that input run the
  Rust jobs. `make check-src-coupling` fails otherwise. It is a text
  heuristic, so review the entry when adding the read.
- **crates.io only.** No git or other registry sources. Commit `Cargo.lock`,
  and build with `--locked`.
- **Advisory ignores** need a `# POAM:` ID and must be identical in
  `deny.toml` and `.cargo/audit.toml` (`make check-advisory-sync`).
- **Run `make ci` before opening a PR.** It mirrors the CI jobs locally.
  `make markdownlint` is separate because it needs npm registry access.

## Compliance annotations

The control baseline is **NIST SP 800-53 Rev 5** (Release 5.2.0 catalog,
SP 800-53B baselines; Moderate assumed). Cite controls in Rev 5 form,
including enhancements: `AC-3`, `CM-2(2)`, `SR-3`. Do not use Rev 4 IDs or
spaced forms such as `CM-2 (2)`; `scripts/check-annotations.sh` rejects them.

Example (the STIG V-ID and Java method are illustrative):

```rust
// COMPLIANCE MAPPING:
// - NIST 800-53: AC-3 (Access Enforcement)
// - ASD STIG: V-222425
// PARITY: src/takserver-core/.../SubmissionService.java#processCot
```

- `// COMPLIANCE MAPPING:` marks a control implementation. Each block has at
  least one `NIST 800-53:` or `ASD STIG:` line. In TOML, YAML, shell and
  Makefiles, use `#` comments with the same layout.
- `// PARITY: src/<path>#<method>` marks ported behaviour (see
  [Provenance](#provenance)).
- `// POAM: POAM-<date>-<nn> <description>` marks an open gap tracked in
  [docs/compliance/POAM.md](docs/compliance/POAM.md). It does not mean the
  gap is accepted; acceptance, if any, is recorded in POAM.md. Every
  referenced ID must exist there (`scripts/check-poam-ids.sh`).
- Every control ID on a `NIST 800-53:` line must be mapped in
  [docs/compliance/NIST_800-53_MAPPING.md](docs/compliance/NIST_800-53_MAPPING.md)
  (`scripts/check-annotations.sh` rule (d)).

Status claims in `docs/compliance/` use the vocabulary in
[NIST_800-53_MAPPING.md §Vocabulary](docs/compliance/NIST_800-53_MAPPING.md#vocabulary):
Implemented, Partially Implemented, Planned, Alternative Implementation, Not
Applicable. "Implemented" needs code or configuration **and** passing,
recorded evidence (a PASS in
[docs/compliance/TEST_EVIDENCE.md](docs/compliance/TEST_EVIDENCE.md) or a
recorded CI run). A gate that did not run is NOT RUN and is never evidence.
"Risk Accepted" is used only after the authorizing official accepts a POA&M
entry.

## CI

- Workflows live in the repository root `.github/workflows/`:
  `rust-ci.yml` (every PR and push to `main`), `rust-supply-chain.yml`
  (nightly and manual) and `rust-parity.yml` (nightly, manual, and changes to
  `rust/parity/**`).
- **`rust-ci-gate` is the required check**, and the only one. It always runs
  and passes only if every job that should have run succeeded. A PR that
  touches only `src/**` (an upstream merge) runs the change detector,
  workflow lint and the gate, and passes quickly.
- `rust-parity.yml` is never a required check, and neither is the
  `annotations-strict` job in `rust-ci.yml`, which only makes a pending
  compliance-mapping ID visible on every PR.
- Workflow rules: every action is pinned to a full commit SHA with a version
  comment; `permissions: contents: read`; no `pull_request_target`; no
  `${{ github.event.* }}`, `${{ inputs.* }}` or `${{ needs.* }}` inside
  `run:` (pass values through `env:`). `make check-workflows` enforces these
  on the parsed YAML of every workflow file.
- Configuring `rust-ci-gate` as a required check in the branch ruleset is
  tracked as POAM-2026-10-26-04 until the repository administrator does it.
