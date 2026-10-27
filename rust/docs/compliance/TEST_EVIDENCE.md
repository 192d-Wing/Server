# TAK-RS Test Evidence

**Document Version:** 0.4
**Date:** 2026-10-27
**TAK-RS Version:** 0.0.1 (Phase 0, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 0 build run, before the gate. Two local run records (QA Round 1 and QA Round 2, both overall PASS); no CI run recorded.
**Run:** `20261026-210255-phase0-foundations`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** (SA-11, SA-11(1), CM-3(2)) |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed) |
| Overlay | CNSSI 1253 (TBD) |
| STIG | DISA ASD STIG, release TBD |

### Scope

Phase 0: development environment only. Tests cover the stub crates, the
policy and CI-helper scripts, the supply-chain policy and the parity harness.
There is no TAK-RS runtime (TOE) yet.

### Result vocabulary

| Result | Meaning |
|--------|---------|
| PASS | The command ran and exited 0 with the expected output |
| FAIL | The command ran and did not meet the expectation |
| NOT RUN | The command did not run; the reason is given. A missing environment (Docker, Postgres, Java 17, a tool that is not installed, GitHub) is NOT RUN, **never PASS** |

"0 tests" is a FAIL, not a PASS. A test count is recorded with every PASS.

## 1. Test environment

Filled per run record.

| Component | Local (macOS) | CI (`ubuntu-24.04`) |
|-----------|---------------|---------------------|
| Rust toolchain | 1.99.1 (from [rust-toolchain.toml](../../rust-toolchain.toml)) | 1.99.1 |
| cargo-deny | 0.20.2 | 0.20.2 (pinned in the workflows; not yet run) |
| cargo-audit | 0.22.2 | 0.22.2 (pinned in the workflows; not yet run) |
| Other tools | jq 1.7.1, `/bin/bash` 3.2.57, BSD awk 20200816, BSD grep, Ruby 2.6.10 with stdlib psych (YAML parsing; `check-workflows.rb` from fix round 2), npx with markdownlint-cli2 0.24.0 | shellcheck 0.11.0, zizmor 1.31.1, actionlint 1.7.12, cargo-cyclonedx 0.5.9 (pinned; not yet run) |
| Not installed locally | docker, podman, shellcheck, actionlint, zizmor, hadolint, cargo-cyclonedx, wasm-pack, trunk, gawk, mawk | — |
| Container engine | None installed | Docker (runner image) |
| Java | Temurin 27 (cannot run Gradle 8.10.2) | JDK 17 in the build container |
| Host | macOS (Darwin 27.0.0, aarch64) | `ubuntu-24.04` (not yet run) |

## 2. Run-record template

Each run adds a section `### <date> <run id>` with this table, followed by the
NOT RUN items and their reasons.

| Gate | Command | Result | Counts |
|------|---------|--------|--------|
| Format | `make fmt-check` | — | — |
| Lint (native) | `make clippy` | — | — |
| Lint (wasm32) | `make clippy-wasm build-wasm` | — | — |
| Unit tests | `make test` | — | passed / failed / ignored |
| Doc tests | `make test-doc` | — | passed / failed / ignored |
| Docs | `make doc` | — | — |
| Supply chain | `make deny`, `make audit` | — | advisories, bans, licences, sources |
| Native-code gate | `make check-native-deps` | — | crates found per target |
| Policy | `make policy` | — | — |
| Policy self-tests | `make policy-selftest` | — | cases passed / total |
| Markdown | `make markdownlint` | — | files, errors |
| Shell | `shellcheck scripts/*.sh parity/harness.sh` | — | — |
| Harness self-test | `parity/harness.sh selftest` | — | — |
| Harness exit-code contract | `parity/harness.sh test StartupTests --target rust` (expect 2); `parity/harness.sh preflight` without an engine (expect 3) | — | — |
| Licence | `cargo metadata --no-deps --format-version 1` licence set (expect exactly `Apache-2.0`) | — | — |
| NIST family coverage | §9.1 20-family loop over `NIST_800-53_MAPPING.md` (expect no output) | — | — |
| Upstream tree untouched | `git -C .. status --porcelain src/` (expect no output) | — | — |

## 3. Run records

