#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# TAK-RS parity harness v0: build the Java TAK Server reference from src/ in a
# throwaway clone, package TAKCL into an image, run TAKCL suites against it
# with PostgreSQL/PostGIS, and verify the JUnit results.
#
# NIST SP 800-53 Rev 5: CM-2 (pinned baseline, pins.env), SR-11 (wrapper and
# distribution checksums, digest-pinned images), SA-11 (developer testing),
# SR-4 (provenance recorded in build-info.json).
#
# POAM: POAM-2026-10-26-02 - the Java build's Maven, Gradle-plugin and npm
# downloads are not hash-verified (containment is the control; see cmd_test).
#
# See README.md in this directory. Compatible with bash 3.2 (macOS).
set -euo pipefail

readonly EX_OK=0
readonly EX_FAIL=1
readonly EX_UNSUPPORTED=2
readonly EX_PREREQ=3
readonly EX_INTEGRITY=4

PARITY_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
readonly PARITY_DIR
readonly SELF="$PARITY_DIR/harness.sh"
readonly WORK_DIR="$PARITY_DIR/.work"
readonly COMPOSE_FILE="$PARITY_DIR/compose.yaml"
readonly FIXTURES_DIR="$PARITY_DIR/tests/fixtures"
readonly COMPOSE_PROJECT="tak-parity"
readonly BUILDER_IMAGE="tak-parity-java-build:local"
readonly MIN_FREE_GB=25

# Pins file override (used by the local integrity negative test; the file is
# still fully validated, and its SHA-256 is recorded in build-info.json).
PINS_FILE="${TAK_PARITY_PINS:-$PARITY_DIR/pins.env}"
RESULTS_DIR="$PARITY_DIR/results"
ENGINE="${CONTAINER_ENGINE:-docker}"
ENGINE_BIN=""
PINS_LOADED=0
CURRENT_CONTAINER=""
TMP_DIRS=""
VERSION_TAG=""

JDK_BUILD_IMAGE=""
JRE_RUN_IMAGE=""
POSTGIS_IMAGE=""
GRADLE_VERSION=""
GRADLE_DIST_SHA256=""
GRADLE_WRAPPER_JAR_SHA256=""

readonly RE_IMAGE='^[a-z0-9][a-z0-9._/-]*:[A-Za-z0-9_][A-Za-z0-9_.-]*@sha256:[0-9a-f]{64}$'
readonly RE_SHA256='^[0-9a-f]{64}$'
readonly RE_VERSION='^[0-9]+\.[0-9]+(\.[0-9]+)?$'
readonly RE_TEST_ID='^[A-Za-z][A-Za-z0-9_.]{0,127}$'
readonly RE_REF='^[A-Za-z0-9][A-Za-z0-9._/-]{0,199}$'
readonly RE_COMMIT='^[0-9a-f]{40}$'
readonly RE_VERSION_TAG='^[0-9]+\.[0-9]+-RELEASE-[0-9]+$'

# ---------------------------------------------------------------------------
# Utilities

log() { printf 'harness: %s\n' "$*" >&2; }

die() {
  local code=$1
  shift
  printf 'harness: error: %s\n' "$*" >&2
  exit "$code"
}

on_exit() {
  local rc=$?
  if [ -n "$CURRENT_CONTAINER" ] && [ -n "$ENGINE_BIN" ]; then
    "$ENGINE_BIN" rm -f "$CURRENT_CONTAINER" >/dev/null 2>&1 || true
  fi
  if [ -n "$TMP_DIRS" ]; then
    local d
    for d in $TMP_DIRS; do
      rm -rf "$d"
    done
  fi
  exit "$rc"
}
trap on_exit EXIT

make_tmp() {
  local d
  d=$(mktemp -d "${TMPDIR:-/tmp}/tak-parity.XXXXXX")
  TMP_DIRS="$TMP_DIRS $d"
  printf '%s\n' "$d"
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    die "$EX_PREREQ" "neither sha256sum nor shasum is available"
  fi
}

require_tool() {
  command -v "$1" >/dev/null 2>&1 || die "$EX_PREREQ" "required tool '$1' not found on PATH"
}

utc_stamp() { date -u +%Y%m%dT%H%M%SZ; }

usage() {
  cat <<'EOF'
Usage: harness.sh [--engine docker|podman] [--results <dir>]
                  [--version-tag <X.Y-RELEASE-N>|auto] <command>...

Commands run in order and stop at the first failure.

  preflight                 Check engine, compose v2, git, python3, openssl,
                            >= 25 GB free disk, repo root and pins.
  build-java [--ref <rev>]  Clone the repo to .work/clone, verify the Gradle
                            wrapper pins, build integrationTesterDir in the
                            JDK 17 builder container, write .work/build-info.json.
  image                     Build tak-parity-java:<src sha> from .work/tester.
  up                        Create the DB password and TAKCL env (mode 0600)
                            and start PostgreSQL/PostGIS (no published ports).
  test <TestId> [--target java|rust]
                            Run a TAKCL suite (default target java), collect
                            results under <results>/<UTC>-<TestId>/ and run
                            verify-junit. --target rust exits 2 until Phase 3.
  verify-junit <dir>        Require >= 1 TEST-*.xml, >= 1 executed test and
                            0 failures/errors (0 tests = FAIL).
  list                      List TAKCL tests.
  logs                      Save compose logs under <results>/.
  down                      Stop containers and remove the DB volume.
  clean                     down, then remove .work/ (results are kept).
  selftest                  Check verify-junit and the exit-code contract with
                            fixtures; needs no containers.

Options:
  --engine docker|podman    Container engine (default: $CONTAINER_ENGINE or docker).
  --results <dir>           Results directory (default: rust/parity/results).
  --version-tag <tag>|auto  Only when no tag is reachable from the build commit
                            (origin carries no tags): create an annotated tag
                            <X.Y-RELEASE-N> in the throwaway clone on the newest
                            first-parent merge "Merge pull request #N from
                            TAK-Product-Center/upstream/<tag>"; `auto` takes the
                            newest such merge. Never touches the real checkout.
  -h, --help                Show this help.

Exit codes: 0 ok, 1 test or build failure, 2 unsupported command/target or
bad usage, 3 missing prerequisite, 4 integrity check failed.

Environment:
  CONTAINER_ENGINE          Default engine.
  TAKCL_MEM_LIMIT           Memory limit of the takcl container (default 10g).
  TAK_PARITY_PINS           Alternate pins file (validated like pins.env).
  TAK_PARITY_ENGINE_BIN     Test only: engine binary name override.
EOF
}

# ---------------------------------------------------------------------------
# Pins (parsed, never sourced)

