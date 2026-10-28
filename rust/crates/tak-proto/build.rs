// SPDX-License-Identifier: Apache-2.0

//! Generates the TAK Protocol message types from the reference `.proto`
//! schemas, without a `protoc` binary.
//!
//! The schemas are read **in place** from `src/takserver-protobuf` (plan
//! section 1.2, `rust/CONTRIBUTING.md` section Provenance). Nothing is copied
//! into `rust/`: `protox` (a pure-Rust protobuf compiler) turns the files into
//! a `FileDescriptorSet` in memory and `prost-build` writes Rust into
//! `$OUT_DIR` under `target/`, which is never committed or packaged. Schema
//! comments are not emitted (`disable_comments` and no source info), so only
//! the wire contract (package, message and field names, numbers and types)
//! reaches the generated code. Whether that is a derivative of the GPLv3
//! distribution is tracked in POAM-2026-10-26-05.
//!
//! Coupling rule (SEC-008): `scripts/ci-changes.sh` lists
//! `src/takserver-protobuf/` in `rust_inputs`, so an upstream change to these
//! schemas runs the full Rust CI, and `scripts/check-src-coupling.sh` checks
//! that every parent-relative path into `src/` in this file is covered. It sees
//! literal paths, which is why `SCHEMA_DIR` is one string literal and there is
//! no environment-variable override of it.
//!
//! Drift control: the build fails closed when the directory is missing, when a
//! listed schema is missing, when the directory or any subdirectory holds a
//! `.proto` file (extension matched case-insensitively) that is not listed,
//! when a schema's SHA-256 differs from the committed manifest `schema.sha256`
//! (SEC-005), or when the compiled descriptor set holds a file that is
//! neither in `SCHEMAS` nor a bundled `google/protobuf` well-known type
//! (CRY-002: `include_imports` compiles transitive imports, which the digest
//! would otherwise not cover). An upstream schema addition is reviewed and
//! added to `SCHEMAS` by hand, and any content change needs a reviewed update
//! of the manifest; nothing is compiled silently. The digest is taken over the
//! raw bytes: a CRLF checkout fails with a hint rather than being normalised
//! (CRY-003).
//!
//! The digest is a change detector over files already in the repository, not
//! a security function protecting data: anyone who can change a schema can
//! change the manifest, and review of that change is the control. It runs on
//! the build host only and is not linked into any TAK-RS binary, so it is
//! written here (FIPS 180-4, checked against the standard's test vectors
//! before use) rather than adding a hash crate to the graph; the FIPS
//! provider remains the only cryptography in the product. It is a developer
//! configuration-management integrity check (SA-10(1)) outside the TOE, not
//! a runtime integrity verification mechanism, and is not claimed for SI-7
//! or its enhancements (CRY-001). Were it ever relied on against a malicious
//! upstream or reused in product code, it would move to the aws-lc-rs FIPS
//! provider.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: CM-3 (Configuration Change Control), SA-10(1) (Developer
//   Configuration Management | Software and Firmware Integrity Verification),
//   SA-15 (Development Process, Standards, and Tools), SR-3 (Supply Chain
//   Controls and Processes)

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

/// Schema directory, relative to this crate's manifest directory.
const SCHEMA_DIR: &str = "../../../src/takserver-protobuf/src/main/proto";

/// Every schema in `SCHEMA_DIR`, by file name. Keep sorted.
const SCHEMAS: [&str; 15] = [
    "binarypayload.proto",
    "contact.proto",
    "cotevent.proto",
    "detail.proto",
    "fig.proto",
    "group.proto",
    "message.proto",
    "missionannouncement.proto",
    "precisionlocation.proto",
    "status.proto",
    "streaminginput.proto",
    "takcontrol.proto",
    "takmessage.proto",
    "takv.proto",
    "track.proto",
];

/// `google/protobuf` well-known types that protox bundles and a schema may
/// import. They are compiled from protox, not from `SCHEMA_DIR`, so they are
/// not in the manifest; a file of the same name under `SCHEMA_DIR` (which
/// would shadow the bundled one) still fails the build. None is imported
/// today. Keep sorted.
const WELL_KNOWN: [&str; 12] = [
    "google/protobuf/any.proto",
    "google/protobuf/api.proto",
    "google/protobuf/compiler/plugin.proto",
    "google/protobuf/descriptor.proto",
    "google/protobuf/duration.proto",
    "google/protobuf/empty.proto",
    "google/protobuf/field_mask.proto",
    "google/protobuf/source_context.proto",
    "google/protobuf/struct.proto",
    "google/protobuf/timestamp.proto",
    "google/protobuf/type.proto",
    "google/protobuf/wrappers.proto",
];

