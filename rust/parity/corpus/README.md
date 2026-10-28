<!-- SPDX-License-Identifier: Apache-2.0 -->

# Golden corpus: CoT XML and TAK Protocol v1 conversion

This corpus is the evidence for the Phase 1a exit criterion: TAK-RS converts
CoT XML to TAK Protocol v1 (and back) byte for byte as the Java reference
server does. The inputs were written for TAK-RS. The goldens are the Java
reference's own output on those inputs, captured by the parity harness.
`rust/crates/tak-cot/tests/golden.rs` compares every case on every PR
(`make test`).

## Layout

```text
corpus/
  README.md          this file
  .gitattributes     "* -text": no end-of-line conversion (some inputs contain CR on purpose)
  MANIFEST.tsv       one row per (id, direction); the five number tables have one row each
  cases/xml/         X2P and X2X inputs (exact bytes, UTF-8)
  cases/proto/       P2X and P2P inputs: <id>.txtpb (text format) or <id>.pb.hex (raw bytes)
  cases/numbers/     doubles.tsv, double-strings.txt, int-strings.txt, times-ms.txt, time-strings.txt
  cases/REUSE.toml   licence and provenance of every input file
  driver/CorpusDriver.java   Apache-2.0 driver run by `harness.sh capture-corpus`
  golden/            imported captures (never edited by hand)
    PROVENANCE.json  build, pins, images, JVM environment, SHA-256 of the driver, MANIFEST and every input
    results.tsv      id, direction, outcome (ok|reject), output path, detail
    DEVIATIONS.tsv   id, direction, deviation, approved_on (user-approved rows only)
    proto-inputs/<id>.pb.hex   protobuf-java's encoding of each .txtpb input
    x2p/<id>.pb.hex, p2p/<id>.pb.hex   TakMessage bytes
    x2x/<id>.xml, p2x/<id>.xml         exact XML bytes
    numbers/{dbl-fmt,dbl-parse,int-parse,time-fmt,time-parse}.tsv
```

`golden/` appears after the first capture is imported (WU7). Until then
`verify-corpus` exits 3 and the golden test fails, and neither is ever
silently skipped.

## Input format

Owned by the qa-engineer (WU6a). The driver enforces these rules and fails
the capture (exit 1) on any violation. A bad input is never recorded as a
`reject`.

- **`MANIFEST.tsv`**: Tab-separated, UTF-8.
  - Lines starting with `#` are comments.
  - The first other line is the header. It must name the columns `id`,
    `direction`, `input`, `creation_ms` and `submission_ms`, in any order.
    Other columns, such as `purpose`, are ignored.
  - Each row holds one (id, direction) pair. A pair must not appear twice,
    and an id names one input file.
- **`id`**: matches `[A-Za-z0-9][A-Za-z0-9._-]{0,127}`.
- **`direction`**: one of these:
  - `x2p` or `x2x`, with an input under `cases/xml/`;
  - `p2x` or `p2p`, with an input under `cases/proto/`;
  - a number mode (`dbl-fmt`, `dbl-parse`, `int-parse`, `time-fmt`,
    `time-parse`), with an input under `cases/numbers/`. Exactly one row
    per mode is required.
- **`input`**: a path relative to this directory, under `cases/`. Paths
  with `..` and symlinks are refused.
- **`creation_ms` and `submission_ms`**: a decimal i64, or `-`, which means
  the setter is not called.
  - X2P calls `setCreationTime(creation_ms)` and
    `setSubmissionTime(new Date(submission_ms))`. Every X2P row needs a
    numeric `creation_ms`. Otherwise the container's wall-clock time leaks
    into the output, and the run-twice check fails.
  - P2P calls `setCreationTime(creation_ms)` only when the input has no
    `creationTime`.
  - X2X and P2X ignore both columns.
- **XML inputs**: exact bytes, decoded strictly as UTF-8 into the `String`
  given to `CotParser.parse`, as the server's stream path does. There is no
  byte-level encoding detection. Each file is at most 16 MiB.
- **`.txtpb`**: protobuf text format, UTF-8. It is encoded by the reference's
  protobuf-java, and the result is recorded in `golden/proto-inputs/`, so
  TAK-RS needs no text-format parser.
- **`.pb.hex`**: hex digits. ASCII whitespace is ignored, and `#` starts a
  comment that runs to the end of the line. The committed inputs are one
  lowercase line plus LF.
- **`doubles.tsv`**: `#` comment lines, then a header row that names a
  `bits` column (16 hex digits, the IEEE-754 bit pattern). Other columns,
  such as `set`, are not interpreted by the driver.
- **`.txt` tables**: one raw string per line.
  - Lines are split on LF only and never trimmed.
  - An empty line is the empty string.
  - There is no escaping and no comment lines.
  - `times-ms.txt` lines must be decimal i64 values.

## Output format (goldens)

- **`results.tsv`**: the header
  `id<TAB>direction<TAB>outcome<TAB>output<TAB>detail`, then one row per
  manifest case in manifest order.
  - `ok` rows have `output` = `<direction>/<id>.<ext>` and an empty
    `detail`.
  - `reject` rows have `output` = `-`, and `detail` holds the Java exception
    class, or `null` when the reference returned null. `detail` is
    informational only: TAK-RS is compared on ok or reject, never on the
    exception class.
