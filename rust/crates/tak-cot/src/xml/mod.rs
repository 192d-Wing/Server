// SPDX-License-Identifier: Apache-2.0

//! Bounded XML 1.0 parsing into an arena document, and its serialization.
//!
//! Filled in by WU3 (plan section 1.5): quick-xml pull reader plus DOCTYPE
//! rejection, reference and character validation, normalisation, namespace
//! checks and the limits in `Limits`, with no recursion anywhere.

mod document;
mod names;
mod parse;
mod serialize;
