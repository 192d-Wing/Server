// SPDX-License-Identifier: Apache-2.0

//! Shared Yew components for the TAK-RS web apps in `rust/web`.
//!
//! This crate is a Phase 0 stub. It exists so that CI exercises a real Yew
//! (`csr`) dependency graph for both the native and `wasm32-unknown-unknown`
//! targets. Real components (Department of Defense consent banner,
//! classification marking, forms) arrive with the first web app.
//!
//! Rules for every component in this crate:
//!
//! - all server and user data is rendered as text nodes through `html!`; no
//!   `Html::from_html_unchecked`, no `inner_html`, no raw HTML strings
//!   (`disallowed-methods` in `rust/web/clippy.toml` rejects the known raw-HTML,
//!   `eval` and browser-storage APIs);
//! - no inline scripts or styles, so the apps run under a strict CSP whose only
//!   relaxation is `'wasm-unsafe-eval'`;
//! - request and response types come from `tak-api-types`, never redefined
//!   here;
//! - the crate never depends on `tak-crypto` or any native-code crate.

#![forbid(unsafe_code)]

use yew::{Html, function_component, html};

/// Returns the human-readable build label shown by [`BuildInfo`].
///
/// The label is derived only from compile-time Cargo metadata, so it contains
/// no server or user data.
///
/// ```
/// assert!(tak_web_components::build_label().starts_with("tak-web-components "));
/// ```
#[must_use]
pub fn build_label() -> String {
    format!(
        "{} {} (API types: {})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        tak_api_types::CRATE_NAME,
    )
}

/// Displays the component library name and version.
///
/// The label is rendered as a single text node inside a semantic `<p>`
/// element and is never parsed as HTML; no markup is built from strings.
#[function_component(BuildInfo)]
pub fn build_info() -> Html {
    html! {
        <p class="tak-build-info">{ build_label() }</p>
    }
}

#[cfg(test)]
mod tests {
    use super::build_label;

    #[test]
    fn build_label_names_crate_version_and_api_types() {
        let label = build_label();
        assert_eq!(
            label,
            format!(
                "tak-web-components {} (API types: tak-api-types)",
                env!("CARGO_PKG_VERSION")
            )
        );
    }
}
