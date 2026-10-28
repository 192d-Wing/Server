// SPDX-License-Identifier: Apache-2.0

//! The bounded, single-pass parser that builds a `Document` from a `&str`
//! and enforces every `Limits` value before the allocation it bounds.
//!
//! quick-xml supplies the markup events. On top of them this module refuses
//! document type declarations, resolves only the predefined entities and
//! character references, checks `Char`/`Name`/`QName` rules, normalises line
//! ends and attribute values (XML 1.0 sections 2.11 and 3.3.3), tokenizes
//! attributes itself (quick-xml accepts attributes with no white space
//! between them), resolves namespace prefixes, and checks well-formedness
//! (matched tags, one root, nothing but comments, processing instructions
//! and white space outside it). Element nesting is tracked on an explicit
//! stack; nothing recurses.
//!
//! Derived from the XML 1.0 (Fifth Edition) and Namespaces in XML 1.0
//! (Third Edition) specifications and the reference's parser configuration
//! (DOCTYPE refused, no external entities, namespace-aware), not from the
//! reference's code.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SI-10(3) (Predictable
//   Behavior), SC-5 (Denial-of-service Protection)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002550 / V-222608
//   (unverified)
// PARITY: src/takserver-plugins/src/main/java/tak/server/cot/CotParser.java#CotParser

use std::sync::Arc;

use quick_xml::Reader;
use quick_xml::errors::{Error as QxError, IllFormedError, SyntaxError};
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesDecl, BytesPI, BytesStart, Event};

use super::document::{Attr, Document, ElementData, NodeData, NsDecl, PiData, XmlDeclaration};
use super::names::{is_name, is_ncname, is_xml_char, is_xml_space, split_qname};
use super::scope::{Interner, Ns, NsScope, XML_NS, XMLNS_NS};
use crate::error::{LimitKind, XmlError, XmlErrorKind};
use crate::limits::Limits;

/// Index of the document node in the arena.
const DOC: u32 = 0;

/// A failure inside a sub-step: the kind and a byte offset relative to the
/// sub-step's input, turned into an absolute [`XmlError`] by the caller.
type Local<T> = Result<T, (XmlErrorKind, usize)>;

/// Parses `input` into a [`Document`]; see [`Document::parse`].
pub(crate) fn parse(input: &str, limits: &Limits) -> Result<Document, XmlError> {
    if input.len() > limits.max_input_bytes {
        return Err(XmlError::at(
            XmlErrorKind::LimitExceeded(LimitKind::InputBytes),
            limits.max_input_bytes,
        ));
    }
    // A leading byte-order mark is an encoding signature, not content. It is
    // stripped here (quick-xml would strip it too, but then report offsets
    // relative to the text after it). A second one is content before the
    // root element.
    let (body, base) = match input.strip_prefix('\u{FEFF}') {
        Some(rest) => (rest, '\u{FEFF}'.len_utf8()),
        None => (input, 0),
    };
    if body.starts_with('\u{FEFF}') {
        return Err(XmlError::at(XmlErrorKind::ContentOutsideRoot, base));
    }

    let mut reader = Reader::from_str(body);
    let config = reader.config_mut();
    config.allow_dangling_amp = false;
    config.allow_unmatched_ends = false;
    config.check_comments = true;
    config.check_end_names = true;
    config.expand_empty_elements = false;
    config.trim_markup_names_in_closing_tags = true;
    config.trim_text(false);

    let mut parser = Parser::new(limits, base);
    loop {
        let at = parser.absolute(reader.buffer_position());
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => parser.event(event, at)?,
            Err(e) => {
                let at = parser.absolute(reader.error_position());
                return Err(XmlError::at(map_quick_xml_error(&e), at));
            }
        }
    }
    let end = parser.absolute(reader.buffer_position());
    parser.finish(end)
}

