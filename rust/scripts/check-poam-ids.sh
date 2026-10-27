#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Every POA&M ID referenced in rust/ (and in .github/workflows/rust-*.yml and
# .github/dependabot.yml, NIST-020) must exist as an entry in
# docs/compliance/POAM.md. An entry is a table row whose first cell is the ID
# (`| POAM-... |`) or a heading containing it.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CA-5 (Plan of Action and Milestones)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT
# (default: parent of RUST_ROOT).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
poam="$root/docs/compliance/POAM.md"
id_re='POAM-[0-9]{4}-[0-9]{2}-[0-9]{2}-[0-9]{2}'

if [ ! -f "$poam" ]; then
  echo "check-poam-ids: FAIL ${poam#"$root"/} does not exist"
  exit 1
fi

entries=$(grep -E "^[[:space:]]*\|[[:space:]]*\**$id_re\**[[:space:]]*\||^#+[[:space:]].*$id_re" "$poam" |
  grep -oE "$id_re" | LC_ALL=C sort -u || true)

workflows=""
for w in "$repo"/.github/workflows/rust-*.yml "$repo"/.github/workflows/rust-*.yaml "$repo"/.github/dependabot.yml; do
  [ -f "$w" ] && workflows="$workflows $w"
done

# shellcheck disable=SC2086 # workflow paths contain no spaces
refs=$( { grep -rhoE "$id_re" "$root" --exclude-dir=target --exclude-dir=.work \
            --exclude-dir=results --exclude-dir=.git --exclude-dir=node_modules || true
          [ -n "$workflows" ] && grep -hoE "$id_re" $workflows || true; } | LC_ALL=C sort -u)

missing=$(LC_ALL=C comm -23 <(printf '%s\n' "$refs" | sed '/^$/d') <(printf '%s\n' "$entries" | sed '/^$/d'))
if [ -n "$missing" ]; then
  echo "check-poam-ids: FAIL referenced but not an entry in ${poam#"$root"/}:"
  for id in $missing; do
    where=$(grep -rlE "$id" "$root" --exclude-dir=target --exclude-dir=.work --exclude-dir=results 2>/dev/null | sed "s|^$root/||" | tr '\n' ' ')
    echo "  $id  ($where)"
  done
  exit 1
fi
n_refs=$(printf '%s\n' "$refs" | sed '/^$/d' | wc -l | tr -d ' ')
n_entries=$(printf '%s\n' "$entries" | sed '/^$/d' | wc -l | tr -d ' ')
echo "check-poam-ids: ok ($n_refs referenced ID(s), $n_entries entr(ies) in POAM.md)"
