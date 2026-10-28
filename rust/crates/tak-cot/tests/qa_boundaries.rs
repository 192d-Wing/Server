// SPDX-License-Identifier: Apache-2.0

//! QA additions for Phase 1a (plan sections 3.4, 5.2, 5.3, 8.1):
//!
//! - the `xmlDetail` limits at the limit (accepted) and one past it
//!   (refused), in both directions, including where the `detail` wrapper
//!   meets `max_input_bytes`;
//! - limits of the event being built apply to the elements copied from a
//!   proto `xmlDetail` (depth, attributes, names), not only the wrapper's;
//! - hostile constructs inside a proto `xmlDetail` (entity expansion,
//!   external entities, a second declaration, breaking out of `detail`);
//! - no error text, `Debug` output or error source chain contains input
//!   content, for every XML error kind, every limit kind and every
//!   conversion error kind (SI-11).

use std::error::Error;
use std::fmt::Write as _;

use tak_cot::xml::Document;
use tak_cot::{
    ConvertError, ConvertErrorKind, CotEvent, LimitKind, Limits, ToProtoOptions, XmlErrorKind,
    from_tak_message, to_tak_message,
};
use tak_proto::v1::{CotEvent as ProtoEvent, Detail, TakMessage};

/// A marker that appears in no error description or code.
const MARK: &str = "Qz7Mark";