/// Maps a quick-xml error to a kind. Its own text is dropped: it can quote
/// input (tag names), which errors must not carry (SI-11).
fn map_quick_xml_error(e: &QxError) -> XmlErrorKind {
    match e {
        QxError::Syntax(SyntaxError::UnclosedDoctype)
        | QxError::IllFormed(IllFormedError::MissingDoctypeName) => XmlErrorKind::Doctype,
        QxError::IllFormed(
            IllFormedError::MismatchedEndTag { .. } | IllFormedError::UnmatchedEndTag(_),
        ) => XmlErrorKind::MismatchedEndTag,
        QxError::IllFormed(IllFormedError::MissingEndTag(_)) => XmlErrorKind::UnexpectedEof,
        QxError::InvalidAttr(AttrError::Duplicated(..)) => XmlErrorKind::DuplicateAttribute,
        _ => XmlErrorKind::Syntax,
    }
}

/// One attribute or declaration as tokenized from a start tag.
struct RawAttr<'i> {
    name: &'i str,
    value: String,
    /// Offset of the name, relative to the start tag.
    offset: usize,
}

struct Parser<'l> {
    limits: &'l Limits,
    /// Length of a stripped byte-order mark, added to quick-xml offsets.
    base: usize,
    doc: Document,
    /// Open elements, innermost last.
    open: Vec<u32>,
    scope: NsScope,
    interner: Interner,
    xml_ns: Ns,
    /// Character data not yet stored: adjacent text events and references
    /// merge into one text node.
    text: String,
    text_at: Option<usize>,
    seen_event: bool,
    root_closed: bool,
}

impl<'l> Parser<'l> {
    fn new(limits: &'l Limits, base: usize) -> Self {
        let mut interner = Interner::default();
        let xml_ns = interner.intern(XML_NS);
        Self {
            limits,
            base,
            doc: Document::empty(limits),
            open: Vec::new(),
            scope: NsScope::default(),
            interner,
            xml_ns,
            text: String::new(),
            text_at: None,
            seen_event: false,
            root_closed: false,
        }
    }

    fn absolute(&self, pos: u64) -> usize {
        usize::try_from(pos)
            .unwrap_or(usize::MAX)
            .saturating_add(self.base)
    }

    fn max_text(&self) -> usize {
        usize::try_from(self.limits.max_text_bytes).unwrap_or(usize::MAX)
    }

    /// The node new content goes under: the innermost open element, or the
    /// document node outside the root element.
    fn parent(&self) -> u32 {
        self.open.last().copied().unwrap_or(DOC)
    }

    fn event(&mut self, event: Event<'_>, at: usize) -> Result<(), XmlError> {
        if !matches!(event, Event::Text(_) | Event::GeneralRef(_)) {
            self.flush_text()?;
        }
        let first = !self.seen_event;
        self.seen_event = true;
        match event {
            Event::Text(t) => self.text(&t, at),
            Event::GeneralRef(r) => self.reference(&r, at),
            Event::Start(e) => self.start(&e, at, false),
            Event::Empty(e) => self.start(&e, at, true),
            Event::End(e) => self.end(e.name().as_ref(), at),
            Event::CData(c) => self.cdata(&c, at),
            Event::Comment(c) => self.comment(&c, at),
            Event::Decl(d) => self.declaration(&d, at, first),
            Event::PI(pi) => self.processing_instruction(&pi, at),
            Event::DocType(_) => Err(XmlError::at(XmlErrorKind::Doctype, at)),
            Event::Eof => Ok(()),
        }
    }

    fn finish(mut self, at: usize) -> Result<Document, XmlError> {
        self.flush_text()?;
        if !self.open.is_empty() {
            return Err(XmlError::at(XmlErrorKind::UnexpectedEof, at));
        }
        if !self.root_closed {
            return Err(XmlError::at(XmlErrorKind::NoRoot, at));
        }
        Ok(self.doc)
    }

    // ---- character data ----

