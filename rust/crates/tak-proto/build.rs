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
//! listed schema is missing, or when the directory holds a `.proto` file that
//! is not listed. An upstream schema addition is reviewed and added to
//! `SCHEMAS` by hand; it is never compiled silently.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: CM-3 (Configuration Change Control), SA-15 (Development
//   Process, Standards, and Tools)

use std::collections::BTreeSet;
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
    let mut unlisted = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "proto") {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !listed.contains(name.as_str()) {
                unlisted.push(name);
            }
        }
    }
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

/// Writes one line of the cargo build-script protocol to stdout (the channel
/// cargo reads; this is not diagnostic output).
fn emit(line: &str) -> io::Result<()> {
    writeln!(io::stdout().lock(), "{line}")
}
