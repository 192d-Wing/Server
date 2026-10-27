# Protocol Parity: Java TAK Server to TAK-RS

**Document Version:** 0.1
**Date:** 2026-10-26
**TAK-RS Version:** 0.0.1 (Phase 0, unreleased)
**Java Reference:** TAK Server `5.7-RELEASE-14` (`../src`, commit `5187abd4`)
**Status:** Skeleton (Phase 0). Every feature is Not started.
**Run:** `20261026-210255-phase0-foundations`

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

Phase 0: development environment only. This document inventories the Java
features TAK-RS must reproduce, from [ROADMAP.md](../../ROADMAP.md) §1 and §4,
and the TAKCL suites that will prove parity. No feature has been ported.

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
2026-10-26. A path names the main entry point for the feature, not every file
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
| Protobuf schemas (15 `.proto` files) | [takserver-protobuf/src/main/proto/](../../../src/takserver-protobuf/src/main/proto/) | `tak-proto` | 1 | Not started | — | — |
| TAK Protocol v1 negotiation | [StreamingProtoBufOrCoTProtocol.java](../../../src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/StreamingProtoBufOrCoTProtocol.java) | `tak-net`, `tak-proto` | 3 | Not started | — | — |
| TAK Protocol v1 streaming framing and XML↔proto conversion | [StreamingProtoBufHelper.java](../../../src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java) | `tak-proto`, `tak-cot` | 1 | Not started | — | Phase 1 golden corpus (planned) |
| CoT XML model and streaming parser | TBD (Phase 1 plan) | `tak-cot` | 1 | Not started | — | Phase 1 golden corpus and fuzz targets (planned) |

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
| CoreConfig round-trip | [CoreConfig.xsd](../../../src/takserver-common/src/main/xsd/CoreConfig.xsd) | `tak-config` | 1 | Not started | — | Round-trip of every `CoreConfig.example*.xml` (planned) |
| UserAuthenticationFile round-trip | [UserAuthenticationFile.xsd](../../../src/takserver-common/src/main/xsd/UserAuthenticationFile.xsd) | `tak-config` | 1 | Not started | — | — |
| TAKIgniteConfig round-trip | [TAKIgniteConfig.xsd](../../../src/takserver-common/src/main/xsd/TAKIgniteConfig.xsd) | `tak-config` | 1 | Not started | — | — |
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

Inventory count: 14 + 10 + 5 + 12 + 6 + 11 = 58 features, all Not started.

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
| StartupTests.jarStartupValiationTest | Yes | Pending first harness run | Not run (Rust target Phase 3; POAM-2026-10-26-06) |
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

| Feature | Java behaviour | TAK-RS behaviour | Reason | Approved by | Date | Parity test |
|---------|----------------|------------------|--------|-------------|------|-------------|
| — | — | — | None yet | — | — | — |

## Normalisation rules

None yet. Nothing in Phase 0 compares Java and Rust output.

## Document Change History

| Date | Version | Change | Run |
|------|---------|--------|-----|
| 2026-10-26 | 0.1 | Initial skeleton: status vocabulary, Table A (58 features from ROADMAP §1 and §4, all Not started, Java entry points checked to exist), Table B (TAKCL inventory from `testing.md` and `TestRunnerModule.java`; StartupTests baseline pending first harness run), empty Table C, provenance note | `20261026-210255-phase0-foundations` (WU6) |