load_pins() {
  [ "$PINS_LOADED" -eq 1 ] && return 0
  [ -f "$PINS_FILE" ] || die "$EX_PREREQ" "pins file not found: $PINS_FILE"
  local line key val re n=0
  while IFS= read -r line || [ -n "$line" ]; do
    n=$((n + 1))
    case "$line" in
      '' | '#'*) continue ;;
    esac
    case "$line" in
      *=*) ;;
      *) die "$EX_INTEGRITY" "pins:$n: not KEY=VALUE" ;;
    esac
    key=${line%%=*}
    val=${line#*=}
    case "$key" in
      JDK_BUILD_IMAGE | JRE_RUN_IMAGE | POSTGIS_IMAGE) re=$RE_IMAGE ;;
      GRADLE_VERSION) re=$RE_VERSION ;;
      GRADLE_DIST_SHA256 | GRADLE_WRAPPER_JAR_SHA256) re=$RE_SHA256 ;;
      *) die "$EX_INTEGRITY" "pins:$n: unknown key '$key'" ;;
    esac
    if ! [[ $val =~ $re ]]; then
      die "$EX_INTEGRITY" "pins:$n: $key has an invalid or placeholder value '$val'"
    fi
    if [ -n "${!key}" ]; then
      die "$EX_INTEGRITY" "pins:$n: duplicate key '$key'"
    fi
    printf -v "$key" '%s' "$val"
  done <"$PINS_FILE"
  for key in JDK_BUILD_IMAGE JRE_RUN_IMAGE POSTGIS_IMAGE GRADLE_VERSION \
    GRADLE_DIST_SHA256 GRADLE_WRAPPER_JAR_SHA256; do
    [ -n "${!key}" ] || die "$EX_INTEGRITY" "pins: required key '$key' missing in $PINS_FILE"
  done
  PINS_LOADED=1
}

# ---------------------------------------------------------------------------
# Engine and compose

engine_present() {
  local bin="${TAK_PARITY_ENGINE_BIN:-$ENGINE}"
  command -v "$bin" >/dev/null 2>&1 || return 1
  "$bin" compose version >/dev/null 2>&1 || return 1
  ENGINE_BIN=$bin
  return 0
}

require_engine() {
  [ -n "$ENGINE_BIN" ] && return 0
  local bin="${TAK_PARITY_ENGINE_BIN:-$ENGINE}"
  command -v "$bin" >/dev/null 2>&1 ||
    die "$EX_PREREQ" "container engine '$bin' not found on PATH (install Docker or Podman with compose v2, or pass --engine)"
  "$bin" compose version >/dev/null 2>&1 ||
    die "$EX_PREREQ" "'$bin compose' is not available (compose v2 is required)"
  ENGINE_BIN=$bin
}

repo_root() {
  require_tool git
  local root
  root=$(git -C "$PARITY_DIR" rev-parse --show-toplevel 2>/dev/null) ||
    die "$EX_PREREQ" "not inside a git checkout; cannot find the repository root"
  [ -f "$root/src/build.gradle" ] || die "$EX_PREREQ" "repository root $root has no src/build.gradle"
  printf '%s\n' "$root"
}

build_info_field() {
  local file="$WORK_DIR/build-info.json"
  [ -f "$file" ] || die "$EX_PREREQ" "no $file; run 'harness.sh build-java' first"
  python3 -I -c 'import json, sys
v = json.load(open(sys.argv[1]))[sys.argv[2]]
print(v)' "$file" "$1"
}

src_sha() {
  local sha
  sha=$(build_info_field src_commit)
  [[ $sha =~ $RE_COMMIT ]] || die "$EX_INTEGRITY" "build-info.json src_commit is not a commit id: '$sha'"
  printf '%s\n' "$sha"
}

# Writes .work/pg-password and .work/takcl.env (mode 0600). Compose needs both
# files to exist for every command, so this runs before any compose call.
ensure_secrets() {
  require_tool openssl
  (umask 077 && mkdir -p "$WORK_DIR")
  chmod 700 "$WORK_DIR"
  local pw_file="$WORK_DIR/pg-password" env_file="$WORK_DIR/takcl.env" pw
  if [ ! -s "$pw_file" ]; then
    (umask 077 && openssl rand -hex 24 >"$pw_file.tmp" && mv "$pw_file.tmp" "$pw_file")
  fi
  chmod 600 "$pw_file"
  pw=$(cat "$pw_file")
  [[ $pw =~ ^[0-9a-f]{48}$ ]] || die "$EX_INTEGRITY" "$pw_file is malformed; run 'harness.sh clean'"
  (
    umask 077
    printf 'TAKCL_SERVER_POSTGRES_PASSWORD=%s\nTAKCL_SERVER0_DB_HOST=db\nTAKCL_ENABLE_DB=true\n' "$pw" >"$env_file.tmp"
    mv "$env_file.tmp" "$env_file"
  )
  chmod 600 "$env_file"
}

compose_env() {
  load_pins
  export POSTGIS_IMAGE
  if [ -f "$WORK_DIR/build-info.json" ]; then
    SRC_SHA=$(src_sha)
  else
    SRC_SHA=unbuilt
  fi
  export SRC_SHA
  ensure_secrets
}

compose() {
  "$ENGINE_BIN" compose --project-name "$COMPOSE_PROJECT" --file "$COMPOSE_FILE" "$@"
}

# ---------------------------------------------------------------------------
# Integrity checks (SR-11)

# Read-only check of <src>/gradle/wrapper against the pins. Exit 4 on mismatch.
verify_wrapper() {
  local src=$1
  local props="$src/gradle/wrapper/gradle-wrapper.properties"
  local jar="$src/gradle/wrapper/gradle-wrapper.jar"
  local expected_url url count existing actual
  [ -f "$props" ] || die "$EX_INTEGRITY" "missing $props"
  [ -f "$jar" ] || die "$EX_INTEGRITY" "missing $jar"
  count=$(grep -c '^distributionUrl=' "$props" || true)
  [ "$count" = 1 ] || die "$EX_INTEGRITY" "$props must contain exactly one distributionUrl (found $count)"
  url=$(sed -n 's/^distributionUrl=//p' "$props" | tr -d '\r')
  expected_url="https\\://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip"
  [ "$url" = "$expected_url" ] ||
    die "$EX_INTEGRITY" "distributionUrl is '$url', pinned '$expected_url'; update pins.env deliberately"
  existing=$(sed -n 's/^distributionSha256Sum=//p' "$props" | tr -d '\r')
  if [ -n "$existing" ] && [ "$existing" != "$GRADLE_DIST_SHA256" ]; then
    die "$EX_INTEGRITY" "distributionSha256Sum in $props ($existing) differs from the pin"
  fi
  actual=$(sha256_of "$jar")
  [ "$actual" = "$GRADLE_WRAPPER_JAR_SHA256" ] ||
    die "$EX_INTEGRITY" "gradle-wrapper.jar SHA-256 $actual does not match the pin $GRADLE_WRAPPER_JAR_SHA256"
  log "wrapper verified: Gradle $GRADLE_VERSION, wrapper jar $actual"
}