    fn text(&mut self, raw: &str, at: usize) -> Result<(), XmlError> {
        if raw.is_empty() {
            return Ok(());
        }
        if self.open.is_empty() {
            // Outside the root only white space is allowed, and it is not
            // kept.
            return match raw.find(|c: char| !is_xml_space(c)) {
                Some(i) => Err(XmlError::at(
                    XmlErrorKind::ContentOutsideRoot,
                    at.saturating_add(i),
                )),
                None => Ok(()),
            };
        }
        if let Some(i) = raw.find("]]>") {
            return Err(XmlError::at(XmlErrorKind::Syntax, at.saturating_add(i)));
        }
        if self.text_at.is_none() {
            self.text_at = Some(at);
        }
        let max = self.max_text();
        push_char_data(&mut self.text, raw, max, LimitKind::TextBytes)
            .map_err(|(kind, i)| XmlError::at(kind, at.saturating_add(i)))
    }

    fn reference(&mut self, name: &str, at: usize) -> Result<(), XmlError> {
        if self.open.is_empty() {
            return Err(XmlError::at(XmlErrorKind::ContentOutsideRoot, at));
        }
        let c = resolve_reference(name).map_err(|kind| XmlError::at(kind, at))?;
        if self.text_at.is_none() {
            self.text_at = Some(at);
        }
        if self
            .text
            .len()
            .checked_add(c.len_utf8())
            .is_none_or(|n| n > self.max_text())
        {
            return Err(XmlError::at(
                XmlErrorKind::LimitExceeded(LimitKind::TextBytes),
                at,
            ));
        }
        self.text.push(c);
        Ok(())
    }

    /// Stores pending character data as one text node.
    fn flush_text(&mut self) -> Result<(), XmlError> {
        let Some(at) = self.text_at.take() else {
            return Ok(());
        };
        let text = core::mem::take(&mut self.text).into_boxed_str();
        self.doc
            .append(self.parent(), NodeData::Text(text))
            .map(|_| ())
            .map_err(|kind| XmlError::at(kind, at))
    }

    fn cdata(&mut self, raw: &str, at: usize) -> Result<(), XmlError> {
        if self.open.is_empty() {
            return Err(XmlError::at(XmlErrorKind::ContentOutsideRoot, at));
        }
        let content = self.char_data(raw, at, "<![CDATA[".len())?;
        self.append(NodeData::CData(content), at)
    }

    fn comment(&mut self, raw: &str, at: usize) -> Result<(), XmlError> {
        // quick-xml reports "--" inside the comment; a trailing "-" (which
        // would make "--->") is checked here.
        if raw.contains("--") || raw.ends_with('-') {
            return Err(XmlError::at(XmlErrorKind::Syntax, at));
        }
        let content = self.char_data(raw, at, "<!--".len())?;
        self.append(NodeData::Comment(content), at)
    }

    fn processing_instruction(&mut self, pi: &BytesPI<'_>, at: usize) -> Result<(), XmlError> {
        let target = pi.target();
        self.check_name_len(target, at)?;
        // Namespace-aware: a target is an NCName (no colon).
        if !is_ncname(target) {
            return Err(XmlError::at(XmlErrorKind::InvalidName, at));
        }
        if target.eq_ignore_ascii_case("xml") {
            return Err(XmlError::at(XmlErrorKind::ReservedPiTarget, at));
        }
        // The target ends at white space, so non-empty content starts with
        // the separating white space, which is not part of the data.
        let raw = pi.content().trim_start_matches(is_xml_space);
        let data_offset = "<?"
            .len()
            .saturating_add(pi.len())
            .saturating_sub(raw.len());
        let data = self.char_data(raw, at, data_offset)?;
        let pi = PiData {
            target: target.into(),
            data,
        };
        self.append(NodeData::Pi(Box::new(pi)), at)
    }

    /// Validates and line-end-normalises the content of a CDATA section,
    /// comment or processing instruction that starts `skip` bytes after
    /// `at`.
    fn char_data(&self, raw: &str, at: usize, skip: usize) -> Result<Box<str>, XmlError> {
        let mut out = String::new();
        push_char_data(&mut out, raw, self.max_text(), LimitKind::TextBytes)
            .map_err(|(kind, i)| XmlError::at(kind, at.saturating_add(skip).saturating_add(i)))?;
        Ok(out.into_boxed_str())
    }

