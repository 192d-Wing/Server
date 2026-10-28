// SPDX-License-Identifier: Apache-2.0

//! Cursor-on-Target (`CoT`) XML events for TAK-RS.
//!
//! - [`xml`]: a bounded, non-recursive XML 1.0 parser into an arena document,
//!   and a serializer whose output matches the reference server's (no DTDs,
//!   no external entities, every size and depth limited by `Limits`);
//! - [`javafmt`]: the number and `CoT` time text forms the reference server
//!   reads and writes;
//! - the `CoT` event view on a parsed document and the conversion between `CoT`
//!   XML and TAK Protocol v1 (`tak_proto::v1::TakMessage`), including the
//!   split of `<detail>` into typed sub-messages plus `xmlDetail`.
//!
//! Every input is untrusted: parsing and conversion return typed errors and
//! never panic. Nothing here performs I/O, reads a clock, holds global state
//! or calls cryptography; the crate builds for `wasm32-unknown-unknown`.
//!
//! Phase 1a skeleton: the modules are filled in by work units WU3 (`xml`,
//! `limits`, XML errors), WU4 (`javafmt`) and WU5 (`event`, `convert`,
//! conversion errors).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)

#![forbid(unsafe_code)]
#![deny(clippy::arithmetic_side_effects)]

pub mod javafmt;
pub mod xml;

mod convert;
mod error;
mod event;
mod limits;
