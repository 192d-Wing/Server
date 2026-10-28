#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Positive and negative self-tests for every scripts/check-*.sh and
# scripts/ci-*.sh, using throwaway fixtures under a temp directory
# (plan section 8.2). Run via `make policy-selftest`.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SA-11 (Developer Testing and Evaluation), SA-15
#   (Development Process, Standards, and Tools), CM-3(2) (Testing,
#   Validation, and Documentation of Changes)
#
# Marker strings, POA&M IDs and RUSTSEC IDs used in fixtures are assembled at
# run time so the real checks never see them in this file.
#
# Exit: 0 all cases behaved as expected, 1 otherwise.
# Fixtures hold literal backticks and ${{ }} expressions on purpose.
# shellcheck disable=SC2016
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(cd "$here/.." && pwd)
T=$(mktemp -d "${TMPDIR:-/tmp}/tak-rs-selftest.XXXXXX")
trap 'rm -rf "$T"' EXIT

# Fixtures must not inherit the caller's check inputs: CI exports BASE_REF for
# `make policy`, and the fixture trees have no git history to compare with.
# Cases that need one of these set it explicitly.
unset BASE_REF REPO_ROOT RUST_ROOT MAPPING_FILE MAPPING_PENDING ANNOTATIONS_STRICT

# Fixture workspaces live outside rust/, so pin the same toolchain explicitly.
if [ -z "${RUSTUP_TOOLCHAIN:-}" ] && command -v rustup >/dev/null 2>&1; then
  RUSTUP_TOOLCHAIN=$(cd "$root" && rustup show active-toolchain 2>/dev/null | awk 'NR == 1 { print $1 }') || true
  [ -n "$RUSTUP_TOOLCHAIN" ] && export RUSTUP_TOOLCHAIN
fi

CM="COMPLIANCE"; CM="$CM MAPPING:"
PAR="PARITY"; PAR="$PAR:"
P="POAM"; POAM_REAL="$P-2026-10-26-01"; POAM_FAKE="$P-2099-01-01-99"
ADV1="RUSTSEC-2099-""0001"; ADV2="RUSTSEC-2099-""0002"
SHA=0123456789abcdef0123456789abcdef01234567

passed=0
failed=0
log="$T/out.log"

# expect <exit> <description> <command...>
expect() {
  local want=$1 desc=$2 got=0
  shift 2
  "$@" >"$log" 2>&1 || got=$?
  if [ "$got" -eq "$want" ]; then
    passed=$((passed + 1)); printf 'ok    %s\n' "$desc"
  else
    failed=$((failed + 1)); printf 'FAIL  %s (exit %s, want %s)\n' "$desc" "$got" "$want"
    sed 's/^/        | /' "$log"
  fi
}

# expect_stdout <want> <description> <stdin> <command...>
expect_stdout() {
  local want=$1 desc=$2 input=$3 got
  shift 3
  got=$(printf '%s' "$input" | "$@" 2>&1) || true
  if [ "$got" = "$want" ]; then
    passed=$((passed + 1)); printf 'ok    %s\n' "$desc"
  else
    failed=$((failed + 1)); printf 'FAIL  %s (got %s, want %s)\n' "$desc" "$got" "$want"
  fi
}

# expect_file_stdout <want> <description> <stdin-file> <command...>
expect_file_stdout() {
  local want=$1 desc=$2 input=$3 got
  shift 3
  got=$("$@" <"$input" 2>&1) || true
  if [ "$got" = "$want" ]; then
    passed=$((passed + 1)); printf 'ok    %s\n' "$desc"
  else
    failed=$((failed + 1)); printf 'FAIL  %s (got %s, want %s)\n' "$desc" "$got" "$want"
  fi
}

S="$here"

# ---------------------------------------------------------------------------
echo "== check-unsafe"
U="$T/unsafe"; mkdir -p "$U/crates/a/src"
printf '// SPDX-License-Identifier: Apache-2.0\npub fn f() {\n    unsafe {}\n}\n' >"$U/crates/a/src/lib.rs"
expect 1 "unsafe without SAFETY comment fails" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"
printf '// SPDX-License-Identifier: Apache-2.0\npub fn f() {\n    // SAFETY: fixture, empty block.\n    unsafe {}\n}\n' >"$U/crates/a/src/lib.rs"
expect 0 "unsafe with SAFETY comment passes" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"
printf '// SPDX-License-Identifier: Apache-2.0\n//! Never uses unsafe code.\n#![forbid(unsafe_code)]\n' >"$U/crates/a/src/lib.rs"
expect 0 "unsafe in a comment and unsafe_code lint pass" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"

# ---------------------------------------------------------------------------
echo "== check-manifests"
M="$T/manifests"; mkdir -p "$M/a/src"
cat >"$M/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = ["a"]

[workspace.package]
version = "0.0.1"
edition = "2021"
license = "Apache-2.0"
publish = false

[workspace.lints.rust]
unsafe_code = "forbid"
EOF
echo '' >"$M/a/src/lib.rs"
manifest_good='[package]
name = "a"
version.workspace = true
edition.workspace = true
license.workspace = true
publish.workspace = true

[lints]
workspace = true'
printf '%s\n' "$manifest_good" >"$M/a/Cargo.toml"
expect 0 "conforming member passes" env RUST_ROOT="$M" bash "$S/check-manifests.sh"
printf '%s\n' "$manifest_good" | sed '/^\[lints\]/,$d' >"$M/a/Cargo.toml"
expect 1 "member without [lints] workspace = true fails" env RUST_ROOT="$M" bash "$S/check-manifests.sh"
printf '%s\n' "$manifest_good" | sed 's/^license.workspace = true/license = "GPL-3.0-only"/' >"$M/a/Cargo.toml"
expect 1 "member with license = GPL-3.0-only fails" env RUST_ROOT="$M" bash "$S/check-manifests.sh"
printf '%s\n' "$manifest_good" | sed 's/^license.workspace = true/license = "Apache-2.0"/' >"$M/a/Cargo.toml"
expect 1 "member without license.workspace = true fails" env RUST_ROOT="$M" bash "$S/check-manifests.sh"
printf '%s\n' "$manifest_good" | sed 's/^publish.workspace = true/publish = true/' >"$M/a/Cargo.toml"
expect 1 "member with publish = true fails" env RUST_ROOT="$M" bash "$S/check-manifests.sh"

# ---------------------------------------------------------------------------
echo "== check-annotations"
A="$T/annot"; mkdir -p "$A/crates/x/src" "$A/docs/compliance"
# Mapping fixture for rule (d): two headings, a Planned row, an N/A row and a
# "not claimed" row.
write_mapping() {
  printf '%s\n' '# Mapping fixture' '' '### CM-2(2) Automation Support' '' '### SI-7 Integrity' '' \
    '| Control | Baseline | Origination | Status | Phase | Note |' \
    '|---------|----------|-------------|--------|-------|------|' \
    '| CM-2 Baseline Configuration | L M H | Hybrid | Planned | 0 | fixture |' \
    '| SA-11 Developer Testing | M H | Hybrid | Planned | 0 | fixture |' \
    '| AC-6 Least Privilege | M H | System-specific | Planned | 4 | Not claimed in Phase 0 |' \
    '| PM-4 POA&M Process | — | Inherited | Not Applicable | Organizational |' \
    >"$A/docs/compliance/NIST_800-53_MAPPING.md"
}
write_mapping
write_rs() { printf '%s\n' "$@" >"$A/crates/x/src/lib.rs"; }
write_rs '// SPDX-License-Identifier: Apache-2.0' '//! Fixture.' '//!' "//! $CM" \
  '//! - NIST 800-53: CM-2(2), SI-7 (Software, Firmware, and Information Integrity)' \
  "// $PAR src/takserver-core/src/main/java/Foo.java#bar" 'pub fn f() {}'
expect 0 "well-formed block, Rev 5 IDs, parity and SPDX pass" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '//! Fixture without a licence header.' 'pub fn f() {}'
expect 1 "missing SPDX header fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '// SPDX-License-Identifier: Apache-2.0' "// $CM" '// Nothing mapped here.' 'pub fn f() {}'
expect 1 "mapping block without a control line fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '// SPDX-License-Identifier: Apache-2.0' "// $CM" '// - NIST 800-53: CM-2 (2)' 'pub fn f() {}'
expect 1 "control ID 'CM-2 (2)' fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '// SPDX-License-Identifier: Apache-2.0' "// $CM" '// - NIST 800-53: CM2' 'pub fn f() {}'
expect 1 "control ID 'CM2' fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '// SPDX-License-Identifier: Apache-2.0' "// $PAR see Foo.java" 'pub fn f() {}'
expect 1 "parity annotation without src/...#method fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
mkdir -p "$A/scripts"; write_rs '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'
printf '#!/usr/bin/env bash\necho no header\n' >"$A/scripts/x.sh"
expect 1 "script without SPDX header fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
rm -f "$A/scripts/x.sh"

if command -v git >/dev/null 2>&1; then
  G="$T/git"; mkdir -p "$G/rust/crates/x/src"
  gitc() { git -C "$G" -c user.name=selftest -c user.email=selftest@invalid -c commit.gpgsign=false "$@"; }
  mkdir -p "$G/rust/docs/compliance"
  cp "$A/docs/compliance/NIST_800-53_MAPPING.md" "$G/rust/docs/compliance/"
  git -C "$G" init -q
  git -C "$G" checkout -q -b main
  printf '%s\n' '// SPDX-License-Identifier: Apache-2.0' "// $CM" '// - NIST 800-53: CM-2' \
    'pub fn a() {}' "// $CM" '// - NIST 800-53: SA-11' 'pub fn b() {}' >"$G/rust/crates/x/src/lib.rs"
  gitc add -A; gitc commit -qm base
  gitc checkout -q -b feature
  expect 0 "unchanged block count vs BASE_REF passes" env RUST_ROOT="$G/rust" BASE_REF=main bash "$S/check-annotations.sh"
  printf '%s\n' '// SPDX-License-Identifier: Apache-2.0' "// $CM" '// - NIST 800-53: CM-2' \
    'pub fn a() {}' 'pub fn b() {}' >"$G/rust/crates/x/src/lib.rs"
  gitc commit -qam drop
  expect 1 "block count lower than BASE_REF fails" env RUST_ROOT="$G/rust" BASE_REF=main bash "$S/check-annotations.sh"
else
  echo "skip  BASE_REF cases (git not installed)"
fi

# ---------------------------------------------------------------------------
echo "== check-poam-ids"
PO="$T/poam"; mkdir -p "$PO/docs/compliance"
printf '# POA&M\n\n| ID | Weakness |\n|----|----------|\n| %s | fixture |\n' "$POAM_REAL" >"$PO/docs/compliance/POAM.md"
printf '# see %s\n' "$POAM_REAL" >"$PO/deny.toml"
expect 0 "referenced ID present in POAM.md passes" env RUST_ROOT="$PO" REPO_ROOT="$PO" bash "$S/check-poam-ids.sh"
printf '# see %s\n' "$POAM_FAKE" >"$PO/notes.toml"
expect 1 "referenced ID absent from POAM.md fails" env RUST_ROOT="$PO" REPO_ROOT="$PO" bash "$S/check-poam-ids.sh"
rm -f "$PO/notes.toml" "$PO/docs/compliance/POAM.md"
expect 1 "missing POAM.md fails" env RUST_ROOT="$PO" REPO_ROOT="$PO" bash "$S/check-poam-ids.sh"

# ---------------------------------------------------------------------------
echo "== check-poam-markers (NIST-010)"
PM="$T/poammark"
pm_reset() { rm -rf "$PM"; mkdir -p "$PM/rust/crates/a/src" "$PM/rust/docs" "$PM/.github/workflows"; }
pm() { env RUST_ROOT="$PM/rust" REPO_ROOT="$PM" bash "$S/check-poam-markers.sh"; }
# pm_case <exit> <description> <file under $PM> <line>
pm_case() { pm_reset; printf '%s\n' "$4" >"$PM/$3"; expect "$1" "$2" pm; }
pm_reset
expect 0 "no markers passes" pm
pm_case 1 "bare // marker in .rs -> FAIL" rust/crates/a/src/lib.rs "// $P: the bounds are per element"
pm_case 0 "// marker with an ID passes" rust/crates/a/src/lib.rs "// $P: $POAM_REAL - fixture"
pm_case 0 "# marker with an ID and no space passes" rust/deny.toml "# $P:$POAM_REAL"
pm_case 1 "bare /// doc-comment marker -> FAIL" rust/crates/a/src/lib.rs "/// $P: later"
pm_case 1 "bare /* marker -> FAIL" rust/crates/a/src/lib.rs "/* $P: later */"
pm_case 1 "bare * continuation marker -> FAIL" rust/crates/a/src/lib.rs " * $P: later"
pm_case 1 "marker whose ID is not the first token -> FAIL" rust/crates/a/src/lib.rs "// $P: see $POAM_REAL"
pm_case 1 "marker with a malformed ID (3-digit sequence) -> FAIL" rust/crates/a/src/lib.rs "// $P: ${POAM_REAL}1"
pm_case 1 "trailing # marker on a TOML value -> FAIL" rust/deny.toml "x = 1 # $P: later"
pm_case 1 "bare marker in Makefile -> FAIL" rust/Makefile "# $P: later"
pm_case 1 "bare marker in a rust-*.yml workflow -> FAIL" .github/workflows/rust-x.yml "# $P: later"
pm_case 0 "bare marker under docs/ is out of scope" rust/docs/x.toml "# $P: later"
pm_case 0 "marker text without a comment introducer is not a marker" rust/crates/a/src/lib.rs "let s = \"$P: x\";"
pm_case 0 "commented-out example citing the placeholder passes" rust/deny.toml "#   # $P: $P-YYYY-MM-DD-NN - <disposition>"
pm_case 1 "placeholder in a live marker -> FAIL" rust/deny.toml "# $P: $P-YYYY-MM-DD-NN - <disposition>"
# Sequencing ratchet: while check-poam-markers is outside POLICY_CHECKS (it is
# strict-only until WU10 assigns IDs to the bare markers in tak-cot), the real
# tree must still fail it; once it passes, it must join POLICY_CHECKS.
pm_seq() {
  local pc
  pc=$(awk '/^POLICY_CHECKS[[:space:]]*:=/ { g = 1 } g { print } g && !/\\$/ { exit }' "$root/Makefile")
  case "$pc" in
    *check-poam-markers*) bash "$S/check-poam-markers.sh" ;;
    *) if bash "$S/check-poam-markers.sh" >/dev/null 2>&1; then
         echo "real tree passes check-poam-markers: add it to POLICY_CHECKS in rust/Makefile (WU10)"; return 1
       fi ;;
  esac
}
expect 0 "check-poam-markers is in POLICY_CHECKS, or the real tree still has bare markers" pm_seq
pm_strict() { # the check-annotations-strict recipe calls the script
  awk '/^check-annotations-strict:/ { r = 1; next } r && !/^\t/ { exit } r' "$root/Makefile" |
    grep -qE 'scripts/check-poam-markers\.sh'
}
expect 0 "check-annotations-strict runs check-poam-markers" pm_strict