    fn append(&mut self, data: NodeData, at: usize) -> Result<(), XmlError> {
        self.doc
            .append(self.parent(), data)
            .map(|_| ())
            .map_err(|kind| XmlError::at(kind, at))
    }

    // ---- declaration ----

    fn declaration(
        &mut self,
        decl: &BytesDecl<'_>,
        at: usize,
        first: bool,
    ) -> Result<(), XmlError> {
        // Only at the very start (after a byte-order mark); anywhere else
        // `<?xml ...?>` is a processing instruction with a reserved target.
        if !first || at != self.base {
            return Err(XmlError::at(XmlErrorKind::XmlDeclaration, at));
        }
        let decl = parse_declaration(decl).map_err(|kind| XmlError::at(kind, at))?;
        self.doc.declaration = Some(decl);
        Ok(())
    }

    // ---- elements ----

    fn check_name_len(&self, name: &str, at: usize) -> Result<(), XmlError> {
        if name.len() > usize::from(self.limits.max_name_bytes) {
            return Err(XmlError::at(
                XmlErrorKind::LimitExceeded(LimitKind::NameBytes),
                at,
            ));
        }
        Ok(())
    }

    fn start(&mut self, tag: &BytesStart<'_>, at: usize, empty: bool) -> Result<(), XmlError> {
        if self.root_closed {
            return Err(XmlError::at(XmlErrorKind::MultipleRoots, at));
        }
        if self.open.len() >= usize::from(self.limits.max_depth) {
            return Err(XmlError::at(
                XmlErrorKind::LimitExceeded(LimitKind::Depth),
                at,
            ));
        }
        let qname = tag.name();
        let name = qname.as_ref();
        self.check_name_len(name, at)?;
        let Some((prefix, _)) = split_qname(name) else {
            return Err(XmlError::at(XmlErrorKind::InvalidName, at));
        };

        // Attributes start right after `<` and the name.
        let attrs_at = at.saturating_add(1).saturating_add(name.len());
        let (attributes, decls) = self
            .tokenize_attributes(tag.attributes_raw())
            .map_err(|(kind, i)| XmlError::at(kind, attrs_at.saturating_add(i)))?;

        self.scope.push_frame();
        let namespace_decls = self.bind_declarations(decls, attrs_at)?;
        let namespace = self.resolve_element(prefix, at)?;
        let attributes = self.resolve_attributes(&attributes, attrs_at)?;

        let element = ElementData {
            name: name.into(),
            namespace,
            attributes,
            namespace_decls,
        };
        let id = self
            .doc
            .append(self.parent(), NodeData::Element(Box::new(element)))
            .map_err(|kind| XmlError::at(kind, at))?;
        if self.open.is_empty() {
            self.doc.root = id;
        }
        if empty {
            self.close_element();
        } else {
            self.open.push(id);
        }
        Ok(())
    }

    fn end(&mut self, name: &str, at: usize) -> Result<(), XmlError> {
        // quick-xml already checks end names (`check_end_names`); this is
        // the independent check against the arena.
        let matches = self
            .open
            .last()
            .and_then(|&id| self.doc.element_data(id))
            .is_some_and(|e| &*e.name == name);
        if !matches {
            return Err(XmlError::at(XmlErrorKind::MismatchedEndTag, at));
        }
        self.open.pop();
        self.close_element();
        Ok(())
    }

    /// Closes the namespace frame of the element just ended, and notes when
    /// that was the root.
    fn close_element(&mut self) {
        self.scope.pop_frame();
        if self.open.is_empty() {
            self.root_closed = true;
        }
    }

