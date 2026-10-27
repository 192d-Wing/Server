# Triage Matrix — which roles run

The orchestrator picks the roster from the paths a change touches. When in
doubt, include the reviewer: a skipped reviewer is a silent gap in ATO
evidence; an extra one costs only tokens.

Paths below assume the planned workspace layout in `rust/` (see
`rust/ARCHITECTURE.md` once it exists). Update this table when crates are
added or renamed.

## Always-on for any code change

- `qa-engineer`
- `security-reviewer`
- `nist-800-53-reviewer`
- `compliance-scribe`

## Path-triggered roles

| Touched path / topic | Add |
|----------------------|-----|
| `rust/crates/tak-cot/**` (CoT XML model, parse, serialize, detail handling) | `tak-protocol-reviewer` |
| `rust/crates/tak-proto/**`, `rust/proto/**` (TAK Protocol v1, federation protos) | `tak-protocol-reviewer` |
| `rust/crates/tak-net/**` (inputs: TCP/UDP/TLS/QUIC/WebSocket/multicast, framing) | `tak-protocol-reviewer`, `crypto-tls-reviewer` |
| `rust/crates/tak-router/**` (brokering, groups, filters, repeater, data feeds, subscriptions) | `tak-protocol-reviewer` (flag group-isolation focus to `security-reviewer`) |
| `rust/crates/tak-federation/**`, `rust/services/fed-hub/**` | `tak-protocol-reviewer`, `crypto-tls-reviewer` |
| `rust/crates/tak-api/**` (Marti REST, Data Sync / missions, enterprise sync) | `tak-protocol-reviewer` |
| `rust/crates/tak-auth/**` (X.509, file users, LDAP/AD, OAuth/JWT, CAC/PIV, groups) | `crypto-tls-reviewer` (flag AC/IA focus) |
| `rust/crates/tak-pki/**` (certificate enrollment, CRL/OCSP checking, truststores) | `crypto-tls-reviewer` |
| Any TLS config, rustls setup, cipher/group choice, key generation, hashing of secrets | `crypto-tls-reviewer` |
| `rust/crates/tak-audit/**` | (always-on set; flag AU-9/AU-10 focus) |
| `rust/crates/tak-db/**`, `rust/migrations/**` | `release-engineer` |
| `rust/crates/tak-api-types/**` (shared backend/frontend DTOs) | `tak-protocol-reviewer`, `yew-frontend-reviewer` |
| `rust/web/**` (Yew apps, components, WASM build, Trunk config) | `yew-frontend-reviewer` |
| Server-side HTTP headers, CSP, cookies, static asset serving | `yew-frontend-reviewer` |
| `rust/crates/tak-plugin*/**` (plugin host: WASM component / gRPC sidecar) | `tak-protocol-reviewer` (flag sandboxing focus to `security-reviewer`) |
| `Cargo.toml`, `Cargo.lock`, `deny.toml`, `rust-toolchain.toml` | `release-engineer` |
| `rust/packaging/**`, `Dockerfile*`, `docker/**`, `deploy/**` (Helm, k8s) | `release-engineer` |
| `.github/workflows/**`, `.github/dependabot.yml`, `rust/Makefile` | `release-engineer` |
| `rust/scripts/**`, `rust/.cargo/**`, `rust/LICENSE`, `rust/NOTICE` | `release-engineer` |
| `rust/parity/docker/**`, `rust/parity/compose.yaml` | `release-engineer`, `qa-engineer` |
| `rust/CONTRIBUTING.md` | `compliance-scribe` |
| `rust/fuzz/**`, `rust/tests/**`, `rust/parity/**`, `rust/benches/**` only | `qa-engineer` only (+ scribe if TEST_EVIDENCE changes) |

## Mode overrides

| Mode | Roster change |
|------|---------------|
| `port` | `tak-protocol-reviewer` always; QA must add parity tests |
| any change to a `rust/web/**` crate | `yew-coder` builds, `yew-frontend-reviewer` reviews |

## Phase skips

| Situation | Skip |
|-----------|------|
| Docs / comments only | Phases 1–4; scribe + markdownlint only |
| Bug fix with an obvious one-file cause | Phase 1 may be a short inline `plan.md` written by the orchestrator |
| Internal refactor, no behaviour/API/wire change | Phase 6 |
| `review` mode | Phases 1–2 |

Record every skipped role or phase, with the reason, in `brief.md`.
