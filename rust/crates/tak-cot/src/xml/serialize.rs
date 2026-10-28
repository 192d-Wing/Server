// SPDX-License-Identifier: Apache-2.0

//! Document and element-fragment serialization.
//!
//! One writer, two entry points: the whole document (the reference's
//! `asXML()` of a document, X2X and P2X output) and one element with its
//! subtree (each `xmlDetail` element). The walk follows the arena's sibling
//! and parent links, so no input depth can exhaust the stack.
//!
//! The output rules below are XML 1.0 defaults chosen so every output
//! re-parses to the same document. Each one is a parity point fixed later
//! by the X2X and P2X goldens (parity-map C-01 to C-05, C-11):
//!
//! - declaration `<?xml version="1.0" encoding="UTF-8"?>` then a line feed
//!   (C-01), whatever declaration the input had;
//! - comments and processing instructions outside the root are written in
//!   order with no separator; white space outside the root is not kept
//!   (C-05);
//! - text: `&`, `<`, `>` escaped as `&amp;`, `&lt;`, `&gt;`, and a carriage
//!   return (only possible from `&#13;`) as `&#13;` so it survives
//!   re-parsing (C-02);
//! - attribute values in double quotes: `&`, `<`, `>`, `"` escaped, and tab,
//!   line feed and carriage return as `&#9;`, `&#10;`, `&#13;` so value
//!   normalisation does not turn them into spaces on re-parse; `'` is not
//!   escaped (C-02);
//! - an element with no child node is written `<a/>`; one with any child,
//!   even an empty text node, as `<a>...</a>` (C-03);
//! - CDATA sections, comments and processing instructions are kept as such
//!   (C-05); non-ASCII characters are written as UTF-8, not as references;
//! - per element: its own namespace declarations in document order, then a
//!   declaration for its prefix (or default namespace) and for each
//!   attribute prefix when the output so far does not bind it to the right
//!   name, then attributes in document order (C-04, C-11).
//
// Output size: the whole-document and unbounded fragment writers are not
// capped. A document's output stays close to its parsed size, but a
// fragment re-declares at its root every namespace it uses from ancestors,
// so many fragments of one document can be far larger than the document;
// `write_subtree_bounded` is the capped form used for `xmlDetail`.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation; supporting role: the
//   output is well-formed and every value is escaped, so nothing a sender
//   wrote can break out of its element at a receiver)
// POAM: POAM-2026-10-27-03 - unbounded output amplification of
// write_document and write_subtree; Phase 3 bounds what a connection writes
// per event.
// PARITY: src/takserver-plugins/src/main/java/tak/server/cot/XmlContainer.java#asXml

use std::sync::Arc;

use super::document::{Document, ElementData, Node, NodeData, link};
use super::scope::{Ns, NsScope};
use crate::error::{LimitKind, XmlError, XmlErrorKind};

const DECLARATION: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

/// Writes the whole document: declaration, then the document node's
/// children.
pub(crate) fn write_document(doc: &Document, out: &mut Vec<u8>) {
    out.extend_from_slice(DECLARATION);
    let mut writer = Writer::new(doc, out, usize::MAX);
    let mut child = doc.get(0).and_then(|n| link(n.first_child));
    while let Some(id) = child {
        // Unbounded: cannot fail.
        let _ = writer.subtree(id);
        child = doc.get(id).and_then(|n| link(n.next_sibling));
    }
}

/// Writes node `id` and its subtree.
pub(crate) fn write_subtree(doc: &Document, id: u32, out: &mut Vec<u8>) {
    // Unbounded: cannot fail.
    let _ = Writer::new(doc, out, usize::MAX).subtree(id);
}

/// Writes node `id` and its subtree, appending at most `max` bytes; on
/// overflow `out` is restored to its length on entry.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SC-5 (Denial-of-service Protection)
pub(crate) fn write_subtree_bounded(
    doc: &Document,
    id: u32,
    out: &mut Vec<u8>,
    max: usize,
) -> Result<(), XmlError> {
    let start = out.len();
    let result = Writer::new(doc, out, max).subtree(id);
    if result.is_err() {
        out.truncate(start);
    }
    result
        .map_err(|Overflow| XmlError::edit(XmlErrorKind::LimitExceeded(LimitKind::XmlDetailBytes)))
}

/// The output passed the writer's byte budget.
struct Overflow;

struct Writer<'d, 'o> {
    doc: &'d Document,
    out: &'o mut Vec<u8>,
    /// Output length above which writing stops.
    end: usize,
    scope: NsScope,
}

impl<'d, 'o> Writer<'d, 'o> {
    fn new(doc: &'d Document, out: &'o mut Vec<u8>, max: usize) -> Self {
        let end = out.len().saturating_add(max);
        Self {
            doc,
            out,
            end,
            scope: NsScope::default(),
        }
    }

    fn check(&self) -> Result<(), Overflow> {
        if self.out.len() > self.end {
            Err(Overflow)
        } else {
            Ok(())
        }
    }

    /// Pre-order walk from `start`: start tags on the way down, end tags on
    /// the way up, never past `start`.
    fn subtree(&mut self, start: u32) -> Result<(), Overflow> {
        let mut cur = start;
        loop {
            let Some(node) = self.doc.get(cur) else {
                return Ok(());
            };
            let open = self.enter(node);
            self.check()?;
            if open && let Some(child) = link(node.first_child) {
                cur = child;
                continue;
            }
            loop {
                if cur == start {
                    return Ok(());
                }
                let Some(node) = self.doc.get(cur) else {
                    return Ok(());
                };
                if let Some(next) = link(node.next_sibling) {
                    cur = next;
                    break;
                }
                cur = node.parent;
                if let Some(NodeData::Element(e)) = self.doc.get(cur).map(|n| &n.data) {
                    self.end_tag(e);
                }
                self.check()?;
            }
        }
    }

