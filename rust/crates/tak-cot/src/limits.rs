// SPDX-License-Identifier: Apache-2.0

//! `Limits`: the size, depth and count bounds applied to every parse (plan
//! section 3.4).
//!
//! The reference server applies none of these in its XML parser (it relies
//! on its stream splitter's 8 Mi-character cap); TAK-RS bounds every resource
//! an input can make the parser allocate. Inputs within the limits are parsed
//! exactly as before; inputs beyond them are rejected with
//! `XmlErrorKind::LimitExceeded` (parity-map DEV-01a, approved 2026-10-27).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SC-5 (Denial-of-service Protection), SI-10 (Information
//   Input Validation), CM-6 (Configuration Settings; library defaults only,
//   no operator setting and no ceiling until Phase 3)
// - ASD STIG: APSC-DV-002530 / V-222606, APSC-DV-002590 / V-222612
//   (unverified)
// POAM: POAM-2026-10-27-02 - the bounds are per element and per document, not per process: one
// conversion can hold about 23 times its input in memory (SEC-002), and
// nothing caps the conversions in flight. Phase 3 adds a document-wide
// allocation budget or a bounded ingest semaphore.
// PARITY: src/takserver-core/src/main/java/com/bbn/marti/nio/protocol/connections/StreamingCotProtocol.java#add

/// Resource bounds for parsing one `CoT` XML event (and the `xmlDetail` of a
/// TAK Protocol message, which is parsed with the same values).
///
/// Each bound is checked during the single parse pass, before the
/// allocation it limits. `Limits::default()` holds the approved values of
/// plan section 3.4. All lengths are in bytes of UTF-8 after line-end and
/// reference processing, except `max_input_bytes`, which bounds the raw
/// input.
///
/// These are library defaults, not operator configuration: nothing reads
/// them from a configuration file, and no field has an upper ceiling, so a
/// caller can raise them without bound. Phase 3 adds the operator setting
/// and its ceilings; until then callers use `Limits::default()` or lower
/// values.
///
/// # Memory
///
/// The bounds are per element (attributes, names, values) and per document
/// (input bytes, nodes, depth), so peak memory is a multiple of the input,
/// not equal to it. Measured on the worst shape (elements of 256 short
/// attributes filling 8 MiB): parsing peaks at about 95 MB, and the same
/// text as a TAK Protocol `xmlDetail` at about 194 MB through
/// `from_tak_message`, about 23 times the input. Phase 3 must bound this
/// across connections, with a document-wide allocation budget or an ingest
/// semaphore sized for about 200 MB per message in flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
    /// Largest accepted input document, in bytes (default 8 MiB).
    pub max_input_bytes: usize,
    /// Deepest element nesting; the root element is at depth 1 (default 64).
    pub max_depth: u16,
    /// Most nodes in one document: elements, text, CDATA sections, comments
    /// and processing instructions, including those outside the root
    /// element (default 100 000).
    pub max_nodes: u32,
    /// Most attributes on one element, not counting namespace declarations
    /// (default 256).
    pub max_attributes: u16,
    /// Most namespace declarations (`xmlns`, `xmlns:p`) on one element
    /// (default 32).
    pub max_namespace_decls: u16,
    /// Longest element, attribute or processing-instruction target name,
    /// prefix included (default 1 024).
    pub max_name_bytes: u16,
    /// Longest attribute or namespace-declaration value after normalisation
    /// (default 1 MiB).
    pub max_attr_value_bytes: u32,
    /// Longest text node (adjacent text and references merged), CDATA
    /// section, comment or processing-instruction data (default 8 MiB).
    pub max_text_bytes: u32,
    /// Longest `xmlDetail` string accepted from a TAK Protocol message
    /// (default 8 MiB); also the cap on the `xmlDetail` text built for a TAK
    /// Protocol client. Not used by the XML parser itself.
    ///
    /// The cap is exact: a string of up to this many bytes is parsed (with
    /// the other limits of this value, and `max_input_bytes` replaced by this
    /// cap, so the wrapper the conversion adds does not count), and a longer
    /// one is refused with `xml.limit.xml_detail_bytes` before parsing.
    pub max_xml_detail_bytes: usize,
}

impl Limits {
    /// 8 MiB, the default for `max_input_bytes`, `max_text_bytes` and
    /// `max_xml_detail_bytes`.
    pub const EIGHT_MIB: usize = 8 * 1024 * 1024;

    /// 1 MiB, the default for `max_attr_value_bytes`.
    pub const ONE_MIB: u32 = 1024 * 1024;
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_bytes: Self::EIGHT_MIB,
            max_depth: 64,
            max_nodes: 100_000,
            max_attributes: 256,
            max_namespace_decls: 32,
            max_name_bytes: 1024,
            max_attr_value_bytes: Self::ONE_MIB,
            // 8 MiB fits in u32.
            max_text_bytes: 8 * 1024 * 1024,
            max_xml_detail_bytes: Self::EIGHT_MIB,
        }
    }
}
