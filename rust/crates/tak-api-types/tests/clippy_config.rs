// SPDX-License-Identifier: Apache-2.0

//! Guards `crates/tak-api-types/clippy.toml` against drift from
//! `rust/web/clippy.toml`.
//!
//! This crate builds for wasm32-unknown-unknown, and clippy uses the nearest
//! `clippy.toml` without merging, so the crate carries its own copy of the
//! web configuration (WEB-003). Both files keep one entry per line, which lets
//! this check compare setting lines; comments and blank lines are ignored.

const API_TYPES: &str = include_str!("../clippy.toml");
const WEB: &str = include_str!("../../../web/clippy.toml");

fn settings(config: &str) -> Vec<&str> {
    config
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

#[test]
fn config_matches_web_config() {
    assert_eq!(
        settings(API_TYPES),
        settings(WEB),
        "crates/tak-api-types/clippy.toml must have the same settings, in the same order, as rust/web/clippy.toml"
    );
}

#[test]
fn config_bans_wasm_panicking_std_calls() {
    let api_types = settings(API_TYPES);
    for path in [
        "std::time::Instant::now",
        "std::time::SystemTime::now",
        "std::thread::spawn",
        "std::thread::Scope::spawn",
        "std::thread::sleep",
    ] {
        let entry = format!("{{ path = \"{path}\",");
        assert!(
            api_types.iter().any(|line| line.starts_with(&entry)),
            "crates/tak-api-types/clippy.toml does not ban {path}"
        );
    }
}
