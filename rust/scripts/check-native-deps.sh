#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Pure-Rust policy, layer two: every package with a `links` key in a shipping
# target's dependency graph must be allow-listed for that target in
# scripts/native-allowlist.txt. Layer one is deny.toml [bans].
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SR-3 (Supply Chain Controls and Processes), SA-15
#   (Development Process, Standards, and Tools)
#
# Graph per target (feature-resolved by `cargo tree --all-features`):
# - Linux targets: the whole workspace (all members; normal, build and dev
#   edges).
# - wasm32-unknown-unknown: only the closure of the crates built for wasm32
#   (WASM_ROOTS; normal, build and dev edges), so native-only crates elsewhere
#   in the workspace do not count but any path from a web crate to native code
#   does.
# Build dependencies count for the native-code (`links`) rule, because build
# scripts run on the build host; dev dependencies count because `make test`
# builds them. Both rules below use these full graphs except the wasm32
# crypto-name rule.
#
# Also enforced here:
# - every workspace member whose manifest is under web/ must be in WASM_ROOTS
#   (WEB-001), so a new web crate cannot skip the wasm32 graph;
# - the wasm32 SHIPPED closure must not contain any cryptographic or TLS crate
#   by name (WEB-004; list in crypto_re below). Crypto belongs to the native
#   FIPS module only (D5, SC-13); RustCrypto crates have no `links` key.
#   "Shipped" = what can end up in a wasm artifact: the closure of WASM_ROOTS
#   over NORMAL edges only (`cargo tree -e normal`), so a dev-only crate (for
#   example proptest's ChaCha RNG, chacha20, in tests) or a build-host-only
#   crate does not trip the rule, while a crypto crate reachable through any
#   chain of normal dependencies still fails. Proc-macro crates and their
#   normal deps are included (over-approximation: they run on the host, not
#   in the artifact; a crypto crate there still fails, which is intended).
#   Dev/build crypto crates remain subject to deny.toml and the `links` rule.
#
# HEURISTIC: the `links` key is how Cargo declares native linking, but a build
# script can emit `cargo:rustc-link-lib` without it, and a crate can ship a
# pre-built archive. Those are covered by deny.toml [bans.build]
# (`include-archives = true`, `allow-build-scripts`), not by this script.
#
# Environment (overrides, used by selftest.sh):
#   RUST_ROOT        workspace root (default: parent of this script)
#   NATIVE_ALLOWLIST allow-list file (default: scripts/native-allowlist.txt)
#   NATIVE_TARGETS   space-separated triples (default: the three shipping targets)
#   WASM_ROOTS       space-separated package names built for wasm32
# No network beyond what cargo itself needs; set CARGO_NET_OFFLINE=true to forbid it.
#
# Exit: 0 pass, 1 policy violation, 2 usage/tooling error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=${RUST_ROOT:-$(cd "$here/.." && pwd)}
allowlist=${NATIVE_ALLOWLIST:-$here/native-allowlist.txt}
targets=${NATIVE_TARGETS:-"x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu wasm32-unknown-unknown"}
wasm_roots=${WASM_ROOTS:-"tak-api-types tak-web-components tak-proto tak-cot"}  # keep equal to Makefile WASM_CRATES
all_targets="x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu,wasm32-unknown-unknown"
# Crate names never allowed in the wasm32 closure (anchored ERE on the name).
crypto_re='^(tak-crypto|tak-auth|aws-lc-.*|rustls.*|ring|openssl.*|boring.*|sha1|sha2|sha3|hmac|aes.*|chacha20.*|rsa|p256|p384|p521|ecdsa|ed25519.*|x25519.*|blowfish|bcrypt|pbkdf2|hkdf|digest|md-5|md5|argon2|scrypt|curve25519-.*|libsodium.*|sodiumoxide)$'