### 2026-10-26 20261026-210255-phase0-foundations (QA Round 1, local)

- **Who and where:** qa-engineer, macOS (environment in §1), from
  `/Users/jwillman/Dev/Server/rust` unless noted.
- **What was tested:** the uncommitted Phase 0 working tree after fix round 1
  (base `HEAD` = `5187abd4`). Fix round 2 changed scripts, the parity
  harness and workflows after this record (the release-engineer reports
  `make policy-selftest` 269/0, harness selftest 44/0, and that 9 of the 14
  upstream merges now give `rust=true` because they touch
  `src/takserver-protobuf/`, SEC-008). Those are not QA results; QA
  re-verifies after fix round 2 and the results are added as a separate
  record.
- **Source:** the run's `qa-report.md` (Round 1). Logs were kept in the
  session scratch directory, not in the repository.
- **Overall: PASS.** Every gate that ran passed, except the expected
  `make check-annotations-strict` failure (CA-5 unmapped, fixed in this
  revision; see below). Tool-dependent gates are NOT RUN.

**Round 0 (before fix round 1): FAIL.** Every local gate passed, but QA-001
(MAJOR) failed the run: `ci-changes.sh` took 16 s on 170 paths and more than
300 s on 668 paths. QA-002 (MINOR): `check-workflows` did not catch
`continue-on-error`, job-level permission escalation, or `github.head_ref` and
`steps.*` in `run:`. Both were fixed in fix round 1 and are resolved in
Round 1, with regression tests.