# Python (stdlib only, run with -I). Untrusted input: symlinks and special
# files anywhere under <dir> are rejected and never followed (SEC-003); an
# unreadable directory fails the walk (SEC-009); DTDs
# and entity declarations are rejected by expat itself, in any encoding
# (SEC-006), plus a raw-byte check; 64 MiB cap per file; counts taken as the
# max of attributes and elements.
readonly VERIFY_JUNIT_PY='
import fnmatch, os, stat, sys
import xml.etree.ElementTree as ET
import xml.parsers.expat

def fail(msg):
    print("verify-junit: FAIL: " + msg)
    sys.exit(1)

d = sys.argv[1]
try:
    st = os.lstat(d)
except OSError as e:
    fail("cannot stat %s (%s)" % (d, e))
if not stat.S_ISDIR(st.st_mode):
    fail("not a directory (or a symlink): " + d)
files = []
def walk_error(e):
    fail("cannot read %s (%s)" % (getattr(e, "filename", "?"), e))
for top, dirs, names in os.walk(d, onerror=walk_error, followlinks=False):
    for n in dirs + names:
        p = os.path.join(top, n)
        m = os.lstat(p).st_mode
        if not (stat.S_ISREG(m) or stat.S_ISDIR(m)):
            fail(p + ": symlink or special file is not accepted")
        if stat.S_ISREG(m) and fnmatch.fnmatchcase(n, "TEST-*.xml"):
            files.append(p)
files.sort()
if not files:
    fail("no TEST-*.xml under " + d)

class Rejected(Exception):
    pass

def reject(*_):
    raise Rejected()

def no_dtd(data, path):
    p = xml.parsers.expat.ParserCreate()
    p.StartDoctypeDeclHandler = reject
    p.EntityDeclHandler = reject
    p.UnparsedEntityDeclHandler = reject
    p.NotationDeclHandler = reject
    p.ExternalEntityRefHandler = reject
    try:
        p.Parse(data, True)
    except Rejected:
        fail(path + ": DTD or entity declarations are not accepted")
    except xml.parsers.expat.ExpatError as e:
        fail("%s: not well-formed XML (%s)" % (path, e))

def attr(el, name, path):
    v = el.get(name)
    if v is None:
        return 0
    try:
        n = int(float(v))
    except ValueError:
        fail("%s: attribute %s=%r is not a number" % (path, name, v))
    if n < 0:
        fail("%s: attribute %s is negative" % (path, name))
    return n

tot = {"tests": 0, "failures": 0, "errors": 0, "skipped": 0}
for path in files:
    if os.lstat(path).st_size > 64 * 1024 * 1024:
        fail(path + ": larger than 64 MiB")
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(fd, "rb") as fh:
        if not stat.S_ISREG(os.fstat(fh.fileno()).st_mode):
            fail(path + ": not a regular file")
        data = fh.read(64 * 1024 * 1024 + 1)
    if len(data) > 64 * 1024 * 1024:
        fail(path + ": larger than 64 MiB")
    if b"<!DOCTYPE" in data or b"<!ENTITY" in data:
        fail(path + ": DTD or entity declarations are not accepted")
    no_dtd(data, path)
    try:
        root = ET.fromstring(data)
    except ET.ParseError as e:
        fail("%s: not well-formed XML (%s)" % (path, e))
    if root.tag == "testsuite":
        suites = [root]
    elif root.tag == "testsuites":
        suites = list(root.iter("testsuite"))
    else:
        fail("%s: root element <%s> is not testsuite/testsuites" % (path, root.tag))
    for s in suites:
        cases = s.findall("testcase")
        c = {
            "tests": max(attr(s, "tests", path), len(cases)),
            "failures": max(attr(s, "failures", path), sum(1 for x in cases if x.find("failure") is not None)),
            "errors": max(attr(s, "errors", path), sum(1 for x in cases if x.find("error") is not None)),
            "skipped": max(attr(s, "skipped", path), sum(1 for x in cases if x.find("skipped") is not None)),
        }
        for k in tot:
            tot[k] += c[k]
        print("verify-junit: %s [%s] tests=%d failures=%d errors=%d skipped=%d"
              % (os.path.relpath(path, d), s.get("name", "?"), c["tests"], c["failures"], c["errors"], c["skipped"]))

executed = tot["tests"] - tot["skipped"]
summary = "files=%d tests=%d executed=%d failures=%d errors=%d skipped=%d" % (
    len(files), tot["tests"], executed, tot["failures"], tot["errors"], tot["skipped"])
if executed < 1:
    fail("0 tests executed = FAIL (" + summary + ")")
if tot["failures"] or tot["errors"]:
    fail(summary)
print("verify-junit: PASS: " + summary)
'

verify_junit() {
  require_tool python3
  python3 -I -c "$VERIFY_JUNIT_PY" "$1"
}

# ---------------------------------------------------------------------------
# Commands

cmd_preflight() {
  local misses=0 root free_kb free_gb
  load_pins
  log "pins: $PINS_FILE"
  log "  JDK_BUILD_IMAGE=$JDK_BUILD_IMAGE"
  log "  JRE_RUN_IMAGE=$JRE_RUN_IMAGE"
  log "  POSTGIS_IMAGE=$POSTGIS_IMAGE"
  log "  GRADLE_VERSION=$GRADLE_VERSION"
  if engine_present; then
    log "engine: $("$ENGINE_BIN" --version 2>&1 | head -1)"
    log "compose: $("$ENGINE_BIN" compose version 2>&1 | head -1)"
  else
    log "MISSING: container engine '${TAK_PARITY_ENGINE_BIN:-$ENGINE}' with compose v2 (install Docker or Podman, or pass --engine)"
    misses=$((misses + 1))
  fi
  local t
  for t in git python3 openssl; do
    if command -v "$t" >/dev/null 2>&1; then
      log "$t: $("$t" --version 2>&1 | head -1)"
    else
      log "MISSING: $t"
      misses=$((misses + 1))
    fi
  done
  if root=$(git -C "$PARITY_DIR" rev-parse --show-toplevel 2>/dev/null) && [ -f "$root/src/build.gradle" ]; then
    log "repo root: $root"
  else
    log "MISSING: repository root with src/build.gradle"
    misses=$((misses + 1))
  fi
  free_kb=$(df -Pk "$PARITY_DIR" | awk 'NR == 2 {print $4}')
  free_gb=$((free_kb / 1024 / 1024))
  if [ "$free_gb" -ge "$MIN_FREE_GB" ]; then
    log "free disk: ${free_gb} GB (>= ${MIN_FREE_GB} GB)"
  else
    log "MISSING: free disk ${free_gb} GB < ${MIN_FREE_GB} GB"
    misses=$((misses + 1))
  fi
  [ "$misses" -eq 0 ] || die "$EX_PREREQ" "preflight: $misses prerequisite(s) missing"
  log "preflight: ok"
}