# ---------------------------------------------------------------------------
echo "== check-advisory-sync"
AS="$T/adv"; mkdir -p "$AS/.cargo"
write_deny() { printf '[advisories]\nversion = 2\nignore = [\n%s]\n\n[licenses]\nallow = []\n' "$1" >"$AS/deny.toml"; }
write_audit() { printf '[advisories]\nignore = [\n%s]\n\n[yanked]\nenabled = true\n' "$1" >"$AS/.cargo/audit.toml"; }
write_deny "    { id = \"$ADV1\", reason = \"fixture\" },
"
write_audit "    # fixture rationale
    # $P: $POAM_REAL - fixture
    \"$ADV1\",
"
expect 0 "identical lists with POAM lines pass" env RUST_ROOT="$AS" bash "$S/check-advisory-sync.sh"
write_deny "    { id = \"$ADV1\", reason = \"fixture\" },
    { id = \"$ADV2\", reason = \"fake, deny.toml only\" },
"
expect 1 "ignore in deny.toml only fails" env RUST_ROOT="$AS" bash "$S/check-advisory-sync.sh"
write_deny "    { id = \"$ADV1\", reason = \"fixture\" },
"
write_audit "    # fixture rationale without a POA&M line
    \"$ADV1\",
"
expect 1 "audit.toml entry without POAM line fails" env RUST_ROOT="$AS" bash "$S/check-advisory-sync.sh"

# ---------------------------------------------------------------------------
echo "== check-native-deps"
N="$T/native"; mkdir -p "$N/web/src" "$N/fakesys/src"
cat >"$N/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = ["web", "fakesys"]
EOF
cat >"$N/fakesys/Cargo.toml" <<'EOF'
[package]
name = "fakesys"
version = "0.1.0"
edition = "2021"
links = "foo"
build = "build.rs"
EOF
echo 'fn main() {}' >"$N/fakesys/build.rs"
echo '' >"$N/fakesys/src/lib.rs"
echo '' >"$N/web/src/lib.rs"
web_manifest() {
  printf '[package]\nname = "web"\nversion = "0.1.0"\nedition = "2021"\n' >"$N/web/Cargo.toml"
  [ "$1" = dep ] && printf '\n[dependencies]\nfakesys = { path = "../fakesys" }\n' >>"$N/web/Cargo.toml"
  [[ "$1" = devdep ]] && printf '\n[dev-dependencies]\nfakesys = { path = "../fakesys" }\n' >>"$N/web/Cargo.toml"
  (cd "$N" && cargo generate-lockfile --offline -q)
}
nd() { env RUST_ROOT="$N" NATIVE_ALLOWLIST="$T/allow.txt" WASM_ROOTS=web bash "$S/check-native-deps.sh"; }
web_manifest nodep
printf '# empty\n' >"$T/allow.txt"
expect 1 "crate with links not allow-listed fails" nd
printf 'fakesys native all # fixture\n' >"$T/allow.txt"
expect 0 "crate allow-listed for every target passes" nd
printf 'fakesys native x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu # fixture\n' >"$T/allow.txt"
expect 0 "linux-only entry, not reachable from wasm roots, passes" nd
web_manifest dep
expect 1 "linux-only entry reached from a wasm root fails" nd
web_manifest devdep
expect 1 "linux-only entry reached from a wasm root by a dev edge fails (links rule keeps the full graph)" nd
web_manifest dep
printf 'fakesys nativ all # bad class\n' >"$T/allow.txt"
expect 1 "malformed allow-list class fails" nd
printf 'fakesys native all\naws-lc-rs marker all # never on wasm32\n' >"$T/allow.txt"
expect 1 "aws-lc-* entry covering wasm32 fails" nd

echo "== fix-round-1: check-native-deps web/ members and wasm32 crypto names (WEB-001, WEB-004)"
N2="$T/native2"; mkdir -p "$N2/web/a/src" "$N2/web/b/src" "$N2/sha2/src"
printf '[workspace]\nresolver = "2"\nmembers = ["web/a", "web/b", "sha2"]\n' >"$N2/Cargo.toml"
printf '[package]\nname = "sha2"\nversion = "0.10.0"\nedition = "2021"\n' >"$N2/sha2/Cargo.toml"
printf '[package]\nname = "b"\nversion = "0.1.0"\nedition = "2021"\n' >"$N2/web/b/Cargo.toml"
for f in "$N2/web/a/src/lib.rs" "$N2/web/b/src/lib.rs" "$N2/sha2/src/lib.rs"; do echo '' >"$f"; done
web_a() {
  printf '[package]\nname = "a"\nversion = "0.1.0"\nedition = "2021"\n' >"$N2/web/a/Cargo.toml"
  [ "$1" = crypto ] && printf '\n[dependencies]\nsha2 = { path = "../../sha2" }\n' >>"$N2/web/a/Cargo.toml"
  [[ "$1" = devcrypto ]] && printf '\n[dev-dependencies]\nsha2 = { path = "../../sha2" }\n' >>"$N2/web/a/Cargo.toml"
  [[ "$1" = buildcrypto ]] && printf '\n[build-dependencies]\nsha2 = { path = "../../sha2" }\n' >>"$N2/web/a/Cargo.toml"
  [[ "$1" = transcrypto ]] && printf '\n[dependencies]\nb = { path = "../b" }\n' >>"$N2/web/a/Cargo.toml"
  (cd "$N2" && cargo generate-lockfile --offline -q)
}
nd2() { env RUST_ROOT="$N2" NATIVE_ALLOWLIST="$T/allow.txt" NATIVE_TARGETS=wasm32-unknown-unknown WASM_ROOTS="$1" bash "$S/check-native-deps.sh"; }
printf '# empty\n' >"$T/allow.txt"
web_a nodep
expect 0 "every web/ member in WASM_ROOTS passes" nd2 "a b"
expect 1 "web/ member missing from WASM_ROOTS fails" nd2 "a"
web_a crypto
expect 1 "crypto crate (sha2) in the wasm32 closure fails" nd2 "a b"
echo "== WU2: wasm32 crypto rule judges the shipped (normal-edge) closure"
web_a devcrypto
expect 0 "dev-only crypto crate (sha2) of a wasm32 root passes" nd2 "a b"
web_a buildcrypto
expect 0 "build-only crypto crate (sha2) of a wasm32 root passes" nd2 "a b"
printf '[package]\nname = "b"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\nsha2 = { path = "../../sha2" }\n' >"$N2/web/b/Cargo.toml"
web_a transcrypto
expect 1 "crypto crate reached by a chain of normal edges fails" nd2 "a b"
printf '[package]\nname = "b"\nversion = "0.1.0"\nedition = "2021"\n\n[dev-dependencies]\nsha2 = { path = "../../sha2" }\n' >"$N2/web/b/Cargo.toml"
expect 0 "crypto crate only behind a dev edge of a normal dependency passes" nd2 "a b"
printf '[package]\nname = "b"\nversion = "0.1.0"\nedition = "2021"\n' >"$N2/web/b/Cargo.toml"
web_a nodep

# ---------------------------------------------------------------------------
echo "== check-toolchain"
TC="$T/tc"; mkdir -p "$TC/rust" "$TC/.github/workflows"
printf '[workspace]\nmembers = []\n\n[workspace.package]\nrust-version = "1.99"\n' >"$TC/rust/Cargo.toml"
printf '[toolchain]\nchannel = "1.99.1"\n' >"$TC/rust/rust-toolchain.toml"
expect 0 "channel 1.99.1 with rust-version 1.99 passes" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf '[toolchain]\nchannel = "1.98.0"\n' >"$TC/rust/rust-toolchain.toml"
expect 1 "channel 1.98.0 with rust-version 1.99 fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf '[toolchain]\nchannel = "1.99.1"\n' >"$TC/rust/rust-toolchain.toml"
printf 'jobs:\n  a:\n    steps:\n      - uses: dtolnay/rust-toolchain@%s # v1\n' "$SHA" >"$TC/.github/workflows/rust-ci.yml"
expect 1 "workflow using dtolnay/rust-toolchain fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf 'jobs:\n  a:\n    steps:\n      - run: cargo +nightly build\n' >"$TC/.github/workflows/rust-ci.yml"
expect 1 "workflow using cargo +nightly fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"

# ---------------------------------------------------------------------------
echo "== check-workflows"
W="$T/wf"; WF="$W/.github/workflows"; mkdir -p "$WF" "$W/rust"
good_ci() {
  sed "s/@SHA/@$SHA/" <<'EOF'
# Fixture. Job IDs are the two-space-indented keys under `jobs:`.
name: rust-ci
on:
  pull_request:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  changes:
    runs-on: ubuntu-24.04
    outputs:
      rust: ${{ steps.d.outputs.rust }}
    steps:
      - uses: actions/checkout@SHA # v7.0.1
        with:
          persist-credentials: false
      - id: d
        run: |
          git diff --name-only HEAD~1 | rust/scripts/ci-changes.sh >> "$GITHUB_OUTPUT"
  workflow-lint:
    runs-on: ubuntu-24.04
    steps:
      - name: Check out
        uses: actions/checkout@SHA # v7.0.1
        with:
          persist-credentials: false
      - run: rust/scripts/check-workflows.sh
  test:
    needs: [changes]
    if: needs.changes.outputs.rust == 'true'
    runs-on: ubuntu-24.04
    steps:
      - run: make -C rust test
  sbom:
    needs: [changes]
    runs-on: ubuntu-24.04
    steps:
      - run: make -C rust sbom
  rust-ci-gate:
    if: always()
    needs: [changes, workflow-lint, test]
    runs-on: ubuntu-24.04
    steps:
      - env:
          NEEDS_JSON: ${{ toJSON(needs) }}
          RUST_CHANGED: ${{ needs.changes.outputs.rust }}
        run: rust/scripts/ci-gate.sh
EOF
}
cw() { env RUST_ROOT="$W/rust" REPO_ROOT="$W" bash "$S/check-workflows.sh"; }
good_ci >"$WF/rust-ci.yml"
expect 0 "conforming rust-ci.yml passes" cw
good_ci | sed "s|actions/checkout@$SHA # v7.0.1|actions/checkout@v4|" >"$WF/rust-ci.yml"
expect 1 "uses: actions/checkout@v4 (not SHA-pinned) fails" cw
good_ci | sed 's|^  pull_request:$|  pull_request_target:|' >"$WF/rust-ci.yml"
expect 1 "pull_request_target trigger fails" cw
# shellcheck disable=SC2016 # literal template expression is the fixture
good_ci | sed 's|make -C rust test|echo "${{ github.event.pull_request.title }}"|' >"$WF/rust-ci.yml"
expect 1 "github.event expression inside run: fails" cw
good_ci | sed '/^permissions:/d; /^  contents: read$/d' >"$WF/rust-ci.yml"
expect 1 "missing top-level permissions fails" cw
good_ci | sed 's|^  contents: read$|  contents: write|' >"$WF/rust-ci.yml"
expect 1 "top-level permissions other than contents: read fails" cw
good_ci | awk '/persist-credentials: false/ && !done { done = 1; next } { print }' >"$WF/rust-ci.yml"
expect 1 "checkout without persist-credentials: false fails" cw
good_ci | awk '{ print } /^  pull_request:$/ { print "    paths: [\"rust/**\"]" }' >"$WF/rust-ci.yml"
expect 1 "paths: under on.pull_request fails" cw
good_ci | sed 's|needs: \[changes, workflow-lint, test\]|needs: [changes, workflow-lint]|' >"$WF/rust-ci.yml"
expect 1 "job missing from rust-ci-gate needs fails" cw
good_ci | sed '/^    if: always()$/d' >"$WF/rust-ci.yml"
expect 1 "rust-ci-gate without if: always() fails" cw
good_ci >"$WF/rust-ci.yml"
printf 'name: rust-parity\non:\n  workflow_dispatch:\npermissions:\n  contents: read\njobs:\n  rust-ci-gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/rust-parity.yml"
expect 1 "second workflow defining job rust-ci-gate fails" cw
rm -f "$WF/rust-parity.yml"
good_ci | awk '{ if ($0 == "        run: |") $0 = "        run: &blk |" } { print }' >"$WF/rust-ci.yml"
expect 1 "YAML anchor on run: (hides the block from W5) fails" cw
good_ci | awk '{ print } /^      - run: make -C rust test$/ { print "      - *reused" }' >"$WF/rust-ci.yml"
expect 1 "YAML alias as a step fails" cw
good_ci >"$WF/rust-ci.yml"
# REL-001: a missing rust-ci.yml is a failure, never a vacuous pass.
printf 'name: rust-parity\non:\n  workflow_dispatch:\npermissions:\n  contents: read\njobs:\n  parity:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/rust-parity.yml"
rm -f "$WF/rust-ci.yml"
expect 1 "rust-parity.yml without rust-ci.yml fails" cw
rm -f "$WF/rust-parity.yml"
expect 1 "no rust-*.yml at all (rust-ci.yml missing) fails" cw
good_ci >"$WF/rust-ci.yml"
expect 0 "conforming rust-ci.yml restored passes" cw

# ---------------------------------------------------------------------------
echo "== ci-changes"
chg() { bash "$S/ci-changes.sh"; }
expect_stdout rust=false "src/foo.java -> rust=false" 'src/foo.java' chg
expect_stdout rust=true "rust/docs/x.md -> rust=true" 'rust/docs/x.md' chg
expect_stdout rust=true ".github/workflows/rust-ci.yml -> rust=true" '.github/workflows/rust-ci.yml' chg
expect_stdout rust=true ".github/dependabot.yml -> rust=true" '.github/dependabot.yml' chg
expect_stdout rust=true ".github/workflows/other.yml -> rust=true (allow-list, SEC-001)" '.github/workflows/other.yml' chg
expect_stdout rust=true "src/a + rust/b -> rust=true" "src/a
rust/b" chg
expect_stdout rust=true "empty input -> rust=true" '' chg
expect_stdout rust=true "__all__ -> rust=true" '__all__' chg

# ---------------------------------------------------------------------------
echo "== ci-gate"
cg() { env NEEDS_JSON="$1" RUST_CHANGED="$2" bash "$S/ci-gate.sh"; }
ok_base='"changes":{"result":"success"},"workflow-lint":{"result":"success"}'
expect 0 "rust=false, heavy jobs skipped -> pass" cg "{$ok_base,\"test\":{\"result\":\"skipped\"},\"clippy\":{\"result\":\"skipped\"}}" false
expect 1 "rust=false, one success and one failure -> fail" cg "{$ok_base,\"test\":{\"result\":\"success\"},\"clippy\":{\"result\":\"failure\"}}" false
expect 0 "rust=true, all success -> pass" cg "{$ok_base,\"test\":{\"result\":\"success\"}}" true
expect 1 "rust=true, one job skipped -> fail" cg "{$ok_base,\"test\":{\"result\":\"skipped\"}}" true
expect 1 "a job cancelled -> fail" cg "{$ok_base,\"test\":{\"result\":\"cancelled\"}}" true
expect 1 "changes failure (rust=true) -> fail" cg '{"changes":{"result":"failure"},"workflow-lint":{"result":"success"}}' true
expect 1 "changes failure (rust=false) -> fail" cg '{"changes":{"result":"failure"},"workflow-lint":{"result":"success"},"test":{"result":"skipped"}}' false
expect 1 "workflow-lint failure with rust=false -> fail" cg '{"changes":{"result":"success"},"workflow-lint":{"result":"failure"},"test":{"result":"skipped"}}' false
expect 1 "malformed NEEDS_JSON -> fail" cg '{"changes":' true
expect 1 "workflow-lint missing from needs -> fail" cg '{"changes":{"result":"success"}}' true

# ---------------------------------------------------------------------------
# QA additions (Phase 0 verify, qa-report.md). Edge cases for the helpers and
# the gate, beyond plan section 8.2.
echo "== qa: check-unsafe"
printf '// SPDX-License-Identifier: Apache-2.0\npub struct S(*mut u8);\nunsafe impl Send for S {}\n' >"$U/crates/a/src/lib.rs"
expect 1 "unsafe impl without SAFETY comment fails" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"
printf '// SPDX-License-Identifier: Apache-2.0\npub unsafe fn f() {}\n' >"$U/crates/a/src/lib.rs"
expect 1 "unsafe fn without SAFETY comment fails" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"
printf '// SPDX-License-Identifier: Apache-2.0\n// SAFETY: too far away.\n\n\n\npub fn f() {\n    unsafe {}\n}\n' >"$U/crates/a/src/lib.rs"
expect 1 "SAFETY comment more than 3 lines above fails" env RUST_ROOT="$U" bash "$S/check-unsafe.sh"

echo "== qa: check-annotations"
write_rs '// SPDX-License-Identifier: MIT' 'pub fn f() {}'
expect 1 "SPDX header with a licence other than Apache-2.0 fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '//! Fixture.' '//!' '//! Header comes too late.' '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'
expect 1 "SPDX header after line 3 fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_rs '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'

echo "== fix-round-1: check-annotations (d) IDs must be mapped (NIST-005)"
ann_ids() { write_rs '// SPDX-License-Identifier: Apache-2.0' "// $CM" "// - NIST 800-53: $1" 'pub fn f() {}'; }
ann_ids 'CM-2 (Baseline Configuration), SA-11 (Developer Testing)'
expect 0 "IDs with a Planned row in the mapping pass" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'CM-2(2), SI-7'
expect 0 "IDs with a ### heading in the mapping pass" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'PM-4 (Plan of Action and Milestones Process)'
expect 1 "ID whose mapping row is Not Applicable (disclaimed) fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'AC-6 (Least Privilege)'
expect 1 "ID whose mapping row says not claimed fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'SI-16 (Memory Protection)'
expect 1 "ID absent from the mapping fails" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
mkdir -p "$A/scripts"
printf 'SI-16  # fixture pending\n' >"$A/scripts/mapping-pending.txt"
expect 0 "absent ID listed in mapping-pending.txt is PENDING, not FAIL" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
expect 1 "pending ID fails with ANNOTATIONS_STRICT=1" env RUST_ROOT="$A" ANNOTATIONS_STRICT=1 bash "$S/check-annotations.sh"
rm -f "$A/scripts/mapping-pending.txt"
rm -f "$A/docs/compliance/NIST_800-53_MAPPING.md"
ann_ids 'CM-2'
expect 1 "annotated IDs with no mapping file fail" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
write_mapping
mkdir -p "$T/annot-repo/.github/workflows"
printf '# %s\n# - NIST 800-53: SI-17 (Fail-safe Procedures)\nname: rust-x\n' "$CM" >"$T/annot-repo/.github/workflows/rust-x.yml"
write_rs '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'
expect 1 "unmapped ID in a rust-*.yml workflow annotation fails" env RUST_ROOT="$A" REPO_ROOT="$T/annot-repo" bash "$S/check-annotations.sh"
rm -rf "$T/annot-repo"
expect 0 "fixture restored passes" env RUST_ROOT="$A" bash "$S/check-annotations.sh"

echo "== qa: check-toolchain"
printf '[toolchain]\nchannel = "stable"\n' >"$TC/rust/rust-toolchain.toml"
rm -f "$TC/.github/workflows/rust-ci.yml"
expect 1 "floating channel 'stable' fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf '[toolchain]\nchannel = "1.990.0"\n' >"$TC/rust/rust-toolchain.toml"
expect 1 "channel 1.990.0 does not match rust-version 1.99" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf '[toolchain]\nchannel = "1.99.1"\n' >"$TC/rust/rust-toolchain.toml"
printf 'jobs:\n  a:\n    steps:\n      - run: rustup default stable\n' >"$TC/.github/workflows/rust-ci.yml"
expect 1 "workflow running rustup default fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
printf 'jobs:\n  a:\n    steps:\n      - uses: dtolnay/rust-toolchain@%s # v1\n' "$SHA" >"$TC/.github/workflows/rust-extra.yaml"
printf 'jobs:\n  a:\n    steps:\n      - run: make test\n' >"$TC/.github/workflows/rust-ci.yml"
expect 1 "toolchain override in a rust-*.yaml (not .yml) file fails" env RUST_ROOT="$TC/rust" REPO_ROOT="$TC" bash "$S/check-toolchain.sh"
rm -f "$TC/.github/workflows/rust-extra.yaml"

echo "== qa: check-workflows"
good_ci | sed "s|actions/checkout@$SHA # v7.0.1|actions/checkout@0123456 # v7.0.1|" >"$WF/rust-ci.yml"
expect 1 "uses: pinned to a short SHA fails" cw
good_ci | sed "s|actions/checkout@$SHA # v7.0.1|actions/checkout@$SHA|" >"$WF/rust-ci.yml"
expect 1 "uses: SHA without '# vX' version comment fails" cw
good_ci | sed 's|^  pull_request:$|  workflow_run:|' >"$WF/rust-ci.yml"
expect 1 "workflow_run trigger fails" cw
# shellcheck disable=SC2016 # literal template expressions are the fixtures
good_ci | sed 's|make -C rust test|echo "${{ inputs.ref }}"|' >"$WF/rust-ci.yml"
expect 1 "inputs expression inside run: fails" cw
# shellcheck disable=SC2016
good_ci | sed 's|make -C rust test|echo "${{ needs.changes.outputs.rust }}"|' >"$WF/rust-ci.yml"
expect 1 "needs expression inside run: fails" cw
# shellcheck disable=SC2016
good_ci | sed 's|      - run: make -C rust test|      - run: echo "${{ github.event.pull_request.head.ref }}"|' >"$WF/rust-ci.yml"
expect 1 "github.event expression in a single-line run: fails" cw
good_ci | sed 's|^  contents: read$|  contents: read\n  pull-requests: write|' >"$WF/rust-ci.yml"
expect 1 "extra top-level write permission fails" cw
good_ci | sed '/^  contents: read$/d; s|^permissions:$|permissions: write-all|' >"$WF/rust-ci.yml"
expect 1 "permissions: write-all fails" cw
good_ci | sed '/^  contents: read$/d; s|^permissions:$|permissions: { contents: read }|' >"$WF/rust-ci.yml"
expect 0 "flow-form permissions { contents: read } passes" cw
good_ci | awk '{ print } /^  pull_request:$/ { print "    paths-ignore: [\"src/**\"]" }' >"$WF/rust-ci.yml"
expect 1 "paths-ignore: under on.pull_request fails" cw
good_ci | sed '/^  pull_request:$/d' >"$WF/rust-ci.yml"
expect 1 "rust-ci.yml without a pull_request trigger fails" cw
# shellcheck disable=SC2016
good_ci | sed 's|^    if: always()$|    if: ${{ always() }}|' >"$WF/rust-ci.yml"
expect 0 "rust-ci-gate with if: \${{ always() }} passes" cw
good_ci | sed 's|^    if: always()$|    if: success()|' >"$WF/rust-ci.yml"
expect 1 "rust-ci-gate with if: success() fails" cw
good_ci | sed 's|needs: \[changes, workflow-lint, test\]|needs: [changes, workflow-lint, test, sbom]|' >"$WF/rust-ci.yml"
expect 1 "sbom (best-effort) listed in gate needs fails" cw
good_ci >"$WF/rust-ci.yml"
sed "s/@$SHA # v7.0.1/@v4/" "$WF/rust-ci.yml" | sed 's/^name: rust-ci$/name: rust-extra/; /rust-ci-gate/,$d' >"$WF/rust-extra.yaml"
expect 1 "unpinned action in a rust-*.yaml (not .yml) file fails" cw
rm -f "$WF/rust-extra.yaml"
printf 'name: other\non:\n  push:\npermissions:\n  contents: read\njobs:\n  x:\n    name: rust-ci-gate\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/other.yml"
expect 1 "non-rust workflow with a job named rust-ci-gate fails" cw
rm -f "$WF/other.yml"

echo "== fix-round-1: check-workflows W3b, W5, W7, B2 (QA-002, SEC-004)"
good_ci | awk '{ print } /^  test:$/ { print "    continue-on-error: true" }' >"$WF/rust-ci.yml"
expect 1 "job-level continue-on-error on test fails" cw
good_ci | awk '{ print } /^      - run: make -C rust test$/ { print "        continue-on-error: true" }' >"$WF/rust-ci.yml"
expect 1 "step-level continue-on-error fails" cw
good_ci | awk '{ print } /^  sbom:$/ { print "    continue-on-error: true" }' >"$WF/rust-ci.yml"
expect 0 "job-level continue-on-error on sbom passes" cw
good_ci | awk '{ print } /^      - run: make -C rust sbom$/ { print "        continue-on-error: true" }' >"$WF/rust-ci.yml"
expect 1 "step-level continue-on-error inside sbom fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions:"; print "      contents: write"; print "      id-token: write" }' >"$WF/rust-ci.yml"
expect 1 "job-level permissions contents: write, id-token: write fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions: { contents: write }" }' >"$WF/rust-ci.yml"
expect 1 "job-level flow permissions { contents: write } fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions: write-all" }' >"$WF/rust-ci.yml"
expect 1 "job-level permissions: write-all fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions:"; print "      contents: read" }' >"$WF/rust-ci.yml"
expect 0 "job-level permissions contents: read passes" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions: { contents: read }" }' >"$WF/rust-ci.yml"
expect 0 "job-level flow permissions { contents: read } passes" cw
# shellcheck disable=SC2016 # literal template expressions are the fixtures
good_ci | sed 's|make -C rust test|echo "${{ github.head_ref }}"|' >"$WF/rust-ci.yml"
expect 1 "github.head_ref expression inside run: fails" cw
# shellcheck disable=SC2016
good_ci | sed 's|make -C rust test|echo "${{ steps.x.outputs.y }}"|' >"$WF/rust-ci.yml"
expect 1 "steps.*.outputs expression inside run: fails" cw
good_ci >"$WF/rust-ci.yml"
# shellcheck disable=SC2016
printf 'name: spoof\non:\n  push:\npermissions:\n  contents: read\njobs:\n  x:\n    name: "rust-ci-${{ %sgate%s }}"\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' "'" "'" >"$WF/spoof.yml"
expect 1 "other workflow: job name built with \${{ }} (rust-ci-\${{ 'gate' }}) fails" cw
printf 'name: spoof\non:\n  push:\npermissions:\n  contents: read\njobs: {rust-ci-gate: {runs-on: ubuntu-24.04, steps: [{run: "true"}]}}\n' >"$WF/spoof.yml"
expect 1 "other workflow: flow-style jobs: {rust-ci-gate: ...} fails" cw
printf 'name: spoof\non:\n  push:\npermissions:\n  contents: read\njobs:\n  "rust-ci-gate":\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof.yml"
expect 1 "other workflow: quoted job ID \"rust-ci-gate\" fails" cw
printf 'name: spoof\non:\n  push:\npermissions:\n  contents: read\njobs:\n  x:\n    name: Rust-CI rust-ci check\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof.yml"
expect 1 "other workflow: job name containing rust-ci fails" cw
printf 'name: other\non:\n  push:\npermissions:\n  contents: read\njobs:\n  build:\n    name: build docs\n    runs-on: ubuntu-24.04\n    steps:\n      - name: rust-ci is mentioned in a step name only\n        uses: actions/upload-artifact@%s # v7.0.2\n        with:\n          name: art-${{ github.run_id }}\n' "$SHA" >"$WF/spoof.yml"
expect 0 "other workflow with ordinary job names (step/input names ignored) passes" cw
rm -f "$WF/spoof.yml"
expect 0 "conforming rust-ci.yml passes after qa cases" cw

