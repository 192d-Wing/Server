// SPDX-License-Identifier: Apache-2.0

//! Guards `rust/web/clippy.toml` against drift from `rust/clippy.toml`.
//!
//! Clippy uses the nearest `clippy.toml` and does not merge files, so the web
//! file must repeat every setting of the workspace file. Both files keep one
//! entry per line, which lets this check compare lines.

const WORKSPACE: &str = include_str!("../../../clippy.toml");
const WEB: &str = include_str!("../../clippy.toml");

fn settings(config: &str) -> impl Iterator<Item = &str> {
    config
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

#[test]
fn web_config_repeats_every_workspace_setting() {
    let web: Vec<&str> = settings(WEB).collect();
    let missing: Vec<&str> = settings(WORKSPACE)
        .filter(|line| !web.contains(line))
        .collect();
    assert!(
        missing.is_empty(),
        "rust/web/clippy.toml is missing lines from rust/clippy.toml: {missing:#?}"
    );
}

#[test]
fn web_config_bans_wasm_panicking_std_calls() {
    for path in [
        "std::time::Instant::now",
        "std::time::SystemTime::now",
        "std::thread::spawn",
        "std::thread::Scope::spawn",
        "std::thread::sleep",
    ] {
        let entry = format!("{{ path = \"{path}\",");
        assert!(
            settings(WEB).any(|line| line.starts_with(&entry)),
            "rust/web/clippy.toml does not ban {path}"
        );
        assert!(
            !settings(WORKSPACE).any(|line| line.starts_with(&entry)),
            "{path} must not be banned workspace-wide (native crates use it)"
        );
    }
}
