#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Verdict for the single required check `rust-ci-gate` (option B).
# Inputs (environment):
#   NEEDS_JSON    `${{ toJSON(needs) }}` of the gate job
#   RUST_CHANGED  output of the `changes` job (`true`/`false`); anything other
#                 than `false` is treated as `true` (the stricter rule)
# Rules:
#   - FAIL unless `changes` and `workflow-lint` are present and `success`;
#   - RUST_CHANGED=false: PASS only if every other need is `skipped`;
#   - RUST_CHANGED=true:  PASS only if every need is `success`;
#   - FAIL for any `failure`, `cancelled`, unknown value or malformed JSON.
# Prints a table of job -> result -> verdict.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-3 (Configuration Change Control), SA-11 (Developer Testing
#   and Evaluation)
#
# Exit: 0 PASS, 1 FAIL.
set -euo pipefail

trap 'echo "ci-gate: FAIL (internal error)"; exit 1' ERR

command -v jq >/dev/null 2>&1 || { echo "ci-gate: FAIL jq is required"; exit 1; }

needs_json=${NEEDS_JSON:-}
changed=${RUST_CHANGED:-}
[ "$changed" = "false" ] || changed=true

if [ -z "$needs_json" ] || ! table=$(jq -er 'if type == "object" and length > 0
      then to_entries[] | "\(.key)\t\(.value.result // "missing" | tostring)"
      else error("needs is not a non-empty object") end' <<<"$needs_json" 2>/dev/null); then
  echo "ci-gate: FAIL NEEDS_JSON is missing or malformed"
  exit 1
fi

status=0
seen_changes=0
seen_lint=0
printf '%-24s %-10s %s\n' "JOB" "RESULT" "VERDICT"
while IFS="$(printf '\t')" read -r job result; do
  verdict=ok
  case "$job" in
    changes | workflow-lint)
      [ "$job" = changes ] && seen_changes=1
      [ "$job" = workflow-lint ] && seen_lint=1
      [ "$result" = success ] || verdict="FAIL (must be success)"
      ;;
    *)
      case "$result" in
        success)
          [ "$changed" = true ] || verdict="FAIL (rust=false: expected skipped)" ;;
        skipped)
          [ "$changed" = false ] || verdict="FAIL (rust=true: expected success)" ;;
        *)
          verdict="FAIL ($result)" ;;
      esac
      ;;
  esac
  [ "$verdict" = ok ] || status=1
  printf '%-24s %-10s %s\n' "$job" "$result" "$verdict"
done <<EOF
$table
EOF

[ "$seen_changes" -eq 1 ] || { echo "ci-gate: FAIL job 'changes' missing from needs"; status=1; }
[ "$seen_lint" -eq 1 ] || { echo "ci-gate: FAIL job 'workflow-lint' missing from needs"; status=1; }

if [ "$status" -eq 0 ]; then
  echo "ci-gate: PASS (rust=$changed)"
else
  echo "ci-gate: FAIL (rust=$changed)"
fi
exit "$status"
