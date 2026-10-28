# TAK-RS Roadmap — pure Rust + Yew rewrite of TAK Server

Status: **DRAFT for review** (2026-10-26). Reference: Java TAK Server
`5.7-RELEASE-14` in `../src`.

## 1. What we are replacing

About 233k lines of production Java and about 26k lines of first-party web UI
code. The size is concentrated in a few modules:

| Area | Java size | Notes |
|------|-----------|-------|
| Messaging core (`takserver-core`) | 69.5k LOC | NIO + Netty inputs, brokering, groups, federation, cluster |
| API (`takserver-war`) | 71.9k LOC | 49 controllers + 27 servlets, 378 OpenAPI operations; missions alone ≈ 10k LOC / 78 ops |
| Common / config / plugins SDK | 21k LOC | JAXB config (CoreConfig.xsd 1,590 lines), Ignite service interfaces |
| Federation (fig-core, federation-common, hub) | 27k LOC | FIG gRPC, ROL grammar, hub broker on MongoDB |
| CLIs and jobs (usermanager, schemamanager, retention, plugin-manager) | 22k LOC | |
| Web UIs | about 320 JSP/HTML pages | jQuery/AngularJS Marti admin, React tool-ui, Angular 19 + AngularJS fedhub UI |
| Database | 94 Flyway migrations, about 70 tables | PostGIS use is light (points, bounding boxes) |

Contract artifacts we can reuse directly: the 15 `.proto` files, the XSDs, the
290-path OpenAPI spec (`src/takserver-takcl-core/src/main/openapi/`), the
Flyway SQL, and the **TAKCL integration suites (205 tests)**, which test the
server only over the wire and so become the parity harness.

## 2. Architecture decisions (proposed)

