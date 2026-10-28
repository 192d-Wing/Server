#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Keeps the coverage-guided fuzz workspace rust/fuzz/ (cargo-fuzz, libFuzzer;
# development-only native-code exception, Phase 1 plan Q2) isolated from the
# product graph and in step with the product supply-chain policy. Static: no
# network, no nightly toolchain (runs in `make policy` on every PR).
#
# Rules:
#   F1 fuzz/ exists with Cargo.toml, Cargo.lock, deny.toml,
#      native-allowlist.txt and rust-toolchain.toml
#   F2 fuzz/Cargo.toml is its own workspace root (a `[workspace]` table) and
#      depends on libfuzzer-sys
#   F3 rust/Cargo.toml `[workspace] exclude` lists "fuzz", and no member of
#      the main workspace (cargo metadata --no-deps) has its manifest under
#      fuzz/
#   F4 no main-workspace member declares a libfuzzer-sys dependency (any
#      kind, any target) and rust/Cargo.lock has no libfuzzer-sys package
#   F5 fuzz/deny.toml equals rust/deny.toml line for line, except inside
#      marked blocks
#        # >>> FUZZ-DELTA replaces=<n>[: why]
#        ...
#        # <<< FUZZ-DELTA
#      where <n> lines of rust/deny.toml are replaced (0 = insertion). A
#      block may replace only comment/blank lines or the `cc` ban line, and
#      may contain only comments plus the approved exception, each exactly
#      once and in its own table/array:
#        [licenses] exceptions:      { allow = ["NCSA"], crate = "libfuzzer-sys" },
#        [bans] deny:                the replaced `cc` line with "libfuzzer-sys"
#                                    appended to its wrappers (reason free)
#        [bans.build] allow-build-scripts: "libfuzzer-sys",
#      All three must be present (a missing block fails).
#   F6 fuzz/native-allowlist.txt has exactly one entry, for libfuzzer-sys
#   F7 fuzz/rust-toolchain.toml pins a dated nightly: one `channel =
#      "nightly-YYYY-MM-DD"` under [toolchain], no `path` override
#   F8 the fuzz targets agree everywhere: the `[[bin]]` names in
#      fuzz/Cargo.toml (each with fuzz_targets/<name>.rs), the `target:`
#      matrix list in .github/workflows/rust-fuzz.yml and `targets=` in
#      scripts/fuzz-seeds.sh, so a new target cannot skip CI or its seeds
#   F9 every prefix in `rust_inputs` of scripts/ci-changes.sh (paths outside
#      rust/ that the Rust build reads, e.g. the .proto schemas) appears as
#      "<prefix>**" under `on: pull_request: paths:` of rust-fuzz.yml, so an
#      upstream change to a fuzzed input re-runs the fuzzers (NIST-009,
#      REL-005). rust_inputs must stay a `|`-list of `^dir/` literals; any
#      other ERE fails here as unverifiable.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: CM-3 (Configuration Change Control), CM-7 (Least
#   Functionality), SA-15 (Development Process, Standards, and Tools), SR-3
#   (Supply Chain Controls and Processes)
#
# Environment: RUST_ROOT (default: parent of this script), REPO_ROOT
# (default: parent of RUST_ROOT).
# Exit: 0 pass, 1 violation, 2 tooling error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
repo=${REPO_ROOT:-$(cd "$root/.." && pwd)}
fz="$root/fuzz"
status=0
fail() { echo "check-fuzz-policy: FAIL $*"; status=1; }

