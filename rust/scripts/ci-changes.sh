#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# CI change detector (option B). Reads repo-relative paths on stdin and prints
# `rust=true` or `rust=false`.
#
# Usage: ci-changes.sh [-z]
#   (default) newline-separated paths; a trailing CR is ignored
#   -z        NUL-separated paths, as from `git -c core.quotePath=false diff
#             --no-renames --name-only -z` (what rust-ci.yml uses)
#
# ALLOW-LIST of paths that can NOT affect the Rust build (SEC-001). Derived
# from what the upstream release merges touch (`git log --first-parent
# --merges main`; 5.7-RELEASE-14 back to 4.5-RELEASE-72: only src/**,
# utils/** and top-level *.md / *.txt files):
#   ^src/            the Java TAK Server (upstream)
#   ^utils/          upstream cluster/deployment utilities
#   ^[^/]+\.(md|txt)$  top-level Markdown/text (README.md, LICENSE.txt, ...)
# EXCEPTIONS inside the allow-list (SEC-008): paths under src/ or utils/ that
# the Rust build READS. They give rust=true. Current list (`rust_inputs`
# below, an awk ERE):
#   ^src/takserver-protobuf/   .proto schemas, read in place at build time
#                              by rust/crates/tak-proto/build.rs (SCHEMA_DIR
#                              = ../../../src/takserver-protobuf/src/main/proto;
#                              Phase 1a, POAM-2026-10-26-05). A schema change
#                              therefore runs the Rust jobs, where build.rs
#                              also checks each schema's SHA-256 against the
#                              committed rust/crates/tak-proto/schema.sha256
#                              and fails closed on any difference (SEC-005).
#                              Running the jobs is what makes that check
#                              fire: the upstream merge stays red until the
#                              manifest gets a reviewed update
#                              (tak-protocol-reviewer). The manifest is under
#                              rust/, so a change to it alone is rust=true.
# COUPLING: whenever a file under rust/ starts to read from src/ or utils/
# (a parent-relative path in build.rs, a Cargo `path =`, include_bytes!, a
# script) or a symlink under rust/ points there, the target directory MUST be
# added to `rust_inputs`. scripts/check-src-coupling.sh (make policy) fails
# when a reference or symlink resolves to a path this detector would judge
# rust=false.
# EVERY other path, including anything under rust/, .github/, a repo-root
# .cargo/ (Cargo merges parent configs into rust/ builds), a root
# rust-toolchain.toml, or a git-quoted path (a line starting with `"`),
# gives rust=true. New upstream top-level directories therefore run the Rust
# jobs (fail-safe) until they are added here deliberately (CM-3).
#
# FAIL-SAFE: empty input, unreadable input, the sentinel `__all__` or any
# internal error prints `rust=true`, so the heavy jobs run rather than being
# skipped. Linear time: one tr and one awk pass, no shell-variable copy of
# the input (QA-001).
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-3 (Configuration Change Control), SA-11 (Developer Testing
#   and Evaluation)
#
# Exit: always 0 (the verdict is the printed line).
set -euo pipefail

trap 'printf "rust=true\n"; exit 0' ERR

mode='nl'
case "${1:-}" in
  -z) mode=nul ;;
  '') ;;
  *) printf 'rust=true\n'; exit 0 ;;
esac

# In NUL mode a newline inside a path becomes \001, so the path is judged as
# one record (and, being unusual, fails the anchored allow-list).
split() {
  if [ "$mode" = nul ]; then
    tr '\n\000' '\001\n'
  else
    tr -d '\r'
  fi
}

# Rust build inputs under the allow-listed trees (see EXCEPTIONS above).
rust_inputs='^src/takserver-protobuf/'

# awk reads all of its input (no early pipe close under pipefail).
# shellcheck disable=SC2016 # awk program, not shell
judge='
$0 == "" { next }
{ n++ }
$0 == "__all__" { rel = 1; next }
$0 ~ ri { rel = 1; next }
$0 ~ /^src\// || $0 ~ /^utils\// || $0 ~ /^[^\/]+\.(md|txt)$/ { next }
{ rel = 1 }
END { if (n == 0 || rel) print "rust=true"; else print "rust=false" }'

if ! verdict=$(LC_ALL=C split | LC_ALL=C awk -v ri="$rust_inputs" "$judge"); then
  printf 'rust=true\n'
  exit 0
fi
case "$verdict" in
  rust=false) printf 'rust=false\n' ;;
  *) printf 'rust=true\n' ;;
esac