command -v jq >/dev/null 2>&1 || { echo "check-native-deps: jq is required" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { echo "check-native-deps: cargo is required" >&2; exit 2; }
[ -f "$allowlist" ] || { echo "check-native-deps: allow-list not found: $allowlist" >&2; exit 2; }

# Normalise the allow-list to "<crate> <class> <targets>" lines; reject malformed rows.
entries=$(sed -e 's/#.*$//' "$allowlist" | awk 'NF > 0 { print $1, $2, $3, NF }')
bad=$(printf '%s\n' "$entries" | awk 'NF > 0 && ($4 != 3 || $2 !~ /^(native|native-unused|build-native|marker)$/) { print }')
if [ -n "$bad" ]; then
  echo "check-native-deps: FAIL malformed allow-list entries (need <crate> <class> <targets>, class in native|native-unused|build-native|marker):" >&2
  printf '  %s\n' "$bad" >&2
  exit 1
fi
if printf '%s\n' "$entries" | awk '$1 ~ /^aws-lc/ && ($3 == "all" || $3 ~ /wasm32/) { found = 1 } END { exit !found }'; then
  echo "check-native-deps: FAIL an aws-lc-* allow-list entry covers wasm32-unknown-unknown" >&2
  exit 1
fi

# allowed <crate> <target> -> prints class and returns 0 if allow-listed for target.
allowed() {
  printf '%s\n' "$entries" | awk -v c="$1" -v t="$2" -v all="$all_targets" '
    $1 == c {
      ts = ($3 == "all") ? all : $3
      n = split(ts, a, ",")
      for (i = 1; i <= n; i++) if (a[i] == t) { print $2; found = 1; exit }
    }
    END { exit !found }'
}

# Reachable set comes from `cargo tree` (feature-resolved, as cargo builds it).
# `cargo metadata .resolve` is NOT used for reachability: it also lists optional
# dependencies named only through weak features (e.g. rustls-webpki
# `ring?/alloc` makes `ring` appear although it is never built). metadata is
# used only for `links` and the workspace member list.
# <edges> is a cargo tree edge list: normal,build,dev (full graph) or normal
# (shipped closure). An argument, not an environment variable, so a caller's
# environment cannot narrow the full graph.
reachable() { # <edges> <target> <tree args...>
  local e=$1 t=$2; shift 2
  (cd "$root" && cargo tree --locked --all-features --target "$t" -e "$e" \
      --prefix none --format '{p}' "$@") |
    awk 'NF >= 2 && $2 ~ /^v[0-9]/ { v = $2; sub(/^v/, "", v); print $1, v }' | sort -u
}

status=0
for t in $targets; do
  echo "== $t"
  meta=$(cd "$root" && cargo metadata --format-version 1 --locked --all-features --filter-platform "$t") || {
    echo "check-native-deps: cargo metadata failed for $t" >&2; exit 2; }
  mode=all
  case "$t" in
    wasm32-*)
      mode=closure
      # shellcheck disable=SC2086 # one root per line
      roots_json=$(printf '%s\n' $wasm_roots | jq -R . | jq -s .)
      missing=$(printf '%s' "$meta" | jq -r --argjson roots "$roots_json" '
        [.workspace_members[] as $m | .packages[] | select(.id == $m) | .name] as $names
        | $roots[] | select(. as $r | $names | index($r) | not)')
      if [ -n "$missing" ]; then
        echo "  FAIL wasm32 root(s) not workspace members: $missing (update WASM_ROOTS / Makefile WASM_CRATES)"
        status=1
        continue
      fi
      # WEB-001: every member under web/ must be a wasm32 root.
      unlisted=$(printf '%s' "$meta" | jq -r --argjson roots "$roots_json" '
        (.workspace_root + "/web/") as $web
        | [.workspace_members[] as $m | .packages[] | select(.id == $m)
         | select(.manifest_path | startswith($web)) | .name] | .[]
        | select(. as $n | $roots | index($n) | not)')
      if [ -n "$unlisted" ]; then
        echo "  FAIL web/ member(s) not in WASM_ROOTS: $(echo "$unlisted" | tr '\n' ' ')(add them to Makefile WASM_CRATES)"
        status=1
        continue
      fi
      echo "  graph: closure of $wasm_roots"
      ;;
    *) echo "  graph: whole workspace" ;;
  esac
  links=$(printf '%s' "$meta" | jq -r '.packages[] | select(.links != null) | "\(.name) \(.version) \(.links)"' | sort -u)
  if [ "$mode" = all ]; then
    pkgs=$(reachable normal,build,dev "$t" --workspace) || { echo "check-native-deps: cargo tree failed for $t" >&2; exit 2; }
  else
    pargs=""
    for r in $wasm_roots; do pargs="$pargs -p $r"; done
    # shellcheck disable=SC2086 # word splitting of -p args is intended
    pkgs=$(reachable normal,build,dev "$t" $pargs) || { echo "check-native-deps: cargo tree failed for $t" >&2; exit 2; }
    # Shipped closure for the crypto-name rule: normal edges only.
    # shellcheck disable=SC2086 # word splitting of -p args is intended
    shipped=$(reachable normal "$t" $pargs) || { echo "check-native-deps: cargo tree failed for $t" >&2; exit 2; }
  fi
  if [ "$mode" = closure ]; then
    crypto=$(printf '%s\n' "$shipped" | awk -v re="$crypto_re" 'NF >= 2 && $1 ~ re { print $1 " " $2 }')
    if [ -n "$crypto" ]; then
      while read -r name version; do
        echo "  FAIL  $name $version is a crypto/TLS crate in the shipped wasm32 closure (normal edges; D5: crypto only in the native FIPS module)"
        (cd "$root" && cargo tree --locked --all-features -i "$name@$version" -e normal --target "$t" 2>&1 || true) | sed 's/^/          /'
      done <<EOF
$crypto
EOF
      status=1
    fi
  fi
  found=$(awk 'NR == FNR { r[$1 " " $2] = 1; next } ($1 " " $2) in r' <(printf '%s\n' "$pkgs") <(printf '%s\n' "$links") | sort -u)
  present=""
  if [ -n "$found" ]; then
    while read -r name version lk; do
      present="$present $name"
      if class=$(allowed "$name" "$t"); then
        echo "  ok    $name $version (links=$lk) class=$class"
      else
        echo "  FAIL  $name $version (links=$lk) is not allow-listed for $t"
        echo "        dependency path:"
        (cd "$root" && cargo tree --locked --all-features -i "$name@$version" -e normal,build --target "$t" 2>&1 || true) | sed 's/^/          /'
        status=1
      fi
    done <<EOF
$found
EOF
  fi
  # Pre-approved entries for this target that are absent: info only.
  for c in $(printf '%s\n' "$entries" | awk '{ print $1 }'); do
    case " $present " in *" $c "*) continue ;; esac
    if class=$(allowed "$c" "$t"); then
      echo "  info  $c ($class) allow-listed for $t, not in graph"
    fi
  done
done

if [ "$status" -ne 0 ]; then
  echo "check-native-deps: FAIL"
else
  echo "check-native-deps: ok"
fi
exit "$status"