command -v jq >/dev/null 2>&1 || { echo "check-fuzz-policy: jq is required" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { echo "check-fuzz-policy: cargo is required" >&2; exit 2; }

# ---- F1 ---------------------------------------------------------------------
if [[ ! -d "$fz" ]]; then
  fail "F1 fuzz/ is missing (rust/Cargo.toml excludes it and the Makefile runs it)"
  exit 1
fi
for f in Cargo.toml Cargo.lock deny.toml native-allowlist.txt rust-toolchain.toml; do
  [[ -f "$fz/$f" ]] || fail "F1 fuzz/$f is missing"
done
for f in "$root/Cargo.toml" "$root/Cargo.lock" "$root/deny.toml"; do
  [[ -f "$f" ]] || fail "F1 ${f#"$root"/} is missing"
done
[[ "$status" -eq 0 ]] || exit 1

# ---- F2 ---------------------------------------------------------------------
grep -qE '^[[:space:]]*\[workspace\][[:space:]]*(#.*)?$' "$fz/Cargo.toml" ||
  fail "F2 fuzz/Cargo.toml has no [workspace] table (it must be its own workspace root)"
grep -qE '^[[:space:]]*libfuzzer-sys[[:space:]]*=' "$fz/Cargo.toml" ||
  fail "F2 fuzz/Cargo.toml does not depend on libfuzzer-sys"

# ---- F3 ---------------------------------------------------------------------
# `exclude = [...]` inside [workspace] (single- or multi-line array).
excl=$(awk '
  /^[[:space:]]*\[/ { s = $0; gsub(/[[:space:]]/, "", s); sub(/#.*/, "", s); inws = (s == "[workspace]"); next }
  inws && /^[[:space:]]*exclude[[:space:]]*=/ { grab = 1 }
  grab { l = $0; sub(/#.*$/, "", l); printf "%s ", l; if (l ~ /\]/) grab = 0 }' "$root/Cargo.toml")
case "$excl" in
  *'"fuzz"'*) ;;
  *) fail "F3 rust/Cargo.toml [workspace] exclude does not list \"fuzz\"" ;;
esac
if ! meta=$(cd "$root" && cargo metadata --no-deps --format-version 1 2>&1); then
  echo "check-fuzz-policy: cargo metadata failed for the main workspace:" >&2
  printf '%s\n' "$meta" | sed 's/^/  /' >&2
  exit 2
fi
inside=$(printf '%s' "$meta" | jq -r '
  (.workspace_root + "/fuzz/") as $fz
  | [.workspace_members[] as $m | .packages[] | select(.id == $m)
     | select(.manifest_path | startswith($fz)) | .name] | .[]')
[[ -z "$inside" ]] || fail "F3 main-workspace member(s) under fuzz/: $(echo "$inside" | tr '\n' ' ')"

# ---- F4 ---------------------------------------------------------------------
dep=$(printf '%s' "$meta" | jq -r '
  [.workspace_members[] as $m | .packages[] | select(.id == $m)
   | .name as $n | .dependencies[] | select(.name == "libfuzzer-sys" or .rename == "libfuzzer-sys")
   | "\($n) (\(.kind // "normal"))"] | unique | .[]')
[[ -z "$dep" ]] || fail "F4 main-workspace crate(s) depend on libfuzzer-sys: $(echo "$dep" | tr '\n' ' ')"
if grep -qE '^name[[:space:]]*=[[:space:]]*"libfuzzer-sys"' "$root/Cargo.lock"; then
  fail "F4 rust/Cargo.lock contains libfuzzer-sys (it must stay in fuzz/Cargo.lock only)"
fi

# ---- F5 ---------------------------------------------------------------------
if grep -q 'FUZZ-DELTA' "$root/deny.toml"; then
  fail "F5 rust/deny.toml contains a FUZZ-DELTA marker (the product policy has no fuzz delta)"
fi
drift=$(awk -v mainf="$root/deny.toml" '
  function bad(msg) { printf "F5 fuzz/deny.toml:%d: %s\n", NR, msg }
  function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
  function blankc(s) { return s ~ /^[ \t]*(#.*)?$/ }
  # The cc ban line: prints its wrappers list, or "" if s is not that line.
  function ccw(s,  t) {
    t = trim(s)
    if (t !~ /^\{ crate = "cc", wrappers = \[[^]]*\], reason = "[^"]*" \},$/) return ""
    sub(/^\{ crate = "cc", wrappers = \[/, "", t); sub(/\], reason = .*$/, "", t)
    return t
  }
  function ctx(s,  t) { # track [table] and the open `key = [` array
    t = s; sub(/#.*$/, "", t); t = trim(t)
    if (t ~ /^\[[^]]+\]$/) { sect = substr(t, 2, length(t) - 2); arr = ""; return }
    if (t ~ /^[A-Za-z0-9_-]+[ \t]*=[ \t]*\[$/) { arr = t; sub(/[ \t]*=.*$/, "", arr); return }
    if (t == "]") arr = ""
  }
  BEGIN {
    n = 0
    while ((getline l < mainf) > 0) m[++n] = l
    close(mainf)
    i = 1; inblk = 0; sect = ""; arr = ""
  }
  {
    line = $0
    if (line ~ /^[ \t]*# >>> FUZZ-DELTA replaces=[0-9]+(:.*)?$/) {
      if (inblk) { bad("nested FUZZ-DELTA block"); next }
      r = line; sub(/^.*replaces=/, "", r); sub(/[^0-9].*$/, "", r); r += 0
      if (i + r - 1 > n) { bad("block replaces " r " line(s) past the end of rust/deny.toml"); r = n - i + 1 }
      inblk = 1; blk_start = NR; repl_cc = ""; repl_ccn = 0; blk_ccn = 0
      for (k = i; k < i + r; k++) {
        if (blankc(m[k])) continue
        w = ccw(m[k])
        if (w != "") { repl_cc = w; repl_ccn++; continue }
        bad("block replaces policy line " k " of rust/deny.toml: " trim(m[k]))
      }
      i += r
      next
    }
    if (line ~ /^[ \t]*# <<< FUZZ-DELTA[ \t]*$/) {
      if (!inblk) { bad("FUZZ-DELTA end without a start"); next }
      if (repl_ccn != blk_ccn) bad("block from line " blk_start " replaces " repl_ccn " cc ban line(s) but contains " blk_ccn)
      inblk = 0
      next
    }
    if (line ~ /^[ \t]*#[ \t]*(>>>|<<<)/ || line ~ /(>>>|<<<)[ \t]*FUZZ-DELTA/) { bad("malformed FUZZ-DELTA marker: " trim(line)); next }
    if (!inblk) {
      if (i > n) bad("extra line after the end of rust/deny.toml: " trim(line))
      else if (line != m[i]) bad("differs from rust/deny.toml line " i " outside a FUZZ-DELTA block")
      i++
      ctx(line)
      next
    }
    # Inside a block: comments, blank lines and the approved exception only.
    if (blankc(line)) next
    t = trim(line)
    if (t == "{ allow = [\"NCSA\"], crate = \"libfuzzer-sys\" },") {
      if (sect != "licenses" || arr != "exceptions") bad("NCSA exception outside [licenses] exceptions")
      if (seen_ncsa++) bad("NCSA exception repeated")
      next
    }
    t2 = t; sub(/[ \t]*#.*$/, "", t2)
    if (t2 == "\"libfuzzer-sys\",") {
      if (sect != "bans.build" || arr != "allow-build-scripts") bad("libfuzzer-sys build-script entry outside [bans.build] allow-build-scripts")
      if (seen_bs++) bad("libfuzzer-sys build-script entry repeated")
      next
    }
    w = ccw(line)
    if (w != "") {
      blk_ccn++
      if (sect != "bans" || arr != "deny") bad("cc ban outside [bans] deny")
      if (repl_ccn != 1) bad("cc ban in a block that does not replace the rust/deny.toml cc line")
      else if (w != repl_cc ", \"libfuzzer-sys\"") bad("cc wrappers must be rust/deny.toml [" repl_cc "] plus \"libfuzzer-sys\" (got [" w "])")
      if (seen_cc++) bad("cc ban repeated")
      next
    }
    bad("not part of the approved fuzz exception: " t)
  }
  END {
    if (inblk) bad("FUZZ-DELTA block from line " blk_start " is not closed")
    if (i <= n) printf "F5 fuzz/deny.toml: ends before rust/deny.toml line %d (%s)\n", i, trim(m[i])
    if (!seen_ncsa) print "F5 fuzz/deny.toml: missing FUZZ-DELTA for the libfuzzer-sys NCSA licence exception"
    if (!seen_cc) print "F5 fuzz/deny.toml: missing FUZZ-DELTA for the cc ban (libfuzzer-sys wrapper)"
    if (!seen_bs) print "F5 fuzz/deny.toml: missing FUZZ-DELTA for libfuzzer-sys in allow-build-scripts"
  }' "$fz/deny.toml")
if [[ -n "$drift" ]]; then
  while IFS= read -r l; do fail "$l"; done <<EOF
$drift
EOF
fi

# ---- F6 ---------------------------------------------------------------------
entries=$(sed -e 's/#.*$//' "$fz/native-allowlist.txt" | awk 'NF > 0 { print $1 }')
if [[ "$entries" != "libfuzzer-sys" ]]; then
  fail "F6 fuzz/native-allowlist.txt must name only libfuzzer-sys (got: $(printf '%s' "$entries" | tr '\n' ' '))"
fi

# ---- F7 ---------------------------------------------------------------------
tcf="$fz/rust-toolchain.toml"
tc=$(awk '
  /^[[:space:]]*\[/ { s = $0; gsub(/[[:space:]]/, "", s); sub(/#.*/, "", s); intc = (s == "[toolchain]"); next }
  /^[[:space:]]*(#|$)/ { next }
  intc { print }' "$tcf")
nchan=$(printf '%s\n' "$tc" | grep -cE '^[[:space:]]*channel[[:space:]]*=' || true)
chan=$(printf '%s\n' "$tc" | sed -n -E 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([^"]*)"[[:space:]]*(#.*)?$/\1/p')
if [[ "$nchan" -ne 1 ]]; then
  fail "F7 fuzz/rust-toolchain.toml needs exactly one channel under [toolchain] (found $nchan)"
elif ! [[ "$chan" =~ ^nightly-[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$ ]]; then
  fail "F7 fuzz/rust-toolchain.toml channel must be a dated nightly-YYYY-MM-DD (got '${chan:-<unparsed>}')"
fi
if printf '%s\n' "$tc" | grep -qE '^[[:space:]]*path[[:space:]]*='; then
  fail "F7 fuzz/rust-toolchain.toml must not set path (a local, unpinned toolchain)"
fi

# ---- F8 ---------------------------------------------------------------------
norm() { tr ' ' '\n' | sed '/^$/d' | LC_ALL=C sort | tr '\n' ' ' | sed 's/ $//'; }
bins=$(awk '
  /^[[:space:]]*\[/ { s = $0; gsub(/[[:space:]]/, "", s); sub(/#.*/, "", s); inbin = (s == "[[bin]]"); next }
  inbin && /^[[:space:]]*name[[:space:]]*=/ { v = $0; sub(/^[^"]*"/, "", v); sub(/".*$/, "", v); print v }' "$fz/Cargo.toml" | norm)
if [[ -z "$bins" ]]; then
  fail "F8 fuzz/Cargo.toml defines no [[bin]] fuzz targets"
fi
for b in $bins; do
  [[ -f "$fz/fuzz_targets/$b.rs" ]] || fail "F8 fuzz target $b has no fuzz/fuzz_targets/$b.rs"
done
wf="$repo/.github/workflows/rust-fuzz.yml"
if [[ ! -f "$wf" ]]; then
  fail "F8 ${wf#"$repo"/} is missing"
else
  # The block list under `target:` inside `matrix:` (one `- name` per line).
  mx=$(awk '
    /^[[:space:]]*matrix:[[:space:]]*$/ { inm = 1; next }
    inm && /^[[:space:]]*target:[[:space:]]*$/ { grab = 1; next }
    grab && /^[[:space:]]*#/ { next }
    grab && /^[[:space:]]*-[[:space:]]+[A-Za-z0-9_]+[[:space:]]*$/ { v = $0; sub(/^[[:space:]]*-[[:space:]]+/, "", v); sub(/[[:space:]]+$/, "", v); print v; next }
    grab { exit }' "$wf" | norm)
  [[ "$mx" = "$bins" ]] || fail "F8 rust-fuzz.yml matrix targets [$mx] != fuzz/Cargo.toml [[bin]] [$bins]"
fi
seeds="$root/scripts/fuzz-seeds.sh"
if [[ ! -f "$seeds" ]]; then
  fail "F8 scripts/fuzz-seeds.sh is missing"
else
  st=$(sed -n -E 's/^targets="([^"]*)"[[:space:]]*$/\1/p' "$seeds" | head -n 1 | norm)
  [[ "$st" = "$bins" ]] || fail "F8 scripts/fuzz-seeds.sh targets [$st] != fuzz/Cargo.toml [[bin]] [$bins]"
fi

# ---- F9 ---------------------------------------------------------------------
cc="$root/scripts/ci-changes.sh"
if [[ ! -f "$cc" ]]; then
  fail "F9 scripts/ci-changes.sh is missing"
elif [[ -f "$wf" ]]; then
  nri=$(grep -cE "^rust_inputs='[^']*'[[:space:]]*$" "$cc" || true)
  ri=$(sed -n -E "s/^rust_inputs='([^']*)'[[:space:]]*$/\1/p" "$cc")
  if [[ "$nri" -ne 1 ]] || [[ -z "$ri" ]]; then
    fail "F9 scripts/ci-changes.sh needs exactly one non-empty rust_inputs='...' line (found $nri)"
  else
    # Block list under `pull_request:` -> `paths:` (quotes stripped).
    prpaths=$(awk -v q="'" '
      /^[^[:space:]#]/ { inon = ($0 ~ /^on:[[:space:]]*(#.*)?$/); inpr = 0; inp = 0; next }
      !inon { next }
      /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
      { match($0, /^[[:space:]]*/); ind = RLENGTH }
      inp && ind <= pind { inp = 0 }
      inpr && ind <= prind { inpr = 0 }
      /^[[:space:]]*pull_request:[[:space:]]*(#.*)?$/ { inpr = 1; prind = ind; next }
      inpr && /^[[:space:]]*paths:[[:space:]]*(#.*)?$/ { inp = 1; pind = ind; next }
      inp && /^[[:space:]]*-[[:space:]]/ {
        v = $0; sub(/^[[:space:]]*-[[:space:]]+/, "", v); sub(/[[:space:]]+#.*$/, "", v); sub(/[[:space:]]+$/, "", v)
        if (v ~ /^".*"$/ || (substr(v, 1, 1) == q && substr(v, length(v)) == q)) v = substr(v, 2, length(v) - 2)
        print v
      }' "$wf")
    IFS='|' read -r -a alts <<<"$ri"
    for a in "${alts[@]}"; do
      if ! [[ "$a" =~ ^\^[A-Za-z0-9._-]+(/[A-Za-z0-9._-]+)*/$ ]]; then
        fail "F9 rust_inputs alternative '$a' is not a literal ^dir/ prefix; F9 cannot check it against rust-fuzz.yml"
        continue
      fi
      want="${a#^}**"
      printf '%s\n' "$prpaths" | grep -qxF "$want" ||
        fail "F9 rust-fuzz.yml on.pull_request.paths lacks \"$want\" (rust_inputs in scripts/ci-changes.sh; the fuzzers read it through the build)"
    done
  fi
fi

[[ "$status" -eq 0 ]] && echo "check-fuzz-policy: ok (fuzz/ excluded, no libfuzzer-sys in the product graph, deny.toml delta approved, toolchain $chan, targets: $bins, rust_inputs in rust-fuzz.yml PR paths)"
exit "$status"
