# Report Formats

## Findings file (`findings-<role>.md`)

```markdown
# Findings — <role>

Scope: <base>...<head> (<n> files), plan: <run>/plan.md
Verdict: APPROVE | APPROVE WITH COMMENTS | CHANGES REQUESTED

## <ROLE-PREFIX>-001 [BLOCKER] <one-line title>

- **Location**: `rust/crates/tak-x/src/file.rs:123`
- **Java reference**: `src/takserver-core/src/main/java/.../Foo.java:88` (when relevant)
- **Reference**: NIST 800-53 AC-3 | ASD STIG V-222425 | FIPS 140-3 | CoT schema | CWE-...
- **Problem**: what is wrong, in one or two sentences.
- **Failure scenario**: concrete input/state → concrete bad outcome.
- **Fix**: the smallest change that resolves it.
- **Confidence**: high | medium | low
```

Role prefixes: `SEC` (security), `NIST` (800-53/STIG), `CRY` (crypto/TLS),
`PROTO` (TAK protocol parity), `WEB` (Yew frontend), `QA` (testing),
`REL` (release).

### Severity

| Severity | Meaning | Gate effect |
|----------|---------|-------------|
| BLOCKER | Exploitable vulnerability, cross-group data leak, key/secret exposure, non-FIPS crypto on a protected path, missing authn/authz, audit bypass, data loss, wire break with fielded clients | Must fix |
| MAJOR | Missing required control/test, STIG CAT I/II violation, fail-open path, unvalidated input not yet reaching a sink, unapproved parity deviation | Must fix |
| MINOR | Hardening, STIG CAT III, missing annotation, weak test, unclear error type | Advisory |
| NIT | Style, naming, comments | Advisory |

A finding without a location and a failure scenario is MINOR at most.

## Fix response (`fixes-round-<n>.md`)

```markdown
| Finding | Action | Change |
|---------|--------|--------|
| SEC-001 | Fixed | `router.rs:120-131` check group intersection before fan-out |
| PROTO-002 | Rebutted | Java also drops `<__group>` on federation egress (`FederationProcessor.java:412`) |
```

## Parity map (`parity-map.md`, port mode)

```markdown
| Java (src/...) | Rust (rust/...) | Status | Derived from | Notes |
|----------------|-----------------|--------|--------------|-------|
| `SubmissionService#processCot` | `tak_router::Router::submit` | Ported | wire capture | |
| `GroupFederationUtil#...` | — | Deferred | — | phase N |
| `XyzFilter` legacy quirk | `tak_router::filter` | Deviation (needs approval) | black-box test | Java NPEs on empty detail; Rust rejects with 400 |

`Derived from` is one of `schema`, `wire capture`, `black-box test`, `Java read`
(provenance rule, `rust/CONTRIBUTING.md`).
```

## Gate summary

`gate.md`, written by the orchestrator:

```markdown
# Gate — <task>

Verdict: PASS | PASS WITH POA&M | FAIL

| Gate | Result | Evidence |
|------|--------|----------|
| fmt | PASS | `cargo fmt --all --check` |
| clippy | PASS | `cargo clippy --workspace --all-targets -- -D warnings` |
| unit tests | PASS (212) | `cargo test --workspace --lib` |
| parity | PASS (48 cases) vs Java 5.7-RELEASE-14 | `cargo test -p tak-parity` |
| integration | NOT RUN | PostGIS unavailable |
| security review | APPROVE | findings-security.md (0 open) |
| NIST/STIG review | APPROVE WITH COMMENTS | findings-nist.md (2 MINOR) |
| crypto/TLS review | APPROVE | findings-crypto.md |
| protocol review | APPROVE | findings-protocol.md |
| frontend review | SKIPPED | no rust/web changes |
| compliance docs | UPDATED | docs-update.md |
| release | CHANGELOG + minor bump suggested | release-notes.md |

## Open items
- NIST-003 [MINOR] ...

## POA&M
- (none) | `// POAM:` at file:line, accepted by user on <date>

## Approved parity deviations
- (none) | parity-map.md row, approved by user on <date>

## Dropped findings
- SEC-004: not reachable; input already validated at `handler.rs:40`.
```
