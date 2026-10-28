# Protocol Parity: Java TAK Server to TAK-RS

**Document Version:** 0.4
**Date:** 2026-10-28
**TAK-RS Version:** 0.0.1 (Phase 1a, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`, commit `5187abd4`)
**Status:** Phase 1a build run, before its gate. Three Phase 1 features are In progress; none is Ported, because the golden corpus has not been captured (GitHub Actions disabled; WU7 and WU8 NOT RUN). Nine deviations approved by the user (ten Table C rows: DEV-01c has a stream row approved 2026-10-27 and a mesh row approved 2026-10-28); no deviation decision pending. Approval does not make a feature Ported. Java baseline recorded for StartupTests only (PR 7 CI, 2026-10-27).
**Run:** `20261027-180458-phase1-protocol-config`

## Document Control

### Pinned references

| Reference | Value |
|-----------|-------|
| Java reference | `5.7-RELEASE-14` in `../src` (read-only; never edited by rewrite work) |
| Compatibility bar | ROADMAP D10: byte- and JSON-compatible with Java for ATAK, WinTAK, iTAK and federates; deviations need approval |
| Control catalog | NIST SP 800-53 Rev. 5, **Release 5.2.0** (parity testing is SA-11 evidence) |
| Baselines | NIST SP 800-53B, Release 5.2.0; target **Moderate** (assumed, not confirmed) |
| Overlay | CNSSI 1253 (TBD) |
| STIG | DISA ASD STIG, release TBD |

### Scope

This document inventories the Java features TAK-RS must reproduce, from
[ROADMAP.md](../../ROADMAP.md) §1 and §4, and the TAKCL suites that will prove
parity. Phase 1a ports the TAK Protocol v1 types and framing (`tak-proto`) and
the CoT XML model, parser, serializer and XML ↔ protobuf conversion
(`tak-cot`), with no network wiring. Their row-level parity map (Java
behaviour, Rust location, status and derivation per wire behaviour) is the
run's `parity-map.md` (revision 4, `20261027-180458-phase1-protocol-config`);
this document carries the feature-level status and the approved deviations.
No feature has been ported.

### Vocabulary

| Status | Meaning |
|--------|---------|
| Not started | No Rust implementation |
| In progress | A run is porting it; not yet gated |
| Ported | Gated, with passing parity tests listed in the row |
| Deviation (approved) | Differs from Java by design; listed in [Table C](#table-c-approved-deviations) with the approval |
| Deferred | Postponed to a later phase than planned; reason given |
| Dropped | Will not be ported; reason and approval given |

A row moves to **Ported** only with a passing parity test recorded in
[TEST_EVIDENCE.md](TEST_EVIDENCE.md).

### Provenance

`rust/` is Apache-2.0 and the Java reference in `src/` is GPLv3. Behaviour is
derived from schemas, observed wire behaviour, black-box tests and public
specs; Java source is read to understand behaviour but not translated. The
**Derived from** column records the source for each ported item: `schema`,
`wire capture`, `black-box test` or `Java read`. The **Java
reference** column cites the Java location for traceability only. See
[CONTRIBUTING.md](../../CONTRIBUTING.md) §Provenance and POAM-2026-10-26-05.

Java paths below are relative to `src/`. They were checked to exist on
2026-10-26; the two paths added in Phase 1a (`CotParser.java`,
`XmlContainer.java`) on 2026-10-28. A path names the main entry point for the feature, not every file
involved. "TBD" means the entry point is identified in that phase's plan.

## Table A: Feature inventory

### Streaming inputs and protocol

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| TCP input | [NioNettyTcpServerHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/netty/handlers/NioNettyTcpServerHandler.java) | `tak-net` | 3 | Not started | — | GeneralTests.tcpTest |
| Streaming TCP input (`stcp`) | [NioNettyStcpServerHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/netty/handlers/NioNettyStcpServerHandler.java) | `tak-net` | 3 | Not started | — | GeneralTests.streamTcpTest |
| UDP input | [NioNettyUdpHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/netty/handlers/NioNettyUdpHandler.java) | `tak-net` | 3 | Not started | — | GeneralTests.udpTest |
| TLS input | [NioNettyTlsServerHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/netty/handlers/NioNettyTlsServerHandler.java) | `tak-net` | 3 | Not started | — | GeneralTests.sslTest |
| Multicast input | [MulticastServerBinder.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/binder/impls/MulticastServerBinder.java) | `tak-net` | 3 | Not started | — | GeneralTests.mcastTest, GeneralTests.mcastSendTest |
| QUIC input | [QuicStreamingServer.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/quic/QuicStreamingServer.java) | `tak-net` | 6 | Not started | — | — |
| gRPC InputChannel | [GrpcStreamingServer.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/grpc/GrpcStreamingServer.java) | `tak-net` | 6 | Not started | — | — |
| WebSocket `/takproto/1` | [TakProtoWebSocketHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/websockets/TakProtoWebSocketHandler.java) | `tak-net` | 6 | Not started | — | WebsocketsTests.basicSecureWebsocketTest, WebsocketsTests.simpleWSS |
| WebSocket `/payload/1` | [BinaryPayloadWebSocketHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/websockets/BinaryPayloadWebSocketHandler.java) | `tak-net` | 6 | Not started | — | — |
| STOMP COP | [WebSocketConfiguration.java](../../../src/takserver-core/src/main/java/tak/server/config/WebSocketConfiguration.java) | `tak-net` | 6 | Not started | — | — |
| Protobuf schemas (15 `.proto` files) | [takserver-protobuf/src/main/proto/](../../../src/takserver-protobuf/src/main/proto/) | `tak-proto` | 1 | In progress (1a) | schema | Golden corpus inputs written; capture (WU7) and golden test (WU8) NOT RUN. Schemas read in place and digest-checked by `build.rs` |
| TAK Protocol v1 negotiation | [StreamingProtoBufOrCoTProtocol.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/StreamingProtoBufOrCoTProtocol.java) | `tak-net`, `tak-proto` | 3 | Not started | — | — |
| TAK Protocol v1 streaming framing and XML↔proto conversion | [StreamingProtoBufHelper.java](../../../src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java) | `tak-proto`, `tak-cot` | 1 | In progress (1a) | black-box test (corpus, not yet captured), Java read, schema | Golden corpus X2P, P2X, P2P directions: inputs written, capture and golden test NOT RUN. Rust tests check the parity-map behaviour statements, not Java output. Stream splitting and connection handling Phase 3 |
| CoT XML model and streaming parser | [CotParser.java](../../../src/takserver-plugins/src/main/java/tak/server/cot/CotParser.java), [XmlContainer.java](../../../src/takserver-plugins/src/main/java/tak/server/cot/XmlContainer.java) (`asXml`) | `tak-cot` | 1 | In progress (1a) | black-box test (corpus, not yet captured), Java read | Golden corpus X2X direction: inputs written, capture and golden test NOT RUN; 7 fuzz targets (local, 300 s each, 0 crashes). `tak-cot` parses one document; splitting a stream into events is Phase 3 (parity map D-01) |

### Routing and state

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| Submission and brokering | [SubmissionService.java](../../../src/takserver-core/src/main/java/com/bbn/marti/service/SubmissionService.java) | `tak-router` | 3 | Not started | — | StartupTests.jarStartupValiationTest (delivery A to B) |
| Groups and group isolation | [GroupManager.java](../../../src/takserver-common/src/main/java/com/bbn/marti/remote/groups/GroupManager.java) | `tak-auth`, `tak-router` | 2–3 | Not started | — | GeneralTests.anonWithGroupInputTest, GeneralTests.groupToNonGroup |
| Filters | [filter/](../../../src/takserver-core/src/main/java/com/bbn/marti/filter/) | `tak-router` | 3 | Not started | — | — |
| Subscriptions | [SubscriptionManagerLite.java](../../../src/takserver-common/src/main/java/com/bbn/marti/remote/SubscriptionManagerLite.java) | `tak-router` | 3 | Not started | — | SubscriptionTests.clientSubscriptions |
| Point-to-point delivery | [MessagingUtilImpl.java](../../../src/takserver-core/src/main/java/com/bbn/marti/groups/MessagingUtilImpl.java) | `tak-router` | 3 | Not started | — | PointToPointTests (4 tests) |
| Repeater | [RepeaterManager.java](../../../src/takserver-common/src/main/java/com/bbn/marti/remote/RepeaterManager.java) | `tak-router` | 3 | Not started | — | — |
| Latest SA | TBD (Phase 3 plan) | `tak-router` | 3 | Not started | — | GeneralTests.latestSAAnon, GeneralTests.latestSADisconnectTest, GeneralTests.LatestSAFileAuth, GeneralTests.LatestSAInputGroups |
| Contacts | [ContactManager.java](../../../src/takserver-common/src/main/java/com/bbn/marti/remote/ContactManager.java) | `tak-router` | 3 | Not started | — | — |
| Data feeds | [DataFeedService.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/feeds/DataFeedService.java) | `tak-router`, `tak-api` | 7 | Not started | — | StreamingDataFeedsTests.dataFeedRemoveAddTest |
| `cot_router` persistence | [RepositoryService.java](../../../src/takserver-core/src/main/java/com/bbn/marti/service/RepositoryService.java) | `tak-db`, `tak-router` | 3 | Not started | — | — |

### Federation

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| Federation v1 | [NioNettyFederationServerHandler.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/netty/handlers/NioNettyFederationServerHandler.java) | `tak-federation` | 6 | Not started | — | FederationV1Tests (4 tests) |
| Federation v2 (FIG) | [FederationServer.java](../../../src/takserver-core/src/main/java/tak/server/federation/FederationServer.java) | `tak-federation` | 6 | Not started | — | FederationV2Tests (4 tests) |
| ROL grammar | [Rol.g4](../../../src/takserver-fig-core/rol/src/main/antlr/mil/af/rl/rol/Rol.g4) | `tak-federation` | 6 | Not started | — | — |
| Federated WebSockets | [WebsocketsFederationTests.java](../../../src/takserver-core/src/integrationTest/java/com/bbn/marti/tests/WebsocketsFederationTests.java) (test) | `tak-federation`, `tak-net` | 6 | Not started | — | WebsocketsFederationTests (2 tests) |
| Federation Hub | [FederationHubServer.java](../../../src/federation-hub-broker/src/main/java/tak/server/federation/hub/broker/FederationHubServer.java) | `fed-hub` | 8 | Not started | — | FedHubTests |

### Marti REST API

The OpenAPI contract is [takserver-openapi-spec.json](../../../src/takserver-takcl-core/src/main/openapi/takserver-openapi-spec.json).

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| Version | [VersionApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/util/VersionApi.java) | `tak-api` | 4 | Not started | — | — |
| Groups | [GroupsApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/groups/GroupsApi.java) | `tak-api` | 4 | Not started | — | — |
| Contacts | [ContactsApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/api/ContactsApi.java) | `tak-api` | 4 | Not started | — | — |
| Subscriptions | [SubscriptionApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/api/SubscriptionApi.java) | `tak-api` | 4 | Not started | — | — |
| Inputs | [SubmissionApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/network/SubmissionApi.java) | `tak-api` | 4 | Not started | — | InputTests.inputRemoveAddTest |
| CoT | [CotApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/api/CotApi.java) | `tak-api` | 4 | Not started | — | — |
| Enterprise sync (upload, search, content) | [EnterpriseSyncService.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/EnterpriseSyncService.java) | `tak-api` | 4 | Not started | — | missions.EnterpriseFileSync, FederationEnterpriseFileSync |
| Certificate enrollment | [CertManagerApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/tak/tls/CertManagerApi.java) | `tak-api`, `tak-crypto` | 4 | Not started | — | — |
| OAuth token | [TokenApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/oauth/TokenApi.java) | `tak-api`, `tak-auth` | 4 | Not started | — | — |
| Missions (78 operations) | [MissionApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/api/MissionApi.java) | `tak-api` | 5 | Not started | — | missions.MissionAddRetrieveRemove, missions.MissionDataFlowTests, missions.MissionFileSync, missions.MissionUserCustomRolesTests, missions.MissionUserDefaultRolesTests |
| Mission layers | [MissionLayerRepository.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/repository/MissionLayerRepository.java) | `tak-api` | 5 | Not started | — | — |
| Mission packages | [MissionPackageUploadServlet.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/sync/MissionPackageUploadServlet.java) | `tak-api` | 5 | Not started | — | — |

### Configuration, database and CLIs

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| CoreConfig round-trip | [CoreConfig.xsd](../../../src/takserver-common/src/main/xsd/CoreConfig.xsd) | `tak-config` | 1b | Not started | — | Round-trip of every `CoreConfig.example*.xml` (planned, Phase 1b) |
| UserAuthenticationFile round-trip | [UserAuthenticationFile.xsd](../../../src/takserver-common/src/main/xsd/UserAuthenticationFile.xsd) | `tak-auth` | 2 | Deferred (from Phase 1 to Phase 2, decision Q8, 2026-10-27: modelled with its consumer) | — | — |
| TAKIgniteConfig round-trip | [TAKIgniteConfig.xsd](../../../src/takserver-common/src/main/xsd/TAKIgniteConfig.xsd) | — | 8 (decision) | Deferred (decision Q8, 2026-10-27: not modelled until Phase 8; Ignite is replaced, ROADMAP D2) | — | — |
| Schema migrations (94 Flyway files) | [db/migration/](../../../src/takserver-schemamanager/src/main/resources/db/migration/) | `tak-db` | 2 | Not started | — | Schema-diff against a Flyway-built DB (planned) |
| SchemaManager CLI | [SchemaManager.java](../../../src/takserver-schemamanager/src/main/java/com/bbn/tak/schema/SchemaManager.java) | `tak-schema` | 2 | Not started | — | Identical DB from Java and Rust CLIs (planned) |
| UserManager CLI | [UserManager.java](../../../src/takserver-usermanager/src/main/java/com/bbn/marti/UserManager.java) | `tak-usermanager` | 2 | Not started | — | UserManagementTests.UserManagerTest |

### Services and long tail

| Feature | Java reference (file) | Rust crate | Phase | Status | Derived from | Parity tests |
|---------|-----------------------|------------|-------|--------|--------------|--------------|
| Retention service | [RetentionApplication.java](../../../src/takserver-retention/src/main/java/tak/server/retention/RetentionApplication.java) | `takserver` (retention role) | 7 | Not started | — | — |
| Plugins (out-of-process gRPC) | [takserver-plugin-manager/](../../../src/takserver-plugin-manager/) | `tak-plugin-host` | 8 | Not started | — | PluginStartupTests |
| Video | [VideoConnectionManager.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/video/VideoConnectionManager.java) | `tak-api` | 7 | Not started | — | — |
| Device profiles | [ProfileAPI.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/device/profile/api/ProfileAPI.java) | `tak-api` | 7 | Not started | — | — |
| ExCheck | [ExCheckAPI.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/excheck/ExCheckAPI.java) | `tak-api` | 7 | Not started | — | — |
| CITrap | [CITrapReportAPI.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/citrap/CITrapReportAPI.java) | `tak-api` | 7 | Not started | — | — |
| KML | [KmlMasterSaServlet.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/KmlMasterSaServlet.java) | `tak-api` | 7 | Not started | — | — |
| Map layers | [MapLayersApi.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/maplayer/api/MapLayersApi.java) | `tak-api` | 7 | Not started | — | — |
| QoS and rate limiting | [QoSManager.java](../../../src/takserver-common/src/main/java/tak/server/qos/QoSManager.java) | `tak-router` | 7 | Not started | — | — |
| Metrics | TBD (Phase 7 plan) | `tak-api` | 7 | Not started | — | — |
| XMPP (only if still in scope; ROADMAP §6 Q4) | [XmppAPI.java](../../../src/takserver-core/takserver-war/src/main/java/com/bbn/marti/xmpp/XmppAPI.java) | `tak-api` | 7 | Not started | — | — |

Inventory count: 14 + 10 + 5 + 12 + 6 + 11 = 58 features: 3 In progress
(Phase 1a), 2 Deferred (UserAuthenticationFile, TAKIgniteConfig) and
58 − 3 − 2 = 53 Not started. None is Ported.

## Table B: TAKCL suite inventory

Source: the "Current Test List" in
[testing.md](../../../src/takserver-takcl-core/docs/testing.md) (lines
144–200, "Updated 08/09/2022") and the runner's class list in
[TestRunnerModule.java](../../../src/takserver-takcl-core/src/exe/java/com/bbn/marti/takcl/AppModules/TestRunnerModule.java).
"Needs DB" follows the upstream runner
([testrunner.sh](../../../src/takserver-takcl-core/scripts/testrunner.sh)
line 65: names containing `ebsocket`, `ission` or `StartupTests`).

| Suite.test | Needs DB | Java baseline | Rust result |
|------------|----------|---------------|-------------|
| StartupTests.jarStartupValiationTest | Yes | PASS, 2026-10-27, [rust-parity 40539495328](https://github.com/192d-Wing/Server/actions/runs/40539495328) (PR 7, head `ce9caf79`; `[dbAvailableEnabled][Java17.0.20.1]`; tests 1, failures 0, errors 0; artifact `parity-results-40539495328-1`) | Not run (Rust target Phase 3; POAM-2026-10-26-06) |
| FederationV1Tests.advancedFederationTest | No | Not run | Not run |
| FederationV1Tests.basicFederationTest | No | Not run | Not run |
| FederationV1Tests.basicMultiInputFederationTest | No | Not run | Not run |
| FederationV1Tests.federateConnectionInitiatorWaitTest | No | Not run | Not run |
| FederationV2Tests.advancedFederationTest | No | Not run | Not run |
| FederationV2Tests.basicFederationTest | No | Not run | Not run |
| FederationV2Tests.basicMultiInputFederationTest | No | Not run | Not run |
| FederationV2Tests.federateConnectionInitiatorWaitTest | No | Not run | Not run |
| GeneralTests.LatestSAFileAuth | No | Not run | Not run |
| GeneralTests.LatestSAInputGroups | No | Not run | Not run |
| GeneralTests.anonWithGroupInputTest | No | Not run | Not run |
| GeneralTests.groupToNonGroup | No | Not run | Not run |
| GeneralTests.latestSAAnon | No | Not run | Not run |
| GeneralTests.latestSADisconnectTest | No | Not run | Not run |
| GeneralTests.mcastSendTest | No | Not run | Not run |
| GeneralTests.mcastTest | No | Not run | Not run |
| GeneralTests.sslTest | No | Not run | Not run |
| GeneralTests.streamTcpTest | No | Not run | Not run |
| GeneralTests.tcpTest | No | Not run | Not run |
| GeneralTests.udpTest | No | Not run | Not run |
| InputTests.inputRemoveAddTest | No | Not run | Not run |
| PointToPointTests.basicPointToPointTest | No | Not run | Not run |
| PointToPointTests.callsignIdentificationTest | No | Not run | Not run |
| PointToPointTests.mixedIdentificationTest | No | Not run | Not run |
| PointToPointTests.uidIdentificationTest | No | Not run | Not run |
| StreamingDataFeedsTests.dataFeedRemoveAddTest | No | Not run | Not run |
| SubscriptionTests.clientSubscriptions | No | Not run | Not run |
| UserManagementTests.UserManagerTest | No | Not run | Not run |
| WebsocketsFederationTests.advancedWebsocketsFederationV1Test | Yes | Not run | Not run |
| WebsocketsFederationTests.advancedWebsocketsFederationV2Test | Yes | Not run | Not run |
| WebsocketsTests.basicSecureWebsocketTest | Yes | Not run | Not run |
| WebsocketsTests.simpleWSS | Yes | Not run | Not run |
| missions.EnterpriseFileSync (suite) | Yes | Not run | Not run |
| missions.MissionAddRetrieveRemove (suite) | Yes | Not run | Not run |
| missions.MissionDataFlowTests (suite) | Yes | Not run | Not run |
| missions.MissionFileSync (suite) | Yes | Not run | Not run |
| missions.MissionUserCustomRolesTests (suite) | Yes | Not run | Not run |
| missions.MissionUserDefaultRolesTests (suite) | Yes | Not run | Not run |

The list above has 33 individual tests and 6 mission suites listed by class
only.

Notes:

- `SimpleTests` is commented out of the runner's class list
  (`TestRunnerModule.java` line 110), so it cannot be run through `takcl.jar`
  today. The ROADMAP Phase 3 exit names it; the Phase 3 plan must resolve this.
- The runner also registers `GenerateOpenApiSpec`, `PluginStartupTests`,
  `FederationEnterpriseFileSync` and `FedHubTests`, which the 2022 list does
  not show. Their methods are not yet inventoried.
- The "205 tests" figure in [ROADMAP.md](../../ROADMAP.md) §1 is
  **unverified**. It will be checked with `takcl.jar tests list` in the
  harness.

## Table C: Approved deviations

Only deviations the user has approved are listed. The IDs are those of the
Phase 1a parity map. Approvals, both recorded in the run's `decisions.md`:

- 2026-10-27: the user's "approve all" of the Phase 1a plan's recommended
  answers (questions Q3 to Q5).
- 2026-10-28, at the Phase 1a gate: the user's answer "Approve DEV-07 (b)"
  chose PROTO-001 option (b); `decisions.md` records that answer as also
  approving the three other pending deviations (DEV-01c mesh part, DEV-01d,
  DEV-05). This closes gate blockers 2 and 3 (NIST-003).

"Parity test" names the Rust test that pins the TAK-RS behaviour; the golden
comparison that would show the deviation against Java output is NOT RUN (WU7,
WU8).

| Feature | Java behaviour | TAK-RS behaviour | Reason | Approved by | Date | Parity test |
|---------|----------------|------------------|--------|-------------|------|-------------|
| DEV-01a: CoT XML parser resource limits | No parser limit; the stream splitter stops at 8 Mi characters (`StreamingCotProtocol.java`) | Depth 64, 100 000 nodes, 256 attributes and 32 namespace declarations per element, 1 024-byte names, 1 MiB attribute values, 8 MiB input; inputs within the limits behave as Java | SC-5, SI-10: bounded allocation for untrusted input | User (John Edward Willman V), Q3 | 2026-10-27 | `crates/tak-cot/tests/xml_limits.rs` (each limit at its value and one past it) |
| DEV-01b: XML 1.1 | Xerces accepts `version="1.1"` | Refused; not reachable through the reference's stream splitter, which starts every message at `<event` | SI-10: one character model | User, Q3 | 2026-10-27 | `crates/tak-cot/tests/xml_wellformed.rs::xml_declaration_rules` |
| DEV-01c (stream part only): length prefix and TakMessage size | Any varint length prefix accepted (shift unchecked) and buffered until it arrives | Prefix over 5 bytes or above `u32` refused; a frame over 8 MiB refused before the payload arrives | SC-5: an unbounded length prefix is a denial-of-service vector | User, Q3 | 2026-10-27 | `crates/tak-proto/tests/framing.rs` `stream_decode_rejects_a_sixth_varint_byte`, `stream_decode_rejects_a_varint_above_u32`, `stream_decode_checks_the_limit_before_the_payload_arrives` |
| DEV-02 (informational): `xmlDetail` charset | Encoded with the JVM default charset before being parsed as UTF-8, so a non-UTF-8 locale turns non-ASCII text into `?` | Always UTF-8; equal to the reference under a UTF-8 locale (the pinned images) | No wire value in locale-dependent corruption | User, Q3 | 2026-10-27 | `crates/tak-cot/tests/convert.rs::dev02_xml_detail_is_read_as_utf8` |
| DEV-03: `Double.toString` digits | JDK 17 occasionally prints more digits than needed (fixed in JDK 19) | Shortest round-trip digits | Exact JDK 17 digit generation not reproduced; the curated set must still match exactly, the random set is measured | User, Q4 | 2026-10-27 | `crates/tak-cot/tests/javafmt.rs` `double_to_string_*`; golden `dbl-fmt` curated set NOT RUN |
| DEV-04: known field with the wrong wire type | Treated as an unknown field; parsing continues | Whole message refused | SI-10; only malformed senders are affected | User, Q5 | 2026-10-27 | `crates/tak-proto/tests/schema.rs::decode_rejects_a_known_field_with_the_wrong_wire_type` |
| DEV-01c (mesh part): mesh version varint | `StreamingProtoBufOrCotProtocol#byteBufToCot` reads the version with `StreamingProtoBufHelper#readVarint` without a length check into a 32-bit accumulator; with Java shift-distance masking, an encoding of any length is accepted as version 1 whenever that accumulator equals 1 | A non-minimal encoding of 1 is accepted up to 5 bytes; an encoding over 5 bytes or above `u32::MAX` is refused (`FrameError::VarintTooLong` / `VarintOverflow`) | SC-5, SI-10: bounded varint decoding; only hand-crafted mesh datagrams are affected (ATAK writes the one-byte `0x01`) | User | 2026-10-28 | `crates/tak-proto/tests/framing.rs` `mesh_rejects_bad_version_varints`, `mesh_accepts_a_non_minimal_version` |
| DEV-01d: conversion-path size and structure limits | No size or structure limit: X2P concatenates `xmlDetail` without a limit (`StreamingProtoBufHelper#cot2protoBuf`); P2X sets attribute values of any length and appends `xmlDetail` children with no depth, node or size limit beyond the parser's (`StreamingProtoBufHelper#proto2cot`) | X2P refuses a concatenated `xmlDetail` over `max_xml_detail_bytes` (8 MiB; `convert.xml_detail_too_large`); P2X refuses a proto string over `max_attr_value_bytes` (1 MiB; `convert.build`) and an `xmlDetail` over `max_xml_detail_bytes` or a merged event over depth 64 or 100 000 nodes (`convert.xml_detail`) | SC-5, SI-10: extends DEV-01a to the conversion paths; only single events of several MiB or with very deep or wide `detail` trees are affected | User | 2026-10-28 | `crates/tak-cot/tests/convert.rs::k10_xml_detail_output_is_capped`; `crates/tak-cot/tests/qa_boundaries.rs` `x2p_xml_detail_output_exactly_at_the_cap_accepted_one_byte_less_refused`, `p2x_xml_detail_exactly_at_the_cap_accepted_one_byte_more_refused`, `p2x_xml_detail_cap_is_exact_at_default_limits`, `p2x_xml_detail_depth_counts_from_the_event_root`, `p2x_xml_detail_nodes_count_against_the_event`; `crates/tak-cot/tests/xml_limits.rs::attr_value_bytes_1_mib_accepted_plus_one_refused` |
| DEV-05: CoT time outside the signed 64-bit millisecond range | `DateUtil#millisFromCotTimeStr` (Joda-Time) returns a silently wrapped value, e.g. `292278994-08-17T07:12:56Z` → `-9223372036854775000`, and the event is converted with it | The value is unparseable, so the X2P conversion fails like any bad time (`ConvertError::Time`) and the event is not delivered to proto receivers | SI-10: no meaningless time is forwarded; only CoT with years near ±292 million is affected; no fielded client, no corpus case | User | 2026-10-28 | `crates/tak-cot/tests/javafmt.rs::parse_cot_time_rejects_where_the_reference_wraps_around`; `crates/tak-cot/tests/convert.rs::dev05_out_of_range_time_rejected_like_any_bad_time` |
| DEV-07: proto strings that are not XML 1.0 characters (PROTO-001 option (b)) | `StreamingProtoBufHelper#proto2cot` sets attribute values from proto strings without a character check and relays the event: proto receivers get the strings unchanged, XML receivers get output that is not well-formed XML 1.0 (expected `&#N;` for C0 controls; exact form fixed by the C-14 goldens) | The whole event is refused in P2X (`ConvertError::Build`, `convert.build`), so it reaches no receiver (P2P goes through P2X, as in the reference) | SI-10: TAK-RS never emits XML that is not well-formed; no fielded client is known to send such strings. Revisit in Phase 3 when the router decides whether proto-to-proto traffic bypasses P2X | User | 2026-10-28 | `crates/tak-cot/tests/convert.rs::p2x_strings_xml_cannot_carry_fail_the_build`; C-14 golden cases get `DEVIATIONS.tsv` rows at golden import (NOT RUN) |

**PROTO-001 decision.** Recorded 2026-10-28: option (b). TAK-RS keeps
refusing a proto event whose strings hold a code point that is not an XML 1.0
`Char` (U+0000 to U+0008, U+000B, U+000C, U+000E to U+001F, U+FFFE, U+FFFF);
no code change. This behaviour is DEV-07 above. Option (a), reproducing the
reference's pass-through, was not chosen. The C-14 corpus cases get
`DEVIATIONS.tsv` rows when the goldens are imported; until then the
divergence from the Java output is not shown by any golden (WU7, WU8 NOT RUN).

DEV-02 is informational and DEV-06 is not allocated (folded into DEV-01d).

**Phase 1a findings carried to Phase 3** (parity map §F; no Phase 1a code
change):

- D-17: every ingest path must set the receipt time as `creationTime`, as the
  reference always writes one (PROTO-007).
- D-18: duplicate local-name identity attributes (`p:uid` and `uid`) are
  accepted (SEC-003). The reference uses two lookup rules: the converter and
  `getUid`/`getType` match by local name, while `getCallsign`, `getTeam` and
  `getRole` use namespace-sensitive XPath that skips prefixed `contact` and
  `__group` elements (PROTO-009). Phase 3 mirrors both rules where the
  reference uses them, or records a deviation for approval.
- D-19: whether a root element other than `event` is refused (SEC-004).
- D-20: a process-wide memory budget or bounded ingest (SEC-002,
  POAM-2026-10-27-02).

## Normalisation rules

**None: exact bytes.** Phase 1a defines every comparison as exact byte
equality with no normalisation, and the ok/reject outcome must match in every
direction:

| Direction | Equal means |
|-----------|-------------|
| X2P (CoT XML → TakMessage) | Identical TakMessage bytes, without stream framing; the framed form is checked separately as `0xbf ‖ varint(len) ‖ bytes` |
| P2X (TakMessage → CoT XML) | Identical UTF-8 bytes, including the XML declaration, any newline after it, attribute order, quoting and escaping |
| X2X (CoT XML → CoT XML) | Identical bytes; the reference re-serialises every XML event it forwards |
| P2P (TakMessage → TakMessage) | Identical bytes; what a proto client receives when a proto client sends |

Java numbers and CoT times are compared as identical strings, bits, millisecond
values or reject outcomes.

**Gating rule.** Every golden case must match, except a case listed in
`rust/parity/corpus/golden/DEVIATIONS.tsv`, and a case may be listed there
only against a deviation in Table C. The seeded random-double set (`dbl-fmt`)
is measured, not gated (DEV-03); the curated double set is gated.

**Status.** Exact bytes is the target, not a result: no golden exists yet
(WU7 NOT RUN), and the golden test (WU8) is not written. Nothing in this
document claims byte-equivalence with the Java reference until WU8 passes.

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-28 | 0.4 | Phase 1a gate decisions (decisions.md, "User decisions on the 1a gate"). Table C: DEV-01c (mesh part), DEV-01d, DEV-05 and DEV-07 added as approved 2026-10-28; approval paragraph cites the user's answer "Approve DEV-07 (b)" and decisions.md's record that it also approves the other three; "Pending the user's decision" list removed. PROTO-001 recorded as option (b) (DEV-07). Status line: nine deviations approved in ten Table C rows (DEV-01c counted once), none pending. No feature status changed: 3 In progress, 2 Deferred, 53 Not started, 0 Ported; WU7 and WU8 still NOT RUN | `20261027-180458-phase1-protocol-config` (gate decisions) |
| 2026-10-28 | 0.3 | Phase 1a (WU10, pre-gate). Table A: "Protobuf schemas", "TAK Protocol v1 streaming framing and XML↔proto conversion" and "CoT XML model and streaming parser" → In progress (1a), Derived from filled, golden capture and test NOT RUN; CoT parser Java reference set to `CotParser.java` and `XmlContainer.java#asXml`; CoreConfig to Phase 1b; UserAuthenticationFile → Deferred to Phase 2 and TAKIgniteConfig → Deferred to a Phase 8 decision (Q8). Inventory: 3 In progress, 2 Deferred, 53 Not started, 0 Ported. Table C: the six approved deviations DEV-01a, DEV-01b, DEV-01c (stream part), DEV-02 (informational), DEV-03, DEV-04 (approved 2026-10-27, Q3–Q5); DEV-01c (mesh), DEV-01d, DEV-05 and PROTO-001 (a)/DEV-07 listed as pending, not approved. Normalisation rules: none, exact bytes, four directions and the gating rule; byte-equivalence stays a target until WU8 | `20261027-180458-phase1-protocol-config` (Phase 5, WU10) |
| 2026-10-27 | 0.2 | PR 7 CI: Table B Java baseline for StartupTests.jarStartupValiationTest recorded as PASS (rust-parity 40539495328, `verify-junit: PASS: files=1 tests=1 executed=1 failures=0 errors=0 skipped=0`). Rust result unchanged (Not run, POAM-2026-10-26-06); no feature status changed; no deviation | `20261026-210255-phase0-foundations` (PR 7 CI) |
| 2026-10-26 | 0.1 | Initial skeleton: status vocabulary, Table A (58 features from ROADMAP §1 and §4, all Not started, Java entry points checked to exist), Table B (TAKCL inventory from `testing.md` and `TestRunnerModule.java`; StartupTests baseline pending first harness run), empty Table C, provenance note | `20261026-210255-phase0-foundations` (WU6) |
