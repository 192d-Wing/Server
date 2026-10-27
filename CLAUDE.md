# CLAUDE.md

Guidance for Claude Code in this repository.

## Repository layout

- `src/` — the upstream Java TAK Server (Java 17, Gradle, Spring Boot, Ignite,
  Netty, PostgreSQL/PostGIS). It tracks `TAK-Product-Center` releases and is
  the **behavioural reference** for the rewrite. Do not modify it as part of
  rewrite work; upstream merges must stay conflict-free.
- `rust/` — the pure-Rust + Yew rewrite (TAK-RS). Cargo workspace; backend
  crates in `rust/crates`, binaries in `rust/services`, Yew/WASM apps in
  `rust/web`, compliance docs in `rust/docs/compliance`.

## Multi-agent workflow

Use the `/tak-rs-workflow` skill (`.claude/skills/tak-rs-workflow/SKILL.md`)
for features, ports, fixes and reviews in `rust/`. The main conversation is
the orchestrator and delegates to `.claude/agents/`: `tak-architect`,
`rust-coder`, `yew-coder`, `qa-engineer`, `security-reviewer`,
`nist-800-53-reviewer`, `crypto-tls-reviewer`, `tak-protocol-reviewer`,
`yew-frontend-reviewer`, `compliance-scribe`, `release-engineer`.

- `/tak-rs-workflow <task>` — plan → build → test → parallel review → fix loop → docs → release gate
- `/tak-rs-workflow port <java module>` — same, plus a Java→Rust parity map and parity tests
- `/tak-rs-workflow review [base-ref]` — review the current diff only
- `/tak-rs-workflow plan <task>` — plan only

## Rewrite rules

- **Pure Rust.** No JVM, JNI, or hand-written JavaScript. The only native code
  is the FIPS 140-3 crypto module: rustls on **aws-lc-rs with the `fips`
  feature**, the single crypto provider for everything.
- **License: Apache-2.0** for `rust/` (the Java reference in `src/` is GPLv3).
  Derive behaviour from schemas (`.proto`, XSD, OpenAPI, SQL), wire behaviour
  and black-box tests; read Java to understand behaviour, but do not translate
  it line by line. `// PARITY:` cites the Java location for traceability only.
- **Compliance baseline: NIST SP 800-53 Rev 5** (5.2.0 catalog, 800-53B
  baselines). Cite controls in Rev 5 form (`AC-3`, `CM-2(2)`, `SR-3`).
- **Wire compatibility.** CoT XML, TAK Protocol v1, federation v1/v2 and the
  Marti REST API must match the Java server for fielded ATAK/WinTAK/iTAK
  clients. Deviations are recorded and approved, never silent.
- **Annotate** control implementations with `// COMPLIANCE MAPPING:` (NIST
  800-53 Rev 5 control, ASD STIG V-ID) and ported behaviour with
  `// PARITY: <java file>#<method>`. Accepted gaps get `// POAM:`.
- **Security baseline:** validate all network input with size/depth limits,
  bounded queues only, group isolation on every fan-out, secrets in
  `Zeroizing`, fail closed and audit on denial, TLS 1.2+ everywhere.

## Commands (from `rust/`)

- `cargo build`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`
- `cargo deny check`, `cargo audit`
- Web apps: `trunk serve` / `trunk build --release` in `rust/web/<app>`

Java reference build (from `src/`): `./gradlew clean bootWar bootJar shadowJar` (see README.md).