echo "== qa: ci-changes"
expect_stdout rust=false "src/rust/x.java (rust/ not at start) -> rust=false" 'src/rust/x.java' chg
expect_stdout rust=true "rustacean/x (not allow-listed) -> rust=true" 'rustacean/x' chg
expect_stdout rust=true ".github/dependabot.yml.orig (not allow-listed) -> rust=true" '.github/dependabot.yml.orig' chg
expect_stdout rust=true ".github/workflows/rust-parity.yml -> rust=true" '.github/workflows/rust-parity.yml' chg
expect_stdout rust=true ".github/workflows/rust-new.yaml -> rust=true" '.github/workflows/rust-new.yaml' chg
expect_stdout rust=true "whitespace-only input -> rust=true" '

  ' chg
expect_stdout rust=true "CRLF input src/a + rust/b -> rust=true" "$(printf 'src/a\r\nrust/b\r')" chg
expect_stdout rust=false "CRLF input src/a only -> rust=false" "$(printf 'src/a\r\nsrc/b\r')" chg
src_list=$(i=0; while [ "$i" -lt 40 ]; do printf 'src/takserver-core/src/main/java/com/bbn/F%s.java\n' "$i"; i=$((i + 1)); done)
expect_stdout rust=false "40 src/** paths (upstream-merge shape) -> rust=false" "$src_list" chg
expect_stdout rust=true "40 src/** paths plus one rust/ path last -> rust=true" "$src_list
rust/Cargo.lock" chg
# Unreadable stdin (a directory): `cat` fails, so the fail-safe verdict applies.
got=$(bash "$S/ci-changes.sh" <"$T" 2>/dev/null) || true
if [ "$got" = rust=true ]; then passed=$((passed + 1)); echo "ok    unreadable stdin -> rust=true"
else failed=$((failed + 1)); echo "FAIL  unreadable stdin (got '$got', want rust=true)"; fi

