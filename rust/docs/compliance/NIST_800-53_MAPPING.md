# NIST SP 800-53 Rev 5 Security Control Mapping

**Document Version:** 0.8
**Date:** 2026-10-28
**TAK-RS Version:** 0.0.1 (Phase 1a, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 1a build run, before its gate. 17 of 47 detailed blocks are Partially Implemented, 30 Planned, none Implemented. The Phase 1a statuses rest on the local QA run of 2026-10-28; no CI run exists for this branch, and the golden parity tests (the Phase 1a exit criterion) are NOT RUN.
**Run:** `20261027-180458-phase1-protocol-config`

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

Phase 0 covered the **development environment only**: the Cargo workspace,
supply-chain policy, CI, compliance documents and the parity harness. Every
Phase 0 entry carries the scope note "development environment / CI; no TOE
runtime yet", and the developer-process blocks keep it.

Phase 1a adds the first **TOE library components**: the `tak-proto` and
`tak-cot` crates (TAK Protocol v1 types, framing, the CoT XML parser and
serializer, and the XML ↔ protobuf conversion). They are libraries with no
network I/O, no listener and no logging, and no shipped binary links them
until Phase 3. Blocks for their functions carry the scope note "TOE library
component (`tak-proto`, `tak-cot`); no network wiring until Phase 3". Two
Phase 1a pieces stay **outside the TOE**: the `tak-proto` build script
(build host only; its schema digest is SA-10(1), not SI-7 or SC-13) and the
excluded `rust/fuzz/` workspace (development only).

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

This revision records the **Phase 1a build run before its gate** (run
`20261027-180458-phase1-protocol-config`). Its evidence is the local QA run
of 2026-10-28 on macOS against the uncommitted working tree (base `main`
`df6a4662`, `HEAD` `04774cc0`), plus the results that fix rounds 1 and 2
report after it ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3, "Phase 1a"
records). Rules 6 and 7 below govern that evidence. Line numbers in the
Phase 1a cells were verified with `grep -n` on 2026-10-28 against that
working tree. The orchestrator is still to add the POA&M IDs to the markers
in `limits.rs`, `xml/serialize.rs` and the fuzz workspace, which may shift
lines after those markers.

The Phase 0 record below is unchanged. Phase 0 was a build run whose local
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
   the block claims, and never above the block's target ("Phase 0 target",
   or "Phase 1a target" where the block has one).
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
6. **Phase 1a fix rounds.** Fix rounds 1 and 2 ran after the Phase 1a QA
   run. Their results are the fixers' own runs plus the reviewers'
   re-verification (`findings-*-round1.md`, `findings-*-round2.md`), not a QA
   re-run. They may narrow a gap in a block whose status QA evidence already
   supports (for example the SC-5 CPU bound), but on their own they raise no
   status. SA-10(1) therefore stays Planned.
7. **Phase 1a NOT RUN.** GitHub Actions is disabled for the repository
   ("Actions has been disabled for this repository"). No CI run exists for
   this branch: no `rust-ci-gate`, no `rust-fuzz` run, no corpus capture.
   WU7 (golden capture) and WU8 (golden test) are NOT RUN, so no
   byte-equivalence with the Java reference is shown and the Phase 1a exit
   criterion is **not met**. Nothing is committed after `04774cc0` (commit
   signing temporarily unavailable). No claim in this revision rests on a CI
   run, a golden result or a commit.

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

Phase 0 addressed only developer-process controls (CM, SA, SR, RA, SI, CA)
that the workspace and CI satisfy. Phase 1a adds the first TOE library
components (`tak-proto`, `tak-cot`) and five blocks: SC-5, SI-10, SI-10(3),
SI-11 and SA-10(1). SI-10 and SI-11 were rows before. There are 47 detailed
control blocks. Status at this revision:

| Status (this revision) | Controls | Count |
|------------------------|----------|-------|
| Implemented | — | 0 |
| Partially Implemented | CA-5, CM-2, CM-6, CM-10, SA-10, SA-11, SA-11(1), SA-11(8), SA-15, SA-22, SC-5, SI-10, SI-10(3), SI-11, SR-3, SR-4, SR-11 | 17 |
| Planned | CA-2, CA-7, CM-2(2), CM-3, CM-3(2), CM-5, CM-7, CM-8, CM-14, PL-8, RA-5, RA-5(2), SA-3, SA-8, SA-10(1), SA-11(2), SA-15(7), SA-15(13), SA-17, SA-24, SC-13, SI-2, SI-2(2), SI-2(7), SI-7, SI-7(12), SI-7(15), SR-2, SR-5, SR-6 | 30 |
| **Total** | | **47** |

Phase 1a transitions: SA-11(8) Planned → Partially Implemented (recorded
local fuzz run). New blocks: SC-5, SI-10, SI-10(3) and SI-11 at Partially
Implemented, and SA-10(1) at Planned (rule 6). Before: 12 + 30 = 42. After:
12 + 1 + 4 = 17 Partially Implemented; 30 − 1 + 1 = 30 Planned; 17 + 30 =
47.

Phase 1a targets (plan §7.1 of the Phase 1a run, as amended by the
nist-800-53-reviewer for CM-6 and SA-10(1)), for the blocks Phase 1a
touches. The other blocks keep their Phase 0 target.

| Control | Phase 1a target | Status at this revision |
|---------|-----------------|-------------------------|
| SI-10 | Partially Implemented (library; network wiring Phase 3) | Partially Implemented |
| SI-10(3) | Partially Implemented | Partially Implemented |
| SC-5 | Partially Implemented (component; rate limiting Phase 3/7) | Partially Implemented |
| SI-11 | Partially Implemented | Partially Implemented |
| SA-11 | Partially Implemented (already; evidence added) | Partially Implemented |
| SA-11(8) | Partially Implemented after a recorded fuzz run | Partially Implemented |
| SA-10 | Partially Implemented (already) | Partially Implemented |
| SA-10(1) | Partially Implemented (reviewer proposal; not in plan §7.1) | Planned (rule 6: only fix-round and reviewer runs exercise it) |
| SA-15 | Partially Implemented (already) | Partially Implemented |
| SR-3 | Partially Implemented (already) | Partially Implemented |
| SR-4 | Partially Implemented (already) | Partially Implemented |
| CM-10 | Partially Implemented (already) | Partially Implemented |
| CM-6 | Partially Implemented (library defaults; NIST-006) | Partially Implemented |
| CA-5 | Partially Implemented (already) | Partially Implemented |
| CM-3, CM-3(2) | Planned until a nightly run is recorded (rule 2) | Planned |
| CM-7 | Planned | Planned |
| SC-13 | Planned (Phase 2) | Planned |

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

Every status above is a **component** status: for the TAK-RS development
environment, and for SC-5, SI-10, SI-10(3) and SI-11 also for the TOE library
components `tak-proto` and `tak-cot`. It is not a system-level determination;
that is made for the system at the ATO boundary.

## Family Coverage

| Family | Origination | Status (this revision) | Phase 0 target | Reason / planned phase |
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
| SA System and Services Acquisition | System-specific | Partially Implemented (SA-10, SA-11, SA-11(1), SA-11(8), SA-15, SA-22) | Partially Implemented | Developer-side controls; SA blocks below |
| SC System and Communications Protection | System-specific | Partially Implemented (SC-5, TOE library component) | Planned (Phase 1a: SC-5 Partially Implemented) | SC-5 from Phase 1a; Phases 2–3 (SC-8, SC-13, SC-23, SC-28) |
| SI System and Information Integrity | Hybrid | Partially Implemented (SI-10, SI-10(3), SI-11, TOE library component) | Partially Implemented (SI-2, SI-2(2)); Phase 1a: SI-10, SI-10(3), SI-11 | SI-10, SI-10(3), SI-11 from Phase 1a (`tak-proto`, `tak-cot`) |
| SR Supply Chain Risk Management | Hybrid | Partially Implemented (SR-3, SR-4, SR-11) | Partially Implemented | SR blocks below |

Coverage: 20 of 20 Rev 5 families have a section below.

### Per-control block fields