    /// Splits the raw attribute text of a start tag into attributes and
    /// namespace declarations, checking names, counts and value lengths and
    /// normalising values. Offsets are relative to `raw`.
    fn tokenize_attributes<'i>(&self, raw: &'i str) -> Local<(Vec<RawAttr<'i>>, Vec<RawAttr<'i>>)> {
        let max_attrs = usize::from(self.limits.max_attributes);
        let max_decls = usize::from(self.limits.max_namespace_decls);
        let max_value = usize::try_from(self.limits.max_attr_value_bytes).unwrap_or(usize::MAX);
        let max_name = usize::from(self.limits.max_name_bytes);
        let offset_of = |rest: &str| raw.len().saturating_sub(rest.len());

        let mut attributes: Vec<RawAttr<'i>> = Vec::new();
        let mut decls: Vec<RawAttr<'i>> = Vec::new();
        let mut rest = raw;
        loop {
            let token = rest.trim_start_matches(is_xml_space);
            if token.is_empty() {
                break;
            }
            // Every attribute must be preceded by white space.
            if token.len() == rest.len() {
                return Err((XmlErrorKind::Syntax, offset_of(token)));
            }
            let name_offset = offset_of(token);
            let name_end = token
                .find(|c: char| c == '=' || is_xml_space(c))
                .unwrap_or(token.len());
            let (name, after_name) = token
                .split_at_checked(name_end)
                .ok_or((XmlErrorKind::Syntax, name_offset))?;
            if name.len() > max_name {
                return Err((
                    XmlErrorKind::LimitExceeded(LimitKind::NameBytes),
                    name_offset,
                ));
            }
            if split_qname(name).is_none() {
                return Err((XmlErrorKind::InvalidName, name_offset));
            }
            let is_decl = name == "xmlns" || name.starts_with("xmlns:");
            let (list, max, limit) = if is_decl {
                (&mut decls, max_decls, LimitKind::NamespaceDecls)
            } else {
                (&mut attributes, max_attrs, LimitKind::Attributes)
            };
            if list.len() >= max {
                return Err((XmlErrorKind::LimitExceeded(limit), name_offset));
            }

            let after_eq = after_name
                .trim_start_matches(is_xml_space)
                .strip_prefix('=')
                .ok_or((XmlErrorKind::Syntax, offset_of(after_name)))?
                .trim_start_matches(is_xml_space);
            let Some(quote @ ('"' | '\'')) = after_eq.chars().next() else {
                return Err((XmlErrorKind::Syntax, offset_of(after_eq)));
            };
            let body = after_eq.get(1..).unwrap_or_default();
            let close = body
                .find(quote)
                .ok_or((XmlErrorKind::Syntax, offset_of(after_eq)))?;
            let (raw_value, after_value) = body
                .split_at_checked(close)
                .ok_or((XmlErrorKind::Syntax, offset_of(body)))?;
            let value_offset = offset_of(body);
            let value = normalize_attribute_value(raw_value, max_value)
                .map_err(|(kind, i)| (kind, value_offset.saturating_add(i)))?;
            list.push(RawAttr {
                name,
                value,
                offset: name_offset,
            });
            rest = after_value.get(1..).unwrap_or_default();
        }

        // Duplicate names, declarations included. Sorting keeps this
        // n log n for the largest permitted element.
        let mut names: Vec<&str> = attributes
            .iter()
            .chain(decls.iter())
            .map(|a| a.name)
            .collect();
        names.sort_unstable();
        if names.windows(2).any(|w| w.first() == w.last()) {
            return Err((XmlErrorKind::DuplicateAttribute, 0));
        }
        Ok((attributes, decls))
    }

    /// Checks and binds an element's namespace declarations (Namespaces in
    /// XML 1.0 section 3 constraints) and returns them for storage.
    fn bind_declarations(
        &mut self,
        decls: Vec<RawAttr<'_>>,
        attrs_at: usize,
    ) -> Result<Vec<NsDecl>, XmlError> {
        let mut out = Vec::with_capacity(decls.len());
        for decl in decls {
            let prefix = decl.name.strip_prefix("xmlns:");
            let uri = decl.value.as_str();
            let allowed = match prefix {
                Some("xmlns") => false,
                Some("xml") => uri == XML_NS,
                // XML 1.0 namespaces cannot undeclare a prefix.
                Some(_) => !uri.is_empty() && uri != XML_NS && uri != XMLNS_NS,
                None => uri != XML_NS && uri != XMLNS_NS,
            };
            if !allowed {
                return Err(XmlError::at(
                    XmlErrorKind::InvalidNamespaceDecl,
                    attrs_at.saturating_add(decl.offset),
                ));
            }
            let ns = self.interner.intern(uri);
            // `xml` is bound by definition; declaring it changes nothing.
            if prefix != Some("xml") {
                self.scope.bind(prefix.unwrap_or(""), ns.clone());
            }
            out.push(NsDecl {
                prefix: prefix.map(Into::into),
                uri: ns.uri,
            });
        }
        Ok(out)
    }

    fn resolve_prefix(&self, prefix: &str) -> Option<Ns> {
        if prefix == "xml" {
            return Some(self.xml_ns.clone());
        }
        self.scope.lookup(prefix).cloned()
    }

    fn resolve_element(
        &self,
        prefix: Option<&str>,
        at: usize,
    ) -> Result<Option<Arc<str>>, XmlError> {
        match prefix {
            Some("xmlns") => Err(XmlError::at(XmlErrorKind::InvalidName, at)),
            Some(p) => self
                .resolve_prefix(p)
                .map(|ns| Some(ns.uri))
                .ok_or_else(|| XmlError::at(XmlErrorKind::UndeclaredPrefix, at)),
            None => Ok(self
                .scope
                .lookup("")
                .filter(|ns| !ns.uri.is_empty())
                .map(|ns| Arc::clone(&ns.uri))),
        }
    }

    /// Resolves attribute prefixes and rejects two attributes with the same
    /// namespace and local name.
    fn resolve_attributes(
        &self,
        raw: &[RawAttr<'_>],
        attrs_at: usize,
    ) -> Result<Vec<Attr>, XmlError> {
        let mut out = Vec::with_capacity(raw.len());
        let mut expanded: Vec<(u32, &str)> = Vec::new();
        for attr in raw {
            let namespace = match attr.name.split_once(':') {
                None => None,
                Some((prefix, local)) => {
                    let ns = self.resolve_prefix(prefix).ok_or_else(|| {
                        XmlError::at(
                            XmlErrorKind::UndeclaredPrefix,
                            attrs_at.saturating_add(attr.offset),
                        )
                    })?;
                    expanded.push((ns.id, local));
                    Some(ns.uri)
                }
            };
            out.push(Attr {
                name: attr.name.into(),
                namespace,
                value: attr.value.as_str().into(),
            });
        }
        // Unprefixed attributes are in no namespace, so only prefixed ones
        // can share an expanded name without sharing a raw name.
        expanded.sort_unstable();
        if expanded.windows(2).any(|w| w.first() == w.last()) {
            return Err(XmlError::at(XmlErrorKind::DuplicateAttribute, attrs_at));
        }
        Ok(out)
    }
}

