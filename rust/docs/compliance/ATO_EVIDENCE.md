# Authority to Operate (ATO) Evidence Guide

**Document Version:** 0.4
**Date:** 2026-10-27
**TAK-RS Version:** 0.0.1 (Phase 0, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 0 build run. Local evidence recorded (QA Rounds 1 and 2), the CM-2 pin inventory filled in, and the first CI evidence recorded (PR 7, 2026-10-27). No nightly run yet.
**Run:** `20261026-210255-phase0-foundations`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Framework | NIST Risk Management Framework (RMF) |
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed; [ROADMAP.md](../../ROADMAP.md) §6 Q5) |
| Overlay | CNSSI 1253 (TBD) |
| STIG | DISA ASD STIG, release TBD |
| Catalog file hashes | See [Pin inventory](#pin-inventory-cm-2) (`usnistgov/oscal-content` tag `v1.6.0`) |

### Scope

Phase 0: development environment only. Evidence describes the developer
process, CI and supply chain. There is no TAK-RS runtime (TOE) yet.

### Vocabulary

Control status and origination use the vocabulary in
[NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) §Vocabulary. Gate results use
PASS, FAIL and NOT RUN (see [TEST_EVIDENCE.md](TEST_EVIDENCE.md)).

## 1. Purpose

This guide says what evidence TAK-RS produces for the SSP, SAR, STIG checklist
and POA&M, where it lives and how long it is kept. It does not claim that any
control is met; status is in [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md).

## 2. Evidence types and locations

| Evidence | Produced by | Location | Retention |
|----------|-------------|----------|-----------|
| CI run record (run URL, commit SHA, job conclusions, `rust-ci-gate` verdict table from `ci-gate.sh`) | `rust-ci.yml` (WU5) | GitHub Actions run history; first record: [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) (PR 7), summarised in [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3 | GitHub default log retention; long-term store TBD |
| SBOM (CycloneDX JSON, `rust-sbom`) | `sbom` job (non-blocking, POAM-2026-10-26-01) | Actions artifact; first: `rust-sbom` from run 40539495255 (13544 bytes, 2 files) | 90 days |
| Parity results (JUnit XML, TAKCL and server logs, `build-info.json`; keystores and keys excluded) | `rust-parity.yml` (WU8) | Actions artifact `parity-results-<run_id>-<attempt>` (regular files only; symlinks and special files rejected); first: `parity-results-40539495328-1` (76550 bytes) | 90 days (the first expires 2027-01-25) |
| Nightly supply-chain results | `rust-supply-chain.yml` (WU5) | GitHub Actions run history (no scheduled run yet) | GitHub default log retention |
| Workflow run record (`plan.md`, `impl-notes.md`, `qa-report.md`, `findings-*.md`, `gate.md`, `wu0-applied.md`) | `/tak-rs-workflow` | `.agent-runs/<run>/` (not committed; ignored by the root `.gitignore`) | Until copied to the evidence store |
| Compliance documents | compliance-scribe | `rust/docs/compliance/` (this directory) | git history |

The long-term evidence store is **TBD**. AU-11 retention of developer evidence
is Planned ([NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) §AU). Until the
store exists, a run record kept as ATO evidence must be copied out of
`.agent-runs/` by hand.

## 3. Multi-agent development workflow (CM-3, CM-4, SA-11)

Changes to `rust/` go through the `/tak-rs-workflow` skill
([.claude/skills/tak-rs-workflow/](../../../.claude/skills/tak-rs-workflow/)).
The orchestrator delegates to specialist agents: tak-architect, rust-coder,
yew-coder, qa-engineer, security-reviewer, nist-800-53-reviewer,
crypto-tls-reviewer, tak-protocol-reviewer, yew-frontend-reviewer,
compliance-scribe and release-engineer. Reviewers are read-only and work
independently; must-fix findings go back to the coder and are re-checked by
the reviewer that raised them.

| Control | Evidence | Status |
|---------|----------|--------|
| CM-3 | Per-change record in `.agent-runs/<run>/`: `plan.md`, `findings-*.md`, fix rounds, `gate.md` (PASS, PASS WITH POA&M or FAIL) | Planned. The Phase 0 run has a plan, a QA report (Rounds 0, 1 and 2), reports from three reviewers plus their re-verification reports, and three fix rounds (fix round 3 not QA-verified); it has not reached its gate |
| CM-4 | `plan.md` impact sections (interfaces, threat sketch, compliance pre-map) | Planned |
| SA-11 | `qa-report.md` lists every gate command with its result; gates not run are NOT RUN ([quality-gates.md](../../../.claude/skills/tak-rs-workflow/references/quality-gates.md)) | Two local records (QA Rounds 1 and 2) and one CI record (PR 7, 2026-10-27) in [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3. SA-11 is Partially Implemented in [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) |

The workflow is a developer-side aid. Agent reviews are SA-11 evidence, not an
independent CA-2 assessment, and they do not replace human change approval
(POAM-2026-10-26-04).

## 4. Supply-chain evidence (RA-5, SI-2, SR-3, SR-4, SR-11, SA-22, CM-10)

| Evidence | Command or source | Frequency | Status |
|----------|-------------------|-----------|--------|
| Advisory scan | `make audit` (`cargo audit --deny warnings`) | Every Rust-relevant PR and nightly | Local QA Round 1 PASS (93 crates, 1378 advisories), but only with the two ignores of the open POAM-2026-10-26-09. PR CI: PASS in run 40539495255 (93 crates, 1380 advisories), also only with the same two ignores. Nightly: NOT RUN |
| Policy scan (advisories, bans, licences, sources) | `make deny` (`cargo deny --locked check`) | Every Rust-relevant PR and nightly | Local QA Round 1 PASS (all four sections ok; advisories ok only with the same two ignores). PR CI: PASS in run 40539495255 (all four sections ok; advisories ok only with the two ignores). Nightly: NOT RUN |
| Native-code gate | `make check-native-deps` (per shipping target) | Every Rust-relevant PR and nightly | Local QA Round 1 PASS (3 targets; wasm32 markers only). PR CI: PASS in run 40539495255 (x86_64 and aarch64 Linux, wasm32; markers only; the AWS-LC crates are allow-listed but not in the graph). Nightly: NOT RUN |
| Advisory-ignore sync | `make check-advisory-sync` | Every Rust-relevant PR and nightly | Local QA Round 1 PASS (2 IDs, identical, each with a POA&M). PR CI: PASS in run 40539495255 (same result) |
| Supply-chain negative tests | Plan §8.3 scratch checks, recorded in `qa-report.md` | Build run | Round 1: `allow-build-scripts` negative rejects (exit 2, `build-script-not-allowed`). The other §8.3 negatives were run by the release-engineer in WU3 scratch copies (impl notes, not re-run by QA); the FIPS-graph row of that table is not evidence (QA-004, [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues). Round 2: POAM-09 detection (SA-22): with the two ignores removed in scratch copies, `cargo deny … check advisories` and `cargo audit --deny warnings` each exit 1 on exactly RUSTSEC-2024-0370 and RUSTSEC-2025-0141; controls with the unmodified files exit 0 ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3, QA Round 2) |
| SBOM | `make sbom` | Every Rust-relevant PR (non-blocking) | Locally NOT RUN (cargo-cyclonedx not installed). PR CI: job succeeded in run 40539495255 and uploaded `rust-sbom` (2 `.cdx.json` files). Still best-effort, not gating, not signed (POAM-2026-10-26-01) |
| Dependabot repository settings | `gh api repos/192d-Wing/Server/vulnerability-alerts`; `gh api repos/192d-Wing/Server/automated-security-fixes` | At each phase gate | Read 2026-10-26: alerts on (HTTP 204); security updates `{"enabled":true,"paused":true}`, so **paused** (POAM-2026-10-26-10) |

### Pin inventory (CM-2)

Recorded on 2026-10-26 (WU10, pre-gate) from the release-engineer's impl
notes. Every value below was checked against the file that holds it with
`grep` on that date. Action SHAs were re-resolved with
`gh api repos/<owner>/<repo>/git/ref/tags/<tag>` (annotated tags dereferenced
through `git/tags/<object>`), and the actionlint and Gradle checksums were
re-fetched from their publishers. All matched. QA Round 2 re-checked 8 of
these pins independently on 2026-10-26 (toolchain, action SHAs, actionlint,
`pins.env` digests and Gradle checksums, OSCAL hashes, tool versions, licence
files); every one matched. Fix rounds 2 and 3 edit the workflows, so this
inventory is re-checked at the gate.

**Toolchain and tools**

| Item | Pin | Where pinned | Evidence |
|------|-----|--------------|----------|
| Rust toolchain | 1.99.1 (MSRV `rust-version = "1.99"`) | [rust-toolchain.toml](../../rust-toolchain.toml) line 12; [Cargo.toml](../../Cargo.toml) line 22 | `rustup show active-toolchain` and `check-toolchain` PASS (QA Round 1) |
| cargo-deny | 0.20.2 | `taiki-e/install-action` `tool:` in `rust-ci.yml` and `rust-supply-chain.yml` | Local 0.20.2 used for QA Round 1 |
| cargo-audit | 0.22.2 | same | Local 0.22.2 used for QA Round 1 |
| zizmor | 1.31.1 | `rust-ci.yml` `workflow-lint` | Checksum from the install-action manifest at the pinned SHA; run 40539495255 installed it with "verifying sha256 checksum" and logged `zizmor 1.31.1` |
| shellcheck | 0.11.0 | `rust-ci.yml` `policy` | same; run 40539495255 verified the checksum and logged `version: 0.11.0` |
| cargo-cyclonedx | 0.5.9 | `rust-ci.yml` `sbom` | same; run 40539495255 verified the checksum and logged `cargo-cyclonedx-cyclonedx 0.5.9` |
| actionlint | 1.7.12, `actionlint_1.7.12_linux_amd64.tar.gz` SHA-256 `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8` | `rust-ci.yml` `workflow-lint` (`ACTIONLINT_VERSION`, `ACTIONLINT_SHA256`, `sha256sum --check --strict`) | The release `actionlint_1.7.12_checksums.txt` gives the same value (re-fetched 2026-10-26). install-action has no actionlint manifest, so Dependabot does not update this pin; review it at each install-action bump (WU5 REL-009). Run 40539495255: `actionlint_1.7.12_linux_amd64.tar.gz: OK` |
| markdownlint-cli2 | 0.24.0 | [Makefile](../../Makefile) `MARKDOWNLINT_CLI2_VERSION`; bundled by `DavidAnson/markdownlint-cli2-action` v25.0.0 in CI | npm integrity `sha512-c5hU2MDoTlS230MLzdNlkIyxImXq5acxGKz4nYz4aDhH/2BgFtpsA7iK9/OoTHMgZ4FuR0wedTQA31K6Lp9I/Q==` (release-engineer, `npm view`); `make markdownlint` PASS (QA Round 1) |

**GitHub Actions** (each pinned to the full commit SHA with a version comment)

| Action | Tag | Tag type | Commit SHA | Used in |
|--------|-----|----------|------------|---------|
| `actions/checkout` | v7.0.1 | lightweight | `3d3c42e5aac5ba805825da76410c181273ba90b1` | `rust-ci.yml`, `rust-supply-chain.yml`, `rust-parity.yml` |
| `Swatinem/rust-cache` | v2.9.3 | annotated (tag object `2201cc584a476d1b508be977bbe4b7d1c7a52e9c`) | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` | `rust-ci.yml` |
| `taiki-e/install-action` | v2.87.31 | lightweight | `1e29709c0ec63babbe8f266397a465f29a825f91` | `rust-ci.yml`, `rust-supply-chain.yml` |
| `actions/upload-artifact` | v7.0.2 | lightweight | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` | `rust-ci.yml` (`sbom`), `rust-parity.yml` |
| `actions/cache/restore`, `actions/cache/save` | v6.1.0 | lightweight | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` | `rust-parity.yml` |
| `DavidAnson/markdownlint-cli2-action` | v25.0.0 | annotated (tag object `913db5cf77ef624b914090bad9e1615f7f6e099d`) | `696eb3f2509270d7b882d2aa68347bc676e7af34` | `rust-ci.yml` |
| `gradle/actions/wrapper-validation` | v6.4.0 | annotated (tag object `b9bee63ef481cc545e4517ec730544a4d8b1dfa0`) | `3f5f9adaf7d9fecd50b5935e54106014257a94e6` | `rust-parity.yml` |

The repository Actions policy does not yet restrict actions to this list or
require SHA pins (POAM-2026-10-26-04).

**Container images** ([parity/pins.env](../../parity/pins.env); test
fixtures for the parity harness, never shipped)

| Key | Image and digest | Note |
|-----|------------------|------|
| `JDK_BUILD_IMAGE` | `docker.io/library/eclipse-temurin:17-jdk-jammy@sha256:1dcfdf8a7f4f9d139931e2c98a0f95211462ddb11809f775590ac0003600c53a` | OCI index (multi-arch); resolved 2026-10-26 from `registry-1.docker.io` |
| `JRE_RUN_IMAGE` | `docker.io/library/eclipse-temurin:17-jre-jammy@sha256:131f924767c0cde364fd7d3739dd3be79afd4e92d434bedc76b60ebd2ccb48f0` | OCI index; `17-jre-jammy`, not the plan's `17-jammy` (which is a JDK) |
| `POSTGIS_IMAGE` | `docker.io/postgis/postgis:15-3.3@sha256:a2fc46b52819d25add4ed93d3da83e106e17ea3ace6883cce3791631c9820bb2` | Single linux/amd64 manifest; tag last pushed 2023-08-14 (POAM-2026-10-26-03) |

The digests were not re-resolved: the tags move, and the digest is the pin.

**Gradle (Java reference build in the parity harness)**

| Item | Value | Evidence |
|------|-------|----------|
| `GRADLE_VERSION` | 8.10.2 | Matches `src/gradle/wrapper/gradle-wrapper.properties` line 3 (`distributionUrl`) |
| `GRADLE_DIST_SHA256` | `31c55713e40233a8303827ceb42ca48a47267a0ad4bab9177123121e71524c26` | Same value from `https://services.gradle.org/distributions/gradle-8.10.2-bin.zip.sha256` (re-fetched 2026-10-26) |
| `GRADLE_WRAPPER_JAR_SHA256` | `91a239400bb638f36a1795d8fdf7939d532cdc7d794d1119b7261aac158b1e60` | All 6 `gradle-wrapper.jar` files under `src/` hash to this value; it equals Gradle's published `gradle-7.5.1-wrapper.jar.sha256` (re-fetched 2026-10-26). Upstream ships the 7.5.1 wrapper with an 8.10.2 distribution URL; any wrapper launches any distribution, and the distribution is pinned separately |

**Reference documents**

| Item | Value |
|------|-------|
| NIST OSCAL catalog | `NIST_SP-800-53_rev5_catalog.json` from `usnistgov/oscal-content` release tag `v1.6.0` (commit `f0de6d8d8c059d374f15e0ae07508d82e370fc1d`): metadata version 5.2.0, OSCAL 1.2.3, SHA-256 `64ca6b4161de51ab6af098a7d101d326f14c95e1abf721396cd8a049e8115eeb` |
| NIST OSCAL MODERATE profile | `NIST_SP-800-53_rev5_MODERATE-baseline_profile.json`, same tag: metadata version 5.2.0, SHA-256 `e3a0f11237cfaac24dfcf1586e07683f17423757fd84208deb33196b12e8f590` |
| Earlier 5.2.0 publication (for comparison) | Tag `v1.4.0` (2025-08-27; OSCAL 1.1.3): catalog SHA-256 `1645df6a370dcb931db2e2d5d70c2f77bc89c38499a416c23a70eb2c0e595bcc`, MODERATE `b5dd6ef0d1c359e5c9942a14bbe996894909897fec65319645f5e4fbc1582ab0`. Both releases carry control-catalog version 5.2.0; the files differ in OSCAL format version |

The titles and baseline membership in the compliance documents were checked
against the `v1.6.0` files. Those files are byte-identical to the `main`
files used for the WU6 check (same SHA-256), so the result holds for the
tag. Fetched by the compliance-scribe on 2026-10-26.

**Licence files**

| File | SHA-256 |
|------|---------|
| [LICENSE](../../LICENSE) | `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30` (11358 bytes; the canonical Apache License 2.0 text) |
| [NOTICE](../../NOTICE) | `cce95ce440ad3dda5b50fa2c4f4631e24e3bebfebdf4e958a61775d6d97869f3` |

`parity/pins.env` and the workflow files are not hashed here, because fix
round 2 is still changing them; the gate record carries their hashes.

## 5. Parity evidence (SA-11)

| Item | Value |
|------|-------|
| Java reference | `5.7-RELEASE-14`, commit recorded per run in `build-info.json` |
| Normalisation rules | None yet (nothing compared in Phase 0) |
| First result | PASS, 2026-10-27, [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328) (PR 7, head `ce9caf79`): TAKCL `StartupTests` against Java, `verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0`; artifact `parity-results-40539495328-1`. This is a Java baseline only; nothing was compared with TAK-RS (POAM-2026-10-26-06) |
| Local harness evidence | `parity/harness.sh selftest` 39 passed, 0 failed; integrity negative exits 4; exit-code contract PASS (QA Round 1, [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3). QA Round 2: harness selftest 44 passed, 0 failed; `reject_special` fault cases reject (SEC-009). CI (run 40539495328): selftest 44 passed, 0 failed; `wrapper verified: Gradle 8.10.2` |
| Dockerfile lint | None. hadolint is in no workflow and has never run; the two Dockerfiles in [parity/docker/](../../parity/docker/) are not linted (gap; see [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3, PR 7 CI record) |
| Container policy deviation | The parity fixtures use Docker Hub `eclipse-temurin` and `postgis/postgis` bases, not UBI or Iron Bank, and the `takcl` container is not read-only (WU7 REL-004). They are test fixtures outside the TOE and never shipped. Recorded as a deviation; **not accepted** by anyone yet. The whole deviation (all three images and the `takcl` root filesystem) is tracked as open POA&M POAM-2026-10-26-03 (extended by NIST-021) |

Feature and suite status: [PROTOCOL_PARITY.md](PROTOCOL_PARITY.md).

## 6. Licence and provenance evidence (CM-10, SR-4)

| Evidence | Command or source | Status |
|----------|-------------------|--------|
| `rust/LICENSE` is the canonical Apache License 2.0 text | SHA-256 in [Pin inventory](#pin-inventory-cm-2); `head -3 LICENSE \| grep -q 'Apache License'` | QA Round 1 PASS |
| `rust/NOTICE` includes the OpenSSL attribution for AWS-LC | `grep -q 'OpenSSL Project' NOTICE` | QA Round 1 PASS |
| Every workspace member is Apache-2.0 | `cargo metadata --no-deps --format-version 1` licence list; `check-manifests.sh` | QA Round 1 PASS (licence set exactly `Apache-2.0`; 2 members) |
| SPDX headers on every `.rs` and `.sh` file | `check-annotations.sh` rule (c), in `make policy` | QA Round 1 PASS |
| No copyleft in the dependency graph | `cargo deny check licenses`; `cargo metadata --locked` licence grep for GPL, AGPL, MPL | QA Round 1 PASS (`licenses ok`; grep finds nothing) |
| Provenance rule | [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance | Written; not yet exercised by a port |
| Reviewer provenance checklist answers | `findings-*.md` per `port` run | None yet (no port in Phase 0) |
| Legal review | POAM-2026-10-26-05 | Open |

## 7. OSCAL

Decision: no OSCAL artifacts in Phase 0. The OSCAL 1.1.x `component-definition`
for the `takserver` component is **Planned** for the Phase 2 plan run,
generated from [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) and validated
in CI. Reasons are in [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md)
§OSCAL. Not a POA&M item.

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-27 | 0.4 | PR 7 CI evidence: §2 first CI run record, `rust-sbom` and `parity-results-40539495328-1` artifacts; §3 SA-11 row cites the CI record; §4 PR-CI results for `make audit`, `make deny`, `check-native-deps`, `check-advisory-sync` and the SBOM job (nightly still NOT RUN; the advisory passes still depend on the open POAM-2026-10-26-09); pin inventory: zizmor, shellcheck, cargo-cyclonedx and actionlint checksums verified in CI; §5 first parity result PASS (Java baseline only) and a Dockerfile-lint gap row (hadolint in no workflow) | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.3 | Fix round 3 (docs): QA Round 2 added as evidence (§3 workflow row, §4 POAM-09 detection negatives for SA-22, §5 harness selftest 44/0); QA Round 2 pin re-check noted in the pin inventory; container-policy deviation now wholly under POAM-2026-10-26-03, still not accepted (NIST-021) | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | WU10 (pre-gate): CM-2 pin inventory (toolchain, tools, 7 actions with tag and commit SHA, 3 image digests, Gradle checksums, OSCAL catalog and MODERATE profile SHA-256 from release tag `v1.6.0`, licence file hashes), each checked against the repository; supply-chain and licence tables filled from QA Round 1; Dependabot settings row (POAM-2026-10-26-10); parity container-policy deviation recorded, not accepted. CI-only items stay NOT RUN | `20261026-210255-phase0-foundations` (WU10) |
| 2026-10-26 | 0.1 | Initial skeleton: evidence types, locations and retention; workflow, supply-chain, parity and licence/provenance evidence tables (all pending); pin inventory template; OSCAL decision | `20261026-210255-phase0-foundations` (WU6) |
