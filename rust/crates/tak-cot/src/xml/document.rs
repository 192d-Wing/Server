// SPDX-License-Identifier: Apache-2.0

//! The arena document model: nodes in one `Vec` linked by `u32` indices, so
//! building, walking, cloning and dropping never recurse.
//!
//! Index 0 is the document node; its children are the root element and any
//! comments and processing instructions around it. Every other node is an
//! element, a text node, a CDATA section, a comment or a processing
//! instruction. Read access goes through [`ElementRef`] and [`NodeKind`];
//! the few edits the `CoT` conversion needs (build an event, append copies of
//! `xmlDetail` elements, detach an element) are methods on [`Document`] and
//! enforce the same [`Limits`] as parsing.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)
// - ASD STIG: APSC-DV-002530 / V-222606 (unverified)

use core::fmt;
use std::sync::Arc;

use super::names::{first_invalid_char, is_ncname};
use crate::error::{LimitKind, XmlError, XmlErrorKind};
use crate::limits::Limits;

/// "No node" in a link field.
const NIL: u32 = u32::MAX;
/// Index of the document node.
const DOC: u32 = 0;

/// Identifies a node of one [`Document`]. Only meaningful for the document
/// it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) data: NodeData,
    pub(crate) parent: u32,
    pub(crate) first_child: u32,
    pub(crate) last_child: u32,
    pub(crate) prev_sibling: u32,
    pub(crate) next_sibling: u32,
}

#[derive(Clone)]
pub(crate) enum NodeData {
    Document,
    Element(Box<ElementData>),
    Text(Box<str>),
    CData(Box<str>),
    Comment(Box<str>),
    Pi(Box<PiData>),
}

#[derive(Clone)]
pub(crate) struct ElementData {
    /// Qualified name as written (`p:local` or `local`).
    pub(crate) name: Box<str>,
    /// Namespace name the element's prefix (or the default namespace)
    /// resolved to; `None` for no namespace. Shared with every other node
    /// in the same namespace.
    pub(crate) namespace: Option<Arc<str>>,
    /// Attributes in document order, without namespace declarations.
    pub(crate) attributes: Vec<Attr>,
    /// `xmlns` / `xmlns:p` declarations in document order.
    pub(crate) namespace_decls: Vec<NsDecl>,
}

#[derive(Clone)]
pub(crate) struct Attr {
    pub(crate) name: Box<str>,
    pub(crate) namespace: Option<Arc<str>>,
    pub(crate) value: Box<str>,
}

#[derive(Clone)]
pub(crate) struct NsDecl {
    /// `None` for the default namespace (`xmlns="..."`).
    pub(crate) prefix: Option<Box<str>>,
    /// Empty for `xmlns=""`.
    pub(crate) uri: Arc<str>,
}

#[derive(Clone)]
pub(crate) struct PiData {
    pub(crate) target: Box<str>,
    pub(crate) data: Box<str>,
}

/// The XML declaration of a parsed document. Only version 1.0 is accepted
/// (DEV-01b).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XmlDeclaration {
    pub(crate) encoding: Option<Box<str>>,
    pub(crate) standalone: Option<bool>,
}

impl XmlDeclaration {
    /// The `encoding` pseudo-attribute as written, if present. The parser
    /// reads `&str` input, so it does not act on it.
    #[must_use]
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }

    /// The `standalone` pseudo-attribute, if present.
    #[must_use]
    pub const fn standalone(&self) -> Option<bool> {
        self.standalone
    }
}

/// A parsed (or built) XML document.
///
/// Built only by [`Document::parse`] or [`Document::with_root`], so it
/// always has exactly one root element. Cloning and dropping are iterative.
#[derive(Clone)]
pub struct Document {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: u32,
    pub(crate) declaration: Option<XmlDeclaration>,
    pub(crate) limits: Limits,
}

impl fmt::Debug for Document {
    /// Summarises the document without its content: `CoT` events carry
    /// positions and identities that do not belong in logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Document")
            .field("nodes", &self.node_count())
            .finish_non_exhaustive()
    }
}