- **`*.pb.hex`**: lowercase hex of the unframed TakMessage, plus one LF.
- **`*.xml`**: the exact UTF-8 bytes of `asXml()`, with nothing added.
- **`numbers/*.tsv`**: the header `line<TAB>input<TAB>outcome<TAB>value`,
  one row per input line. `line` is the 1-based line number in the input
  table.
  - `input` is escaped and ASCII-only: `\\`, `\t`, `\n`, `\r`, and `\uXXXX`
    for controls and non-ASCII characters.
  - In `dbl-fmt`, `input` is the bit pattern, `outcome` is `ok`, and `value`
    is `Double.toString`.
  - In `dbl-parse` and `int-parse`, the reference is called with two
    different defaults (a sentinel probe). `outcome` is `default` (empty
    `value`) when both calls return their default. Otherwise it is `ok`,
    with `value` = the 16-hex-digit bit pattern or the decimal int.
  - In `time-fmt` and `time-parse`, `outcome` is `ok` (`value` = the CoT
    string or the milliseconds) or `reject` (`value` = the exception class).

## Byte equivalence

Byte equality is exact, and no normalisation is applied (plan §1.4.5).
Outcome equality (ok or reject) is required in every direction.

| Dir | Java (reference, black box) | Equal means |
|-----|-----------------------------|-------------|
| X2P | `cot2protoBuf(new CotEventContainer(new CotParser(false).parse(xml)))`, with the manifest's times | identical TakMessage bytes, no framing |
| X2X | `new CotEventContainer(parse(xml)).asXml()`, UTF-8 | identical bytes |
| P2X | `proto2cot(TakMessage.parseFrom(pb)).asXml()`, UTF-8 | identical bytes, including the XML declaration and anything after it |
| P2P | `cot2protoBuf(proto2cot(parseFrom(pb)))` | identical bytes |
| numbers, times | `Double.toString`, `NumericUtil`, `DateUtil` | identical string, bits, value or outcome |

A case may differ only if it has a `golden/DEVIATIONS.tsv` row, keyed by
(id, direction), for a deviation the user approved in the parity map. The
`random` and `probe` rows of `dbl-fmt` are measured, not gated (Q4, DEV-03).

## Capturing and updating goldens

The capture runs in CI (`rust-parity.yml`): nightly, on PRs that touch
`rust/parity/**`, `rust/crates/tak-cot/**` or `rust/crates/tak-proto/**`,
and on `workflow_dispatch`. Locally it needs Docker or Podman:

```sh
rust/parity/harness.sh preflight build-java image capture-corpus verify-corpus
```

`capture-corpus` does the following:

1. Extracts `WEB-INF/lib/*.jar` from the built `takserver.war`. Absolute
   paths, `..`, symlink entries, duplicate entries and entries over 256 MiB
   are refused with exit 4.
2. Requires `StreamingProtoBufHelper` on that classpath, else exit 3.
3. Compiles the driver with `javac --release 17` in the JDK builder image.
4. Runs the driver **twice** in the pinned JRE 17 image. Each run has no
   network, a read-only root filesystem, no capabilities and
   `no-new-privileges`, and runs as the invoking non-root uid. The inputs
   and classpath are mounted read-only, and only the run's output
   directory is writable.
5. Checks the output before anything reads it: symlinks and special files
   are removed and fail the capture with exit 4.
6. Fails unless both runs are byte-identical.
7. Writes `<results>/<UTC>-corpus/`, including `PROVENANCE.json`.

`verify-corpus` fails with exit 1 and lists the drifting cases when the
capture differs from `golden/`.

To update the goldens after an intended change, such as new inputs or an
upstream reference change:

1. Run the `rust-parity` workflow with `workflow_dispatch`. Set
   `verify_corpus: false` for the very first capture.
2. Download the `parity-corpus-<run>` artifact.
3. Run `rust/parity/harness.sh import-corpus <dir>/<UTC>-corpus`. This
   refuses a capture whose pins, driver or inputs differ from the
   checkout's (exit 4), and keeps `golden/DEVIATIONS.tsv`.
4. Review the diff of `golden/` in the PR, and record the run URL and the
   artifact digest in TEST_EVIDENCE.

Never edit a golden by hand. The next capture would report it as drift.

## Licence and provenance

- **Inputs and driver**: Apache-2.0, written for TAK-RS (see
  `cases/REUSE.toml`). They are synthetic: no fielded captures, no real
  callsigns, UIDs, devices or sites, and no CUI or PII. Nothing is taken
  from `src/`, including its tests and test resources.
- **Driver**: calls only public APIs of the reference jars (`CotParser`,
  `CotEventContainer`, `StreamingProtoBufHelper`, `DateUtil`,
  `NumericUtil`, protobuf-java `TextFormat`). It contains no code from
  `src/`. It links against the GPLv3 jars only inside the test container,
  and neither it nor the jars ship with TAK-RS.
- **Goldens**: machine output of the GPLv3 reference on the inputs above.
  They hold only input data and wire-visible literals. They are committed
  as Apache-2.0 test data with `PROVENANCE.json` (SR-4). Whether this needs
  anything further is in the scope of POAM-2026-10-26-05 (decision Q9).
- **Trust**: the goldens are trusted because they reproduce, not because
  they were reviewed. Every capture re-derives them, and any difference
  fails.

## Selftest fixtures

`../tests/fixtures/corpus/` holds a small synthetic input tree and a
capture-shaped output tree. The output came from running the driver against
API stubs, so it is not Java output and is used only for the layout,
verify and import checks in `harness.sh selftest`.
