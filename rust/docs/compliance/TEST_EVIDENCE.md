# TAK-RS Test Evidence

**Document Version:** 0.7
**Date:** 2026-10-28
**TAK-RS Version:** 0.0.1 (Phase 1a, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 1a build run, before its gate. Phase 1a adds one local QA record (2026-10-28, overall **FAIL** for the Phase 1a gate: golden capture and golden test NOT RUN, QA-001 open at that time) and one record of the fix rounds' own runs (not QA). No CI run exists for Phase 1a: GitHub Actions is disabled for the repository. Phase 0 records (QA Rounds 1 and 2, PR 7 CI) unchanged. No scheduled (nightly) run recorded. hadolint is in no workflow and has never run.
**Run:** `20261027-180458-phase1-protocol-config`

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

Phase 1a adds tests of the first TOE library components, `tak-proto` and
`tak-cot` (unit, negative, property, integration and fuzz), the excluded fuzz
workspace, and the golden-corpus inputs and capture driver. The golden
comparison with the Java reference has not run.

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
| cargo-deny | 0.20.2 | 0.20.2 (ran in PR 7 CI, run 40539495255) |
| cargo-audit | 0.22.2 | 0.22.2 (ran in PR 7 CI, run 40539495255) |
| Other tools | jq 1.7.1, `/bin/bash` 3.2.57, BSD awk 20200816, BSD grep, Ruby 2.6.10 with stdlib psych (YAML parsing; `check-workflows.rb` from fix round 2), npx with markdownlint-cli2 0.24.0 | shellcheck 0.11.0, zizmor 1.31.1, actionlint 1.7.12, cargo-cyclonedx 0.5.9 (all ran in PR 7 CI, run 40539495255); Ruby 3.2.3; Docker 28.0.4 and Docker Compose 2.38.2 (parity run 40539495328). hadolint: in no workflow |
| Not installed locally | docker, podman, shellcheck, actionlint, zizmor, hadolint, cargo-cyclonedx, wasm-pack, trunk, gawk, mawk | — |
| Container engine | None installed | Docker (runner image) |
| Java | Temurin 27 (cannot run Gradle 8.10.2) | JDK 17 in the build container |
| Host | macOS (Darwin 27.0.0, aarch64) | `ubuntu-24.04`, runner image version 20261012.334.1 (PR 7 CI) |
| Phase 1a local host and tools | Apple M2 Max, 12 cores, 32 GiB; rustc 1.99.1 (58cf8f9c3 2026-10-13); fuzz toolchain `nightly-2026-10-23` (rustc 1.101.0-nightly 7189d39d9), cargo-fuzz 0.13.2; cargo-deny 0.20.2, cargo-audit 0.22.2, shellcheck 0.11.0, markdownlint-cli2 0.24.0. actionlint and zizmor not installed | No Phase 1a CI run (GitHub Actions disabled) |

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
| `shellcheck scripts/*.sh parity/harness.sh` | Not installed locally; runs in the CI `policy` job, which had not run at this record. It ran on 2026-10-27 (PR 7 CI record) |
| actionlint, zizmor (`workflow-lint`) | Not installed locally; CI only, not run at this record. They ran on 2026-10-27 (PR 7 CI record) |
| hadolint, `docker compose config` | No tools and no container engine. hadolint is also in no workflow, so it has not run in CI either (see the PR 7 CI record) |
| `make sbom` | cargo-cyclonedx not installed (best-effort job, POAM-2026-10-26-01) |
| `wasm-pack test`, `trunk build --release` | Not applicable in Phase 0: no wasm-bindgen tests and no app crate |
| Fuzzing | Not applicable in Phase 0: no parsers |
| TAKCL `StartupTests` against the Java reference (`harness.sh up test StartupTests`) | No container engine; CI only. The SEC-003 container-output path (`docker cp` → `reject_special` → upload strip) is covered only by self-tests. It ran in CI on 2026-10-27 (PR 7 CI record, §4) |
| Plan §9.2 items 1 to 7 (GitHub runs, probes, rulesets) | CI only; no GitHub run existed at this record. Status since then: §5 |
| mawk and bash 5.2 (the `ubuntu-24.04` defaults) | Not installed; the first CI run of `make policy-selftest` confirms the detector under mawk. It ran on 2026-10-27 (PR 7 CI record: 331/0); the log does not print the awk or bash version |
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
| shellcheck, actionlint, zizmor | Not installed locally; CI only, not run at this record. They ran on 2026-10-27 (PR 7 CI record) |
| hadolint | Not installed locally, and in **no workflow**: hadolint has never run anywhere. Until 2026-10-27 this row grouped it with the CI-only tools as "CI only, not yet run", which was wrong |
| `make sbom` | cargo-cyclonedx not installed (POAM-2026-10-26-01) |
| TAKCL `StartupTests`, plan §9.2 probes | No container engine; CI only. `StartupTests` against Java ran on 2026-10-27 (§4); the probes are in §5 |
| mawk, bash 5.2, Ruby 3.2 (the `ubuntu-24.04` defaults) | Not installed; the parsed-YAML rules are verified only on Ruby 2.6.10 until the first CI run (REL-012). On 2026-10-27 `check-workflows` passed under Ruby 3.2.3 on the runner (PR 7 CI record) |
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

### 2026-10-27 PR 7 CI (GitHub Actions)

- **Who and where:** GitHub Actions on GitHub-hosted `ubuntu-24.04` runners
  (runner image version 20261012.334.1), for
  [PR 7](https://github.com/192d-Wing/Server/pull/7). Recorded by the
  compliance-scribe from the run logs.
- **What was tested:** final PR head `ce9caf79` (event `pull_request`,
  2026-10-27T16:17:46Z). The PR was merged to `main` as `9f754245` (merge
  commit) on 2026-10-27. Nothing enforced the merge: `main` has no branch
  ruleset (POAM-2026-10-26-04).
- **Runs:**
  [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255)
  and
  [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328).
  CodeQL (repository default setup, `java-kotlin`, `javascript-typescript`,
  `python`) also succeeded; it is not part of `rust-ci-gate`.
- **Overall: PASS.** Every job of both workflows succeeded.
  `rust-ci-gate` printed `ci-gate: PASS (rust=true)`.

`rust-ci` run 40539495255, one row per job:

| Job | Command or step | Result | Counts and versions |
|-----|-----------------|--------|---------------------|
| `changes` | `scripts/ci-changes.sh` | PASS | `rust=true` |
| `workflow-lint` | actionlint, zizmor, `check-workflows`, `check-toolchain` | PASS | actionlint 1.7.12, tarball `sha256sum --check --strict` "OK"; zizmor 1.31.1 (install-action SHA-256 verified), "No findings to report"; `check-workflows: ok (3 rust workflow file(s), 3 parsed)` under Ruby 3.2.3; `check-toolchain` 1.99.1 vs 1.99 |
| `annotations-strict` | `make check-annotations-strict` | PASS | `block count 18`, `check-annotations: ok` |
| `fmt` | `make fmt-check` | PASS | rustc 1.99.1 |
| `lint (native)` | `make clippy` | PASS | 0 warnings with `-D warnings` |
| `lint (wasm32)` | `make clippy-wasm`, then the wasm32 build | PASS | 0 warnings; `cargo build … --target wasm32-unknown-unknown --locked` |
| `test` | `make test test-doc` | PASS | 5 passed, 0 failed (`tak-api-types` lib 0, `clippy_config` 2; `tak-web-components` lib 1, `clippy_config` 2); doc tests 2 passed, 0 failed |
| `doc` | `make doc` | PASS | exit 0 |
| `policy` | `make policy policy-selftest`; `shellcheck scripts/*.sh parity/harness.sh` | PASS | `check-unsafe` 0 uses; `check-manifests` 2 members; `check-poam-ids: ok (10 referenced ID(s), 10 entr(ies) in POAM.md)`; `check-workflows` ok; `check-src-coupling` ok; `selftest: 331 passed, 0 failed`; shellcheck 0.11.0 (install-action SHA-256 verified) exit 0 |
| `supply-chain` | `make deny`, `make audit`, `make check-native-deps check-advisory-sync` | PASS | cargo-deny 0.20.2 `advisories ok, bans ok, licenses ok, sources ok`; cargo-audit 0.22.2: 1380 advisories loaded, 93 crates, no findings; both pass only with the 2 ignores of the open POAM-2026-10-26-09 (not RA-5 or SA-22 evidence; see the QA Round 2 "Not evidence" note). `check-native-deps: ok` on x86_64 and aarch64 Linux (whole workspace) and wasm32 (only the `prettyplease` and `wasm-bindgen-shared` markers); `check-advisory-sync: ok (2 ignored advisory ID(s) …)` |
| `markdown` | markdownlint-cli2 action | PASS | markdownlint-cli2 0.24.0: 11 files, 0 issues |
| `sbom` | `make sbom` (non-blocking, POAM-2026-10-26-01) | PASS | cargo-cyclonedx 0.5.9; 2 `.cdx.json` files; artifact `rust-sbom` (13544 bytes, retention 90 days). Best-effort job, not part of `rust-ci-gate` |
| `rust-ci-gate` | `scripts/ci-gate.sh` | PASS | every needed job `success`, verdict `ok`; `ci-gate: PASS (rust=true)` |

`rust-parity` run 40539495328, job "parity (Java reference, TAKCL)":

| Step | Result | Detail |
|------|--------|--------|
| `gradle/actions/wrapper-validation` | PASS | — |
| Harness self-test | PASS | `selftest: 44 passed, 0 failed` |
| Java reference build (preflight, `build-java`, image) | PASS | Clone of the PR merge ref (`refs/pull/7/merge`); tag `5.7-RELEASE-14` created at `5187abd4` in the clone only; `wrapper verified: Gradle 8.10.2`, wrapper jar SHA-256 `91a2…1e60`; Docker 28.0.4, Compose 2.38.2. `git diff 5187abd4 9f754245 -- src` is empty, so `src/` is the 5.7-RELEASE-14 tree |
| TAKCL `StartupTests` against Java | PASS | `harness: test StartupTests: PASS`; test `[dbAvailableEnabled][Java17.0.20.1]StartupTests.jarStartupValiationTest`; `verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0` |
| Teardown | PASS | `harness: down: ok` |
| Strip key material, symlinks and special files | PASS | — |
| Upload | PASS | artifact `parity-results-40539495328-1`, 76550 bytes, retention 90 days (expires 2027-01-25) |

**Earlier runs on PR 7 (the gates detected failures).** Each failure was
fixed in a later commit on the PR. The failing `rust-ci` runs on `77beace3`
and `0c56b62a` show that a failing required job turns `rust-ci-gate` red.

| Head | rust-ci | rust-parity | What failed | Fix |
|------|---------|-------------|-------------|-----|
| `77beace3` | [40525846385](https://github.com/192d-Wing/Server/actions/runs/40525846385) failure | [40525846019](https://github.com/192d-Wing/Server/actions/runs/40525846019) failure | `make policy-selftest`: 17 of 331 cases failed, because the CI's `BASE_REF` leaked into the fixtures. Parity: Gradle `applyGrpcPatch` needs `patch`, absent from the build image | `0c56b62a`: the self-test unsets the CI inputs; `Dockerfile.java-build` installs `patch` |
| `0c56b62a` | [40536002508](https://github.com/192d-Wing/Server/actions/runs/40536002508) failure | [40536002486](https://github.com/192d-Wing/Server/actions/runs/40536002486) cancelled | First shellcheck run: SC2209 in `ci-changes.sh`, plus intentional patterns | `de62d29e`: SC2209 fixed; inline disables with reasons |
| `de62d29e` | [40536818117](https://github.com/192d-Wing/Server/actions/runs/40536818117) success | [40536815081](https://github.com/192d-Wing/Server/actions/runs/40536815081) failure | `takserver-tool-ui` `react-scripts build`: ESLint config "jest/globals" fails with no npm lockfile | `ce9caf79`: `DISABLE_ESLINT_PLUGIN=true` for the reference build (under POAM-2026-10-26-02) |

NOT RUN or not shown by these runs, with reasons:

| Gate | Reason |
|------|--------|
| hadolint | In **no workflow**; it has never run, locally or in CI. The two parity Dockerfiles ([parity/docker/](../../parity/docker/)) have no linter. BuildKit printed two `InvalidDefaultArgInFrom` build-check warnings (the base-image `ARG`s have no default, so the digest from `pins.env` must be passed); no gate evaluates them |
| Branch protection | `main` has no ruleset; `rust-ci-gate` is not a required check (POAM-2026-10-26-04). The merge followed green checks by the user's decision, not by enforcement |
| Nightly runs | No scheduled run of `rust-supply-chain.yml` or `rust-parity.yml` yet (§6) |
| Push-to-`main` runs on `9f754245` | In progress when this record was written (rust-ci 40557332845, rust-parity 40557332959); not claimed |
| Plan §9.2 probes | Not run as separate PRs (§5) |
| Parity against the Rust server | Not possible yet (POAM-2026-10-26-06) |
| Dependabot security updates | Still paused (POAM-2026-10-26-10) |
| awk and bash versions on the runner | `make policy-selftest` passed on the runner, but the log does not print which awk or bash ran it |

### 2026-10-28 20261027-180458-phase1-protocol-config (Phase 1a QA, local)

- **Who and where:** qa-engineer, macOS (Phase 1a row of §1), from
  `/Users/jwillman/Dev/Server/rust` unless noted.
- **What was tested:** `main` (`df6a4662`) to `HEAD` (`04774cc0`: WU1, WU2,
  WU6a, WU6b committed) plus the uncommitted working tree (WU3, WU4, WU5,
  WU9), **before** fix rounds 1 and 2. Nothing has been committed since
  `04774cc0` (commit signing temporarily unavailable).
- **Source:** the run's `qa-report.md`. Logs are in the run directory
  (`qa-gate-logs/`, `qa-fuzz-logs/`), not in the repository.
- **Overall: FAIL for the Phase 1a gate.** Every build, lint, test,
  supply-chain, policy and fuzz gate that could run here passed, except the
  expected `make check-annotations-strict` failure. The gate fails because
  the golden capture (WU7) and golden test (WU8), the Phase 1a exit
  criterion, are NOT RUN, and QA-001 (MAJOR) was open. QA-001 was fixed in
  fix round 1 (next record), which QA has not re-run.

| Gate | Command | Result | Counts |
|------|---------|--------|--------|
| Format | `make fmt-check` | PASS | no diff |
| Lint (native) | `make clippy` (`--workspace --all-targets --all-features --locked -- -D warnings`) | PASS | 0 warnings; re-run after the last test edit, still 0 |
| Lint (wasm32) | `make clippy-wasm` (`tak-api-types`, `tak-web-components`, `tak-proto`, `tak-cot`) | PASS | 0 warnings |
| Build (wasm32) | `make build-wasm` | PASS | `tak-proto` and `tak-cot` built for `wasm32-unknown-unknown` |
| Unit, property and integration tests | `make test` | PASS | 19 test binaries, **220 passed, 0 failed, 0 ignored** (198 before the 22 QA tests); property tests 512 fixed-seed cases each |
| Golden parity (WU8) | `tak-cot/tests/golden.rs` in `make test` | **NOT RUN** | No goldens exist (WU7 blocked); `golden.rs` was not added, because it must fail when `golden/` is missing |
| Corpus capture (WU7) | `rust-parity` dispatch: `harness.sh capture-corpus verify-corpus` | **NOT RUN** | GitHub Actions dispatch fails: "Actions has been disabled for this repository"; no run URL |
| Doc tests | `make test-doc` | PASS | 2 passed |
| Docs | `make doc` (`RUSTDOCFLAGS=-D warnings`) | PASS | 4 crates |
| Supply chain: policy | `make deny` | PASS | `advisories ok, bans ok, licenses ok, sources ok`; 22 known warnings (unmatched wrapper or allow entries, duplicate versions) |
| Supply chain: advisories | `make audit` | PASS | 1380 advisories, 143 crates, no findings; passes only with the 2 ignores of the open POAM-2026-10-26-09 (not RA-5 or SA-22 evidence) |
| Native-code gate | `make check-native-deps` | PASS | wasm32 closure of `tak-api-types`, `tak-web-components`, `tak-proto`, `tak-cot`: `check-native-deps: ok` |
| Advisory-ignore sync | `make check-advisory-sync` | PASS | 2 IDs, lists identical, each with a POA&M |
| Policy | `make policy` | PASS | `check-unsafe`, `check-manifests`, `check-annotations` (2 PENDING), `check-poam-ids` (10/10), `check-toolchain`, `check-workflows` (4 files), `check-src-coupling` (3 references, all covered), `check-fuzz-policy` ok |
| Policy self-tests | `make policy-selftest` | PASS | 381 passed, 0 failed |
| Annotations | `make check-annotations` | PASS | 2 PENDING (SC-5, SI-10(3)); block count 50 |
| Annotations (strict) | `make check-annotations-strict` | FAIL (expected) | exit 2: `FAIL (d) SC-5 …` and `FAIL (d) SI-10(3) …` only; their mapping blocks are added in WU10 (this revision) |
| Markdown | `make markdownlint` | PASS | 13 files, 0 issues |
| Harness self-test | `parity/harness.sh selftest` | PASS | 97 passed, 0 failed |
| Shell | `shellcheck scripts/*.sh parity/harness.sh` | PASS | 15 files, no output |
| Fuzz supply chain | `make fuzz-deny` (cargo-deny with `fuzz/deny.toml`, cargo audit on `fuzz/Cargo.lock`, native allow-list) | PASS | `libfuzzer-sys (native) allow-listed for aarch64-apple-darwin, not in graph`; `check-native-deps: ok` |
| Fuzz build | `make fuzz-build` (`cargo fuzz build --debug-assertions`, AddressSanitizer) | PASS | 7 binaries |
| Fuzz format and lint | `cd fuzz && cargo fmt --all -- --check`; `cd fuzz && cargo clippy --all-targets -- -D warnings` | PASS | exit 0 and 0 |
| Fuzz runs | 7 targets × 300 s (table below) | PASS | 0 crashes, 0 timeouts, 0 OOMs; `fuzz/artifacts/` empty |
| No generated code committed | `git ls-files rust \| grep -E 'tak_proto_includes\|atakmap\.commoncommo'` | PASS | no match |
| Upstream tree untouched | `git status --porcelain src/` (repository root) | PASS | no output, checked at the end of the run |
| Unsafe | `make check-unsafe` (in `policy`) | PASS | both crates `#![forbid(unsafe_code)]`; fuzz crate `unsafe_code = "forbid"` |

**Fuzz runs.** `make fuzz-build` and `make fuzz-seeds` ran once (exit 0).
The seven targets then ran in parallel, each with the `fuzz-run` recipe body
(`cargo fuzz run --debug-assertions <t> corpus/<t> seeds/<t> --
-max_total_time=300 -max_len=65536 -rss_limit_mb=2048 -timeout=1`) under
`/usr/bin/time -l`, starting 2026-10-28T06:11:01Z from the corpus left by
the WU9 60 s runs plus seeds from `parity/corpus/cases/`. (`make fuzz-run`
itself could not run in parallel then; QA-003, fixed in fix round 1.)

| Target | Exit | Executions in 301 s | Final coverage | Peak RSS | Crash / timeout / OOM |
|--------|------|---------------------|----------------|----------|------------------------|
| `cot_parse` | 0 | 3 454 337 | cov 2602, ft 14085 | 632 MB | 0 / 0 / 0 |
| `cot_to_proto` | 0 | 3 779 863 | cov 4412, ft 17310 | 636 MB | 0 / 0 / 0 |
| `proto_decode` | 0 | 40 808 939 | cov 1317, ft 2626 | 818 MB | 0 / 0 / 0 |
| `proto_to_cot` | 0 | 16 804 603 | cov 3161, ft 7730 | 583 MB | 0 / 0 / 0 |
| `proto_roundtrip` | 0 | 19 010 364 | cov 2830, ft 6778 | 703 MB | 0 / 0 / 0 |
| `stream_frame` | 0 | 15 394 966 | cov 154, ft 609 | 696 MB | 0 / 0 / 0 |
| `javafmt` | 0 | 26 166 426 | cov 916, ft 2206 | 557 MB | 0 / 0 / 0 |
| **Total** | | **125 419 498** | | max 818 MB of 2048 MB | **0 / 0 / 0** |

The total is the sum of the seven rows. `qa-report.md` §7 says "79 M fuzz
executions in total"; the table it reports sums to 125 419 498, which is the
figure used here.

**Limits at the boundary.** Every documented limit is accepted at its value
and refused one past it with a typed error, and no case returns a partial
document or event (`qa-report.md` §3): `max_input_bytes`, `max_depth`
(including depth measured from the event root for proto `xmlDetail`),
`max_nodes`, `max_attributes`, `max_namespace_decls`, `max_name_bytes`
(including `xmlns:` names), `max_attr_value_bytes`, `max_text_bytes`,
`max_xml_detail_bytes` (P2X and X2P), the 8 MiB TakMessage and stream-frame
caps, the 5-byte / u32 varint length prefix and prost's recursion limit
(100 accepted, 101 refused; 10^6 refused without stack overflow).

**Error content (SI-11).** With the marker `Qz7Mark` in every name, value
and text, no `Display`, `Debug` or `source()` chain contains it, for 25 XML
error and limit kinds through three entry points, every conversion error
kind and `DecodeError`; `Debug` of `CotEvent`, `Document` and `ElementRef` is
content-free.

**Tests added by QA** (test code only, both new files):
`crates/tak-cot/tests/qa_boundaries.rs` (11 tests) and
`crates/tak-proto/tests/decode_hostile.rs` (11 tests); `cargo test -p tak-cot
--test qa_boundaries --locked` 11 passed, `cargo test -p tak-proto --test
decode_hostile --locked` 11 passed. Both are included in the 220.

NOT RUN in this record, with reasons:

| Gate | Reason |
|------|--------|
| Golden parity (WU8) and corpus capture (WU7) | GitHub Actions is disabled for the repository, so the capture cannot be dispatched and no goldens exist. Equivalence with the Java reference is unproven in every direction; the Phase 1a exit criterion is not met |
| actionlint, zizmor | Not installed locally; the `workflow-lint` job in rust-ci would run them, but no CI run exists |
| `rust-ci-gate` on a PR | No PR, and GitHub Actions is disabled |
| Nightly or PR fuzzing (`rust-fuzz.yml`) | GitHub Actions is disabled; only the local runs above exist |
| WASM browser tests, `trunk` | Not applicable: no `rust/web/**` change |
| Integration with PostgreSQL | Not applicable: no service behaviour in Phase 1a |
| `encode_stream_frame` with a message over `u32::MAX` bytes | Needs more than 4 GiB of memory; the code path uses a checked conversion (by inspection) |

**Open QA findings at this record:**

| Finding | Severity | Status | Note |
|---------|----------|--------|------|
| QA-001 | MAJOR | Open at this record; fixed in fix round 1, not QA re-verified | Quadratic CPU in the proto `xmlDetail` merge: 49 000 + 49 000 elements (686 000 bytes) took 4.31 s in a release build |
| QA-002 | MINOR | Open at this record; fixed in fix round 1, not QA re-verified | Effective P2X `xmlDetail` cap was 8 MiB − 55 bytes, refused with `xml.limit.input_bytes` |
| QA-003 | MINOR | Open at this record; fixed in fix round 1 (release-engineer), not QA re-verified | `make fuzz-run` regenerated every seed directory, so parallel runs raced |
| QA-004 | MINOR | Open | No coverage for size-dependent CPU cost: fuzzing capped at 64 KiB, tests cannot time themselves |

### 2026-10-28 20261027-180458-phase1-protocol-config (Phase 1a fix rounds 1 and 2; fixers' runs, not QA)

- **Who and where:** rust-coder and release-engineer, macOS, the same
  working tree, after the QA record above; then re-verification by the
  security, nist-800-53, tak-protocol and crypto-tls reviewers.
- **Source:** the run's `fixes-round-1.md`, `fixes-round-1-release.md`,
  `fixes-round-2.md`, `fixes-round-2-release.md` and the
  `findings-*-round1.md` / `findings-*-round2.md` re-verification reports.
- **Status of this evidence:** these are the fixers' own runs and the
  reviewers' checks, **not** a QA re-run. They are recorded so the gaps they
  close are visible; they raise no control status on their own
  ([NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) rule 6).

| Gate or check | Round | Result | Counts or output |
|---------------|-------|--------|------------------|
| `make test` | 1 | PASS | 223 passed, 0 failed, 0 ignored (220 + 2 conversion unit tests, −1 + 2 in `qa_boundaries.rs`) |
| QA-001 regression test | 1 | PASS | `convert::tests::xml_detail_merge_is_linear_in_the_element_count` (`crates/tak-cot/src/convert.rs` line 701): at most 8·n child steps at n = 1 000 and 4 000. With the old scan restored it failed (exit 101, 1 002 999 steps at n = 1 000); code restored |
| QA-001 timings (release build, reproducer copied in and removed) | 1 | — | n = 5 000 / 10 000 / 20 000 / 40 000: 2.2 / 3.7 / 7.4 / 13.3 ms (were 69 ms / 234 ms / 826 ms / 2.98 s); n = 49 000: 15.0 ms (was 4.31 s); n = 98 000 (`max_nodes` raised): 28.6 ms, ratio 1.98; P2P worst case 19.4 ms (was 4.30 s) |
| QA-002 exact cap | 1 | PASS | `p2x_xml_detail_cap_is_exact_at_default_limits`, `p2x_xml_detail_cap_does_not_depend_on_max_input_bytes` in `make test` |
| Schema digest, tampered manifest | 1 | PASS (fails closed) | changed digest and deleted line each fail the build (exit 101); manifest restored |
| `make policy-selftest` | 1 | PASS | 392 passed, 0 failed (381 + 11 `fuzz-seeds` cases) |
| Concurrent `make fuzz-run` (QA-003) | 1 | PASS | `proto_decode` and `javafmt`, 20 s each, started together: exit 0 and 0; each regenerated only its own seeds |
| `make check-annotations` | 1 | PASS | 2 PENDING (SC-5, SI-10(3)); block count 54 |
| `make check-annotations-strict` | 1 | FAIL (expected) | exit 2 on SC-5 and SI-10(3) only |
| fmt, clippy, clippy-wasm, test-doc, doc, build-wasm, fuzz-build, `make policy` | 1 | PASS | exit 0 each |
| `make test` | 2 | PASS | exit 0 (count not reported) |
| Schema digest cases A–F (scratch copy outside the repository) | 2 | PASS (as expected) | A nested import, B symlinked import, D shadowed well-known import, E CRLF checkout, F `EXTRA.PROTO`: each exit 101 with the expected message; C bundled well-known import: exit 0; baseline and restored: exit 0 |
| `make policy-selftest` | 2 | PASS | 419 passed, 0 failed (adds 9 F9 cases and 17 `check-poam-markers` cases) |
| `make check-annotations` | 2 | PASS | 3 PENDING (SA-10(1), SC-5, SI-10(3)); block count 55 |
| `make check-poam-markers` | 2 | FAIL (expected) | make exit 2 (script exit 1): 2 of 13 markers without an ID, `crates/tak-cot/src/limits.rs:18` and `crates/tak-cot/src/xml/serialize.rs:45` |
| `make check-annotations-strict` | 2 | FAIL (expected) | exit 2: the 3 pending IDs and the 2 bare markers |
| `make policy`, `make fuzz-deny`, `make check-workflows`, shellcheck | 2 | PASS | `check-poam-ids` 10/10; F9 ok |
| Schema digest review (crypto-tls-reviewer) | 1 and 2 | PASS | Round 1 review: `shasum` of the 15 schemas equals `schema.sha256`; SHA-256 differential against Python `hashlib` on lengths 0 to 1100 and one 5 000 003-byte input, 1102/1102 match; FIPS 180-4 vectors pass. Round 2: `cargo build -p tak-proto --offline` exit 0 |

After these rounds the reviewers' verdicts were: security PASS, crypto-tls
PASS, nist-800-53 APPROVE WITH COMMENTS (round scope), tak-protocol APPROVE
WITH COMMENTS for the round, with the run level still CHANGES REQUESTED for
PROTO-001 (user decision) and PROTO-002 (needs goldens).

The WU10 results of this revision (strict annotations, policy, markdownlint
after the mapping blocks were added) are in the run's `docs-update.md`; they
are the compliance-scribe's runs, not QA evidence.

The round 2 `make check-poam-markers` and `make check-annotations-strict`
FAIL rows above describe the tree before the WU10 follow-up, not its current
state. In that follow-up the orchestrator gave every POA&M marker its ID and
added `check-poam-markers` to `POLICY_CHECKS`. The compliance-scribe's local
runs on 2026-10-28 afterwards: `make policy` exit 0, with
`check-poam-markers: ok (15 marker(s), each with a POA&M ID)` and
`check-poam-ids: ok (13 referenced ID(s), 13 entr(ies) in POAM.md)`;
`make check-annotations-strict` exit 0. These are not QA evidence: QA has not
re-run either check, and no CI run exists (details in the run's
`docs-update.md`, section "WU10 follow-up").

## 4. TAKCL parity results

| Date | Run | Target | Test ID | Tests | Failures | Errors | Result | Artifact |
|------|-----|--------|---------|-------|----------|--------|--------|----------|
| 2026-10-27 | [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328) (PR 7, head `ce9caf79`) | Java (`5.7-RELEASE-14`) | StartupTests (`[dbAvailableEnabled][Java17.0.20.1]StartupTests.jarStartupValiationTest`) | 1 | 0 | 0 | PASS (`verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0`) | `parity-results-40539495328-1` (76550 bytes, expires 2027-01-25) |
| — | — | Rust | StartupTests | — | — | — | NOT RUN (the Rust target is not possible yet; POAM-2026-10-26-06) | — |

Suite inventory and Java baselines: [PROTOCOL_PARITY.md](PROTOCOL_PARITY.md)
Table B.

**Phase 1a golden corpus** (plan §1.4; exact bytes, see
[PROTOCOL_PARITY.md](PROTOCOL_PARITY.md) §Normalisation rules):

| Direction | Cases (inputs) | Java capture | Golden test | Result |
|-----------|----------------|--------------|-------------|--------|
| X2P, P2X, X2X, P2P, numbers, times | 715 manifest rows, 471 case IDs ([parity/corpus/MANIFEST.tsv](../../parity/corpus/MANIFEST.tsv)) | NOT RUN (GitHub Actions disabled; no run URL) | Not written (WU8; it must fail when `golden/` is missing) | **NOT RUN**. No byte-equivalence with the Java reference is shown |

## 5. CI probes (plan §9.2)

| Probe | Expectation | Result | Evidence |
|-------|-------------|--------|----------|
| Phase 0 PR | Every `rust-ci.yml` job succeeds; `rust-ci-gate` green; `sbom` uploads `rust-sbom` | PASS | PR 7, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255): 13 of 13 jobs `success`, `ci-gate: PASS (rust=true)`, `rust-sbom` uploaded (13544 bytes). §3, PR 7 CI record |
| `src/**`-only PR | `changes` reports `rust=false`; heavy jobs skipped; `rust-ci-gate` green in under 2 minutes; no parity run | NOT RUN | No `src/**`-only PR has been opened. Locally, the extracted `changes` step gives `rust=false` for the last upstream merge (§3); not evidence for this probe |
| Gate fails closed | Draft PR with a mis-formatted `.rs` file: `fmt` fails and `rust-ci-gate` fails | NOT RUN (as specified) | The deliberate probe was not run. Related evidence: on PR 7, a real `policy` failure turned `rust-ci-gate` red in [40525846385](https://github.com/192d-Wing/Server/actions/runs/40525846385) (head `77beace3`) and [40536002508](https://github.com/192d-Wing/Server/actions/runs/40536002508) (head `0c56b62a`); §3, PR 7 CI record. That shows a failing required job fails the gate; it does not exercise the `fmt` path |
| Nightly supply chain | `audit`, `deny`, `native-deps` green on the first scheduled run | NOT RUN | — |
| Parity dispatch | `StartupTests` JUnit: tests at least 1, failures 0, errors 0; teardown ran | PASS (on `pull_request`) | [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328): tests 1, failures 0, errors 0; `harness: down: ok`. Triggered by the PR, not by `workflow_dispatch`; the manual trigger itself is not exercised |
| Required check | Ruleset shows `rust-ci-gate` as the only required check (closes POAM-2026-10-26-04) | NOT RUN | No ruleset on `main`; POAM-2026-10-26-04 stays Open |

## 6. Nightly results

Listed at each phase gate (POAM-2026-10-26-07).

| Period | Workflow | Runs | Failures | Notes |
|--------|----------|------|----------|-------|
| — | `rust-supply-chain.yml` | — | — | No scheduled run yet (as of 2026-10-27) |
| — | `rust-parity.yml` | — | — | No scheduled run yet (as of 2026-10-27). The PR 7 run (§4) was a `pull_request` run, not a nightly one |
| — | `rust-fuzz.yml` | — | — | No run of any kind (as of 2026-10-28): the workflow is new in Phase 1a and GitHub Actions is disabled for the repository |

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-28 | 0.7 | Phase 1a WU10 follow-up (pre-gate). §3 fix-rounds record: note after the table that the round 2 `check-poam-markers` and `check-annotations-strict` FAIL rows predate the follow-up; the compliance-scribe's local runs afterwards (`make policy` exit 0 with `check-poam-markers` 15/15 and `check-poam-ids` 13/13; `make check-annotations-strict` exit 0), marked as not QA evidence. Round 2 rows unchanged | `20261027-180458-phase1-protocol-config` (WU10 follow-up) |
| 2026-10-28 | 0.6 | Phase 1a (WU10, pre-gate). New run record "Phase 1a QA, local" (overall FAIL for the gate: WU7 and WU8 NOT RUN, QA-001 open; every runnable gate PASS except the expected strict-annotations FAIL; `make test` 220/0; `make policy-selftest` 381/0; harness selftest 97/0; fuzz 7 × 300 s, 0 crashes, 125 419 498 executions by the sum of QA's table, not the 79 M in its prose; limits and SI-11 summaries; NOT RUN list; QA-001 to QA-004). New record of fix rounds 1 and 2 as the fixers' and reviewers' runs, not QA (`make test` 223/0, policy-selftest 392/0 then 419/0, QA-001 timings, schema-digest fail-closed cases, `check-poam-markers` 2 of 13 bare). §1 Phase 1a host and tool row; §4 golden corpus table (NOT RUN); §6 `rust-fuzz.yml` (no run) | `20261027-180458-phase1-protocol-config` (Phase 5, WU10) |
| 2026-10-27 | 0.5 | PR 7 CI evidence: new run record "2026-10-27 PR 7 CI (GitHub Actions)" (rust-ci 40539495255 and rust-parity 40539495328 on head `ce9caf79`, merged as `9f754245`; every job PASS; tool versions; `StartupTests` against Java PASS with the `verify-junit` line; artifacts `rust-sbom` and `parity-results-40539495328-1`); the three earlier failing runs recorded as detection evidence; §1 CI column filled; §4 Java `StartupTests` row; §5 Phase 0 PR and parity probes PASS, the others NOT RUN; NOT RUN reasons in the QA Round 1 and 2 records annotated with the later CI result. hadolint: separated from the CI-only tools; it is in no workflow and has never run (the QA Round 2 row had said "CI only") | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.4 | Gate: QA Round 2 "Not evidence" note records that POAM-2026-10-26-09 was accepted for development by the system owner on 2026-10-27, pending AO review (still Open). POAM-09 detection rows: the recorded commands are unchanged; current ignore line numbers added (`deny.toml` 54–55, `.cargo/audit.toml` 34 and 42) after the 2026-10-27 comment edits. No result changed | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs): second run record from QA Round 2 (overall PASS; every Round 1 gate re-run; POAM-09 detection negative gates for `cargo deny` and `cargo audit`, exit 1 on RUSTSEC-2024-0370 and RUSTSEC-2025-0141, cited as SA-22 detection evidence; fix round 2 re-verification; NOT RUN list; open findings QA-003, QA-004, QA-005, QA-007, QA-008); Round 1 QA-006 row points to Round 2; Ruby version in §1 | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | WU10 (pre-gate): first run record from QA Round 1 (overall PASS; local gates with commands and counts; NOT RUN list with reasons; Round 0 FAIL summary; open QA findings QA-003 to QA-006); local environment filled in; parity and §9.2 probes marked NOT RUN | `20261026-210255-phase0-foundations` (WU10) |
| 2026-10-26 | 0.1 | Initial skeleton: result vocabulary (NOT RUN never PASS), environment table, run-record template from plan §9.1, TAKCL parity and CI-probe sections (all pending), nightly results table | `20261026-210255-phase0-foundations` (WU6) |