| Gate | Command | Result | Counts |
|------|---------|--------|--------|
| Toolchain | `rustup show active-toolchain` | PASS | `1.99.1-aarch64-apple-darwin`, overridden by `rust-toolchain.toml` |
| Lockfile | `cargo metadata --locked --format-version 1 >/dev/null` | PASS | exit 0 |
| Format | `make fmt-check` | PASS | exit 0 |
| Lint (native) | `make clippy` | PASS | 0 warnings with `-D warnings`, including the `clippy.toml` bans |
| Lint (wasm32) | `make clippy-wasm` | PASS | 0 warnings |
| Build (wasm32) | `make build-wasm` | PASS | exit 0 |
| Unit and integration tests | `make test` | PASS | 5 passed, 0 failed (`tak-api-types`: lib 0, `tests/clippy_config.rs` 2; `tak-web-components`: lib 1, `tests/clippy_config.rs` 2) |
| Doc tests | `make test-doc` | PASS | 2 passed, 0 failed |
| Docs | `make doc` | PASS | exit 0 |
| Supply chain: policy | `make deny` | PASS | `advisories ok, bans ok, licenses ok, sources ok`; warnings only (12 unused wrapper, 6 licence not encountered, 1 licence exception not encountered, 3 duplicate) |
| Supply chain: negative | `cargo deny --locked --config <scratch>/deny-nozmij.toml check bans` (`deny.toml` without the `"zmij",` line) | PASS (rejects) | exit 2, `error[build-script-not-allowed]: crate 'zmij = 1.0.23'`, `bans FAILED` |
| Supply chain: advisories | `make audit` | PASS | 93 crates, 1378 advisories, no findings; 2 IDs ignored under the open POAM-2026-10-26-09 (the pass depends on them) |
| Native-code gate | `make check-native-deps` | PASS | 3 targets; wasm32: `prettyplease` 0.2.37 and `wasm-bindgen-shared` 0.2.129 (both `marker`); no unlisted `web/` member; no crypto crate names |
| Advisory-ignore sync | `make check-advisory-sync` | PASS | 2 IDs, lists identical, each with a POA&M ID |
| Policy | `make policy` | PASS | unsafe 0; manifests 2; annotations ok with 1 PENDING (CA-5); POA&M IDs 9 referenced, 9 entries; toolchain 1.99.1 vs 1.99; workflows ok (3 files) |
| Policy self-tests | `make policy-selftest` | PASS | 173 passed, 0 failed before the QA additions; 199 passed, 0 failed after (26 QA cases added) |
| Annotations (strict) | `make check-annotations-strict` | FAIL (expected) | make exit 2 (the script exits 1): `FAIL (d) CA-5` cited by `check-advisory-sync.sh` and `check-poam-ids.sh`; 17 blocks. The compliance-scribe's item (NIST-004); the CA-5 block is added in this revision, see the note below |
| Markdown | `make markdownlint` | PASS | markdownlint-cli2 0.24.0: 11 files, 0 issues |
| Shell syntax | `bash -n parity/harness.sh; for f in scripts/*.sh; do bash -n $f; done` | PASS | 12 files parse |
| Harness self-test | `bash parity/harness.sh selftest` | PASS | 39 passed, 0 failed |
| Harness help | `parity/harness.sh --help` | PASS | exit 0 |
| Harness exit-code contract | `parity/harness.sh test StartupTests --target rust` (expect 2); `parity/harness.sh preflight` and `--engine docker preflight` without an engine (expect 3); `parity/harness.sh frobnicate` (expect 2) | PASS | 2; 3 and 3; 2 |
| Harness `--version-tag` validation | `parity/harness.sh --version-tag bogus preflight`; `--version-tag '5.7-RELEASE-14;x' preflight` | PASS | exit 2 each |
| Harness integrity negative (plan §8.4) | `TAK_PARITY_PINS=<scratch>/pins-wrongjar.env parity/harness.sh --version-tag auto build-java`, then `parity/harness.sh clean` | PASS | exit 4: wrapper-jar SHA-256 `91a2…1e60` does not match the pin `0…0`; stops before any engine step. `clean` exit 3, `.work/` removed; the repository's tags unchanged |
| Licence: members | `cargo metadata --no-deps --format-version 1 \| jq -r '.packages[].license' \| sort -u` | PASS | `Apache-2.0` |
| Licence files | `head -3 LICENSE \| grep -q 'Apache License' && grep -q 'OpenSSL Project' NOTICE && grep -q '^## Provenance' CONTRIBUTING.md` | PASS | exit 0 |
| No copyleft in the graph | `cargo metadata --format-version 1 --locked \| jq -r '.packages[].license // ""' \| grep -E 'GPL\|AGPL\|MPL'` | PASS | grep exit 1 (no match) |
| NIST family coverage | §9.1 20-family loop over `NIST_800-53_MAPPING.md` | PASS | no output (20 of 20) |
| Catalog pin | `grep -q 'Release 5.2.0' … && grep -q '800-53B' …` | PASS | exit 0 |
| CI shape: permissions | `grep -L 'contents: read' ../.github/workflows/rust-*.yml` | PASS | no output |
| CI shape: no `pull_request_target` | `grep -n pull_request_target ../.github/workflows/rust-*.yml` | PASS | exit 1 (no match) |
| CI shape: single gate | `grep -l rust-ci-gate ../.github/workflows/*.yml` | PASS | only `rust-ci.yml` |
| Workflow parse | `ruby -ryaml -e 'YAML.safe_load(File.read(ARGV[0]), aliases: false)'` on the 3 workflows and `dependabot.yml` | PASS | 4 of 4 parse |
| `changes` step, extracted | step `detect` extracted with Ruby and run with `EVENT_NAME=<event>` at `HEAD` `5187abd4` | PASS | `pull_request`: 170 paths, `rust=false`; `workflow_dispatch`: `rust=true`; `push` with an invalid `BEFORE`: `rust=true` (fail-safe) |
| Change detector timing (QA-001) | `git -c core.quotePath=false diff --no-renames --name-only -z $m^1 $m \| scripts/ci-changes.sh -z` for each of the 14 first-parent merges on `main` | PASS | every merge `rust=false`; 170 paths 0.02 s, 668 paths 0.02 s, 12335 paths 0.06 s (Round 0: 16 s and more than 300 s) |
| Change detector paths (SEC-001, SEC-002) | newline and `-z` mode over the path list in `qa-report.md` §2.2 | PASS | `.cargo/config.toml`, `.cargo/config`, root `Cargo.toml` and `rust-toolchain.toml`, `.gitignore`, `.github/**`, raw and C-quoted non-ASCII `rust/` paths → `rust=true`; `src/…`, `utils/…`, `README.md`, `CLAUDE.md` → `rust=false`; empty input, a lone NUL, `__all__` and an unknown option → `rust=true` |
| QA-002 mutations replayed | the five Round 0 mutations applied to a scratch copy of the real `rust-ci.yml`, then `check-workflows` | PASS | each exits 1; the unmodified file passes |
| Upstream tree untouched | `git -C .. status --porcelain src/` | PASS | no output (checked at the start and the end) |