/// Deepest subdirectory level of `SCHEMA_DIR` scanned for `.proto` files.
const MAX_SCAN_DEPTH: usize = 8;

/// The committed SHA-256 manifest of `SCHEMAS`, relative to this crate's
/// manifest directory.
const MANIFEST: &str = "schema.sha256";

/// Name of the file `prost-build` writes into `$OUT_DIR`; `src/lib.rs`
/// includes it.
const INCLUDE_FILE: &str = "tak_proto_includes.rs";

type BuildResult<T> = Result<T, Box<dyn Error>>;

fn main() -> BuildResult<()> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or("CARGO_MANIFEST_DIR is not set; run this build through cargo")?;
    let schema_dir = manifest_dir.join(SCHEMA_DIR);

    check_schema_dir(&schema_dir)?;
    let manifest = manifest_dir.join(MANIFEST);
    check_schema_digests(&schema_dir, &manifest)?;
    emit(&format!("cargo:rerun-if-changed={}", manifest.display()))?;

    // Cargo scans a directory given to rerun-if-changed recursively, so an
    // added file also triggers a rebuild (and then fails the check above).
    emit(&format!("cargo:rerun-if-changed={}", schema_dir.display()))?;
    for name in SCHEMAS {
        emit(&format!(
            "cargo:rerun-if-changed={}",
            schema_dir.join(name).display()
        ))?;
    }

    let mut compiler = protox::Compiler::new([&schema_dir])?;
    compiler.include_imports(true);
    compiler.include_source_info(false);
    compiler.open_files(SCHEMAS)?;
    let descriptors = compiler.file_descriptor_set();
    check_compiled_files(
        &schema_dir,
        descriptors.file.iter().map(|f| f.name().to_owned()),
    )?;

    let mut config = prost_build::Config::new();
    config
        .disable_comments(["."])
        .enable_type_names()
        .include_file(INCLUDE_FILE);
    config.compile_fds(descriptors)?;
    Ok(())
}

/// Fails unless `dir` exists and holds exactly the `.proto` files in
/// `SCHEMAS`.
fn check_schema_dir(dir: &Path) -> BuildResult<()> {
    if !dir.is_dir() {
        return Err(format!(
            "tak-proto reads the schemas in place from src/takserver-protobuf, \
             but {} is not a directory; build from a full checkout of the repository",
            dir.display()
        )
        .into());
    }

    let missing: Vec<&str> = SCHEMAS
        .iter()
        .copied()
        .filter(|name| !dir.join(name).is_file())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "tak-proto: listed schema(s) missing from {}: {}",
            dir.display(),
            missing.join(", ")
        )
        .into());
    }

    let listed: BTreeSet<&str> = SCHEMAS.iter().copied().collect();
    let mut found = Vec::new();
    scan_protos(dir, "", 0, &mut found)?;
    let mut unlisted: Vec<String> = found
        .into_iter()
        .filter(|name| !listed.contains(name.as_str()))
        .collect();
    if !unlisted.is_empty() {
        unlisted.sort();
        return Err(format!(
            "tak-proto: {} contains schema(s) not listed in build.rs SCHEMAS: {}; \
             review the upstream change and add them deliberately",
            dir.display(),
            unlisted.join(", ")
        )
        .into());
    }
    Ok(())
}

/// Appends to `out` the path, relative to the schema directory and joined
/// with `/`, of every file under `dir` whose extension is `proto` in any
/// case. Subdirectories are followed to `MAX_SCAN_DEPTH`; symbolic links are
/// not followed (a link named `*.proto` is reported like a file).
fn scan_protos(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) -> BuildResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = format!("{prefix}{name}");
        if entry.file_type()?.is_dir() {
            if depth >= MAX_SCAN_DEPTH {
                return Err(format!(
                    "tak-proto: schema directory nesting deeper than {MAX_SCAN_DEPTH} at {rel}"
                )
                .into());
            }
            scan_protos(
                &entry.path(),
                &format!("{rel}/"),
                depth.saturating_add(1),
                out,
            )?;
        } else if Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("proto"))
        {
            out.push(rel);
        }
    }
    Ok(())
}

