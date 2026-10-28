// SPDX-License-Identifier: Apache-2.0

//! Bounded XML 1.0 parsing into an arena document, and its serialization.
//!
//! [`Document::parse`] reads one document with a quick-xml pull reader and
//! adds what the reference's namespace-aware, DOCTYPE-refusing parser
//! enforces and quick-xml does not: DOCTYPE rejection, reference and
//! character validation, name checks, line-end and attribute-value
//! normalisation, namespace resolution, and every bound in
//! [`Limits`](crate::Limits). The result is an arena of nodes linked by
//! index ([`Document`], [`ElementRef`], [`NodeKind`]): parsing, walking,
//! serializing, cloning and dropping never recurse, so no input depth can
//! exhaust the stack.
//!
//! [`Document::to_xml_bytes`] and [`ElementRef::to_xml_fragment`] write the
//! document or one element back out (plan section 1.5).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SI-10(3) (Predictable
//   Behavior), SC-5 (Denial-of-service Protection)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002550 / V-222608
//   (unverified)

mod document;
mod names;
mod parse;
mod scope;
mod serialize;

#[cfg(test)]
pub(crate) use document::child_steps;
pub use document::{
    AttributeRef, Attributes, Children, Document, ElementRef, NamespaceDecl, NamespaceDecls,
    NodeId, NodeKind, XmlDeclaration,
};