# Prints "<commit> <tag>" for --version-tag: the newest first-parent commit
# reachable from <commit> in <repo> whose subject is an upstream release merge
# (for <spec> = auto) or the merge of release <spec>. Exit 3 if none.
resolve_version_tag() {
  local repo=$1 commit=$2 spec=$3 sha subject name
  case "$spec" in
    auto) ;;
    *) [[ $spec =~ $RE_VERSION_TAG ]] || die "$EX_UNSUPPORTED" "invalid --version-tag '$spec'" ;;
  esac
  while IFS=' ' read -r sha subject; do
    name=$(sed -n -E 's#^Merge pull request \#[0-9]+ from TAK-Product-Center/upstream/([0-9]+\.[0-9]+-RELEASE-[0-9]+)$#\1#p' <<<"$subject")
    [[ $name =~ $RE_VERSION_TAG ]] || continue
    if [ "$spec" = auto ] || [ "$spec" = "$name" ]; then
      [[ $sha =~ $RE_COMMIT ]] || continue
      printf '%s %s\n' "$sha" "$name"
      return 0
    fi
  done < <(git -C "$repo" log --first-parent --format='%H %s' "$commit")
  die "$EX_PREREQ" "--version-tag $spec: no matching upstream release merge on the first-parent history of $commit"
}

