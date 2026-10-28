# TAK-RS Plan of Action and Milestones (POA&M)

**Document Version:** 0.7
**Date:** 2026-10-28
**TAK-RS Version:** 0.0.1 (Phase 1a, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 1a build run, before its gate. 13 open entries, 0 closed, 0 risk-accepted by the authorizing official. New: POAM-2026-10-27-01 (libFuzzer exception, approved by the user as plan Q2; still Open), -02 and -03. POAM-2026-10-26-05 widened (decision Q9) and -02 extended to the corpus capture. POAM-2026-10-26-09 unchanged: accepted for development by the system owner (2026-10-27), Open pending AO review. WU10 follow-up: the code markers of POAM-2026-10-27-01 to -03 carry their IDs, and `check-poam-markers` runs in `make policy`; no status changed.
**Run:** `20261027-180458-phase1-protocol-config`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed; [ROADMAP.md](../../ROADMAP.md) §6 Q5) |
| Overlay | CNSSI 1253 (TBD) |
| STIG | DISA ASD STIG, release TBD |

### Scope

Phase 0: development environment only. Entries record weaknesses in the
developer process, CI, supply chain and licensing.

Phase 1a adds the first TOE library components (`tak-proto`, `tak-cot`; no
network wiring until Phase 3). POAM-2026-10-27-02 and -03 are the first
entries about TOE code; POAM-2026-10-27-01 concerns the development-only fuzz
workspace.

### Conventions

- **ID format:** `POAM-<opened date>-<two-digit sequence>`. IDs are never
  reused once a committed document refers to them.
  [scripts/check-poam-ids.sh](../../scripts/check-poam-ids.sh) (run by
  `make policy`) fails if any ID referenced under `rust/` or in
  `.github/workflows/rust-*.yml` has no entry in the register below.
- **Code locations:** an open gap in code or configuration carries a
  `// POAM: <ID> <description>` comment (`# POAM:` in TOML, YAML and shell).
  The marker records that the gap is tracked here; it does not mean the gap is
  accepted. Acceptance, if any, is recorded in this register. The
  **Locations** column lists the markers and other references. A location is
  listed only after it is verified with `grep -n`. Line numbers are given only
  for files that fix rounds 2 and 3 do not change; for the others the element
  is named instead. Phase 1a line numbers were verified on 2026-10-28,
  including the POA&M markers the orchestrator added in the WU10 follow-up. A
  marker that does not exist yet is listed as **pending**, never as a
  location.
- **Marker IDs:** since Phase 1a fix round 2,
  [scripts/check-poam-markers.sh](../../scripts/check-poam-markers.sh) fails
  on any `POAM:` marker comment whose first token is not a
  `POAM-YYYY-MM-DD-NN` ID. Since the Phase 1a WU10 follow-up it runs in
  `make policy` (`POLICY_CHECKS`) and in `make check-annotations-strict`. It
  scans `*.rs *.sh *.toml *.yml *.yaml Makefile` only (NIST-013), so a marker
  in a `.txt` file is not checked by it.
- **Risk** is the developer's proposed rating (H, M or L). The ISSO or
  authorizing official confirms it; until then it is marked "proposed".
- **Owner** is a role. A named person is assigned when the program office
  staffs the role.
