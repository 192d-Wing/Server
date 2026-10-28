// SPDX-License-Identifier: Apache-2.0

//! In-scope namespace bindings for the parser and the serializer, and the
//! per-parse namespace-name interner.
//!
//! A flat binding stack plus a prefix map with an undo record per binding,
//! so a lookup costs one map probe however many elements and declarations
//! are open (an adversarial document cannot make prefix resolution
//! quadratic). Frames follow element nesting.
//!
//! Namespace names can be up to `max_attr_value_bytes` long. The parser
//! interns each one, so every element and attribute in that namespace
//! shares one allocation (no per-element copies) and equal names compare by
//! id, not by content.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SC-5 (Denial-of-service Protection)

use std::collections::BTreeMap;
use std::sync::Arc;

/// The namespace name the `xml` prefix is bound to by definition.
pub(crate) const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
/// The namespace name of `xmlns` declarations, which may never be bound.
pub(crate) const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";

/// A namespace name and its intern id (unique per parse; the serializer,
/// which does not intern, uses 0 and compares content).
#[derive(Clone)]
pub(crate) struct Ns {
    pub(crate) id: u32,
    pub(crate) uri: Arc<str>,
}

struct Binding {
    /// `""` for the default namespace.
    prefix: Box<str>,
    /// An empty name means "no namespace" (`xmlns=""`).
    ns: Ns,
    /// Index of the binding this one hides, restored when its frame closes.
    hidden: Option<usize>,
}

/// A stack of namespace frames.
#[derive(Default)]
pub(crate) struct NsScope {
    bindings: Vec<Binding>,
    current: BTreeMap<Box<str>, usize>,
    frames: Vec<usize>,
}

impl NsScope {
    /// Opens a frame for an element's declarations.
    pub(crate) fn push_frame(&mut self) {
        self.frames.push(self.bindings.len());
    }

    /// Closes the innermost frame, restoring the bindings it hid.
    pub(crate) fn pop_frame(&mut self) {
        let mark = self.frames.pop().unwrap_or(0);
        while self.bindings.len() > mark {
            let Some(b) = self.bindings.pop() else { break };
            match b.hidden {
                Some(i) => {
                    self.current.insert(b.prefix, i);
                }
                None => {
                    self.current.remove(&b.prefix);
                }
            }
        }
    }

    /// Binds `prefix` (`""` for the default namespace) in the innermost
    /// frame.
    pub(crate) fn bind(&mut self, prefix: &str, ns: Ns) {
        let index = self.bindings.len();
        let hidden = self.current.insert(prefix.into(), index);
        self.bindings.push(Binding {
            prefix: prefix.into(),
            ns,
            hidden,
        });
    }

    /// The namespace bound to `prefix` (`""` for the default namespace), or
    /// `None` when unbound. A default namespace undeclared with `xmlns=""`
    /// returns a binding with an empty name.
    pub(crate) fn lookup(&self, prefix: &str) -> Option<&Ns> {
        let index = *self.current.get(prefix)?;
        self.bindings.get(index).map(|b| &b.ns)
    }
}

/// Interns namespace names for one parse.
#[derive(Default)]
pub(crate) struct Interner {
    names: BTreeMap<Arc<str>, u32>,
}

impl Interner {
    /// The shared copy of `uri` and its id. Ids are dense from 1; the
    /// number of distinct names is bounded by the input size, so a `u32`
    /// cannot run out for any input below 4 GiB, and saturates beyond.
    pub(crate) fn intern(&mut self, uri: &str) -> Ns {
        if let Some((name, id)) = self.names.get_key_value(uri) {
            return Ns {
                id: *id,
                uri: Arc::clone(name),
            };
        }
        let id = u32::try_from(self.names.len())
            .unwrap_or(u32::MAX)
            .saturating_add(1);
        let name: Arc<str> = Arc::from(uri);
        self.names.insert(Arc::clone(&name), id);
        Ns { id, uri: name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ns(uri: &str) -> Ns {
        Ns {
            id: 0,
            uri: Arc::from(uri),
        }
    }

    fn uri<'a>(s: &'a NsScope, prefix: &str) -> Option<&'a str> {
        s.lookup(prefix).map(|n| &*n.uri)
    }

    #[test]
    fn frames_hide_and_restore() {
        let mut s = NsScope::default();
        s.push_frame();
        s.bind("p", ns("u1"));
        s.push_frame();
        s.bind("p", ns("u2"));
        s.bind("", ns("d"));
        assert_eq!(uri(&s, "p"), Some("u2"));
        assert_eq!(uri(&s, ""), Some("d"));
        s.pop_frame();
        assert_eq!(uri(&s, "p"), Some("u1"));
        assert_eq!(uri(&s, ""), None);
        s.pop_frame();
        assert_eq!(uri(&s, "p"), None);
        // Popping with no frame open is harmless.
        s.pop_frame();
    }

    #[test]
    fn interner_shares_equal_names() {
        let mut i = Interner::default();
        let a = i.intern("urn:a");
        let b = i.intern("urn:b");
        let a2 = i.intern("urn:a");
        assert_eq!(a.id, a2.id);
        assert_ne!(a.id, b.id);
        assert!(Arc::ptr_eq(&a.uri, &a2.uri));
    }
}