echo "== fix-round-1: ci-changes allow-list, NUL input, scale (SEC-001, SEC-002, QA-001)"
expect_stdout rust=true ".cargo/config.toml at the repo root -> rust=true" '.cargo/config.toml' chg
expect_stdout rust=true ".cargo/config at the repo root -> rust=true" '.cargo/config' chg
expect_stdout rust=true "rust-toolchain.toml at the repo root -> rust=true" 'rust-toolchain.toml' chg
expect_stdout rust=true "src/a + .cargo/config.toml -> rust=true" "src/a
.cargo/config.toml" chg
expect_stdout rust=false "utils/cluster/x.yml -> rust=false" 'utils/cluster/x.yml' chg
expect_stdout rust=false "top-level README.md, LICENSE.txt -> rust=false" "README.md
LICENSE.txt" chg
expect_stdout rust=true "docs/x.md (not top-level, not src/) -> rust=true" 'docs/x.md' chg
expect_stdout rust=true "git-quoted path line -> rust=true" '"rust/crates/x/tests/\303\251.rs"' chg
expect_stdout rust=true "unknown option -> rust=true" 'src/a' bash "$S/ci-changes.sh" --bogus
chgz() { bash "$S/ci-changes.sh" -z; }
printf 'src/a.java\000src/b.java\000' >"$T/z-src"
expect_file_stdout rust=false "-z: src/** only -> rust=false" "$T/z-src" chgz
printf 'src/a.java\000rust/crates/x/tests/\303\251.rs\000' >"$T/z-utf8"
expect_file_stdout rust=true "-z: non-ASCII rust/ path -> rust=true" "$T/z-utf8" chgz
printf 'README.md\nrust/x\000' >"$T/z-nl"
expect_file_stdout rust=true "-z: top-level name with an embedded newline -> rust=true" "$T/z-nl" chgz
printf '' >"$T/z-empty"
expect_file_stdout rust=true "-z: empty input -> rust=true" "$T/z-empty" chgz
printf '__all__\000' >"$T/z-all"
expect_file_stdout rust=true "-z: __all__ -> rust=true" "$T/z-all" chgz
if command -v git >/dev/null 2>&1; then
  ZG="$T/zgit"; mkdir -p "$ZG/rust/crates/x/tests" "$ZG/src"
  git -C "$ZG" init -q
  : >"$ZG/src/a.java"
  git -C "$ZG" add -A
  git -C "$ZG" -c user.name=selftest -c user.email=selftest@invalid -c commit.gpgsign=false commit -qm base
  printf '#[test] fn t() {}\n' >"$ZG/rust/crates/x/tests/$(printf '\303\251').rs"
  git -C "$ZG" add -A
  git -C "$ZG" -c user.name=selftest -c user.email=selftest@invalid -c commit.gpgsign=false commit -qm utf8
  git -C "$ZG" -c core.quotePath=false diff --no-renames --name-only -z HEAD^1 HEAD >"$T/z-git"
  expect_file_stdout rust=true "real git -z diff adding rust/.../\303\251.rs -> rust=true" "$T/z-git" chgz
  git -C "$ZG" -c core.quotePath=true diff --no-renames --name-only HEAD^1 HEAD >"$T/q-git"
  expect_file_stdout rust=true "real git quoted (core.quotePath=true) diff -> rust=true" "$T/q-git" chg
fi
# QA-001 regression: 4000 src/** paths (larger than the biggest upstream
# merge, 3678 paths) must give rust=false, fast.
big="$T/big-src"
awk 'BEGIN { for (i = 0; i < 4000; i++) printf "src/takserver-core/src/main/java/com/bbn/marti/sub%d/File%d.java\n", i % 50, i }' >"$big"
t0=$(date +%s)
expect_file_stdout rust=false "4000 src/** paths -> rust=false" "$big" chg
tr '\n' '\000' <"$big" >"$big.z"
expect_file_stdout rust=false "4000 src/** paths, -z -> rust=false" "$big.z" chgz
printf 'rust/Cargo.lock\n' >>"$big"
expect_file_stdout rust=true "4000 src/** paths plus rust/Cargo.lock last -> rust=true" "$big" chg
t1=$(date +%s)
if [ $((t1 - t0)) -le 10 ]; then
  passed=$((passed + 1)); echo "ok    three 4000-path runs took $((t1 - t0)) s (<= 10 s)"
else
  failed=$((failed + 1)); echo "FAIL  three 4000-path runs took $((t1 - t0)) s (> 10 s; QA-001 regression)"
fi

echo "== qa round 1: ci-changes allow-list edges and real upstream merges (SEC-001, SEC-002, QA-001)"
expect_stdout rust=true ".github/CODEOWNERS -> rust=true" '.github/CODEOWNERS' chg
expect_stdout rust=true "top-level file named .github -> rust=true" '.github' chg
expect_stdout rust=true "top-level Cargo.toml -> rust=true" 'Cargo.toml' chg
expect_stdout rust=true "top-level .gitignore -> rust=true" '.gitignore' chg
expect_stdout rust=true "top-level README.rs (not md/txt) -> rust=true" 'README.rs' chg
expect_stdout rust=true "nested src-like name xsrc/a -> rust=true" 'xsrc/a' chg
expect_stdout rust=true "nested utils-like name utilsx/a -> rust=true" 'utilsx/a' chg
printf 'src/a.java\000.cargo/config.toml\000' >"$T/z-cargo"
expect_file_stdout rust=true "-z: src/a + .cargo/config.toml -> rust=true" "$T/z-cargo" chgz
printf 'src/a.java\000.github/workflows/other.yml\000' >"$T/z-gh"
expect_file_stdout rust=true "-z: src/a + .github/workflows/other.yml -> rust=true" "$T/z-gh" chgz
printf '\000' >"$T/z-nul"
expect_file_stdout rust=true "-z: a single NUL (no paths) -> rust=true" "$T/z-nul" chgz
printf 'src/a.java\000src/b.java' >"$T/z-noterm"
expect_file_stdout rust=false "-z: last record without a NUL terminator, src/** only -> rust=false" "$T/z-noterm" chgz
printf 'src/a.java\000rust/x' >"$T/z-noterm-rust"
expect_file_stdout rust=true "-z: last record without a NUL terminator is rust/x -> rust=true" "$T/z-noterm-rust" chgz
# Every first-parent upstream release merge in this repository's history must
# give rust=false in the CI (-z) form (rust=true if it touches
# src/takserver-protobuf/, SEC-008), in linear time. Skipped (not counted)
# when the history is not available, e.g. a shallow CI checkout.
repo="$root/.."
if command -v git >/dev/null 2>&1 &&
  git -C "$repo" rev-parse -q --verify '5187abd46d827d37cfc5708805eced197a837e49^{commit}' >/dev/null 2>&1 &&
  git -C "$repo" rev-parse -q --verify 'a116b5e4^{commit}' >/dev/null 2>&1; then
  merges=$(git -C "$repo" log --first-parent --merges --format='%H %s' 5187abd46d827d37cfc5708805eced197a837e49 |
    awk '$0 ~ / from TAK-Product-Center\/upstream\// { print $1 }')
  nm=0
  t0=$(date +%s)
  for m in $merges; do
    git -C "$repo" -c core.quotePath=false diff --no-renames --name-only -z "$m^1" "$m" >"$T/merge.z"
    # SEC-008 (round 2): a merge that touches src/takserver-protobuf/ (a
    # Rust build input) must give rust=true; every other one rust=false.
    want=rust=false
    # grep without -q reads all input (no SIGPIPE to tr under pipefail).
    if tr '\000' '\n' <"$T/merge.z" | grep '^src/takserver-protobuf/' >/dev/null; then want=rust=true; fi
    expect_file_stdout "$want" "upstream merge ${m:0:8} ($(tr -cd '\000' <"$T/merge.z" | wc -c | tr -d ' ') paths, -z) -> $want" "$T/merge.z" chgz
    nm=$((nm + 1))
  done
  t1=$(date +%s)
  if [ "$nm" -ge 2 ] && [ $((t1 - t0)) -le 20 ]; then
    passed=$((passed + 1)); echo "ok    $nm upstream merges judged in $((t1 - t0)) s (<= 20 s, linear; QA-001)"
  else
    failed=$((failed + 1)); echo "FAIL  $nm upstream merges judged in $((t1 - t0)) s (want >= 2 merges, <= 20 s)"
  fi
else
  echo "skip  upstream merge history not available (shallow checkout)"
fi