/// Appends `raw` to `dst` with line ends normalised (CR LF and lone CR
/// become LF, XML 1.0 section 2.11), rejecting non-`Char`s and stopping at
/// `max` bytes. The error offset is relative to `raw`.
fn push_char_data(dst: &mut String, raw: &str, max: usize, limit: LimitKind) -> Local<()> {
    let mut chars = raw.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let out = match c {
            '\r' => {
                chars.next_if(|&(_, next)| next == '\n');
                '\n'
            }
            c if is_xml_char(c) => c,
            _ => return Err((XmlErrorKind::InvalidChar, i)),
        };
        if dst
            .len()
            .checked_add(out.len_utf8())
            .is_none_or(|n| n > max)
        {
            return Err((XmlErrorKind::LimitExceeded(limit), i));
        }
        dst.push(out);
    }
    Ok(())
}

/// Attribute-value normalisation for CDATA-type attributes (XML 1.0
/// section 3.3.3; no DTD means every attribute is CDATA): line ends
/// normalised first, then each white-space character becomes a space,
/// references are resolved and the characters they produce are kept as is
/// (so `&#10;` stays a line feed). A literal `<` is not allowed. The error
/// offset is relative to `raw`.
fn normalize_attribute_value(raw: &str, max: usize) -> Local<String> {
    let mut out = String::new();
    let mut rest = raw;
    while let Some(c) = rest.chars().next() {
        let offset = raw.len().saturating_sub(rest.len());
        let (produced, consumed) = match c {
            '<' => return Err((XmlErrorKind::Syntax, offset)),
            '&' => {
                let (name, _) = rest
                    .get(1..)
                    .and_then(|r| r.split_once(';'))
                    .ok_or((XmlErrorKind::Syntax, offset))?;
                let resolved = resolve_reference(name).map_err(|kind| (kind, offset))?;
                // `&`, the name and `;`.
                (resolved, name.len().saturating_add(2))
            }
            '\r' if rest.get(1..).is_some_and(|r| r.starts_with('\n')) => (' ', 2),
            '\r' | '\n' | '\t' => (' ', 1),
            c if is_xml_char(c) => (c, c.len_utf8()),
            _ => return Err((XmlErrorKind::InvalidChar, offset)),
        };
        if out
            .len()
            .checked_add(produced.len_utf8())
            .is_none_or(|n| n > max)
        {
            return Err((
                XmlErrorKind::LimitExceeded(LimitKind::AttrValueBytes),
                offset,
            ));
        }
        out.push(produced);
        rest = rest.get(consumed..).unwrap_or_default();
    }
    Ok(out)
}