/// Fails unless every file in the compiled descriptor set is a hashed entry
/// of `SCHEMAS` or a bundled well-known type that `dir` does not shadow
/// (CRY-002).
fn check_compiled_files(dir: &Path, names: impl Iterator<Item = String>) -> BuildResult<()> {
    let listed: BTreeSet<&str> = SCHEMAS.iter().copied().collect();
    let well_known: BTreeSet<&str> = WELL_KNOWN.iter().copied().collect();
    let bundled = |name: &str| well_known.contains(name) && !dir.join(name).exists();
    let mut unhashed: Vec<String> = names
        .filter(|name| !listed.contains(name.as_str()))
        .filter(|name| !bundled(name))
        .collect();
    if unhashed.is_empty() {
        return Ok(());
    }
    unhashed.sort();
    Err(format!(
        "tak-proto: the compiled descriptor set holds file(s) not covered by \
         build.rs SCHEMAS and {MANIFEST}: {}; an import outside SCHEMAS would be \
         compiled without a digest check: list it in SCHEMAS and the manifest \
         after review, or remove the import",
        unhashed.join(", ")
    )
    .into())
}

/// Fails unless `manifest` lists exactly `SCHEMAS`, once each, and every
/// listed digest equals the SHA-256 of that file in `dir`.
fn check_schema_digests(dir: &Path, manifest: &Path) -> BuildResult<()> {
    sha256::self_test()?;
    let update = "an upstream schema change needs a reviewed update of \
                  rust/crates/tak-proto/schema.sha256 (tak-protocol-reviewer), \
                  after checking the hand encoder in src/lib.rs and the parity map";
    let text = fs::read_to_string(manifest).map_err(|e| {
        format!(
            "tak-proto: cannot read the schema manifest {}: {e}",
            manifest.display()
        )
    })?;

    let mut expected: BTreeMap<&str, &str> = BTreeMap::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((digest, name)) = line.split_once("  ") else {
            return Err(format!(
                "tak-proto: {MANIFEST} line {} is not `<sha256>  <file>`",
                number.saturating_add(1)
            )
            .into());
        };
        let well_formed = digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if !well_formed || expected.insert(name, digest).is_some() {
            return Err(format!(
                "tak-proto: {MANIFEST} line {} has a malformed digest or repeats a file",
                number.saturating_add(1)
            )
            .into());
        }
    }

    let listed: BTreeSet<&str> = SCHEMAS.iter().copied().collect();
    let in_manifest: BTreeSet<&str> = expected.keys().copied().collect();
    if listed != in_manifest {
        let missing: Vec<&str> = listed.difference(&in_manifest).copied().collect();
        let extra: Vec<&str> = in_manifest.difference(&listed).copied().collect();
        return Err(format!(
            "tak-proto: {MANIFEST} does not list exactly the schemas in build.rs SCHEMAS \
             (missing: [{}], not in SCHEMAS: [{}]); {update}",
            missing.join(", "),
            extra.join(", ")
        )
        .into());
    }

    let mut changed = Vec::new();
    let mut crlf = Vec::new();
    for (name, digest) in &expected {
        let bytes = fs::read(dir.join(name))?;
        if sha256::hex(&sha256::digest(&bytes)) != *digest {
            changed.push(*name);
            if bytes.windows(2).any(|w| w == b"\r\n") {
                crlf.push(*name);
            }
        }
    }
    if !changed.is_empty() {
        // The digests are of the LF files in the repository; the bytes are
        // hashed as read, never normalised (CRY-003).
        let hint = if crlf.is_empty() {
            String::new()
        } else {
            format!(
                " (hint: CRLF line endings in {}; the manifest records the \
                 repository's LF bytes, so check `core.autocrlf` / `.gitattributes` \
                 and re-checkout before treating this as a schema change)",
                crlf.join(", ")
            )
        };
        return Err(format!(
            "tak-proto: schema content differs from {MANIFEST}: {}{hint}; {update}",
            changed.join(", ")
        )
        .into());
    }
    Ok(())
}