# ---------------------------------------------------------------------------
echo "== fix-round-2: check-workflows on parsed YAML (QA-006, SEC-004)"
good_ci >"$WF/rust-ci.yml"
expect 0 "conforming rust-ci.yml passes (parsed rules)" cw
# QA-006: the five mutations that bypassed the line-based W3b/W7.
good_ci | awk '{ print } /^  test:$/ { print "    \"continue-on-error\": true" }' >"$WF/rust-ci.yml"
expect 1 "QA-006 1: \"continue-on-error\": true on job test fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    continue-on-error : true" }' >"$WF/rust-ci.yml"
expect 1 "QA-006 2: continue-on-error : true (space before colon) on test fails" cw
good_ci | awk '{ print } /^  rust-ci-gate:$/ { print "    \"continue-on-error\": true" }' >"$WF/rust-ci.yml"
expect 1 "QA-006 3: \"continue-on-error\": true on rust-ci-gate fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    \"permissions\": { contents: write }" }' >"$WF/rust-ci.yml"
expect 1 "QA-006 4: \"permissions\": { contents: write } on test fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    permissions : write-all" }' >"$WF/rust-ci.yml"
expect 1 "QA-006 5: permissions : write-all on test fails" cw
sq="'"
good_ci | awk -v l="        ${sq}continue-on-error${sq}: true" '{ print } /^      - run: make -C rust test$/ { print l }' >"$WF/rust-ci.yml"
expect 1 "single-quoted step-level continue-on-error fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    Continue-On-Error: true" }' >"$WF/rust-ci.yml"
expect 1 "case variant Continue-On-Error fails" cw
good_ci | awk '{ print } /^  test:$/ { print "    \"permissions\":"; print "      contents: read" }' >"$WF/rust-ci.yml"
expect 0 "quoted job-level permissions: contents: read passes" cw
# Other rules now on the parsed tree.
good_ci | sed "s|      - uses: actions/checkout@$SHA # v7.0.1|      - \"uses\": actions/checkout@v4|" >"$WF/rust-ci.yml"
expect 1 "quoted \"uses\": key with a tag pin fails (W1)" cw
good_ci | sed 's|          persist-credentials: false|          "persist-credentials" : true|' >"$WF/rust-ci.yml"
expect 1 "quoted persist-credentials : true fails (W4)" cw
# shellcheck disable=SC2016 # literal template expression is the fixture
good_ci | sed 's|      - run: make -C rust test|      - "run": echo "${{ github.head_ref }}"|' >"$WF/rust-ci.yml"
expect 1 "quoted \"run\": key with github.head_ref fails (W5)" cw
# shellcheck disable=SC2016
good_ci | awk '{ print } /^      - run: make -C rust test$/ { print "      - run: >-"; print "          echo ${{ github.event.issue.title }}" }' >"$WF/rust-ci.yml"
expect 1 "folded run: >- block with github.event fails (W5)" cw
good_ci | awk '{ print } /^  test:$/ { print "    timeout-minutes: 5"; print "    timeout-minutes: 6" }' >"$WF/rust-ci.yml"
expect 1 "duplicate key in a job fails (P3)" cw
good_ci | sed 's|^    runs-on: ubuntu-24.04$|    runs-on: !!str ubuntu-24.04|' >"$WF/rust-ci.yml"
expect 1 "explicit YAML tag fails (W6)" cw
printf 'jobs: [\n' >"$WF/rust-ci.yml"
expect 1 "invalid YAML fails closed (P0)" cw
good_ci | awk '{ print } /^  test:$/ { print "    name: rust-ci-gate" }' >"$WF/rust-ci.yml"
expect 1 "rust-ci.yml: a second job named rust-ci-gate fails (B2)" cw
# shellcheck disable=SC2016
good_ci | awk '{ print } /^  test:$/ { print "    name: ${{ github.event.pull_request.title }}" }' >"$WF/rust-ci.yml"
expect 1 "rust-ci.yml: job name with a non-matrix expression fails (B2)" cw
# shellcheck disable=SC2016
good_ci | awk '{ print } /^  test:$/ { print "    name: t-${{ matrix.x }}"; print "    strategy:"; print "      matrix:"; print "        x: [rust-ci-gate]" }' >"$WF/rust-ci.yml"
expect 1 "rust-ci.yml: matrix value rust-ci-gate in a job name fails (B2)" cw
# shellcheck disable=SC2016
good_ci | awk '{ print } /^  test:$/ { print "    name: t (${{ matrix.x }})"; print "    strategy:"; print "      matrix:"; print "        x: [a, b]" }' >"$WF/rust-ci.yml"
expect 0 "rust-ci.yml: job name with \${{ matrix.x }} and plain values passes" cw
good_ci | sed 's|^    needs: \[changes, workflow-lint, test\]$|    needs:\n      - changes\n      - workflow-lint\n      - test|' >"$WF/rust-ci.yml"
expect 0 "gate needs as a block list (parsed) passes" cw
# NIST-013: annotations-strict is a non-required job; it must stay out of the
# gate's needs.
good_ci | awk '/^  rust-ci-gate:$/ { print "  annotations-strict:"; print "    runs-on: ubuntu-24.04"; print "    steps:"; print "      - run: make -C rust check-annotations-strict" } { print }' >"$WF/rust-ci.yml"
expect 0 "non-required job annotations-strict outside the gate needs passes" cw
sed 's|needs: \[changes, workflow-lint, test\]|needs: [changes, workflow-lint, test, annotations-strict]|' "$WF/rust-ci.yml" >"$WF/rust-ci.yml.t" && mv "$WF/rust-ci.yml.t" "$WF/rust-ci.yml"
expect 1 "annotations-strict listed in the gate needs fails (B3)" cw
good_ci >"$WF/rust-ci.yml"
# SEC-004: the four B2 spoof forms in another workflow.
spoof_hdr='name: spoof\non:\n  push:\npermissions:\n  contents: read\njobs:\n'
# shellcheck disable=SC2059 # the header is a printf format on purpose
printf "${spoof_hdr}"'  x:\n    name: >-\n      rust-ci-gate\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof1.yml"
expect 1 "SEC-004 1: folded block-scalar name >- rust-ci-gate fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  x:\n    name:\n      rust-ci-gate\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof1.yml"
expect 1 "SEC-004 2: plain-scalar name on the next line fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  x:\n    name: "rust\\x2Dci-gate"\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof1.yml"
expect 1 "SEC-004 3: double-quoted escape in name (rust\\x2Dci-gate) fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  "rust\\x2Dci-gate":\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof1.yml"
expect 1 "SEC-004 4: escaped job ID \"rust\\x2Dci-gate\": fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  x:\n    name: &n rust-ci-gate\n    runs-on: ubuntu-24.04\n  y:\n    name: *n\n    runs-on: ubuntu-24.04\n' >"$WF/spoof1.yml"
expect 1 "other workflow: name through an anchor/alias fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  x:\n    name: ok\n    runs-on: ubuntu-24.04\n  x:\n    name: rust-ci-gate\n' >"$WF/spoof1.yml"
expect 1 "other workflow: duplicate job ID hiding a rust-ci-gate name fails" cw
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  docs:\n    name: build docs\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/spoof1.yml"
expect 0 "other workflow with an ordinary job passes" cw
rm -f "$WF/spoof1.yml"
# Fail closed when Ruby is missing or does not complete.
expect 1 "ruby missing -> FAIL (fail closed, no skip)" env CHECK_WORKFLOWS_RUBY=tak-rs-no-such-ruby RUST_ROOT="$W/rust" REPO_ROOT="$W" bash "$S/check-workflows.sh"
expect 1 "ruby replaced by a no-op (no sentinel) -> FAIL" env CHECK_WORKFLOWS_RUBY=true RUST_ROOT="$W/rust" REPO_ROOT="$W" bash "$S/check-workflows.sh"
expect 0 "conforming rust-ci.yml passes after round-2 cases" cw

echo "== fix-round-2: ci-changes Rust inputs under src/ (SEC-008)"
expect_stdout rust=true "src/takserver-protobuf/src/main/proto/x.proto -> rust=true" 'src/takserver-protobuf/src/main/proto/x.proto' chg
expect_stdout rust=true "src/a.java + src/takserver-protobuf/build.gradle -> rust=true" "src/a.java
src/takserver-protobuf/build.gradle" chg
expect_stdout rust=false "src/takserver-protobufx/a (prefix is anchored to the directory) -> rust=false" 'src/takserver-protobufx/a' chg
expect_stdout rust=false "src/takserver-core/x.java -> rust=false" 'src/takserver-core/x.java' chg
printf 'src/a.java\000src/takserver-protobuf/a.proto\000' >"$T/z-proto"
expect_file_stdout rust=true "-z: src/a + src/takserver-protobuf/a.proto -> rust=true" "$T/z-proto" chgz

echo "== fix-round-2: check-src-coupling (SEC-008)"
SC="$T/sc"; mkdir -p "$SC/rust/crates/p" "$SC/src/takserver-protobuf/src/main/proto" "$SC/src/takserver-core" "$SC/utils/x"
scc() { env RUST_ROOT="$SC/rust" REPO_ROOT="$SC" bash "$S/check-src-coupling.sh"; }
up='..''/..''/..'
printf 'fn main() {}\n' >"$SC/rust/crates/p/build.rs"
expect 0 "no parent-relative src/utils reference passes" scc
printf 'fn main() { let _ = "%s/src/takserver-protobuf/src/main/proto"; }\n' "$up" >"$SC/rust/crates/p/build.rs"
expect 0 "build.rs reading src/takserver-protobuf (a Rust input) passes" scc
printf 'fn main() { let _ = "%s/src/takserver-core/x.proto"; }\n' "$up" >"$SC/rust/crates/p/build.rs"
expect 1 "build.rs reading src/takserver-core (not a Rust input) fails" scc
printf '[dependencies]\nx = { path = "%s/utils/x" }\n' "$up" >"$SC/rust/crates/p/Cargo.toml"
printf 'fn main() {}\n' >"$SC/rust/crates/p/build.rs"
expect 1 "Cargo.toml path dependency into utils/ fails" scc
rm -f "$SC/rust/crates/p/Cargo.toml"
printf 'see %s/src/takserver-core\n' "$up" >"$SC/rust/crates/p/README.md"
expect 0 "a Markdown link into src/ is not a build input (passes)" scc
rm -f "$SC/rust/crates/p/README.md"
ln -s "$up/src/takserver-core" "$SC/rust/crates/p/core"
expect 1 "symlink from rust/ to src/takserver-core fails" scc
rm -f "$SC/rust/crates/p/core"
ln -s "$up/src/takserver-protobuf/src/main/proto" "$SC/rust/crates/p/proto"
expect 0 "symlink from rust/ to src/takserver-protobuf/... passes" scc
rm -f "$SC/rust/crates/p/proto"
ln -s "$up/README.md" "$SC/rust/crates/p/readme"
: >"$SC/README.md"
expect 1 "symlink from rust/ to the top-level README.md (allow-listed) fails" scc
rm -f "$SC/rust/crates/p/readme"
expect 0 "real tree: rust/ has no uncovered src/utils coupling" bash "$S/check-src-coupling.sh"

echo "== fix-round-2: check-annotations (d) disclaimers (NIST-013, NIST-014)"
write_mapping2() {
  printf '%s\n' '# Mapping fixture 2' '' \
    '### SI-7 Integrity' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None for the TOE. CI integrity is mapped elsewhere |' '' \
    '### SC-13 Cryptographic Protection' '' '| Field | Value |' '|---|---|' '| Status | Not Applicable |' \
    '| Implementation | — |' '' \
    '### SI-7(15) Code Authentication' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None |' '' \
    '### CA-7 Continuous Monitoring' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | Not claimed in Phase 0 |' '' \
    '## Notes' '' 'In Phase 0, `CA-5` is not claimed. RA-3 is not claimed either.' '' \
    '### CM-3 Configuration Change Control' '' '| Field | Value |' '|---|---|' '| Status | Planned |' '' \
    '## Revision history' '' '| 0.2 | "CM-3 is not claimed" sentence removed |' '' \
    '| Control | Baseline | Origination | Status | Phase | Note |' \
    '|---------|----------|-------------|--------|-------|------|' \
    '| PM-4 POA&M Process | — | Inherited | Not Applicable | Organizational |' \
    '| CM-2 Baseline Configuration | L M H | Hybrid | Planned | 0 | fixture |' \
    >"$A/docs/compliance/NIST_800-53_MAPPING.md"
}
write_mapping2
ann_ids 'SI-7 (Software, Firmware, and Information Integrity)'
expect 1 "NIST-014: ### block with Implementation 'None for the TOE' does not map (fails)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'SC-13 (Cryptographic Protection)'
expect 1 "NIST-014: ### block with Status Not Applicable does not map (fails)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'CA-7 (Continuous Monitoring)'
expect 1 "NIST-014: ### block with Implementation 'Not claimed' does not map (fails)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'SI-7(15) (Code Authentication)'
expect 0 "### block with a bare 'None' Implementation (Planned) still maps (passes)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'CM-2 (Baseline Configuration)'
expect 0 "Planned table row still maps (passes)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'CM-3 (Configuration Change Control)'
expect 0 "a quoted '\"CM-3 is not claimed\" sentence removed' history row is not a disclaimer (passes)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
printf 'CA-5  # fixture\nPM-4  # fixture\nSI-7  # fixture\nSI-16 # fixture\n' >"$A/scripts/mapping-pending.txt"
ann_ids 'CA-5 (Plan of Action and Milestones)'
expect 1 "NIST-013: pending ID with a '\`CA-5\` is not claimed' sentence fails (non-strict)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'PM-4 (Plan of Action and Milestones Process)'
expect 1 "NIST-013: pending ID with a Not Applicable row fails (non-strict)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'SI-7 (Software, Firmware, and Information Integrity)'
expect 1 "NIST-013: pending ID whose block says None for the TOE fails (non-strict)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
ann_ids 'SI-16 (Memory Protection)'
expect 0 "pending ID the mapping is silent about is still PENDING (passes)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
expect 1 "... and fails with ANNOTATIONS_STRICT=1" env RUST_ROOT="$A" ANNOTATIONS_STRICT=1 bash "$S/check-annotations.sh"
ann_ids 'RA-3 (Risk Assessment)'
expect 1 "plain 'RA-3 is not claimed' sentence disclaims RA-3 (fails)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"
rm -f "$A/scripts/mapping-pending.txt"
write_mapping
write_rs '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'
expect 0 "fixture restored passes (round 2)" env RUST_ROOT="$A" bash "$S/check-annotations.sh"