Each detailed block uses the same fields, so the document converts cleanly to
an OSCAL component definition later (see [OSCAL](#oscal)):
Control, 800-53B baseline, Origination, Status, Phase 0 target, Scope,
Implementation, Evidence, Gaps, POA&M, Phase, Annotation locations. Blocks
that Phase 1a touches also carry **Phase 1a target**; blocks added in Phase
1a give "— (block added in Phase 1a)" as the Phase 0 target.

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
40539495328, 2026-10-27). "Phase 1a QA" evidence is the local run recorded in
[TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3, "2026-10-28 Phase 1a QA" (macOS,
uncommitted working tree before fix round 1). "Phase 1a fix round 1" and
"fix round 2" are the fixers' runs and the reviewers' re-verification in the
same section (rule 6).

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
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | TAK-RS keeps its POA&M register in [POAM.md](POAM.md): weakness, controls, proposed risk, mitigation, milestones, owner role, status and code locations. Every POA&M ID referenced under `rust/` or in `.github/workflows/rust-*.yml` must have a register entry (`scripts/check-poam-ids.sh`, run by `make policy`). Every advisory ignore must carry a POA&M ID and be identical in `deny.toml` and `.cargo/audit.toml` (`scripts/check-advisory-sync.sh`). Since Phase 1a fix round 2 (NIST-010), every `POAM:` marker comment must cite a `POAM-YYYY-MM-DD-NN` ID (`scripts/check-poam-markers.sh`). Since the Phase 1a WU10 follow-up it is in `POLICY_CHECKS` ([Makefile](../../Makefile) lines 26–28), so `make policy` runs it, as does `make check-annotations-strict` |
| Evidence | Phase 1a: [POAM.md](POAM.md) now has 13 open entries, 0 closed (POAM-2026-10-27-01 to -03 added). Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make policy` PASS with `check-poam-ids` 10/10 and `check-advisory-sync` PASS. Phase 1a fix round 2 (release-engineer, rule 6): `make policy-selftest` 419 passed, 0 failed, including 17 `check-poam-markers` cases and a ratchet case that fails once the tree passes the rule but `POLICY_CHECKS` lacks it; `make check-poam-markers` exits 2 (script exit 1) on exactly the two bare markers, `crates/tak-cot/src/limits.rs:18` and `crates/tak-cot/src/xml/serialize.rs:45`, and 11 of the 13 markers it finds carry an ID. The security-reviewer and the nist-800-53-reviewer re-ran it with the same result. Phase 1a WU10 follow-up, 2026-10-28 (compliance-scribe's local runs after the orchestrator added the IDs, not QA evidence, rule 6): `make policy` exit 0 with `check-poam-markers: ok (15 marker(s), each with a POA&M ID)`, i.e. 15/15 (14 markers in code, CI and Dependabot files plus the `POAM-YYYY-MM-DD-NN` format example at `.cargo/audit.toml` line 21), and `check-poam-ids: ok (13 referenced ID(s), 13 entr(ies) in POAM.md)`; `make check-annotations-strict` exit 0. Phase 0: [POAM.md](POAM.md) (10 open entries, 0 closed). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make policy` PASS (`check-poam-ids`: 9 IDs referenced, 9 entries at that time); `make check-advisory-sync` PASS (2 IDs, each with a POA&M); `make policy-selftest` PASS (199 passed, 0 failed). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `policy` job `check-poam-ids: ok (10 referenced ID(s), 10 entr(ies) in POAM.md)`; `supply-chain` job `check-advisory-sync: ok (2 ignored advisory ID(s), lists identical, each with a POA&M)` |
| Gaps | Risk ratings are proposed, not confirmed by the ISSO. No entry is accepted by the authorizing official. POAM-2026-10-26-09 is accepted for development by the system owner (2026-10-27), pending AO review; it stays Open. The organizational POA&M process (PM-4) and the update frequency are Inherited, TBD (ATO boundary). Both checks ran in CI on PR 7; they are not yet a required check (POAM-2026-10-26-04). Phase 1a: `check-poam-markers` is now required through `make policy`, but no QA or CI run has exercised it (rule 7); only the fixers', reviewers' and compliance-scribe's local runs exist. It scans only `*.rs *.sh *.toml *.yml *.yaml Makefile` (NIST-013), so for example `parity/pins.env` and the POAM-2026-10-27-01 marker in `fuzz/native-allowlist.txt` are not checked; `check-poam-ids` still sees every referenced ID |
| POA&M | — |
| Phase | 0; 1a (`check-poam-markers`) |
| Annotation locations | [scripts/check-poam-ids.sh](../../scripts/check-poam-ids.sh), [scripts/check-advisory-sync.sh](../../scripts/check-advisory-sync.sh) and [scripts/check-poam-markers.sh](../../scripts/check-poam-markers.sh) (line 26) header comments |

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
| Phase 1a target | Planned until a nightly run is recorded (rule 2) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Pull-request workflow, `rust-ci-gate` required check, `/tak-rs-workflow` run records, `CHANGELOG.md`, [CONTRIBUTING.md](../../CONTRIBUTING.md). Phase 1a: the `tak-proto` build script fails closed when the in-place schema set or any schema's content changes without a reviewed update of [crates/tak-proto/schema.sha256](../../crates/tak-proto/schema.sha256) (see SA-10(1)); the change detector runs the Rust jobs for `src/takserver-protobuf/` changes; `check-fuzz-policy.sh` rule F9 requires every `rust_inputs` prefix in `ci-changes.sh` to be in the `rust-fuzz.yml` pull-request paths |
| Evidence | Present: [CONTRIBUTING.md](../../CONTRIBUTING.md), [CHANGELOG.md](../../CHANGELOG.md), [rust-ci.yml](../../../.github/workflows/rust-ci.yml). Local helpers pass (QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3)): `ci-changes.sh` allow-list and `ci-gate.sh` selftests within `make policy-selftest` (199/0); detector linear on every upstream merge (12335 paths in 0.06 s). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `changes` gave `rust=true`; `rust-ci-gate` printed `ci-gate: PASS (rust=true)`. Earlier PR 7 heads `77beace3` and `0c56b62a` failed the `policy` job and `rust-ci-gate` went red (runs 40525846385, 40536002508) |
| Gaps | `rust-ci-gate` ran (green on PR 7, red on two earlier heads) but is not a required check: PR 7 was merged with all checks green by the user's decision, with nothing enforcing it. No nightly run, so the status stays Planned (rule 2). No `src/**`-only PR has tested the `rust=false` path end to end. Branch ruleset (required check, required review, code-owner review) not configured; no `CODEOWNERS`; repository Actions policy allows any action and does not require SHA pinning (POAM-2026-10-26-04). QA-001 (change detector super-linear: 16 s on 170 paths, more than 300 s on 668) was found and fixed in this run; QA-002 and QA-006 showed that the line-based `check-workflows` rules could be bypassed (QA-002 fixed; QA-006 fixed in fix round 2 by parsing the YAML, verified at QA Round 2). Since fix round 2 the detector also runs the Rust jobs for `src/takserver-protobuf/` changes (SEC-008), so 9 of the 14 upstream merges would now run them (QA Round 2 confirmed 9 of 14, each explained by a `src/takserver-protobuf/` path). QA-008 (`..` segments after a covered prefix pass `check-src-coupling` rule (r)) was open at QA Round 2; fix round 3 addresses it, not QA-verified. Phase 1a: no CI run exists for this branch (rule 7), so neither the schema-change path nor F9 has run in CI; the schema digest check was exercised only by fix-round runs (rule 6); nothing after `04774cc0` is committed. No nightly corpus re-capture exists yet (WU7 NOT RUN) |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 |
| Annotation locations | [scripts/ci-changes.sh](../../scripts/ci-changes.sh), [scripts/ci-gate.sh](../../scripts/ci-gate.sh), [scripts/check-src-coupling.sh](../../scripts/check-src-coupling.sh) (added in fix round 2), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments; Phase 1a: [crates/tak-proto/build.rs](../../crates/tak-proto/build.rs) (line 50), [scripts/check-fuzz-policy.sh](../../scripts/check-fuzz-policy.sh) (line 48) |

### CM-3(2) Testing, Validation, and Documentation of Changes

| Field | Value |
|-------|-------|
| Control | CM-3(2) Testing, Validation, and Documentation of Changes |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Partially Implemented (gate not yet required; POAM-2026-10-26-04) |
| Phase 1a target | Planned until a nightly run is recorded (rule 2) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | CI gates before merge; run records (`plan.md`, `qa-report.md`, findings, `gate.md`). Phase 1a: `rust-fuzz.yml` runs on pull requests that touch `tak-cot`, `tak-proto`, the fuzz workspace or `src/takserver-protobuf/**` (the schemas `tak-proto` compiles in place; NIST-009), and `check-fuzz-policy.sh` F9 keeps that path list in step with the change detector (NIST-012) |
| Evidence | Present: [rust-ci.yml](../../../.github/workflows/rust-ci.yml). First local run record: [TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3. Pre-merge CI gates: PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): 13 of 13 jobs `success`, `ci-gate: PASS (rust=true)`; the two earlier failing heads were fixed before the merge. Phase 1a: present, not run in CI: [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) `pull_request.paths`; F9 has 9 self-test cases in `make policy-selftest` (419/0, fix round 2, release-engineer; rule 6) |
| Gaps | `rust-ci-gate` ran on PR 7 but is not a required check, so testing before merge is not enforced (POAM-2026-10-26-04). No nightly run, so the status stays Planned (rule 2). QA-001 (super-linear change detector) fixed in this run. QA-002 (job-level `continue-on-error` and permission escalation not caught) fixed; QA-006 (quoted or spaced keys bypass the same rules) fixed in fix round 2 and verified at QA Round 2. Phase 1a: no CI run for this branch (rule 7); `rust-fuzz.yml` is never a required check; no nightly corpus re-capture |
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
| Phase 1a target | Partially Implemented (library defaults; NIST-006) |
| Scope | Development environment / CI; for the Phase 1a limits, TOE library component (`tak-cot`, `tak-proto`) |
| Implementation | Workspace lints (`unsafe_code = "forbid"`, strict clippy); `.cargo/config.toml` build pins `AWS_LC_FIPS_SYS_USE_SYSTEM=0`, `GOTOOLCHAIN=local`, `GOPROXY=off` (all `force = true`); least-privilege CI token (`permissions: contents: read`). Phase 1a: secure library defaults for the input bounds, documented in rustdoc and the Phase 1a plan §3.4: `tak_cot::Limits::default()` (8 MiB input, depth 64, 100 000 nodes, 256 attributes and 32 namespace declarations per element, 1 024-byte names, 1 MiB attribute values, text bounded by the input, `xmlDetail` 8 MiB) and `tak_proto::MAX_MESSAGE_BYTES_DEFAULT` (8 MiB). Since fix round 1 the proto `xmlDetail` cap is exactly `max_xml_detail_bytes` (QA-002). They are library defaults only: no CoreConfig key, no operator setting and no ceiling until Phase 3 |
| Evidence | [Cargo.toml](../../Cargo.toml) `[workspace.lints.rust]` (line 32; `unsafe_code = "forbid"` at line 35) and `[workspace.lints.clippy]` (line 42), [.cargo/config.toml](../../.cargo/config.toml), top-level `permissions: contents: read` in each `.github/workflows/rust-*.yml`. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make clippy` and `make clippy-wasm` PASS (0 warnings with `-D warnings`); `make policy` PASS (`check-manifests`: 2 members with `[lints] workspace = true`; `check-unsafe`: 0 uses); `grep -L 'contents: read'` over the three workflows prints nothing, so each contains `contents: read` (grep alone does not show the setting is top-level). QA Round 2, 2026-10-26, local (same section): the same gates PASS; `make policy` `check-workflows` ok on 3 parsed files, whose rule W3 requires top-level `permissions` to be exactly `contents: read` and rule W3b allows job-level `permissions` only absent or exactly `contents: read`; the QA-006 permission mutations (quoted key, spaced key, `write-all`, added `id-token: write`) each fail. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): the runner logged `GITHUB_TOKEN Permissions` as `Contents: read`, `Metadata: read` for all 13 `rust-ci` jobs and the `rust-parity` job. Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): every default limit accepted at its value and refused one past it with a typed code (QA §3 table); at QA time the effective `xmlDetail` cap was 8 MiB − 55 bytes (QA-002). Phase 1a fix round 1 (rule 6): `p2x_xml_detail_cap_is_exact_at_default_limits` and `p2x_xml_detail_cap_does_not_depend_on_max_input_bytes` in `crates/tak-cot/tests/qa_boundaries.rs` pass in `make test` 223/0; re-verified by the nist-800-53-reviewer (NIST-006) |
| Gaps | No product configuration settings yet (CoreConfig model in Phase 1b `tak-config`). Phase 1a limits have no operator setting and no upper bound until Phase 3; the exact `xmlDetail` cap is not QA re-verified. The `.cargo/config.toml` FIPS build pins are inert until Phase 2, so no gate exercises them. The CI-token setting takes effect only in CI; it was observed in the PR 7 runs. At QA Round 1 the `check-workflows` permission rules were line-based and could be bypassed (QA-006); the parsed-YAML rules are verified at QA Round 2. Fix round 3 adds `checks: write`, `statuses: write` and `write-all` rejection in every workflow file (SEC-010); not QA-verified. The parity fixture containers do not set a read-only root filesystem and use non-UBI bases (open POA&M POAM-2026-10-26-03) |
| POA&M | POAM-2026-10-26-03 (parity fixture containers) |
| Phase | 0 (build settings); 1a (library defaults); 1b (CoreConfig model); 3 (operator settings and ceilings) |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml), [Cargo.toml](../../Cargo.toml), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml) header comments; Phase 1a: [crates/tak-cot/src/limits.rs](../../crates/tak-cot/src/limits.rs) (line 13) |

### CM-7 Least Functionality

| Field | Value |
|-------|-------|
| Control | CM-7 Least Functionality |
| 800-53B baseline | L M H |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | Planned |
| Phase 1a target | Planned (plan §7.1: noted, not raised) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | None in Phase 0. Pre-mapped because rustls 0.23 `fips` also builds the non-FIPS `aws-lc-sys`, which would be linked but unused code. Phase 1a: the libFuzzer native code is confined to the excluded `rust/fuzz/` workspace (own lockfile, `deny.toml`, native allow-list), and `scripts/check-fuzz-policy.sh` (in `make policy`) fails if `libfuzzer-sys` or `cc` reaches the product graph; `tak-proto` generates message types only (no gRPC service stubs), and neither Phase 1a crate does I/O |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make policy` PASS including `check-fuzz-policy` (the product-graph exclusion); `make fuzz-deny` PASS (cargo-deny with `fuzz/deny.toml`, cargo audit on `fuzz/Cargo.lock`, native allow-list; it printed `libfuzzer-sys (native) allow-listed for aarch64-apple-darwin, not in graph`). Recorded as evidence only; the status stays at its target |
| Gaps | Linked-but-unused `aws-lc-sys` from Phase 2. Yew `csr` brings all of `gloo` (81 wasm32 crates), including `gloo-worker` and `gloo-file`, which create `blob:` URLs; nothing uses them today, and the Phase 4 plan keeps `blob:` out of the CSP (WEB-005). Phase 1a: the development-only libFuzzer exception (open POA&M POAM-2026-10-27-01) |
| POA&M | POAM-2026-10-26-08, POAM-2026-10-27-01 |
| Phase | 2 |
| Annotation locations | Phase 1a: [fuzz/Cargo.toml](../../fuzz/Cargo.toml) (line 13), [fuzz/deny.toml](../../fuzz/deny.toml) (line 18), [scripts/check-fuzz-policy.sh](../../scripts/check-fuzz-policy.sh) (line 48) header comments. Fix round 3 of Phase 0 removed CM-7 from the [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh) header, which cites SR-3 and SA-15 (NIST-019) |

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
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Apache-2.0 workspace licence; cargo-deny licence allow-list without copyleft (denied by absence); OpenSSL exception scoped to `aws-lc-fips-sys` with NOTICE attribution; provenance rule against the GPLv3 Java reference. Phase 1a: the `.proto` schemas are read in place from `src/` at build time and never copied into `rust/`; corpus inputs are synthetic and Apache-2.0 ([parity/corpus/MANIFEST.tsv](../../parity/corpus/MANIFEST.tsv) line 2); the corpus driver is Apache-2.0; the fuzz workspace allows the NCSA licence for `libfuzzer-sys` only |
| Evidence | [Cargo.toml](../../Cargo.toml) `license = "Apache-2.0"`, [LICENSE](../../LICENSE), [NOTICE](../../NOTICE), [deny.toml](../../deny.toml) `[licenses]`, [CONTRIBUTING.md](../../CONTRIBUTING.md) §Licence and §Provenance. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): member licence set is exactly `Apache-2.0` PASS; `LICENSE` is the Apache text, `NOTICE` has the OpenSSL attribution, CONTRIBUTING has §Provenance (PASS); no GPL, AGPL or MPL in the locked graph (PASS); `make deny` `licenses ok`; `make policy` `check-manifests` ok. Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `licenses ok` on the Phase 1a graph; `make fuzz-deny` PASS; `git status --porcelain src/` empty (nothing in `src/` changed). The tak-protocol-reviewer found no Java comments, identifiers or structure carried over (provenance check, D11) |
| Gaps | Derivative-work and schema-reuse questions not reviewed by counsel. Per decision Q9 (2026-10-27) Phase 1a merges without waiting for the opinion; the build-time schema read, the Java-generated goldens (none committed yet, WU7 NOT RUN) and the Apache-2.0 Java driver linked against the GPLv3 jars at capture time are added to POAM-2026-10-26-05's scope |
| POA&M | POAM-2026-10-26-05 |
| Phase | 0; legal opinion before the first release (Phase 3) |
| Annotation locations | [Cargo.toml](../../Cargo.toml), [deny.toml](../../deny.toml), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml) header comments; Phase 1a: [fuzz/deny.toml](../../fuzz/deny.toml) (line 18) |

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
(SA-10, SA-11, SA-11(1), SA-11(8), SA-15, SA-22). SA-10(1) is Planned.

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
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | git, `Cargo.lock`, `CHANGELOG.md`, Dependabot, SPDX header and provenance rules. Phase 1a: the schema digest manifest [crates/tak-proto/schema.sha256](../../crates/tak-proto/schema.sha256), checked fail-closed by the `tak-proto` build script (SA-10(1)); corpus inputs under version control with one [MANIFEST.tsv](../../parity/corpus/MANIFEST.tsv) row per (id, direction); a separate lockfile for the fuzz workspace ([fuzz/Cargo.lock](../../fuzz/Cargo.lock)) |
| Evidence | [Cargo.lock](../../Cargo.lock), [CHANGELOG.md](../../CHANGELOG.md), [CONTRIBUTING.md](../../CONTRIBUTING.md), [POAM.md](POAM.md) (flaw tracking). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `cargo metadata --locked` PASS and every `make` cargo target runs `--locked`; `make policy` PASS (`check-annotations` SPDX rule (c) and mapping-block rules ok); `make deny` `sources ok`. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): the cargo jobs ran `--locked`; `make policy` (SPDX and mapping-block rules) and `make deny` (`sources ok`) PASS. Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `sources ok`; `make policy` PASS (SPDX rule, mapping-block rules, 50 blocks); no generated code committed (`git ls-files rust \| grep -E 'tak_proto_includes\|atakmap\.commoncommo'` empty) |
| Gaps | Change approval not enforced by a ruleset (POAM-2026-10-26-04): PR 7 was merged to `main` as `9f754245` with green checks but no required check or review. Dependabot has not run. Phase 1a: nothing after `04774cc0` is committed (commit signing temporarily unavailable), so the WU3–WU5 and WU9 work, both fix rounds and `schema.sha256` exist only in the working tree; `schema.sha256` must be committed together with `build.rs`. No goldens and no `PROVENANCE.json` exist yet (WU7 NOT RUN) |
| POA&M | POAM-2026-10-26-04 |
| Phase | 0 |
| Annotation locations | [deny.toml](../../deny.toml), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml) header comments |