- **Status:** Open, Risk Accepted (approved by the authorizing official),
  Closed (moved to [Closed items](#closed-items) with the closing evidence).
  A decision by the user or the program office is recorded in the entry with
  its date, and the status stays Open until the authorizing official
  decides.
- Control IDs are in Rev 5 form; see
  [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md).

## Register

| ID | Control(s) | Weakness | Source | Risk (H/M/L) | Mitigation | Milestone/Phase | Owner | Opened | Status | Locations (// POAM: refs) |
|----|-----------|----------|--------|--------------|------------|-----------------|-------|--------|--------|---------------------------|
| POAM-2026-10-26-01 | CM-8, SR-4 | SBOM generated best-effort only; not a required, signed release artifact | Plan §7.2 | L (proposed) | CycloneDX SBOM job uploads `rust-sbom` but does not gate | Blocking at the first shipped binary (Phase 3); signed in Phase 9 | release-engineer | 2026-10-26 | Open | [rust-ci.yml](../../../.github/workflows/rust-ci.yml): `# POAM:` comment on the `sbom` job and the header comment; [Makefile](../../Makefile): `sbom` target comment |
| POAM-2026-10-26-02 | SR-3, SR-4, SR-11 | Parity harness builds the Java reference with unverified Maven, Gradle-plugin, Node and npm downloads; cached Gradle modules are trust-on-first-use. Since Phase 1a the golden-corpus capture runs on the same unverified build | Plan §5.2, §7.2; SEC-005; Phase 1a plan §7.2 | M (proposed) | Test tooling outside the TOE; isolated workflow, no secrets, read-only token, wrapper and distribution checksums; cache limited to downloaded modules. Capture: sandboxed container, outputs imported only through `import-corpus` | Revisit if upstream adds dependency-verification metadata | release-engineer | 2026-10-26 | Open | [parity/harness.sh](../../parity/harness.sh): `# POAM:` marker in the header (line 12); [rust-parity.yml](../../../.github/workflows/rust-parity.yml): header comment (line 20); [parity/README.md](../../parity/README.md): "Residual risk" bullet |
| POAM-2026-10-26-03 | SR-3, CM-6 | Parity fixture containers deviate from the container policy: all three parity images are Docker Hub images, not UBI or Iron Bank (`eclipse-temurin` 17 JDK and JRE on Ubuntu 22.04; community `postgis/postgis` 15-3.3, whose pinned tag was last pushed 2023-08-14 and is amd64 only), pinned by digest only; the `takcl` container's root filesystem is not read-only | Plan §7.2; WU7 REL-001, REL-004; NIST-021 | L (proposed) | Test fixtures only, never shipped, outside the TOE; digest pins; ephemeral; `db` and `takcl` on an internal network with no egress; every parity container runs with `no-new-privileges` and `--cap-drop ALL` (`db` adds back five capabilities) | Consider Iron Bank or UBI images for the JDK, JRE and PostGIS fixtures when the STIG and Iron Bank baseline is set; Phase 2 `tak-db` decides whether the parity DB moves to a maintained tag in step with upstream `testrunner.sh`; a read-only `takcl` root filesystem with a writable volume for `/opt/tak` (proposed) | release-engineer | 2026-10-26 | Open (deviation **not accepted**) | [parity/pins.env](../../parity/pins.env): `# POAM:` marker above `POSTGIS_IMAGE`; [parity/compose.yaml](../../parity/compose.yaml): "Not read_only" comment on the `takcl` service (no `POAM:` marker); [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §5 container policy deviation |
| POAM-2026-10-26-04 | CM-3, CM-3(2), CM-5, SA-10 | Branch ruleset not configured or verified; no `CODEOWNERS`; repository Actions policy allows any action and does not require full-SHA pins | Plan §3.4, §7.2; NIST-009; WU5 REL-008 | M (proposed) | Repository administrator configures the ruleset after the first green `rust-ci-gate` run | First green `rust-ci-gate` run, then admin action | Repository administrator | 2026-10-26 | Open | [rust-ci.yml](../../../.github/workflows/rust-ci.yml): header comment; [CONTRIBUTING.md](../../CONTRIBUTING.md) §CI |
| POAM-2026-10-26-05 | CM-10, SR-4 | Derivative-work and licensing review of Apache-2.0 TAK-RS against the GPLv3 Java reference not done. Widened in Phase 1a (decision Q9): generated prost code compiled from the in-place GPLv3 schemas; Java-generated corpus goldens; the Apache-2.0 Java corpus driver linked at capture time against the GPLv3 jars | Plan §1.6, §7.2; user decision D11; Phase 1a plan §7.2, decision Q9 (2026-10-27) | M (proposed) | Provenance rule ([CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance); schemas consumed in place, never copied into `rust/`; corpus inputs synthetic; goldens only through `import-corpus` with `PROVENANCE.json` | Per Q9, Phase 1a merges without waiting for the opinion; opinion at the latest before the first release (Phase 3) | Program office / legal counsel | 2026-10-26 | Open | No `POAM:` marker (no code gap). Referenced by [deny.toml](../../deny.toml) line 210 (`tak-proto` build-script entry), [crates/tak-proto/build.rs](../../crates/tak-proto/build.rs) line 14, [scripts/ci-changes.sh](../../scripts/ci-changes.sh) line 25, [parity/harness.sh](../../parity/harness.sh) line 670 (capture provenance text), [parity/corpus/README.md](../../parity/corpus/README.md) line 189, [fuzz/deny.toml](../../fuzz/deny.toml) (`tak-proto` build-script entry), [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance, [ARCHITECTURE.md](../../ARCHITECTURE.md) §Licence boundary and [NOTICE](../../NOTICE) line 8 |
| POAM-2026-10-26-06 | SA-11 | TAKCL cannot target an external server; the Rust target relies on a planned `java` launcher shim, and the Ignite file-auth path is unsolved | Plan §1.2, §7.2 | L (proposed) | `harness.sh test --target rust` fails closed (exit 2) | Phase 3 plan run designs it before the Phase 3 exit tests | tak-architect (Phase 3 plan) | 2026-10-26 | Open | [parity/harness.sh](../../parity/harness.sh): `# POAM:` marker at the `--target rust` refusal |
| POAM-2026-10-26-07 | SI-2(2), RA-5 | Nightly supply-chain and parity failures are notified only by GitHub email to the cron editor; no ticketing | Plan §3.4, §7.2 | L (proposed) | GitHub email to the cron editor only, during development (not accepted) | Issue-filing job (separate job with `issues: write`) or organizational alerting by Phase 3 | release-engineer | 2026-10-26 | Open | [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml): header comment |
| POAM-2026-10-26-08 | SC-13, SR-3, CM-7 | rustls 0.23 `fips` feature also builds the non-FIPS AWS-LC (`aws-lc-sys`): linked-but-unused code outside the FIPS boundary | Plan orchestrator decision (Open question 2, option (a), approved by the user 2026-10-26) | M (proposed) | Allowed only through `aws-lc-rs` (cargo-deny `wrappers`), Linux only, never on wasm32 | Phase 2 `tak-crypto`: startup FIPS assertion and provider test; remove the exception when rustls offers a FIPS-only feature set; review at each rustls minor upgrade | Phase 2 `tak-crypto` owner (rust-coder, crypto-tls-reviewer) | 2026-10-26 | Open | [deny.toml](../../deny.toml) line 93 (licence comment), lines 135–136 (`# POAM:` marker and the `aws-lc-sys` wrapper entry); [scripts/native-allowlist.txt](../../scripts/native-allowlist.txt): header and the `aws-lc-sys` row; [NOTICE](../../NOTICE) line 35; [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| POAM-2026-10-26-09 | SA-22, RA-5, SI-2 | Two unmaintained transitive dependencies of `yew` 0.23 are advisory ignores (an open POA&M, accepted for development by the system owner 2026-10-27; **not** risk-accepted by the authorizing official): `proc-macro-error` 1.0.4 (RUSTSEC-2024-0370) and `bincode` 1.3.3 (RUSTSEC-2025-0141). `make deny` and `make audit` pass only because of them | Raised by WU3 (release-engineer) during the build run; NIST-003; accepted for development by the system owner (John Edward Willman V) on 2026-10-27, pending AO review (see Decision below) | L (proposed) | `proc-macro-error` runs only inside rustc at build time; `bincode` is reached only through `gloo-worker`, whose API TAK-RS does not use. Ignores are identical in `deny.toml` and `.cargo/audit.toml` | Remove each ignore when `yew-macro` drops `proc-macro-error` and when `gloo`/`yew` drop `bincode` 1.x or make `gloo-worker` optional; review at each `yew` upgrade; the Phase 4 frontend plan records "no `gloo-worker` use" as a constraint | release-engineer | 2026-10-26 | Open | [deny.toml](../../deny.toml) lines 49–52 (`# POAM:` marker), lines 54–55 (ignores), lines 200–201 (`allow-build-scripts` comments; were 194–195 before Phase 1a WU2); [.cargo/audit.toml](../../.cargo/audit.toml) lines 31 and 39 (`# POAM:` markers), lines 34 and 42 (ignores); [fuzz/deny.toml](../../fuzz/deny.toml): the same marker, ignores and `allow-build-scripts` comments, mirrored for the fuzz workspace; [ASD_STIG_CHECKLIST.md](ASD_STIG_CHECKLIST.md) APSC-DV-003240 |
| POAM-2026-10-26-10 | SI-2, SI-2(2), RA-5 | Dependabot alerts and Dependabot security updates are repository settings that are not tracked or verified in the repository. On 2026-10-26 alerts were on, but automated security updates were **paused** | NIST-007 | M (proposed) | `dependabot.yml` version updates (weekly, security group); nightly `rust-supply-chain.yml` scan catches new advisories against an unchanged lockfile | Repository administrator un-pauses security updates and records both settings at each phase gate; closing evidence below | Repository administrator | 2026-10-26 | Open | [.github/dependabot.yml](../../../.github/dependabot.yml): `# POAM:` marker above the `cargo-security` group (added in fix round 3, NIST-020; outside the scan of `check-poam-ids.sh`, which reads `rust/` and `.github/workflows/rust-*.yml`); referenced by [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) SI-2, SI-2(2) and RA-5 |
| POAM-2026-10-27-01 | SR-3, CM-7, SA-11(8) | Development-only native-code exception: the fuzz workspace builds LLVM libFuzzer (C++) through `libfuzzer-sys` 0.4.13 and `cc`, with a pinned nightly toolchain, against the pure-Rust rule | Phase 1a plan §1.8, §7.2; decision Q2, approved by the user 2026-10-27; NIST-002 | L (proposed) | Excluded `rust/fuzz/` workspace (`exclude = ["fuzz"]`) with its own `Cargo.lock`, `deny.toml` and native allow-list (`libfuzzer-sys` only; NCSA licence allowed for it only); `scripts/check-fuzz-policy.sh` in `make policy` keeps `libfuzzer-sys` and `cc` out of the product graph; never shipped, never in the SBOM; `rust-fuzz.yml` never a required check, read-only token, no caches, no secrets; toolchain `nightly-2026-10-23` and `cargo-fuzz` 0.13.2 pinned | Remove when a maintained pure-Rust coverage-guided fuzzer is adopted; review at each `libfuzzer-sys` upgrade and nightly bump | release-engineer | 2026-10-27 | Open (user decision, not Risk Accepted) | [fuzz/deny.toml](../../fuzz/deny.toml) line 9 (`# POAM:` marker in the header `FUZZ-DELTA` block; the `libfuzzer-sys` NCSA licence exception at line 106, the `cc` wrapper entry at line 154 and the `allow-build-scripts` entry at line 213 cite "plan Q2", not the ID); [fuzz/native-allowlist.txt](../../fuzz/native-allowlist.txt) line 19 (`# POAM:` marker above the `libfuzzer-sys` row, line 20); [fuzz/Cargo.toml](../../fuzz/Cargo.toml) line 8 (header reference `(POAM: POAM-2026-10-27-01)`); [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) line 8 (`# POAM:` header marker); referenced by [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) SA-11(8), CM-7, SR-3 and [ATO_EVIDENCE.md](ATO_EVIDENCE.md) |
| POAM-2026-10-27-02 | SC-5, SI-10 | `tak-cot` limits are per element and per document, not per process: one conversion can hold about 23 times its input in memory, and nothing caps the conversions in flight | SEC-002; NIST-010; Phase 1a parity map D-20 | L (proposed; no network input until Phase 3, re-rate in the Phase 3 plan) | Per-message bounds (8 MiB input, 100 000 nodes, 256 attributes per element); `merge_xml_detail` drops its intermediate copies early; `Limits` rustdoc (`# Memory`) records the amplification; the Phase 1a crates have no network input | Phase 3: a document-wide allocation budget (attribute tokenising, `push_attribute`, `copy_subtree`) or a bounded ingest semaphore sized for the per-message peak; a budget that refuses what the reference accepts becomes a DEV-01 sub-row needing approval | Phase 3 `tak-net` owner (rust-coder, tak-architect) | 2026-10-27 | Open | [crates/tak-cot/src/limits.rs](../../crates/tak-cot/src/limits.rs) line 18: `// POAM: POAM-2026-10-27-02` marker |
| POAM-2026-10-27-03 | SC-5 | `write_document` and `write_subtree` output is unbounded; escaping and the namespace re-declaration of each fragment can make an output larger than its input, so an event at the 8 MiB input cap can produce output a peer's 8 MiB splitter refuses | NIST-008, NIST-010; Phase 1a QA §6 item 3 | L (proposed) | Input bounded at 8 MiB; the X2P `xmlDetail` output is capped (`write_subtree_bounded`); no network output in Phase 1a | Phase 3: the framing design bounds what a connection writes per event | Phase 3 `tak-net` owner (rust-coder, tak-architect) | 2026-10-27 | Open | [crates/tak-cot/src/xml/serialize.rs](../../crates/tak-cot/src/xml/serialize.rs) line 45: `// POAM: POAM-2026-10-27-03` marker (the run-local "(NIST-008)" citation is removed) |

## Entry details

### POAM-2026-10-26-01

- **Weakness:** The CycloneDX SBOM (`make sbom`, CI job `sbom`) is
  best-effort. The job has `continue-on-error: true` and is not part of
  `rust-ci-gate`. The SBOM is not signed.
- **Why it is tolerable now (not accepted):** Phase 0 ships no binary.
- **Progress (2026-10-27):** the `sbom` job ran for the first time in PR 7
  CI ([rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255)) and uploaded `rust-sbom` (2 `.cdx.json` files,
  13544 bytes). It is still non-blocking and unsigned, so the entry stays
  Open.
- **Milestones:** make the job blocking when the first shipped binary exists
  (Phase 3); sign the SBOM with the release artifacts (Phase 9).

### POAM-2026-10-26-02

- **Weakness:** The Java reference build in `src/` downloads Maven Central and
  Gradle Plugin Portal artifacts and runs the Gradle Node plugin
  (`download = true`) and npm with no dependency-verification metadata. `src/`
  has no `gradle/verification-metadata.xml`, and generating one ourselves
  would be trust-on-first-use.
- **Mitigations:** separate workflow that never runs on `src/**` or ordinary
  Rust PRs; read-only token, not passed to the build container; no secrets;
  ephemeral GitHub-hosted runner; Gradle wrapper jar validated and
  distribution SHA-256 enforced in a throwaway clone; images pinned by digest;
  outputs used only as test fixtures and never shipped.
- **Gradle cache (SEC-005 residual):** the parity cache now holds only
  `caches/modules-2` and `caches/jars-*`. `init.d`, `gradle.properties` and
  `wrapper/dists` are deleted after restore, so executable Gradle state is no
  longer cached and the distribution SHA-256 is checked on every run. The
  cached modules and jars themselves were downloaded without verification, so
  a cache hit is trust-on-first-use, the same risk as a fresh download. The
  cache is saved only from non-PR runs on `main`.
- **Observed (2026-10-27):** the first parity runs on PR 7 built the
  reference with these unverified downloads. Two build problems surfaced and
  were fixed in the parity tooling: the build image lacked `patch` for
  Gradle `applyGrpcPatch` (`0c56b62a`), and the `takserver-tool-ui`
  `react-scripts build` failed on ESLint "jest/globals" with no npm lockfile,
  so the reference UI build now sets `DISABLE_ESLINT_PLUGIN=true`
  (`ce9caf79`). [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328) then passed. The weakness is unchanged.
- **Phase 1a scope (2026-10-28):** the golden-corpus capture
  (`harness.sh capture-corpus`, the `rust-parity.yml` capture) runs the
  Apache-2.0 corpus driver against jars from the same unverified Java build,
  so the goldens inherit this weakness. The capture container runs without
  network, with a read-only root filesystem, all capabilities dropped and a
  non-root user, and `import-corpus` rejects links and special files and
  checks the provenance record (security review of Phase 1a). The capture
  has not run yet: GitHub Actions is disabled for the repository (WU7 NOT
  RUN).
- **Milestone:** revisit if upstream adds verification metadata.

### POAM-2026-10-26-03

- **Weakness:** The parity fixtures do not follow the container policy
  (UBI or Iron Bank bases, read-only root filesystem). Extended on
  2026-10-27 from the PostGIS image alone to every parity fixture image and
  the `takcl` root filesystem (NIST-021, WU7 REL-004):
  - **PostGIS:** the harness database is the community `postgis/postgis`
    image (15-3.3, as used by the upstream TAKCL runner), pinned by digest
    only. Docker Hub reports the tag as last pushed on 2023-08-14,
    linux/amd64 only, so it probably carries unpatched OS and PostgreSQL
    CVEs.
  - **Temurin:** `JDK_BUILD_IMAGE` (`eclipse-temurin:17-jdk-jammy`, the Java
    reference build) and `JRE_RUN_IMAGE` (`eclipse-temurin:17-jre-jammy`,
    the base of `parity/docker/Dockerfile.takcl`) are Docker Hub images on
    Ubuntu 22.04, not UBI or Iron Bank, pinned by digest only.
  - **`takcl` root filesystem:** the `takcl` service in
    [parity/compose.yaml](../../parity/compose.yaml) is not `read_only`,
    because TAKCL writes its server farm, certificates and test results
    under `/opt/tak`. Neither the `db` service nor the Gradle build
    container sets a read-only root filesystem either.
- **Why it is tolerable now (not accepted):** these are test fixtures
  outside the TOE and are never shipped. They run ephemerally; `db` and
  `takcl` share an internal network with no egress and no published ports;
  every parity container runs with `no-new-privileges` and all capabilities
  dropped (`db` adds back `CHOWN`, `DAC_OVERRIDE`, `FOWNER`, `SETGID`,
  `SETUID` for the postgres entrypoint). No one has accepted the deviation.
- **Milestones (proposed):** consider Iron Bank or UBI images for the JDK,
  JRE and PostGIS fixtures once the STIG and Iron Bank baseline is set
  ([ROADMAP.md](../../ROADMAP.md) §6 Q5); try a read-only `takcl` root
  filesystem with a writable volume or tmpfs for `/opt/tak`.

### POAM-2026-10-26-04

- **Weakness:** No branch ruleset on `main` requires the `rust-ci-gate` check
  from the GitHub Actions app, an approving review, or code-owner review of
  `.github/**` and `rust/scripts/**`. There is no `CODEOWNERS` file. The
  repository Actions policy is `allowed_actions: all` with
  `sha_pinning_required: false` (seen 2026-10-26), so a workflow outside
  `rust-*.yml`, which `check-workflows` does not lint, can use any action at
  any ref (CM-5, CM-3).
- **Interim:** `check-workflows` rejects a spoofed `rust-ci-gate` job name in
  other workflows. Fix round 2 moved it from line matching to parsed YAML
  (SEC-004, QA-006; verified at QA Round 2). Fix round 3 extends it to
  reject `checks: write`, `statuses: write` and `write-all` in every
  workflow file, including dot-prefixed ones (SEC-010); not QA-verified.
  Only a ruleset bound to
  `rust-ci.yml` closes check-name spoofing.
- **Ruleset content:** `rust-ci-gate` is the only required check. The
  `annotations-strict` job and `rust-parity.yml` must **not** be marked
  required.
- **Milestone:** the repository administrator configures the ruleset after the
  first green `rust-ci-gate` run, and where the plan allows also a "require
  workflow" rule pinned to `.github/workflows/rust-ci.yml`. Add `CODEOWNERS`
  once team names exist.
- **Progress (2026-10-27):** the first green `rust-ci-gate` run happened
  ([rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255), PR 7 head `ce9caf79`), so the milestone's
  trigger is met. The ruleset is still not configured: PR 7 was merged to
  `main` as `9f754245` by the user's decision with all checks green, but
  nothing enforced it. The entry stays Open.
- **Closing evidence:** `gh api repos/192d-Wing/Server/rulesets` (or the
  branch rules endpoint) shows `rust-ci-gate` as the only required status
  check; `gh api repos/192d-Wing/Server/actions/permissions` shows
  `allowed_actions` restricted and `sha_pinning_required: true`.

### POAM-2026-10-26-05

- **Weakness:** TAK-RS is Apache-2.0 and is developed with reference to the
  GPLv3 Java TAK Server. No legal review has been done. Open points:
  1. whether TAK-RS is a derivative work under the provenance control;
  2. reuse of GPLv3-distributed schema files (`.proto` for the `tak-proto`
     `build.rs`, XSD, OpenAPI, Flyway SQL; ROADMAP D3 "copied SQL") as build
     inputs or as copies;
  3. the effect of the repository's `INTENT.md` (portions written by federal
     employees are public domain in the U.S.);
  4. the NOTICE copyright holder;
  5. OpenSSL/SSLeay attribution in binary distributions.
- **Interim controls:** the provenance rule in
  [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance; schemas consumed in
  place from `src/`; no file copied from `src/` into `rust/` while this entry
  is open; reviewer checklist item (MAJOR, or BLOCKER for copied files).
- **Scope widened in Phase 1a (decision Q9, 2026-10-27):** the user approved
  merging Phase 1a without waiting for the legal opinion and adding three
  points to this entry (Phase 1a plan §7.2 a–c):
  1. generated prost code, compiled into binaries from the GPLv3 `.proto`
     schemas that the `tak-proto` build script reads in place from
     `src/takserver-protobuf/`;
  2. corpus goldens generated by the GPLv3 reference and committed under
     `rust/parity/corpus/golden/` (none exist yet; WU7 NOT RUN);
  3. the Apache-2.0 Java corpus driver
     ([parity/corpus/driver/CorpusDriver.java](../../parity/corpus/driver/CorpusDriver.java)),
     linked at capture time against the GPLv3 jars of the reference build.

  The TAK protocol schemas may also be published by TAK Product Center under
  other terms; counsel is to check this (unverified).
- **Milestones:** Phase 1a merges without the opinion (Q9). The opinion is
  due at the latest before the first release (Phase 3). Does not block Phase
  0 or Phase 1a.

### POAM-2026-10-26-06

- **Weakness:** TAKCL starts and manages the server under test itself (local
  processes, log scraping, Ignite for file-auth changes). It cannot target an
  externally started server. The Rust target depends on a planned `java`
  launcher shim, and the Ignite file-auth path has no design yet.
- **Interim:** `harness.sh test <id> --target rust` exits 2 (fail closed).
- **Progress (2026-10-27):** the Java side of the harness works in CI:
  TAKCL `StartupTests` passed against the Java reference in [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328).
  The Rust target is still not possible; the entry stays Open.
- **Milestone:** the Phase 3 plan run designs the Rust target before the Phase 3
  exit tests (InputTests, SimpleTests, PointToPointTests, SubscriptionTests).

### POAM-2026-10-26-07

- **Weakness:** Scheduled workflow failures notify only the user who last
  edited the cron schedule, by GitHub email. There is no ticket or alert.
- **Milestone:** an issue-filing job (a separate job with `issues: write`) or
  an organizational alerting integration by Phase 3. Until then, nightly
  results are listed in [TEST_EVIDENCE.md](TEST_EVIDENCE.md) at each phase
  gate.

### POAM-2026-10-26-08

- **Weakness:** The rustls 0.23 `fips` feature enables `aws_lc_rs`, which
  enables `aws-lc-rs/aws-lc-sys`. The non-FIPS AWS-LC library is therefore
  compiled and linked next to `aws-lc-fips-sys`, though not selected when
  `fips` is on. It is code outside the FIPS boundary (CM-7) and a second
  native library (SR-3).
- **Mitigations:** `aws-lc-sys` reachable only through `aws-lc-rs` (cargo-deny
  `wrappers`); allow-listed for Linux x86_64 and aarch64 only, never for
  wasm32. No Phase 0 crate depends on it.
- **Milestones:** Phase 2 `tak-crypto` asserts `aws_lc_rs::try_fips_mode()` at
  startup and tests that the rustls provider in use is the FIPS one; remove
  the exception when rustls offers a FIPS-only feature set; review at each
  rustls minor upgrade.
- **See also:** [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues for
  Phase 2.

### POAM-2026-10-26-09

- **Weakness:** `unmaintained = "all"` (SA-22) reports two crates that come in
  transitively through `yew` 0.23 and cannot be removed with `yew` feature
  flags:
  - `proc-macro-error` 1.0.4 (RUSTSEC-2024-0370), a build-time proc-macro
    dependency of `yew-macro` 0.23.0;
  - `bincode` 1.3.3 (RUSTSEC-2025-0141), through `yew` 0.23.0 → `gloo` 0.11.0
    → `gloo-worker` 0.5.0.
- **Why the exposure is low (developer's assessment):** `proc-macro-error`
  runs only inside rustc and puts no code in any shipped artifact. TAK-RS uses
  no `gloo-worker` API, so no attacker-controlled data reaches `bincode`.
- **Why it matters (NIST-003):** the two ignores are the only reason
  `make deny` and `make audit` pass, so `rust-ci-gate` would go green on an
  exception that no authorizing official has accepted. `bincode` 1.3.3 is a
  runtime library on the `yew` → `gloo` → `gloo-worker` path and will compile
  into the Phase 4 wasm bundle; `proc-macro-error` is build-time only. ASD STIG APSC-DV-003240
  (supported components) is affected; see
  [ASD_STIG_CHECKLIST.md](ASD_STIG_CHECKLIST.md).
- **Origin:** this entry was not in plan §7.2. It was raised by the
  release-engineer while writing `deny.toml` and `.cargo/audit.toml` (WU3) in
  the same build run, and recorded here so that every referenced ID has an
  entry. Fix round 1 reworded both files to "exception (POAM-2026-10-26-09,
  pending acceptance)".
- **Decision:** accepted for development by the system owner on 2026-10-27.
  John Edward Willman V (project owner and developer) stated, verbatim:
  "I accept POAM-2026-10-26-09 (proc-macro-error and bincode 1.x via yew 0.23) as an open POA&M for development, pending AO review."
  This is a developer and system-owner acceptance for development use
  only. It is **not** an authorizing-official risk acceptance and does not
  make the entry "Risk Accepted". The status stays **Open** pending AO
  review. It raises no control status: RA-5, SA-15(7), SI-2 and SA-22 stay
  as recorded in [NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md), and the
  `make deny` and `make audit` passes with the ignores are still not
  evidence.
- **Milestones:** remove each ignore when upstream drops the crate (see the
  register); review at each `yew` upgrade; the Phase 4 frontend plan records
  "no `gloo-worker` use" as a constraint.

### POAM-2026-10-26-10

- **Weakness:** Dependabot alerts and Dependabot security updates are
  repository settings, not files. `.github/dependabot.yml` configures version
  updates only, so nothing in the repository shows or enforces that security
  updates are on. Read on 2026-10-26:
  - `gh api repos/192d-Wing/Server/vulnerability-alerts`: HTTP 204 (alerts
    enabled);
  - `gh api repos/192d-Wing/Server/automated-security-fixes`:
    `{"enabled":true,"paused":true}`, so security-update PRs are **not**
    being opened.
- **Interim:** weekly Dependabot version updates (with a security group) and
  the nightly `rust-supply-chain.yml` scan, which fails on a new advisory
  against the unchanged lockfile (once it runs; POAM-2026-10-26-07 covers its
  weak notification).
- **Milestones (proposed):** the repository administrator un-pauses automated
  security updates before the first Phase 1 merge; both settings are read and recorded
  in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) at each phase gate.
- **Closing evidence:** `vulnerability-alerts` returns 204 and
  `automated-security-fixes` returns `"enabled":true,"paused":false`,
  recorded with the date. The SI-2 Dependabot part stays Hybrid after
  closure, because the settings remain with the repository administrator.

### POAM-2026-10-27-01

- **Weakness:** coverage-guided fuzzing uses `cargo-fuzz` and
  `libfuzzer-sys` 0.4.13, whose build script compiles LLVM libFuzzer (C++)
  with `cc`. This is native code outside the single permitted exception
  (aws-lc-rs FIPS) and needs a nightly toolchain.
- **Decision:** approved by the user on 2026-10-27 as Phase 1a plan open
  question Q2 ("approve all" of the recommended answers, recorded in the
  run's `decisions.md`): a development-only exception confined to the
  excluded `rust/fuzz/` workspace and tracked here. This is a user decision,
  not an authorizing-official risk acceptance; the status stays **Open**.
- **Mitigations:** the workspace is excluded from `rust/Cargo.toml` and has
  its own `Cargo.lock`, `deny.toml` and native allow-list, which names only
  `libfuzzer-sys`; NCSA is allowed for `libfuzzer-sys` only;
  `scripts/check-fuzz-policy.sh` (in `make policy`) fails if `libfuzzer-sys`
  or `cc` enters the product graph; nothing in `rust/fuzz/` is shipped or in
  the SBOM; `rust-fuzz.yml` is never a required check and runs with a
  read-only token, no caches and no secrets. Pins: `nightly-2026-10-23`
  ([fuzz/rust-toolchain.toml](../../fuzz/rust-toolchain.toml) line 15),
  `cargo-fuzz` 0.13.2 (`rust-fuzz.yml` `CARGO_FUZZ_VERSION`), `libfuzzer-sys`
  0.4.13 ([fuzz/Cargo.lock](../../fuzz/Cargo.lock) line 177).
- **Evidence:** Phase 1a QA, 2026-10-28: `make fuzz-deny` and `make policy`
  (with `check-fuzz-policy`) PASS ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3).
- **Markers:** added by the orchestrator in the WU10 follow-up and verified
  with `grep -n` on 2026-10-28: `# POAM: POAM-2026-10-27-01` at
  `fuzz/deny.toml` line 9 (one marker in the header `FUZZ-DELTA` block,
  covering the three delta entries, which cite "plan Q2"),
  `fuzz/native-allowlist.txt` line 19 (above the `libfuzzer-sys` row) and the
  `rust-fuzz.yml` header (line 8); the `fuzz/Cargo.toml` header (line 8)
  cites the ID in parentheses. `check-poam-markers` checks the `deny.toml`
  and `rust-fuzz.yml` markers; the `.txt` marker is outside its scan
  (NIST-013), and the parenthesised `Cargo.toml` citation is a reference, not
  a marker. `check-poam-ids` sees the ID in all four files. NIST-002 asked
  for a marker at each of the three `fuzz/deny.toml` delta entries; one
  header marker was placed instead.
- **Milestones:** remove the exception when a maintained pure-Rust
  coverage-guided fuzzer is adopted; review at each `libfuzzer-sys` upgrade
  and each bump of the pinned nightly.

### POAM-2026-10-27-02

- **Weakness:** the `tak-cot::Limits` bounds apply per element and per
  document, not per process (SEC-002). Measured by the security-reviewer
  (release build, M2 Max): an 8 387 375-byte XML event of 256-attribute
  elements peaks at 94.5 MB RSS on parse; the same content as one proto
  `xmlDetail` peaks at 193.6 MB through `from_tak_message`, about 23 times
  the input. Nothing caps how many conversions run at once; about 20
  concurrent senders would need about 4 GB in Phase 3.
- **Why it is tolerable now (not accepted):** the Phase 1a crates are
  libraries with no network input. Fix round 1 drops the intermediate copies
  in `merge_xml_detail` early and documents the amplification in the
  `Limits` rustdoc.
- **Marker:** `crates/tak-cot/src/limits.rs` line 18
  (`// POAM: POAM-2026-10-27-02`, ID added in the WU10 follow-up).
- **Milestone:** Phase 3 (parity map D-20) adds a document-wide allocation
  budget or a bounded ingest semaphore sized for the per-message peak. Any
  budget that refuses inputs the reference accepts becomes a DEV-01 sub-row
  for the user's approval.

### POAM-2026-10-27-03

- **Weakness:** the serializer's `write_document` and `write_subtree` have no
  output cap (NIST-008). Text escapes `&`, `<` and `>`, attributes also `"`,
  TAB, LF and CR, and each detached fragment re-declares the namespace
  prefixes it uses, so an 8 MiB X2X or P2X output can exceed 8 MiB and be
  refused by a receiving TAK-RS or Java splitter (Phase 1a QA §6 item 3; not
  measured). Only `write_subtree_bounded`, used for the X2P `xmlDetail`, is
  capped.
- **Why it is tolerable now (not accepted):** Phase 1a writes to no
  connection, and the input is bounded at 8 MiB.
- **Marker:** `crates/tak-cot/src/xml/serialize.rs` line 45
  (`// POAM: POAM-2026-10-27-03`, ID added and the run-local "(NIST-008)"
  removed in the WU10 follow-up).
- **Milestone:** the Phase 3 framing design bounds what a connection writes
  per event.

## Closed items

| ID | Control(s) | Weakness | Closed | Closing evidence |
|----|-----------|----------|--------|------------------|
| — | — | None closed | — | — |

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-28 | 0.7 | Phase 1a WU10 follow-up (pre-gate). Locations of POAM-2026-10-27-01, -02 and -03 updated from pending or ID-less to the markers the orchestrator added, verified with `grep -n`: -01 `fuzz/deny.toml` line 9, `fuzz/native-allowlist.txt` line 19, `fuzz/Cargo.toml` line 8 (parenthesised reference), `rust-fuzz.yml` line 8; -02 `limits.rs` line 18; -03 `xml/serialize.rs` line 45 ("(NIST-008)" removed). -01 records that one header marker replaces the three per-entry markers NIST-002 asked for. Conventions: `check-poam-markers` now in `make policy`; its `.txt` blind spot noted. No status changed: 13 open, 0 closed, 0 risk-accepted by the authorizing official | `20261027-180458-phase1-protocol-config` (WU10 follow-up) |
| 2026-10-28 | 0.6 | Phase 1a (WU10, pre-gate). New entries POAM-2026-10-27-01 (libFuzzer native-code exception in `rust/fuzz/`; SR-3, CM-7, SA-11(8); approved by the user 2026-10-27 as plan Q2; Open, not Risk Accepted; markers pending), -02 (per-process memory budget; SC-5, SI-10; marker `limits.rs` line 18) and -03 (unbounded serializer output; SC-5; marker `xml/serialize.rs` line 45); both markers are still without their IDs. POAM-2026-10-26-05 widened per decision Q9 (generated prost code from the in-place schemas, Java-generated goldens, the Java corpus driver), milestone updated, Locations now list the Phase 1a references. POAM-2026-10-26-02 extended to the golden-corpus capture. POAM-2026-10-26-09 `allow-build-scripts` comments now `deny.toml` lines 200–201 (were 194–195) and mirrored in `fuzz/deny.toml`. Conventions: Phase 1a line numbers, pending markers, `check-poam-markers`. 13 open, 0 closed, 0 risk-accepted by the authorizing official | `20261027-180458-phase1-protocol-config` (Phase 5, WU10) |
| 2026-10-27 | 0.5 | PR 7 CI evidence (rust-ci 40539495255, rust-parity 40539495328): progress notes on POAM-2026-10-26-01 (first `rust-sbom` upload), -02 (parity build ran with the unverified downloads; `patch` and ESLint fixes), -04 (first green `rust-ci-gate`, ruleset still absent, PR 7 merged without enforcement) and -06 (Java `StartupTests` PASS; Rust target still impossible). No status changed: 10 open, 0 closed, 0 risk-accepted by the authorizing official; POAM-09 unchanged (accepted for development, pending AO review). No new entry: hadolint's absence is recorded as a gap in the mapping and test evidence | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.4 | POAM-2026-10-26-09: the user's decision recorded. On 2026-10-27 the system owner (John Edward Willman V) accepted it for development, verbatim: "I accept POAM-2026-10-26-09 (proc-macro-error and bincode 1.x via yew 0.23) as an open POA&M for development, pending AO review." Register row (Weakness, Source), Weakness detail and Decision bullet updated; status stays Open pending AO review; not Risk Accepted; no control status raised. Locations re-verified with `grep -n` after the 2026-10-27 `deny.toml` and `.cargo/audit.toml` comment edits: POAM-09 `deny.toml` 49–52 (marker), 54–55 (ignores), 194–195 (`allow-build-scripts`), `.cargo/audit.toml` 31 and 39 (markers), 34 and 42 (ignores); POAM-08 `deny.toml` 93, 135–136. Header status: 10 open, 0 closed, 0 risk-accepted by the authorizing official | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs). POAM-2026-10-26-03 extended to every parity fixture image (non-UBI `eclipse-temurin` JDK and JRE, community PostGIS) and the `takcl` container's non-read-only root filesystem; CM-6 added; deviation not accepted (NIST-021). POAM-2026-10-26-10 Locations: `.github/dependabot.yml` marker (NIST-020). POAM-04: QA-006 verified at QA Round 2. POAM-2026-10-26-09 unchanged: Open, not accepted, the user decides at the gate. 10 open, 0 closed, 0 accepted | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | Phase 5 and WU10 (pre-gate). New POAM-2026-10-26-10 (Dependabot alerts and security-update settings; security updates found paused; NIST-007). POAM-09: wording "accepted" removed; open exception, acceptance pending the user's decision at the gate (no acceptance recorded); gate dependency and APSC-DV-003240 noted (NIST-003). POAM-02: SEC-005 residual (cached Gradle modules trust-on-first-use). POAM-03: tag age and architecture (WU7 REL-001). POAM-04: CM-5 added; Actions policy (WU5 REL-008). Locations verified with `grep -n` after fix round 1 for all ten entries. Conventions: `POAM:` marks an open gap, not an accepted one (NIST-015) | `20261026-210255-phase0-foundations` (Phase 5, WU10) |
| 2026-10-26 | 0.1 | Initial register: POAM-2026-10-26-01 to -07 from plan §7.2, and -08 (`aws-lc-sys`) from the orchestrator decision approved by the user 2026-10-26; POAM-2026-10-26-09 (two unmaintained `yew` 0.23 transitive crates) added because WU3's `deny.toml` and `.cargo/audit.toml` already cite it, marked not yet reviewed at the gate. Risk ratings proposed; most locations pending the build run | `20261026-210255-phase0-foundations` (WU6) |
