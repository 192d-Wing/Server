// SPDX-License-Identifier: Apache-2.0

//! Typed errors with stable codes: the XML part from WU3, the conversion part
//! from WU5. `Display` never includes input text (SI-11).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SI-11 (Error Handling)
// - ASD STIG: APSC-DV-002570 / V-222610 (unverified)

use core::fmt;

/// Why an XML document was rejected, or why a document edit was refused.
///
/// Carries a category ([`XmlErrorKind`], with a stable code) and, for parse
/// errors, the byte offset in the input where the offending construct
/// starts. `Display` names only the category and the offset: no input text,
/// names or attribute values (plan section 5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct XmlError {
    kind: XmlErrorKind,
    offset: Option<usize>,
}

impl XmlError {
    /// An error found while parsing, at byte `offset` of the input.
    pub(crate) const fn at(kind: XmlErrorKind, offset: usize) -> Self {
        Self {
            kind,
            offset: Some(offset),
        }
    }

    /// An error from a document edit, which has no input offset.
    pub(crate) const fn edit(kind: XmlErrorKind) -> Self {
        Self { kind, offset: None }
    }

    /// The error category.
    #[must_use]
    pub const fn kind(&self) -> XmlErrorKind {
        self.kind
    }

    /// Byte offset in the input where the rejected construct starts (for a
    /// limit, where the limit was reached), or `None` for an edit error.
    #[must_use]
    pub const fn offset(&self) -> Option<usize> {
        self.offset
    }
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} at byte {offset}", self.kind),
            None => fmt::Display::fmt(&self.kind, f),
        }
    }
}

impl core::error::Error for XmlError {}

/// Category of an [`XmlError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum XmlErrorKind {
    /// Markup that does not follow the XML 1.0 grammar: unterminated
    /// constructs, malformed attributes, a bare `&`, `<` in an attribute
    /// value, `]]>` in text, `--` in a comment.
    Syntax,
    /// A document type declaration (`<!DOCTYPE`). Refused anywhere, so no
    /// entity can ever be declared (XXE, entity expansion).
    Doctype,
    /// A malformed XML declaration, or one that is not at the very start.
    XmlDeclaration,
    /// An XML declaration with a version other than `1.0` (DEV-01b).
    UnsupportedVersion,
    /// A reference to an entity other than the five predefined ones.
    UndefinedEntity,
    /// A character reference that is malformed or names a code point that
    /// is not an XML 1.0 `Char` (for example `&#0;`).
    InvalidCharRef,
    /// A character that is not an XML 1.0 `Char` (for example U+0001).
    InvalidChar,
    /// An element, attribute or processing-instruction name that is not a
    /// valid XML `Name`/`QName`.
    InvalidName,
    /// A processing instruction whose target is `xml` in any letter case.
    ReservedPiTarget,
    /// An element or attribute prefix with no namespace declaration in scope.
    UndeclaredPrefix,
    /// A namespace declaration the Namespaces in XML 1.0 rules forbid
    /// (empty prefixed URI, rebinding `xml` or `xmlns`, binding their URIs).
    InvalidNamespaceDecl,
    /// Two attributes with the same name, or the same namespace and local
    /// name, on one element.
    DuplicateAttribute,
    /// An end tag that does not match the open element, or has none.
    MismatchedEndTag,
    /// The input ended inside an element.
    UnexpectedEof,
    /// No root element.
    NoRoot,
    /// A second root element.
    MultipleRoots,
    /// Text, a reference or CDATA outside the root element.
    ContentOutsideRoot,
    /// A resource limit was exceeded; nothing was returned (DEV-01a).
    LimitExceeded(LimitKind),
    /// A document edit named a node that does not exist, is of the wrong
    /// kind for the edit, or may not be detached.
    InvalidNode,
}

