#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Static policy for .github/workflows/*.yml|yaml. The rules run on the
# PARSED YAML (scripts/check-workflows.rb, Ruby stdlib psych; QA-006,
# SEC-004), so quoted keys, a space before the colon, escapes, block scalars,
# plain scalars on the next line and flow style are all seen as GitHub sees
# them. CI also runs actionlint and zizmor.
#
# Rules (every rust-*.yml|yaml):
#   P0-P3 the file is valid UTF-8 YAML, one document, a mapping, scalar keys
#      only, no duplicate keys (parsers disagree on which one wins)
#   W1 every `uses:` (step or job level) is `./...` or
#      `<owner>/<repo>[/path]@<40 hex>` with a `# v<version>` comment on the
#      same line
#   W2 no `pull_request_target` or `workflow_run` trigger (parsed `on:`, and
#      any mention outside a comment)
#   W3 top-level `permissions:` present and exactly `contents: read`
#   W3b job-level `permissions:` absent or exactly `contents: read`
#   W4 every actions/checkout step sets `persist-credentials: false`
#   W5 every `${{ ... }}` in a `run:` value is exactly `matrix.<key>`,
#      `runner.<x>`, `github.workspace`, `github.sha`, `github.run_id`,
#      `github.run_attempt` or `github.repository` (allow-list, SEC-011);
#      everything else fails, including `env.*`, index syntax, function
#      calls and case variants (template-injection guard; pass values
#      through `env:` and use "$VAR")
#   W5b no `${{` anywhere under a job's `strategy:` (matrix values are
#      literals; SEC-010 (a))
#   W6 no YAML anchors, aliases, merge keys (`<<`) or explicit tags
#   W7 `continue-on-error` (any spelling, any depth) only as the job-level key
#      of job `sbom` in rust-ci.yml (POAM-2026-10-26-01)
# Every workflow file (*.yml|*.yaml, any case, dotfiles included):
#   W8 top-level, job-level, step-level and permissions keys are spelled as
#      GitHub spells them (`Name:`, `Uses:`, `Permissions:` fail; SEC-012)
#   W9 top-level `permissions:` present; no `write-all`, `checks: write` or
#      `statuses: write` at the top or job level (a token that writes check
#      runs or statuses can spoof rust-ci-gate; SEC-010 (b))
# Option B invariants:
#   B0 .github/workflows/rust-ci.yml exists (missing = FAIL, also when no
#      rust-*.yml exists at all)
#   B1 rust-ci.yml `on.pull_request` exists and has no `paths`/`paths-ignore`
#   B2 in rust-ci.yml only job `rust-ci-gate` has an ID or `name:` containing
#      `rust-ci` (case-insensitive), the gate's name is absent or exactly
#      `rust-ci-gate`, and job names use no `${{ }}` other than
#      `${{ matrix.<key> }}` with no matrix value containing `rust-ci` or
#      `${{`. In
#      every OTHER workflow file: no job ID or decoded job `name:` contains
#      `rust-ci`, no job `name:` contains `${{`, no merge keys under `jobs:`
#   B3 `rust-ci-gate` has `if: always()`, no `strategy:`/`uses:`, and its
#      `needs:` equals all job IDs in rust-ci.yml minus `rust-ci-gate` and
#      the non-required jobs `sbom` (POAM-2026-10-26-01) and
#      `annotations-strict` (NIST-013: visible, never required)
# Ruby is required. If it is missing, fails or does not print its sentinel
# line, the check FAILS (fail closed); it never skips.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-3 (Configuration Change Control), SA-10 (Developer
#   Configuration Management), SA-15 (Development Process, Standards, and
#   Tools), SR-3 (Supply Chain Controls and Processes), SR-11 (Component
#   Authenticity)
#
# Environment: REPO_ROOT (default: parent of the workspace root).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
wfdir="$repo/.github/workflows"

rust_wfs=""
for w in "$wfdir"/rust-*.yml "$wfdir"/rust-*.yaml; do
  [ -f "$w" ] && rust_wfs="$rust_wfs $w"
done
# B0: rust-ci.yml carries the single required check; its absence is a
# failure, never a vacuous pass (REL-001).
if [ -z "$rust_wfs" ]; then
  echo "check-workflows: FAIL B0 .github/workflows/rust-ci.yml is missing (no rust-*.yml at all)"
  exit 1
fi

# Every workflow file GitHub may load (B2/W8/W9 cover files that are not
# rust-*): *.yml and *.yaml in any letter case, dot-prefixed names included
# (SEC-010 (c)); a glob would skip dotfiles.
all_wfs=()
while IFS= read -r -d '' w; do
  [ -f "$w" ] && all_wfs+=("$w")
done < <(find "$wfdir" -mindepth 1 -maxdepth 1 \( -type f -o -type l \) \
  \( -iname '*.yml' -o -iname '*.yaml' \) -print0 | LC_ALL=C sort -z)
for w in $rust_wfs; do
  case " ${all_wfs[*]} " in
    *" $w "*) ;;
    *) echo "check-workflows: FAIL workflow discovery missed $w (find failed?)"; exit 1 ;;
  esac
done

status=0
fail() { echo "check-workflows: FAIL $*"; status=1; }

# B0: rust-ci.yml present.
[ -f "$wfdir/rust-ci.yml" ] || fail "B0 .github/workflows/rust-ci.yml is missing (other rust-*.yml exist)"

ruby_bin=${CHECK_WORKFLOWS_RUBY:-ruby}
if ! command -v "$ruby_bin" >/dev/null 2>&1; then
  fail "ruby not found ('$ruby_bin'); the parsed-YAML rules cannot run (fail closed; install ruby)"
  exit 1
fi
# No RUBYOPT/RUBYLIB from the environment, no gems: only the stdlib psych.
set +e
out=$(env -u RUBYOPT -u RUBYLIB -u RUBYLIB_PREFIX "$ruby_bin" --disable=gems \
  "$here/check-workflows.rb" "$repo" "${all_wfs[@]}" 2>&1)
rc=$?
set -e
last=$(printf '%s\n' "$out" | tail -n 1)
printf '%s\n' "$out" | sed '$d' | sed '/^$/d; s|^FAIL |check-workflows: FAIL |; s|^ERROR |check-workflows: FAIL ruby |'
if [ "$last" != "parsed ${#all_wfs[@]} file(s)" ]; then
  fail "parsed-YAML pass did not complete (exit $rc, last line: $last)"
elif [ "$rc" -ne 0 ]; then
  status=1
  [ "$rc" -eq 1 ] || fail "parsed-YAML pass exited $rc"
fi

n_wf=0
for w in $rust_wfs; do n_wf=$((n_wf + 1)); done
[ "$status" -eq 0 ] && echo "check-workflows: ok ($n_wf rust workflow file(s), ${#all_wfs[@]} parsed)"
exit "$status"
