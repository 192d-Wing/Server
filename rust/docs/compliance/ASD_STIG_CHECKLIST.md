# ASD STIG Checklist

**Document Version:** 0.5
**Date:** 2026-10-27
**TAK-RS Version:** 0.0.1 (Phase 0, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`)
**Status:** Phase 0 build run. Every rule is Not Reviewed (STIG release not fixed); PR 7 CI evidence added 2026-10-27.
**Run:** `20261026-210255-phase0-foundations`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| STIG | DISA Application Security and Development (ASD) STIG, **release TBD** ([ROADMAP.md](../../ROADMAP.md) §6 Q5) |
| CCI | DISA CCI list (Rev 5 mapping) of the same release, TBD |
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed) |
| Overlay | CNSSI 1253 (TBD) |

### Scope

Phase 0: development environment only. There is no TAK-RS runtime (TOE) to
check. The rules listed below concern the development process, which Phase 0
starts to put in place.

### Vocabulary

Status uses the STIG Viewer vocabulary exactly:

- **Open**: the requirement is not met.
- **Not a Finding**: the requirement is met, with evidence.
- **Not Applicable**: the requirement does not apply; a reason is given.
- **Not Reviewed**: not yet assessed.

NIST 800-53 controls are cited in Rev 5 form; see
[NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md).

### How this checklist is filled

The V-ID, title, CAT and CCI columns are filled once the STIG release is fixed.
The nist-800-53-reviewer confirms each rule ID and title against that release.
Until then, the **Title** column gives the topic only. V-IDs and CAT values
marked "(unverified)" were given by the nist-800-53-reviewer from memory
(confidence: medium) and must be checked against the release before use. A
rule moves from Not Reviewed only with evidence that has passed a recorded
gate.

The **Evidence** column lists what exists at this revision. Local results are
from QA Round 1 ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3). On 2026-10-27 the
first CI run was recorded (PR 7, same section). The STIG release, V-IDs and
CAT values are still unconfirmed, so no rule is assessed yet.

## Checklist

| Rule ID | V-ID | Title | CAT | CCI | NIST 800-53 Rev 5 | Status | Evidence | POA&M |
|---------|------|-------|-----|-----|-------------------|--------|----------|-------|
| APSC-DV-003110 | V-222642 (unverified) | Topic: no embedded authentication data (title TBD) | I (unverified) | TBD | IA-5 | Not Reviewed | Positive evidence to assess: no secrets or tokens used in `.github/workflows/rust-*.yml` (grep on 2026-10-26: `github.token` and `GITHUB_TOKEN` appear only in comments; no `secrets.`); the parity harness generates a random per-run database password (`openssl rand`) into 0600 files under `parity/.work/`; the database reads it as a compose secret and TAKCL from a 0600 env file, never from a command line ([parity/harness.sh](../../parity/harness.sh), [parity/compose.yaml](../../parity/compose.yaml)). No TOE code yet | — |
| APSC-DV-003130 | TBD | Topic: TBD (named by the nist-800-53-reviewer; confirm against the release) | TBD | TBD | TBD | Not Reviewed | — | — |
| APSC-DV-003140 | TBD | Topic: TBD (named by the nist-800-53-reviewer; confirm against the release) | TBD | TBD | TBD | Not Reviewed | — | — |
| APSC-DV-003150 | V-222646 (unverified) | Topic: security testing (title TBD) | TBD | TBD | SA-11, CA-2 | Not Reviewed | Multi-agent review workflow; local test gates PASS (QA Round 1). CI test gates PASS on PR 7 ([rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255); TAKCL `StartupTests` against Java PASS in rust-parity 40539495328). Gated run record (post-gate WU10): not yet | — |
| APSC-DV-003170 | V-222648 (unverified) | Topic: code review (title TBD) | TBD | TBD | SA-11, SA-11(1) | Not Reviewed | Review gate in `/tak-rs-workflow` (reports from three reviewers and two re-verifications in this run), [CONTRIBUTING.md](../../CONTRIBUTING.md) §Review checklist. No required human review (ruleset absent); PR 7 was merged with no required review | POAM-2026-10-26-04 |
| APSC-DV-003180 | TBD | Topic: TBD (named by the nist-800-53-reviewer; confirm against the release) | TBD | TBD | SA-11 | Not Reviewed | No code-coverage measurement in Phase 0 | — |
| APSC-DV-003190 | V-222650 (unverified) | Topic: flaws tracked (title TBD) | TBD | TBD | SI-2, CA-5 | Not Reviewed | [POAM.md](POAM.md) register (10 entries); [scripts/check-poam-ids.sh](../../scripts/check-poam-ids.sh) PASS locally (QA Round 1) and in PR 7 CI (10 IDs, 10 entries); reviewer findings and fix rounds in the run record | — |
| APSC-DV-003200 | TBD | Topic: change impact assessment (title TBD) | TBD | TBD | CM-4 | Not Reviewed | `plan.md` impact sections per run; CM-4 is Planned ([NIST_800-53_MAPPING.md](NIST_800-53_MAPPING.md) §CM) | — |
| APSC-DV-003210 | V-222652 (unverified) | Topic: flaws addressed (title TBD) | TBD | TBD | SI-2, SI-2(2) | Not Reviewed | Blocking advisory gate and nightly scan configured; local and PR 7 CI `make audit` and `make deny` PASS only with the two ignores of the open POAM-2026-10-26-09; no nightly run yet; Dependabot security updates paused | POAM-2026-10-26-07, POAM-2026-10-26-09, POAM-2026-10-26-10 |
| APSC-DV-003215 | V-222653 (unverified) | Topic: coding standards (title TBD) | TBD | TBD | SA-15 | Not Reviewed | Workspace lints, `rustfmt.toml`, `clippy.toml`; `make fmt-check` and `make clippy` PASS locally (QA Round 1) and in PR 7 CI ([rust-ci 40539495255](https://github.com/192d-Wing/Server/actions/runs/40539495255)). Plan §7.1 expects a Not a Finding candidate once the gates pass in CI; they now have, but the rule stays Not Reviewed until the STIG release is fixed and the rule confirmed | — |
| APSC-DV-003220 | V-222654 (unverified) | Topic: design document (title TBD) | TBD | TBD | SA-17, PL-8 | Not Reviewed | `plan.md` per run, [ARCHITECTURE.md](../../ARCHITECTURE.md) (short) | — |
| APSC-DV-003230 | V-222655 (unverified) | Topic: threat model (title TBD) | TBD | TBD | SA-11(2) | Not Reviewed | Threat sketch in `plan.md` §5 (not yet accepted at a gate) | — |
| APSC-DV-003240 | V-222658 (unverified) | Topic: supported components (title TBD) | I (unverified) | TBD | SA-22 | Not Reviewed | `unmaintained = "all"` in [deny.toml](../../deny.toml) and `--deny warnings` for cargo-audit; local PASS only with two ignored unmaintained crates (`proc-macro-error`, `bincode` 1.3.3) whose exception is an open POA&M, accepted for development by the system owner on 2026-10-27, pending AO review (not Risk Accepted). Detection is shown by the QA Round 2 negative gate: without the ignores, `cargo deny` and `cargo audit` each exit 1 on RUSTSEC-2024-0370 and RUSTSEC-2025-0141 ([TEST_EVIDENCE.md](TEST_EVIDENCE.md) §3). `rust-ci-gate` went green on PR 7 (run 40539495255) with the two ignores in place. If reviewed today, this CAT I rule would likely be Open | POAM-2026-10-26-09 |
| APSC-DV-003270 | TBD | Topic: configuration management (title TBD) | TBD | TBD | CM-3, SA-10 | Not Reviewed | git, `Cargo.lock`, `CHANGELOG.md`; no ruleset or `CODEOWNERS` | POAM-2026-10-26-04 |
| APSC-DV-003280 | TBD | Topic: configuration management (title TBD) | TBD | TBD | CM-3, SA-10 | Not Reviewed | [CONTRIBUTING.md](../../CONTRIBUTING.md) (change process); no ruleset | POAM-2026-10-26-04 |
| APSC-DV-003290 | TBD | Topic: configuration management (title TBD) | TBD | TBD | CM-3, SA-10 | Not Reviewed | Change review through `/tak-rs-workflow` gates; no required human review | POAM-2026-10-26-04 |

Every other ASD STIG rule: **Not Reviewed**.

## Summary

| Status | Count |
|--------|-------|
| Open | 0 |
| Not a Finding | 0 |
| Not Applicable | 0 |
| Not Reviewed | 16 listed, plus every unlisted rule |

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-27 | 0.5 | PR 7 CI evidence: "How this checklist is filled" and the Evidence cells of APSC-DV-003150, -003170, -003190, -003210 and -003215 cite the PR 7 CI run (rust-ci 40539495255, rust-parity 40539495328); APSC-DV-003240 notes that the gate went green with the two POAM-09 ignores. No status changed; all rules still Not Reviewed | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-27 | 0.4 | Gate: APSC-DV-003240 evidence records that POAM-2026-10-26-09 was accepted for development by the system owner on 2026-10-27, pending AO review (still Open, not Risk Accepted). No status changed; all rules still Not Reviewed | `20261026-210255-phase0-foundations` (gate) |
| 2026-10-27 | 0.3 | Fix round 3 (docs): APSC-DV-003240 evidence cites the QA Round 2 POAM-09 detection negative gate (NIST-018). No status changed; all rules still Not Reviewed | `20261026-210255-phase0-foundations` (fix round 3) |
| 2026-10-26 | 0.2 | Phase 5 (pre-gate). Eight rows added (APSC-DV-003110, -003130, -003140, -003180, -003200, -003270, -003280, -003290; NIST-008), 16 listed in total. V-ID and CAT pre-filled as "(unverified)" where the nist-800-53-reviewer supplied them. POAM-2026-10-26-09 on APSC-DV-003240 (NIST-003); POAM-2026-10-26-04 on -003270 to -003290; POAM-2026-10-26-10 on -003210. Evidence column refreshed from QA Round 1. All rules still Not Reviewed | `20261026-210255-phase0-foundations` (Phase 5) |
| 2026-10-26 | 0.1 | Initial skeleton: STIG release TBD, STIG Viewer status vocabulary, 8 development-process rules (APSC-DV-003150 to -003240) with planned Phase 0 evidence; all Not Reviewed | `20261026-210255-phase0-foundations` (WU6) |
