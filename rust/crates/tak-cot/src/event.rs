// SPDX-License-Identifier: Apache-2.0

//! `CotEvent`: a parsed `CoT` document plus the numeric cache and the creation
//! and submission times that the conversion to TAK Protocol v1 reads (plan
//! section 1.6).
//!
//! An event parsed from XML has an empty cache, so every value sent to a
//! TAK Protocol client is read from the document. An event built from a
//! `TakMessage` remembers that message's exact numbers (millisecond times,
//! point doubles, battery, speed and course): its XML form shows whole
//! seconds and `Double.toString` text, but converting it back to TAK Protocol
//! prefers the remembered values (parity-map K-16), so a proto-to-proto path
//! keeps milliseconds.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation), SC-5 (Denial-of-service
//   Protection)
// PARITY: src/takserver-plugins/src/main/java/tak/server/cot/CotEventContainer.java#CotEventContainer

use crate::error::CotError;
use crate::limits::Limits;
use crate::xml::Document;

/// A `CoT` event: the XML document and the state kept beside it.
///
/// Immutable after construction except for the creation and submission
/// times. `Debug` shows no document content (positions and identities do not
/// belong in logs).
#[derive(Clone)]
pub struct CotEvent {
    doc: Document,
    pub(crate) cache: NumericCache,
    creation_ms: Option<i64>,
    submission_ms: Option<i64>,
}

/// Exact values remembered from the `TakMessage` an event was built from.
///
/// `None` means "not remembered" (an event parsed from XML, or no `Status` /
/// `Track` in the message). A remembered value equal to the reference's
/// "unset" marker, `-1` (or `-1.0`), is treated as not remembered when it is
/// read (K-16): see [`NumericCache::time`] and friends.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NumericCache {
    pub(crate) send: Option<i64>,
    pub(crate) start: Option<i64>,
    pub(crate) stale: Option<i64>,
    pub(crate) lat: Option<f64>,
    pub(crate) lon: Option<f64>,
    pub(crate) hae: Option<f64>,
    pub(crate) ce: Option<f64>,
    pub(crate) le: Option<f64>,
    pub(crate) battery: Option<i32>,
    pub(crate) speed: Option<f64>,
    pub(crate) course: Option<f64>,
}

impl NumericCache {
    /// A remembered time, unless it is the "unset" marker `-1`.
    pub(crate) fn time(value: Option<i64>) -> Option<i64> {
        value.filter(|&v| v != -1)
    }

    /// A remembered integer, unless it is the "unset" marker `-1`.
    pub(crate) fn int(value: Option<i32>) -> Option<i32> {
        value.filter(|&v| v != -1)
    }

    /// A remembered double, unless it equals `-1.0`. Only one bit pattern
    /// equals `-1.0`, so comparing bits is the same as comparing values; NaN
    /// and `-0.0` are kept.
    pub(crate) fn double(value: Option<f64>) -> Option<f64> {
        value.filter(|v| v.to_bits() != (-1.0_f64).to_bits())
    }
}

impl core::fmt::Debug for CotEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CotEvent")
            .field("doc", &self.doc)
            .finish_non_exhaustive()
    }
}

impl CotEvent {
    /// Parses one `CoT` XML event under `limits`.
    ///
    /// The document is parsed as [`Document::parse`] does; like the
    /// reference, no `CoT` schema is applied here (any root element is
    /// accepted, and missing attributes are handled by the conversion).
    ///
    /// # Errors
    ///
    /// [`CotError::Xml`] when the input is not an acceptable XML document.
    pub fn parse(input: &str, limits: &Limits) -> Result<Self, CotError> {
        Ok(Self::from_document(Document::parse(input, limits)?))
    }

    /// Wraps an already parsed or built document, with an empty numeric
    /// cache and no creation or submission time.
    #[must_use]
    pub fn from_document(doc: Document) -> Self {
        Self {
            doc,
            cache: NumericCache::default(),
            creation_ms: None,
            submission_ms: None,
        }
    }

    pub(crate) const fn from_parts(
        doc: Document,
        cache: NumericCache,
        creation_ms: Option<i64>,
        submission_ms: Option<i64>,
    ) -> Self {
        Self {
            doc,
            cache,
            creation_ms,
            submission_ms,
        }
    }

    /// The XML document.
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.doc
    }

    /// The document in the serializer's form (the reference's `asXml()`).
    #[must_use]
    pub fn to_xml_bytes(&self) -> Vec<u8> {
        self.doc.to_xml_bytes()
    }

    /// The root element's `uid` attribute.
    ///
    /// # Identity attributes match by local name
    ///
    /// This accessor, [`Self::type_`] and [`Self::how`] return the **first**
    /// root attribute whose local name matches, whatever its namespace (see
    /// [`ElementRef::attribute`](crate::xml::ElementRef::attribute)), which
    /// is the lookup [`to_tak_message`](crate::to_tak_message) uses for the
    /// same fields. In `<event xmlns:p="urn:x" p:uid="A" uid="B">` both read
    /// `A`, while the serialized XML keeps both attributes, so a receiver
    /// that reads only the unprefixed `uid` sees `B` (parity-map C-06 and
    /// D-18). The root attributes `time`, `start` and `stale`, read by the
    /// conversion, follow the same rule.
    ///
    /// Phase 3 must route every authorization, filtering and persistence
    /// decision about an event's identity through these accessors (the same
    /// lookup the converter uses), or reject events that carry more than one
    /// attribute with one of these local names (D-18); a second, different
    /// lookup would let one event present two identities.
    #[must_use]
    pub fn uid(&self) -> Option<&str> {
        self.doc.root().attribute("uid")
    }

    /// The root element's `type` attribute: the first attribute with local
    /// name `type`, whatever its namespace. See [`Self::uid`] for why
    /// identity decisions must use this accessor (D-18).
    #[must_use]
    pub fn type_(&self) -> Option<&str> {
        self.doc.root().attribute("type")
    }

    /// The root element's `how` attribute: the first attribute with local
    /// name `how`, whatever its namespace. See [`Self::uid`] for why identity
    /// decisions must use this accessor (D-18).
    #[must_use]
    pub fn how(&self) -> Option<&str> {
        self.doc.root().attribute("how")
    }

    /// When the event was created, in milliseconds since the epoch: carried
    /// from a `TakMessage` (when positive) or set by the caller.
    #[must_use]
    pub const fn creation_ms(&self) -> Option<i64> {
        self.creation_ms
    }

    /// When the event was submitted, in milliseconds since the epoch:
    /// carried from a `TakMessage` (when positive) or set by the caller.
    #[must_use]
    pub const fn submission_ms(&self) -> Option<i64> {
        self.submission_ms
    }

    /// Sets or clears the creation time. Only a positive value reaches a
    /// `TakMessage` (K-11).
    pub const fn set_creation_ms(&mut self, ms: Option<i64>) {
        self.creation_ms = ms;
    }

    /// Sets or clears the submission time. Only a positive value reaches a
    /// `TakMessage` (K-11).
    pub const fn set_submission_ms(&mut self, ms: Option<i64>) {
        self.submission_ms = ms;
    }
}
