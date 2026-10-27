#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# The advisory exception lists must agree:
#   - set of RUSTSEC IDs in deny.toml [advisories].ignore
#     == set in .cargo/audit.toml [advisories].ignore
#   - every audit.toml entry is preceded (since the previous entry) by a
#     `# POAM: POAM-YYYY-MM-DD-NN` line.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: RA-5 (Vulnerability Monitoring and Scanning), SI-2 (Flaw
#   Remediation), CA-5 (Plan of Action and Milestones)
#
# Environment: RUST_ROOT (default: parent of this script).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
deny="$root/deny.toml"
audit="$root/.cargo/audit.toml"

for f in "$deny" "$audit"; do
  [ -f "$f" ] || { echo "check-advisory-sync: FAIL missing ${f#"$root"/}"; exit 1; }
done

# Lines of the [advisories] table only, comments stripped.
advisories_section() {
  awk '
    /^[[:space:]]*\[/ { s = $0; gsub(/[[:space:]]/, "", s); insec = (s == "[advisories]"); next }
    insec' "$1"
}

id_re='RUSTSEC-[0-9]{4}-[0-9]{4}'

deny_ids=$(advisories_section "$deny" | sed -e 's/#.*$//' |
  grep -oE "(id[[:space:]]*=[[:space:]]*\"$id_re\"|^[[:space:]]*\"$id_re\")" |
  grep -oE "$id_re" | LC_ALL=C sort -u || true)
audit_ids=$(advisories_section "$audit" | sed -e 's/#.*$//' |
  grep -oE "^[[:space:]]*\"$id_re\"" | grep -oE "$id_re" | LC_ALL=C sort -u || true)

status=0
only_deny=$(LC_ALL=C comm -23 <(printf '%s\n' "$deny_ids" | sed '/^$/d') <(printf '%s\n' "$audit_ids" | sed '/^$/d'))
only_audit=$(LC_ALL=C comm -13 <(printf '%s\n' "$deny_ids" | sed '/^$/d') <(printf '%s\n' "$audit_ids" | sed '/^$/d'))
if [ -n "$only_deny" ]; then
  echo "check-advisory-sync: FAIL ignored in deny.toml only: $(echo "$only_deny" | tr '\n' ' ')"
  status=1
fi
if [ -n "$only_audit" ]; then
  echo "check-advisory-sync: FAIL ignored in .cargo/audit.toml only: $(echo "$only_audit" | tr '\n' ' ')"
  status=1
fi

missing_poam=$(advisories_section "$audit" | awk -v re="$id_re" '
  /^[[:space:]]*#[[:space:]]*POAM:[[:space:]]*POAM-[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]-[0-9][0-9]/ { poam = 1; next }
  /^[[:space:]]*#/ { next }
  {
    line = $0; sub(/#.*$/, "", line)
    if (match(line, /^[[:space:]]*"RUSTSEC-[0-9]+-[0-9]+"/)) {
      id = substr(line, RSTART, RLENGTH); gsub(/[[:space:]"]/, "", id)
      if (!poam) print id
      poam = 0
    }
  }')
if [ -n "$missing_poam" ]; then
  echo "check-advisory-sync: FAIL audit.toml entries without a '# POAM: POAM-...' line: $(echo "$missing_poam" | tr '\n' ' ')"
  status=1
fi

if [ "$status" -eq 0 ]; then
  n=$(printf '%s\n' "$audit_ids" | sed '/^$/d' | wc -l | tr -d ' ')
  echo "check-advisory-sync: ok ($n ignored advisory ID(s), lists identical, each with a POA&M)"
fi
exit "$status"
