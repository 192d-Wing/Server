# Parity harness v0

The parity harness builds the Java TAK Server reference from `src/`, runs the
TAKCL integration suites against it with PostgreSQL/PostGIS, and checks the
JUnit results. Later phases point the same suites at the Rust server.

Phase 0 exit criterion: TAKCL `StartupTests` passes against the Java reference
in this harness. That run happens in CI only (`rust-parity.yml`; see
[What is verified where](#what-is-verified-where)). Phase 1a adds the
[golden corpus](#golden-corpus) commands.

## Topology

```text
 host (Docker or Podman + compose v2)
 |
 |  build-java:  .work/clone (throwaway git clone of this repo)
 |               --bind--> [tak-parity-java-build: JDK 17, non-root]
 |                          ./gradlew takserver-package:buildIntegrationTesterDir
 |                          (egress: Gradle dist, Maven Central, Plugin Portal, Node/npm)
 |
 |  image:       .work/tester --> tak-parity-java:<src sha>  (JRE 17, uid 10001)
 |
 |  up / test:   network "parity" (internal: no egress, no published ports)
 |    +------------------------------------------+      +--------------------+
 |    | takcl: java -jar /opt/tak/utils/takcl.jar | 5432 | db: PostgreSQL 15  |
 |    |   spawns config/messaging/api/retention  |----->|     PostGIS 3.3    |
 |    |   JVMs of the Java server (same container)|     | martiuser / cot    |
 |    +------------------------------------------+      +--------------------+
 |
 +--> results/<UTC>-<TestId>/  (TEST-*.xml, takcl logs, build-info.json)
```

## Prerequisites

- Docker, or Podman with `podman compose`, providing compose v2 (`--wait` support).
  Podman is best-effort and untested in Phase 0.
- x86_64 recommended: the pinned `postgis/postgis:15-3.3` image is linux/amd64 only.
- At least 25 GB free disk (`preflight` checks this) and at least 12 GB RAM. The takcl
  container is limited to 10 GB by default; override with `TAKCL_MEM_LIMIT`.
- `git`, `python3` (stdlib only), `openssl`, and `sha256sum` or `shasum`.
- Run as an unprivileged user. `build-java` refuses to run as root.

## Commands

```text
harness.sh [--engine docker|podman] [--results <dir>]
           [--version-tag <X.Y-RELEASE-N>|auto] <command>...
```

Commands can be chained. They run in order and stop at the first failure. The
whole chain is validated before anything runs, so `up test X --target rust`
exits 2 without starting anything.

| Command | Effect |
|---------|--------|
| `preflight` | Checks the engine and compose v2, git, python3, openssl, free disk, repo root, and `pins.env` |
| `build-java [--ref <rev>]` | Clones to `.work/clone`, adds an annotated tag in the clone if needed, verifies the Gradle wrapper pins (exit 4), enforces the distribution SHA-256, builds in the JDK 17 container, copies `integrationTesterDir` to `.work/tester`, writes `.work/build-info.json` |
| `image` | Builds `tak-parity-java:<src sha>` from `docker/Dockerfile.takcl` |
| `up` | Creates `.work/pg-password` and `.work/takcl.env` (mode 0600) and starts `db` (`--wait`) |
| `test <TestId> [--target java\|rust]` | Runs `takcl.jar tests run <TestId>`, copies results, removes key material, runs `verify-junit`, and propagates failure. `rust` exits 2 |
| `verify-junit <dir>` | Requires at least one `TEST-*.xml`, at least one executed test, and 0 failures and errors |
| `list` | Runs `takcl.jar tests list` |
| `logs` | Saves compose logs to `<results>/<UTC>-compose-logs/` |
| `down` | Runs `compose down --volumes` (removes the DB volume) |
| `clean` | Runs `down`, then removes `.work/`, which deletes the password file. Results are kept |
| `capture-corpus` | Runs the golden-corpus driver against the built reference's jars, twice, in the JRE 17 image with no network. Fails unless both runs are identical. Writes `<results>/<UTC>-corpus/` with `PROVENANCE.json`. Needs `build-java` and `image`. See [Golden corpus](#golden-corpus) |
| `verify-corpus [<dir>]` | Compares a capture with `corpus/golden/` byte for byte: every file except `PROVENANCE.json` and `DEVIATIONS.tsv`, with no missing or extra files. Without `<dir>`, it uses the capture made earlier in the same chain. Drift exits 1, and having no goldens exits 3 |
| `import-corpus <dir>` | Validates a capture (layout, no special files, and pins, driver and input hashes equal to the checkout's, else exit 4). Then replaces `corpus/golden/`, keeping `DEVIATIONS.tsv` |
| `selftest` | Checks fixtures, the exit-code contract, pin validation, wrapper integrity, and the corpus extractor, layout, verify and import checks. Needs no containers |

Typical run:

```sh
rust/parity/harness.sh preflight build-java image up test StartupTests
rust/parity/harness.sh logs down
```

Golden corpus (also `make parity-capture-corpus` from `rust/`):

```sh
rust/parity/harness.sh preflight build-java image capture-corpus verify-corpus
```

Exit codes: `0` ok · `1` test or build failure · `2` unsupported command or
target, or bad usage · `3` missing prerequisite · `4` integrity check failed.

### Environment

| Variable | Purpose |
|----------|---------|
| `CONTAINER_ENGINE` | Default engine (`docker`) |
| `TAKCL_MEM_LIMIT` | takcl container memory limit (default `10g`) |
| `TAK_PARITY_PINS` | Alternate pins file. It is validated like `pins.env` and its SHA-256 is recorded in `build-info.json`. Used by the local integrity negative test |
| `TAK_PARITY_ENGINE_BIN` | Test only: overrides the engine binary name (selftest "missing engine" case) |
| `TAK_PARITY_CORPUS_DIR` | Test only: alternate corpus root (selftest), validated like `corpus/` |

## Pins

`pins.env` is the single source of pins for this script and for
`.github/workflows/rust-parity.yml`. The harness parses it and never sources
it. An unknown key, a missing key, a placeholder value, or an image without
`@sha256:` makes it exit 4.

| Key | Value source |
|-----|--------------|
| `JDK_BUILD_IMAGE` | `eclipse-temurin:17-jdk-jammy`, multi-arch index digest (Docker Hub registry API) |
| `JRE_RUN_IMAGE` | `eclipse-temurin:17-jre-jammy`, index digest. Upstream `testrunner.sh` uses `17-jammy`, which is the JDK image (same digest as `17-jdk-jammy`) |
| `POSTGIS_IMAGE` | `postgis/postgis:15-3.3`, amd64 manifest digest (tag last pushed 2023-08-14) |
| `GRADLE_VERSION` | Must equal the `distributionUrl` version in `src/gradle/wrapper/gradle-wrapper.properties` |
| `GRADLE_DIST_SHA256` | `https://services.gradle.org/distributions/gradle-8.10.2-bin.zip.sha256` |
| `GRADLE_WRAPPER_JAR_SHA256` | SHA-256 of `src/gradle/wrapper/gradle-wrapper.jar`. It is the genuine Gradle 7.5/7.5.1 published wrapper (`gradle-7.5.1-wrapper.jar.sha256`), not the 8.10.2 one |

When upstream bumps Gradle or replaces the wrapper jar, `build-java` fails
with exit 4 until someone re-resolves the pins, updates this file, and records
the change in the CM-2 pin inventory (`rust/docs/compliance/ATO_EVIDENCE.md`).

## What is verified where

| Check | Local (macOS, no Docker) | CI (`rust-parity.yml`) |
|-------|--------------------------|------------------------|
| `bash -n`, shellcheck | yes (shellcheck if installed) | yes |
| `--help`, unknown command, `--target rust`, missing engine | yes (`selftest`) | yes |
| `verify-junit` fixtures (pass, fail, error, empty, forged counts, DTD) | yes (`selftest`) | yes |
| Pin validation and wrapper integrity (synthetic trees plus real `src/`) | yes (`selftest`) | yes |
| `build-java` with a wrong wrapper pin exits 4 before any download | yes (`TAK_PARITY_PINS=<scratch>`) | — |
| Corpus war extractor negatives (zip-slip, absolute path, symlink, duplicate, oversize, not a zip), classpath probe | yes (`selftest`) | yes |
| `verify-corpus` pass, drift, missing, extra, symlink, layout errors; `import-corpus` pins, inputs and provenance mismatches; `DEVIATIONS.tsv` kept | yes (`selftest`, fixtures in `tests/fixtures/corpus/`) | yes |
| Gradle build, image build, DB, `StartupTests` | no | yes |
| `capture-corpus` twice-identical against the real reference, then `verify-corpus` | no | yes |

## Golden corpus

Phase 1a's exit criterion is that the CoT XML and TAK Protocol v1
conversion is byte-equivalent to the Java server's in all four directions
(`x2p`, `x2x`, `p2x`, `p2p`), plus the number and time formatting tables.
The evidence is a golden corpus in `corpus/`. Its inputs, output format,
byte-equivalence rules, licence and update procedure are in
[corpus/README.md](corpus/README.md). `corpus/MANIFEST.tsv` is the list of
cases (one row per id and direction); this file gives no counts.

**Status: inputs only.** `corpus/golden/` does not exist yet: no capture has
been run against the reference, so the exit criterion is not met. Until the
first capture is imported:

- `verify-corpus` exits 3 ("no committed goldens"). The scheduled and
  pull-request `rust-parity` runs therefore fail at the corpus step; the
  TAKCL suite step still runs and reports on its own.
- The first capture is a `workflow_dispatch` run with
  `verify_corpus: false`, followed by `import-corpus` on its
  `parity-corpus-<run>` artifact and a reviewed PR with the goldens (see
  [corpus/README.md](corpus/README.md)).

`capture-corpus` adds no network access and no new image. It reuses the two
pinned images:

- **Builder image** (`tak-parity-java-build:local`, JDK 17): runs
  `javac --release 17` on `corpus/driver/CorpusDriver.java`.
- **Reference image** (`tak-parity-java:<src sha>`, JRE 17): runs the driver
  with `--entrypoint java` against the `WEB-INF/lib` jars extracted from
  `.work/tester/takserver.war`.

Both containers run under the same sandbox:

- `--network none`, `--read-only`, a `noexec` tmpfs `/tmp`, `--cap-drop ALL`,
  `no-new-privileges`, 4 GB memory and 512 pids;
- the invoking non-root uid, the same choice as `build-java`. With uid 10001
  the bind-mounted output directory would not be writable without
  world-writable modes.

The corpus and the classpath are mounted read-only. Each run writes only to
its own directory under `.work/corpus/`. That directory goes through
`reject_special` before the host reads it (SEC-003, SEC-009). The two runs
must be byte-identical.

`rust-parity.yml` runs `capture-corpus verify-corpus` after the Java build
(`capture-corpus` only when dispatched with `verify_corpus: false`). It
uploads the capture as its own `parity-corpus-<run>` artifact, which is the
input to `import-corpus`.

### Fuzz seeds from the corpus inputs

The corpus inputs also seed the fuzz targets in `rust/fuzz/`:
`rust/scripts/fuzz-seeds.sh` (`make fuzz-seeds`, and per target through
`make fuzz-run FUZZ_TARGET=<t>`) builds `fuzz/seeds/<target>/` from
`corpus/cases/`. Inputs over `FUZZ_SEED_MAX_BYTES` (64 KiB, the same value as
libFuzzer's `-max_len`) are skipped and counted (the 8 MiB, 1 MiB and 400 KB
limit cases). Goldens are not used as seeds. `rust-fuzz.yml` also runs on
pull requests that change `corpus/cases/`, and new inputs are picked up
without any change to the fuzz workspace.

## How TAKCL runs StartupTests (findings)

All paths are under `src/`. The full analysis is in plan §1.2 of the Phase 0
run.

- `takcl.jar tests run <TestId>` dispatches to `TestRunnerModule.run`
  (`takserver-takcl-core/src/exe/java/com/bbn/marti/takcl/AppModules/TestRunnerModule.java:264-291`).
  It exits 0 only if every test passed, and writes
  `TEST-<name>.xml` under `/opt/tak/TEST_RESULTS/TEST_ARTIFACTS` (`:612-619`).
- `SimpleTests` is commented out of the runner's class list (`:110`).
- `StartupTests.jarStartupValiationTest`
  (`takserver-core/src/integrationTest/java/com/bbn/marti/tests/StartupTests.java:24-45`)
  starts the config, messaging, api and retention processes. Two anonymous
  `stcp` clients connect, client A sends, and the test verifies delivery to B.
- TAKCL picks the server under test in `RunnableServerManager.buildServerInstance`
  (`takserver-takcl-core/src/main/java/com/bbn/marti/takcl/connectivity/server/RunnableServerManager.java:93-117`).
  The choice is local processes, or Kubernetes when `TAKCL_K8S_MODE` is set.
  There is no external-server mode.
- `LocalRunnableServer` (`LocalRunnableServer.java:82-136`) spawns `java` from
  `PATH` with `-jar takserver.war -Dspring.profiles.active=<profile>` and
  `-Xmx2000m` per process. That is why the takcl memory limit is 10 GB, not 6 GB.
- TAKCL detects readiness by scraping `logs/takserver-<profile>.log`
  (`ServerProcessDefinition.java:32-68`).
- When `CoreConfig` has `<File>` auth, which the default does, online
  file-auth changes join the server's Ignite cluster
  (`OnlineFileAuthModule.java:68`).
- DB settings: `TAKCL_SERVER0_DB_HOST`, `TAKCL_SERVER_POSTGRES_PASSWORD`,
  `TAKCL_ENABLE_DB`
  (`takserver-usermanager/src/main/java/com/bbn/marti/takcl/TAKCLCore.java:68-95`).
  The user is `martiuser`, the database `cot`, and the port is fixed at 5432
  (`AbstractRunnableServer.java:246`).
- TAKCL generates certificates with `certs/makeCert.sh` (needs `openssl`;
  `keytool` ships with the JRE). It runs `ps -aux` when a process dies
  (`AbstractRunnableServer.java:470`), so the image installs `procps`.
- `RunnableServerManager.java:156` refuses to start if `TEST_RESULTS` already
  exists in the model dir. The image therefore never creates it, and each
  `test` runs in a fresh container.
- Upstream `takserver-takcl-core/scripts/testrunner.sh` is not used, for these
  reasons:
  - it needs a TTY (`docker run -it`);
  - it discards the result (`|| true`);
  - it uses `POSTGRES_HOST_AUTH_METHOD=trust` and published DB ports;
  - it runs `SchemaManager.jar` on the host.
- Java build pitfalls:
  - `src/build.gradle` derives the version from annotated
    `git describe` through Grgit. This repo's tags are lightweight, so the
    harness re-creates the nearest tag as an annotated tag **in the clone only**.
  - A checkout of origin has no tags at all. Then `--version-tag auto` (what
    `rust-parity.yml` passes) creates an annotated tag in the clone on the
    newest first-parent merge "Merge pull request #N from
    TAK-Product-Center/upstream/<X.Y-RELEASE-N>"; `--version-tag
    <X.Y-RELEASE-N>` names the release. The name is validated; without the
    option the harness exits 3 when no tag is reachable.
  - `gradle-wrapper.properties` has no `distributionSha256Sum`, so the
    harness appends the pinned one in the clone.
  - The build task is `takserver-package:buildIntegrationTesterDir`
    (`takserver-package/build.gradle:402-405`,
    `takserver-package/utils/utils.gradle:70`).

## Rust target

`test <TestId> --target rust` exits 2 until Phase 3. TAKCL cannot target an
already-running server without changes to `src/`. The planned Phase 3 route
leaves `src/` untouched: a `java` launcher shim placed first on `PATH` in the
harness image.

- The shim recognises `-jar takserver.war -Dspring.profiles.active=<p>` and
  starts the Rust `takserver --roles …` in the farm dir, against the
  `CoreConfig.xml` that TAKCL wrote.
- After a health probe, it writes the readiness lines TAKCL scrapes for.
- It passes every other `java` call (`takcl.jar`, `SchemaManager.jar`,
  `UserManager.jar`) through to the real JVM.

The blocker is TAKCL's Ignite-based online file-auth path. It needs its own
Phase 3 design.

## Versions recorded per run

`build-java` writes `.work/build-info.json`, and `test` copies it, together
with `image-id`, into every results directory. It records:

- the src commit and `git describe`;
- every image pin;
- the Gradle version and both checksums;
- the pins file path and its SHA-256;
- the engine version and the UTC build time.

## Security notes

- **No writes to `src/`.** Builds run in `.work/clone` (`git clone
  --no-hardlinks`) and nothing from the checkout is bind-mounted. Check with
  `git status --porcelain src/`, which must be empty.
- **Integrity (SR-11).**
  - Images are pinned by digest.
  - The wrapper jar SHA-256 and the exact `distributionUrl` are verified
    before any download.
  - Gradle verifies the distribution against `distributionSha256Sum`.
  - The pins file is parsed, not sourced, and fails closed.
- **Residual risk (POAM-2026-10-26-02).** Maven, Gradle Plugin Portal and
  Node/npm artifacts fetched by the Java build are not checksum-verified,
  because `src/` has no `gradle/verification-metadata.xml`. Apt packages
  in the two images are unpinned. All of this is test tooling outside the TOE.
- **Least privilege.**
  - Both containers run with `no-new-privileges`.
  - `takcl` and the builder drop all capabilities and run as non-root.
  - `db` keeps only the capabilities the postgres entrypoint needs.
  - The build container gets no `GITHUB_TOKEN` and no other secrets.
- **Network.** The `parity` network is `internal: true` and the DB has no
  published ports. Only `build-java` has egress.
- **Secrets.** The DB password is random per work dir (`openssl rand -hex 24`).
  - It is stored mode 0600 under `.work/` and passed by compose secret file
    and env file, never on a command line the harness controls.
  - There is no trust auth.
  - TAKCL itself passes it to `SchemaManager.jar` as `-password`, so it can
    appear in TAKCL debug logs. That is accepted: the DB is ephemeral and
    internal.
  - `clean` deletes it.
- **Artifacts.** `test` copies only `TEST_ARTIFACTS` and the TAKCL logs into
  a private staging directory. Before the host reads anything, it makes the
  copy owner-readable (`chmod -R u+rwX`; `docker cp` keeps the container's
  modes) and deletes every symlink, FIFO, socket or device file there (the
  run then fails with exit 4). A directory that still cannot be listed is a
  failure, never a pass. It moves the rest into the results directory and
  deletes `*.jks`, `*.p12`, `*.pfx`, `*.key`, `*.pem`, `*.csr`, `*.srl` and
  `*.truststore` files. `verify-junit` also rejects symlinks and special
  files, never follows them, and fails on an unreadable directory. The
  workflow repeats both deletions before the upload, and uploads nothing if
  it could not list the whole results tree.
- **Fail-secure.**
  - Every script uses `set -euo pipefail`.
  - An unknown command or target exits 2.
  - A missing prerequisite exits 3, never "skipped".
  - An integrity mismatch exits 4.
  - 0 executed tests is a FAIL.
  - TAKCL's exit status is propagated (unlike `testrunner.sh`).
- **Container policy deviation.** These images are test fixtures, never
  published or shipped.
  - They are based on `eclipse-temurin` and `postgis/postgis`, not UBI, to
    match the upstream reference environment.
  - The takcl container is not read-only, because TAKCL writes its server
    farm under `/opt/tak`.
  - The runtime image is a JRE with no compiler.

## Licence

The harness files in this directory are Apache-2.0 (see `../LICENSE`,
`../NOTICE`). The Java TAK Server built from `../../src` is GPLv3. Its build
output is used only as a local or CI test fixture and is never redistributed
with TAK-RS.