/// Resolves the text between `&` and `;`: the five predefined entities and
/// decimal or hexadecimal character references to XML 1.0 `Char`s. No
/// other entity exists, because no DTD is accepted.
fn resolve_reference(name: &str) -> Result<char, XmlErrorKind> {
    match name {
        "lt" => return Ok('<'),
        "gt" => return Ok('>'),
        "amp" => return Ok('&'),
        "apos" => return Ok('\''),
        "quot" => return Ok('"'),
        _ => {}
    }
    let Some(number) = name.strip_prefix('#') else {
        return Err(if is_name(name) {
            XmlErrorKind::UndefinedEntity
        } else {
            XmlErrorKind::Syntax
        });
    };
    let (digits, radix) = match number.strip_prefix('x') {
        Some(hex) => (hex, 16),
        None => (number, 10),
    };
    if digits.is_empty() {
        return Err(XmlErrorKind::InvalidCharRef);
    }
    let mut value: u32 = 0;
    for d in digits.chars() {
        value = d
            .to_digit(radix)
            .and_then(|d| value.checked_mul(radix)?.checked_add(d))
            .ok_or(XmlErrorKind::InvalidCharRef)?;
    }
    char::from_u32(value)
        .filter(|&c| is_xml_char(c))
        .ok_or(XmlErrorKind::InvalidCharRef)
}

/// Parses the XML declaration (XML 1.0 productions 23-26, 32, 80-81):
/// `version` first and required, then optional `encoding` and
/// `standalone`, in that order, each preceded by white space.
fn parse_declaration(decl: &BytesDecl<'_>) -> Result<XmlDeclaration, XmlErrorKind> {
    let bad = XmlErrorKind::XmlDeclaration;
    let mut rest = decl.strip_prefix("xml").ok_or(bad)?;

    let version = pseudo_attribute(&mut rest, "version")?.ok_or(bad)?;
    let is_version_num = version
        .strip_prefix("1.")
        .is_some_and(|minor| !minor.is_empty() && minor.bytes().all(|b| b.is_ascii_digit()));
    if !is_version_num {
        return Err(bad);
    }
    if version != "1.0" {
        // DEV-01b: XML 1.1 (and any later 1.x) is refused.
        return Err(XmlErrorKind::UnsupportedVersion);
    }

    let encoding = pseudo_attribute(&mut rest, "encoding")?;
    if let Some(enc) = encoding {
        let mut bytes = enc.bytes();
        let valid = bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
            && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
        if !valid {
            return Err(bad);
        }
    }
    let standalone = match pseudo_attribute(&mut rest, "standalone")? {
        None => None,
        Some("yes") => Some(true),
        Some("no") => Some(false),
        Some(_) => return Err(bad),
    };
    if !rest.trim_start_matches(is_xml_space).is_empty() {
        return Err(bad);
    }
    Ok(XmlDeclaration {
        encoding: encoding.map(Into::into),
        standalone,
    })
}