/// One node, as seen through [`ElementRef::children`] or
/// [`Document::node`].
#[derive(Clone, Copy, Debug)]
pub enum NodeKind<'a> {
    /// An element.
    Element(ElementRef<'a>),
    /// Character data, with adjacent text and references merged, line ends
    /// normalised and references resolved.
    Text(&'a str),
    /// The content of a CDATA section (line ends normalised).
    CData(&'a str),
    /// The content of a comment (line ends normalised).
    Comment(&'a str),
    /// A processing instruction; `data` excludes the white space after the
    /// target.
    ProcessingInstruction {
        /// The target name.
        target: &'a str,
        /// The data, possibly empty.
        data: &'a str,
    },
}

/// A borrowed view of one element.
#[derive(Clone, Copy)]
pub struct ElementRef<'a> {
    doc: &'a Document,
    id: u32,
}

impl fmt::Debug for ElementRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementRef")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// A borrowed view of one attribute (namespace declarations excluded).
#[derive(Clone, Copy)]
pub struct AttributeRef<'a> {
    attr: &'a Attr,
}

impl fmt::Debug for AttributeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No name either: attribute names in `CoT` details can carry data.
        f.debug_struct("AttributeRef").finish_non_exhaustive()
    }
}

/// A borrowed view of one namespace declaration.
#[derive(Clone, Copy, Debug)]
pub struct NamespaceDecl<'a> {
    decl: &'a NsDecl,
}

// ---- crate-internal arena operations ----

impl Document {
    /// A document holding only the document node.
    pub(crate) fn empty(limits: &Limits) -> Self {
        Self {
            nodes: vec![Node::new(NodeData::Document)],
            root: NIL,
            declaration: None,
            limits: *limits,
        }
    }

    pub(crate) fn get(&self, id: u32) -> Option<&Node> {
        self.nodes.get(usize::try_from(id).ok()?)
    }

    fn get_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.get_mut(usize::try_from(id).ok()?)
    }

    pub(crate) fn element_data(&self, id: u32) -> Option<&ElementData> {
        match &self.get(id)?.data {
            NodeData::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Adds `data` as the last child of `parent`, after checking
    /// `max_nodes`.
    pub(crate) fn append(&mut self, parent: u32, data: NodeData) -> Result<u32, XmlErrorKind> {
        let limit = usize::try_from(self.limits.max_nodes).unwrap_or(usize::MAX);
        if self.node_count() >= limit {
            return Err(XmlErrorKind::LimitExceeded(LimitKind::Nodes));
        }
        let id = match u32::try_from(self.nodes.len()) {
            Ok(id) if id != NIL => id,
            _ => return Err(XmlErrorKind::LimitExceeded(LimitKind::Nodes)),
        };
        let last = self
            .get(parent)
            .ok_or(XmlErrorKind::InvalidNode)?
            .last_child;
        let mut node = Node::new(data);
        node.parent = parent;
        node.prev_sibling = last;
        self.nodes.push(node);
        if let Some(prev) = self.get_mut(last) {
            prev.next_sibling = id;
        }
        if let Some(p) = self.get_mut(parent) {
            if p.first_child == NIL {
                p.first_child = id;
            }
            p.last_child = id;
        }
        Ok(id)
    }

    /// Nodes in the arena, the document node excluded (detached nodes
    /// included).
    fn node_count(&self) -> usize {
        self.nodes.len().saturating_sub(1)
    }

    /// Element depth of `id` (the root element is 1, the document node 0).
    fn depth(&self, id: u32) -> usize {
        let mut depth = 0_usize;
        let mut cur = id;
        // Bounded by the arena size even if links were corrupt.
        for _ in 0..self.nodes.len() {
            match self.get(cur) {
                Some(n) if cur != DOC => {
                    depth = depth.saturating_add(1);
                    cur = n.parent;
                }
                _ => break,
            }
        }
        depth
    }

    fn max_depth(&self) -> usize {
        usize::from(self.limits.max_depth)
    }

    fn check_text(&self, text: &str) -> Result<(), XmlError> {
        if first_invalid_char(text).is_some() {
            return Err(XmlError::edit(XmlErrorKind::InvalidChar));
        }
        let max = usize::try_from(self.limits.max_text_bytes).unwrap_or(usize::MAX);
        if text.len() > max {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::TextBytes,
            )));
        }
        Ok(())
    }

    fn check_new_name(&self, name: &str) -> Result<(), XmlError> {
        if name.len() > usize::from(self.limits.max_name_bytes) {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::NameBytes,
            )));
        }
        if !is_ncname(name) {
            return Err(XmlError::edit(XmlErrorKind::InvalidName));
        }
        Ok(())
    }
}

