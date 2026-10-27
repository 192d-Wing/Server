#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Annotation and licence-header checks for first-party code:
#   (a) every compliance-mapping block is followed by at least one
#       `NIST 800-53:` or `ASD STIG:` line; NIST control IDs are in Rev 5 form
#       (AC-2, AC-2(4); no `AC-2 (4)`, no `AC2`); every parity annotation in
#       Rust code reads `PARITY: src/<path>#<method>`;
#   (b) with BASE_REF set (CI on PRs), the number of compliance-mapping blocks
#       must not drop compared with `git merge-base HEAD "$BASE_REF"`;
#       without it, the count is printed only;
#   (c) every *.rs under crates/services/web and every *.sh under
#       scripts/parity has `SPDX-License-Identifier: Apache-2.0` in its first
#       3 lines;
#   (d) every control ID on a `NIST 800-53:` line of a mapping block is
#       mapped in docs/compliance/NIST_800-53_MAPPING.md: a `### <ID> ...`
#       heading, or a table row whose first cell starts with `<ID> ` and no
#       cell of which disclaims (NIST-005). Disclaimer matching is case-
#       insensitive and ignores `*`, `_` and backticks (QA-007, NIST-016): a
#       cell disclaims when it starts with "Not Applicable" (any suffix, e.g.
#       "Not Applicable (component)"), "N/A", "None for the TOE", "None
#       claimed" or "Not implemented", or says "not claimed". A `###` block
#       does not count when its Status or Implementation cell disclaims
#       (NIST-014); a bare "None" (Planned, nothing built yet) still maps. A
#       sentence "<ID> is not claimed" disclaims the ID everywhere. An
#       annotation must not cite a control the mapping disclaims or omits.
#       scripts/mapping-pending.txt lists IDs whose block the compliance
#       scribe still has to write; they print PENDING instead of FAIL, and
#       fail too with ANNOTATIONS_STRICT=1 (`make check-annotations-strict`,
#       a non-required CI job). A pending ID that the mapping explicitly
#       disclaims always FAILS (NIST-013): the pending list only covers IDs
#       the mapping is silent about.
#       A pending ID that is now mapped prints a note to remove it.
# Scanned for (a)/(b): *.rs *.sh *.toml *.yml *.yaml Makefile under the
# workspace root, excluding target/, docs/, parity/.work/, parity/results/.
# Scanned for (a)/(d) only (not counted in (b)): the repository's
# .github/workflows/rust-*.yml|yaml and .github/dependabot.yml.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SA-15 (Development Process, Standards, and Tools), CM-10
#   (Software Usage Restrictions), SA-11 (Developer Testing and Evaluation)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT (default:
# parent of RUST_ROOT), BASE_REF (optional), MAPPING_FILE, MAPPING_PENDING
# (defaults under RUST_ROOT), ANNOTATIONS_STRICT (1 = pending IDs fail).
# Exit: 0 pass, 1 violation, 2 tooling error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
mapping=${MAPPING_FILE:-$root/docs/compliance/NIST_800-53_MAPPING.md}
pending_file=${MAPPING_PENDING:-$root/scripts/mapping-pending.txt}
strict=${ANNOTATIONS_STRICT:-0}

# Marker strings are assembled so this script does not match itself.
cm_marker="COMPLIANCE"
cm_marker="$cm_marker MAPPING:"
parity_marker="PARITY"
parity_marker="$parity_marker:"

list_files() { # <dir>
  find "$1" \( -path "$1/target" -o -path "$1/docs" -o -path "$1/parity/.work" \
      -o -path "$1/parity/results" -o -name .git -o -name node_modules \) -prune -o \
      -type f \( -name '*.rs' -o -name '*.sh' -o -name '*.toml' -o -name '*.yml' \
      -o -name '*.yaml' -o -name Makefile \) -print | LC_ALL=C sort
}

count_blocks() { # <dir>
  local n=0 c f
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    c=$(grep -cF "$cm_marker" "$f" || true)
    n=$((n + c))
  done <<EOF
$(list_files "$1")
EOF
  echo "$n"
}

status=0