echo "== fix-round-2: rust-parity.yml strip step fails closed (SEC-009)"
real_parity="$root/../.github/workflows/rust-parity.yml"
if [ -f "$real_parity" ] && command -v ruby >/dev/null 2>&1; then
  env -u RUBYOPT -u RUBYLIB ruby --disable=gems -rpsych -e '
    d = Psych.safe_load(File.read(ARGV[0]))
    d["jobs"].each_value { |j| (j["steps"] || []).each { |s| (puts s["run"]; exit 0) if s["id"] == "strip" } }
    exit 1' "$real_parity" >"$T/strip.sh"
  grep -q 'clean=true' "$T/strip.sh"
  expect 0 "strip step extracted from the real rust-parity.yml" test -s "$T/strip.sh"
  strip() { env RESULTS_DIR="$1" GITHUB_OUTPUT="$T/strip.out" bash "$T/strip.sh"; }
  R="$T/res"; mkdir -p "$R/d"; : >"$R/d/ok.xml"; : >"$R/a.key"; ln -s /etc/hosts "$R/d/l"; chmod 0300 "$R/d"
  : >"$T/strip.out"
  expect 1 "symlink inside a 0300 directory: found, removed, step fails" strip "$R"
  expect 1 "... the symlink is gone" test -L "$R/d/l"
  expect 0 "... key material is gone, the regular file kept" test ! -e "$R/a.key" -a -f "$R/d/ok.xml"
  expect 0 "... clean=true (the cleaned tree may be uploaded)" grep -qx 'clean=true' "$T/strip.out"
  : >"$T/strip.out"
  expect 0 "clean results: step passes" strip "$R"
  expect 0 "... clean=true" grep -qx 'clean=true' "$T/strip.out"
  mkdir -p "$T/failbin"; printf '#!/bin/sh\nexit 1\n' >"$T/failbin/find"; chmod +x "$T/failbin/find"
  : >"$T/strip.out"
  expect 1 "find fails: step fails" env PATH="$T/failbin:$PATH" RESULTS_DIR="$R" GITHUB_OUTPUT="$T/strip.out" bash "$T/strip.sh"
  expect 1 "... and sets no clean output (upload skipped)" grep -q 'clean' "$T/strip.out"
  expect 0 "upload step requires steps.strip.outputs.clean == 'true'" \
    grep -qF "if: always() && steps.strip.outputs.clean == 'true'" "$real_parity"
else
  echo "skip  rust-parity.yml strip step (file or ruby not available)"
fi

# ---------------------------------------------------------------------------
echo "== qa round 2: parsed-YAML key forms, annotations-strict job, reject_special (QA-006, NIST-013, SEC-009)"
# QA-006: key spellings that psych (and GitHub) decode to continue-on-error.
# Inserted with bash `read -r`/printf '%s', not awk: BSD awk decodes `\u`
# escapes even in ENVIRON values, which would make the escaped-key case
# vacuous.
qa_add() {
  local l
  good_ci | while IFS= read -r l; do
    printf '%s\n' "$l"
    [ "$l" != "  test:" ] || printf '%s\n' "$1"
  done
}
qa_add '    ? continue-on-error
    : true' >"$WF/rust-ci.yml"
expect 1 "QA-006: explicit-key form '? continue-on-error' / ': true' on job test fails (W7)" cw
qa_add "    \"continue\\u002Don-error\": true" >"$WF/rust-ci.yml"
grep -qF 'u002D' "$WF/rust-ci.yml" || echo "FAIL  fixture lost its \\u escape (case would be vacuous)"
expect 1 "QA-006: unicode-escaped key \"continue\\u002Don-error\" on job test fails (W7)" cw
{ printf '\357\273\277'; qa_add '    "continue-on-error": true'; } >"$WF/rust-ci.yml"
expect 1 "QA-006: UTF-8 BOM before a quoted continue-on-error still fails (W7)" cw
{ printf '\357\273\277'; good_ci; } >"$WF/rust-ci.yml"
expect 0 "UTF-8 BOM alone does not break the parse (control)" cw
qa_add '    permissions:
      contents: read
      id-token: write' >"$WF/rust-ci.yml"
expect 1 "QA-006: job permissions contents: read plus id-token: write fails (W3b)" cw
good_ci >"$WF/rust-ci.yml"
expect 0 "conforming rust-ci.yml passes after qa round 2 workflow cases" cw
# NIST-013 (2): on the REAL rust-ci.yml the annotations-strict job exists,
# always runs (no if/needs), cannot report success on failure, runs the
# strict check, and is not in the gate's needs.
real_ci_r2="$root/../.github/workflows/rust-ci.yml"
if [ -f "$real_ci_r2" ] && command -v ruby >/dev/null 2>&1; then
  expect 0 "real rust-ci.yml: annotations-strict always runs, runs the strict check, not in rust-ci-gate needs" \
    env -u RUBYOPT -u RUBYLIB ruby --disable=gems -rpsych -e '
      j = Psych.safe_load(File.read(ARGV[0]))["jobs"]
      a = j["annotations-strict"] or abort("annotations-strict job missing")
      %w[if needs continue-on-error strategy].each { |k| abort("annotations-strict has #{k}") if a.key?(k) }
      (a["steps"] || []).any? { |s| s["run"].to_s.strip == "make check-annotations-strict" } or abort("no strict step")
      abort("in gate needs") if Array(j["rust-ci-gate"]["needs"]).include?("annotations-strict")' "$real_ci_r2"
else
  echo "skip  real rust-ci.yml annotations-strict job (file or ruby not available)"
fi
# SEC-009: reject_special in library mode, called as cmd_test calls it.
H="$root/parity/harness.sh"
qa_rs() { TAK_PARITY_LIB=1 bash -c 'source "$0"; special=0; reject_special "$1" || special=1; [ "$special" -eq 0 ]' "$H" "$1"; }
R2="$T/qa-r2-rs"; mkdir -p "$R2/a/out/d/e" "$R2/b/out" "$R2/bin"
: >"$R2/a/out/d/ok.txt"; ln -s /etc/hosts "$R2/a/out/d/e/l"; chmod 0000 "$R2/a/out/d/e" "$R2/a/out/d"
expect 1 "SEC-009: symlink under nested 0000 directories is found (rejection)" qa_rs "$R2/a"
expect 1 "... and removed" test -L "$R2/a/out/d/e/l"
expect 0 "... and the regular file is kept" test -f "$R2/a/out/d/ok.txt"
printf '#!/bin/sh\nexit 1\n' >"$R2/bin/chmod"; chmod +x "$R2/bin/chmod"
expect 1 "SEC-009: chmod failure in reject_special is a rejection, never clean" env PATH="$R2/bin:$PATH" bash -c "$(declare -f qa_rs); H='$H'; qa_rs '$R2/b'"
rm -f "$R2/bin/chmod"
expect 0 "reject_special: clean tree passes (control)" qa_rs "$R2/b"

# ---------------------------------------------------------------------------
echo "== fix-round-3: check-annotations (d) disclaimer forms (QA-007, NIST-016)"
write_mapping3() {
  printf '%s\n' '# Mapping fixture 3' '' \
    '### SR-5 Acquisition Strategies' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None claimed. Developer-side inputs exist |' '' \
    '### SR-6 Supplier Assessments' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | **None for the TOE** |' '' \
    '### SI-7 Integrity' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | Not implemented for the TOE |' '' \
    '### SA-24 Cyber Resiliency' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | `Not Applicable` (component) |' '' \
    '### CM-14 Signed Components' '' '| Field | Value |' '|---|---|' '| **Status** | _not applicable_ |' \
    '| Implementation | — |' '' \
    '### RA-5 Vulnerability Monitoring' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None |' '' \
    '### CA-2 Control Assessments' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None yet. Agent reviews are SA-11 |' '' \
    '### CM-7 Least Functionality' '' '| Field | Value |' '|---|---|' '| Status | Planned |' \
    '| Implementation | None in Phase 0 |' '' \
    '## Notes' '' 'In Phase 0, **AU-2** Is Not Claimed.' '' \
    '| Control | Baseline | Origination | Status | Note |' \
    '|---------|----------|-------------|--------|------|' \
    '| PE-3 Physical Access Control | L M H | Inherited (system level) | Not Applicable (component) | Facility control |' \
    '| PE-2 Physical Access Authorizations | L M H | Inherited (system level) | not applicable | Facility control |' \
    '| MA-2 Controlled Maintenance | L M H | Inherited (system level) | **Not Applicable** | Facility |' \
    '| MA-3 Maintenance Tools | M H | Inherited (system level) | N/A (component) | Facility |' \
    '| **CP-9** System Backup | L M H | Hybrid | Not Applicable (component) | emphasised ID cell |' \
    '| CM-2 Baseline Configuration | L M H | Hybrid | Planned | fixture |' \
    '| MA-4 Nonlocal Maintenance | L M H | Hybrid | Planned | Applicable: Phase 4 |' \
    >"$A/docs/compliance/NIST_800-53_MAPPING.md"
}
write_mapping3
ca() { env RUST_ROOT="$A" bash "$S/check-annotations.sh"; }
for c in 'PE-3|Not Applicable (component) row' 'PE-2|lower-case not applicable row' \
  'MA-2|**Not Applicable** row' 'MA-3|N/A (component) row' 'CP-9|row with an emphasised **ID** cell' \
  'SR-5|block Implementation None claimed' 'SR-6|block Implementation **None for the TOE**' \
  'SI-7|block Implementation Not implemented' 'SA-24|block Implementation `Not Applicable` (component)' \
  'CM-14|block Status _not applicable_ under a **Status** label' 'AU-2|sentence **AU-2** Is Not Claimed'; do
  ann_ids "${c%%|*} (fixture)"
  expect 1 "QA-007/NIST-016: ${c%%|*} cited in code, ${c#*|}: disclaimed (fails)" ca
done
for c in 'RA-5|bare None' 'CA-2|None yet' 'CM-7|None in Phase 0' 'CM-2|Planned row' 'MA-4|Planned row whose note says Applicable'; do
  ann_ids "${c%%|*} (fixture)"
  expect 0 "QA-007/NIST-016: ${c%%|*} (${c#*|}) still maps (passes)" ca
done
printf 'PE-3  # fixture\nSR-5  # fixture\n' >"$A/scripts/mapping-pending.txt"
ann_ids 'PE-3 (Physical Access Control)'
expect 1 "QA-007: PE-3 pending, Not Applicable (component) row: disclaimed, pending cannot excuse it (fails)" ca
grep -q 'PE-3 is now mapped' "$log" && echo "FAIL  ... PE-3 reported as now mapped" && failed=$((failed + 1))
ann_ids 'SR-5 (Acquisition Strategies)'
expect 1 "NIST-016: SR-5 pending, block None claimed: disclaimed (fails)" ca
rm -f "$A/scripts/mapping-pending.txt"
# On the REAL mapping: the first Not Applicable (component) row must classify
# as disclaimed, so a future rewording cannot silently disable rule (d).
real_map="$root/docs/compliance/NIST_800-53_MAPPING.md"
na_id=$(grep -E '^\|[[:space:]]*[A-Z][A-Z]-[0-9]+(\([0-9]+\))?[[:space:]].*\|[[:space:]]*Not Applicable' "$real_map" 2>/dev/null |
  head -n 1 | sed -E 's/^\|[[:space:]]*//; s/[[:space:]].*$//' || true)
if [ -n "$na_id" ]; then
  ann_ids "$na_id (fixture)"
  expect 1 "QA-007: real mapping: $na_id (first Not Applicable row) cited in code is disclaimed (fails)" \
    env RUST_ROOT="$A" MAPPING_FILE="$real_map" bash "$S/check-annotations.sh"
  ann_ids 'CM-2 (Baseline Configuration)'
  expect 0 "real mapping: CM-2 (### block, Partially Implemented) still maps (control)" \
    env RUST_ROOT="$A" MAPPING_FILE="$real_map" bash "$S/check-annotations.sh"
else
  failed=$((failed + 1)); echo "FAIL  real mapping has no Not Applicable table row (rule (d) cannot be exercised)"
fi
write_mapping
write_rs '// SPDX-License-Identifier: Apache-2.0' 'pub fn f() {}'
expect 0 "fixture restored passes (round 3)" ca

echo "== fix-round-3: check-src-coupling '..' segments (QA-008)"
printf 'fn main() { let _ = "%s/src/takserver-protobuf/../takserver-core/x"; }\n' "$up" >"$SC/rust/crates/p/build.rs"
expect 1 "QA-008: <up>/src/takserver-protobuf/<..>/takserver-core/x is judged as src/takserver-core (fails)" scc
printf 'fn main() { let _ = "%s/src/takserver-core/../takserver-protobuf/./src/main/proto"; }\n' "$up" >"$SC/rust/crates/p/build.rs"
expect 0 "QA-008: src/takserver-core/../takserver-protobuf/./... normalises to a Rust input (passes)" scc
printf 'fn main() { let _ = "%s/src/../../etc/x"; }\n' "$up" >"$SC/rust/crates/p/build.rs"
expect 1 "QA-008: a reference climbing above the repository root fails" scc
printf 'fn main() {}\n' >"$SC/rust/crates/p/build.rs"
expect 0 "check-src-coupling fixture restored passes" scc

echo "== fix-round-3: check-workflows W5 allow-list, W5b, W8, W9, discovery (SEC-010, SEC-011, SEC-012)"
good_ci >"$WF/rust-ci.yml"
# shellcheck disable=SC2016 # literal template expressions are the fixtures
qa_add '    name: ${{ matrix.n }}
    strategy:
      matrix:
        n: ["${{ format('"'"'rust-{0}-gate'"'"', '"'"'ci'"'"') }}"]' >"$WF/rust-ci.yml"
grep -qF "format('rust-{0}-gate', 'ci')" "$WF/rust-ci.yml" || echo "FAIL  m1 fixture lost its format() matrix value"
expect 1 "SEC-010 m1: job name \${{ matrix.n }} with matrix value \${{ format('rust-{0}-gate','ci') }} fails" cw
# shellcheck disable=SC2016
qa_add '    strategy:
      matrix: ${{ fromJSON(github.event.pull_request.title) }}' >"$WF/rust-ci.yml"