impl Node {
    const fn new(data: NodeData) -> Self {
        Self {
            data,
            parent: NIL,
            first_child: NIL,
            last_child: NIL,
            prev_sibling: NIL,
            next_sibling: NIL,
        }
    }
}

pub(crate) const fn link(id: u32) -> Option<u32> {
    if id == NIL { None } else { Some(id) }
}

// ---- public read API ----

impl Document {
    /// Parses one XML 1.0 document under `limits`.
    ///
    /// Namespace-aware and non-validating. Refused: any document type
    /// declaration, entities other than the five predefined ones,
    /// characters outside XML 1.0 `Char`, invalid names, undeclared
    /// prefixes, duplicate attributes, an XML declaration other than
    /// version 1.0 at the very start, and anything beyond `limits`. Line
    /// ends and attribute values are normalised; adjacent text and
    /// references merge into one text node; white space outside the root
    /// element is dropped. Nothing is returned on error, and no input can
    /// cause a panic or unbounded recursion.
    ///
    /// # Errors
    ///
    /// An [`XmlError`] naming the first problem found and its byte offset.
    pub fn parse(input: &str, limits: &Limits) -> Result<Self, XmlError> {
        super::parse::parse(input, limits)
    }

    /// The whole document in the serializer's form (the reference's
    /// `asXML()` equivalent; the exact rules are listed in the `serialize`
    /// module and fixed by the X2X goldens).
    #[must_use]
    pub fn to_xml_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        super::serialize::write_document(self, &mut out);
        out
    }

    /// The root element.
    #[must_use]
    pub fn root(&self) -> ElementRef<'_> {
        ElementRef {
            doc: self,
            id: self.root,
        }
    }

    /// The XML declaration, when the input had one.
    #[must_use]
    pub const fn declaration(&self) -> Option<&XmlDeclaration> {
        self.declaration.as_ref()
    }

    /// The limits this document was parsed or built with; edits are held to
    /// them too.
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }

    /// The children of the document node in order: comments and processing
    /// instructions before and after the root element, and the root itself.
    #[must_use]
    pub fn top_level(&self) -> Children<'_> {
        Children::of(self, DOC)
    }

    /// The node with this id, or `None` for the document node or an id from
    /// another document that is out of range.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<NodeKind<'_>> {
        node_kind(self, id.0)
    }

    /// The element with this id, or `None` when `id` is not an element.
    #[must_use]
    pub fn element(&self, id: NodeId) -> Option<ElementRef<'_>> {
        self.element_data(id.0).map(|_| ElementRef {
            doc: self,
            id: id.0,
        })
    }
}

fn node_kind(doc: &Document, id: u32) -> Option<NodeKind<'_>> {
    Some(match &doc.get(id)?.data {
        NodeData::Document => return None,
        NodeData::Element(_) => NodeKind::Element(ElementRef { doc, id }),
        NodeData::Text(t) => NodeKind::Text(t),
        NodeData::CData(t) => NodeKind::CData(t),
        NodeData::Comment(t) => NodeKind::Comment(t),
        NodeData::Pi(pi) => NodeKind::ProcessingInstruction {
            target: &pi.target,
            data: &pi.data,
        },
    })
}

