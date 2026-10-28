#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Toolchain pin consistency:
#   - `channel` in rust-toolchain.toml starts with the workspace `rust-version`
#     (both bump together);
#   - .github/workflows/rust-*.yml never override the pinned toolchain: no
#     dtolnay/rust-toolchain, actions-rust-lang/setup-rust-toolchain,
#     `toolchain:` input, `cargo +<toolchain>`, or `rustup default/override`.
#     rust-fuzz.yml needs no exception: it runs cargo in rust/fuzz/, where
#     rustup selects the dated nightly pinned in fuzz/rust-toolchain.toml
#     (checked by scripts/check-fuzz-policy.sh F7).
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-2 (Baseline Configuration), CM-6 (Configuration Settings),
#   SA-15 (Development Process, Standards, and Tools)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT
# (default: parent of RUST_ROOT).
# Exit: 0 pass, 1 violation.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
status=0

tc="$root/rust-toolchain.toml"
ct="$root/Cargo.toml"
[ -f "$tc" ] || { echo "check-toolchain: FAIL missing rust-toolchain.toml"; exit 1; }
[ -f "$ct" ] || { echo "check-toolchain: FAIL missing Cargo.toml"; exit 1; }

channel=$(sed -n -E 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/p' "$tc" | head -n 1)
msrv=$(awk '
  /^[[:space:]]*\[/ { s = $0; gsub(/[[:space:]]/, "", s); insec = (s == "[workspace.package]"); next }
  insec && /^[[:space:]]*rust-version[[:space:]]*=/ { v = $0; sub(/^[^"]*"/, "", v); sub(/".*$/, "", v); print v; exit }' "$ct")

if [ -z "$channel" ]; then
  echo "check-toolchain: FAIL no channel in rust-toolchain.toml"; status=1
elif [ -z "$msrv" ]; then
  echo "check-toolchain: FAIL no rust-version in Cargo.toml [workspace.package]"; status=1
else
  case "$channel" in
    "$msrv" | "$msrv".*) echo "check-toolchain: ok channel $channel matches rust-version $msrv" ;;
    *) echo "check-toolchain: FAIL channel '$channel' does not start with rust-version '$msrv'"; status=1 ;;
  esac
fi

found_wf=0
for w in "$repo"/.github/workflows/rust-*.yml "$repo"/.github/workflows/rust-*.yaml; do
  [ -f "$w" ] || continue
  found_wf=1
  hits=$(grep -nE 'dtolnay/rust-toolchain|actions-rust-lang/setup-rust-toolchain|^[[:space:]]*toolchain[[:space:]]*:|cargo[[:space:]]+\+[A-Za-z0-9._-]+|rustup[[:space:]]+(default|override|toolchain[[:space:]]+install)' "$w" | grep -vE '^[0-9]+:[[:space:]]*#' || true)
  if [ -n "$hits" ]; then
    echo "check-toolchain: FAIL ${w#"$repo"/} overrides the pinned toolchain:"
    printf '  %s\n' "$hits"
    status=1
  fi
done
[ "$found_wf" -eq 0 ] && echo "check-toolchain: note: no .github/workflows/rust-*.yml yet (WU5)"

[ "$status" -eq 0 ] && echo "check-toolchain: ok"
exit "$status"