NOT RUN in this record, with reasons:

| Gate | Reason |
|------|--------|
| `shellcheck scripts/*.sh parity/harness.sh` | Not installed locally; runs in the CI `policy` job, which has not run |
| actionlint, zizmor (`workflow-lint`) | Not installed locally; CI only, not yet run |
| hadolint, `docker compose config` | No tools and no container engine |
| `make sbom` | cargo-cyclonedx not installed (best-effort job, POAM-2026-10-26-01) |
| `wasm-pack test`, `trunk build --release` | Not applicable in Phase 0: no wasm-bindgen tests and no app crate |
| Fuzzing | Not applicable in Phase 0: no parsers |
| TAKCL `StartupTests` against the Java reference (`harness.sh up test StartupTests`) | No container engine; CI only. The SEC-003 container-output path (`docker cp` → `reject_special` → upload strip) is covered only by self-tests |
| Plan §9.2 items 1 to 7 (GitHub runs, probes, rulesets) | CI only; no GitHub run exists yet |
| mawk and bash 5.2 (the `ubuntu-24.04` defaults) | Not installed; the first CI run of `make policy-selftest` confirms the detector under mawk |
| `aws-lc-fips-sys` build | Not part of Phase 0 |

**Note on CA-5.** The only failing gate was `make check-annotations-strict`,
because `CA-5` had no block in the mapping. This revision adds the
`### CA-5` block and removes the "CA-5 is not claimed" sentence. The
compliance-scribe re-ran the strict check after the edit; the result is in the
run's `docs-update.md`. It does not change the QA Round 1 record above, and
QA confirms it in the post-round-2 record.

**Open QA findings at this record:**