impl<'a> ElementRef<'a> {
    fn data(&self) -> Option<&'a ElementData> {
        self.doc.element_data(self.id)
    }

    /// This element's id.
    #[must_use]
    pub const fn id(&self) -> NodeId {
        NodeId(self.id)
    }

    /// The qualified name as written (`p:local` or `local`).
    #[must_use]
    pub fn name(&self) -> &'a str {
        self.data().map_or("", |d| &d.name)
    }

    /// The local part of the name.
    #[must_use]
    pub fn local_name(&self) -> &'a str {
        local_part(self.name())
    }

    /// The prefix, if the name has one.
    #[must_use]
    pub fn prefix(&self) -> Option<&'a str> {
        self.name().split_once(':').map(|(p, _)| p)
    }

    /// The namespace name the element is in, if any.
    #[must_use]
    pub fn namespace_uri(&self) -> Option<&'a str> {
        self.data().and_then(|d| d.namespace.as_deref())
    }

    /// The attributes in document order, namespace declarations excluded.
    #[must_use]
    pub fn attributes(&self) -> Attributes<'a> {
        Attributes {
            inner: self.data().map_or(&[][..], |d| &d.attributes).iter(),
        }
    }

    /// The value of the first attribute whose local name is `local_name`,
    /// whatever its prefix.
    ///
    /// With `p:uid="A" uid="B"` on one element this returns `A`; the
    /// unprefixed attribute is not preferred. `CotEvent`'s identity
    /// accessors and the conversion rely on this one rule (parity-map C-06,
    /// D-18).
    #[must_use]
    pub fn attribute(&self, local_name: &str) -> Option<&'a str> {
        self.attributes()
            .find(|a| a.local_name() == local_name)
            .map(|a| a.value())
    }

    /// The namespace declarations written on this element, in document
    /// order.
    #[must_use]
    pub fn namespace_decls(&self) -> NamespaceDecls<'a> {
        NamespaceDecls {
            inner: self.data().map_or(&[][..], |d| &d.namespace_decls).iter(),
        }
    }

    /// The child nodes in document order.
    #[must_use]
    pub fn children(&self) -> Children<'a> {
        Children::of(self.doc, self.id)
    }

    /// The child elements in document order.
    pub fn child_elements(&self) -> impl Iterator<Item = ElementRef<'a>> + 'a {
        self.children().filter_map(|n| match n {
            NodeKind::Element(e) => Some(e),
            _ => None,
        })
    }

    /// The first child element whose local name is `local_name`, whatever
    /// its prefix.
    #[must_use]
    pub fn first_child_element(&self, local_name: &str) -> Option<ElementRef<'a>> {
        self.child_elements().find(|e| e.local_name() == local_name)
    }

    /// Whether the element has any child node.
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.doc.get(self.id).is_some_and(|n| n.first_child != NIL)
    }

    /// The parent element, or `None` for the root element (or a detached
    /// element).
    #[must_use]
    pub fn parent(&self) -> Option<ElementRef<'a>> {
        let parent = self.doc.get(self.id)?.parent;
        self.doc.element_data(parent).map(|_| ElementRef {
            doc: self.doc,
            id: parent,
        })
    }

    /// Appends this element and its subtree to `out` in the serializer's
    /// form, declaring every namespace it uses on the way, so the fragment
    /// is well-formed on its own (the form of each `xmlDetail` element).
    pub fn to_xml_fragment(&self, out: &mut Vec<u8>) {
        super::serialize::write_subtree(self.doc, self.id, out);
    }

    /// As [`to_xml_fragment`](Self::to_xml_fragment), appending at most
    /// `max_bytes` (the overshoot before the check is at most one node's
    /// output). A fragment re-declares every namespace it uses that is
    /// declared on an ancestor, so its size is not bounded by the input's:
    /// use this form, with `Limits::max_xml_detail_bytes`, when building
    /// `xmlDetail`.
    ///
    /// # Errors
    ///
    /// `XmlErrorKind::LimitExceeded(LimitKind::XmlDetailBytes)` when the
    /// fragment is longer; `out` is then left as it was.
    pub fn try_to_xml_fragment(&self, out: &mut Vec<u8>, max_bytes: usize) -> Result<(), XmlError> {
        super::serialize::write_subtree_bounded(self.doc, self.id, out, max_bytes)
    }
}