/// SHA-256 (FIPS 180-4) for the schema manifest check; see the module
/// documentation for why it is written here.
mod sha256 {
    use std::fmt::Write as _;

    use super::BuildResult;

    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    const H0: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    /// The SHA-256 digest of `data`.
    pub(super) fn digest(data: &[u8]) -> [u8; 32] {
        let mut state = H0;
        let (blocks, rest) = data.as_chunks::<64>();
        for block in blocks {
            compress(&mut state, block);
        }
        // Padding: 0x80, zeros, then the length in bits as a big-endian u64,
        // filling one or two final blocks.
        let mut tail = Vec::with_capacity(128);
        tail.extend_from_slice(rest);
        tail.push(0x80);
        let tail_len = if rest.len() < 56 { 64 } else { 128 };
        tail.resize(tail_len - 8, 0);
        let bits = u64::try_from(data.len())
            .unwrap_or(u64::MAX)
            .wrapping_mul(8);
        tail.extend_from_slice(&bits.to_be_bytes());
        for block in tail.as_chunks::<64>().0 {
            compress(&mut state, block);
        }

        let mut out = [0_u8; 32];
        for (bytes, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(state) {
            *bytes = word.to_be_bytes();
        }
        out
    }

    /// One 64-byte block. The message schedule is kept as a sliding window
    /// of 16 words: round t uses the window's first word (W[t]) and then
    /// appends W[t + 16].
    fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
        let mut window = [0_u32; 16];
        for (word, bytes) in window.iter_mut().zip(block.as_chunks::<4>().0) {
            *word = u32::from_be_bytes(*bytes);
        }
        // FIPS 180-4 working variables a to h.
        let [
            mut va,
            mut vb,
            mut vc,
            mut vd,
            mut ve,
            mut vf,
            mut vg,
            mut vh,
        ] = *state;
        for k in K {
            let [w0, w1, _, _, _, _, _, _, _, w9, _, _, _, _, w14, _] = window;
            let s1 = ve.rotate_right(6) ^ ve.rotate_right(11) ^ ve.rotate_right(25);
            let ch = (ve & vf) ^ (!ve & vg);
            let t1 = vh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k)
                .wrapping_add(w0);
            let s0 = va.rotate_right(2) ^ va.rotate_right(13) ^ va.rotate_right(22);
            let maj = (va & vb) ^ (va & vc) ^ (vb & vc);
            let t2 = s0.wrapping_add(maj);
            vh = vg;
            vg = vf;
            vf = ve;
            ve = vd.wrapping_add(t1);
            vd = vc;
            vc = vb;
            vb = va;
            va = t1.wrapping_add(t2);

            let sigma0 = w1.rotate_right(7) ^ w1.rotate_right(18) ^ (w1 >> 3);
            let sigma1 = w14.rotate_right(17) ^ w14.rotate_right(19) ^ (w14 >> 10);
            let next = sigma1
                .wrapping_add(w9)
                .wrapping_add(sigma0)
                .wrapping_add(w0);
            window.rotate_left(1);
            if let Some(last) = window.last_mut() {
                *last = next;
            }
        }
        for (s, v) in state.iter_mut().zip([va, vb, vc, vd, ve, vf, vg, vh]) {
            *s = s.wrapping_add(v);
        }
    }

    /// Lower-case hex of `bytes`.
    pub(super) fn hex(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len().saturating_mul(2));
        for b in bytes {
            let _ = write!(out, "{b:02x}");
        }
        out
    }

    /// Checks the implementation against FIPS 180-4 / NIST CSRC example
    /// vectors (empty, one block, two blocks of padding) before it is
    /// trusted; fails the build otherwise.
    pub(super) fn self_test() -> BuildResult<()> {
        let vectors: [(&[u8], &str); 4] = [
            (
                b"",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
            (
                b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
                "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
            ),
        ];
        for (input, expected) in vectors {
            if hex(&digest(input)) != expected {
                return Err("tak-proto: build.rs SHA-256 self-test failed".into());
            }
        }
        Ok(())
    }
}

/// Writes one line of the cargo build-script protocol to stdout (the channel
/// cargo reads; this is not diagnostic output).
fn emit(line: &str) -> io::Result<()> {
    writeln!(io::stdout().lock(), "{line}")
}
