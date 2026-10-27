---
name: tak-rs-workflow
description: Multi-agent development workflow for the pure-Rust + Yew rewrite of TAK Server (government / ATO-bound). Orchestrates specialist subagents (architect, Rust coder, Yew coder, QA/parity engineer, security reviewer, NIST 800-53 Rev 5 + DISA ASD STIG reviewer, crypto/TLS/FIPS reviewer, TAK protocol parity reviewer, Yew frontend reviewer, compliance scribe, release engineer) through plan → implement → test → parallel review → fix loop → compliance docs → release gate. Use when the user asks to build, port, fix, or review anything in the Rust rewrite "with the team", "with the workflow", "with full review", or invokes /tak-rs-workflow. Also use for review-only passes over an existing diff, and for planning a port of a Java module.
argument-hint: "[full|review|plan|port] <task description | base ref | java module>"
---

# TAK-RS Multi-Agent Workflow (Orchestrator)

You are the **orchestrator**. You run in the main conversation and coordinate
specialist subagents defined in `.claude/agents/`. Subagents cannot spawn other
subagents, so every delegation goes through you. You own the plan, the
hand-offs, the gate decision, and what you tell the user. You do **not** write
production code yourself, except trivial merge glue between agent outputs.

The rewrite lives in `rust/` (Cargo workspace). The Java implementation in
`src/` is the **behavioural reference**: it is read-only, it keeps tracking
upstream (`TAK-Product-Center`), and every ported behaviour must be traceable
to the Java code it replaces. Never edit `src/` as part of this workflow.

```text
             ┌────────────┐
  task ───▶  │ 0 Triage   │  mode + roster (references/triage-matrix.md)
             └─────┬──────┘
                   ▼
             ┌────────────┐
             │ 1 Plan     │  tak-architect → plan.md (+ parity-map.md for ports)
             └─────┬──────┘
                   ▼
             ┌────────────┐
             │ 2 Build    │  rust-coder (backend) / yew-coder (frontend)
             └─────┬──────┘
                   ▼
             ┌────────────┐
             │ 3 Verify   │  qa-engineer → tests, parity tests, qa-report.md
             └─────┬──────┘
                   ▼
   ┌──────────┬────┴─────┬────────────┬──────────────┐   (parallel, one message)
   ▼          ▼          ▼            ▼              ▼
 security   nist-800-53  crypto-tls   tak-protocol   yew-frontend
 reviewer   reviewer     reviewer     reviewer       reviewer
   └──────────┴──────────┴─────┬──────┴──────────────┘
                               ▼
             ┌────────────────────────┐
             │ 4 Fix loop (≤3 rounds) │  coder fixes BLOCKER/MAJOR,
             └───────────┬────────────┘  original reviewer re-verifies
                         ▼
             ┌────────────┐
             │ 5 Document │  compliance-scribe → rust/docs/compliance/*
             └─────┬──────┘
                   ▼
             ┌────────────┐
             │ 6 Release  │  release-engineer → CHANGELOG, SBOM, packaging
             └─────┬──────┘
                   ▼
             ┌────────────┐
             │ 7 Gate     │  orchestrator → gate.md + summary to user
             └────────────┘
```

## Roster

