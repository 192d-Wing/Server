#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
#
# Builds the libFuzzer seed directories for the rust/fuzz/ targets from the
# golden-corpus inputs in parity/corpus/cases/ (plan section 1.8, WU9).
#
#   scripts/fuzz-seeds.sh <out-dir>                 every target
#   scripts/fuzz-seeds.sh <out-dir> --only <target>  that target only
#
# Creates <out-dir>/<target>/ for every target (or only <target>):
#   cot_parse, cot_to_proto            xml/*.xml as is
#   proto_decode, proto_to_cot,
#   proto_roundtrip                    proto/*.pb.hex decoded to bytes
#   stream_frame                       each decoded .pb.hex as a stream frame
#                                      (0xbf, varint length, payload) and raw
#   javafmt                            one seed per line of numbers/*.txt
# Inputs larger than FUZZ_SEED_MAX_BYTES (default 65536) are skipped and
# counted: the 8 MiB, 1 MiB and 400 KB limit cases would otherwise raise
# libFuzzer's max_len to their size and slow every target. Limit enforcement
# is covered by the tak-cot/tak-proto tests and proptest properties.
# Seeds are generated, never committed (rust/fuzz/.gitignore).
#
# Concurrency (QA-003): `make fuzz-run FUZZ_TARGET=<t>` passes `--only <t>`,
# so parallel runs for different targets never touch each other's seed
# directory. Seeds are built in a private staging directory under <out-dir>
# (mktemp, same filesystem) and each <out-dir>/<target> is replaced by a
# rename, never emptied in place, so a reader sees the old or the new set,
# not a half-written one. Two runs for the SAME target are not supported
# (they share corpus/<t> and artifacts/<t> anyway); the seeds they see are
# still complete.
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SA-11(8) (Dynamic Code Analysis), SA-15 (Development
#   Process, Standards, and Tools)
#
# Environment: CORPUS_CASES (default: parity/corpus/cases under the workspace
# root), FUZZ_SEED_MAX_BYTES.
# Exit: 0 done, 2 usage or input error.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
root=$(cd "$here/.." && pwd)
cases=${CORPUS_CASES:-$root/parity/corpus/cases}
max=${FUZZ_SEED_MAX_BYTES:-65536}

usage() { echo "usage: fuzz-seeds.sh <out-dir> [--only <target>]" >&2; exit 2; }
targets="cot_parse cot_to_proto proto_decode proto_to_cot proto_roundtrip stream_frame javafmt"
case "$#" in
  1) out=$1; want=$targets ;;
  3) [ "$2" = "--only" ] || usage
     out=$1; want=$3
     [ -n "$want" ] || { echo "fuzz-seeds: --only needs a target name (set FUZZ_TARGET); one of: $targets" >&2; exit 2; }
     [[ "$want" =~ ^[a-z0-9_]+$ ]] || { echo "fuzz-seeds: bad target name; one of: $targets" >&2; exit 2; }
     case " $targets " in
       *" $want "*) ;;
       *) echo "fuzz-seeds: unknown target '$want'; one of: $targets" >&2; exit 2 ;;
     esac ;;
  *) usage ;;
esac
[ -n "$out" ] || usage
[[ "$max" =~ ^[1-9][0-9]{0,8}$ ]] || { echo "fuzz-seeds: FUZZ_SEED_MAX_BYTES must be a positive integer (got '$max')" >&2; exit 2; }
for d in xml proto numbers; do
  [ -d "$cases/$d" ] || { echo "fuzz-seeds: missing $cases/$d" >&2; exit 2; }
done
command -v perl >/dev/null 2>&1 || { echo "fuzz-seeds: perl is required" >&2; exit 2; }

