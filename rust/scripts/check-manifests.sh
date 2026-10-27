#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Every workspace member inherits the lint baseline and the licence, and is
# never published:
#   - `[lints]` with `workspace = true`
#   - `license.workspace = true` (or `license = { workspace = true }`)
#   - `publish` resolves to false
#   - resolved licence equals `Apache-2.0` (D11)
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-2 (Baseline Configuration), CM-6 (Configuration Settings),
#   CM-10 (Software Usage Restrictions), SA-15 (Development Process, Standards,
#   and Tools)
#
# Environment: RUST_ROOT (default: parent of this script).
# Exit: 0 pass, 1 violation, 2 tooling error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}

command -v jq >/dev/null 2>&1 || { echo "check-manifests: jq is required" >&2; exit 2; }

meta=$(cd "$root" && cargo metadata --no-deps --format-version 1) || {
  echo "check-manifests: cargo metadata failed" >&2; exit 2; }

members=$(printf '%s' "$meta" | jq -r '
  .workspace_members as $m
  | .packages[] | select(.id as $i | $m | index($i))
  | [.name, .manifest_path, (.license // "<none>"), (if .publish == [] then "false" else "true" end)]
  | @tsv')

if [ -z "$members" ]; then
  echo "check-manifests: FAIL no workspace members found"
  exit 1
fi

# section_has <manifest> <section> <key-regex>: key appears inside [section].
section_has() {
  awk -v sec="$2" -v key="$3" '
    /^[[:space:]]*\[/ {
      s = $0; sub(/^[[:space:]]*\[/, "", s); sub(/\].*$/, "", s); gsub(/[[:space:]]/, "", s)
      insec = (s == sec); next
    }
    insec && $0 ~ key { found = 1 }
    END { exit !found }' "$1"
}

status=0
n=0
while IFS="$(printf '\t')" read -r name manifest license publish; do
  n=$((n + 1))
  fail=""
  if ! section_has "$manifest" lints '^[[:space:]]*workspace[[:space:]]*=[[:space:]]*true'; then
    fail="$fail; missing [lints] workspace = true"
  fi
  if ! section_has "$manifest" package '^[[:space:]]*license(\.workspace[[:space:]]*=[[:space:]]*true|[[:space:]]*=[[:space:]]*\{[[:space:]]*workspace[[:space:]]*=[[:space:]]*true)'; then
    fail="$fail; licence not inherited (need license.workspace = true)"
  fi
  if [ "$publish" != "false" ]; then
    fail="$fail; publish is not false"
  fi
  if [ "$license" != "Apache-2.0" ]; then
    fail="$fail; resolved licence is '$license', expected 'Apache-2.0'"
  fi
  if [ -n "$fail" ]; then
    echo "check-manifests: FAIL $name (${manifest#"$root"/})${fail}"
    status=1
  else
    echo "check-manifests: ok   $name"
  fi
done <<EOF
$members
EOF

[ "$status" -eq 0 ] && echo "check-manifests: ok ($n member(s))"
exit "$status"