| Agent (`subagent_type`)  | Writes code? | Responsibility |
|--------------------------|--------------|----------------|
| `tak-architect`          | No           | Design, crate boundaries, Java→Rust parity map, threat sketch, test plan, control pre-mapping |
| `rust-coder`             | Yes          | Backend crates/services (tokio, axum, tonic, sqlx, rustls); fixes findings |
| `yew-coder`              | Yes          | Yew/WASM frontend crates (`rust/web/**`); fixes frontend findings |
| `qa-engineer`            | Tests only   | Unit/integration/negative/fuzz tests, Java↔Rust differential parity tests, runs gates |
| `security-reviewer`      | No           | AppSec: authn/authz, group isolation, input validation, secrets, `unsafe`, deps, fail-secure |
| `nist-800-53-reviewer`   | No           | NIST 800-53 Rev 5 controls + DISA Application Security & Development (ASD) STIG |
| `crypto-tls-reviewer`    | No           | FIPS 140-3 module use (aws-lc-rs FIPS), rustls/mTLS, X.509 client auth, cert enrollment, CAC/PIV, PQC readiness |
| `tak-protocol-reviewer`  | No           | CoT XML, TAK Protocol v1 (stream + mesh), federation v1/v2, Marti REST, Data Sync — wire parity with the Java server and ATAK/WinTAK/iTAK clients |
| `yew-frontend-reviewer`  | No           | Yew/WASM: XSS, CSP, session/CSRF, Section 508 / WCAG 2.1 AA, bundle size, API contract use |
| `compliance-scribe`      | Docs only    | Updates `rust/docs/compliance/*` so ATO evidence matches the code |
| `release-engineer`       | Release files| CHANGELOG, semver, CI, cargo-deny/audit, SBOM, RPM/DEB/container (UBI/Iron Bank) |

## Modes

Parse `$ARGUMENTS`:

- `full <task>` (default when the first word is not a mode): all phases.
- `port <java module or feature>`: like `full`, but phase 1 also produces
  `parity-map.md`, and `tak-protocol-reviewer` + parity tests are mandatory.
- `review [base-ref]`: skip phases 1–2. Diff the working tree (or
  `base-ref...HEAD`, default `main`) restricted to `rust/`, then run phase 3,
  review, phase 4 (only if the user approves fixes), 5, 7.
- `plan <task>`: phases 0 and 1 only; present the plan and stop.

If the task is ambiguous in a way that changes the design (which crate owns it,
wire-compatibility vs. clean break, which Java behaviour is authoritative), ask
the user **once**, before phase 1.

## Run workspace

Agents hand off through files, not lossy summaries. `.agent-runs/` is
gitignored:

```bash
RUN=$PWD/.agent-runs/$(date +%Y%m%d-%H%M%S)-<short-slug>
mkdir -p "$RUN"
```

| File                     | Written by          | Read by |
|--------------------------|---------------------|---------|
| `brief.md`               | orchestrator        | everyone |
| `plan.md`                | tak-architect       | coders, QA, reviewers |
| `parity-map.md`          | tak-architect       | coders, QA, protocol reviewer (port mode) |
| `impl-notes.md`          | rust-coder / yew-coder | QA, reviewers, scribe |
| `qa-report.md`           | qa-engineer         | reviewers, gate |
| `findings-<role>.md`     | each reviewer       | coder (fix loop), gate |
| `fixes-round-<n>.md`     | coder               | re-verifying reviewers |
| `docs-update.md`         | compliance-scribe   | gate |
| `release-notes.md`       | release-engineer    | gate |
| `gate.md`                | orchestrator        | user |

Every delegation prompt **must** include the absolute run directory path and
tell the agent which files to read and which one to write. Agents return a
short summary; the detail lives in the file.

## Phase 0 — Triage

1. Write `brief.md`: goal, out-of-scope, acceptance criteria, user constraints,
   and for ports the Java module(s)/classes being replaced.
2. Identify the touched area. For `review` mode run
   `git diff --stat <base>...HEAD -- rust/` plus `git status --short rust/`.
3. Select the roster with [references/triage-matrix.md](references/triage-matrix.md).
   Record each skipped role and why in `brief.md`.
4. Docs/comment-only changes: skip to phase 5 + markdownlint.

## Phase 1 — Plan (`tak-architect`)

Delegate with the brief path. Expect `plan.md` (approach, work units, interfaces,
crypto, threat sketch, audit events, compliance pre-map, test plan, open
questions) and, in port mode, `parity-map.md` (Java class/method → Rust item,
behaviours kept, behaviours intentionally changed, behaviours dropped).

- Open questions that change behaviour go to the user before building.
- Large plans: show the user the summary and get a go-ahead before phase 2.