fn local_part(name: &str) -> &str {
    name.split_once(':').map_or(name, |(_, l)| l)
}

impl<'a> AttributeRef<'a> {
    /// The qualified name as written.
    #[must_use]
    pub fn name(&self) -> &'a str {
        &self.attr.name
    }

    /// The local part of the name.
    #[must_use]
    pub fn local_name(&self) -> &'a str {
        local_part(&self.attr.name)
    }

    /// The prefix, if the name has one.
    #[must_use]
    pub fn prefix(&self) -> Option<&'a str> {
        self.attr.name.split_once(':').map(|(p, _)| p)
    }

    /// The namespace name of a prefixed attribute; unprefixed attributes
    /// are in no namespace.
    #[must_use]
    pub fn namespace_uri(&self) -> Option<&'a str> {
        self.attr.namespace.as_deref()
    }

    /// The normalised value (XML 1.0 section 3.3.3, references resolved).
    #[must_use]
    pub fn value(&self) -> &'a str {
        &self.attr.value
    }
}

impl<'a> NamespaceDecl<'a> {
    /// The declared prefix, or `None` for the default namespace.
    #[must_use]
    pub fn prefix(&self) -> Option<&'a str> {
        self.decl.prefix.as_deref()
    }

    /// The namespace name; empty for `xmlns=""`.
    #[must_use]
    pub fn uri(&self) -> &'a str {
        &self.decl.uri
    }
}

/// Iterator over an element's attributes.
#[derive(Clone, Debug)]
pub struct Attributes<'a> {
    inner: core::slice::Iter<'a, Attr>,
}

impl<'a> Iterator for Attributes<'a> {
    type Item = AttributeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|attr| AttributeRef { attr })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Attributes<'_> {}

/// Iterator over an element's namespace declarations.
#[derive(Clone, Debug)]
pub struct NamespaceDecls<'a> {
    inner: core::slice::Iter<'a, NsDecl>,
}

impl<'a> Iterator for NamespaceDecls<'a> {
    type Item = NamespaceDecl<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|decl| NamespaceDecl { decl })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for NamespaceDecls<'_> {}

/// Iterator over child nodes.
#[derive(Clone, Debug)]
pub struct Children<'a> {
    doc: &'a Document,
    next: u32,
}

impl<'a> Children<'a> {
    fn of(doc: &'a Document, parent: u32) -> Self {
        Self {
            doc,
            next: doc.get(parent).map_or(NIL, |n| n.first_child),
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = link(self.next)?;
        #[cfg(test)]
        child_steps::bump();
        self.next = self.doc.get(id).map_or(NIL, |n| n.next_sibling);
        node_kind(self.doc, id)
    }
}

impl fmt::Debug for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attr").finish_non_exhaustive()
    }
}

impl fmt::Debug for NsDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NsDecl").finish_non_exhaustive()
    }
}

// ---- public edit API (used by the proto -> XML conversion) ----