### SA-10(1) Software and Firmware Integrity Verification

New in Phase 1a.

| Field | Value |
|-------|-------|
| Control | SA-10(1) Software and Firmware Integrity Verification |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Planned |
| Phase 0 target | — (block added in Phase 1a) |
| Phase 1a target | Partially Implemented (nist-800-53-reviewer proposal, fix round 2; not in plan §7.1) |
| Scope | Build host (developer configuration management), outside the TOE |
| Implementation | The `tak-proto` build script checks the 15 `.proto` schemas it compiles in place from `src/takserver-protobuf/` against the committed manifest [crates/tak-proto/schema.sha256](../../crates/tak-proto/schema.sha256) before compiling them. The build fails if the manifest is unreadable or malformed, if it does not list exactly the build script's `SCHEMAS`, or if any SHA-256 digest differs (with a CRLF hint when the file has CRLF line endings). It also fails if any file in the compiled descriptor set is neither in `SCHEMAS` nor one of the 12 bundled `google/protobuf/*.proto` names, or if a local file shadows a bundled one. The schema directory scan is recursive, matches the extension case-insensitively, does not follow symlinks and fails closed past depth 8. SHA-256 is written in the build script (FIPS 180-4) and self-tested against the standard's test vectors before use. It is a change detector over files already in the repository, unkeyed and linked into no artifact: not an SC-13 cryptographic function and not an SI-7 mechanism (CRY-001; [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md)) |
| Evidence | Phase 1a fix round 1 (rust-coder, rule 6): a tampered digest and a truncated manifest each fail the build (exit 101) with the expected message; manifest restored. Phase 1a fix round 2 (rust-coder, scratch copy outside the repository, rule 6): cases A to F, each with the manifest digest recomputed so only the new check could fire: nested import in a real subdirectory, the same through a symlink, a well-known import shadowed by a symlinked local file, CRLF checkout and an `EXTRA.PROTO` file each exit 101 with the expected message; a bundled well-known import exits 0. crypto-tls-reviewer: in its first review (`findings-crypto.md`) `shasum` of the 15 schemas equals `schema.sha256` and a differential test against Python `hashlib` matched on 1102 of 1102 inputs; in its re-verification (`findings-crypto-round2.md`) `cargo build -p tak-proto --offline` exit 0. None of these is a QA or CI run |
| Gaps | No QA run and no CI run has exercised the check (rules 6 and 7); the Phase 1a QA gates ran before it existed. The manifest is protected only by review: whoever can change a schema can change the manifest, and change approval is not enforced (POAM-2026-10-26-04). No signature. The build-script checks are proven by one manual scratch run only (CRY-004). `schema.sha256` is untracked until committed with `build.rs`. If the check is ever relied on against a malicious upstream, or reused in product code, it moves to the aws-lc-rs FIPS provider |
| POA&M | POAM-2026-10-26-04 |
| Phase | 1a |
| Annotation locations | [crates/tak-proto/build.rs](../../crates/tak-proto/build.rs) (line 50) |