impl XmlErrorKind {
    /// Stable code for metrics and audit records (e.g. `"xml.doctype"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Syntax => "xml.syntax",
            Self::Doctype => "xml.doctype",
            Self::XmlDeclaration => "xml.declaration",
            Self::UnsupportedVersion => "xml.unsupported_version",
            Self::UndefinedEntity => "xml.undefined_entity",
            Self::InvalidCharRef => "xml.invalid_char_ref",
            Self::InvalidChar => "xml.invalid_char",
            Self::InvalidName => "xml.invalid_name",
            Self::ReservedPiTarget => "xml.reserved_pi_target",
            Self::UndeclaredPrefix => "xml.undeclared_prefix",
            Self::InvalidNamespaceDecl => "xml.invalid_namespace_decl",
            Self::DuplicateAttribute => "xml.duplicate_attribute",
            Self::MismatchedEndTag => "xml.mismatched_end_tag",
            Self::UnexpectedEof => "xml.unexpected_eof",
            Self::NoRoot => "xml.no_root",
            Self::MultipleRoots => "xml.multiple_roots",
            Self::ContentOutsideRoot => "xml.content_outside_root",
            Self::LimitExceeded(limit) => limit.as_str(),
            Self::InvalidNode => "xml.invalid_node",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Syntax => "malformed XML markup",
            Self::Doctype => "document type declaration not allowed",
            Self::XmlDeclaration => "malformed or misplaced XML declaration",
            Self::UnsupportedVersion => "unsupported XML version",
            Self::UndefinedEntity => "reference to an undefined entity",
            Self::InvalidCharRef => "invalid character reference",
            Self::InvalidChar => "character not allowed in XML 1.0",
            Self::InvalidName => "invalid XML name",
            Self::ReservedPiTarget => "reserved processing-instruction target",
            Self::UndeclaredPrefix => "undeclared namespace prefix",
            Self::InvalidNamespaceDecl => "invalid namespace declaration",
            Self::DuplicateAttribute => "duplicate attribute",
            Self::MismatchedEndTag => "mismatched end tag",
            Self::UnexpectedEof => "unexpected end of input inside an element",
            Self::NoRoot => "no root element",
            Self::MultipleRoots => "more than one root element",
            Self::ContentOutsideRoot => "content outside the root element",
            Self::LimitExceeded(limit) => limit.description(),
            Self::InvalidNode => "invalid node for this document edit",
        }
    }
}

impl fmt::Display for XmlErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.description(), self.as_str())
    }
}

/// Which [`Limits`](crate::Limits) bound was exceeded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LimitKind {
    /// `max_input_bytes`.
    InputBytes,
    /// `max_depth`.
    Depth,
    /// `max_nodes`.
    Nodes,
    /// `max_attributes`.
    Attributes,
    /// `max_namespace_decls`.
    NamespaceDecls,
    /// `max_name_bytes`.
    NameBytes,
    /// `max_attr_value_bytes`.
    AttrValueBytes,
    /// `max_text_bytes`.
    TextBytes,
    /// `max_xml_detail_bytes`.
    XmlDetailBytes,
}

impl LimitKind {
    /// Stable code for metrics and audit records (e.g. `"xml.limit.depth"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputBytes => "xml.limit.input_bytes",
            Self::Depth => "xml.limit.depth",
            Self::Nodes => "xml.limit.nodes",
            Self::Attributes => "xml.limit.attributes",
            Self::NamespaceDecls => "xml.limit.namespace_decls",
            Self::NameBytes => "xml.limit.name_bytes",
            Self::AttrValueBytes => "xml.limit.attr_value_bytes",
            Self::TextBytes => "xml.limit.text_bytes",
            Self::XmlDetailBytes => "xml.limit.xml_detail_bytes",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::InputBytes => "input exceeds the size limit",
            Self::Depth => "element nesting exceeds the depth limit",
            Self::Nodes => "document exceeds the node limit",
            Self::Attributes => "element exceeds the attribute limit",
            Self::NamespaceDecls => "element exceeds the namespace declaration limit",
            Self::NameBytes => "name exceeds the length limit",
            Self::AttrValueBytes => "attribute value exceeds the length limit",
            Self::TextBytes => "character data exceeds the length limit",
            Self::XmlDetailBytes => "xmlDetail exceeds the size limit",
        }
    }
}

impl fmt::Display for LimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}

// ---- conversion errors (WU5) ----

/// Why `CotEvent::parse` rejected its input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CotError {
    /// The input is not an acceptable XML document.
    Xml(XmlError),
}

impl CotError {
    /// The error category.
    #[must_use]
    pub const fn kind(&self) -> CotErrorKind {
        match self {
            Self::Xml(e) => CotErrorKind::Xml(e.kind()),
        }
    }
}

impl From<XmlError> for CotError {
    fn from(e: XmlError) -> Self {
        Self::Xml(e)
    }
}

impl fmt::Display for CotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl core::error::Error for CotError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Xml(e) => Some(e),
        }
    }
}