| Finding | Severity | Status | Note |
|---------|----------|--------|------|
| QA-003 | NIT | Open (deferred) | `env: RUSTUP_TOOLCHAIN: nightly` on a job passes `check-toolchain`; a `/* w */ unsafe { … }` line passes `check-unsafe`. Backstops: the per-job `rustc --version` check and `unsafe_code = "forbid"` |
| QA-004 | MINOR | Open (deferred to Phase 2) | The aws-lc-rs `fips` graph fails `[bans.build]` on two `libloading` test DLLs; see [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues |
| QA-005 | NIT | Open (deferred) | `make check-native-deps` lists `prettyplease` as well as `wasm-bindgen-shared` on wasm32; plan §3.2 expected only `wasm-bindgen-shared`. Both are `marker` entries in `scripts/native-allowlist.txt` (`prettyplease` has `links = "prettyplease02"` and no native code) |
| QA-006 | MINOR | Open at this record; fix round 2 reports it fixed | `check-workflows` rules W3b and W7 miss a quoted key or a space before the colon (five reproduced mutations). Fix round 2 moved the checks to parsed YAML; QA had not re-verified it at this record (resolved in the QA Round 2 record below) |

### 2026-10-26 20261026-210255-phase0-foundations (QA Round 2, local)

- **Who and where:** qa-engineer, macOS, the same environment as Round 1
  (§1), from `/Users/jwillman/Dev/Server/rust` unless noted. Network was up
  (`gh`, `curl`).
- **What was tested:** the uncommitted Phase 0 working tree after fix round 2
  and the Phase 5 documents (base `HEAD` = `5187abd4`). Every Round 1 gate was
  re-run, and the fix round 2 findings were re-verified. Fix round 3 changes
  scripts and `.github/` files after this record (the run's
  `fixes-round-3.md`); those changes are **not** covered by this record.
- **Source:** the run's `qa-report.md` (Round 2). Logs were kept in the
  session scratch directory, not in the repository. `<scratch>` below is that
  directory.
- **Overall: PASS.** Every gate that ran passed, including
  `make check-annotations-strict`. Tool-dependent gates are NOT RUN. Two new
  advisory findings, QA-007 (MINOR) and QA-008 (NIT), are listed below.

| Gate | Command | Result | Counts |
|------|---------|--------|--------|
| Toolchain | `rustup show active-toolchain` | PASS | `1.99.1-aarch64-apple-darwin`, overridden by `rust-toolchain.toml` |
| Lockfile | `cargo metadata --locked --format-version 1 >/dev/null` | PASS | exit 0 |
| Format | `make fmt-check` | PASS | exit 0 |
| Lint (native) | `make clippy` | PASS | 0 warnings |
| Lint (wasm32) | `make clippy-wasm` | PASS | 0 warnings |
| Build (wasm32) | `make build-wasm` | PASS | exit 0 |
| Unit and integration tests | `make test` | PASS | 5 passed, 0 failed (`tak-api-types`: lib 0, `clippy_config` 2; `tak-web-components`: lib 1, `clippy_config` 2) |
| Doc tests | `make test-doc` | PASS | 2 passed, 0 failed |
| Docs | `make doc` | PASS | exit 0 |
| Supply chain: policy | `make deny` | PASS | `advisories ok, bans ok, licenses ok, sources ok`; warnings 12 / 6 / 1 / 3, as in Round 1 |
| Supply chain: advisories | `make audit` | PASS | 93 crates, 1378 advisories, no findings; passes only with the 2 ignores of the open POAM-2026-10-26-09 |
| POAM-09 detection, deny (negative; NIST-018) | `grep -v 'RUSTSEC-2024-0370\|RUSTSEC-2025-0141' deny.toml > <scratch>/poam09/deny-no-poam09.toml` (removes only lines 53–54 as they were at the run; the ignores are now lines 54–55 after the 2026-10-27 comment edit); `cargo deny --locked --config <scratch>/poam09/deny-no-poam09.toml check advisories` | PASS (rejects) | **exit 1**, `advisories FAILED`, exactly 2 errors: `error[unmaintained]` RUSTSEC-2024-0370 (`proc-macro-error`) and RUSTSEC-2025-0141 (`bincode`). Full `check` with the same config: exit 1, `advisories FAILED, bans ok, licenses ok, sources ok`, the same 2 errors. Control with `deny.toml`: exit 0 |
| POAM-09 detection, audit (negative; NIST-018) | `<scratch>/poam09/audit/` holding a copy of `Cargo.lock` and `.cargo/audit.toml` without the two ignore lines (lines 33 and 40 removed, as they were at the run; now lines 34 and 42 after the 2026-10-27 comment edit); `cargo audit --deny warnings --file Cargo.lock` in that directory | PASS (rejects) | **exit 1**, `error: 2 denied warnings found!`: `bincode 1.3.3` RUSTSEC-2025-0141, `proc-macro-error 1.0.4` RUSTSEC-2024-0370. Control with the unmodified `audit.toml` in the same directory: exit 0. The repository files were not edited |
| Native-code gate | `make check-native-deps` | PASS | 3 targets; wasm32 `prettyplease` 0.2.37 and `wasm-bindgen-shared` 0.2.129 (`marker`) |
| Advisory-ignore sync | `make check-advisory-sync` | PASS | 2 IDs, lists identical, each with a POA&M ID |
| Policy | `make policy` | PASS | unsafe 0; manifests 2; annotations ok (0 PENDING); POA&M IDs 10 referenced, 10 entries; toolchain 1.99.1 vs 1.99; `check-workflows: ok (3 rust workflow file(s), 3 parsed)`; `check-src-coupling: ok` |
| Policy self-tests | `make policy-selftest` | PASS | 269 passed, 0 failed before the QA additions; 281 passed, 0 failed after (12 QA cases added) |
| Annotations (strict) | `make check-annotations-strict` | PASS | exit 0; 18 blocks; `check-annotations: ok` |
| Mapping-pending list empty | `grep -v '^#' scripts/mapping-pending.txt \| grep -v '^[[:space:]]*$' \| wc -l` | PASS | 0 |
| POA&M IDs | `make check-poam-ids` | PASS | 10 referenced, 10 entries |
| Markdown | `make markdownlint` | PASS | markdownlint-cli2 0.24.0: 11 files, 0 issues |
| Shell syntax | `for f in parity/harness.sh scripts/*.sh; do bash -n $f; done` | PASS | 13 files parse (adds `check-src-coupling.sh`) |
| Ruby syntax | `ruby -c scripts/check-workflows.rb`; `ruby -w -c scripts/check-workflows.rb` | PASS | `Syntax OK` twice, no warnings |
| Harness self-test | `bash parity/harness.sh selftest` | PASS | 44 passed, 0 failed |
| Workflow parse | `ruby -ryaml -e 'YAML.safe_load(File.read(ARGV[0]), aliases: false)'` on the 3 workflows and `dependabot.yml` | PASS | 4 of 4 parse |
| CI shape | `grep -L 'contents: read' ../.github/workflows/rust-*.yml`; `grep -n pull_request_target …`; `grep -l rust-ci-gate ../.github/workflows/*.yml` | PASS | no output; exit 1; only `rust-ci.yml` |
| Licence, copyleft, family coverage, licence files | as in the Round 1 record | PASS | `Apache-2.0`; grep exit 1; 20 of 20; exit 0 |
| QA-006 and SEC-004 mutations | the real `.github/workflows/` copied to a scratch `REPO_ROOT`, one mutation each, then `REPO_ROOT=<scratch> bash scripts/check-workflows.sh` | PASS | the unmodified copy exits 0; all 5 QA-006 mutations, the SEC-004 forms (block scalar, next-line scalar, `\x2D` escape in a name and in a job ID, template expression, flow style) and the missing or no-op Ruby cases each exit 1 |
| SEC-008 detector and coupling | detector in newline and `-z` mode over the paths in `qa-report.md` R2.2; `check-src-coupling` on a scratch `RUST_ROOT` | PASS | `src/takserver-protobuf/…` → `rust=true`, other `src/`, `utils/` → `rust=false`; reads of and links to `src/takserver-core`, `../src/foo`, `../utils/y` exit 1. The `..`-segment case is QA-008 |
| SEC-009 `reject_special` | library mode (`TAK_PARITY_LIB=1`) with a symlink under nested `0000` directories, a failing `chmod` stub, a failing `find` stub, a clean tree; `harness.sh verify-junit` on a directory with a `0000` subdirectory | PASS | rejection in the three fault cases, clean tree passes; `verify-junit` exit 1, then `PASS: files=1 tests=1` once readable |
| Change detector over upstream merges | `git log --first-parent --merges main`, each merge through `scripts/ci-changes.sh` in `-z` and newline mode | PASS | 14 merges; 9 `rust=true`, each explained by a `src/takserver-protobuf/` path; 0.060 s to 0.099 s per merge; both modes agree |
| Compliance-doc spot checks | 8 pins and 8 commands from this document and [ATO_EVIDENCE.md](ATO_EVIDENCE.md) re-checked | PASS | every pin matches its file; every command reproduces the Round 1 value |
| Upstream tree untouched | `git -C .. status --porcelain src/` | PASS | no output |

NOT RUN in this record, with reasons:

| Gate | Reason |
|------|--------|
| shellcheck, actionlint, zizmor, hadolint | Not installed locally; CI only, not yet run |
| `make sbom` | cargo-cyclonedx not installed (POAM-2026-10-26-01) |
| TAKCL `StartupTests`, plan §9.2 probes | No container engine; CI only |
| mawk, bash 5.2, Ruby 3.2 (the `ubuntu-24.04` defaults) | Not installed; the parsed-YAML rules are verified only on Ruby 2.6.10 until the first CI run (REL-012) |
| `wasm-pack test`, fuzzing | Not applicable in Phase 0: none exist |

**Not evidence.** The `make deny` and `make audit` passes depend on the two
ignores of the open POAM-2026-10-26-09. The two POAM-09 detection rows above
show that the policy detects both crates; they are the SA-22 evidence. They
do not accept the exception. On 2026-10-27 the system owner accepted
POAM-2026-10-26-09 for development, pending AO review; it stays Open and
the pass with the ignores is still not evidence.

**Open QA findings at this record:**

| Finding | Severity | Status | Note |
|---------|----------|--------|------|
| QA-003 | NIT | Open (deferred) | Unchanged from Round 1 |
| QA-004 | MINOR | Open (deferred to Phase 2) | Unchanged; [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues |
| QA-005 | NIT | Open (deferred) | `check-native-deps` still lists `prettyplease` on wasm32 |
| QA-006 | MINOR | Resolved | Mutations above; fixer selftests plus QA selftests |
| QA-007 | MINOR | Open at this record; fix round 3 addresses it, not QA-verified | `check-annotations.sh` rule (d) matched only an exact `Not Applicable` or `N/A` cell, so the 60 `Not Applicable (component)` rows counted as mapped: a probe citing PE-3 exited 0 in normal and strict mode, also with PE-3 in `mapping-pending.txt` |
| QA-008 | NIT | Open at this record; fix round 3 addresses it, not QA-verified | `check-src-coupling.sh` rule (r) accepted `../../src/takserver-protobuf/../takserver-core/x.java` as covered (exit 0) |

Also verified as resolved in this round: SEC-004, SEC-008, SEC-009, NIST-013
(2) and (4), NIST-014; NIST-013 (3) is partial for Not Applicable rows
(QA-007). REL-012 stays open.

## 4. TAKCL parity results

| Date | Run | Target | Test ID | Tests | Failures | Errors | Result | Artifact |
|------|-----|--------|---------|-------|----------|--------|--------|----------|
| — | — | Java | StartupTests | — | — | — | NOT RUN (no container engine locally; CI only, not yet run) | — |

Suite inventory and Java baselines: [PROTOCOL_PARITY.md](PROTOCOL_PARITY.md)
Table B.

## 5. CI probes (plan §9.2)

| Probe | Expectation | Result | Evidence |
|-------|-------------|--------|----------|
| Phase 0 PR | Every `rust-ci.yml` job succeeds; `rust-ci-gate` green; `sbom` uploads `rust-sbom` | NOT RUN | No PR opened yet |
| `src/**`-only PR | `changes` reports `rust=false`; heavy jobs skipped; `rust-ci-gate` green in under 2 minutes; no parity run | NOT RUN | Locally, the extracted `changes` step gives `rust=false` for the last upstream merge (§3); not evidence for this probe |
| Gate fails closed | Draft PR with a mis-formatted `.rs` file: `fmt` fails and `rust-ci-gate` fails | NOT RUN | — |
| Nightly supply chain | `audit`, `deny`, `native-deps` green on the first scheduled run | NOT RUN | — |
| Parity dispatch | `StartupTests` JUnit: tests at least 1, failures 0, errors 0; teardown ran | NOT RUN | — |
| Required check | Ruleset shows `rust-ci-gate` as the only required check (closes POAM-2026-10-26-04) | NOT RUN | — |

## 6. Nightly results

Listed at each phase gate (POAM-2026-10-26-07).

| Period | Workflow | Runs | Failures | Notes |
|--------|----------|------|----------|-------|
| — | `rust-supply-chain.yml` | — | — | No scheduled run yet |
| — | `rust-parity.yml` | — | — | No scheduled run yet |

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-27 | 0.4 | Gate: QA Round 2 "Not evidence" note records that POAM-2026-10-26-09 was accepted for development by the system owner on 2026-10-27, pending AO review (still Open). POAM-09 detection rows: the recorded commands are unchanged; current ignore line numbers added (`deny.toml` 54–55, `.cargo/audit.toml` 34 and 42) after the 2026-10-27 comment edits. No result changed | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs): second run record from QA Round 2 (overall PASS; every Round 1 gate re-run; POAM-09 detection negative gates for `cargo deny` and `cargo audit`, exit 1 on RUSTSEC-2024-0370 and RUSTSEC-2025-0141, cited as SA-22 detection evidence; fix round 2 re-verification; NOT RUN list; open findings QA-003, QA-004, QA-005, QA-007, QA-008); Round 1 QA-006 row points to Round 2; Ruby version in §1 | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | WU10 (pre-gate): first run record from QA Round 1 (overall PASS; local gates with commands and counts; NOT RUN list with reasons; Round 0 FAIL summary; open QA findings QA-003 to QA-006); local environment filled in; parity and §9.2 probes marked NOT RUN | `20261026-210255-phase0-foundations` (WU10) |
| 2026-10-26 | 0.1 | Initial skeleton: result vocabulary (NOT RUN never PASS), environment table, run-record template from plan §9.1, TAKCL parity and CI-probe sections (all pending), nightly results table | `20261026-210255-phase0-foundations` (WU6) |
