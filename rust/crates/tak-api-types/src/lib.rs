// SPDX-License-Identifier: Apache-2.0

//! Data-transfer types shared by the TAK-RS REST API (`tak-api`) and the Yew
//! web apps in `rust/web`.
//!
//! This crate is a Phase 0 stub. It stays empty until Phase 4, when the DTOs
//! for the first ported Marti REST operations are added.
//!
//! The crate must remain buildable for `wasm32-unknown-unknown`: no tokio, no
//! I/O, no native code, and it must never depend on `tak-crypto`.

#![forbid(unsafe_code)]

/// The name of this crate, as recorded by Cargo.
///
/// ```
/// assert_eq!(tak_api_types::CRATE_NAME, "tak-api-types");
/// ```
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