/// Category of a [`CotError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CotErrorKind {
    /// See [`CotError::Xml`]; carries the XML category.
    Xml(XmlErrorKind),
}

impl CotErrorKind {
    /// Stable code for metrics and audit records: the XML code (e.g.
    /// `"xml.doctype"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Xml(kind) => kind.as_str(),
        }
    }
}

/// Which `CoT` time attribute of the `event` element could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TimeField {
    /// `time` (proto `sendTime`).
    Time,
    /// `start` (proto `startTime`).
    Start,
    /// `stale` (proto `staleTime`).
    Stale,
}

impl TimeField {
    /// The attribute name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Start => "start",
            Self::Stale => "stale",
        }
    }
}

/// Why a conversion between `CoT` XML and TAK Protocol v1 produced no
/// result. The reference returns no message in each of these cases
/// (parity-map K-12, K-21); TAK-RS names the cause.
///
/// `Display` names the category and, for XML errors, the XML category and
/// offset; never input text (SI-11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConvertError {
    /// XML to proto: a `time`, `start` or `stale` attribute that is not a
    /// `CoT` time, or is outside the millisecond range (K-02, DEV-05).
    Time(TimeField),
    /// XML to proto: the `xmlDetail` text built from the remaining `detail`
    /// children would exceed `Limits::max_xml_detail_bytes`.
    XmlDetailTooLarge,
    /// Proto to XML: the `xmlDetail` string is longer than
    /// `Limits::max_xml_detail_bytes`, is not well-formed once wrapped in a
    /// `detail` element, or its elements do not fit the event's limits
    /// (K-21).
    XmlDetail(XmlError),
    /// Proto to XML: a value cannot be written into the event document (a
    /// character XML 1.0 cannot carry, or a limit of the event).
    Build(XmlError),
}

impl ConvertError {
    /// The error category.
    #[must_use]
    pub const fn kind(&self) -> ConvertErrorKind {
        match self {
            Self::Time(field) => ConvertErrorKind::Time(*field),
            Self::XmlDetailTooLarge => ConvertErrorKind::XmlDetailTooLarge,
            Self::XmlDetail(_) => ConvertErrorKind::XmlDetail,
            Self::Build(_) => ConvertErrorKind::Build,
        }
    }

    /// The underlying XML error, for [`ConvertError::XmlDetail`] and
    /// [`ConvertError::Build`].
    #[must_use]
    pub const fn xml_error(&self) -> Option<XmlError> {
        match self {
            Self::XmlDetail(e) | Self::Build(e) => Some(*e),
            Self::Time(_) | Self::XmlDetailTooLarge => None,
        }
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind();
        write!(f, "{} ({})", kind.description(), kind.as_str())?;
        match self.xml_error() {
            Some(e) => write!(f, ": {e}"),
            None => Ok(()),
        }
    }
}

impl core::error::Error for ConvertError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::XmlDetail(e) | Self::Build(e) => Some(e),
            Self::Time(_) | Self::XmlDetailTooLarge => None,
        }
    }
}

/// Category of a [`ConvertError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConvertErrorKind {
    /// See [`ConvertError::Time`].
    Time(TimeField),
    /// See [`ConvertError::XmlDetailTooLarge`].
    XmlDetailTooLarge,
    /// See [`ConvertError::XmlDetail`].
    XmlDetail,
    /// See [`ConvertError::Build`].
    Build,
}

impl ConvertErrorKind {
    /// Stable code for metrics and audit records (e.g.
    /// `"convert.time.stale"`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time(TimeField::Time) => "convert.time.time",
            Self::Time(TimeField::Start) => "convert.time.start",
            Self::Time(TimeField::Stale) => "convert.time.stale",
            Self::XmlDetailTooLarge => "convert.xml_detail_too_large",
            Self::XmlDetail => "convert.xml_detail",
            Self::Build => "convert.build",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Time(TimeField::Time) => "unreadable event time",
            Self::Time(TimeField::Start) => "unreadable event start time",
            Self::Time(TimeField::Stale) => "unreadable event stale time",
            Self::XmlDetailTooLarge => "xmlDetail output exceeds the size limit",
            Self::XmlDetail => "malformed or oversized xmlDetail",
            Self::Build => "value cannot be written as XML",
        }
    }
}

impl fmt::Display for ConvertErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.description(), self.as_str())
    }
}