### SA-11 Developer Testing and Evaluation

| Field | Value |
|-------|-------|
| Control | SA-11 Developer Testing and Evaluation |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented |
| Phase 1a target | Partially Implemented (already; evidence added) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | CI `fmt`, `clippy` (native and wasm32), `test`, `doc`; policy self-tests; parity harness v0 running TAKCL `StartupTests` against the Java reference. Phase 1a: unit, negative, property (proptest, fixed seeds) and integration tests for `tak-proto` and `tak-cot`; golden-corpus inputs and an Apache-2.0 Java capture driver (`harness.sh capture-corpus`, `verify-corpus`, `import-corpus`); the golden test itself (WU8) is not written yet |
| Evidence | [Makefile](../../Makefile), `rust/scripts/`, [parity/harness.sh](../../parity/harness.sh). QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 5 passed, 0 failed; `make test-doc` 2 passed; `make fmt-check`, `make clippy`, `make clippy-wasm`, `make build-wasm`, `make doc` PASS; `make policy-selftest` 199 passed, 0 failed; `parity/harness.sh selftest` 39 passed, 0 failed; harness integrity negative (wrong wrapper-jar pin) exits 4 before any engine step. QA Round 2, 2026-10-26, local (same section): the same gates PASS with the same test counts; `make policy-selftest` 269/0 before and 281/0 after 12 QA cases; harness selftest 44/0. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 5 passed, 0 failed; doc tests 2 passed; `fmt`, `lint (native)`, `lint (wasm32)`, `doc` PASS; `make policy-selftest` 331 passed, 0 failed; `ci-gate: PASS (rust=true)`. [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328): harness selftest 44/0; TAKCL `StartupTests` against the Java reference PASS (`verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0`). Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 220 passed, 0 failed, 0 ignored in 19 test binaries (198 before 22 QA tests were added; property tests 512 fixed-seed cases each); `make test-doc` 2 passed; `make fmt-check`, `make clippy`, `make clippy-wasm`, `make build-wasm` (now including `tak-proto` and `tak-cot`), `make doc` PASS; `make policy-selftest` 381 passed, 0 failed; `parity/harness.sh selftest` 97 passed, 0 failed; shellcheck on 15 files PASS. Phase 1a fix round 1 (rule 6): `make test` 223 passed, 0 failed, 0 ignored; `make policy-selftest` 392/0. Phase 1a fix round 2 (rule 6): `make test` exit 0 (count not reported); `make policy-selftest` 419/0 |
| Gaps | Phase 1a: the golden parity tests, which are the Phase 1a exit criterion, are NOT RUN: GitHub Actions is disabled, so the corpus capture (WU7) could not run and the golden test (WU8) was not written (rule 7). No CI run exists for this branch. No performance regression guard: tests cannot time themselves (`Instant::now` is banned for wasm32) and fuzzing stops at 64 KiB (QA-004). QA-007 (rule (d) of `check-annotations.sh` treated every `Not Applicable (component)` row as mapped) was open at QA Round 2; fix round 3 addresses it, not QA-verified (its self-tests pass in PR 7 CI). TAKCL `StartupTests` against the Java reference ran once (PR 7); it is a Java baseline, not a comparison. No nightly run yet. No wasm runtime tests, and CI builds wasm32 in dev mode only with no bundle-size baseline; the Phase 4 plan adds a headless `wasm-bindgen-test` job, a release wasm build with `--all-features` and a size report (WEB-003, WEB-006). TAKCL cannot yet target the Rust server (POAM-2026-10-26-06) |
| POA&M | POAM-2026-10-26-06 |
| Phase | 0 |
| Annotation locations | [Makefile](../../Makefile), [scripts/check-unsafe.sh](../../scripts/check-unsafe.sh), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [scripts/ci-changes.sh](../../scripts/ci-changes.sh), [scripts/ci-gate.sh](../../scripts/ci-gate.sh), [scripts/check-src-coupling.sh](../../scripts/check-src-coupling.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments; Phase 1a: [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) (line 33) |

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
| Status | Partially Implemented |
| Phase 0 target | Planned (fuzzing, Phase 1) |
| Phase 1a target | Partially Implemented after a recorded fuzz run |
| Scope | Development environment (the excluded `rust/fuzz/` workspace), exercising the TOE library components `tak-proto` and `tak-cot` |
| Implementation | Seven cargo-fuzz (libFuzzer) targets: `cot_parse`, `cot_to_proto`, `proto_decode`, `proto_to_cot`, `proto_roundtrip`, `stream_frame`, `javafmt`. They live in the excluded `rust/fuzz/` workspace (own `Cargo.lock`, `deny.toml`, native allow-list, toolchain pinned to `nightly-2026-10-23`) and run under AddressSanitizer with debug assertions, `-timeout=1`, `-rss_limit_mb=2048` and `-max_len=65536`, seeded from the corpus inputs (`scripts/fuzz-seeds.sh`). [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) runs them for 60 s on pull requests, 600 s nightly, and for the `max_total_time` input (default 300 s) on dispatch; it is never a required check. Property tests (proptest, 512 fixed-seed cases each) run in `make test` |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3, "Fuzz runs"): `make fuzz-build` PASS (7 binaries); `make fuzz-seeds` PASS; each target ran 300 s from 2026-10-28T06:11:01Z, all seven exit 0 with 0 crashes, 0 timeouts and 0 OOMs, and `fuzz/artifacts/` stayed empty; 125 419 498 executions in total; peak RSS 557 to 818 MB against the 2048 MB limit. `make fuzz-deny`, fuzz `cargo fmt --check` and `cargo clippy -D warnings` PASS. Phase 1a fix round 1 (release-engineer, rule 6): two concurrent `make fuzz-run` calls (20 s each) exit 0 and each regenerated only its own seeds (QA-003 fix) |
| Gaps | No CI or nightly fuzz run is recorded: GitHub Actions is disabled (rules 2 and 7). Inputs are capped at 64 KiB, so size-dependent cost is not explored; QA-001 was missed this way (QA-004). libFuzzer is C++ built by `libfuzzer-sys`, a development-only native-code exception (open POA&M POAM-2026-10-27-01). `cargo-fuzz`'s own dependency graph is installed with `--locked` but not checked by cargo-deny. Fuzzing finds crashes, not differences from the Java reference |
| POA&M | POAM-2026-10-27-01 |
| Phase | 1a |
| Annotation locations | The seven targets under [fuzz/fuzz_targets/](../../fuzz/fuzz_targets/) (`cot_parse.rs`, `proto_decode.rs`, `proto_roundtrip.rs`, `proto_to_cot.rs` line 14; `cot_to_proto.rs` line 16; `javafmt.rs` line 18; `stream_frame.rs` line 19); [fuzz/Cargo.toml](../../fuzz/Cargo.toml) (line 13); [fuzz/deny.toml](../../fuzz/deny.toml) (line 18); [scripts/fuzz-seeds.sh](../../scripts/fuzz-seeds.sh) (line 33); [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) (line 33) |

