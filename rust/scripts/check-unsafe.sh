#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Every `unsafe` in first-party Rust code must have a `// SAFETY:` comment
# within the 3 preceding lines. Comment-only lines are not code and are skipped.
# The workspace forbids unsafe_code by default; this backs up the lint for a
# crate that opts out with its own approved `[lints]` table.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SA-11 (Developer Testing and Evaluation), SA-15 (Development
#   Process, Standards, and Tools)
#
# Environment: RUST_ROOT (default: parent of this script).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}

dirs=""
for d in crates services web; do
  [ -d "$root/$d" ] && dirs="$dirs $root/$d"
done
if [ -z "$dirs" ]; then
  echo "check-unsafe: no crates/services/web directories; nothing to check"
  exit 0
fi

# shellcheck disable=SC2086 # $dirs is a space-separated list of paths without spaces
hits=$(grep -rnE --include='*.rs' '(^|[^A-Za-z0-9_])unsafe([^A-Za-z0-9_]|$)' $dirs || true)

status=0
count=0
if [ -n "$hits" ]; then
  while IFS= read -r hit; do
    file=${hit%%:*}
    rest=${hit#*:}
    line=${rest%%:*}
    text=${rest#*:}
    # Skip comment lines (`//`, `///`, `//!`, block-comment continuation).
    if printf '%s\n' "$text" | grep -qE '^[[:space:]]*(//|\*|/\*)'; then
      continue
    fi
    count=$((count + 1))
    from=$((line - 3))
    [ "$from" -lt 1 ] && from=1
    to=$((line - 1))
    ok=0
    if [ "$to" -ge 1 ] && sed -n "${from},${to}p" "$file" | grep -qE '//[[:space:]]*SAFETY:'; then
      ok=1
    fi
    if [ "$ok" -eq 0 ]; then
      echo "check-unsafe: FAIL ${file#"$root"/}:$line: \`unsafe\` without \`// SAFETY:\` in the 3 preceding lines"
      echo "    $text"
      status=1
    fi
  done <<EOF
$hits
EOF
fi

if [ "$status" -eq 0 ]; then
  echo "check-unsafe: ok ($count unsafe use(s), all justified)"
fi
exit "$status"