/// Reads `S name Eq quoted-value` from the front of `rest` when the next
/// pseudo-attribute is `name`, advancing `rest`; `Ok(None)` when it is not.
fn pseudo_attribute<'i>(rest: &mut &'i str, name: &str) -> Result<Option<&'i str>, XmlErrorKind> {
    let bad = XmlErrorKind::XmlDeclaration;
    let trimmed = rest.trim_start_matches(is_xml_space);
    let Some(after_name) = trimmed.strip_prefix(name) else {
        return Ok(None);
    };
    if trimmed.len() == rest.len() {
        return Err(bad);
    }
    let value = after_name
        .trim_start_matches(is_xml_space)
        .strip_prefix('=')
        .ok_or(bad)?
        .trim_start_matches(is_xml_space);
    let Some(quote @ ('"' | '\'')) = value.chars().next() else {
        return Err(bad);
    };
    let (inner, after) = value
        .get(1..)
        .and_then(|v| v.split_once(quote))
        .ok_or(bad)?;
    *rest = after;
    Ok(Some(inner))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references() {
        assert_eq!(resolve_reference("amp"), Ok('&'));
        assert_eq!(resolve_reference("#65"), Ok('A'));
        assert_eq!(resolve_reference("#x41"), Ok('A'));
        assert_eq!(resolve_reference("#xFFFD"), Ok('\u{FFFD}'));
        assert_eq!(resolve_reference("#0000000000000000065"), Ok('A'));
        assert_eq!(resolve_reference("#X41"), Err(XmlErrorKind::InvalidCharRef));
        assert_eq!(resolve_reference("#0"), Err(XmlErrorKind::InvalidCharRef));
        assert_eq!(
            resolve_reference("#xD800"),
            Err(XmlErrorKind::InvalidCharRef)
        );
        assert_eq!(
            resolve_reference("#xFFFE"),
            Err(XmlErrorKind::InvalidCharRef)
        );
        assert_eq!(
            resolve_reference("#99999999999"),
            Err(XmlErrorKind::InvalidCharRef)
        );
        assert_eq!(resolve_reference("#"), Err(XmlErrorKind::InvalidCharRef));
        assert_eq!(resolve_reference("#x"), Err(XmlErrorKind::InvalidCharRef));
        assert_eq!(
            resolve_reference("nbsp"),
            Err(XmlErrorKind::UndefinedEntity)
        );
        assert_eq!(resolve_reference("a b"), Err(XmlErrorKind::Syntax));
    }

    #[test]
    fn attribute_values() {
        let n = |s: &str| normalize_attribute_value(s, 64);
        assert_eq!(n("a\tb\nc\r\nd\re").as_deref(), Ok("a b c d e"));
        assert_eq!(n("&#9;&#10;&#13;").as_deref(), Ok("\t\n\r"));
        assert_eq!(n("&lt;&amp;&quot;").as_deref(), Ok("<&\""));
        assert_eq!(n("a<b"), Err((XmlErrorKind::Syntax, 1)));
        assert_eq!(n("a&b"), Err((XmlErrorKind::Syntax, 1)));
        assert_eq!(n("\u{1}"), Err((XmlErrorKind::InvalidChar, 0)));
        assert_eq!(
            normalize_attribute_value("abc", 2),
            Err((XmlErrorKind::LimitExceeded(LimitKind::AttrValueBytes), 2))
        );
    }

    #[test]
    fn char_data() {
        let mut s = String::new();
        push_char_data(&mut s, "a\r\nb\rc\n", 64, LimitKind::TextBytes).unwrap();
        assert_eq!(s, "a\nb\nc\n");
        assert_eq!(
            push_char_data(&mut s, "\u{0}", 64, LimitKind::TextBytes),
            Err((XmlErrorKind::InvalidChar, 0))
        );
    }
}