### SA-15 Development Process, Standards, and Tools

| Field | Value |
|-------|-------|
| Control | SA-15 Development Process, Standards, and Tools |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (SA-15b review cadence is organization-defined, TBD) |
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | Pinned toolchain, `Makefile` as the single entry point (CI runs the same targets), lint policy, checksum-verified CI tools (`taiki-e/install-action`, `fallback: none`), least-privilege CI token, [CONTRIBUTING.md](../../CONTRIBUTING.md). Phase 1a: pure-Rust schema compilation (`protox` + `prost-build`, no `protoc`); a separate fuzz workspace with a pinned dated nightly and `cargo-fuzz` 0.13.2; the POA&M-marker rule (`check-poam-markers.sh`) |
| Evidence | [rust-toolchain.toml](../../rust-toolchain.toml), [Cargo.toml](../../Cargo.toml), [rustfmt.toml](../../rustfmt.toml), [clippy.toml](../../clippy.toml), [Makefile](../../Makefile), [CONTRIBUTING.md](../../CONTRIBUTING.md); tool pins in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): toolchain 1.99.1 from the pin; every gate ran through its `make` target; `make fmt-check`, `make clippy`, `make policy` PASS. Checksum-verified tool installation in PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `taiki-e/install-action` logged "verifying sha256 checksum" for zizmor, shellcheck, cargo-cyclonedx, cargo-deny and cargo-audit; the actionlint tarball passed `sha256sum --check --strict` ("OK"); every job's token was `Contents: read`, `Metadata: read`. Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): toolchain 1.99.1 for the product, `nightly-2026-10-23` (rustc 1.101.0-nightly) and cargo-fuzz 0.13.2 for the fuzz workspace; every gate ran through its `make` target; `make policy` PASS including `check-toolchain` and `check-fuzz-policy` |
| Gaps | SA-15b review cadence not defined. Phase 1a: the fuzz toolchain is a nightly (dated and pinned, but not a stable release); its pins are in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory |
| POA&M | — |
| Phase | 0 |
| Annotation locations | [rust-toolchain.toml](../../rust-toolchain.toml), [Cargo.toml](../../Cargo.toml), [Makefile](../../Makefile), [scripts/check-annotations.sh](../../scripts/check-annotations.sh), [scripts/check-manifests.sh](../../scripts/check-manifests.sh), [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh), [scripts/check-toolchain.sh](../../scripts/check-toolchain.sh), [scripts/check-unsafe.sh](../../scripts/check-unsafe.sh), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml) header comments; Phase 1a: [crates/tak-proto/build.rs](../../crates/tak-proto/build.rs) (line 50), [fuzz/Cargo.toml](../../fuzz/Cargo.toml) (line 13), [fuzz/deny.toml](../../fuzz/deny.toml) (line 18), [scripts/check-fuzz-policy.sh](../../scripts/check-fuzz-policy.sh) (line 48), [scripts/fuzz-seeds.sh](../../scripts/fuzz-seeds.sh) (line 33), [scripts/check-poam-markers.sh](../../scripts/check-poam-markers.sh) (line 26) |

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

Origination: System-specific. Status: Partially Implemented (SC-5 only, TOE
library component); the rest Planned. Phase 0 and Phase 1a contain no crypto
code in the product and no crypto dependency.

| Control | Baseline | Origination | Status | Phase | Note |
|---------|----------|-------------|--------|-------|------|
| SC-8 Transmission Confidentiality and Integrity | M H | System-specific | Planned | 3 | TLS 1.2+ on every input |
| SC-12 Cryptographic Key Establishment and Management | L M H | Hybrid | Planned | 2, 4 | Keystores and key handling in `tak-crypto` (Phase 2); certificate enrollment (Phase 4); key custody procedures organizational |
| SC-17 Public Key Infrastructure Certificates | M H | Hybrid | Planned | 2, 4 | Certificate validation and CRL in `tak-auth` (Phase 2); TAK CA operations (Phase 4); the issuing PKI is organizational |
| SC-23 Session Authenticity | M H | System-specific | Planned | 4 | Admin sessions |
| SC-28 Protection of Information at Rest | M H | Hybrid | Planned | 2 | Keystores and secrets; disk encryption organizational |

### SC-5 Denial-of-service Protection

New in Phase 1a.

| Field | Value |
|-------|-------|
| Control | SC-5 Denial-of-service Protection |
| 800-53B baseline | L M H |
| Origination | Hybrid (per-message resource bounds are TAK-RS; network-level and boundary protection is the hosting organization's, Inherited, TBD (ATO boundary)) |
| Status | Partially Implemented |
| Phase 0 target | — (block added in Phase 1a) |
| Phase 1a target | Partially Implemented (component; rate limiting Phase 3/7) |
| Scope | TOE library component (`tak-proto`, `tak-cot`); no network wiring until Phase 3 |
| Implementation | Every message is bounded before it can cost memory or CPU, and a breach fails closed with a typed error. `tak_cot::Limits` defaults (CM-6) are checked by the streaming parser as it reads: 8 MiB input, depth 64, 100 000 nodes, 256 attributes and 32 namespace declarations per element, 1 024-byte names, 1 MiB attribute values, text bounded by the input. A proto `xmlDetail` longer than 8 MiB is refused before parsing, and its parsed copy counts against the event's depth and node limits. `tak_proto` refuses a TakMessage or stream frame over `MAX_MESSAGE_BYTES_DEFAULT` (8 MiB); the frame length is checked before the payload arrives, and the varint length prefix is limited to 5 bytes and the u32 range. prost's recursion limit (100) bounds nested groups. Parsing, copying and serializing use no recursion. CPU: since fix round 1 the proto-to-XML `xmlDetail` merge is linear in the element count (`merge_xml_detail` with one queue per typed name; QA-001, SEC-001, NIST-001), and `merge_xml_detail` drops its intermediate copies early (SEC-002). The X2P `xmlDetail` output is capped (`write_subtree_bounded`) |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): every limit accepted at its value and refused one past it with a typed code (`crates/tak-cot/tests/xml_limits.rs`, `qa_boundaries.rs`, `crates/tak-proto/tests/decode_hostile.rs`, `framing.rs` unit tests, `schema.rs`), in `make test` 220/0; a million-deep XML document, `xmlDetail` and protobuf group nesting are each refused without stack overflow; fuzz 7 × 300 s with 0 crashes, timeouts or OOMs and peak RSS 557 to 818 MB of the 2048 MB limit. CPU bound, Phase 1a fix round 1 (rust-coder, rule 6): `convert::tests::xml_detail_merge_is_linear_in_the_element_count` ([crates/tak-cot/src/convert.rs](../../crates/tak-cot/src/convert.rs) line 701) asserts at most 8·n child steps at n = 1 000 and 4 000 through a test-only step counter, and fails (1 002 999 steps at n = 1 000) with the old scan restored; the QA reproducer (49 000 non-typed then 49 000 `contact` elements, 686 000 bytes) took 4.31 s before and 15.0 ms after in a release build; `make test` 223/0. Re-verified by the nist-800-53-reviewer (NIST-001), the security-reviewer (SEC-001) and the tak-protocol-reviewer (output unchanged, 20 000-case differential test). Not re-run by QA |
| Gaps | No rate limiting or connection limits until Phase 3 (connections) and Phase 7 (QoS). No process-wide memory budget: one conversion can hold about 23 times its input (measured 193.6 MB through `from_tak_message`), and nothing caps the conversions in flight (open POA&M POAM-2026-10-27-02). `write_document` and `write_subtree` output is unbounded, and namespace re-declaration in fragments amplifies it (open POA&M POAM-2026-10-27-03). The limits have no ceiling and no operator setting until Phase 3 (CM-6). Fuzzing is capped at 64 KiB and tests cannot time themselves, so a new size-dependent CPU path would go unnoticed (QA-004). The CPU fix is not QA re-verified (rule 6). Network-level denial-of-service protection is the hosting organization's |
| POA&M | POAM-2026-10-27-02, POAM-2026-10-27-03 |
| Phase | 1a (per-message bounds); 3 (ingest budget, connection limits); 7 (QoS and rate limiting) |
| Annotation locations | [crates/tak-cot/src/lib.rs](../../crates/tak-cot/src/lib.rs) (line 23), [limits.rs](../../crates/tak-cot/src/limits.rs) (13), [event.rs](../../crates/tak-cot/src/event.rs) (16), [convert.rs](../../crates/tak-cot/src/convert.rs) (21, 216, 364, 572), [xml/mod.rs](../../crates/tak-cot/src/xml/mod.rs) (20), [xml/parse.rs](../../crates/tak-cot/src/xml/parse.rs) (23), [xml/document.rs](../../crates/tak-cot/src/xml/document.rs) (15), [xml/scope.rs](../../crates/tak-cot/src/xml/scope.rs) (17), [xml/serialize.rs](../../crates/tak-cot/src/xml/serialize.rs) (80, `write_subtree_bounded` only); [crates/tak-proto/src/lib.rs](../../crates/tak-proto/src/lib.rs) (22, 87), [framing.rs](../../crates/tak-proto/src/framing.rs) (18, 98): 16 sites. The fuzz targets do not cite SC-5; fuzzing is evidence here |

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
| Gaps | No crypto in Phase 0 or in the Phase 1a crates. The SHA-256 in the `tak-proto` build script is a build-host change detector (SA-10(1)), not an SC-13 function (CRY-001; [FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md)). rustls `fips` also builds non-FIPS `aws-lc-sys`; the FIPS graph will fail `[bans.build]` on two `libloading` test DLLs until Phase 2 adds a checksum-pinned bypass ([FIPS_COMPLIANCE.md](FIPS_COMPLIANCE.md) §Known issues, QA-004) |
| POA&M | POAM-2026-10-26-08 |
| Phase | 2 (`tak-crypto`) |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml) header comment (pre-positioned settings; the file says they are inert until Phase 2) |