# ---- (a) block structure and control-ID form -------------------------------
check_blocks_awk='
function is_comment(l) { return l ~ /^[[:space:]]*(\/\/|#|\*|--|;)/ }
function body(l) { sub(/^[[:space:]]*(\/\/[\/!]?|#+|\*|--|;+)/, "", l); return l }
function bad(msg) { printf "check-annotations: FAIL %s:%d: %s\n", FILENAME, FNR, msg; err = 1 }
function check_ids(text,   t, tok, n) {
  n = 0
  if (text ~ /[A-Z][A-Z]-[0-9]+ \([0-9]/) bad("control enhancement written with a space; use the Rev 5 form, e.g. CM-2(2)")
  t = " " text " "
  if (t ~ /[^A-Za-z0-9_][A-Z][A-Z][0-9]/) bad("control ID without hyphen; use the Rev 5 form, e.g. CM-2")
  while (match(t, /[^A-Za-z0-9][A-Z][A-Z]-[0-9]+(\([0-9]+\))?/)) {
    tok = substr(t, RSTART + 1, RLENGTH - 1); t = substr(t, RSTART + RLENGTH)
    n++
    if (tok !~ /^[A-Z][A-Z]-[0-9][0-9]?(\([0-9][0-9]?\))?$/) bad("control ID \"" tok "\" is not in Rev 5 form")
    else if (idfile != "") printf "%s\t%s:%d\n", tok, FILENAME, FNR > idfile
  }
  return n
}
FNR == 1 { if (open) close_block(); open = 0; innist = 0 }
function close_block() {
  if (!found) { printf "check-annotations: FAIL %s:%d: mapping block has no NIST 800-53: or ASD STIG: line\n", bfile, bline; err = 1 }
  open = 0
}
{
  line = $0
  if (index(line, cm) > 0) {
    if (open) close_block()
    open = 1; found = 0; innist = 0; bfile = FILENAME; bline = FNR
    next
  }
  if (!open) next
  if (!is_comment(line)) { close_block(); next }
  b = body(line)
  if (index(b, "NIST 800-53:") > 0) {
    found = 1; innist = 1
    if (check_ids(substr(b, index(b, "NIST 800-53:") + 12)) == 0) bad("NIST 800-53: line names no control ID")
    next
  }
  if (index(b, "ASD STIG:") > 0) { found = 1; innist = 0; next }
  # Continuation of a NIST line: indented by 2+ spaces and not a new bullet.
  if (innist && b ~ /^[[:space:]][[:space:]]+[^-[:space:]]/) { check_ids(b); next }
  innist = 0
}
END { if (open) close_block(); exit err }'

work=$(mktemp -d "${TMPDIR:-/tmp}/check-annotations.XXXXXX")
trap 'rm -rf "$work"' EXIT
ids="$work/ids.tsv"
: >"$ids"

files=$(list_files "$root")
for w in "$repo"/.github/workflows/rust-*.yml "$repo"/.github/workflows/rust-*.yaml "$repo"/.github/dependabot.yml; do
  [ -f "$w" ] && files="$files
$w"
done
files=$(printf '%s\n' "$files" | sed '/^$/d')
if [ -n "$files" ]; then
  # shellcheck disable=SC2086 # file paths from find contain no spaces in this tree
  if ! printf '%s\n' "$files" | xargs awk -v cm="$cm_marker" -v idfile="$ids" "$check_blocks_awk"; then
    status=1
  fi
fi

# ---- (d) annotated IDs must be mapped ---------------------------------------
if [ -s "$ids" ]; then
  if [ ! -f "$mapping" ]; then
    echo "check-annotations: FAIL (d) ${mapping#"$root"/} not found; annotated control IDs cannot be checked"
    status=1
  else
    # Mapped (M) / disclaimed (D) IDs. Every cell is normalised first
    # (QA-007, NIST-016): lower case, `*`, `_` and backticks removed, outer
    # blanks trimmed. A cell DISCLAIMS when it starts with "not applicable"
    # (any suffix, e.g. "Not Applicable (component)"), "n/a", "none for the
    # toe", "none claimed" or "not implemented", or says "not claimed"
    # anywhere. A bare "None" ("None", "None yet", "None in Phase 0": Planned,
    # nothing built yet) is not a disclaimer.
    #   `### <ID> ...` block: D when its Status cell disclaims (or contains
    #     "not applicable"/"n/a") or its Implementation cell disclaims
    #     (NIST-014); else M;
    #   table row whose first cell starts with `<ID> `: D when any cell
    #     disclaims, else M (NIST-005);
    #   sentence "<ID> is not claimed" (any case, emphasis allowed; not when
    #     the phrase itself is in double quotes): D (NIST-013).
    # An ID is mapped when it is M and never D.
    classes=$(awk '
      function close_blk() {
        if (blk != "") print (dis ? "D" : "M") "\t" blk
        blk = ""; dis = 0
      }
      function norm(s) {
        s = tolower(s); gsub(/[*_`]/, "", s)
        sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); return s
      }
      function disclaims(v) {
        v = norm(v)
        return v ~ /^(not applicable|n\/a|none for the toe|none claimed|not implemented)/ || v ~ /not claimed/
      }
      function cell(l) { sub(/^\|[^|]*\|[[:space:]]*/, "", l); sub(/[[:space:]]*\|[[:space:]]*$/, "", l); return l }
      {
        # Emphasis-free copy for headings and row IDs (`### **SI-7**`).
        plain = $0; gsub(/[*`]/, "", plain)
      }
      /^##/ { close_blk() }
      plain ~ /^###[[:space:]]+[A-Z][A-Z]-[0-9]/ {
        h = plain; sub(/^###[[:space:]]+/, "", h); sub(/[[:space:]].*$/, "", h)
        blk = h; dis = 0; next
      }
      blk != "" && norm($0) ~ /^\|[[:space:]]*status[[:space:]]*\|/ {
        v = cell($0); if (disclaims(v) || norm(v) ~ /not applicable|n\/a/) dis = 1
      }
      blk != "" && norm($0) ~ /^\|[[:space:]]*implementation[[:space:]]*\|/ {
        if (disclaims(cell($0))) dis = 1
      }
      plain ~ /^\|[[:space:]]*[A-Z][A-Z]-[0-9]+(\([0-9]+\))?[[:space:]]/ {
        r = plain; sub(/^\|[[:space:]]*/, "", r); sub(/[[:space:]].*$/, "", r)
        nc = split($0, cells, "|"); d = 0
        for (i = 2; i <= nc; i++) if (disclaims(cells[i])) d = 1
        print (d ? "D" : "M") "\t" r
      }
      {
        # A quoted mention ("CA-5 is not claimed" sentence removed, in the
        # revision history) is a reference to the text, not a disclaimer.
        t = " " $0; gsub(/[*_`]/, "", t); t = toupper(t)
        while (match(t, /.[A-Z][A-Z]-[0-9]+(\([0-9]+\))?[[:space:]]+IS[[:space:]]+NOT[[:space:]]+CLAIMED/)) {
          pre = substr(t, RSTART, 1)
          m = substr(t, RSTART + 1, RLENGTH - 1); t = substr(t, RSTART + RLENGTH)
          if (pre == "\"" || pre ~ /[A-Za-z0-9]/) continue
          sub(/[[:space:]].*$/, "", m); print "D\t" m
        }
      }
      END { close_blk() }' "$mapping")
    disclaimed=$(printf '%s\n' "$classes" | awk -F '\t' '$1 == "D" { print $2 }' | LC_ALL=C sort -u)
    mapped=$(printf '%s\n' "$classes" | awk -F '\t' '$1 == "M" { print $2 }' | LC_ALL=C sort -u |
      { if [ -n "$disclaimed" ]; then grep -vxF "$disclaimed" || true; else cat; fi; })
    pending=""
    [ -f "$pending_file" ] && pending=$(sed -e 's/#.*$//' "$pending_file" | awk 'NF { print $1 }' | LC_ALL=C sort -u)
    n_pending=0
    for id in $(cut -f1 "$ids" | LC_ALL=C sort -u); do
      if printf '%s\n' "$mapped" | grep -qxF "$id"; then
        continue
      fi
      where=$(awk -F '\t' -v id="$id" '$1 == id { print $2 }' "$ids" | sed "s|^$repo/||" | LC_ALL=C sort -u | tr '\n' ' ')
      is_dis=0
      printf '%s\n' "$disclaimed" | grep -qxF "$id" && is_dis=1
      if printf '%s\n' "$pending" | grep -qxF "$id" && [ "$strict" != 1 ] && [ "$is_dis" -eq 0 ]; then
        echo "check-annotations: PENDING (d) $id has no mapping block yet (scripts/mapping-pending.txt; compliance scribe): $where"
        n_pending=$((n_pending + 1))
      elif [ "$is_dis" -eq 1 ]; then
        echo "check-annotations: FAIL (d) $id is annotated but ${mapping#"$root"/} disclaims it (N/A or not-claimed row, a block whose Status/Implementation disclaims it, or an '$id is not claimed' sentence); mapping-pending.txt cannot excuse a disclaimed ID: $where"
        status=1
      else
        echo "check-annotations: FAIL (d) $id is annotated but has no '### $id' block or non-N/A row in ${mapping#"$root"/}: $where"
        status=1
      fi
    done
    for id in $pending; do
      if printf '%s\n' "$mapped" | grep -qxF "$id"; then
        echo "check-annotations: note (d) $id is now mapped; remove it from scripts/mapping-pending.txt"
      fi
    done
    [ "$n_pending" -eq 0 ] || echo "check-annotations: $n_pending pending ID(s); 'make check-annotations-strict' fails until they are mapped"
  fi
fi

# Parity annotations (Rust code only).
rs_dirs=""
for d in crates services web; do [ -d "$root/$d" ] && rs_dirs="$rs_dirs $root/$d"; done
if [ -n "$rs_dirs" ]; then
  # shellcheck disable=SC2086
  bad_parity=$(grep -rnF --include='*.rs' "$parity_marker" $rs_dirs |
    grep -vE "$parity_marker src/[^[:space:]#]+#[A-Za-z_][A-Za-z0-9_]*" || true)
  if [ -n "$bad_parity" ]; then
    echo "check-annotations: FAIL parity annotations must read '$parity_marker src/<path>#<method>':"
    printf '  %s\n' "$bad_parity"
    status=1
  fi
fi

# ---- (b) block count must not drop -----------------------------------------
current=$(count_blocks "$root")
if [ -n "${BASE_REF:-}" ]; then
  command -v git >/dev/null 2>&1 || { echo "check-annotations: git is required with BASE_REF" >&2; exit 2; }
  base=$(git -C "$root" merge-base HEAD "$BASE_REF") || {
    echo "check-annotations: FAIL cannot compute merge-base with $BASE_REF"; exit 1; }
  prefix=$(git -C "$root" rev-parse --show-prefix)
  top=$(git -C "$root" rev-parse --show-toplevel)
  tmp="$work/base"
  mkdir -p "$tmp"
  if git -C "$root" cat-file -e "$base:$prefix" 2>/dev/null; then
    # Run from the top level: in a subdirectory git archive filters paths again.
    git -C "$top" archive --format=tar "$base:$prefix" | tar -xf - -C "$tmp"
    before=$(count_blocks "$tmp")
  else
    before=0
  fi
  if [ "$current" -lt "$before" ]; then
    echo "check-annotations: FAIL compliance-mapping blocks dropped from $before (merge-base ${base:0:12}) to $current"
    status=1
  else
    echo "check-annotations: block count $current (base $before)"
  fi
else
  echo "check-annotations: block count $current (BASE_REF not set; no comparison)"
fi

# ---- (c) SPDX headers -------------------------------------------------------
spdx_files=$(
  for d in crates services web; do
    [ -d "$root/$d" ] && find "$root/$d" -path "$root/$d/*/target" -prune -o -type f -name '*.rs' -print
  done
  for d in scripts parity; do
    [ -d "$root/$d" ] && find "$root/$d" \( -path "$root/parity/.work" -o -path "$root/parity/results" \) -prune -o -type f -name '*.sh' -print
  done
  true
)
if [ -n "$spdx_files" ]; then
  while IFS= read -r f; do
    if ! head -n 3 "$f" | grep -qF 'SPDX-License-Identifier: Apache-2.0'; then
      echo "check-annotations: FAIL ${f#"$root"/}: no 'SPDX-License-Identifier: Apache-2.0' in the first 3 lines"
      status=1
    fi
  done <<EOF
$spdx_files
EOF
fi

[ "$status" -eq 0 ] && echo "check-annotations: ok"
exit "$status"