expect 1 "SEC-010/W5b: strategy.matrix from an expression fails (job without an expression name)" cw
for e in "format('{0}', github.event.pull_request.title)" 'GITHUB.EVENT.pull_request.body' "github['event'].x" \
  'env.TITLE' 'toJSON(github)' 'github.event_name'; do
  good_ci | E="$e" awk '{ if ($0 == "      - run: make -C rust test") $0 = "      - run: echo \"${{ " ENVIRON["E"] " }}\"" } { print }' >"$WF/rust-ci.yml"
  grep -qF "$e" "$WF/rust-ci.yml" || echo "FAIL  fixture lost expression $e"
  expect 1 "SEC-011 m3: \${{ $e }} inside run: fails (W5 allow-list)" cw
done
# shellcheck disable=SC2016
good_ci | sed 's|      - run: make -C rust test|      - run: echo "${{ github.event.x"|' >"$WF/rust-ci.yml"
expect 1 "SEC-011: unterminated \${{ inside run: fails" cw
# shellcheck disable=SC2016
qa_add '    strategy:
      matrix:
        t: [a, b]' | sed 's|      - run: make -C rust test|      - run: make -C rust test T=${{ matrix.t }} W=${{ github.workspace }} R=${{ runner.temp }} S=${{github.sha}}|' >"$WF/rust-ci.yml"
expect 0 "W5: allow-listed matrix./runner./github.workspace/github.sha in run: passes (control)" cw
good_ci >"$WF/rust-ci.yml"
# SEC-010 (c): dot-prefixed and upper-case-extension workflow files are scanned.
printf 'on: pull_request\njobs:\n  rust-ci-gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/.x.yml"
expect 1 "SEC-010 m6: dot-prefixed .github/workflows/.x.yml defining rust-ci-gate fails" cw
rm -f "$WF/.x.yml"
# shellcheck disable=SC2059 # the header is a printf format on purpose
printf "${spoof_hdr}"'  rust-ci-gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/.hidden.yaml"
expect 1 "SEC-010 (c): dot-prefixed .hidden.yaml (with permissions) defining rust-ci-gate fails (B2)" cw
rm -f "$WF/.hidden.yaml"
# shellcheck disable=SC2059
printf "${spoof_hdr}"'  rust-ci-gate:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: "true"\n' >"$WF/other.YML"
expect 1 "SEC-010 (c): other.YML (upper-case extension) defining rust-ci-gate fails" cw
rm -f "$WF/other.YML"
# SEC-010 (b): token permissions that can write check runs or statuses.
other_wf() { # <top-permissions-block> <job-permissions-block>
  printf 'name: other\non: pull_request\n%bjobs:\n  a:\n    runs-on: ubuntu-24.04\n%b    steps:\n      - run: gh api repos/o/r/check-runs -f name=x\n' "$1" "$2" >"$WF/other.yml"
}
other_wf 'permissions:\n  checks: write\n' ''
expect 1 "SEC-010 m7: other workflow with top-level checks: write fails (W9)" cw
other_wf 'permissions:\n  contents: read\n' '    permissions:\n      statuses: write\n'
expect 1 "SEC-010 (b): other workflow with job-level statuses: write fails (W9)" cw
other_wf 'permissions: write-all\n' ''
expect 1 "SEC-010 (b): other workflow with permissions: write-all fails (W9)" cw
other_wf 'permissions:\n  contents: read\n' '    permissions: WRITE-ALL\n'
expect 1 "SEC-010 (b): other workflow with job-level permissions: WRITE-ALL fails (W9)" cw
other_wf '' ''
expect 1 "SEC-010 (b): other workflow without top-level permissions (repository default token) fails (W9)" cw
other_wf 'permissions:\n  contents: write\n  checks: read\n  pull-requests: write\n' ''
expect 0 "other workflow with checks: read and unrelated write permissions passes (control)" cw
rm -f "$WF/other.yml"
# SEC-012: case variants of known keys.
good_ci | sed 's|^permissions:$|Permissions:|' >"$WF/rust-ci.yml"
expect 1 "SEC-012: top-level Permissions: fails (W8)" cw
good_ci | awk '{ print } /^  test:$/ { print "    Name: t" }' >"$WF/rust-ci.yml"
expect 1 "SEC-012: job-level Name: fails (W8)" cw
good_ci | sed "s|      - uses: actions/checkout@$SHA # v7.0.1|      - Uses: actions/checkout@$SHA # v7.0.1|" >"$WF/rust-ci.yml"
expect 1 "SEC-012: step-level Uses: fails (W8)" cw
good_ci | sed 's|^  contents: read$|  Contents: read|' >"$WF/rust-ci.yml"
expect 1 "SEC-012: permissions key Contents: fails (W8)" cw
good_ci >"$WF/rust-ci.yml"
expect 0 "conforming rust-ci.yml passes after round-3 cases" cw
if [ -d "$root/../.github/workflows" ]; then
  expect 0 "real .github/workflows pass check-workflows (round-3 rules)" bash "$S/check-workflows.sh"
fi

echo "== qa: ci-gate (needs list taken from the real rust-ci.yml)"
real_ci="$root/../.github/workflows/rust-ci.yml"
if [ -f "$real_ci" ]; then
  # Gate needs, minus changes/workflow-lint, as a JSON array.
  heavy=$(sed -n -E '/^  rust-ci-gate:/,$ s/^    needs:[[:space:]]*\[([^]]*)\].*/\1/p' "$real_ci" |
    tr ',' '\n' | sed -E 's/^[[:space:]]*//; s/[[:space:]]*$//' | { grep -vxE 'changes|workflow-lint' || true; } |
    jq -Rsc 'split("\n") | map(select(length > 0))')
  # needs_json <changes-result> <rust-output|""> <heavy-default> [job=result ...]
  needs_json() {
    local cr=$1 ro=$2 hd=$3
    shift 3
    jq -nc --arg cr "$cr" --arg ro "$ro" --arg hd "$hd" --argjson heavy "$heavy" '
      {changes: {result: $cr, outputs: (if $ro == "" then {} else {rust: $ro} end)},
       "workflow-lint": {result: "success", outputs: {}}}
      + ($heavy | map({key: ., value: {result: $hd, outputs: {}}}) | from_entries)
      + ($ARGS.positional | map(split("=") | {key: .[0], value: {result: .[1], outputs: {}}}) | from_entries)' \
      --args "$@"
  }
  if [ "$(jq length <<<"$heavy")" -ge 7 ]; then
    passed=$((passed + 1)); echo "ok    real gate needs list has >= 7 heavy jobs ($(jq -r 'join(",")' <<<"$heavy"))"
  else
    failed=$((failed + 1)); echo "FAIL  real gate needs list parsed as $heavy"
  fi
  expect 0 "src-only PR: rust=false, every real heavy job skipped -> pass" cg "$(needs_json success false skipped)" false
  expect 0 "Rust PR: rust=true, every real job success -> pass" cg "$(needs_json success true success)" true
  expect 1 "Rust PR: fmt failure -> fail" cg "$(needs_json success true success fmt=failure)" true
  expect 1 "Rust PR: lint (matrix) failure -> fail" cg "$(needs_json success true success lint=failure)" true
  expect 1 "Rust PR: supply-chain cancelled -> fail" cg "$(needs_json success true success supply-chain=cancelled)" true
  expect 1 "Rust PR: unknown result 'neutral' -> fail" cg "$(needs_json success true success doc=neutral)" true
  expect 1 "rust=false but a heavy job ran and succeeded -> fail" cg "$(needs_json success false skipped markdown=success)" false
  expect 1 "rust=false but a heavy job failed -> fail" cg "$(needs_json success false skipped test=failure)" false
  expect 1 "changes failed, output empty, heavy skipped -> fail" cg "$(needs_json failure '' skipped)" ''
  expect 1 "changes success but output missing, heavy skipped -> fail" cg "$(needs_json success '' skipped)" ''
  expect 1 "RUST_CHANGED 'FALSE' (not exactly false) treated as true -> fail" cg "$(needs_json success FALSE skipped)" FALSE
  expect 1 "workflow-lint skipped with rust=false -> fail" cg "$(needs_json success false skipped workflow-lint=skipped)" false
  expect 1 "changes missing from needs -> fail" cg '{"workflow-lint":{"result":"success"},"test":{"result":"skipped"}}' false
  expect 1 "job without a result field -> fail" cg '{"changes":{"result":"success"},"workflow-lint":{"result":"success"},"fmt":{}}' false
  expect 1 "NEEDS_JSON {} -> fail" cg '{}' false
  expect 1 "NEEDS_JSON [] -> fail" cg '[]' false
  expect 1 "NEEDS_JSON null -> fail" cg 'null' false
  expect 1 "NEEDS_JSON empty -> fail" cg '' false
  expect 1 "NEEDS_JSON unset -> fail" env -u NEEDS_JSON RUST_CHANGED=false bash "$S/ci-gate.sh"
else
  failed=$((failed + 1)); echo "FAIL  real rust-ci.yml not found at $real_ci"
fi