cmd_build_java() {
  local ref=$1 root commit clone desc tag uid gid tester
  load_pins
  require_tool python3
  root=$(repo_root)
  [ -n "$ref" ] || ref=HEAD
  commit=$(git -C "$root" rev-parse --verify --quiet "${ref}^{commit}") ||
    die "$EX_PREREQ" "revision '$ref' not found in $root"
  [[ $commit =~ $RE_COMMIT ]] || die "$EX_PREREQ" "revision '$ref' did not resolve to a commit"

  # (1) Throwaway clone; src/ in the checkout is never written.
  clone="$WORK_DIR/clone"
  (umask 077 && mkdir -p "$WORK_DIR")
  if [ -e "$clone" ]; then
    chmod -R u+w "$clone"
    rm -rf "$clone"
  fi
  log "cloning $root at $commit into $clone"
  git -c core.hooksPath=/dev/null clone --no-hardlinks --quiet --no-checkout "$root" "$clone"
  git -C "$clone" -c core.hooksPath=/dev/null -c advice.detachedHead=false \
    checkout --quiet -B tak-parity-build "$commit"

  # (2) src/build.gradle needs an annotated tag for git describe (Grgit).
  #     With no tag reachable at all (origin carries none, REL-006),
  #     --version-tag places one, in the clone only.
  if ! desc=$(git -C "$clone" describe 2>/dev/null); then
    local tag_at found
    if tag=$(git -C "$clone" describe --tags --abbrev=0 2>/dev/null); then
      tag_at="${tag}^{commit}"
      log "no annotated tag reachable; re-creating '$tag' as an annotated tag in the clone only"
    elif [ -n "$VERSION_TAG" ]; then
      found=$(resolve_version_tag "$clone" "$commit" "$VERSION_TAG")
      tag_at=${found%% *}
      tag=${found#* }
      log "no tag reachable; --version-tag $VERSION_TAG: creating annotated '$tag' at $tag_at in the clone only"
    else
      die "$EX_PREREQ" "no tag is reachable from $commit; src/build.gradle cannot derive a version (pass --version-tag <X.Y-RELEASE-N>|auto)"
    fi
    git -C "$clone" -c user.name="tak-parity harness" -c user.email="tak-parity@localhost.invalid" \
      -c tag.gpgSign=false -c tag.forceSignAnnotated=false \
      tag --annotate --force --message "tak-parity: version tag $tag (clone only)" \
      "$tag" "$tag_at" >/dev/null
    desc=$(git -C "$clone" describe) || die "$EX_PREREQ" "git describe still fails in the clone"
  fi
  log "git describe: $desc"

  # (3) Wrapper integrity, before any download.
  verify_wrapper "$clone/src"

  # (4) Enforce the distribution checksum in the clone only.
  if ! grep -q '^distributionSha256Sum=' "$clone/src/gradle/wrapper/gradle-wrapper.properties"; then
    printf 'distributionSha256Sum=%s\n' "$GRADLE_DIST_SHA256" >>"$clone/src/gradle/wrapper/gradle-wrapper.properties"
  fi

  # (5) Build in the JDK 17 container as the invoking (non-root) user.
  require_engine
  uid=$(id -u)
  gid=$(id -g)
  [ "$uid" -ne 0 ] || die "$EX_PREREQ" "refusing to build as root; run the harness as an unprivileged user"
  log "building builder image $BUILDER_IMAGE from $JDK_BUILD_IMAGE"
  "$ENGINE_BIN" build --file "$PARITY_DIR/docker/Dockerfile.java-build" \
    --build-arg "JDK_BUILD_IMAGE=$JDK_BUILD_IMAGE" \
    --build-arg "BUILDER_UID=$uid" --build-arg "BUILDER_GID=$gid" \
    --tag "$BUILDER_IMAGE" "$PARITY_DIR/docker" ||
    die "$EX_FAIL" "builder image build failed"
  mkdir -p "$WORK_DIR/gradle-home"
  local -a userns
  userns=()
  if [ "$(basename "$ENGINE_BIN")" = podman ]; then
    userns=(--userns=keep-id)
  fi
  log "running Gradle (takserver-package:buildIntegrationTesterDir); this takes a while"
  "$ENGINE_BIN" run --rm ${userns[@]+"${userns[@]}"} \
    --security-opt no-new-privileges:true --cap-drop ALL \
    --volume "$clone:/work" --volume "$WORK_DIR/gradle-home:/gradle-home" \
    --env GRADLE_USER_HOME=/gradle-home --workdir /work/src \
    "$BUILDER_IMAGE" \
    ./gradlew --no-daemon --console=plain takserver-package:buildIntegrationTesterDir ||
    die "$EX_FAIL" "Gradle build failed"

  # (6) Copy the tester dir out of the clone.
  tester="$clone/src/takserver-package/build/integrationTesterDir"
  [ -f "$tester/utils/takcl.jar" ] || die "$EX_FAIL" "build produced no $tester/utils/takcl.jar"
  [ ! -e "$tester/TEST_RESULTS" ] || die "$EX_FAIL" "$tester/TEST_RESULTS must not exist"
  rm -rf "$WORK_DIR/tester"
  cp -R "$tester" "$WORK_DIR/tester"

  # (7) Provenance record (SR-4).
  python3 -I -c 'import json, sys
keys = ["src_commit", "git_describe", "gradle_version", "gradle_dist_sha256",
        "gradle_wrapper_jar_sha256", "jdk_build_image", "jre_run_image",
        "postgis_image", "pins_file", "pins_file_sha256", "engine", "built_at_utc"]
info = dict(zip(keys, sys.argv[2:]))
info["harness"] = "tak-parity v0"
with open(sys.argv[1], "w") as fh:
    json.dump(info, fh, indent=2, sort_keys=True)
    fh.write("\n")' "$WORK_DIR/build-info.json" \
    "$commit" "$desc" "$GRADLE_VERSION" "$GRADLE_DIST_SHA256" "$GRADLE_WRAPPER_JAR_SHA256" \
    "$JDK_BUILD_IMAGE" "$JRE_RUN_IMAGE" "$POSTGIS_IMAGE" "$PINS_FILE" "$(sha256_of "$PINS_FILE")" \
    "$("$ENGINE_BIN" --version 2>&1 | head -1)" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  log "build-java: ok ($WORK_DIR/build-info.json)"
}

cmd_image() {
  local sha desc
  load_pins
  require_engine
  sha=$(src_sha)
  desc=$(build_info_field git_describe)
  [ -f "$WORK_DIR/tester/utils/takcl.jar" ] || die "$EX_PREREQ" "no $WORK_DIR/tester; run 'harness.sh build-java' first"
  log "building tak-parity-java:$sha from $JRE_RUN_IMAGE"
  "$ENGINE_BIN" build --file "$PARITY_DIR/docker/Dockerfile.takcl" \
    --build-arg "JRE_RUN_IMAGE=$JRE_RUN_IMAGE" \
    --build-arg "SRC_SHA=$sha" --build-arg "TAK_VERSION=$desc" \
    --tag "tak-parity-java:$sha" "$WORK_DIR/tester" ||
    die "$EX_FAIL" "image build failed"
  "$ENGINE_BIN" image inspect --format '{{.Id}}' "tak-parity-java:$sha" >"$WORK_DIR/image-id"
  log "image: ok (tak-parity-java:$sha, $(cat "$WORK_DIR/image-id"))"
}

cmd_up() {
  require_engine
  compose_env
  compose up --detach --wait db || die "$EX_FAIL" "database did not become healthy"
  log "up: ok (db healthy on internal network)"
}

# SEC-003: container output is untrusted. Anything that is neither a regular
# file nor a directory (symlink, FIFO, socket, device) is deleted and reported;
# returns 1 if anything was removed. Never follows links (find -P default).
# SEC-009: the tree is made owner-traversable first (docker cp keeps the
# container's modes, so a 0300 directory would hide its entries from find),
# and a find that fails (unreadable directory, I/O error) is a rejection,
# never a silent pass.
reject_special() {
  local dir=$1 bad
  if ! chmod -R u+rwX "$dir"; then
    log "reject_special: cannot make $dir readable; rejecting"
    return 1
  fi
  if ! bad=$(find "$dir" ! -type f ! -type d -print); then
    log "reject_special: find failed under $dir; rejecting"
    return 1
  fi
  [ -n "$bad" ] || return 0
  printf '%s\n' "$bad" | sed 's/^/harness: removed non-regular entry: /' >&2
  find "$dir" ! -type f ! -type d -delete || log "reject_special: delete failed under $dir"
  return 1
}

scrub_key_material() {
  local dir=$1 n
  n=$(find "$dir" -type f \( -iname '*.jks' -o -iname '*.p12' -o -iname '*.pfx' -o -iname '*.key' \
    -o -iname '*.pem' -o -iname '*.csr' -o -iname '*.srl' -o -iname '*.truststore' \) -print | wc -l | tr -d ' ')
  if [ "$n" -gt 0 ]; then
    find "$dir" -type f \( -iname '*.jks' -o -iname '*.p12' -o -iname '*.pfx' -o -iname '*.key' \
      -o -iname '*.pem' -o -iname '*.csr' -o -iname '*.srl' -o -iname '*.truststore' \) -exec rm -f {} +
    log "removed $n key/certificate file(s) from results"
  fi
}

cmd_test() {
  local id=$1 sha stamp out rc vrc f staging special
  require_engine
  compose_env
  sha=$SRC_SHA
  [ "$sha" != unbuilt ] || die "$EX_PREREQ" "run 'harness.sh build-java image' first"
  "$ENGINE_BIN" image inspect "tak-parity-java:$sha" >/dev/null 2>&1 ||
    die "$EX_PREREQ" "image tak-parity-java:$sha not found; run 'harness.sh image' first"
  stamp=$(utc_stamp)
  out="$RESULTS_DIR/$stamp-$id"
  [ ! -e "$out" ] || die "$EX_PREREQ" "$out already exists"
  mkdir -p "$RESULTS_DIR"
  # Container output lands in a private staging directory first and is
  # checked (reject_special) before anything on the host reads, scrubs,
  # verifies or writes next to it (SEC-003).
  staging=$(make_tmp)
  mkdir "$staging/out" "$staging/logs"
  CURRENT_CONTAINER="tak-parity-takcl-$stamp-$$"
  log "running TAKCL 'tests run $id' (container $CURRENT_CONTAINER)"
  set +e
  compose run -T --name "$CURRENT_CONTAINER" takcl tests run "$id"
  rc=$?
  set -e
  log "TAKCL exit status: $rc"
  "$ENGINE_BIN" cp "$CURRENT_CONTAINER:/opt/tak/TEST_RESULTS/TEST_ARTIFACTS/." "$staging/out/" ||
    log "warning: no TEST_ARTIFACTS to copy"
  for f in takcl-debug.log takcl-profiling.log; do
    "$ENGINE_BIN" cp "$CURRENT_CONTAINER:/opt/tak/$f" "$staging/logs/$f" || log "warning: $f not found"
  done
  "$ENGINE_BIN" rm -f "$CURRENT_CONTAINER" >/dev/null || log "warning: could not remove $CURRENT_CONTAINER"
  CURRENT_CONTAINER=""
  # SEC-009: docker cp keeps container modes; make the copy owner-readable
  # before anything inspects it (reject_special repeats this and fails
  # closed if it cannot).
  chmod -R u+rwX "$staging" || die "$EX_INTEGRITY" "cannot make the copied container output readable"
  special=0
  reject_special "$staging" || special=1
  for f in "$staging"/logs/*; do
    [ -f "$f" ] || continue
    [ ! -e "$staging/out/${f##*/}" ] || die "$EX_INTEGRITY" "TEST_ARTIFACTS already contains ${f##*/}"
    mv "$f" "$staging/out/"
  done
  for f in build-info.json image-id; do
    [ ! -e "$staging/out/$f" ] || die "$EX_INTEGRITY" "TEST_ARTIFACTS contains a reserved name: $f"
  done
  mv "$staging/out" "$out"
  reject_special "$out" || special=1
  scrub_key_material "$out"
  cp "$WORK_DIR/build-info.json" "$out/build-info.json"
  [ ! -f "$WORK_DIR/image-id" ] || cp "$WORK_DIR/image-id" "$out/image-id"
  [ "$special" -eq 0 ] || die "$EX_INTEGRITY" "container output contained symlinks or special files (removed; see above); results: $out"
  set +e
  verify_junit "$out"
  vrc=$?
  set -e
  log "results: $out"
  [ "$rc" -eq 0 ] || die "$EX_FAIL" "TAKCL reported failure (exit $rc)"
  [ "$vrc" -eq 0 ] || die "$EX_FAIL" "JUnit verification failed"
  log "test $id: PASS"
}

cmd_verify_junit() {
  verify_junit "$1" || die "$EX_FAIL" "JUnit verification failed for $1"
}

cmd_list() {
  require_engine
  compose_env
  [ "$SRC_SHA" != unbuilt ] || die "$EX_PREREQ" "run 'harness.sh build-java image' first"
  compose run -T --rm takcl tests list
}

cmd_logs() {
  local out
  require_engine
  compose_env
  out="$RESULTS_DIR/$(utc_stamp)-compose-logs"
  mkdir -p "$out"
  compose logs --no-color >"$out/compose.log" 2>&1 || die "$EX_FAIL" "compose logs failed"
  log "logs: $out/compose.log"
}

cmd_down() {
  require_engine
  compose_env
  compose down --volumes --remove-orphans || die "$EX_FAIL" "compose down failed"
  log "down: ok"
}

cmd_clean() {
  local had_engine=1
  if engine_present; then
    cmd_down
  else
    had_engine=0
    log "warning: engine '${TAK_PARITY_ENGINE_BIN:-$ENGINE}' not found; containers and volumes (if any) were not removed"
  fi
  if [ -e "$WORK_DIR" ]; then
    chmod -R u+w "$WORK_DIR"
    rm -rf "$WORK_DIR"
  fi
  log "clean: removed $WORK_DIR (results kept)"
  [ "$had_engine" -eq 1 ] || exit "$EX_PREREQ"
}

# Runs one selftest check in a subprocess; compares the exit status.
ST_PASS=0
ST_FAIL=0
st_expect() {
  local want=$1 desc=$2 got logf
  shift 2
  logf=$(mktemp "${ST_TMP}/log.XXXXXX")
  set +e
  "$@" >"$logf" 2>&1
  got=$?
  set -e
  if [ "$got" -eq "$want" ]; then
    ST_PASS=$((ST_PASS + 1))
    printf 'selftest: ok   %-58s exit %d\n' "$desc" "$got"
  else
    ST_FAIL=$((ST_FAIL + 1))
    printf 'selftest: FAIL %-58s exit %d, expected %d\n' "$desc" "$got" "$want"
    sed 's/^/    | /' "$logf"
  fi
}

# Selftest helper: resolve_version_tag must print a commit and <want>.
st_check_version_tag() {
  local got
  got=$(resolve_version_tag "$1" "$2" "$3")
  [[ ${got%% *} =~ $RE_COMMIT ]] && [ "${got#* }" = "$4" ]
}

# Selftest helper: calls reject_special the way cmd_test does (in an `||`
# list, where set -e is off), so a fail-open path cannot hide behind set -e.
st_reject_as_cmd_test() {
  local special=0
  reject_special "$1" || special=1
  [ "$special" -eq 0 ]
}

# Runs a harness function in a separate bash process that sources this
# script in library mode. Usage: st_lib <wrapper-sha|-> <function> <args>...
st_lib() {
  TAK_PARITY_LIB=1 TAK_PARITY_PINS="$PINS_FILE" bash -c '
set -euo pipefail
. "$1"
load_pins
if [ "$2" != - ]; then GRADLE_WRAPPER_JAR_SHA256=$2; fi
shift 2
"$@"' _ "$SELF" "$@"
}

cmd_selftest() {
  require_tool python3
  ST_TMP=$(make_tmp)
  local fx dir want root
  for fx in pass fail error empty; do
    dir="$ST_TMP/junit-$fx"
    mkdir -p "$dir"
    cp "$FIXTURES_DIR/junit-$fx.xml" "$dir/TEST-junit-$fx.xml"
    want=1
    [ "$fx" != pass ] || want=0
    st_expect "$want" "verify-junit junit-$fx.xml" bash "$SELF" verify-junit "$dir"
  done
  mkdir -p "$ST_TMP/no-xml"
  st_expect 1 "verify-junit dir without TEST-*.xml" bash "$SELF" verify-junit "$ST_TMP/no-xml"
  mkdir -p "$ST_TMP/dtd"
  printf '<?xml version="1.0"?>\n<!DOCTYPE x [<!ENTITY a "a">]>\n<testsuite tests="1"/>\n' >"$ST_TMP/dtd/TEST-dtd.xml"
  st_expect 1 "verify-junit rejects DTD/entities" bash "$SELF" verify-junit "$ST_TMP/dtd"
  # SEC-006: the same DTD in UTF-16 (no ASCII "<!DOCTYPE" bytes).
  mkdir -p "$ST_TMP/dtd16"
  python3 -I -c 'import sys
open(sys.argv[1], "wb").write("<?xml version=\"1.0\" encoding=\"UTF-16\"?>\n<!DOCTYPE t [<!ENTITY n \"5\">]>\n<testsuite tests=\"&n;\"><testcase name=\"a\"/></testsuite>\n".encode("utf-16"))' \
    "$ST_TMP/dtd16/TEST-dtd16.xml"
  st_expect 1 "verify-junit rejects a UTF-16 DTD/entity" bash "$SELF" verify-junit "$ST_TMP/dtd16"
  # SEC-003: symlinks and special files are rejected, never followed.
  mkdir -p "$ST_TMP/link"
  cp "$FIXTURES_DIR/junit-pass.xml" "$ST_TMP/link/TEST-pass.xml"
  ln -s /etc/hosts "$ST_TMP/link/x.txt"
  st_expect 1 "verify-junit rejects a symlink in the results" bash "$SELF" verify-junit "$ST_TMP/link"
  mkdir -p "$ST_TMP/link2"
  ln -s "$FIXTURES_DIR/junit-pass.xml" "$ST_TMP/link2/TEST-pass.xml"
  st_expect 1 "verify-junit rejects a symlinked TEST-*.xml" bash "$SELF" verify-junit "$ST_TMP/link2"
  mkdir -p "$ST_TMP/fifo"
  cp "$FIXTURES_DIR/junit-pass.xml" "$ST_TMP/fifo/TEST-pass.xml"
  mkfifo "$ST_TMP/fifo/TEST-fifo.xml"
  st_expect 1 "verify-junit rejects a FIFO" bash "$SELF" verify-junit "$ST_TMP/fifo"
  ln -s "$ST_TMP/junit-pass" "$ST_TMP/dirlink"
  st_expect 1 "verify-junit rejects a symlinked results dir" bash "$SELF" verify-junit "$ST_TMP/dirlink"
  mkdir -p "$ST_TMP/special/a"
  : >"$ST_TMP/special/a/ok.txt"
  ln -s /etc/hosts "$ST_TMP/special/a/l"
  st_expect 1 "reject_special removes a symlink and reports it" st_lib - reject_special "$ST_TMP/special"
  st_expect 0 "reject_special leaves only regular files and dirs" st_lib - reject_special "$ST_TMP/special"
  st_expect 0 "reject_special kept the regular file" test -f "$ST_TMP/special/a/ok.txt"
  # SEC-009: a symlink inside an owner-unreadable (0300) directory is still
  # found and removed (chmod u+rwX first); a find that fails is a rejection.
  mkdir -p "$ST_TMP/hidden/d"
  : >"$ST_TMP/hidden/d/ok.txt"
  ln -s /etc/hosts "$ST_TMP/hidden/d/l"
  chmod 0300 "$ST_TMP/hidden/d"
  st_expect 1 "reject_special finds a symlink in a 0300 directory" st_lib - st_reject_as_cmd_test "$ST_TMP/hidden"
  st_expect 1 "reject_special removed that symlink" test -L "$ST_TMP/hidden/d/l"
  st_expect 0 "reject_special kept the regular file in it" test -f "$ST_TMP/hidden/d/ok.txt"
  mkdir -p "$ST_TMP/findfail/d"
  mkdir -p "$ST_TMP/failbin"
  printf '#!/bin/sh\nexit 1\n' >"$ST_TMP/failbin/find"
  chmod +x "$ST_TMP/failbin/find"
  st_failing_find() { (export PATH="$ST_TMP/failbin:$PATH"; st_lib "$@"); }
  st_expect 1 "reject_special rejects when find fails" st_failing_find - st_reject_as_cmd_test "$ST_TMP/findfail"
  mkdir -p "$ST_TMP/unread/d"
  cp "$FIXTURES_DIR/junit-pass.xml" "$ST_TMP/unread/TEST-pass.xml"
  chmod 0000 "$ST_TMP/unread/d"
  if [ -r "$ST_TMP/unread/d" ]; then
    printf 'selftest: skip %-58s (running as root)\n' "verify-junit fails on an unreadable directory"
  else
    st_expect 1 "verify-junit fails on an unreadable directory" bash "$SELF" verify-junit "$ST_TMP/unread"
  fi
  chmod 0700 "$ST_TMP/unread/d"
  mkdir -p "$ST_TMP/forged"
  sed 's/failures="1"/failures="0"/' "$FIXTURES_DIR/junit-fail.xml" >"$ST_TMP/forged/TEST-forged.xml"
  st_expect 1 "verify-junit counts <failure> despite failures=0" bash "$SELF" verify-junit "$ST_TMP/forged"

  st_expect 0 "--help" bash "$SELF" --help
  st_expect 2 "no command" bash "$SELF"
  st_expect 2 "unknown command" bash "$SELF" frobnicate
  st_expect 2 "test StartupTests --target rust" bash "$SELF" test StartupTests --target rust
  st_expect 2 "test StartupTests --target bogus" bash "$SELF" test StartupTests --target bogus
  st_expect 2 "test with an invalid TestId" bash "$SELF" test 'Startup;Tests'
  st_expect 2 "build-java --ref with option injection" bash "$SELF" build-java --ref --upload-pack=x
  st_expect 2 "--engine nerdctl" bash "$SELF" --engine nerdctl preflight
  st_expect 2 "--version-tag with an invalid name" bash "$SELF" --version-tag 'x;y' preflight
  st_expect 2 "--version-tag without a value" bash "$SELF" --version-tag
  st_expect 2 "--target rust rejected before earlier commands run" bash "$SELF" up test StartupTests --target rust
  st_expect 3 "preflight with a missing engine" \
    env TAK_PARITY_ENGINE_BIN=tak-parity-no-such-engine bash "$SELF" --engine docker preflight
  st_expect 3 "up with a missing engine" \
    env TAK_PARITY_ENGINE_BIN=tak-parity-no-such-engine bash "$SELF" up

  sed 's/^GRADLE_DIST_SHA256=.*/GRADLE_DIST_SHA256=PLACEHOLDER-UNRESOLVED/' "$PINS_FILE" >"$ST_TMP/pins-placeholder.env"
  st_expect 4 "placeholder pin fails closed" \
    env TAK_PARITY_PINS="$ST_TMP/pins-placeholder.env" bash "$SELF" preflight
  sed 's/^JDK_BUILD_IMAGE=\(.*\)@sha256:.*/JDK_BUILD_IMAGE=\1/' "$PINS_FILE" >"$ST_TMP/pins-tag-only.env"
  st_expect 4 "image pin without digest fails closed" \
    env TAK_PARITY_PINS="$ST_TMP/pins-tag-only.env" bash "$SELF" preflight
  printf 'EXTRA_KEY=1\n' | cat "$PINS_FILE" - >"$ST_TMP/pins-unknown.env"
  st_expect 4 "unknown pin key fails closed" \
    env TAK_PARITY_PINS="$ST_TMP/pins-unknown.env" bash "$SELF" preflight

  # Wrapper integrity, on synthetic trees and read-only on the real src/.
  load_pins
  local good="$ST_TMP/wrapper-good/gradle/wrapper"
  mkdir -p "$good"
  printf 'synthetic wrapper jar\n' >"$good/gradle-wrapper.jar"
  printf 'distributionUrl=https\\://services.gradle.org/distributions/gradle-%s-bin.zip\n' \
    "$GRADLE_VERSION" >"$good/gradle-wrapper.properties"
  local fake_sha
  fake_sha=$(sha256_of "$good/gradle-wrapper.jar")
  st_expect 0 "verify_wrapper accepts a matching tree" \
    st_lib "$fake_sha" verify_wrapper "$ST_TMP/wrapper-good"
  st_expect 4 "verify_wrapper rejects a wrong wrapper jar SHA-256" \
    st_lib - verify_wrapper "$ST_TMP/wrapper-good"
  local badv="$ST_TMP/wrapper-badversion/gradle/wrapper"
  mkdir -p "$badv"
  cp "$good/gradle-wrapper.jar" "$badv/"
  printf 'distributionUrl=https\\://services.gradle.org/distributions/gradle-0.0.1-bin.zip\n' >"$badv/gradle-wrapper.properties"
  st_expect 4 "verify_wrapper rejects an unpinned Gradle version" \
    st_lib "$fake_sha" verify_wrapper "$ST_TMP/wrapper-badversion"
  local badh="$ST_TMP/wrapper-badhost/gradle/wrapper"
  mkdir -p "$badh"
  cp "$good/gradle-wrapper.jar" "$badh/"
  printf 'distributionUrl=https\\://example.invalid/gradle-%s-bin.zip\n' "$GRADLE_VERSION" >"$badh/gradle-wrapper.properties"
  st_expect 4 "verify_wrapper rejects another distribution host" \
    st_lib "$fake_sha" verify_wrapper "$ST_TMP/wrapper-badhost"
  if root=$(git -C "$PARITY_DIR" rev-parse --show-toplevel 2>/dev/null) && [ -d "$root/src/gradle/wrapper" ]; then
    st_expect 0 "pins match the real src/gradle/wrapper (read-only)" st_lib - verify_wrapper "$root/src"
  fi

  # REL-006: --version-tag resolution on a synthetic first-parent history.
  local vt="$ST_TMP/vt"
  vtgit() { git -C "$vt" -c user.name=selftest -c user.email=selftest@invalid -c commit.gpgsign=false "$@"; }
  git init -q "$vt"
  vtgit commit -q --allow-empty -m "Merge pull request #1 from TAK-Product-Center/upstream/5.6-RELEASE-35"
  vtgit commit -q --allow-empty -m "Merge pull request #2 from TAK-Product-Center/upstream/5.7-RELEASE-14"
  vtgit commit -q --allow-empty -m "rust: unrelated change"
  local vt_head
  vt_head=$(git -C "$vt" rev-parse HEAD)
  st_expect 0 "--version-tag auto picks the newest upstream release merge" \
    st_lib - st_check_version_tag "$vt" "$vt_head" auto 5.7-RELEASE-14
  st_expect 0 "--version-tag 5.6-RELEASE-35 picks that merge" \
    st_lib - st_check_version_tag "$vt" "$vt_head" 5.6-RELEASE-35 5.6-RELEASE-35
  st_expect 3 "--version-tag for a release not on the history fails" \
    st_lib - resolve_version_tag "$vt" "$vt_head" 9.9-RELEASE-1

  printf 'selftest: %d passed, %d failed\n' "$ST_PASS" "$ST_FAIL"
  [ "$ST_FAIL" -eq 0 ] || die "$EX_FAIL" "selftest failed"
}

# ---------------------------------------------------------------------------
# Library mode: st_lib (selftest) sources this script with TAK_PARITY_LIB=1
# so single functions can be exercised in a separate process.

if [ "${TAK_PARITY_LIB:-0}" = 1 ]; then
  return 0 2>/dev/null || exit 0
fi

# ---------------------------------------------------------------------------
# Argument parsing: the whole chain is validated before anything runs.

readonly US=$'\x1f'
QUEUE=()

while [ $# -gt 0 ]; do
  case "$1" in
    -h | --help)
      usage
      exit "$EX_OK"
      ;;
    --engine)
      [ $# -ge 2 ] || die "$EX_UNSUPPORTED" "--engine needs a value"
      ENGINE=$2
      shift 2
      ;;
    --engine=*)
      ENGINE=${1#--engine=}
      shift
      ;;
    --results)
      [ $# -ge 2 ] || die "$EX_UNSUPPORTED" "--results needs a value"
      RESULTS_DIR=$2
      shift 2
      ;;
    --results=*)
      RESULTS_DIR=${1#--results=}
      shift
      ;;
    --version-tag | --version-tag=*)
      if [ "$1" = --version-tag ]; then
        [ $# -ge 2 ] || die "$EX_UNSUPPORTED" "--version-tag needs <X.Y-RELEASE-N> or auto"
        VERSION_TAG=$2
        shift 2
      else
        VERSION_TAG=${1#--version-tag=}
        shift
      fi
      [ "$VERSION_TAG" = auto ] || [[ $VERSION_TAG =~ $RE_VERSION_TAG ]] ||
        die "$EX_UNSUPPORTED" "invalid --version-tag '$VERSION_TAG' (X.Y-RELEASE-N or auto)"
      ;;
    preflight | image | up | list | logs | down | clean | selftest)
      QUEUE+=("$1")
      shift
      ;;
    build-java)
      shift
      ref=""
      if [ "${1:-}" = --ref ]; then
        [ $# -ge 2 ] || die "$EX_UNSUPPORTED" "build-java --ref needs a revision"
        ref=$2
        [[ $ref =~ $RE_REF ]] || die "$EX_UNSUPPORTED" "invalid revision '$ref'"
        shift 2
      fi
      QUEUE+=("build-java${US}${ref}")
      ;;
    test)
      shift
      [ $# -ge 1 ] || die "$EX_UNSUPPORTED" "test needs a <TestId>"
      test_id=$1
      shift
      [[ $test_id =~ $RE_TEST_ID ]] || die "$EX_UNSUPPORTED" "invalid TestId '$test_id'"
      target=java
      if [ "${1:-}" = --target ]; then
        [ $# -ge 2 ] || die "$EX_UNSUPPORTED" "--target needs java or rust"
        target=$2
        shift 2
      else
        case "${1:-}" in
          --target=*)
            target=${1#--target=}
            shift
            ;;
        esac
      fi
      case "$target" in
        java) ;;
        # POAM: POAM-2026-10-26-06 - TAKCL cannot target an external server;
        # the Rust target fails closed (exit 2) until the Phase 3 design.
        rust) die "$EX_UNSUPPORTED" "Rust target not implemented until Phase 3; see parity/README.md §Rust target" ;;
        *) die "$EX_UNSUPPORTED" "unknown target '$target' (java|rust)" ;;
      esac
      QUEUE+=("test${US}${test_id}")
      ;;
    verify-junit)
      shift
      [ $# -ge 1 ] || die "$EX_UNSUPPORTED" "verify-junit needs a <dir>"
      QUEUE+=("verify-junit${US}$1")
      shift
      ;;
    *)
      die "$EX_UNSUPPORTED" "unknown command or option '$1' (see --help)"
      ;;
  esac
done

case "$ENGINE" in
  docker | podman) ;;
  *) die "$EX_UNSUPPORTED" "unsupported engine '$ENGINE' (docker|podman)" ;;
esac

case "$RESULTS_DIR" in
  /*) ;;
  *) RESULTS_DIR="$PWD/$RESULTS_DIR" ;;
esac

if [ ${#QUEUE[@]} -eq 0 ]; then
  usage >&2
  exit "$EX_UNSUPPORTED"
fi

for entry in "${QUEUE[@]}"; do
  cmd=${entry%%"$US"*}
  arg=""
  case "$entry" in
    *"$US"*) arg=${entry#*"$US"} ;;
  esac
  case "$cmd" in
    preflight) cmd_preflight ;;
    build-java) cmd_build_java "$arg" ;;
    image) cmd_image ;;
    up) cmd_up ;;
    test) cmd_test "$arg" ;;
    verify-junit) cmd_verify_junit "$arg" ;;
    list) cmd_list ;;
    logs) cmd_logs ;;
    down) cmd_down ;;
    clean) cmd_clean ;;
    selftest) cmd_selftest ;;
  esac
done
exit "$EX_OK"
