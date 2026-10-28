#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Every POA&M marker comment names its POA&M entry (NIST-010). A marker is
# `POAM:` directly after a comment introducer (`//`, `///`, `//!`, `#`, `*`,
# `/*`, `--`) at the start of a line or after whitespace; the first token
# after it must be an ID `POAM-YYYY-MM-DD-NN`, e.g.
#   // <marker> POAM-YYYY-MM-DD-NN - per-process memory budget (Phase 3)
# A bare marker (`// <marker> the bounds are ...`) fails. A commented-out
# example of the format (`#   # <marker> POAM-YYYY-MM-DD-NN - ...`, the
# literal placeholder inside a comment line, as in .cargo/audit.toml)
# passes. That the ID exists in docs/compliance/POAM.md is check-poam-ids.sh's
# job.
# Scanned: *.rs *.sh *.toml *.yml *.yaml Makefile under the workspace root
# (same exclusions as check-annotations.sh: target/, docs/, parity/.work/,
# parity/results/, fuzz build/corpus dirs) and the repository's
# .github/workflows/rust-*.yml|yaml and .github/dependabot.yml.
#
# Sequencing: part of POLICY_CHECKS in rust/Makefile (so `make policy`) since
# Phase 1a WU10 gave every marker an ID; it also runs in
# `make check-annotations-strict`.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CA-5 (Plan of Action and Milestones), SA-15 (Development
#   Process, Standards, and Tools)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT
# (default: parent of RUST_ROOT).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}

# Assembled so this script does not match itself.
mk="POAM"
mk="$mk:"
marker_re="(^|[[:space:]])(//[/!]?|#|/?\*+|--)[[:space:]]*$mk"
good_re="(^|[[:space:]])(//[/!]?|#|/?\*+|--)[[:space:]]*${mk}[[:space:]]*POAM-[0-9]{4}-[0-9]{2}-[0-9]{2}-[0-9]{2}([^0-9]|\$)"
# A commented-out example of the format (a marker inside a comment line,
# citing the literal placeholder) documents the rule and is not a marker.
example_re="^[[:space:]]*(#|//)[[:space:]]+(#|//)[[:space:]]*${mk}[[:space:]]*POAM-YYYY-MM-DD-NN([^0-9A-Za-z]|\$)"

list_files() { # <dir>
  find "$1" \( -path "$1/target" -o -path "$1/docs" -o -path "$1/parity/.work" \
      -o -path "$1/parity/results" -o -path "$1/fuzz/target" -o -path "$1/fuzz/corpus" \
      -o -path "$1/fuzz/seeds" -o -path "$1/fuzz/artifacts" -o -path "$1/fuzz/coverage" \
      -o -name .git -o -name node_modules \) -prune -o \
      -type f \( -name '*.rs' -o -name '*.sh' -o -name '*.toml' -o -name '*.yml' \
      -o -name '*.yaml' -o -name Makefile \) -print | LC_ALL=C sort
}

files=$(
  list_files "$root"
  for w in "$repo"/.github/workflows/rust-*.yml "$repo"/.github/workflows/rust-*.yaml \
      "$repo"/.github/dependabot.yml; do
    if [[ -f "$w" ]]; then printf '%s\n' "$w"; fi
  done
)

n=0
bad=0
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  while IFS= read -r hit; do
    [[ -n "$hit" ]] || continue
    n=$((n + 1))
    line=${hit#*:}
    if ! printf '%s\n' "$line" | grep -qE "$good_re|$example_re"; then
      rel=${f#"$repo"/}
      echo "check-poam-markers: FAIL $rel:${hit%%:*}: marker without a POAM-YYYY-MM-DD-NN ID: $(printf '%s' "$line" | sed -E 's/^[[:space:]]+//' | cut -c1-100)"
      bad=$((bad + 1))
    fi
  done < <(grep -nE "$marker_re" "$f" || true)
done <<EOF
$files
EOF

if [[ "$bad" -ne 0 ]]; then
  echo "check-poam-markers: $bad of $n marker(s) lack an ID; add an entry to docs/compliance/POAM.md and cite its ID right after the marker"
  exit 1
fi
echo "check-poam-markers: ok ($n marker(s), each with a POA&M ID)"