# ---------------------------------------------------------------------------
echo "== check-fuzz-policy (Phase 1 WU9; plan 8.6)"
# Fixture: the REAL rust/deny.toml and rust/fuzz/{deny.toml, Cargo.toml,
# native-allowlist.txt, rust-toolchain.toml} in a throwaway main workspace,
# so the cases also prove the committed files pass.
FP="$T/fuzzpol"
fp_reset() {
  rm -rf "$FP"; mkdir -p "$FP/a/src" "$FP/fuzz"
  cat >"$FP/Cargo.toml" <<'EOF'
[workspace]
resolver = "3"
members = ["a"]
exclude = ["fuzz"]
EOF
  printf '[package]\nname = "a"\nversion = "0.1.0"\nedition = "2021"\n' >"$FP/a/Cargo.toml"
  echo '' >"$FP/a/src/lib.rs"
  printf '# fixture\nversion = 4\n\n[[package]]\nname = "a"\nversion = "0.1.0"\n' >"$FP/Cargo.lock"
  cp "$root/deny.toml" "$FP/deny.toml"
  for f in deny.toml Cargo.toml native-allowlist.txt rust-toolchain.toml; do
    cp "$root/fuzz/$f" "$FP/fuzz/$f"
  done
  cp -R "$root/fuzz/fuzz_targets" "$FP/fuzz/fuzz_targets"
  printf '# fixture\nversion = 4\n' >"$FP/fuzz/Cargo.lock"
  mkdir -p "$FP/scripts" "$FP/repo/.github/workflows"
  cp "$root/scripts/fuzz-seeds.sh" "$FP/scripts/fuzz-seeds.sh"
  cp "$root/scripts/ci-changes.sh" "$FP/scripts/ci-changes.sh"
  cp "$root/../.github/workflows/rust-fuzz.yml" "$FP/repo/.github/workflows/rust-fuzz.yml"
}
cfp() { env RUST_ROOT="$FP" REPO_ROOT="$FP/repo" CARGO_NET_OFFLINE=true bash "$S/check-fuzz-policy.sh"; }
# fp_perl <file> <perl -0 substitution>: edit a fixture file in place.
# A mutation that matches nothing would leave the fixture unchanged and make the
# case pass or fail for the wrong reason, so an unchanged file aborts the run.
fp_perl() {
  local before
  before=$(cksum <"$FP/$1")
  perl -0pi -e "$2" "$FP/$1"
  if [[ "$(cksum <"$FP/$1")" = "$before" ]]; then
    echo "selftest: fp_perl mutation matched nothing in $1: $2" >&2
    exit 1
  fi
}
if [[ -d "$root/fuzz" ]]; then
  fp_reset
  expect 0 "real fuzz/ files in a conforming workspace pass" cfp
  expect 0 "real rust/ tree passes" env RUST_ROOT="$root" bash "$S/check-fuzz-policy.sh"

  fp_reset; rm -rf "$FP/fuzz"
  expect 1 "F1 fuzz/ missing -> FAIL" cfp
  fp_reset; rm "$FP/fuzz/Cargo.lock"
  expect 1 "F1 fuzz/Cargo.lock missing -> FAIL" cfp
  fp_reset; rm "$FP/fuzz/deny.toml"
  expect 1 "F1 fuzz/deny.toml missing -> FAIL" cfp

  fp_reset; fp_perl fuzz/Cargo.toml 's/^\[workspace\]\nmembers = \["\."\]\n//m'
  expect 1 "F2 fuzz/Cargo.toml without its own [workspace] -> FAIL" cfp

  fp_reset; fp_perl Cargo.toml 's/^exclude = \["fuzz"\]\n//m'
  expect 1 "F3 main workspace without exclude = [\"fuzz\"] -> FAIL" cfp
  fp_reset; fp_perl Cargo.toml 's/^exclude = \["fuzz"\]/exclude = ["other"]/m'
  expect 1 "F3 exclude lists another directory only -> FAIL" cfp
  fp_reset; mkdir -p "$FP/fuzz/inner/src"
  printf '[package]\nname = "inner"\nversion = "0.1.0"\nedition = "2021"\n' >"$FP/fuzz/inner/Cargo.toml"
  echo '' >"$FP/fuzz/inner/src/lib.rs"
  fp_perl Cargo.toml 's/members = \["a"\]/members = ["a", "fuzz\/inner"]/'
  expect 1 "F3 main-workspace member under fuzz/ -> FAIL" cfp

  fp_reset; printf '\n[dependencies]\nlibfuzzer-sys = "0.4"\n' >>"$FP/a/Cargo.toml"
  expect 1 "F4 member with a libfuzzer-sys dependency -> FAIL" cfp
  fp_reset; printf '\n[dev-dependencies]\nlibfuzzer-sys = "0.4"\n' >>"$FP/a/Cargo.toml"
  expect 1 "F4 member with a libfuzzer-sys dev-dependency -> FAIL" cfp
  fp_reset; printf '\n[target.x86_64-unknown-linux-gnu.dependencies]\nlf = { package = "libfuzzer-sys", version = "0.4" }\n' >>"$FP/a/Cargo.toml"
  expect 1 "F4 renamed, target-specific libfuzzer-sys dependency -> FAIL" cfp
  fp_reset; printf '\n[[package]]\nname = "libfuzzer-sys"\nversion = "0.4.13"\n' >>"$FP/Cargo.lock"
  expect 1 "F4 libfuzzer-sys in rust/Cargo.lock -> FAIL" cfp

  fp_reset; fp_perl fuzz/deny.toml 's/^yanked = "deny"$/yanked = "warn"/m'
  expect 1 "F5 shared line changed outside a delta block -> FAIL" cfp
  fp_reset; fp_perl deny.toml 's/^(    \{ crate = "vcpkg".*\n)/$1    { crate = "fixture-sys", reason = "fixture" },\n/m'
  expect 1 "F5 new ban in rust/deny.toml not copied to fuzz/deny.toml -> FAIL" cfp
  fp_reset; fp_perl deny.toml 's/^(    \{ crate = "vcpkg".*\n)/$1    { crate = "fixture-sys", reason = "fixture" },\n/m'
  fp_perl fuzz/deny.toml 's/^(    \{ crate = "vcpkg".*\n)/$1    { crate = "fixture-sys", reason = "fixture" },\n/m'
  expect 0 "F5 the same new ban in both files passes" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^ *# >>> FUZZ-DELTA replaces=0: libFuzzer licence.*?# <<< FUZZ-DELTA\n//ms'
  expect 1 "F5 missing NCSA delta block -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^ *# >>> FUZZ-DELTA replaces=0: libfuzzer-sys build script.*?# <<< FUZZ-DELTA\n//ms'
  expect 1 "F5 missing allow-build-scripts delta block -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/ *# >>> FUZZ-DELTA replaces=1: libfuzzer-sys may wrap cc.*?# <<< FUZZ-DELTA\n/    { crate = "cc", wrappers = ["aws-lc-fips-sys", "aws-lc-sys", "cmake"], reason = "C compilation only for AWS-LC" },\n/ms'
  expect 1 "F5 missing cc delta block (main cc line kept) -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/"cmake", "libfuzzer-sys"\]/"cmake", "libfuzzer-sys", "evil"]/'
  expect 1 "F5 cc delta with an extra wrapper -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/\["aws-lc-fips-sys", "aws-lc-sys", "cmake", "libfuzzer-sys"\]/["libfuzzer-sys"]/'
  expect 1 "F5 cc delta that drops the main wrappers -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^(    "libfuzzer-sys", .*\n)/$1    "evil-sys",\n/m'
  expect 1 "F5 extra crate inside a delta block -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/\{ allow = \["NCSA"\], crate = "libfuzzer-sys" \}/{ allow = ["GPL-3.0-only"], crate = "libfuzzer-sys" }/'
  expect 1 "F5 different licence in the exception block -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^yanked = "deny"\n/# >>> FUZZ-DELTA replaces=1: fixture\nyanked = "warn"\n# <<< FUZZ-DELTA\n/m'
  expect 1 "F5 delta block that replaces a policy line -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^unmaintained = "all"\n/# >>> FUZZ-DELTA replaces=1: fixture\n# dropped\n# <<< FUZZ-DELTA\n/m'
  expect 1 "F5 delta block that deletes a policy line -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^(    "libfuzzer-sys", .*\n)/$1    { allow = ["NCSA"], crate = "libfuzzer-sys" },\n/m'
  expect 1 "F5 NCSA exception in the wrong array -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/^(    # >>> FUZZ-DELTA replaces=0: libfuzzer-sys build script.*\n(?:.*\n)*?)    # <<< FUZZ-DELTA\n/$1/m'
  expect 1 "F5 unterminated delta block -> FAIL" cfp
  fp_reset; fp_perl fuzz/deny.toml 's/# >>> FUZZ-DELTA replaces=0: libfuzzer-sys build/# >>> FUZZ-DELTA replace=0: libfuzzer-sys build/'
  expect 1 "F5 malformed start marker -> FAIL" cfp
  fp_reset; printf '\n[sources.extra]\nx = 1\n' >>"$FP/fuzz/deny.toml"
  expect 1 "F5 extra lines after the shared policy -> FAIL" cfp
  fp_reset; printf '# FUZZ-DELTA\n' >>"$FP/deny.toml"; printf '# FUZZ-DELTA\n' >>"$FP/fuzz/deny.toml"
  expect 1 "F5 FUZZ-DELTA marker in rust/deny.toml -> FAIL" cfp

  fp_reset; printf 'cc native all # fixture\n' >>"$FP/fuzz/native-allowlist.txt"
  expect 1 "F6 allow-list with a second entry -> FAIL" cfp
  fp_reset; fp_perl fuzz/native-allowlist.txt 's/^libfuzzer-sys /honggfuzz /m'
  expect 1 "F6 allow-list naming another crate -> FAIL" cfp

  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^channel = ".*"$/channel = "nightly"/m'
  expect 1 "F7 floating nightly -> FAIL" cfp
  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^channel = ".*"$/channel = "1.99.1"/m'
  expect 1 "F7 stable channel -> FAIL" cfp
  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^channel = ".*"$/channel = "nightly-2026-13-01"/m'
  expect 1 "F7 impossible date -> FAIL" cfp
  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^channel = ".*"$/channel = "nightly-2026-10-23-aarch64-apple-darwin"/m'
  expect 1 "F7 host-suffixed channel -> FAIL" cfp
  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^(channel = ".*"\n)/$1channel = "nightly"\n/m'
  expect 1 "F7 two channel keys -> FAIL" cfp
  fp_reset; printf 'path = "/opt/toolchain"\n' >>"$FP/fuzz/rust-toolchain.toml"
  expect 1 "F7 path override -> FAIL" cfp
  fp_reset; fp_perl fuzz/rust-toolchain.toml 's/^channel = ".*"$/channel = "nightly-2027-01-31"/m'
  expect 0 "F7 another dated nightly passes" cfp

  fp_reset; printf '\n[[bin]]\nname = "extra"\npath = "fuzz_targets/extra.rs"\n' >>"$FP/fuzz/Cargo.toml"
  cp "$FP/fuzz/fuzz_targets/javafmt.rs" "$FP/fuzz/fuzz_targets/extra.rs"
  expect 1 "F8 new [[bin]] target missing from the workflow matrix and seeds -> FAIL" cfp
  fp_perl repo/.github/workflows/rust-fuzz.yml 's/^(          - javafmt\n)/$1          - extra\n/m'
  fp_perl scripts/fuzz-seeds.sh 's/^(targets=".*) javafmt"$/$1 javafmt extra"/m'
  expect 0 "F8 new target listed in Cargo.toml, matrix and seeds passes" cfp
  rm "$FP/fuzz/fuzz_targets/extra.rs"
  expect 1 "F8 [[bin]] without fuzz_targets/<name>.rs -> FAIL" cfp
  fp_reset; fp_perl repo/.github/workflows/rust-fuzz.yml 's/^          - stream_frame\n//m'
  expect 1 "F8 matrix missing a target -> FAIL" cfp
  fp_reset; fp_perl scripts/fuzz-seeds.sh 's/^(targets=".*) javafmt"$/$1"/m'
  expect 1 "F8 fuzz-seeds.sh missing a target -> FAIL" cfp
  fp_reset; rm "$FP/repo/.github/workflows/rust-fuzz.yml"
  expect 1 "F8 rust-fuzz.yml missing -> FAIL" cfp

  # F9 (REL-005): rust_inputs prefixes must trigger rust-fuzz.yml on PRs.
  fp_reset; fp_perl repo/.github/workflows/rust-fuzz.yml 's/^      - "src\/takserver-protobuf\/\*\*"\n//m'
  expect 1 "F9 rust_inputs prefix missing from pull_request.paths -> FAIL" cfp
  fp_reset; fp_perl repo/.github/workflows/rust-fuzz.yml 's/^      - "src\/takserver-protobuf\/\*\*"$/      # - "src\/takserver-protobuf\/**"/m'
  expect 1 "F9 prefix only in a comment -> FAIL" cfp
  fp_reset; fp_perl repo/.github/workflows/rust-fuzz.yml 's/^      - "src\/takserver-protobuf\/\*\*"\n//m'
  fp_perl repo/.github/workflows/rust-fuzz.yml 's/^(  workflow_dispatch:\n)/  push:\n    paths:\n      - "src\/takserver-protobuf\/**"\n$1/m'
  expect 1 "F9 prefix under push.paths only, not pull_request -> FAIL" cfp
  fp_reset; fp_perl repo/.github/workflows/rust-fuzz.yml "s/^      - \"src\\/takserver-protobuf\\/\\*\\*\"\$/      - 'src\\/takserver-protobuf\\/**' # quoted/m"
  expect 0 "F9 single-quoted entry with a trailing comment passes" cfp
  fp_reset; fp_perl scripts/ci-changes.sh "s/^rust_inputs='\\^src\\/takserver-protobuf\\/'/rust_inputs='^src\\/takserver-protobuf\\/|^utils\\/fixture\\/'/m"
  expect 1 "F9 new rust_inputs prefix not in pull_request.paths -> FAIL" cfp
  fp_perl repo/.github/workflows/rust-fuzz.yml 's/^(      - "src\/takserver-protobuf\/\*\*"\n)/$1      - "utils\/fixture\/**"\n/m'
  expect 0 "F9 new prefix listed in pull_request.paths passes" cfp
  fp_reset; fp_perl scripts/ci-changes.sh "s/^rust_inputs='\\^src\\/takserver-protobuf\\/'/rust_inputs='^src\\/(a|b)\\/'/m"
  expect 1 "F9 non-literal rust_inputs alternative -> FAIL (unverifiable)" cfp
  fp_reset; fp_perl scripts/ci-changes.sh "s/^rust_inputs=.*\\n//m"
  expect 1 "F9 ci-changes.sh without rust_inputs -> FAIL" cfp
  fp_reset; rm "$FP/scripts/ci-changes.sh"
  expect 1 "F9 ci-changes.sh missing -> FAIL" cfp
else
  failed=$((failed + 1)); echo "FAIL  rust/fuzz/ not found (check-fuzz-policy cases need the real files)"
fi

# ---------------------------------------------------------------------------
echo "== fuzz-seeds (QA-003: per-target, concurrent-safe)"
FS="$T/fuzzseeds"; mkdir -p "$FS/cases/xml" "$FS/cases/proto" "$FS/cases/numbers"
printf '<event/>' >"$FS/cases/xml/a.xml"
printf '0a00\n' >"$FS/cases/proto/a.pb.hex"
printf '1.0\n2.5\n' >"$FS/cases/numbers/n.txt"
fs() { env CORPUS_CASES="$FS/cases" bash "$S/fuzz-seeds.sh" "$FS/out" "$@"; }
fs_all() { # every target dir exists and is non-empty; no staging or lock left
  local t
  for t in cot_parse cot_to_proto proto_decode proto_to_cot proto_roundtrip stream_frame javafmt; do
    [[ -n "$(find "$FS/out/$t" -type f 2>/dev/null | head -n 1)" ]] || { echo "no seeds for $t"; return 1; }
  done
  [[ -z "$(find "$FS/out" -maxdepth 1 -name '.*' ! -name . ! -path "$FS/out")" ]] || { echo "leftover:"; ls -a "$FS/out"; return 1; }
}
expect 0 "fuzz-seeds without --only builds every target" fs
expect 0 "every target seeded, no staging/lock left" fs_all
touch "$FS/out/cot_parse/marker" "$FS/out/javafmt/marker"
expect 0 "fuzz-seeds --only javafmt" fs --only javafmt
expect 0 "--only javafmt replaced javafmt/" test ! -e "$FS/out/javafmt/marker"
expect 0 "--only javafmt left cot_parse/ untouched" test -e "$FS/out/cot_parse/marker"
expect 2 "--only with an empty target -> usage error" fs --only ""
expect 2 "--only with an unknown target -> usage error" fs --only nope
expect 2 "--only with two names in one argument -> usage error" fs --only "cot_parse javafmt"
expect 2 "--only without a target -> usage error" fs --only
fs_par() { # two targets at once, five times; both complete, cot_parse marker kept
  local i p1 p2 r=0
  for i in 1 2 3 4 5; do
    fs --only proto_decode >/dev/null & p1=$!
    fs --only stream_frame >/dev/null & p2=$!
    wait "$p1" || r=1; wait "$p2" || r=1
  done
  [[ "$r" -eq 0 ]] && fs_all && [[ -e "$FS/out/cot_parse/marker" ]]
}
expect 0 "concurrent --only runs for different targets do not interfere" fs_par
mkdir "$FS/out/.lock.javafmt"
fs_held() { env CORPUS_CASES="$FS/cases" bash "$S/fuzz-seeds.sh" "$FS/out" --only cot_parse; }
expect 0 "a lock held on another target does not block --only cot_parse" fs_held
rmdir "$FS/out/.lock.javafmt"

# ---------------------------------------------------------------------------
echo
echo "selftest: $passed passed, $failed failed"
[ "$failed" -eq 0 ]