wants() { case " $want " in *" $1 "*) return 0 ;; *) return 1 ;; esac; }
mkdir -p "$out"
stage=$(mktemp -d "$out/.staging.XXXXXX")
held=""
cleanup() {
  [ -z "$held" ] || rmdir "$held" 2>/dev/null || true
  rm -rf "$stage"
}
trap cleanup EXIT
# lock <target>: mkdir is atomic (no flock on macOS). Held only for the two
# renames below; a stale lock (killed run) is reported after 30 s.
lock() {
  local l="$out/.lock.$1" i=0
  until mkdir "$l" 2>/dev/null; do
    i=$((i + 1))
    [ "$i" -le 300 ] || { echo "fuzz-seeds: $l held for 30 s; remove it if no fuzz-seeds is running" >&2; exit 2; }
    sleep 0.1
  done
  held=$l
}
unlock() { rmdir "$held"; held=""; }
for t in $targets; do
  mkdir "$stage/$t"
done

skipped=0
size_of() { wc -c <"$1" | tr -d ' '; }
small() { # <file>: true if within the cap, else counts and reports a skip
  local n
  n=$(size_of "$1")
  if [ "$n" -gt "$max" ]; then
    skipped=$((skipped + 1))
    echo "fuzz-seeds: skip ${1#"$cases"/} ($n bytes > $max)"
    return 1
  fi
}

# CoT XML.
while IFS= read -r -d '' f; do
  small "$f" || continue
  b=$(basename "$f")
  cp "$f" "$stage/cot_parse/$b"
  cp "$f" "$stage/cot_to_proto/$b"
done < <(find "$cases/xml" -maxdepth 1 -type f -name '*.xml' -print0 | LC_ALL=C sort -z)

# TAK Protocol v1 payloads (hex text -> bytes), and stream frames of them.
while IFS= read -r -d '' f; do
  b=$(basename "$f" .pb.hex)
  tmp="$stage/proto_decode/$b.pb"
  perl -e 'local $/; my $h = <STDIN>; $h =~ s/\s+//g;
           $h =~ /\A(?:[0-9A-Fa-f]{2})*\z/ or die "not hex\n";
           binmode STDOUT; print pack("H*", $h);' <"$f" >"$tmp" ||
    { echo "fuzz-seeds: $f is not valid hex" >&2; exit 2; }
  if ! small "$tmp"; then rm -f "$tmp"; continue; fi
  cp "$tmp" "$stage/proto_to_cot/$b.pb"
  cp "$tmp" "$stage/proto_roundtrip/$b.pb"
  cp "$tmp" "$stage/stream_frame/$b.raw"
  perl -e 'local $/; binmode STDIN; binmode STDOUT; my $p = <STDIN>; $p = "" unless defined $p;
           my $n = length $p; my $v = "";
           do { my $b = $n & 0x7f; $n >>= 7; $b |= 0x80 if $n; $v .= chr $b } while ($n);
           print "\xbf", $v, $p;' <"$tmp" >"$stage/stream_frame/$b.frame"
done < <(find "$cases/proto" -maxdepth 1 -type f -name '*.pb.hex' -print0 | LC_ALL=C sort -z)

# Java number/time formatting strings: one seed per line.
while IFS= read -r -d '' f; do
  small "$f" || continue
  b=$(basename "$f" .txt)
  perl -e 'my ($dir, $base) = @ARGV; my $i = 0;
           while (my $l = <STDIN>) { chomp $l; $i++;
             open(my $o, ">", sprintf("%s/%s-%04d", $dir, $base, $i)) or die "$!\n";
             binmode $o; print $o $l; close $o or die "$!\n"; }' "$stage/javafmt" "$b" <"$f"
done < <(find "$cases/numbers" -maxdepth 1 -type f -name '*.txt' -print0 | LC_ALL=C sort -z)

# Publish only the requested targets: rename the old directory aside, rename
# the staged one in, then delete the old one. Other targets' directories are
# never touched.
for t in $targets; do
  wants "$t" || continue
  n=$(find "$stage/$t" -type f | wc -l | tr -d ' ')
  lock "$t"
  old=""
  if [ -e "$out/$t" ]; then
    old="$stage/$t.old"
    mv "$out/$t" "$old"
  fi
  mv "$stage/$t" "$out/$t"
  unlock
  [ -z "$old" ] || rm -rf "$old"
  echo "fuzz-seeds: $t $n seed(s)"
done
echo "fuzz-seeds: $skipped input(s) over $max bytes skipped"