impl Document {
    /// A new document whose root element is `name`, with no attributes and
    /// no namespace, held to `limits` for every later edit.
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidName`] unless `name` is an `NCName` (no
    /// prefix); the depth, name-length or node limit.
    pub fn with_root(name: &str, limits: &Limits) -> Result<Self, XmlError> {
        let mut doc = Self::empty(limits);
        if doc.max_depth() == 0 {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::Depth,
            )));
        }
        doc.root = doc.append_new_element(DOC, name)?;
        Ok(doc)
    }

    /// Appends a new, empty element `name` (no prefix, no namespace) as the
    /// last child of the element `parent`.
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidNode`] when `parent` is not an element;
    /// [`XmlErrorKind::InvalidName`] unless `name` is an `NCName`; the depth,
    /// node or name-length limit.
    pub fn append_element(&mut self, parent: NodeId, name: &str) -> Result<NodeId, XmlError> {
        if self.element_data(parent.0).is_none() {
            return Err(XmlError::edit(XmlErrorKind::InvalidNode));
        }
        if self.depth(parent.0) >= self.max_depth() {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::Depth,
            )));
        }
        self.append_new_element(parent.0, name).map(NodeId)
    }

    fn append_new_element(&mut self, parent: u32, name: &str) -> Result<u32, XmlError> {
        self.check_new_name(name)?;
        let data = ElementData {
            name: name.into(),
            namespace: None,
            attributes: Vec::new(),
            namespace_decls: Vec::new(),
        };
        self.append(parent, NodeData::Element(Box::new(data)))
            .map_err(XmlError::edit)
    }

    /// Appends attribute `name="value"` (no prefix) to the element
    /// `element`, after its existing attributes.
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidNode`] when `element` is not an element;
    /// [`XmlErrorKind::InvalidName`] unless `name` is an `NCName`;
    /// [`XmlErrorKind::DuplicateAttribute`] when the element already has an
    /// unprefixed attribute `name`; [`XmlErrorKind::InvalidChar`] when
    /// `value` holds a character XML 1.0 cannot carry; the attribute-count,
    /// name-length or value-length limit.
    pub fn push_attribute(
        &mut self,
        element: NodeId,
        name: &str,
        value: &str,
    ) -> Result<(), XmlError> {
        self.check_new_name(name)?;
        if first_invalid_char(value).is_some() {
            return Err(XmlError::edit(XmlErrorKind::InvalidChar));
        }
        let max_value = usize::try_from(self.limits.max_attr_value_bytes).unwrap_or(usize::MAX);
        if value.len() > max_value {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::AttrValueBytes,
            )));
        }
        let max_attrs = usize::from(self.limits.max_attributes);
        let Some(Node {
            data: NodeData::Element(el),
            ..
        }) = self.get_mut(element.0)
        else {
            return Err(XmlError::edit(XmlErrorKind::InvalidNode));
        };
        if el.attributes.iter().any(|a| &*a.name == name) {
            return Err(XmlError::edit(XmlErrorKind::DuplicateAttribute));
        }
        if el.attributes.len() >= max_attrs {
            return Err(XmlError::edit(XmlErrorKind::LimitExceeded(
                LimitKind::Attributes,
            )));
        }
        el.attributes.push(Attr {
            name: name.into(),
            namespace: None,
            value: value.into(),
        });
        Ok(())
    }

    /// Appends a text node as the last child of the element `parent`. An
    /// empty `text` still adds a node (an element with an empty text child
    /// serializes as a start and end tag).
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidNode`] when `parent` is not an element;
    /// [`XmlErrorKind::InvalidChar`]; the node or text-length limit.
    pub fn append_text(&mut self, parent: NodeId, text: &str) -> Result<NodeId, XmlError> {
        if self.element_data(parent.0).is_none() {
            return Err(XmlError::edit(XmlErrorKind::InvalidNode));
        }
        self.check_text(text)?;
        self.append(parent.0, NodeData::Text(text.into()))
            .map(NodeId)
            .map_err(XmlError::edit)
    }

    /// Appends a deep copy of node `node` of `src` (any kind except the
    /// document node) as the last child of the element `parent`. Namespaces
    /// are kept as resolved in `src`; the serializer declares what the copy
    /// needs. Returns the id of the copy.
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidNode`] when `parent` is not an element or
    /// `node` is not a node of `src`; the depth or node limit of `self`
    /// (nothing is appended then).
    pub fn append_copy(
        &mut self,
        parent: NodeId,
        src: &Self,
        node: NodeId,
    ) -> Result<NodeId, XmlError> {
        if self.element_data(parent.0).is_none() || node_kind(src, node.0).is_none() {
            return Err(XmlError::edit(XmlErrorKind::InvalidNode));
        }
        let rollback_len = self.nodes.len();
        let rollback_last = self.get(parent.0).map_or(NIL, |n| n.last_child);
        let result = self.copy_subtree(parent.0, src, node.0);
        if result.is_err() {
            // Undo a partial copy: drop the new nodes and relink the parent.
            self.nodes.truncate(rollback_len);
            if let Some(p) = self.get_mut(parent.0) {
                p.last_child = rollback_last;
                if rollback_last == NIL {
                    p.first_child = NIL;
                }
            }
            if let Some(prev) = self.get_mut(rollback_last) {
                prev.next_sibling = NIL;
            }
        }
        result.map(NodeId).map_err(XmlError::edit)
    }

    /// Pre-order walk of `src` from `start`, appending each node under the
    /// copy of its parent. Iterative: the walk follows the sibling and
    /// parent links, and `dest_parents` holds one entry per open element.
    fn copy_subtree(&mut self, parent: u32, src: &Self, start: u32) -> Result<u32, XmlErrorKind> {
        let base_depth = self.depth(parent);
        let max_depth = self.max_depth();
        let mut dest_parents: Vec<u32> = vec![parent];
        let mut copy_root = NIL;
        let mut cur = start;
        loop {
            let node = src.get(cur).ok_or(XmlErrorKind::InvalidNode)?;
            let dest_parent = *dest_parents.last().ok_or(XmlErrorKind::InvalidNode)?;
            if matches!(node.data, NodeData::Element(_))
                && base_depth.saturating_add(dest_parents.len()) > max_depth
            {
                return Err(XmlErrorKind::LimitExceeded(LimitKind::Depth));
            }
            let id = self.append(dest_parent, node.data.clone())?;
            if copy_root == NIL {
                copy_root = id;
            }
            if let Some(child) = link(node.first_child) {
                dest_parents.push(id);
                cur = child;
                continue;
            }
            // Climb until a node with a next sibling, stopping at `start`.
            loop {
                if cur == start {
                    return Ok(copy_root);
                }
                let n = src.get(cur).ok_or(XmlErrorKind::InvalidNode)?;
                if let Some(next) = link(n.next_sibling) {
                    cur = next;
                    break;
                }
                cur = n.parent;
                dest_parents.pop();
            }
        }
    }

    /// Unlinks node `node` (and its subtree) from its parent. The root
    /// element cannot be detached. Detached nodes stay in the arena and
    /// still count toward `max_nodes`.
    ///
    /// # Errors
    ///
    /// [`XmlErrorKind::InvalidNode`] for the root element, the document
    /// node, an unknown id, or a node that is already detached.
    pub fn detach(&mut self, node: NodeId) -> Result<(), XmlError> {
        let invalid = XmlError::edit(XmlErrorKind::InvalidNode);
        if node.0 == DOC || node.0 == self.root {
            return Err(invalid);
        }
        let (parent, prev, next) = match self.get(node.0) {
            Some(n) if n.parent != NIL => (n.parent, n.prev_sibling, n.next_sibling),
            _ => return Err(invalid),
        };
        match self.get_mut(prev) {
            Some(p) => p.next_sibling = next,
            None => {
                if let Some(p) = self.get_mut(parent) {
                    p.first_child = next;
                }
            }
        }
        match self.get_mut(next) {
            Some(n) => n.prev_sibling = prev,
            None => {
                if let Some(p) = self.get_mut(parent) {
                    p.last_child = prev;
                }
            }
        }
        if let Some(n) = self.get_mut(node.0) {
            n.parent = NIL;
            n.prev_sibling = NIL;
            n.next_sibling = NIL;
        }
        Ok(())
    }
}

/// Test-only count of child-list steps taken on this thread, so tests can
/// assert that an operation is linear in its input without timing it
/// (QA-001: wall-clock time is banned in this crate and flaky in CI).
#[cfg(test)]
pub(crate) mod child_steps {
    use std::cell::Cell;

    thread_local! {
        static STEPS: Cell<u64> = const { Cell::new(0) };
    }

    pub(crate) fn bump() {
        STEPS.with(|s| s.set(s.get().saturating_add(1)));
    }

    /// The steps counted since the last call, resetting the count.
    pub(crate) fn take() -> u64 {
        STEPS.with(|s| s.replace(0))
    }
}