fn with_xml_detail(xml_detail: String) -> TakMessage {
    TakMessage {
        cot_event: Some(ProtoEvent {
            uid: "u".into(),
            detail: Some(Detail {
                xml_detail,
                ..Detail::default()
            }),
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    }
}

/// `<remarks>xxx…</remarks>` of exactly `len` bytes (`len >= 19`).
fn remarks_of_len(len: usize) -> String {
    let fill = len - "<remarks></remarks>".len();
    format!("<remarks>{}</remarks>", "x".repeat(fill))
}

/// The inner XML error code of a failed proto-to-XML conversion.
fn p2x_inner_code(msg: &TakMessage, limits: &Limits) -> Option<&'static str> {
    let err = from_tak_message(msg, limits).err()?;
    if err.kind() != ConvertErrorKind::XmlDetail {
        return None;
    }
    err.xml_error().map(|e| e.kind().as_str())
}

/// Display, Debug and every `source()` of an error, concatenated.
fn all_text(err: &(dyn Error + 'static)) -> String {
    let mut text = format!("{err} {err:?}");
    let mut source = err.source();
    while let Some(s) = source {
        let _ = write!(text, " | {s} {s:?}");
        source = s.source();
    }
    text
}

// ---- xmlDetail size: proto -> XML ----

#[test]
fn p2x_xml_detail_exactly_at_the_cap_accepted_one_byte_more_refused() {
    let limits = Limits {
        max_xml_detail_bytes: 64,
        ..Limits::default()
    };
    let at = with_xml_detail(remarks_of_len(64));
    let ev = from_tak_message(&at, &limits).unwrap();
    let detail = ev.document().root().first_child_element("detail").unwrap();
    assert!(detail.first_child_element("remarks").is_some());

    let over = with_xml_detail(remarks_of_len(65));
    assert_eq!(
        p2x_inner_code(&over, &limits),
        Some("xml.limit.xml_detail_bytes")
    );
}

#[test]
fn p2x_xml_detail_cap_is_exact_at_default_limits() {
    // QA-002: the wrapper the conversion adds around xmlDetail does not use
    // up the input budget, so at default limits (both 8 MiB) the largest
    // accepted xmlDetail is exactly max_xml_detail_bytes and one byte more
    // is refused with the xmlDetail code, before parsing.
    let l = Limits::default();
    assert_eq!(l.max_xml_detail_bytes, l.max_input_bytes);

    let at = with_xml_detail(remarks_of_len(l.max_xml_detail_bytes));
    assert!(from_tak_message(&at, &l).is_ok());

    let over = with_xml_detail(remarks_of_len(l.max_xml_detail_bytes + 1));
    assert_eq!(
        p2x_inner_code(&over, &l),
        Some("xml.limit.xml_detail_bytes")
    );
}

#[test]
fn p2x_xml_detail_cap_does_not_depend_on_max_input_bytes() {
    // The xmlDetail cap alone bounds the string; max_input_bytes bounds XML
    // documents received as XML.
    let l = Limits {
        max_input_bytes: 64,
        max_xml_detail_bytes: 128,
        ..Limits::default()
    };
    let at = with_xml_detail(remarks_of_len(128));
    assert!(from_tak_message(&at, &l).is_ok());
    let over = with_xml_detail(remarks_of_len(129));
    assert_eq!(
        p2x_inner_code(&over, &l),
        Some("xml.limit.xml_detail_bytes")
    );
}

// ---- xmlDetail size: XML -> proto ----

#[test]
fn x2p_xml_detail_output_exactly_at_the_cap_accepted_one_byte_less_refused() {
    let remaining = r#"<remarks a="1">hello</remarks>"#;
    let xml = format!(r#"<event uid="u"><detail>{remaining}{remaining}</detail></event>"#);
    let len = 2 * remaining.len();

    let at = Limits {
        max_xml_detail_bytes: len,
        ..Limits::default()
    };
    let ev = CotEvent::parse(&xml, &at).unwrap();
    let msg = to_tak_message(&ev, &ToProtoOptions::default()).unwrap();
    let xml_detail = msg.cot_event.unwrap().detail.unwrap().xml_detail;
    assert_eq!(xml_detail.len(), len);

    let under = Limits {
        max_xml_detail_bytes: len - 1,
        ..Limits::default()
    };
    let ev = CotEvent::parse(&xml, &under).unwrap();
    let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
    assert_eq!(err, ConvertError::XmlDetailTooLarge);
}

// ---- event limits apply to elements copied from xmlDetail ----

fn nested(depth: usize) -> String {
    format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth))
}

#[test]
fn p2x_xml_detail_depth_counts_from_the_event_root() {
    // event (1) / detail (2) / copied elements from depth 3: 62 levels of
    // xmlDetail reach depth 64 (accepted), 63 reach 65 (refused), although
    // the wrapper document itself is only 63 or 64 deep.
    let l = Limits::default();
    assert!(from_tak_message(&with_xml_detail(nested(62)), &l).is_ok());
    assert_eq!(
        p2x_inner_code(&with_xml_detail(nested(63)), &l),
        Some("xml.limit.depth")
    );
    // A million-deep xmlDetail fails on depth, without stack use.
    let deep = "<a>".repeat(1_000_000);
    assert_eq!(
        p2x_inner_code(&with_xml_detail(deep), &l),
        Some("xml.limit.depth")
    );
}

#[test]
fn p2x_xml_detail_attribute_name_and_value_limits() {
    let l = Limits::default();
    let attrs = |n: usize| -> String {
        (0..n).fold(String::new(), |mut out, i| {
            let _ = write!(out, " a{i}='v'");
            out
        })
    };
    assert!(from_tak_message(&with_xml_detail(format!("<x{}/>", attrs(256))), &l).is_ok());
    assert_eq!(
        p2x_inner_code(&with_xml_detail(format!("<x{}/>", attrs(257))), &l),
        Some("xml.limit.attributes")
    );

    let name = "n".repeat(1024);
    assert!(from_tak_message(&with_xml_detail(format!("<{name}/>")), &l).is_ok());
    let name = "n".repeat(1025);
    assert_eq!(
        p2x_inner_code(&with_xml_detail(format!("<{name}/>")), &l),
        Some("xml.limit.name_bytes")
    );

    let value = "v".repeat(1024 * 1024);
    assert!(from_tak_message(&with_xml_detail(format!("<x b='{value}'/>")), &l).is_ok());
    let value = "v".repeat(1024 * 1024 + 1);
    assert_eq!(
        p2x_inner_code(&with_xml_detail(format!("<x b='{value}'/>")), &l),
        Some("xml.limit.attr_value_bytes")
    );
}

#[test]
fn p2x_xml_detail_nodes_count_against_the_event() {
    // The P2X event has event, point, detail (3 nodes) before the copy; a
    // small limit shows the copied nodes are counted in the event as well.
    let l = Limits {
        max_nodes: 6,
        ..Limits::default()
    };
    // Wrapper: detail + 3 children = 4 nodes (fits); event: 3 + 3 = 6.
    assert!(from_tak_message(&with_xml_detail("<a/><b/><c/>".into()), &l).is_ok());
    // Wrapper: 5 nodes (fits); event: 3 + 4 = 7.
    assert_eq!(
        p2x_inner_code(&with_xml_detail("<a/><b/><c/><d/>".into()), &l),
        Some("xml.limit.nodes")
    );
}

// ---- names of namespace declarations ----

#[test]
fn namespace_declaration_names_count_toward_max_name_bytes() {
    // `xmlns:` plus the prefix is the attribute name measured against
    // max_name_bytes (1024): 1024 accepted, 1025 refused.
    let l = Limits::default();
    let decl = |total: usize| format!("<a xmlns:{}='u'/>", "p".repeat(total - 6));
    assert!(Document::parse(&decl(1024), &l).is_ok());
    assert_eq!(
        Document::parse(&decl(1025), &l).unwrap_err().kind(),
        XmlErrorKind::LimitExceeded(LimitKind::NameBytes)
    );
}

// ---- hostile xmlDetail content ----

#[test]
fn p2x_xml_detail_hostile_constructs_rejected() {
    let l = Limits::default();
    for (input, code) in [
        // Entity expansion and external entities: any DOCTYPE is refused.
        (
            "<a/><!DOCTYPE lolz [<!ENTITY lol \"lol\"><!ENTITY lol2 \"&lol;&lol;\">]>",
            "xml.doctype",
        ),
        (
            "<!DOCTYPE a SYSTEM \"file:///etc/passwd\"><a/>",
            "xml.doctype",
        ),
        ("<a>&xxe;</a>", "xml.undefined_entity"),
        // Closing the wrapper and opening a second root.
        ("</detail><event/><detail>", "xml.multiple_roots"),
        ("</detail>", "xml.mismatched_end_tag"),
        // A declaration inside the wrapper.
        ("<?xml version=\"1.1\"?><a/>", "xml.declaration"),
        // Content XML 1.0 cannot carry.
        ("<a>\u{1}</a>", "xml.invalid_char"),
        ("<a>&#0;</a>", "xml.invalid_char_ref"),
        ("<a b='1' b='2'/>", "xml.duplicate_attribute"),
        ("<p:a/>", "xml.undeclared_prefix"),
    ] {
        assert_eq!(
            p2x_inner_code(&with_xml_detail(input.into()), &l),
            Some(code),
            "{input}"
        );
    }
}

// ---- no input content in errors (SI-11) ----

/// One rejected input per XML error kind and per parser limit kind, each
/// carrying `MARK` in names, values or text, with the limits it is parsed
/// under and the kind it must produce.
fn xml_error_cases() -> Vec<(String, Limits, XmlErrorKind)> {
    let m = MARK;
    let d = Limits::default();
    vec![
        (format!("<{m} b='1'c='2'/>"), d, XmlErrorKind::Syntax),
        (format!("<!DOCTYPE {m}><{m}/>"), d, XmlErrorKind::Doctype),
        (
            format!("<?xml version='1.0' {m}='x'?><a/>"),
            d,
            XmlErrorKind::XmlDeclaration,
        ),
        (
            format!("<?xml version='1.1'?><{m}/>"),
            d,
            XmlErrorKind::UnsupportedVersion,
        ),
        (format!("<a>&{m};</a>"), d, XmlErrorKind::UndefinedEntity),
        (format!("<{m}>&#0;</{m}>"), d, XmlErrorKind::InvalidCharRef),
        (format!("<{m}>{m}\u{1}</{m}>"), d, XmlErrorKind::InvalidChar),
        (format!("<{m}:b:c/>"), d, XmlErrorKind::InvalidName),
        (
            format!("<{m}><?XmL {m}?></{m}>"),
            d,
            XmlErrorKind::ReservedPiTarget,
        ),
        (format!("<{m}:x/>"), d, XmlErrorKind::UndeclaredPrefix),
        (
            format!("<a xmlns:{m}=''/>"),
            d,
            XmlErrorKind::InvalidNamespaceDecl,
        ),
        (
            format!("<a {m}='1' {m}='2'/>"),
            d,
            XmlErrorKind::DuplicateAttribute,
        ),
        (format!("<{m}></{m}x>"), d, XmlErrorKind::MismatchedEndTag),
        (format!("<{m}>{m}"), d, XmlErrorKind::UnexpectedEof),
        (format!("<!--{m}-->"), d, XmlErrorKind::NoRoot),
        (format!("<{m}/><{m}/>"), d, XmlErrorKind::MultipleRoots),
        (format!("<a/>{m}"), d, XmlErrorKind::ContentOutsideRoot),
        (
            format!("<{m}>{m}</{m}>"),
            Limits {
                max_input_bytes: 8,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::InputBytes),
        ),
        (
            format!("<{m}><{m}/></{m}>"),
            Limits { max_depth: 1, ..d },
            XmlErrorKind::LimitExceeded(LimitKind::Depth),
        ),
        (
            format!("<{m}><{m}/><{m}/></{m}>"),
            Limits { max_nodes: 2, ..d },
            XmlErrorKind::LimitExceeded(LimitKind::Nodes),
        ),
        (
            format!("<a {m}='{m}' b='{m}'/>"),
            Limits {
                max_attributes: 1,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::Attributes),
        ),
        (
            format!("<a xmlns:{m}='u:{m}' xmlns:b='u:{m}'/>"),
            Limits {
                max_namespace_decls: 1,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::NamespaceDecls),
        ),
        (
            format!("<{m}{m}/>"),
            Limits {
                max_name_bytes: 8,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::NameBytes),
        ),
        (
            format!("<a b='{m}{m}'/>"),
            Limits {
                max_attr_value_bytes: 8,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::AttrValueBytes),
        ),
        (
            format!("<a>{m}{m}</a>"),
            Limits {
                max_text_bytes: 8,
                ..d
            },
            XmlErrorKind::LimitExceeded(LimitKind::TextBytes),
        ),
    ]
}

#[test]
fn every_xml_error_kind_is_free_of_input_content() {
    let cases = xml_error_cases();
    for (input, limits, expected) in &cases {
        let err = Document::parse(input, limits).unwrap_err();
        assert_eq!(err.kind(), *expected, "{input}");
        let text = all_text(&err);
        assert!(!text.contains(MARK), "{expected:?}: {text}");
        assert!(text.contains(expected.as_str()), "{text}");

        // The same through CotEvent::parse (the Phase 3 entry point).
        let err = CotEvent::parse(input, limits).unwrap_err();
        assert_eq!(err.kind().as_str(), expected.as_str());
        let text = all_text(&err);
        assert!(!text.contains(MARK), "{expected:?}: {text}");

        // And through a proto xmlDetail, where the input is reachable.
        let msg = with_xml_detail(input.clone());
        if let Err(err) = from_tak_message(&msg, limits) {
            let text = all_text(&err);
            assert!(!text.contains(MARK), "xmlDetail {expected:?}: {text}");
        }
    }
}

#[test]
fn every_conversion_error_kind_is_free_of_input_content() {
    let m = MARK;
    let d = Limits::default();

    // convert.time.{time,start,stale}
    for attr in ["time", "start", "stale"] {
        let xml = format!(r#"<event uid="{m}" {attr}="{m}"/>"#);
        let ev = CotEvent::parse(&xml, &d).unwrap();
        let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
        assert_eq!(err.kind().as_str(), format!("convert.time.{attr}"));
        let text = all_text(&err);
        assert!(!text.contains(m), "{text}");
    }

    // convert.xml_detail_too_large
    let xml = format!(r#"<event uid="{m}"><detail><{m}>{m}</{m}></detail></event>"#);
    let ev = CotEvent::parse(
        &xml,
        &Limits {
            max_xml_detail_bytes: 4,
            ..d
        },
    )
    .unwrap();
    let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
    assert_eq!(err, ConvertError::XmlDetailTooLarge);
    assert!(!all_text(&err).contains(m));

    // convert.build: a proto string XML 1.0 cannot carry.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            uid: format!("{m}\u{0}{m}"),
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let err = from_tak_message(&msg, &d).unwrap_err();
    assert_eq!(err.kind(), ConvertErrorKind::Build);
    let text = all_text(&err);
    assert!(!text.contains(m), "{text}");

    // convert.xml_detail over the cap.
    let msg = with_xml_detail(format!("<{m}>{m}</{m}>"));
    let err = from_tak_message(
        &msg,
        &Limits {
            max_xml_detail_bytes: 4,
            ..d
        },
    )
    .unwrap_err();
    assert_eq!(err.kind(), ConvertErrorKind::XmlDetail);
    assert!(!all_text(&err).contains(m));
}

#[test]
fn cot_event_and_element_debug_output_is_content_free() {
    let xml = format!(
        r#"<event uid="{MARK}" type="{MARK}"><point lat="1"/><detail><contact callsign="{MARK}"/><remarks>{MARK}</remarks></detail></event>"#
    );
    let ev = CotEvent::parse(&xml, &Limits::default()).unwrap();
    let detail = ev.document().root().first_child_element("detail").unwrap();
    let text = format!("{ev:?} {:?} {:?}", ev.document(), detail);
    assert!(!text.contains(MARK), "{text}");
    // SEC-006: attribute names can carry data too.
    let named = format!(r#"<event {MARK}="v" uid="u"/>"#);
    let ev = CotEvent::parse(&named, &Limits::default()).unwrap();
    let attrs: Vec<String> = ev
        .document()
        .root()
        .attributes()
        .map(|a| format!("{a:?}"))
        .collect();
    assert_eq!(attrs.len(), 2);
    assert!(attrs.iter().all(|t| !t.contains(MARK)), "{attrs:?}");
    // ToProtoOptions and Limits hold no input.
    let _ = format!("{:?} {:?}", ToProtoOptions::default(), Limits::default());
}