| # | Decision | Proposal | Why |
|---|----------|----------|-----|
| D1 | Process model | **One `takserver` binary with roles** (`--roles messaging,api,config,retention`); monolith by default | Removes the Spring-profile/Ignite split, which is the largest source of complexity |
| D2 | Replacement for Ignite | Monolith: in-process tokio channels and `moka` caches. Cluster: **NATS (JetStream KV for shared state) + Postgres** | NATS is already the cluster bus; no usable Rust Ignite client exists |
| D3 | Database | **Same schema, same migrations**, run by `sqlx migrate` from the existing SQL | Java and Rust can share one DB during cutover, and parity diffs stay simple |
| D4 | Config | serde + quick-xml models of `CoreConfig.xsd`, with **round-trip tests** on every example XML | Operators keep their CoreConfig.xml unchanged |
| D5 | Crypto | rustls + **aws-lc-rs FIPS** as the only crypto provider; `rcgen` for the CA; `p12` for keystores | FIPS 140-3; the only allowed C dependency. **Decided 2026-10-26** |
| D11 | License | **Apache-2.0** for the `rust/` workspace, with a provenance control (behaviour is derived from schemas, wire behaviour and black-box tests; Java code is not translated line by line) | **Decided 2026-10-26**. The derivative-work question against the GPLv3 Java reference is tracked as a POA&M item for legal review |
| D12 | Control baseline | **NIST SP 800-53 Rev 5** (5.2.0 catalog, 800-53B baselines), all 20 families tracked; DISA ASD STIG for application checks | **Decided 2026-10-26**. Target baseline level and the CNSSI 1253 overlay are still to be confirmed |
| D6 | HTTP / gRPC | axum + tower (route→role middleware generated from the 189 `intercept-url` rules), tonic/prost, quinn for QUIC | |
| D7 | Frontend | Yew + Trunk, one Cargo workspace; shared DTOs in `tak-api-types`; strict CSP | |
| D8 | Plugins | **Out-of-process plugins over gRPC** (same model as today's separate plugin JVM), plus an optional Java bridge so existing plugin jars keep running | Keeps the existing plugin ecosystem; WASM components can come later |
| D9 | Federation Hub | Port it last; **move its storage from MongoDB to Postgres** | One database engine to accredit |
| D10 | Compatibility bar | Byte- and JSON-compatible with Java for ATAK/WinTAK/iTAK and federates; deviations need approval in a parity map | Fielded clients cannot be updated with the server |

## 3. Workspace layout

```text
rust/
  Cargo.toml  rust-toolchain.toml  deny.toml
  crates/
    tak-proto        prost/tonic from ../src/takserver-protobuf (build.rs)
    tak-cot          CoT XML model, streaming parser, proto↔XML detail mapping
    tak-config       CoreConfig / UserAuthenticationFile / TAKIgniteConfig
    tak-crypto       FIPS provider setup, TLS configs, keystores, CA ops
    tak-auth         X.509 identity, file users, LDAP, OAuth/JWT, group bit vectors
    tak-db           sqlx models + migrations (copied SQL)
    tak-audit        structured, tamper-evident audit events
    tak-router       submission, subscriptions, groups, filters, repeater, latest-SA
    tak-net          tcp/stcp/udp/mcast/tls/quic/grpc/websocket inputs
    tak-api          Marti REST (axum)
    tak-api-types    DTOs shared with the Yew apps
    tak-federation   FIG v2 (+ v1), ROL parser
    tak-plugin-host  gRPC plugin protocol
  services/
    takserver        the single server binary
    tak-usermanager  CLI (UserManager.jar replacement)
    tak-schema       CLI (SchemaManager replacement)
    fed-hub          Federation Hub (late phase)
  web/
    admin            Marti admin console (+ tool-ui)
    fedhub-ui        Federation Hub UI
    components       shared Yew components (DoD banner, marking, tables, map)
  parity/            harness driving Java and Rust servers with the same input
  tests/  fuzz/  docs/compliance/  packaging/
```

## 4. Phases

Each phase ends with a `/tak-rs-workflow` gate (PASS or PASS WITH POA&M) and a
parity milestone. "Exit" says what must be true before the next phase starts.

### Phase 0 — Foundations (workspace and guardrails)

- Workspace, toolchain pin, `deny.toml` (adapted from OstrichPKI), CI with
  fmt/clippy/test/deny/audit for native and wasm32 targets.
- `rust/docs/compliance/` skeletons: NIST mapping, ASD STIG checklist, FIPS,
  PROTOCOL_PARITY, POAM.
- **Parity harness v0:** a container of the Java reference server built from
  `src/`, a Postgres/PostGIS fixture, and the TAKCL suites wired up so they
  can target either server.
- Exit: CI green on an empty workspace; the TAKCL `StartupTests` pass against Java in the harness.

### Phase 1 — Protocol and config crates (no network yet)

Split into two parts with separate gates; Phase 1 exits when both pass.

- **Phase 1a — protocol crates (in progress, gate pending):** `tak-proto`,
  `tak-cot` (with fuzz targets) and the golden corpus.
  - Exit: golden-fixture tests show XML↔proto conversion is byte-equivalent
    to Java's `StreamingProtoBufHelper` on a corpus captured from the Java
    server.
- **Phase 1b — config crate (planned):** `tak-config` with round-trip tests
  on every `CoreConfig.example*.xml`.
  - Exit: round trip on all five examples and an XSD-coverage test pass.

### Phase 2 — Data and identity

- `tak-db` (existing migrations, schema-diff test against a Flyway-built DB),
  `tak-crypto`, `tak-auth` (X.509 + CRL, file users with bcrypt compatibility,
  LDAP/AD, group bit vectors), `tak-audit`.
- Warm-up ports: `tak-schema` and `tak-usermanager` CLIs.
- Exit: the Rust CLIs and the Java CLIs produce identical DBs and UserAuthenticationFiles.

### Phase 3 — Messaging MVP (first deployable milestone)

- `tak-net` tls/stcp/tcp/udp/mcast inputs, TAK Protocol negotiation,
  `tak-router` (groups, subscriptions, filters, repeater, latest-SA, contacts),
  `cot_router` persistence; the `takserver` binary in monolith mode.
- Exit: TAKCL InputTests, SimpleTests, PointToPointTests and
  SubscriptionTests pass against Rust. Load test reaches at least the Java
  throughput. **ATAK clients can connect and share SA.**

### Phase 4 — Core REST API and admin UI shell

- `tak-api` with the route→role table; version, groups, contacts,
  subscriptions, inputs, cot, enterprise sync (upload/search/content), cert
  enrollment, OAuth token API.
- `web/admin` shell: DoD banner, login/session, inputs, groups, users,
  certificates pages.
- Exit: JSON diff against Java is clean for each ported operation; Yew pages
  pass the frontend review (CSP, 508).

### Phase 5 — Missions / Data Sync

- The 78 mission operations, change logs, roles and invitations, layers,
  feeds, mission packages; plus the matching admin pages.
- Exit: TAKCL missions suites pass; ATAK Data Sync works end to end.

### Phase 6 — Extra transports and federation

- gRPC InputChannel, QUIC, WebSockets (`/takproto/1`, `/payload/1`), STOMP COP.
- Federation v2 (FIG + ROL), then v1; interop tests against a Java server.
- Exit: FederationV1Tests and FederationV2Tests pass with mixed Java↔Rust federates.

### Phase 7 — Long tail

- Retention service, data feeds, video, device profiles, ExCheck, CITrap,
  KML, map layers, QoS/rate limiting, metrics, XMPP (only if still needed).

### Phase 8 — Cluster, plugins, Federation Hub

- NATS-based cluster mode, Helm charts; gRPC plugin host and Java plugin
  bridge; Federation Hub (broker, policy, Postgres storage) and `web/fedhub-ui`.

### Phase 9 — Accreditation and cutover

- RPM/DEB and hardened UBI/Iron Bank containers, SBOM, signed artifacts.
- Complete the STIG checklist and SSP evidence; side-by-side operation on a
  shared DB; migration guide for operators.

## 5. Biggest risks and how we handle them

| Risk | Mitigation |
|------|------------|
| Ignite semantics are spread across about 50 caches and 16 remote services | D1/D2: monolith first; each distributed manager is redesigned in its own `plan` run before it is ported |
| Mission API rules and JSON shapes | Phase 5 has its own phase; OpenAPI spec + JSON diffs + TAKCL missions suites |
| Streaming wire fidelity under load | Golden corpus from Java, fuzzing, protocol reviewer in every relevant run, load tests in Phase 3 |
| Federation interop with Java servers that have not been migrated | Mixed-federation test topology in the parity harness |
| Auth matrix (CRL, LDAP, OAuth server, WSTEP) | Phase 2 core; WSTEP and the OAuth password grant kept as explicit later items |
| Breadth of the web UI | Rebuild by REST area alongside each backend phase, not as a big bang at the end |
| Plugin ecosystem | D8 Java bridge keeps existing jars working |

## 6. Open questions for the program owner

1. Is the in-repo `rust/` location right, or should TAK-RS be its own repository?
2. Plugins: is the Java plugin bridge required, or can we break plugin compatibility?
3. Federation v1: still required, or v2 only?
4. Which of XMPP, CITrap, ExCheck, VBM and WSTEP are in scope?
5. Target accreditation: which 800-53B baseline (Moderate assumed) and CNSSI
   1253 overlay, which ATO boundary, and does it require Iron Bank?

Resolved: crypto provider (D5), workspace license (D11), control catalog
(D12); CI runs on every PR with a single required `rust-ci-gate` job.