## SI System and Information Integrity

Origination: Hybrid. Status: Partially Implemented (SI-10, SI-10(3) and
SI-11, TOE library component); the rest Planned. SI-10 and SI-11 were table
rows until Phase 1a and are now blocks below.

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
| Implementation | None for the TOE. CI-tool integrity (checksums, SHA pins) is mapped to SR-11 and SA-15, not to TOE SI-7. The Phase 1a schema digest in the `tak-proto` build script is SA-10(1) (build host), not SI-7 |
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

### SI-10 Information Input Validation

Row until Phase 1a; block since Phase 1a.

| Field | Value |
|-------|-------|
| Control | SI-10 Information Input Validation |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Planned (Phase 1) |
| Phase 1a target | Partially Implemented (library; network wiring Phase 3) |
| Scope | TOE library component (`tak-proto`, `tak-cot`); no network wiring until Phase 3 |
| Implementation | `tak-cot` bounded streaming XML parser: DOCTYPE refused, so no DTD, no external entities and no entity expansion; only the predefined entities and character references; XML 1.0 `Char` and `Name` checks; namespace well-formedness (an undeclared prefix is refused); duplicate attributes, mismatched end tags, multiple roots and text after the root refused; `version="1.1"` refused (DEV-01b); the SC-5 limits. A proto `xmlDetail` is parsed again under the same rules inside a `<detail>` wrapper, so a stray `</detail>` or a second root cannot break out, and its copy counts against the event's limits. `tak-proto`: size cap before decoding, proto3 UTF-8 check on every `string` field, a known field with the wrong wire type refused (DEV-04), stream-frame magic, varint and length checked before the payload, mesh version check. `javafmt`: the Java grammars for numbers and CoT times. The serializer has a supporting role: its output is well-formed and every value is escaped, so nothing a sender wrote can break out of its element at a receiver |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make test` 220 passed, 0 failed, including `crates/tak-cot/tests/xml_wellformed.rs`, `xml_limits.rs`, `xml_props.rs`, `convert.rs`, `convert_props.rs`, `javafmt.rs`, `qa_boundaries.rs` (11 tests, among them `p2x_xml_detail_hostile_constructs_rejected`: billion laughs and a SYSTEM entity, an undefined entity, a `</detail><event/><detail>` break-out, a stray `</detail>`, an inner XML declaration, control characters, `&#0;`, duplicate attributes and undeclared prefixes inside `xmlDetail`, each refused with its own code) and `crates/tak-proto/tests/decode_hostile.rs` (11 tests: hostile group nesting, length prefixes up to `u64::MAX` without allocation, over-long varints, the 8 MiB caps); property tests 512 fixed-seed cases each; fuzz 7 × 300 s, 0 crashes. Security review (`findings-security.md`, "Checked, no finding"): DOCTYPE and entities, `xmlDetail` escape attempts, serializer escaping, edit APIs refusing non-`Char` text and non-NCName names. Phase 1a fix round 2 (rule 6): `crates/tak-cot/tests/identity_accessors.rs` pins that `CotEvent::uid`/`type_`/`how` equal the converter's values on a duplicated local name (SEC-008) |
| Gaps | Not wired to any network input until Phase 3: stream splitting (D-01) and connection-level reactions (D-06) are Phase 3. No XSD validation (K-31, Phase 3 decision). Equivalence with the Java reference's accept and reject decisions is unproven: the golden tests are NOT RUN (rule 7). DEV-01c (mesh), DEV-01d, DEV-05 and DEV-07 (PROTO-001 option (b): P2X refuses proto strings that are not XML 1.0 characters) were approved by the user on 2026-10-28 ([PROTOCOL_PARITY.md](PROTOCOL_PARITY.md) Table C); they are pinned by Rust tests, but no golden yet shows them against the Java output (WU7, WU8 NOT RUN). An event carrying both `p:uid` and `uid` is accepted and both are kept on output (D-18, Phase 3 decision; SEC-003); the root element name is not checked (D-19; SEC-004). Fuzz inputs stop at 64 KiB |
| POA&M | — (memory per process: POAM-2026-10-27-02, under SC-5) |
| Phase | 1a (library); 3 (network wiring, XSD decision, D-18, D-19) |
| Annotation locations | `tak-cot`: [lib.rs](../../crates/tak-cot/src/lib.rs) (line 23), [error.rs](../../crates/tak-cot/src/error.rs) (7), [limits.rs](../../crates/tak-cot/src/limits.rs) (13), [event.rs](../../crates/tak-cot/src/event.rs) (16), [javafmt.rs](../../crates/tak-cot/src/javafmt.rs) (21), [convert.rs](../../crates/tak-cot/src/convert.rs) (21, 215, 363, 572), [xml/mod.rs](../../crates/tak-cot/src/xml/mod.rs) (19), [xml/parse.rs](../../crates/tak-cot/src/xml/parse.rs) (22), [xml/names.rs](../../crates/tak-cot/src/xml/names.rs) (13), [xml/document.rs](../../crates/tak-cot/src/xml/document.rs) (15), [xml/serialize.rs](../../crates/tak-cot/src/xml/serialize.rs) (42, supporting role). `tak-proto`: [lib.rs](../../crates/tak-proto/src/lib.rs) (22, 87), [framing.rs](../../crates/tak-proto/src/framing.rs) (18, 98, 165). 19 sites. `rust/fuzz/**` and `rust-fuzz.yml` do not cite SI-10; fuzzing is evidence here |

### SI-10(3) Predictable Behavior

New in Phase 1a.

| Field | Value |
|-------|-------|
| Control | SI-10(3) Predictable Behavior |
| 800-53B baseline | — |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | — (block added in Phase 1a) |
| Phase 1a target | Partially Implemented |
| Scope | TOE library component (`tak-proto`, `tak-cot`); no network wiring until Phase 3 |
| Implementation | On invalid input every entry point returns a typed error with a stable code (`XmlErrorKind`, `ConvertErrorKind`, `DecodeErrorKind`, `FrameError`) and no partial value: `Document::parse` and `CotEvent::parse` return no partial document, `to_tak_message` no partial TakMessage, `from_tak_message` no partial event; `append_copy` rolls back on failure. Input cannot cause a panic: `#![forbid(unsafe_code)]` in both crates, `clippy::arithmetic_side_effects` denied in `tak-cot`, checked arithmetic, no recursion on input |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): every rejection is a typed error and no case returns a partial document or event; each error case in `qa_boundaries.rs` asserts the kind it expects; `xml_limits.rs::append_copy_respects_limits_and_leaves_nothing_on_failure`; 10^6-deep XML, `xmlDetail` and protobuf group nesting refused without stack overflow; fuzz 7 × 300 s under AddressSanitizer with debug assertions (overflows and `debug_assert!` failures count as crashes), 0 crashes, timeouts or OOMs in 125 419 498 executions |
| Gaps | The behaviour is "refuse the whole message". What the server then does (drop, audit, close the connection) is Phase 3 (D-06), and the audit events are Planned (AU-2, AU-12). At QA time an `xmlDetail` just under 8 MiB was refused with `xml.limit.input_bytes` instead of `xml.limit.xml_detail_bytes` (QA-002); fix round 1 made the cap exact, not QA re-verified (rule 6). Fuzz inputs stop at 64 KiB. Agreement with the Java reference's ok/reject outcome is NOT RUN (rule 7) |
| POA&M | — |
| Phase | 1a |
| Annotation locations | [crates/tak-cot/src/convert.rs](../../crates/tak-cot/src/convert.rs) (lines 215, 363), [xml/mod.rs](../../crates/tak-cot/src/xml/mod.rs) (19), [xml/parse.rs](../../crates/tak-cot/src/xml/parse.rs) (22): 4 sites |

