#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# SEC-008: the Rust workspace must not read from the upstream trees (src/,
# utils/, top-level *.md/*.txt) without scripts/ci-changes.sh knowing it.
# Otherwise a change there gets rust=false and the Rust code built from it is
# never built or tested.
#
# Fails when, under the workspace root:
#   (r) a build-relevant file contains a parent-relative path (one or more
#       `../` followed by `src` or `utils`), and the referenced repository
#       path is judged rust=false by ci-changes.sh;
#   (s) a symlink resolves outside the workspace root to a repository path
#       that ci-changes.sh judges rust=false.
# The probe for a reference `<dots>/src/a/b` is the path `src/a/b/__probe__`,
# so the reference must be covered by a `rust_inputs` prefix in
# ci-changes.sh (or not be allow-listed at all). `.` and `..` segments after
# the leading `../` run are resolved first (QA-008), so
# `src/takserver-protobuf/../takserver-core/x` is judged as
# `src/takserver-core/x`; a reference that climbs above the repository root
# fails.
#
# Scanned: every file under the workspace root except target/, docs/,
# parity/ (the Java reference harness has its own workflow and trigger),
# node_modules/, .git, *.md, NOTICE*, LICENSE*. Heuristic: a path built at
# run time from pieces (`"..".join(...)`, env!("CARGO_MANIFEST_DIR") plus
# string joins) is not seen; review and the tak-proto build.rs header are the
# backstop.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-3 (Configuration Change Control), SA-11 (Developer
#   Testing and Evaluation)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT
# (default: parent of RUST_ROOT).
# Exit: 0 pass, 1 violation, 2 tooling error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
root=$(cd "$root" && pwd -P)
repo=$(cd "$repo" && pwd -P)
detector="$here/ci-changes.sh"
[ -f "$detector" ] || { echo "check-src-coupling: $detector not found" >&2; exit 2; }

status=0
n_refs=0
n_links=0

# judge <repo-relative path> -> prints rust=true|rust=false
judge() { printf '%s\n' "$1" | bash "$detector"; }

# Assembled so this file does not match its own pattern.
dots='\.\./'
ref_re="(${dots})+(src|utils)(/[A-Za-z0-9_.+-]+)*"

scan_files() {
  find "$root" \( -path "$root/target" -o -path "$root/docs" -o -path "$root/parity" \
    -o -name node_modules -o -name .git \) -prune -o \
    -type f ! -name '*.md' ! -name 'NOTICE*' ! -name 'LICENSE*' -print
}

# (r) references
while IFS= read -r f; do
  [ -n "$f" ] || continue
  grep -Iq . "$f" 2>/dev/null || continue # skip binary and empty files
  while IFS= read -r m; do
    [ -n "$m" ] || continue
    n_refs=$((n_refs + 1))
    p=$(printf '%s\n' "$m" | sed -E "s#^(${dots})+##")
    # QA-008: resolve `.`/`..` segments lexically before judging.
    p=$(printf '%s\n' "$p" | awk -F/ '{
      n = 0
      for (i = 1; i <= NF; i++) {
        if ($i == "" || $i == ".") continue
        if ($i == "..") { if (n == 0) { print "__escapes__"; exit } n--; continue }
        seg[++n] = $i
      }
      out = ""; for (i = 1; i <= n; i++) out = out (i > 1 ? "/" : "") seg[i]
      print out
    }')
    if [ "$p" = __escapes__ ] || [ -z "$p" ]; then
      echo "check-src-coupling: FAIL (r) ${f#"$root"/} reads '$m', whose '..' segments climb above the repository root"
      status=1
      continue
    fi
    v=$(judge "$p/__probe__")
    if [ "$v" != rust=true ]; then
      echo "check-src-coupling: FAIL (r) ${f#"$root"/} reads '$m' but ci-changes.sh judges $p/ $v; add ^$p/ (or its directory) to rust_inputs in scripts/ci-changes.sh"
      status=1
    fi
  done <<EOF
$(grep -oE "$ref_re" "$f" 2>/dev/null | LC_ALL=C sort -u || true)
EOF
done <<EOF
$(scan_files)
EOF

# (s) symlinks out of the workspace root
while IFS= read -r l; do
  [ -n "$l" ] || continue
  t=$(cd "$(dirname "$l")" && python3 -I -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$(basename "$l")")
  case "$t" in
    "$root" | "$root"/*) continue ;;
  esac
  n_links=$((n_links + 1))
  case "$t" in
    "$repo"/*) rel=${t#"$repo"/} ;;
    *)
      echo "check-src-coupling: FAIL (s) ${l#"$root"/} points outside the repository: $t"
      status=1
      continue
      ;;
  esac
  probe=$rel
  [ -d "$t" ] && probe="$rel/__probe__"
  v=$(judge "$probe")
  if [ "$v" != rust=true ]; then
    echo "check-src-coupling: FAIL (s) ${l#"$root"/} -> $rel, which ci-changes.sh judges $v; add it to rust_inputs in scripts/ci-changes.sh"
    status=1
  fi
done <<EOF
$(find "$root" \( -path "$root/target" -o -path "$root/parity/.work" -o -path "$root/parity/results" \
  -o -name node_modules -o -name .git \) -prune -o -type l -print)
EOF

[ "$status" -eq 0 ] && echo "check-src-coupling: ok ($n_refs parent-relative src/utils reference(s), $n_links symlink(s) out of rust/, all covered)"
exit "$status"
