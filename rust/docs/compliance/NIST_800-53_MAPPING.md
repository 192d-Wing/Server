# NIST SP 800-53 Rev 5 Security Control Mapping

**Document Version:** 0.5
**Date:** 2026-10-27
**TAK-RS Version:** 0.0.1 (Phase 0, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 0 build run. 12 of 42 detailed blocks are Partially Implemented from local evidence (QA Rounds 1 and 2), now also backed by the PR 7 CI run (2026-10-27); none is Implemented. No status changed with the CI run (rule 2 still needs a nightly run).
**Run:** `20261026-210255-phase0-foundations`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** (OSCAL `NIST_SP-800-53_rev5_catalog.json`, `usnistgov/oscal-content`) |
| Baselines | NIST SP 800-53B, Release 5.2.0 (OSCAL `NIST_SP-800-53_rev5_{LOW,MODERATE,HIGH}-baseline_profile.json`; privacy baseline `NIST_SP-800-53_rev5_PRIVACY-baseline_profile.json`) |
| Target baseline | **Moderate** (assumed, not confirmed; [ROADMAP.md](../../ROADMAP.md) §6 Q5) |
| Overlay | CNSSI 1253 (security category and overlays TBD) |
| STIG | DISA Application Security and Development (ASD) STIG, release TBD; see [ASD_STIG_CHECKLIST.md](ASD_STIG_CHECKLIST.md) |
| Catalog file hashes | SHA-256 of the catalog and MODERATE profile files: [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory (`usnistgov/oscal-content` tag `v1.6.0`) |

Control titles and 800-53B baseline membership in this document were checked
against the Release 5.2.0 OSCAL catalog and the LOW, MODERATE, HIGH and
PRIVACY profiles on 2026-10-26 (`usnistgov/oscal-content` tag `v1.6.0`;
the file hashes are in [ATO_EVIDENCE.md](ATO_EVIDENCE.md)). Controls are cited
in Rev 5 form (`CM-2(2)`),
which maps mechanically to OSCAL IDs (`cm-2.2`).

### Scope

Phase 0 covers the **development environment only**: the Cargo workspace,
supply-chain policy, CI, compliance documents and the parity harness. There is
no TAK-RS runtime (TOE) yet. Every Phase 0 entry carries the scope note
"development environment / CI; no TOE runtime yet".

### Vocabulary

**Implementation status** (aligned with OSCAL `implementation-status`):

- **Implemented**: the control is in place, with code or configuration and
  passing, recorded evidence (a PASS in [TEST_EVIDENCE.md](TEST_EVIDENCE.md)
  or a recorded CI run). A NOT RUN gate is never evidence.
- **Partially Implemented**: part of the control is in place, with passing,
  recorded evidence for that part; the gap is named.
- **Planned**: not yet in place; the phase is named.
- **Alternative Implementation**: met by a different mechanism, which is
  described.
- **Not Applicable**: does not apply to the software; a reason is always given.
- **Not Applicable (component)**: the control has no function in the TAK-RS
  software component. At system level it is **Inherited** from the provider
  in the Origination column ("TBD (ATO boundary)" until the boundary is set).
  Such a row is not a claim that the system meets the control.
- **Risk Accepted (POAM-…)**: overlay on any status above, used only after the
  authorizing official has accepted the entry in [POAM.md](POAM.md). An entry
  that is not accepted is cited as "open POA&M (POAM-…)". A user or
  developer decision is recorded in POAM.md but is not "Risk Accepted".

**Control origination** (separate column):

- **System-specific**: provided by TAK-RS (code, build, CI or developer
  process).
- **Hybrid**: shared between TAK-RS and the hosting or authorizing
  organization.
- **Inherited**: provided by another party; the provider is named, or given as
  "TBD (ATO boundary)".

Every Inherited or Not Applicable entry gives a reason.

### Status at this revision

This revision records the Phase 0 build run **before its gate**. The local
gates ran on macOS on 2026-10-26: QA Round 1 after fix round 1, and QA
Round 2 after fix round 2 (both in [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3).
On 2026-10-27 the first GitHub Actions runs were recorded for PR 7 (head
`ce9caf79`, merged to `main` as `9f754245`): `rust-ci` run 40539495255, with
every job green and `ci-gate: PASS (rust=true)`, and `rust-parity` run
40539495328, with TAKCL `StartupTests` passing against the Java reference
(same section, "PR 7 CI" record). There is still no scheduled run, no
Dependabot run and no branch ruleset. The gate verdict is in the run's
`gate.md`; the post-gate part of WU10 (rule 4) has not been recorded here. On 2026-10-27 the system owner accepted
POAM-2026-10-26-09 for development, pending AO review
([POAM.md](POAM.md)); it stays Open and raises no status (rule 3).

Rules applied at this revision:

1. A status is raised only when a recorded local PASS exercises the mechanism
   the block claims, and never above the block's "Phase 0 target".
2. A claim that is true only when GitHub runs it (a CI job, the nightly
   schedule, Dependabot, a ruleset) stays **Planned** until a green
   `rust-ci-gate` run and a nightly run are recorded.
3. A gate that passes only because of an open POA&M exception
   (POAM-2026-10-26-09) does not raise RA-5, SA-15(7), SI-2 or SA-22.
   Carve-out for SA-22: its evidence is **detection evidence only**, the QA
   Round 2 negative gate in which `cargo deny` and `cargo audit` fail on
   RUSTSEC-2024-0370 and RUSTSEC-2025-0141 once the two ignores are removed.
   The pass with the ignores is not evidence.
4. Evidence that needs the gate (plan accepted, `gate.md`) waits for the
   post-gate part of WU10: SA-3 and SA-11(2) stay Planned.
5. QA Round 2 re-ran every Round 1 gate after fix round 2 and verified its
   fixes (QA-006, SEC-004, SEC-008, SEC-009, NIST-013, NIST-014). Fix round
   3 changes `check-annotations.sh` rule (d), `check-src-coupling.sh` rule
   (r), `check-workflows.rb`, the `check-native-deps.sh` annotation and
   `.github/dependabot.yml` while this revision is written (its record is the
   run's `fixes-round-3.md`). No QA re-verification of fix round 3 is
   recorded at this revision, so its results are the release-engineer's own
   runs, not QA evidence, and raise no status.
   Line numbers are given only for files outside fix rounds 2 and 3.

How the rules apply to the PR 7 CI run (2026-10-27): rule 2 has two
conditions. The first, a green `rust-ci-gate` run, is now met (run
40539495255). The second, a recorded nightly run, is not, so no claim that
is true only when GitHub runs it is raised: CA-7, CM-2(2), CM-3, CM-3(2),
RA-5(2) and SI-2(2) stay Planned. Rule 3 still holds RA-5, SA-15(7) and SI-2
(the CI advisory passes also rely on the two POAM-2026-10-26-09 ignores).
Rule 4 still holds SA-3 and SA-11(2). The CI run is added as evidence to the
blocks it exercises; it raises no status. The PR 7 CI run also covers fix
round 3: its `policy` job ran the 331 self-tests on the merged head, but that
is a CI self-test result, not a QA re-verification of the fix round 3
findings.

## Executive Summary

TAK-RS is a pure-Rust and Yew rewrite of the Java TAK Server. This document
maps its NIST SP 800-53 Rev 5 (Release 5.2.0) controls to implementation and
evidence. It covers all 20 Rev 5 control families. Organizational and physical
families are marked Inherited or Not Applicable with a reason rather than
omitted.

Phase 0 addresses only developer-process controls (CM, SA, SR, RA, SI, CA)
that the workspace and CI satisfy. There are 42 detailed control blocks.
Status at this revision:

| Status (this revision) | Controls | Count |
|------------------------|----------|-------|
| Implemented | — | 0 |
| Partially Implemented | CA-5, CM-2, CM-6, CM-10, SA-10, SA-11, SA-11(1), SA-15, SA-22, SR-3, SR-4, SR-11 | 12 |
| Planned | CA-2, CA-7, CM-2(2), CM-3, CM-3(2), CM-5, CM-7, CM-8, CM-14, PL-8, RA-5, RA-5(2), SA-3, SA-8, SA-11(2), SA-11(8), SA-15(7), SA-15(13), SA-17, SA-24, SC-13, SI-2, SI-2(2), SI-2(7), SI-7, SI-7(12), SI-7(15), SR-2, SR-5, SR-6 | 30 |
| **Total** | | **42** |

Phase 0 targets (the ceiling for this run; plan §7.1 as amended by the
nist-800-53-reviewer findings NIST-003, NIST-004, NIST-006 and NIST-009):

| Phase 0 target | Controls | Count |
|----------------|----------|-------|
| Implemented | RA-5(2) (status stays Planned until a recorded CI and nightly run) | 1 |
| Partially Implemented | CA-5, CA-7, CM-2, CM-2(2), CM-3, CM-3(2), CM-6, CM-10, RA-5, SA-3, SA-10, SA-11, SA-11(1), SA-11(2), SA-15, SA-15(7), SA-22 (two open exceptions), SI-2, SI-2(2), SR-3, SR-4, SR-11 | 22 |
| Planned | CA-2, CM-5, CM-7, CM-8, CM-14, PL-8, SA-8, SA-11(8), SA-15(13), SA-17, SA-24, SC-13, SI-2(7), SI-7, SI-7(12), SI-7(15), SR-2, SR-5, SR-6 | 19 |
| **Total** | | **42** |

Not claimed in Phase 0: AC-6 (CI-token least privilege is recorded as CM-6,
SA-15 and SR-3 evidence, not as a TOE AC-6 implementation) and CA-2 as an
independent assessment (agent reviews are SA-11 evidence).

Every status above is a **component** status for the TAK-RS development
environment. It is not a system-level determination; that is made for the
system at the ATO boundary.

## Family Coverage

| Family | Origination | Phase 0 status | Phase 0 target | Reason / planned phase |
|--------|-------------|----------------|----------------|------------------------|
| AC Access Control | System-specific | Planned | Planned | Phases 2–4 (`tak-auth`, route→role table, group isolation) |
| AT Awareness and Training | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Organizational training; no product function (the DoD banner is AC-8) |
| AU Audit and Accountability | System-specific | Planned | Planned | Phase 2 `tak-audit` |
| CA Assessment, Authorization, and Monitoring | Hybrid | Partially Implemented (CA-5) | Partially Implemented (CA-5, CA-7) | Assessment and authorization are organizational; developer evidence in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) |
| CM Configuration Management | Hybrid | Partially Implemented (CM-2, CM-6, CM-10) | Partially Implemented | CM blocks below |
| CP Contingency Planning | Hybrid | Planned | Planned | Plans and alternate sites Inherited; software support for backup and restore (CP-9, CP-10) documented in Phase 9 |
| IA Identification and Authentication | System-specific | Planned | Planned | Phase 2 (X.509, file users, LDAP, OAuth) |
| IR Incident Response | Hybrid | Planned | Planned | Process Inherited (organization); the software supplies audit data (AU) for IR-4 and IR-5 |
| MA Maintenance | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Hardware and facility maintenance by the hosting organization; MA-4 remote administration sessions are Planned through AC, IA and SC in Phase 4 |
| MP Media Protection | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Physical media handled by the hosting organization; data at rest is SC-28 |
| PE Physical and Environmental Protection | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Facility controls of the hosting organization |
| PL Planning | Hybrid | Planned | Planned | SSP is organizational; PL-8 architecture in Phase 2 |
| PM Program Management | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Organization-wide program controls; not allocated to 800-53B baselines |
| PS Personnel Security | Inherited (system level; provider TBD) | Not Applicable (component) | Not Applicable (component) | Personnel screening by the organization |
| PT Personally Identifiable Information Processing and Transparency | Hybrid | Planned | Planned | **Applicable:** TAK processes PII (user identities, callsigns, position tracks). Authority and purpose (PT-2, PT-3) are organizational; software support (retention, minimisation) in Phases 4–7 |
| RA Risk Assessment | Hybrid | Planned | Partially Implemented (RA-5, RA-5(2)) | RA blocks below |
| SA System and Services Acquisition | System-specific | Partially Implemented (SA-10, SA-11, SA-11(1), SA-15, SA-22) | Partially Implemented | Developer-side controls; SA blocks below |
| SC System and Communications Protection | System-specific | Planned | Planned | Phases 2–3 (SC-8, SC-13, SC-23, SC-28) |
| SI System and Information Integrity | Hybrid | Planned | Partially Implemented (SI-2, SI-2(2)) | SI-10 from Phase 1 (`tak-cot`) |
| SR Supply Chain Risk Management | Hybrid | Partially Implemented (SR-3, SR-4, SR-11) | Partially Implemented | SR blocks below |

Coverage: 20 of 20 Rev 5 families have a section below.

### Per-control block fields

Each detailed block uses the same fields, so the document converts cleanly to
an OSCAL component definition later (see [OSCAL](#oscal)):
Control, 800-53B baseline, Origination, Status, Phase 0 target, Scope,
Implementation, Evidence, Gaps, POA&M, Phase, Annotation locations.

Baseline column values: `L`, `M`, `H` for membership in the LOW, MODERATE and
HIGH baselines; `—` means the control is not in any 800-53B security
baseline.

"QA Round 1" evidence is the local run recorded in
[TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3 (macOS, 2026-10-26, uncommitted
working tree after fix round 1). "Present, not yet gated" means the file
exists but no recorded PASS exercises it. "NOT RUN (CI only)" means the
evidence can come only from a GitHub run that has not happened. "PR 7 CI"
evidence is the GitHub Actions run record in
[TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3 (rust-ci 40539495255 and rust-parity
40539495328, 2026-10-27).

---

## AC Access Control

Origination: System-specific. Status: Planned. No access-control function
exists in Phase 0.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| AC-2 Account Management | L M H | Hybrid | Planned | 2 | File users, LDAP, OAuth (`tak-auth`); account review is organizational |
| AC-3 Access Enforcement | L M H | System-specific | Planned | 2–4 | Group bit vectors, route→role table from the Java `intercept-url` rules |
| AC-6 Least Privilege | M H | System-specific | Planned | 4 | Not claimed in Phase 0; CI-token least privilege is CM-6, SA-15 and SR-3 evidence |
| AC-7 Unsuccessful Logon Attempts | L M H | System-specific | Planned | 2–4 | Lockout for file users and LDAP logins (`tak-auth`, Phase 2) and the admin login (Phase 4) |
| AC-8 System Use Notification | L M H | System-specific | Planned | 4 | DoD banner in `web/components` and the admin console |
| AC-10 Concurrent Session Control | H | System-specific | Planned | 4 | Admin console sessions |
| AC-12 Session Termination | M H | System-specific | Planned | 3–4 | Idle and lifetime limits for admin sessions (Phase 4) and streaming connections (Phase 3) |
| AC-17 Remote Access | L M H | Hybrid | Planned | 3–4 | TLS inputs and admin sessions (with SC-8, IA-2) |

## AT Awareness and Training

Origination: Inherited (hosting organization; provider TBD (ATO boundary)).
Status: Not Applicable to the software. These controls govern people and
training programs; TAK-RS has no training function. The DoD consent banner
is a product function and is tracked under AC-8.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| AT-1 Policy and Procedures | L M H | Inherited (system level) | Not Applicable (component) | Organizational policy |
| AT-2 Literacy Training and Awareness | L M H | Inherited (system level) | Not Applicable (component) | Organizational training program |
| AT-2(2) Insider Threat | L M H | Inherited (system level) | Not Applicable (component) | Organizational training content |
| AT-2(3) Social Engineering and Mining | M H | Inherited (system level) | Not Applicable (component) | Organizational training content |
| AT-3 Role-based Training | L M H | Inherited (system level) | Not Applicable (component) | Organizational training program; operator documentation is SA-5 (Phase 9) |
| AT-4 Training Records | L M H | Inherited (system level) | Not Applicable (component) | Organizational records |

## AU Audit and Accountability

Origination: System-specific. Status: Planned. Phase 0 has no runtime state
changes and therefore no `tak-audit` events. Developer evidence comes from CI
records (see [ATO_EVIDENCE.md](ATO_EVIDENCE.md)).

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| AU-2 Event Logging | L M H | System-specific | Planned | 2 | `tak-audit` event catalogue |
| AU-3 Content of Audit Records | L M H | System-specific | Planned | 2 | Structured records; see also SA-15(13) |
| AU-8 Time Stamps | L M H | System-specific | Planned | 2 | UTC time stamps in `tak-audit` records; time source is the host (organizational) |
| AU-9 Protection of Audit Information | L M H | System-specific | Planned | 2 | Tamper-evident audit chain |
| AU-10 Non-repudiation | H | System-specific | Planned | 2 | Audit records bound to the authenticated identity (X.509 subject or user) |
| AU-11 Audit Record Retention | L M H | Hybrid | Planned | 2 | Retention of developer evidence is also Planned; the long-term evidence store is TBD |
| AU-12 Audit Record Generation | L M H | System-specific | Planned | 2 | Generation at every security-relevant decision point |

## CA Assessment, Authorization, and Monitoring

Origination: Hybrid. Status: Partially Implemented (CA-5 only). Assessment
and authorization are performed by the organization; TAK-RS supplies
developer evidence and its POA&M register (CA-5).

### CA-2 Control Assessments

| Field | Value |
|-------|-------|
| Control | CA-2 Control Assessments |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. Agent reviews in `/tak-rs-workflow` are developer testing (SA-11), not an independent assessment. |
| Evidence | — |
| Gaps | Independent assessor and assessment plan are organizational (TBD) |
| POA&M | — |
| Phase | Assessment before ATO (Phase 9) |
| Annotation locations | — |

### CA-5 Plan of Action and Milestones

| Field | Value |
|-------|-------|
| Control | CA-5 Plan of Action and Milestones |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (developer register and its checks; the organizational POA&M process and authorizing-official decisions are outside TAK-RS) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | TAK-RS keeps its POA&M register in [POAM.md](POAM.md): weakness, controls, proposed risk, mitigation, milestones, owner role, status and code locations. Every POA&M ID referenced under `rust/` or in `.github/workflows/rust-*.yml` must have a register entry (`scripts/check-poam-ids.sh`, run by `make policy`). Every advisory ignore must carry a POA&M ID and be identical in `deny.toml` and `.cargo/audit.toml` (`scripts/check-advisory-sync.sh`) |
| Evidence | [POAM.md](POAM.md) (10 open entries, 0 closed). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make policy` PASS (`check-poam-ids`: 9 IDs referenced, 9 entries at that time); `make check-advisory-sync` PASS (2 IDs, each with a POA&M); `make policy-selftest` PASS (199 passed, 0 failed). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `policy` job `check-poam-ids: ok (10 referenced ID(s), 10 entr(ies) in POAM.md)`; `supply-chain` job `check-advisory-sync: ok (2 ignored advisory ID(s), lists identical, each with a POA&M)` |
| Gaps | Risk ratings are proposed, not confirmed by the ISSO. No entry is accepted by the authorizing official. POAM-2026-10-26-09 is accepted for development by the system owner (2026-10-27), pending AO review; it stays Open. The organizational POA&M process (PM-4) and the update frequency are Inherited, TBD (ATO boundary). Both checks ran in CI on PR 7; they are not yet a required check (POAM-2026-10-26-04) |
| POA&M | — |
| Phase | 0 |
| Annotation locations | [scripts/check-poam-ids.sh](../../scripts/check-poam-ids.sh) and [scripts/check-advisory-sync.sh](../../scripts/check-advisory-sync.sh) header comments |

### CA-7 Continuous Monitoring

| Field | Value |
|-------|-------|
| Control | CA-7 Continuous Monitoring |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Partially Implemented (developer-side only; organizational strategy Inherited/TBD) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Nightly supply-chain scans (`rust-supply-chain.yml`: cargo-audit, cargo-deny, native-dependency check) |
| Evidence | Workflow present: [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml) (nightly and manual). First scheduled run: NOT RUN (CI only). The PR 7 CI `supply-chain` job runs the same scans per change; it is not continuous monitoring |
| Gaps | No scheduled run recorded yet. Organizational continuous-monitoring strategy (PM-31) is Inherited, TBD (ATO boundary); failure notification is weak (POAM-2026-10-26-07) |
| POA&M | POAM-2026-10-26-07 |
| Phase | 0 |
| Annotation locations | — (the workflow's mapping block cites RA-5, RA-5(2), SI-2(2), SR-3, SR-4 and CM-10, not CA-7) |

## CM Configuration Management

Origination: Hybrid. Status: Partially Implemented (CM-2, CM-6, CM-10).
CM-4 (Impact Analyses, L M H) is Planned: the `/tak-rs-workflow` plan and
review records are its intended evidence, but no run has passed a gate yet.

### CM-2 Baseline Configuration

| Field | Value |
|-------|-------|
| Control | CM-2 Baseline Configuration |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (build baseline only; no deployment baseline) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Toolchain pin (`rust-toolchain.toml`, channel 1.99.1), `Cargo.lock`, SHA-pinned GitHub Actions, digest-pinned container images, `parity/pins.env`, pin inventory |
| Evidence | [rust-toolchain.toml](../../rust-toolchain.toml) (channel 1.99.1), [Cargo.lock](../../Cargo.lock), [parity/pins.env](../../parity/pins.env), SHA-pinned actions in `.github/workflows/rust-*.yml`; pin inventory in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory (each pin checked against the files with `grep`). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `rustup show active-toolchain` PASS (1.99.1 from `rust-toolchain.toml`); `cargo metadata --locked` PASS; `make policy` PASS (`check-toolchain` 1.99.1 vs 1.99, `check-workflows` ok on 3 files, including the 40-hex SHA rule); `parity/harness.sh selftest` PASS (39/0, includes the strict `pins.env` parser). QA Round 2, 2026-10-26, local (same section): the same gates PASS; `make policy` `check-workflows: ok (3 rust workflow file(s), 3 parsed)`, the parsed-YAML version with rule W1 (40-hex SHA pin); harness selftest 44/0; QA independently re-checked 8 pins against the files (toolchain, action SHAs, actionlint, `pins.env`, OSCAL, tool versions, licence files), all matching. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): every Rust job used rustc 1.99.1 from the pin; `check-toolchain` and `check-workflows: ok (3 rust workflow file(s), 3 parsed)` under Ruby 3.2.3; the cargo jobs ran `--locked`; [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328) used the three `pins.env` image digests and printed `wrapper verified: Gradle 8.10.2` |
| Gaps | No deployment baseline (no shipped binary until Phase 3; packaging Phase 9). Image digests and the Gradle pins are maintained by hand (CM-2(2)). The build baseline was built by CI once, on PR 7; no nightly build yet. At QA Round 1, `check-workflows` was line-based and could be bypassed (QA-006); the parsed-YAML version from fix round 2 was verified at QA Round 2 (QA-006 resolved). Fix round 3 changes `check-workflows.rb` again (SEC-010, SEC-011); that change is not QA-verified, though it passed its self-tests and `check-workflows` in PR 7 CI |
| POA&M | — |
| Phase | 0 (build baseline); 9 (deployment baseline) |
| Annotation locations | [rust-toolchain.toml](../../rust-toolchain.toml), [Cargo.toml](../../Cargo.toml), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments |

### CM-2(2) Automation Support for Accuracy and Currency

| Field | Value |
|-------|-------|
| Control | CM-2(2) Automation Support for Accuracy and Currency |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Dependabot for `github-actions` and `cargo` (`/rust`); `check-toolchain.sh` and `check-workflows.sh` pin checks in CI |
| Evidence | Present: [.github/dependabot.yml](../../../.github/dependabot.yml); `scripts/check-toolchain.sh` and `scripts/check-workflows.sh` pass locally (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)). The pin checks ran in CI on PR 7 (PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)): `workflow-lint` and `policy` jobs, `check-toolchain` ok and `check-workflows` ok. Dependabot runs: NOT RUN |
| Gaps | No Dependabot run and no nightly run recorded; the status stays Planned under rule 2 (the PR 7 CI run alone does not meet it). Container image digests and the Gradle pins are updated by hand. `check-workflows` was line-based and bypassable (QA-006, SEC-004); fix round 2 moved it to parsed YAML (Ruby psych), verified at QA Round 2 (QA-006 and SEC-004 resolved). Fix round 3 changes it again; not QA-verified |
| POA&M | — |
| Phase | 0 |
| Annotation locations | [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh) header comment (cites CM-2, CM-6, SA-15); no block cites CM-2(2) yet |

### CM-3 Configuration Change Control

| Field | Value |
|-------|-------|
| Control | CM-3 Configuration Change Control |
| 800-53B baseline | M H |
| Origination | Hybrid (repository administrator) |
| Status | Planned |
| Phase 0 target | Partially Implemented (ruleset not configured or verified; POAM-2026-10-26-04) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Pull-request workflow, `rust-ci-gate` required check, `/tak-rs-workflow` run records, `CHANGELOG.md`, [CONTRIBUTING.md](../../CONTRIBUTING.md) |
| Evidence | Present: [CONTRIBUTING.md](../../CONTRIBUTING.md), [CHANGELOG.md](../../CHANGELOG.md), [rust-ci.yml](../../../.github/workflows/rust-ci.yml). Local helpers pass (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)): `ci-changes.sh` allow-list and `ci-gate.sh` selftests within `make policy-selftest` (199/0); detector linear on every upstream merge (12335 paths in 0.06 s). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `changes` gave `rust=true`; `rust-ci-gate` printed `ci-gate: PASS (rust=true)`. Earlier PR 7 heads `77beace3` and `0c56b62a` failed the `policy` job and `rust-ci-gate` went red (runs 40525846385, 40536002508) |
| Gaps | `rust-ci-gate` ran (green on PR 7, red on two earlier heads) but is not a required check: PR 7 was merged with all checks green by the user's decision, with nothing enforcing it. No nightly run, so the status stays Planned (rule 2). No `src/**`-only PR has tested the `rust=false` path end to end. Branch ruleset (required check, required review, code-owner review) not configured; no `CODEOWNERS`; repository Actions policy allows any action and does not require SHA pinning (POAM-2026-10-26-04). QA-001 (change detector super-linear: 16 s on 170 paths, more than 300 s on 668) was found and fixed in this run; QA-002 and QA-006 showed that the line-based `check-workflows` rules could be bypassed (QA-002 fixed; QA-006 fixed in fix round 2 by parsing the YAML, verified at QA Round 2). Since fix round 2 the detector also runs the Rust jobs for `src/takserver-protobuf/` changes (SEC-008), so 9 of the 14 upstream merges would now run them (QA Round 2 confirmed 9 of 14, each explained by a `src/takserver-protobuf/` path). QA-008 (`..` segments after a covered prefix pass `check-src-coupling` rule (r)) was open at QA Round 2; fix round 3 addresses it, not QA-verified |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 |
| Annotation locations | [scripts/ci-changes.sh](../../scripts/ci-changes.sh), [scripts/ci-gate.sh](../../scripts/ci-gate.sh), [scripts/check-src-coupling.sh](../../scripts/check-src-coupling.sh) (added in fix round 2), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments |

### CM-3(2) Testing, Validation, and Documentation of Changes

| Field | Value |
|-------|-------|
| Control | CM-3(2) Testing, Validation, and Documentation of Changes |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented (gate not yet required; POAM-2026-10-26-04) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | CI gates before merge; run records (`plan.md`, `qa-report.md`, findings, `gate.md`) |
| Evidence | Present: [rust-ci.yml](../../../.github/workflows/rust-ci.yml). First local run record: [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3. Pre-merge CI gates: PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): 13 of 13 jobs `success`, `ci-gate: PASS (rust=true)`; the two earlier failing heads were fixed before the merge |
| Gaps | `rust-ci-gate` ran on PR 7 but is not a required check, so testing before merge is not enforced (POAM-2026-10-26-04). No nightly run, so the status stays Planned (rule 2). QA-001 (super-linear change detector) fixed in this run. QA-002 (job-level `continue-on-error` and permission escalation not caught) fixed; QA-006 (quoted or spaced keys bypass the same rules) fixed in fix round 2 and verified at QA Round 2 |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 |
| Annotation locations | [rust-ci.yml](../../../.github/workflows/rust-ci.yml) header comment |

### CM-5 Access Restrictions for Change

| Field | Value |
|-------|-------|
| Control | CM-5 Access Restrictions for Change |
| 800-53B baseline | L M H |
| Origination | Hybrid (repository administrator) |
| Status | Planned |
| Phase 0 target | Planned (POAM-2026-10-26-04) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None in place. Intended: a branch ruleset on `main` (required `rust-ci-gate` from the GitHub Actions app, required approving review, code-owner review of `.github/**` and `rust/scripts/**`), a `CODEOWNERS` file, and a repository Actions policy that allows only the pinned actions and requires full-SHA pins |
| Evidence | — (repository settings not configured or verified) |
| Gaps | No ruleset, no `CODEOWNERS`; Actions policy `allowed_actions: all`, `sha_pinning_required: false` (seen 2026-10-26, release-engineer notes). The first green `rust-ci-gate` run (40539495255, 2026-10-27) has happened, so the milestone for the ruleset is due; PR 7 was merged without one |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 (after the first green `rust-ci-gate` run) |
| Annotation locations | — |

### CM-6 Configuration Settings

| Field | Value |
|-------|-------|
| Control | CM-6 Configuration Settings |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (no product configuration yet) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Workspace lints (`unsafe_code = "forbid"`, strict clippy); `.cargo/config.toml` build pins `AWS_LC_FIPS_SYS_USE_SYSTEM=0`, `GOTOOLCHAIN=local`, `GOPROXY=off` (all `force = true`); least-privilege CI token (`permissions: contents: read`) |
| Evidence | [Cargo.toml](../../Cargo.toml) `[workspace.lints.rust]` (line 32; `unsafe_code = "forbid"` at line 35) and `[workspace.lints.clippy]` (line 42), [.cargo/config.toml](../../.cargo/config.toml), top-level `permissions: contents: read` in each `.github/workflows/rust-*.yml`. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make clippy` and `make clippy-wasm` PASS (0 warnings with `-D warnings`); `make policy` PASS (`check-manifests`: 2 members with `[lints] workspace = true`; `check-unsafe`: 0 uses); `grep -L 'contents: read'` over the three workflows prints nothing, so each contains `contents: read` (grep alone does not show the setting is top-level). QA Round 2, 2026-10-26, local (same section): the same gates PASS; `make policy` `check-workflows` ok on 3 parsed files, whose rule W3 requires top-level `permissions` to be exactly `contents: read` and rule W3b allows job-level `permissions` only absent or exactly `contents: read`; the QA-006 permission mutations (quoted key, spaced key, `write-all`, added `id-token: write`) each fail. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): the runner logged `GITHUB_TOKEN Permissions` as `Contents: read`, `Metadata: read` for all 13 `rust-ci` jobs and the `rust-parity` job |
| Gaps | No product configuration settings yet (Phase 1 `tak-config`). The `.cargo/config.toml` FIPS build pins are inert until Phase 2, so no gate exercises them. The CI-token setting takes effect only in CI; it was observed in the PR 7 runs. At QA Round 1 the `check-workflows` permission rules were line-based and could be bypassed (QA-006); the parsed-YAML rules are verified at QA Round 2. Fix round 3 adds `checks: write`, `statuses: write` and `write-all` rejection in every workflow file (SEC-010); not QA-verified. The parity fixture containers do not set a read-only root filesystem and use non-UBI bases (open POA&M POAM-2026-10-26-03) |
| POA&M | POAM-2026-10-26-03 (parity fixture containers) |
| Phase | 0 (build settings); 1+ (product settings) |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml), [Cargo.toml](../../Cargo.toml), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml) header comments |

### CM-7 Least Functionality

| Field | Value |
|-------|-------|
| Control | CM-7 Least Functionality |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None in Phase 0. Pre-mapped because rustls 0.23 `fips` also builds the non-FIPS `aws-lc-sys`, which would be linked but unused code |
| Evidence | — |
| Gaps | Linked-but-unused `aws-lc-sys` from Phase 2. Yew `csr` brings all of `gloo` (81 wasm32 crates), including `gloo-worker` and `gloo-file`, which create `blob:` URLs; nothing uses them today, and the Phase 4 plan keeps `blob:` out of the CSP (WEB-005) |
| POA&M | POAM-2026-10-26-08 |
| Phase | 2 |
| Annotation locations | — (fix round 3 removed CM-7 from the [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh) header, which cites SR-3 and SA-15; NIST-019) |

### CM-8 System Component Inventory

| Field | Value |
|-------|-------|
| Control | CM-8 System Component Inventory |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (POAM-2026-10-26-01) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | CycloneDX SBOM stub (`make sbom`, CI job `sbom`, non-blocking) |
| Evidence | Present: `sbom` target in [Makefile](../../Makefile), `sbom` job in [rust-ci.yml](../../../.github/workflows/rust-ci.yml). `make sbom`: NOT RUN locally (cargo-cyclonedx not installed). PR 7 CI: the `sbom` job succeeded with cargo-cyclonedx 0.5.9 and uploaded `rust-sbom` (2 `.cdx.json` files, 13544 bytes) |
| Gaps | SBOM is best-effort (`continue-on-error`), not required, not signed; generated once, in PR 7 CI. Its content has not been reviewed against the dependency graph |
| POA&M | POAM-2026-10-26-01 |
| Phase | 3 (blocking); 9 (signed) |
| Annotation locations | — (`# POAM:` comment on the `sbom` job in [rust-ci.yml](../../../.github/workflows/rust-ci.yml)) |

### CM-10 Software Usage Restrictions

| Field | Value |
|-------|-------|
| Control | CM-10 Software Usage Restrictions |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (legal review POAM-2026-10-26-05) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Apache-2.0 workspace licence; cargo-deny licence allow-list without copyleft (denied by absence); OpenSSL exception scoped to `aws-lc-fips-sys` with NOTICE attribution; provenance rule against the GPLv3 Java reference |
| Evidence | [Cargo.toml](../../Cargo.toml) `license = "Apache-2.0"`, [LICENSE](../../LICENSE), [NOTICE](../../NOTICE), [deny.toml](../../deny.toml) `[licenses]`, [CONTRIBUTING.md](../../CONTRIBUTING.md) §Licence and §Provenance. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): member licence set is exactly `Apache-2.0` PASS; `LICENSE` is the Apache text, `NOTICE` has the OpenSSL attribution, CONTRIBUTING has §Provenance (PASS); no GPL, AGPL or MPL in the locked graph (PASS); `make deny` `licenses ok`; `make policy` `check-manifests` ok |
| Gaps | Derivative-work and schema-reuse questions not reviewed by counsel; the provenance rule has not yet been exercised by a port |
| POA&M | POAM-2026-10-26-05 |
| Phase | 0; legal opinion before the first release (Phase 3) |
| Annotation locations | [Cargo.toml](../../Cargo.toml), [deny.toml](../../deny.toml), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml) header comments |

### CM-14 Signed Components

| Field | Value |
|-------|-------|
| Control | CM-14 Signed Components |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (Phase 9) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None |
| Evidence | — |
| Gaps | No signed artifacts until release packaging |
| POA&M | — |
| Phase | 9 |
| Annotation locations | — |

## CP Contingency Planning

Origination: Hybrid. Status: Planned. Contingency plans, alternate sites and
backup operations are the hosting organization's (Inherited, provider TBD
(ATO boundary)). The software's part is backup and restore support for the
database and configuration.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| CP-9 System Backup | L M H | Hybrid | Planned | 9 | Document what to back up (Postgres, CoreConfig, keystores); operation is organizational |
| CP-10 System Recovery and Reconstitution | L M H | Hybrid | Planned | 9 | Restore procedure and fail-secure restart |

## IA Identification and Authentication

Origination: System-specific. Status: Planned.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| IA-2 Identification and Authentication (Organizational Users) | L M H | System-specific | Planned | 2 | X.509 client certificates, file users, LDAP, OAuth |
| IA-2(12) Acceptance of PIV Credentials | L M H | System-specific | Planned | 2 | PIV and CAC client certificates through the X.509 path in `tak-auth` |
| IA-5 Authenticator Management | L M H | Hybrid | Planned | 2 | bcrypt compatibility for `UserAuthenticationFile`; secrets in `Zeroizing` |
| IA-7 Cryptographic Module Authentication | L M H | System-specific | Planned | 2 | FIPS module (see [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md)) |
| IA-8 Identification and Authentication (Non-organizational Users) | L M H | Hybrid | Planned | 2, 6 | Non-organizational users and federation peers (Phase 6); which users count as non-organizational is organizational |

## IR Incident Response

Origination: Hybrid. Status: Planned. Incident handling is an organizational
process (Inherited, provider TBD (ATO boundary)). TAK-RS supplies audit data
(AU family, Phase 2) that the process consumes.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| IR-4 Incident Handling | L M H | Hybrid | Planned | 2 | Process Inherited; software supplies audit records |
| IR-5 Incident Monitoring | L M H | Hybrid | Planned | 2 | Process Inherited; software supplies audit records |

## MA Maintenance

Origination: Inherited (hosting organization; provider TBD (ATO boundary)).
Status: Not Applicable to the software, except MA-4.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| MA-1 Policy and Procedures | L M H | Inherited (system level) | Not Applicable (component) | Organizational policy |
| MA-2 Controlled Maintenance | L M H | Inherited (system level) | Not Applicable (component) | Hardware maintenance by the hosting organization |
| MA-3 Maintenance Tools | M H | Inherited (system level) | Not Applicable (component) | Maintenance tooling of the hosting organization |
| MA-3(1) Inspect Tools | M H | Inherited (system level) | Not Applicable (component) | Maintenance tooling of the hosting organization |
| MA-3(2) Inspect Media | M H | Inherited (system level) | Not Applicable (component) | Maintenance media of the hosting organization |
| MA-3(3) Prevent Unauthorized Removal | M H | Inherited (system level) | Not Applicable (component) | Facility process |
| MA-4 Nonlocal Maintenance | L M H | Hybrid | Planned | Applicable: remote administration sessions are protected by AC, IA and SC controls in Phase 4 |
| MA-5 Maintenance Personnel | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| MA-6 Timely Maintenance | M H | Inherited (system level) | Not Applicable (component) | Hardware support contracts |

## MP Media Protection

Origination: Inherited (hosting organization; provider TBD (ATO boundary)).
Status: Not Applicable to the software. These controls govern physical and
removable media. Protection of data at rest held by TAK-RS is SC-28.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| MP-1 Policy and Procedures | L M H | Inherited (system level) | Not Applicable (component) | Organizational policy |
| MP-2 Media Access | L M H | Inherited (system level) | Not Applicable (component) | Physical media handling |
| MP-3 Media Marking | M H | Inherited (system level) | Not Applicable (component) | Physical media handling; on-screen classification marking is a product function (Phase 4, `web/components`) |
| MP-4 Media Storage | M H | Inherited (system level) | Not Applicable (component) | Physical media handling |
| MP-5 Media Transport | M H | Inherited (system level) | Not Applicable (component) | Physical media handling |
| MP-6 Media Sanitization | L M H | Inherited (system level) | Not Applicable (component) | Physical media handling |
| MP-7 Media Use | L M H | Inherited (system level) | Not Applicable (component) | Physical media handling |

## PE Physical and Environmental Protection

Origination: Inherited (hosting facility; provider TBD (ATO boundary)).
Status: Not Applicable to the software. Every control in this family is a
facility control.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| PE-1 Policy and Procedures | L M H | Inherited (system level) | Not Applicable (component) | Facility policy |
| PE-2 Physical Access Authorizations | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-3 Physical Access Control | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-4 Access Control for Transmission | M H | Inherited (system level) | Not Applicable (component) | Facility cabling |
| PE-5 Access Control for Output Devices | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-6 Monitoring Physical Access | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-6(1) Intrusion Alarms and Surveillance Equipment | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-8 Visitor Access Records | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-9 Power Equipment and Cabling | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-10 Emergency Shutoff | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-11 Emergency Power | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-12 Emergency Lighting | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-13 Fire Protection | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-13(1) Detection Systems — Automatic Activation and Notification | M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-14 Environmental Controls | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-15 Water Damage Protection | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-16 Delivery and Removal | L M H | Inherited (system level) | Not Applicable (component) | Facility control |
| PE-17 Alternate Work Site | M H | Inherited (system level) | Not Applicable (component) | Organizational telework policy |

## PL Planning

Origination: Hybrid. Status: Planned. The System Security and Privacy Plan
(PL-2, L M H) is organizational (Inherited, provider TBD (ATO boundary));
these compliance documents are TAK-RS's input to it.

### PL-8 Security and Privacy Architectures

| Field | Value |
|-------|-------|
| Control | PL-8 Security and Privacy Architectures |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (Phase 2) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None yet. [ARCHITECTURE.md](../../ARCHITECTURE.md) is a short workspace map, not a security architecture |
| Evidence | — |
| Gaps | Security and privacy architecture description |
| POA&M | — |
| Phase | 2 |
| Annotation locations | — |

## PM Program Management

Origination: Inherited (organization-wide program; provider TBD (ATO
boundary)). Status: Not Applicable to the software. PM controls are deployed
organization-wide and are not allocated to the 800-53B baselines.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| PM-4 Plan of Action and Milestones Process | — | Inherited (system level) | Not Applicable (component) | Organizational process; TAK-RS supplies [POAM.md](POAM.md) as input |
| PM-5 System Inventory | — | Inherited (system level) | Not Applicable (component) | Organizational inventory; TAK-RS supplies the SBOM (CM-8) from Phase 3 |
| PM-25 Minimization of Personally Identifiable Information Used in Testing, Training, and Research | — | Inherited (system level) | Not Applicable (component) | Organizational policy; the parity harness uses only generated test users |
| PM-30 Supply Chain Risk Management Strategy | — | Inherited (system level) | Not Applicable (component) | Organizational strategy; developer controls are in SR |
| PM-31 Continuous Monitoring Strategy | — | Inherited (system level) | Not Applicable (component) | Organizational strategy; developer monitoring is CA-7 |
| All other PM controls (PM-1 to PM-3, PM-6 to PM-24, PM-26 to PM-29, PM-32) | — | Inherited (system level) | Not Applicable (component) | Organization-wide program, leadership, workforce and governance controls with no software function |

## PS Personnel Security

Origination: Inherited (organization; provider TBD (ATO boundary)).
Status: Not Applicable to the software.

| Control | Baseline | Origination | Status | Reason |
|---------|----------|-------------|--------|--------|
| PS-1 Policy and Procedures | L M H | Inherited (system level) | Not Applicable (component) | Organizational policy |
| PS-2 Position Risk Designation | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-3 Personnel Screening | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-4 Personnel Termination | L M H | Inherited (system level) | Not Applicable (component) | Personnel process; account disablement in the product is AC-2 |
| PS-5 Personnel Transfer | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-6 Access Agreements | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-7 External Personnel Security | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-8 Personnel Sanctions | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |
| PS-9 Position Descriptions | L M H | Inherited (system level) | Not Applicable (component) | Personnel process |

## PT Personally Identifiable Information Processing and Transparency

Origination: Hybrid. Status: Planned. This family is **applicable**:
TAK processes PII (user identities, callsigns, position tracks). PT controls
are in the 800-53B privacy baseline, not the security baselines.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| PT-1 Policy and Procedures | Privacy | Inherited | Planned | — | Organizational policy, provider TBD (ATO boundary) |
| PT-2 Authority to Process Personally Identifiable Information | Privacy | Inherited | Planned | — | Organizational authority, provider TBD (ATO boundary) |
| PT-3 Personally Identifiable Information Processing Purposes | Privacy | Hybrid | Planned | 4–7 | Purpose is organizational; software limits processing to configured purposes |
| PT-4 Consent | Privacy | Hybrid | Planned | 4 | Consent banner (with AC-8) |
| PT-5 Privacy Notice | Privacy | Hybrid | Planned | 4 | Notice text organizational; display is a product function |
| PT-6 System of Records Notice | Privacy | Inherited | Planned | — | Organizational publication |
| PT-7 Specific Categories of Personally Identifiable Information | Privacy | Hybrid | Planned | 4–7 | Depends on deployment data |
| PT-8 Computer Matching Requirements | Privacy | Inherited | Planned | — | Organizational agreements |

## RA Risk Assessment

Origination: Hybrid. Status: Planned. RA-3 (Risk Assessment, L M H) is
organizational; per-run threat sketches in `plan.md` are SA-11(2) evidence.

### RA-5 Vulnerability Monitoring and Scanning

| Field | Value |
|-------|-------|
| Control | RA-5 Vulnerability Monitoring and Scanning |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Partially Implemented (dependency scanning only) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | `cargo audit --deny warnings` and `cargo deny check advisories` on every Rust-relevant PR (inside `rust-ci-gate`) and nightly |
| Evidence | Present: [deny.toml](../../deny.toml), [.cargo/audit.toml](../../.cargo/audit.toml), [rust-ci.yml](../../../.github/workflows/rust-ci.yml) `supply-chain` job, [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml). Local scans pass (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make audit` 93 crates, 1378 advisories; `make deny` `advisories ok`), but only with the two ignores of the open POAM-2026-10-26-09. Per-PR scanning ran in PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `cargo audit --deny warnings` (93 crates, 1380 advisories) and `cargo deny check` (`advisories ok`) pass, again only with the same two ignores. Nightly scanning: NOT RUN (CI only) |
| Gaps | No nightly run recorded. The local and PR 7 CI passes depends on two advisory ignores under the open POAM-2026-10-26-09 (accepted for development by the system owner 2026-10-27, pending AO review; not Risk Accepted); the pass is not evidence (rule 3). No scanning of the TOE itself (no TOE yet); organizational scanning of deployed hosts is Inherited. Dependabot alerts are on but automated security updates are paused (POAM-2026-10-26-10) |
| POA&M | POAM-2026-10-26-07 (notification channel); open POA&M POAM-2026-10-26-09 (two advisory ignores; accepted for development by the system owner 2026-10-27, pending AO review; not Risk Accepted); POAM-2026-10-26-10 (Dependabot repository settings) |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml) (lines 6–15, line 7 for RA-5), [.cargo/audit.toml](../../.cargo/audit.toml), [scripts/check-advisory-sync.sh](../../scripts/check-advisory-sync.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments |

### RA-5(2) Update Vulnerabilities to Be Scanned

| Field | Value |
|-------|-------|
| Control | RA-5(2) Update Vulnerabilities to Be Scanned |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Implemented (status stays Planned until a recorded CI and nightly run shows the fetch) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | RustSec advisory database fetched on every run; `maximum-db-staleness = "P30D"` ([deny.toml](../../deny.toml) line 57) |
| Evidence | Configuration present. Local `make audit` fetched the database (1378 advisories) and `make deny` passed (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)). The control claim is the fetch on every CI and nightly run. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): the `supply-chain` job logged `Fetching advisory database` and `Loaded 1380 security advisories`. Nightly fetch: NOT RUN (CI only) |
| Gaps | No nightly run recorded, so the status stays Planned (rule 2 and this block's target note) |
| POA&M | — |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml) (line 8), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml) header comments |

## SA System and Services Acquisition

Origination: System-specific (developer). Status: Partially Implemented
(SA-10, SA-11, SA-11(1), SA-15, SA-22).

### SA-3 System Development Life Cycle

| Field | Value |
|-------|-------|
| Control | SA-3 System Development Life Cycle |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Partially Implemented |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | `/tak-rs-workflow`: plan → build → test → review → fix loop → docs → gate; [CONTRIBUTING.md](../../CONTRIBUTING.md) |
| Evidence | Present: [CONTRIBUTING.md](../../CONTRIBUTING.md), workflow definition [.claude/skills/tak-rs-workflow/](../../../.claude/skills/tak-rs-workflow/). The Phase 0 run record (plan, QA report, findings, fix rounds) exists under `.agent-runs/`. Gated run record: pending (post-gate WU10, rule 4); the PR 7 CI run is test evidence (SA-11), not SA-3 evidence |
| Gaps | No gated run yet. Organizational SDLC roles and security responsibilities are TBD |
| POA&M | — |
| Phase | 0 |
| Annotation locations | — |

### SA-8 Security and Privacy Engineering Principles

| Field | Value |
|-------|-------|
| Control | SA-8 Security and Privacy Engineering Principles |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (formal principles list in Phase 2) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Threat sketches in plans; dependency rules in [ARCHITECTURE.md](../../ARCHITECTURE.md) |
| Evidence | — |
| Gaps | No formal list of principles applied |
| POA&M | — |
| Phase | 2 |
| Annotation locations | — |

### SA-10 Developer Configuration Management

| Field | Value |
|-------|-------|
| Control | SA-10 Developer Configuration Management |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (change approval not enforced until POAM-2026-10-26-04 closes) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | git, `Cargo.lock`, `CHANGELOG.md`, Dependabot, SPDX header and provenance rules |
| Evidence | [Cargo.lock](../../Cargo.lock), [CHANGELOG.md](../../CHANGELOG.md), [CONTRIBUTING.md](../../CONTRIBUTING.md), [POAM.md](POAM.md) (flaw tracking). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `cargo metadata --locked` PASS and every `make` cargo target runs `--locked`; `make policy` PASS (`check-annotations` SPDX rule (c) and mapping-block rules ok); `make deny` `sources ok`. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): the cargo jobs ran `--locked`; `make policy` (SPDX and mapping-block rules) and `make deny` (`sources ok`) PASS |
| Gaps | Change approval not enforced by a ruleset (POAM-2026-10-26-04): PR 7 was merged to `main` as `9f754245` with green checks but no required check or review. Dependabot has not run |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml) header comments |

### SA-11 Developer Testing and Evaluation

| Field | Value |
|-------|-------|
| Control | SA-11 Developer Testing and Evaluation |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | CI `fmt`, `clippy` (native and wasm32), `test`, `doc`; policy self-tests; parity harness v0 running TAKCL `StartupTests` against the Java reference |
| Evidence | [Makefile](../../Makefile), `rust/scripts/`, [parity/harness.sh](../../parity/harness.sh). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 5 passed, 0 failed; `make test-doc` 2 passed; `make fmt-check`, `make clippy`, `make clippy-wasm`, `make build-wasm`, `make doc` PASS; `make policy-selftest` 199 passed, 0 failed; `parity/harness.sh selftest` 39 passed, 0 failed; harness integrity negative (wrong wrapper-jar pin) exits 4 before any engine step. QA Round 2, 2026-10-26, local (same section): the same gates PASS with the same test counts; `make policy-selftest` 269/0 before and 281/0 after 12 QA cases; harness selftest 44/0. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 5 passed, 0 failed; doc tests 2 passed; `fmt`, `lint (native)`, `lint (wasm32)`, `doc` PASS; `make policy-selftest` 331 passed, 0 failed; `ci-gate: PASS (rust=true)`. [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328): harness selftest 44/0; TAKCL `StartupTests` against the Java reference PASS (`verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0`) |
| Gaps | QA-007 (rule (d) of `check-annotations.sh` treated every `Not Applicable (component)` row as mapped) was open at QA Round 2; fix round 3 addresses it, not QA-verified (its self-tests pass in PR 7 CI). TAKCL `StartupTests` against the Java reference ran once (PR 7); it is a Java baseline, not a comparison. No nightly run yet. No wasm runtime tests, and CI builds wasm32 in dev mode only with no bundle-size baseline; the Phase 4 plan adds a headless `wasm-bindgen-test` job, a release wasm build with `--all-features` and a size report (WEB-003, WEB-006). TAKCL cannot yet target the Rust server (POAM-2026-10-26-06) |
| POA&M | POAM-2026-10-26-06 |
| Phase | 0 |
| Annotation locations | [Makefile](../../Makefile), [scripts/check-unsafe.sh](../../scripts/check-unsafe.sh), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [scripts/ci-changes.sh](../../scripts/ci-changes.sh), [scripts/ci-gate.sh](../../scripts/ci-gate.sh), [scripts/check-src-coupling.sh](../../scripts/check-src-coupling.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments |

### SA-11(1) Static Code Analysis

| Field | Value |
|-------|-------|
| Control | SA-11(1) Static Code Analysis |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (ceiling lowered by NIST-006: three of the five analysers have never run) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | clippy with `-D warnings` (`all` deny, `pedantic` warn, so both block), with `disallowed-methods` and `disallowed-types` bans in [clippy.toml](../../clippy.toml), [web/clippy.toml](../../web/clippy.toml) and [crates/tak-api-types/clippy.toml](../../crates/tak-api-types/clippy.toml); cargo-deny; actionlint, zizmor, shellcheck in CI. No Dockerfile linter (hadolint is in no workflow) |
| Evidence | QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make clippy` and `make clippy-wasm` PASS, 0 warnings; `make test` includes the `tests/clippy_config.rs` drift guards (4 tests); `make deny` PASS. actionlint, zizmor and shellcheck: NOT RUN locally (not installed). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `lint (native)` and `lint (wasm32)` PASS; actionlint 1.7.12 on `.github/workflows/rust-*.yml` PASS; zizmor 1.31.1 "No findings to report"; shellcheck 0.11.0 on `scripts/*.sh` and `parity/harness.sh` PASS. The first shellcheck run (40536002508, head `0c56b62a`) failed on SC2209, fixed in `de62d29e` |
| Gaps | The three CI analysers have run once (PR 7), not yet nightly or as a required check. hadolint is in no workflow and has never run, so the two parity Dockerfiles ([parity/docker/](../../parity/docker/)) get no static analysis; they are test fixtures outside the TOE (gap recorded, no POA&M ID). `check-workflows` was bypassed in QA-002 (fixed in fix round 1) and QA-006 (fixed in fix round 2 with parsed YAML; verified at QA Round 2). Fix round 3 changes it again (SEC-010, SEC-011, SEC-012); not QA-verified |
| POA&M | — |
| Phase | 0 |
| Annotation locations | — |

### SA-11(2) Threat Modeling and Vulnerability Analyses

| Field | Value |
|-------|-------|
| Control | SA-11(2) Threat Modeling and Vulnerability Analyses |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Threat sketch in `plan.md` §5 per run |
| Evidence | Threat sketch present in the run's `plan.md` §5 and used by the security review. Plan accepted at the gate: pending (post-gate WU10) |
| Gaps | No system-level threat model for the TOE yet |
| POA&M | — |
| Phase | 0 (CI and supply chain); 2+ (TOE) |
| Annotation locations | — |

### SA-11(8) Dynamic Code Analysis

| Field | Value |
|-------|-------|
| Control | SA-11(8) Dynamic Code Analysis |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (fuzzing, Phase 1) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None; `rust/fuzz/` arrives with `tak-cot` |
| Evidence | — |
| Gaps | No parsers yet |
| POA&M | — |
| Phase | 1 |
| Annotation locations | — |

### SA-15 Development Process, Standards, and Tools

| Field | Value |
|-------|-------|
| Control | SA-15 Development Process, Standards, and Tools |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (SA-15b review cadence is organization-defined, TBD) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Pinned toolchain, `Makefile` as the single entry point (CI runs the same targets), lint policy, checksum-verified CI tools (`taiki-e/install-action`, `fallback: none`), least-privilege CI token, [CONTRIBUTING.md](../../CONTRIBUTING.md) |
| Evidence | [rust-toolchain.toml](../../rust-toolchain.toml), [Cargo.toml](../../Cargo.toml), [rustfmt.toml](../../rustfmt.toml), [clippy.toml](../../clippy.toml), [Makefile](../../Makefile), [CONTRIBUTING.md](../../CONTRIBUTING.md); tool pins in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): toolchain 1.99.1 from the pin; every gate ran through its `make` target; `make fmt-check`, `make clippy`, `make policy` PASS. Checksum-verified tool installation in PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `taiki-e/install-action` logged "verifying sha256 checksum" for zizmor, shellcheck, cargo-cyclonedx, cargo-deny and cargo-audit; the actionlint tarball passed `sha256sum --check --strict` ("OK"); every job's token was `Contents: read`, `Metadata: read` |
| Gaps | SA-15b review cadence not defined |
| POA&M | — |
| Phase | 0 |
| Annotation locations | [rust-toolchain.toml](../../rust-toolchain.toml), [Cargo.toml](../../Cargo.toml), [Makefile](../../Makefile), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh), [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh), [scripts/check-unsafe.sh](../../scripts/check-unsafe.sh), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments |

### SA-15(7) Automated Vulnerability Analysis

| Field | Value |
|-------|-------|
| Control | SA-15(7) Automated Vulnerability Analysis |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented (dependencies only; no SAST/DAST of the TOE yet) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | cargo-audit and cargo-deny per change and nightly |
| Evidence | Present: [deny.toml](../../deny.toml), [.cargo/audit.toml](../../.cargo/audit.toml), the `supply-chain` job and [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml). Local `make audit` and `make deny` pass only with the two ignores of the open POAM-2026-10-26-09 (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)). Per-change run: PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3), the same result with the same two ignores. Nightly runs: NOT RUN (CI only) |
| Gaps | No nightly run recorded; the local and CI passes depend on an open POA&M (rule 3). No analysis of first-party TOE code beyond clippy |
| POA&M | — |
| Phase | 0 |
| Annotation locations | — |

### SA-15(13) Logging Syntax

New in Release 5.2.0.

| Field | Value |
|-------|-------|
| Control | SA-15(13) Logging Syntax |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. Phase 0 has no runtime logging; CI logs are produced by GitHub Actions |
| Evidence | — |
| Gaps | Logging syntax for `tak-audit` and service logs to be defined (organization-defined parameters TBD) |
| POA&M | — |
| Phase | 2 (`tak-audit`) |
| Annotation locations | — |

### SA-17 Developer Security and Privacy Architecture and Design

| Field | Value |
|-------|-------|
| Control | SA-17 Developer Security and Privacy Architecture and Design |
| 800-53B baseline | H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (Phase 2+) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | [ARCHITECTURE.md](../../ARCHITECTURE.md) (short workspace map only) |
| Evidence | — |
| Gaps | No security architecture and design description |
| POA&M | — |
| Phase | 2+ |
| Annotation locations | — |

### SA-22 Unsupported System Components

| Field | Value |
|-------|-------|
| Control | SA-22 Unsupported System Components |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Partially Implemented (two open exceptions) |
| Phase 0 target | Partially Implemented (two open exceptions) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | `unmaintained = "all"` in `deny.toml`; cargo-audit `--deny warnings` turns unmaintained warnings into failures; each exception is an advisory ignore with a `# POAM:` ID, identical in both files |
| Evidence | [deny.toml](../../deny.toml) line 46 (`unmaintained = "all"`), lines 49–52 (marker) and 54–55 (the two ignores); [.cargo/audit.toml](../../.cargo/audit.toml) lines 31 and 39 (markers) and 34 and 42 (the two ignores). QA Round 2 removed the ignores at their earlier positions (`deny.toml` 53–54, `audit.toml` 33 and 40); the 2026-10-27 comment edits moved them. Detection evidence (the only SA-22 evidence; rule 3): QA Round 2, 2026-10-26, local, negative gate "POAM-09 detection" ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): with the two ignores removed in scratch copies, `cargo deny --locked --config <scratch> check advisories` exits 1 with exactly two errors, RUSTSEC-2024-0370 (`proc-macro-error`) and RUSTSEC-2025-0141 (`bincode`), and `cargo audit --deny warnings` exits 1 with the same two IDs; the controls with the unmodified files exit 0. `make check-advisory-sync` PASS (QA Rounds 1 and 2). The `make deny` and `make audit` passes with the ignores are not SA-22 evidence |
| Gaps | Two unmaintained crates reached through `yew` 0.23 are ignored: `proc-macro-error` (build time only) and `bincode` 1.3.3, a runtime library in `yew` → `gloo` → `gloo-worker` that will compile into a Phase 4 wasm bundle. The ignores are the only reason `make deny` and `make audit` pass, and `rust-ci-gate` did go green on that exception in PR 7 CI (run 40539495255); the exception is accepted only for development (system owner, 2026-10-27), not by the authorizing official. Build-host tools (CMake, Go, Perl, C compiler) from Phase 2 are outside this check |
| POA&M | Open POA&M POAM-2026-10-26-09 (accepted for development by the system owner 2026-10-27, pending AO review; not Risk Accepted) |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml) (line 11), [.cargo/audit.toml](../../.cargo/audit.toml) header comments |

### SA-24 Design for Cyber Resiliency

New in Release 5.2.0.

| Field | Value |
|-------|-------|
| Control | SA-24 Design For Cyber Resiliency |
| 800-53B baseline | — |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. The goals, objectives, techniques, approaches and principles are organization-defined parameters |
| Evidence | — |
| Gaps | Parameters TBD by the organization; design input expected in the Phase 2 and Phase 8 (cluster) plans |
| POA&M | — |
| Phase | 2+ |
| Annotation locations | — |

## SC System and Communications Protection

Origination: System-specific. Status: Planned. Phase 0 contains no crypto code
and no crypto dependency.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| SC-8 Transmission Confidentiality and Integrity | M H | System-specific | Planned | 3 | TLS 1.2+ on every input |
| SC-12 Cryptographic Key Establishment and Management | L M H | Hybrid | Planned | 2, 4 | Keystores and key handling in `tak-crypto` (Phase 2); certificate enrollment (Phase 4); key custody procedures organizational |
| SC-17 Public Key Infrastructure Certificates | M H | Hybrid | Planned | 2, 4 | Certificate validation and CRL in `tak-auth` (Phase 2); TAK CA operations (Phase 4); the issuing PKI is organizational |
| SC-23 Session Authenticity | M H | System-specific | Planned | 4 | Admin sessions |
| SC-28 Protection of Information at Rest | M H | Hybrid | Planned | 2 | Keystores and secrets; disk encryption organizational |

### SC-13 Cryptographic Protection

| Field | Value |
|-------|-------|
| Control | SC-13 Cryptographic Protection |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. Pre-positioned: `.cargo/config.toml` forces the bundled FIPS module (`AWS_LC_FIPS_SYS_USE_SYSTEM=0`); `deny.toml` limits native code to the AWS-LC family |
| Evidence | Present, inert until Phase 2: [.cargo/config.toml](../../.cargo/config.toml) (no crate depends on `aws-lc-rs`, so no gate exercises these settings) |
| Gaps | No crypto in Phase 0; rustls `fips` also builds non-FIPS `aws-lc-sys`; the FIPS graph will fail `[bans.build]` on two `libloading` test DLLs until Phase 2 adds a checksum-pinned bypass ([FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues, QA-004) |
| POA&M | POAM-2026-10-26-08 |
| Phase | 2 (`tak-crypto`) |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml) header comment (pre-positioned settings; the file says they are inert until Phase 2) |

## SI System and Information Integrity

Origination: Hybrid. Status: Planned.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| SI-10 Information Input Validation | M H | System-specific | Planned | 1 | `tak-cot` parser with size and depth limits, and fuzzing |
| SI-11 Error Handling | M H | System-specific | Planned | 3–4 | No internal detail in client-facing errors (streaming inputs, REST API); details only in logs |

### SI-2 Flaw Remediation

| Field | Value |
|-------|-------|
| Control | SI-2 Flaw Remediation |
| 800-53B baseline | L M H |
| Origination | Hybrid (Dependabot alerts and security updates are repository settings owned by the repository administrator) |
| Status | Planned |
| Phase 0 target | Partially Implemented (remediation timeliness metric Planned) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Blocking advisory gate; advisory ignores allowed only with a `# POAM:` ID (kept identical in `deny.toml` and `.cargo/audit.toml`); Dependabot version and security update PRs ([dependabot.yml](../../../.github/dependabot.yml) plus the repository settings) |
| Evidence | Present: [deny.toml](../../deny.toml), [.cargo/audit.toml](../../.cargo/audit.toml), [dependabot.yml](../../../.github/dependabot.yml). Local: `make check-advisory-sync` PASS (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)). Blocking gate in CI: ran in PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3) (`make audit`, `make deny`, `check-advisory-sync` PASS, with the two ignores). Dependabot PRs: NOT RUN. Repository settings read on 2026-10-26: `GET repos/192d-Wing/Server/vulnerability-alerts` returned 204 (alerts on); `GET repos/192d-Wing/Server/automated-security-fixes` returned `{"enabled":true,"paused":true}` |
| Gaps | Automated security updates are **paused**, and the settings are not tracked in the repository (POAM-2026-10-26-10). The local and PR 7 CI advisory passes depend on the open POAM-2026-10-26-09 (rule 3). No remediation-time target (organization-defined) |
| POA&M | Open POA&M POAM-2026-10-26-09 (two advisory ignores; accepted for development by the system owner 2026-10-27, pending AO review; not Risk Accepted); POAM-2026-10-26-10 (Dependabot repository settings) |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml) (line 9), [.cargo/audit.toml](../../.cargo/audit.toml), [scripts/check-advisory-sync.sh](../../scripts/check-advisory-sync.sh), [dependabot.yml](../../../.github/dependabot.yml) header comments |

### SI-2(2) Automated Flaw Remediation Status

| Field | Value |
|-------|-------|
| Control | SI-2(2) Automated Flaw Remediation Status |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented (notification channel POAM-2026-10-26-07) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Nightly `rust-supply-chain.yml` |
| Evidence | Present: [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml). First scheduled run: NOT RUN (CI only) |
| Gaps | No scheduled run recorded. Failures notified only by GitHub email to the cron editor (POAM-2026-10-26-07). Automated security updates paused (POAM-2026-10-26-10) |
| POA&M | POAM-2026-10-26-07, POAM-2026-10-26-10 |
| Phase | 0 |
| Annotation locations | [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments |

### SI-2(7) Root Cause Analysis

New in Release 5.2.0.

| Field | Value |
|-------|-------|
| Control | SI-2(7) Root Cause Analysis |
| 800-53B baseline | — |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None defined. `/tak-rs-workflow` findings and fix rounds record causes per change, but there is no root-cause procedure or effectiveness monitoring |
| Evidence | — |
| Gaps | Root-cause procedure for flaws and CI failures |
| POA&M | — |
| Phase | 3 (first deployable milestone) |
| Annotation locations | — |

### SI-7 Software, Firmware, and Information Integrity

| Field | Value |
|-------|-------|
| Control | SI-7 Software, Firmware, and Information Integrity |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (Phase 9) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None for the TOE. CI-tool integrity (checksums, SHA pins) is mapped to SR-11 and SA-15, not to TOE SI-7 |
| Evidence | — |
| Gaps | No integrity verification of shipped software |
| POA&M | — |
| Phase | 9 |
| Annotation locations | — |

### SI-7(12) Integrity Verification

Revised in Release 5.2.0.

| Field | Value |
|-------|-------|
| Control | SI-7(12) Integrity Verification |
| 800-53B baseline | — |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. The enhancement covers user-installed software verified before execution; for TAK-RS that is the release artifacts and, later, plugins |
| Evidence | — |
| Gaps | Signed artifacts (Phase 9); plugin integrity (Phase 8) |
| POA&M | — |
| Phase | 8–9 |
| Annotation locations | — |

### SI-7(15) Code Authentication

| Field | Value |
|-------|-------|
| Control | SI-7(15) Code Authentication |
| 800-53B baseline | H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned (Phase 9) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None |
| Evidence | — |
| Gaps | No code signing |
| POA&M | — |
| Phase | 9 |
| Annotation locations | — |

## SR Supply Chain Risk Management

Origination: Hybrid. Status: Partially Implemented (SR-3, SR-4, SR-11).

### SR-2 Supply Chain Risk Management Plan

| Field | Value |
|-------|-------|
| Control | SR-2 Supply Chain Risk Management Plan |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned (organizational plan TBD) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None. The organizational SCRM plan is TBD; developer controls are SR-3, SR-4 and SR-11 |
| Evidence | — |
| Gaps | SCRM plan |
| POA&M | — |
| Phase | Before ATO (Phase 9) |
| Annotation locations | — |

### SR-3 Supply Chain Controls and Processes

| Field | Value |
|-------|-------|
| Control | SR-3 Supply Chain Controls and Processes |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (Rust graph controls in place; parity-tooling residual POAM-2026-10-26-02; SCRM plan organizational) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | cargo-deny sources, licences and bans; three-layer pure-Rust gate (`[bans]` wrappers; `check-native-deps.sh` per target, which also rejects crypto crate names on wasm32; `[bans.build]` with `executables = "deny"`, `include-archives = true` and an `allow-build-scripts` list of 15 crates); native code only through `aws-lc-rs` (`aws-lc-fips-sys`, and `aws-lc-sys` under POAM-2026-10-26-08); provenance rule |
| Evidence | [deny.toml](../../deny.toml) `[bans]` (line 116), `[bans.build]` (lines 179–191), `[sources]` (lines 212–216); [scripts/native-allowlist.txt](../../scripts/native-allowlist.txt); [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance and §Supply chain. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `bans ok, sources ok`; `make check-native-deps` PASS on 3 targets (wasm32: only the `prettyplease` and `wasm-bindgen-shared` markers; no crypto names); negative check: removing `zmij` from `allow-build-scripts` gives `error[build-script-not-allowed]`, `bans FAILED` (exit 2). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `bans ok, sources ok`; `check-native-deps: ok` on x86_64 and aarch64 Linux and wasm32 (wasm32: the two markers only); job tokens `Contents: read` |
| Gaps | Parity job downloads unverified Maven, Gradle-plugin, Node and npm artifacts (POAM-2026-10-26-02); parity fixture images are Docker Hub `eclipse-temurin` and community `postgis/postgis`, not UBI or Iron Bank (POAM-2026-10-26-03, extended by NIST-021); non-FIPS `aws-lc-sys` from Phase 2 (POAM-2026-10-26-08). `allow-build-scripts` names crates, not versions, so a new build script in an allowed crate relies on `Cargo.lock` and Dependabot review. The `links` check is a heuristic. One CI run (PR 7); no nightly run yet. The parity Dockerfiles are not linted (hadolint in no workflow) |
| POA&M | POAM-2026-10-26-02, POAM-2026-10-26-03, POAM-2026-10-26-08 |
| Phase | 0 |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml), [deny.toml](../../deny.toml), [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments |

### SR-4 Provenance

| Field | Value |
|-------|-------|
| Control | SR-4 Provenance |
| 800-53B baseline | — |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (no signed build provenance until Phase 9; POAM-2026-10-26-05) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | crates.io-only sources; SHA-pinned actions; image digests; first-party provenance rule (`Derived from` per ported item) |
| Evidence | [deny.toml](../../deny.toml) `[sources]` (lines 212–216: unknown registries and git denied, `allow-git = []`); SHA-pinned actions and digest-pinned images listed in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory; [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `sources ok`; `make policy` `check-workflows` ok (SHA-pin rule); `parity/harness.sh selftest` PASS (rejects an image pin without `@sha256:`). QA Round 2, 2026-10-26, local (same section): the same gates PASS; `check-workflows` ok on 3 parsed files (rule W1, 40-hex SHA with a version comment); QA re-resolved the `actions/checkout` and `Swatinem/rust-cache` tags and re-hashed the OSCAL files, all matching. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `sources ok`; `check-workflows` ok |
| Gaps | At QA Round 1, `check-workflows` was line-based and could be bypassed (QA-006); the parsed-YAML version is verified at QA Round 2; fix round 3 changes it again, not QA-verified. No signed build provenance; SBOM not required, generated once (PR 7 CI, `rust-sbom`) and not signed (POAM-2026-10-26-01); the parity build's transitive downloads are unverified (POAM-2026-10-26-02); legal review open (POAM-2026-10-26-05) |
| POA&M | POAM-2026-10-26-01, POAM-2026-10-26-02, POAM-2026-10-26-05 |
| Phase | 0; 9 (signed provenance) |
| Annotation locations | [deny.toml](../../deny.toml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml) header comments |

### SR-5 Acquisition Strategies, Tools, and Methods

| Field | Value |
|-------|-------|
| Control | SR-5 Acquisition Strategies, Tools, and Methods |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None claimed. Developer-side inputs exist (crates.io-only sources, pinned and checksum-verified tools, a 7-day Dependabot cooldown on version updates), but the acquisition strategy itself is organizational |
| Evidence | — |
| Gaps | Organizational acquisition strategy, tools and methods TBD (ATO boundary) |
| POA&M | — |
| Phase | Before ATO (Phase 9) |
| Annotation locations | — |

### SR-6 Supplier Assessments and Reviews

| Field | Value |
|-------|-------|
| Control | SR-6 Supplier Assessments and Reviews |
| 800-53B baseline | M H |
| Origination | Hybrid |
| Status | Planned |
| Phase 0 target | Planned |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None claimed. Developer-side inputs: RustSec advisories and unmaintained-crate reports per dependency (RA-5, SA-22); per-crate licence and source checks (SR-3) |
| Evidence | — |
| Gaps | No supplier assessment or review process; the review frequency is organization-defined |
| POA&M | — |
| Phase | Before ATO (Phase 9) |
| Annotation locations | — |

### SR-11 Component Authenticity

| Field | Value |
|-------|-------|
| Control | SR-11 Component Authenticity |
| 800-53B baseline | L M H |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | `Cargo.lock` checksums with `--locked`; install-action checksum-verified manifests and a checksum-pinned actionlint download; Gradle distribution SHA-256 and wrapper-jar SHA-256 checks in the parity harness, plus `gradle/actions/wrapper-validation`; the parity Gradle cache holds downloaded modules only, so the distribution is checked on every run |
| Evidence | [Cargo.lock](../../Cargo.lock), [parity/pins.env](../../parity/pins.env) (`GRADLE_DIST_SHA256`, `GRADLE_WRAPPER_JAR_SHA256`); values in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `cargo metadata --locked` PASS; harness integrity negative (wrapper-jar pin set to zeros) exits 4 with "does not match the pin" before any engine step; `parity/harness.sh selftest` 39/0. All 6 `gradle-wrapper.jar` files under `src/` hash to the pinned value (checked 2026-10-26). CI-side checks, PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): install-action "verifying sha256 checksum" for every installed tool; `actionlint_1.7.12_linux_amd64.tar.gz: OK`. [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328): `gradle/actions/wrapper-validation` passed; `wrapper verified: Gradle 8.10.2, wrapper jar 91a2…1e60` |
| Gaps | Maven and npm transitive artifacts in the parity build are unverified, and cached Gradle modules are trust-on-first-use (POAM-2026-10-26-02); the PR 7 parity run built the reference with these downloads, and its ESLint plugin is disabled for the reference UI build (`ce9caf79`). No nightly run yet |
| POA&M | POAM-2026-10-26-02 |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments |

---

## OSCAL

Decision: **no OSCAL artifacts in Phase 0.** This Markdown document is the
single source of truth. It is structured to convert cleanly: Rev 5 IDs,
OSCAL-aligned status and origination vocabularies, and per-control blocks with
stable fields.

Reasons for deferring:

1. There is no TOE component yet. The Phase 0 claims are developer-process
   controls (CM, SA, SR), which belong in the SSP's system and organizational
   parts rather than in a component definition.
2. Hand-maintained Markdown and JSON would drift. A generator pays off only
   once there are enough runtime controls.
3. Validation needs NIST `oscal-cli` (Java) or a JSON Schema validator, which
   is new CI tooling with no current payoff.

Revisit trigger: the Phase 2 plan run (first runtime controls: SC-13, AU-2,
AU-3, AU-12, IA-2, IA-5). That run should add an OSCAL 1.1.x
`component-definition` for the `takserver` component, generated from this
document and validated in CI. This is not a POA&M item (no weakness); it is
recorded as Planned in [ATO_EVIDENCE.md](ATO_EVIDENCE.md).

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-27 | 0.5 | PR 7 CI evidence (rust-ci 40539495255, rust-parity 40539495328; head `ce9caf79`, merged as `9f754245`). "Status at this revision": the first GitHub Actions runs recorded; how rules 2 to 4 apply to them (rule 2's green-gate condition met, its nightly condition not). Evidence cells updated in CA-5, CA-7, CM-2, CM-2(2), CM-3, CM-3(2), CM-6, CM-8, RA-5, RA-5(2), SA-10, SA-11, SA-11(1), SA-15, SA-15(7), SI-2, SR-3, SR-4, SR-11; gaps that said "no CI run" corrected in those blocks and in CM-5, SA-3, SA-22. hadolint recorded as a gap in SA-11(1) and SR-3 (in no workflow; never run). No status changed: 0 Implemented, 12 Partially Implemented, 30 Planned | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.4 | Gate: POAM-2026-10-26-09 accepted for development by the system owner (John Edward Willman V) on 2026-10-27, pending AO review; still Open, not Risk Accepted. Text updated in "Status at this revision", CA-5 gaps, RA-5 gaps and POA&M, SA-22 gaps and POA&M, SI-2 POA&M. No status changed (12 Partially Implemented, 30 Planned, 0 Implemented); rule 3 unchanged, the pass with ignores is still not evidence. Line numbers re-verified with `grep -n` after the 2026-10-27 `deny.toml` and `.cargo/audit.toml` comment edits: SA-22 evidence (deny 49–52, 54–55; audit 31, 39, 34, 42), RA-5(2) `maximum-db-staleness` (57), SR-3 `[bans]` (116), `[bans.build]` (179–191), `[sources]` (212–216), SR-4 `[sources]` (212–216) | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs). No status changed (12 Partially Implemented, 30 Planned, 0 Implemented). "Status at this revision": QA Round 2 added as evidence; rule 3 now covers SA-22 with the carve-out "detection evidence only; the pass with ignores is not evidence" (NIST-018); rule 5 says fix round 3 is not QA-verified. SA-22 evidence is the QA Round 2 POAM-09 detection negative gate (NIST-018). CM-2, CM-6, SR-4: QA Round 2 evidence for the parsed-YAML `check-workflows` and the QA-006 caveat for Round 1; CM-6 permission evidence reworded (grep shows presence, W3 checks top level) (NIST-017). CM-2(2), CM-3, CM-3(2), SA-11(1): QA-006 verified at Round 2. CM-6 and SR-3 cite the extended POAM-2026-10-26-03 (NIST-021). SA-11 and CM-3 note QA-007 and QA-008. CM-7 annotation location removed with the fix round 3 annotation change (NIST-019); CM-7 stays Planned | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | Phase 5 and WU10 (pre-gate). Statuses raised from QA Round 1 local evidence, never above the Phase 0 target: CA-5, CM-2, CM-6, CM-10, SA-10, SA-11, SA-11(1), SA-15, SA-22, SR-3, SR-4, SR-11 → Partially Implemented (12); 30 blocks stay Planned. New blocks CA-5, CM-5, SR-5, SR-6 (42 in total); "CA-5 is not claimed" sentence removed (NIST-004). Targets: SA-11(1) Implemented → Partially Implemented (NIST-006); SA-22 → Partially Implemented (two open exceptions), "Risk Accepted" → "open POA&M" (NIST-003). New Planned rows AC-7, AC-10, AC-12, AU-8, AU-10, IA-2(12), IA-8, SC-12, SC-17, SI-10, SI-11 (NIST-009). "Not Applicable (component)" vocabulary and system-level Inherited wording (NIST-011). Dependabot settings part of SI-2 Hybrid, POAM-2026-10-26-10 (NIST-007). QA-001, QA-002 and QA-006 cited in CM-3, CM-3(2) and SA-11(1) gaps. Evidence and annotation locations refreshed after fix round 1 | `20261026-210255-phase0-foundations` (Phase 5, WU10) |
| 2026-10-26 | 0.1 | Initial skeleton: pinned references (Release 5.2.0, 800-53B), vocabulary, all 20 Rev 5 families, 38 detailed control blocks (all Planned; Phase 0 targets from plan §7.1), Release 5.2.0 pre-map (SA-15(13), SA-24, SI-2(7), SI-7(12)), Inherited and Not Applicable rows with reasons, OSCAL deferral. SA-22 target qualified with Risk Accepted (POAM-2026-10-26-09, raised by WU3) | `20261026-210255-phase0-foundations` (WU6) |