### SI-11 Error Handling

Row until Phase 1a; block since Phase 1a.

| Field | Value |
|-------|-------|
| Control | SI-11 Error Handling |
| 800-53B baseline | M H |
| Origination | System-specific |
| Status | Partially Implemented |
| Phase 0 target | Planned (Phases 3–4) |
| Phase 1a target | Partially Implemented |
| Scope | TOE library component (`tak-proto`, `tak-cot`); no network wiring until Phase 3 |
| Implementation | Error messages carry the error kind and a byte offset or a length, never input content. `Debug` of `CotEvent`, `Document`, `ElementRef` and `AttributeRef` prints no content (`AttributeRef` no longer prints the attribute name since fix round 1, SEC-006) |
| Evidence | Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): with the marker `Qz7Mark` in every name, value and text, the marker never appears in `Display`, `Debug` or the `source()` chain of 25 XML error and limit kinds (through `Document::parse`, `CotEvent::parse` and a proto `xmlDetail`), of every `ConvertErrorKind`, or of `DecodeError`: `crates/tak-cot/tests/qa_boundaries.rs` `every_xml_error_kind_is_free_of_input_content` (line 384), `every_conversion_error_kind_is_free_of_input_content` (409), `cot_event_and_element_debug_output_is_content_free` (465); `crates/tak-proto/tests/decode_hostile.rs` `decode_error_debug_has_no_payload_text` (218). Phase 1a fix round 1 (rule 6): the `AttributeRef` assertion added to the `Debug` test; re-verified by the security-reviewer |
| Gaps | `FrameError` has no content test: its messages hold fixed text and lengths only, which is known from reading `framing.rs`, not from a test. No client-facing error responses and no logging exist yet: streaming inputs are Phase 3, the REST API Phase 4, and who may see error details is designed there (ASD STIG APSC-DV-002580, Phase 3) |
| POA&M | — |
| Phase | 1a (library errors); 3–4 (client-facing errors and logs) |
| Annotation locations | [crates/tak-cot/src/error.rs](../../crates/tak-cot/src/error.rs) (line 7), [crates/tak-proto/src/lib.rs](../../crates/tak-proto/src/lib.rs) (296, `DecodeError`), [crates/tak-proto/src/framing.rs](../../crates/tak-proto/src/framing.rs) (231, `FrameError`) |

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
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | cargo-deny sources, licences and bans; three-layer pure-Rust gate (`[bans]` wrappers; `check-native-deps.sh` per target, which also rejects crypto crate names on wasm32 and since Phase 1a judges that rule on the shipped (normal-edge) closure, while the `links` rule keeps the full graph; `[bans.build]` with `executables = "deny"`, `include-archives = true` and an `allow-build-scripts` list of 20 crates since Phase 1a added `anyhow`, `getrandom`, `num-traits`, `rustix` and `tak-proto`); native code only through `aws-lc-rs` (`aws-lc-fips-sys`, and `aws-lc-sys` under POAM-2026-10-26-08); provenance rule. Phase 1a: the libFuzzer C++ exception is confined to the excluded `rust/fuzz/` workspace with its own `deny.toml`, lockfile and native allow-list (`libfuzzer-sys` only), and `scripts/check-fuzz-policy.sh` (in `make policy`) keeps `libfuzzer-sys` and `cc` out of the product graph; the `tak-proto` build script checks the in-place schemas (SA-10(1)) |
| Evidence | [deny.toml](../../deny.toml) `[bans]` (line 116), `[bans.build]` (lines 179–215, `allow-build-scripts` 194–215), `[sources]` (lines 220–224); [scripts/native-allowlist.txt](../../scripts/native-allowlist.txt); [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance and §Supply chain. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `bans ok, sources ok`; `make check-native-deps` PASS on 3 targets (wasm32: only the `prettyplease` and `wasm-bindgen-shared` markers; no crypto names); negative check: removing `zmij` from `allow-build-scripts` gives `error[build-script-not-allowed]`, `bans FAILED` (exit 2). PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `bans ok, sources ok`; `check-native-deps: ok` on x86_64 and aarch64 Linux and wasm32 (wasm32: the two markers only); job tokens `Contents: read`. Phase 1a QA, 2026-10-28, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `advisories ok, bans ok, licenses ok, sources ok`; `make audit` 143 crates, 1380 advisories, no findings (with the two POAM-2026-10-26-09 ignores, rule 3); `make check-native-deps` ok on the wasm32 closure of `tak-api-types`, `tak-web-components`, `tak-proto` and `tak-cot`; `make fuzz-deny` PASS (cargo-deny with `fuzz/deny.toml`, cargo audit on `fuzz/Cargo.lock`, native allow-list); `make policy` PASS including `check-fuzz-policy` |
| Gaps | Phase 1a: libFuzzer native code in the development-only fuzz workspace (open POA&M POAM-2026-10-27-01); `cargo-fuzz` itself is installed with `--locked` but its graph is not checked by cargo-deny; the corpus capture builds and runs the same unverified Java reference build (POAM-2026-10-26-02); no CI run for this branch (rule 7). Parity job downloads unverified Maven, Gradle-plugin, Node and npm artifacts (POAM-2026-10-26-02); parity fixture images are Docker Hub `eclipse-temurin` and community `postgis/postgis`, not UBI or Iron Bank (POAM-2026-10-26-03, extended by NIST-021); non-FIPS `aws-lc-sys` from Phase 2 (POAM-2026-10-26-08). `allow-build-scripts` names crates, not versions, so a new build script in an allowed crate relies on `Cargo.lock` and Dependabot review. The `links` check is a heuristic. One CI run (PR 7); no nightly run yet. The parity Dockerfiles are not linted (hadolint in no workflow) |
| POA&M | POAM-2026-10-26-02, POAM-2026-10-26-03, POAM-2026-10-26-08, POAM-2026-10-27-01 |
| Phase | 0 |
| Annotation locations | [.cargo/config.toml](../../.cargo/config.toml), [deny.toml](../../deny.toml), [scripts/check-native-deps.sh](../../scripts/check-native-deps.sh), [scripts/check-workflows.sh](../../scripts/check-workflows.sh), [rust-ci.yml](../../../.github/workflows/rust-ci.yml), [rust-parity.yml](../../../.github/workflows/rust-parity.yml), [rust-supply-chain.yml](../../../.github/workflows/rust-supply-chain.yml), [dependabot.yml](../../../.github/dependabot.yml) header comments; Phase 1a: [crates/tak-proto/build.rs](../../crates/tak-proto/build.rs) (line 50), [fuzz/Cargo.toml](../../fuzz/Cargo.toml) (13), [fuzz/deny.toml](../../fuzz/deny.toml) (18), [scripts/check-fuzz-policy.sh](../../scripts/check-fuzz-policy.sh) (48), [rust-fuzz.yml](../../../.github/workflows/rust-fuzz.yml) (33) |

### SR-4 Provenance

| Field | Value |
|-------|-------|
| Control | SR-4 Provenance |
| 800-53B baseline | — |
| Origination | Hybrid |
| Status | Partially Implemented |
| Phase 0 target | Partially Implemented (no signed build provenance until Phase 9; POAM-2026-10-26-05) |
| Phase 1a target | Partially Implemented (already) |
| Scope | Development environment / CI; no TOE runtime yet |
| Implementation | crates.io-only sources; SHA-pinned actions; image digests; first-party provenance rule (`Derived from` per ported item). Phase 1a: schemas read in place from `src/takserver-protobuf/` (never copied); corpus inputs synthetic, Apache-2.0, one [MANIFEST.tsv](../../parity/corpus/MANIFEST.tsv) row per (id, direction); goldens are to be imported only through `harness.sh import-corpus` with a `PROVENANCE.json` |
| Evidence | Phase 1a: corpus inputs present, 715 manifest rows for 471 case IDs and 472 files under `parity/corpus/cases/`, including `REUSE.toml` (counted 2026-10-28 from the working tree); the run's `parity-map.md` records `Derived from` for every Phase 1a row; Phase 1a QA `make deny` `sources ok`. [deny.toml](../../deny.toml) `[sources]` (lines 220–224: unknown registries and git denied, `allow-git = []`); SHA-pinned actions and digest-pinned images listed in [ATO_EVIDENCE.md](ATO_EVIDENCE.md) §Pin inventory; [CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance. QA Round 1, 2026-10-26, local ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `sources ok`; `make policy` `check-workflows` ok (SHA-pin rule); `parity/harness.sh selftest` PASS (rejects an image pin without `@sha256:`). QA Round 2, 2026-10-26, local (same section): the same gates PASS; `check-workflows` ok on 3 parsed files (rule W1, 40-hex SHA with a version comment); QA re-resolved the `actions/checkout` and `Swatinem/rust-cache` tags and re-hashed the OSCAL files, all matching. PR 7 CI, 2026-10-27, [rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255) on head `ce9caf79` ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3): `make deny` `sources ok`; `check-workflows` ok |
| Gaps | At QA Round 1, `check-workflows` was line-based and could be bypassed (QA-006); the parsed-YAML version is verified at QA Round 2; fix round 3 changes it again, not QA-verified. No signed build provenance; SBOM not required, generated once (PR 7 CI, `rust-sbom`) and not signed (POAM-2026-10-26-01); the parity build's transitive downloads are unverified (POAM-2026-10-26-02); legal review open (POAM-2026-10-26-05, widened in Phase 1a to the build-time schema read, the goldens and the corpus driver). Phase 1a: no goldens and no `PROVENANCE.json` exist (WU7 NOT RUN), so provenance is recorded for the inputs only |
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

1. There was no TOE component in Phase 0, and Phase 1a adds only two
   libraries with no runtime of their own. Most claims are developer-process
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
| 2026-10-28 | 0.8 | Phase 1a gate decisions. SI-10 Gaps: the "pending the user's decision" sentence replaced by the 2026-10-28 approvals of DEV-01c (mesh), DEV-01d, DEV-05 and DEV-07 (PROTO-001 option (b)); the option (a) caveat on the serializer's well-formed-output property removed because option (a) was not chosen. No status changed: 17 of 47 detailed blocks Partially Implemented, 30 Planned, none Implemented | `20261027-180458-phase1-protocol-config` (gate decisions) |
| 2026-10-28 | 0.7 | Phase 1a WU10 follow-up (pre-gate). CA-5 only: Implementation (`check-poam-markers` is in `POLICY_CHECKS`, Makefile lines 26–28), Evidence (compliance-scribe's local runs, not QA: `make policy` exit 0 with `check-poam-markers` 15/15 and `check-poam-ids` 13/13; `make check-annotations-strict` exit 0) and Gaps (no QA or CI run of the check; `.txt` marker in `fuzz/native-allowlist.txt` outside its scan). No status changed: 17 of 47 detailed blocks Partially Implemented, 30 Planned, none Implemented | `20261027-180458-phase1-protocol-config` (WU10 follow-up) |
| 2026-10-28 | 0.6 | Phase 1a (WU10, pre-gate). Scope: TOE library components `tak-proto` and `tak-cot` (no network wiring until Phase 3); build script and fuzz workspace outside the TOE. "Status at this revision": Phase 1a evidence, rule 1 extended to Phase 1a targets, new rules 6 (fix rounds 1 and 2 are not QA re-verification) and 7 (Actions disabled: no CI run, WU7 and WU8 NOT RUN, exit criterion not met, nothing committed after `04774cc0`). New blocks SC-5, SI-10, SI-10(3), SI-11 (Partially Implemented) and SA-10(1) (Planned, rule 6); SI-10 and SI-11 rows removed from the SI table. SA-11(8) Planned → Partially Implemented (Phase 1a QA fuzz run). 0 Implemented, 17 Partially Implemented, 30 Planned, 47 blocks (was 0 / 12 / 30, 42). Phase 1a targets table. Evidence, gaps and annotation locations updated in CA-5 (`check-poam-markers`), CM-3, CM-3(2) (F9, `rust-fuzz.yml` paths; still Planned), CM-6 (`Limits` library defaults, exact `xmlDetail` cap; NIST-006), CM-7 (fuzz isolation; still Planned), CM-10 and SR-4 (POAM-2026-10-26-05 widened per Q9), SA-10, SA-11 (golden parity NOT RUN), SA-15, SR-3 (fuzz workspace, POAM-2026-10-27-01), SC-13 and SI-7 (build-script SHA-256 is SA-10(1), CRY-001). `deny.toml` line numbers re-verified with `grep -n` after WU2: `[bans.build]` 179–215, `[sources]` 220–224, 20 build-script crates. New POA&M references POAM-2026-10-27-01, -02, -03 | `20261027-180458-phase1-protocol-config` (Phase 5, WU10) |
| 2026-10-27 | 0.5 | PR 7 CI evidence (rust-ci 40539495255, rust-parity 40539495328; head `ce9caf79`, merged as `9f754245`). "Status at this revision": the first GitHub Actions runs recorded; how rules 2 to 4 apply to them (rule 2's green-gate condition met, its nightly condition not). Evidence cells updated in CA-5, CA-7, CM-2, CM-2(2), CM-3, CM-3(2), CM-6, CM-8, RA-5, RA-5(2), SA-10, SA-11, SA-11(1), SA-15, SA-15(7), SI-2, SR-3, SR-4, SR-11; gaps that said "no CI run" corrected in those blocks and in CM-5, SA-3, SA-22. hadolint recorded as a gap in SA-11(1) and SR-3 (in no workflow; never run). No status changed: 0 Implemented, 12 Partially Implemented, 30 Planned | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.4 | Gate: POAM-2026-10-26-09 accepted for development by the system owner (John Edward Willman V) on 2026-10-27, pending AO review; still Open, not Risk Accepted. Text updated in "Status at this revision", CA-5 gaps, RA-5 gaps and POA&M, SA-22 gaps and POA&M, SI-2 POA&M. No status changed (12 Partially Implemented, 30 Planned, 0 Implemented); rule 3 unchanged, the pass with ignores is still not evidence. Line numbers re-verified with `grep -n` after the 2026-10-27 `deny.toml` and `.cargo/audit.toml` comment edits: SA-22 evidence (deny 49–52, 54–55; audit 31, 39, 34, 42), RA-5(2) `maximum-db-staleness` (57), SR-3 `[bans]` (116), `[bans.build]` (179–191), `[sources]` (212–216), SR-4 `[sources]` (212–216) | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs). No status changed (12 Partially Implemented, 30 Planned, 0 Implemented). "Status at this revision": QA Round 2 added as evidence; rule 3 now covers SA-22 with the carve-out "detection evidence only; the pass with ignores is not evidence" (NIST-018); rule 5 says fix round 3 is not QA-verified. SA-22 evidence is the QA Round 2 POAM-09 detection negative gate (NIST-018). CM-2, CM-6, SR-4: QA Round 2 evidence for the parsed-YAML `check-workflows` and the QA-006 caveat for Round 1; CM-6 permission evidence reworded (grep shows presence, W3 checks top level) (NIST-017). CM-2(2), CM-3, CM-3(2), SA-11(1): QA-006 verified at Round 2. CM-6 and SR-3 cite the extended POAM-2026-10-26-03 (NIST-021). SA-11 and CM-3 note QA-007 and QA-008. CM-7 annotation location removed with the fix round 3 annotation change (NIST-019); CM-7 stays Planned | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | Phase 5 and WU10 (pre-gate). Statuses raised from QA Round 1 local evidence, never above the Phase 0 target: CA-5, CM-2, CM-6, CM-10, SA-10, SA-11, SA-11(1), SA-15, SA-22, SR-3, SR-4, SR-11 → Partially Implemented (12); 30 blocks stay Planned. New blocks CA-5, CM-5, SR-5, SR-6 (42 in total); "CA-5 is not claimed" sentence removed (NIST-004). Targets: SA-11(1) Implemented → Partially Implemented (NIST-006); SA-22 → Partially Implemented (two open exceptions), "Risk Accepted" → "open POA&M" (NIST-003). New Planned rows AC-7, AC-10, AC-12, AU-8, AU-10, IA-2(12), IA-8, SC-12, SC-17, SI-10, SI-11 (NIST-009). "Not Applicable (component)" vocabulary and system-level Inherited wording (NIST-011). Dependabot settings part of SI-2 Hybrid, POAM-2026-10-26-10 (NIST-007). QA-001, QA-002 and QA-006 cited in CM-3, CM-3(2) and SA-11(1) gaps. Evidence and annotation locations refreshed after fix round 1 | `20261026-210255-phase0-foundations` (Phase 5, WU10) |
| 2026-10-26 | 0.1 | Initial skeleton: pinned references (Release 5.2.0, 800-53B), vocabulary, all 20 Rev 5 families, 38 detailed control blocks (all Planned; Phase 0 targets from plan §7.1), Release 5.2.0 pre-map (SA-15(13), SA-24, SI-2(7), SI-7(12)), Inherited and Not Applicable rows with reasons, OSCAL deferral. SA-22 target qualified with Risk Accepted (POAM-2026-10-26-09, raised by WU3) | `20261026-210255-phase0-foundations` (WU6) |