    /// Writes a node, or an element's start tag. Returns whether an element
    /// was left open (it has children).
    fn enter(&mut self, node: &Node) -> bool {
        match &node.data {
            NodeData::Document => false,
            NodeData::Element(e) => {
                let has_children = link(node.first_child).is_some();
                self.start_tag(e, has_children);
                has_children
            }
            NodeData::Text(t) => {
                escape(self.out, t, false);
                false
            }
            NodeData::CData(t) => {
                self.out.extend_from_slice(b"<![CDATA[");
                self.out.extend_from_slice(t.as_bytes());
                self.out.extend_from_slice(b"]]>");
                false
            }
            NodeData::Comment(t) => {
                self.out.extend_from_slice(b"<!--");
                self.out.extend_from_slice(t.as_bytes());
                self.out.extend_from_slice(b"-->");
                false
            }
            NodeData::Pi(pi) => {
                self.out.extend_from_slice(b"<?");
                self.out.extend_from_slice(pi.target.as_bytes());
                if !pi.data.is_empty() {
                    self.out.push(b' ');
                    self.out.extend_from_slice(pi.data.as_bytes());
                }
                self.out.extend_from_slice(b"?>");
                false
            }
        }
    }

    fn start_tag(&mut self, e: &ElementData, open: bool) {
        self.scope.push_frame();
        self.out.push(b'<');
        self.out.extend_from_slice(e.name.as_bytes());
        for decl in &e.namespace_decls {
            self.declare(decl.prefix.as_deref().unwrap_or(""), &decl.uri);
        }
        let prefix = e.name.split_once(':').map_or("", |(p, _)| p);
        self.ensure_bound(prefix, e.namespace.as_ref());
        for attr in &e.attributes {
            if let Some((p, _)) = attr.name.split_once(':') {
                self.ensure_bound(p, attr.namespace.as_ref());
            }
        }
        for attr in &e.attributes {
            self.out.push(b' ');
            self.out.extend_from_slice(attr.name.as_bytes());
            self.out.extend_from_slice(b"=\"");
            escape(self.out, &attr.value, true);
            self.out.push(b'"');
        }
        if open {
            self.out.push(b'>');
        } else {
            self.out.extend_from_slice(b"/>");
            self.scope.pop_frame();
        }
    }

    fn end_tag(&mut self, e: &ElementData) {
        self.out.extend_from_slice(b"</");
        self.out.extend_from_slice(e.name.as_bytes());
        self.out.push(b'>');
        self.scope.pop_frame();
    }

    /// Declares `prefix` (`""` for the default namespace) unless the output
    /// already binds it to `ns` (`None`: no namespace).
    fn ensure_bound(&mut self, prefix: &str, ns: Option<&Arc<str>>) {
        if prefix == "xml" {
            return;
        }
        let wanted = ns.map_or("", |u| &**u);
        let bound = self.scope.lookup(prefix);
        let same = match (bound, ns) {
            (Some(b), Some(u)) => Arc::ptr_eq(&b.uri, u) || *b.uri == **u,
            (Some(b), None) => b.uri.is_empty(),
            (None, _) => wanted.is_empty() && prefix.is_empty(),
        };
        if !same {
            let uri = ns.map_or_else(|| Arc::from(""), Arc::clone);
            self.declare(prefix, &uri);
        }
    }

    fn declare(&mut self, prefix: &str, uri: &Arc<str>) {
        if prefix.is_empty() {
            self.out.extend_from_slice(b" xmlns=\"");
        } else {
            self.out.extend_from_slice(b" xmlns:");
            self.out.extend_from_slice(prefix.as_bytes());
            self.out.extend_from_slice(b"=\"");
        }
        escape(self.out, uri, true);
        self.out.push(b'"');
        self.scope.bind(
            prefix,
            Ns {
                id: 0,
                uri: Arc::clone(uri),
            },
        );
    }
}

/// Appends `s` with the escapes of the module rules: text when
/// `attribute` is false, a double-quoted attribute value when true.
fn escape(out: &mut Vec<u8>, s: &str, attribute: bool) {
    let bytes = s.as_bytes();
    let mut run_start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let replacement: &[u8] = match b {
            b'&' => b"&amp;",
            b'<' => b"&lt;",
            b'>' => b"&gt;",
            b'\r' => b"&#13;",
            b'"' if attribute => b"&quot;",
            b'\t' if attribute => b"&#9;",
            b'\n' if attribute => b"&#10;",
            _ => continue,
        };
        out.extend_from_slice(bytes.get(run_start..i).unwrap_or_default());
        out.extend_from_slice(replacement);
        run_start = i.saturating_add(1);
    }
    out.extend_from_slice(bytes.get(run_start..).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn esc(s: &str, attribute: bool) -> String {
        let mut out = Vec::new();
        escape(&mut out, s, attribute);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn escapes() {
        assert_eq!(
            esc("a<b>&c\"'\t\n\r", false),
            "a&lt;b&gt;&amp;c\"'\t\n&#13;"
        );
        assert_eq!(
            esc("a<b>&c\"'\t\n\r", true),
            "a&lt;b&gt;&amp;c&quot;'&#9;&#10;&#13;"
        );
        assert_eq!(esc("\u{e9}\u{1F600}", true), "\u{e9}\u{1F600}");
        assert_eq!(esc("", true), "");
    }
}