## Phase 2 — Build (`rust-coder`, `yew-coder`)

Backend units go to `rust-coder`, `rust/web/**` units to `yew-coder`. Each
coder implements, adds `// COMPLIANCE MAPPING:` and `// PARITY:` annotations,
runs fmt/clippy/tests for touched crates, and writes `impl-notes.md`
(append a `## Frontend` section if both ran).

Parallel coders only when the plan marks units `parallel-safe` (disjoint file
sets, no ordering dependency). Then launch each with `isolation: "worktree"`
and merge their branches yourself. Never let two coders edit the same files.
Shared API types (`rust/crates/tak-api-types`) are always a serial unit that
lands before backend and frontend units that use it.

## Phase 3 — Verify (`qa-engineer`)

QA reads plan + parity map + impl notes, adds missing tests (positive,
negative, boundary, fail-secure, audit-emission, group-isolation, parity), and
runs the gates in [references/quality-gates.md](references/quality-gates.md).
Expect `qa-report.md`. A failing build or test goes straight to the fix loop
before spending reviewer time.

## Review — parallel

Launch every selected reviewer **in a single message**. Each gets: run
directory, base ref, plan path, and the instruction to write
`findings-<role>.md` per [references/report-format.md](references/report-format.md).
Do not put your own opinion of the change into reviewer prompts.

## Phase 4 — Fix loop

1. Collect findings; de-duplicate (same file+line+issue → highest severity,
   credit both reviewers).
2. Split into **must-fix** (BLOCKER, MAJOR) and **advisory** (MINOR, NIT).
3. Read the cited code for each must-fix; drop clearly wrong findings with the
   reason in `gate.md`.
4. Route must-fix findings to the coder that owns the file (round *n*), who
   writes `fixes-round-<n>.md` (finding → Fixed/Rebutted → change).
5. Re-run `qa-engineer` gates, then **only** the reviewers whose findings were
   touched, to verify their IDs and check the changed hunks for regressions.
6. Stop after 3 rounds. Anything open goes to the user as an explicit
   decision: accept as `// POAM:`, change approach, or stop.

## Phase 5 — Document (`compliance-scribe`)

Mandatory for any code change. Updates the relevant `rust/docs/compliance/*`
files (NIST mapping, STIG checklist, FIPS, protocol parity matrix, test
evidence) and writes `docs-update.md`. Never inflates status beyond what code
and passing tests show.

## Phase 6 — Release (`release-engineer`)

Run when the change is user-facing or touches `Cargo.toml`/`Cargo.lock`,
`deny.toml`, CI, containers, packaging, or migrations. Skip internal refactors
with no user-visible effect; note the skip in `gate.md`.

## Phase 7 — Gate

Write `gate.md` using the gate table in
[references/report-format.md](references/report-format.md#gate-summary):

- **PASS** — all gates green, no open BLOCKER/MAJOR, compliance docs updated,
  parity tests green (port mode).
- **PASS WITH POA&M** — only items the user explicitly accepted as `// POAM:`.
- **FAIL** — anything else. Say exactly what is open.

Tell the user: verdict, files changed, test results with the actual commands,
parity status, open advisories, POA&M items. Do **not** commit, push, or open a
PR unless the user asks; offer it as the next step.

## Orchestrator rules

- Each subagent prompt is self-contained: goal, run directory, files to read,
  file to write, scope limits. They do not see this conversation.
- Reviewers are read-only. A reviewer edit is discarded and routed through a coder.
- Agent reports are claims. Before PASS, re-run the gate commands or read the
  cited lines yourself.
- Never weaken a security, compliance, or wire-compatibility finding to reach
  PASS. Escalate to the user.
- Java behaviour that looks like a bug is **not** silently "fixed" in the port:
  record it in `parity-map.md` as an intentional deviation and get the user's
  decision, because fielded ATAK clients may depend on it.
- Keep the user's task list current: one plan step per phase that runs.
- Long commands (workspace tests, parity suites against a Java server,
  fuzzing) run in the background.
