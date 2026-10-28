// SPDX-License-Identifier: Apache-2.0

//! Every `Limits` bound at the limit (accepted) and one past it (refused),
//! with the default values of plan section 3.4 where the input stays small,
//! and lowered values otherwise (parity-map DEV-01a; plan sections 5.2,
//! 8.1).
//!
//! The deep-nesting and namespace-heavy cases guard against stack use and
//! quadratic work: they finish in well under a second together, so a
//! regression shows as a hung test. They do not time themselves, because
//! the crate's clippy configuration bans `Instant::now` (it panics on
//! wasm32).

use std::fmt::Write as _;

use tak_cot::xml::Document;
use tak_cot::{LimitKind, Limits, XmlErrorKind};

fn limit_hit(input: &str, limits: &Limits) -> Option<LimitKind> {
    match Document::parse(input, limits).err()?.kind() {
        XmlErrorKind::LimitExceeded(kind) => Some(kind),
        _ => None,
    }
}

fn accepts(input: &str, limits: &Limits) -> bool {
    Document::parse(input, limits).is_ok()
}

fn nested(depth: usize) -> String {
    format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth))
}

#[test]
fn defaults_are_the_approved_values() {
    let l = Limits::default();
    assert_eq!(l.max_input_bytes, 8 * 1024 * 1024);
    assert_eq!(l.max_depth, 64);
    assert_eq!(l.max_nodes, 100_000);
    assert_eq!(l.max_attributes, 256);
    assert_eq!(l.max_namespace_decls, 32);
    assert_eq!(l.max_name_bytes, 1024);
    assert_eq!(l.max_attr_value_bytes, 1024 * 1024);
    assert_eq!(l.max_text_bytes, 8 * 1024 * 1024);
    assert_eq!(l.max_xml_detail_bytes, 8 * 1024 * 1024);
}

#[test]
fn input_bytes() {
    let l = Limits::default();
    // Exactly 8 MiB: one element of text.
    let fill = l.max_input_bytes - "<a></a>".len();
    let at_limit = format!("<a>{}</a>", "x".repeat(fill));
    assert_eq!(at_limit.len(), l.max_input_bytes);
    assert!(accepts(&at_limit, &l));
    let over = format!("<a>{}</a>", "x".repeat(fill + 1));
    let err = Document::parse(&over, &l).unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::InputBytes)
    );
    assert_eq!(err.offset(), Some(l.max_input_bytes));
}

#[test]
fn depth_64_accepted_65_refused() {
    let l = Limits::default();
    assert!(accepts(&nested(64), &l));
    assert_eq!(limit_hit(&nested(65), &l), Some(LimitKind::Depth));
    // An empty element counts at its own depth.
    let deep_empty = format!("{}<b/>{}", "<a>".repeat(64), "</a>".repeat(64));
    assert_eq!(limit_hit(&deep_empty, &l), Some(LimitKind::Depth));
}

#[test]
fn million_deep_nesting_fails_fast_without_stack_use() {
    let input = "<a>".repeat(1_000_000);
    let err = Document::parse(&input, &Limits::default()).unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::LimitExceeded(LimitKind::Depth));
    // The 65th start tag.
    assert_eq!(err.offset(), Some(64 * 3));
}

#[test]
fn nodes_100_000_accepted_100_001_refused() {
    let l = Limits::default();
    // Root plus 99 999 empty children is 100 000 nodes.
    let at_limit = format!("<r>{}</r>", "<b/>".repeat(99_999));
    assert!(accepts(&at_limit, &l));
    let over = format!("<r>{}</r>", "<b/>".repeat(100_000));
    assert_eq!(limit_hit(&over, &l), Some(LimitKind::Nodes));
}

#[test]
fn nodes_count_text_comments_and_pis_inside_and_outside_the_root() {
    let l = Limits {
        max_nodes: 5,
        ..Limits::default()
    };
    // Comment, root, text, CDATA, PI = 5.
    assert!(accepts("<!--c--><r>t<![CDATA[d]]><?p?></r>", &l));
    assert_eq!(
        limit_hit("<!--c--><r>t<![CDATA[d]]><?p?></r><!--e-->", &l),
        Some(LimitKind::Nodes)
    );
    // White space between elements is a text node; outside the root it is
    // not kept.
    assert!(accepts("\n<r> <b/> <c/></r>\n", &l));
    assert_eq!(limit_hit("<r> <b/> <c/> </r>", &l), Some(LimitKind::Nodes));
}

/// `n` pieces of `piece(i)` concatenated.
fn repeat_with(n: usize, piece: impl Fn(&mut String, usize)) -> String {
    let mut out = String::new();
    for i in 0..n {
        piece(&mut out, i);
    }
    out
}

fn attrs(n: usize) -> String {
    repeat_with(n, |out, i| {
        let _ = write!(out, " a{i}='v'");
    })
}

fn decls(n: usize) -> String {
    repeat_with(n, |out, i| {
        let _ = write!(out, " xmlns:p{i}='urn:{i}'");
    })
}

#[test]
fn attributes_256_accepted_257_refused() {
    let l = Limits::default();
    assert!(accepts(&format!("<a{}/>", attrs(256)), &l));
    assert_eq!(
        limit_hit(&format!("<a{}/>", attrs(257)), &l),
        Some(LimitKind::Attributes)
    );
    // Namespace declarations have their own budget.
    assert!(accepts(&format!("<a{}{}/>", attrs(256), decls(32)), &l));
}

#[test]
fn namespace_decls_32_accepted_33_refused() {
    let l = Limits::default();
    assert!(accepts(&format!("<a{}/>", decls(32)), &l));
    assert_eq!(
        limit_hit(&format!("<a{}/>", decls(33)), &l),
        Some(LimitKind::NamespaceDecls)
    );
    assert_eq!(
        limit_hit(&format!("<a xmlns='urn:d'{}/>", decls(32)), &l),
        Some(LimitKind::NamespaceDecls)
    );
}

#[test]
fn name_bytes_1024_accepted_1025_refused() {
    let l = Limits::default();
    let ok = "n".repeat(1024);
    let long = "n".repeat(1025);
    assert!(accepts(&format!("<{ok}/>"), &l));
    assert_eq!(
        limit_hit(&format!("<{long}/>"), &l),
        Some(LimitKind::NameBytes)
    );
    assert!(accepts(&format!("<a {ok}='1'/>"), &l));
    assert_eq!(
        limit_hit(&format!("<a {long}='1'/>"), &l),
        Some(LimitKind::NameBytes)
    );
    assert!(accepts(&format!("<a><?{ok} d?></a>"), &l));
    assert_eq!(
        limit_hit(&format!("<a><?{long} d?></a>"), &l),
        Some(LimitKind::NameBytes)
    );
    // The prefix counts.
    let prefixed = format!("p:{}", "n".repeat(1022));
    assert!(accepts(&format!("<{prefixed} xmlns:p='u'/>"), &l));
    let prefixed = format!("p:{}", "n".repeat(1023));
    assert_eq!(
        limit_hit(&format!("<{prefixed} xmlns:p='u'/>"), &l),
        Some(LimitKind::NameBytes)
    );
}

#[test]
fn attr_value_bytes_1_mib_accepted_plus_one_refused() {
    let l = Limits::default();
    let max = 1024 * 1024;
    assert!(accepts(&format!("<a b='{}'/>", "v".repeat(max)), &l));
    assert_eq!(
        limit_hit(&format!("<a b='{}'/>", "v".repeat(max + 1)), &l),
        Some(LimitKind::AttrValueBytes)
    );
    // Measured after normalisation: each `&amp;` is one byte.
    let small = Limits {
        max_attr_value_bytes: 4,
        ..Limits::default()
    };
    assert!(accepts("<a b='&amp;&amp;&amp;&amp;'/>", &small));
    assert_eq!(
        limit_hit("<a b='&amp;&amp;&amp;&amp;&amp;'/>", &small),
        Some(LimitKind::AttrValueBytes)
    );
    // Namespace names are attribute values too.
    assert_eq!(
        limit_hit("<a xmlns:p='urn:x'/>", &small),
        Some(LimitKind::AttrValueBytes)
    );
}

#[test]
fn text_bytes() {
    let l = Limits {
        max_text_bytes: 8,
        ..Limits::default()
    };
    assert!(accepts("<a>12345678</a>", &l));
    assert_eq!(
        limit_hit("<a>123456789</a>", &l),
        Some(LimitKind::TextBytes)
    );
    // Adjacent text and references form one node.
    assert!(accepts("<a>1234&amp;678</a>", &l));
    assert_eq!(
        limit_hit("<a>1234&amp;6789</a>", &l),
        Some(LimitKind::TextBytes)
    );
    assert_eq!(
        limit_hit("<a>12345678&amp;</a>", &l),
        Some(LimitKind::TextBytes)
    );
    // Separate text nodes each get the budget.
    assert!(accepts("<a>12345678<b/>12345678</a>", &l));
    // Measured after line-end normalisation.
    assert!(accepts("<a>1234567\r\n</a>", &l));
    for over in [
        "<a><![CDATA[123456789]]></a>",
        "<a><!--123456789--></a>",
        "<a><?p 123456789?></a>",
        "<!--123456789--><a/>",
    ] {
        assert_eq!(limit_hit(over, &l), Some(LimitKind::TextBytes), "{over}");
    }
    // Multi-byte characters count in bytes.
    assert!(accepts("<a>\u{1F600}\u{1F600}</a>", &l));
    assert_eq!(
        limit_hit("<a>\u{1F600}\u{1F600}x</a>", &l),
        Some(LimitKind::TextBytes)
    );
}

#[test]
fn zero_limits_refuse_everything_without_panicking() {
    let l = Limits {
        max_input_bytes: 0,
        max_depth: 0,
        max_nodes: 0,
        max_attributes: 0,
        max_namespace_decls: 0,
        max_name_bytes: 0,
        max_attr_value_bytes: 0,
        max_text_bytes: 0,
        max_xml_detail_bytes: 0,
    };
    assert_eq!(limit_hit("<a/>", &l), Some(LimitKind::InputBytes));
    let l = Limits {
        max_input_bytes: 64,
        ..l
    };
    assert_eq!(limit_hit("<a/>", &l), Some(LimitKind::Depth));
    assert!(Document::with_root("a", &l).is_err());
}

#[test]
fn edits_respect_the_document_limits() {
    let l = Limits {
        max_depth: 2,
        max_nodes: 3,
        max_attributes: 1,
        max_name_bytes: 3,
        max_attr_value_bytes: 2,
        max_text_bytes: 2,
        ..Limits::default()
    };
    let mut doc = Document::with_root("r", &l).unwrap();
    let root = doc.root().id();
    let child = doc.append_element(root, "c").unwrap();
    let err = doc.append_element(child, "g").unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::LimitExceeded(LimitKind::Depth));
    let err = doc.append_element(root, "long").unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::NameBytes)
    );
    let err = doc.push_attribute(root, "a", "123").unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::AttrValueBytes)
    );
    doc.push_attribute(root, "a", "12").unwrap();
    let err = doc.push_attribute(root, "b", "1").unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::Attributes)
    );
    let err = doc.append_text(root, "123").unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::TextBytes)
    );
    doc.append_text(root, "12").unwrap();
    let err = doc.append_text(root, "1").unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::LimitExceeded(LimitKind::Nodes));
}

#[test]
fn append_copy_respects_limits_and_leaves_nothing_on_failure() {
    let src = Document::parse("<d><a><b><c/></b></a></d>", &Limits::default()).unwrap();
    let a = src.root().first_child_element("a").unwrap().id();

    let shallow = Limits {
        max_depth: 3,
        ..Limits::default()
    };
    let mut doc = Document::with_root("event", &shallow).unwrap();
    let root = doc.root().id();
    let detail = doc.append_element(root, "detail").unwrap();
    let before = doc.to_xml_bytes();
    let err = doc.append_copy(detail, &src, a).unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::LimitExceeded(LimitKind::Depth));
    assert_eq!(doc.to_xml_bytes(), before);
    // The document is still usable after the rolled-back copy.
    doc.append_element(detail, "ok").unwrap();

    let few = Limits {
        max_nodes: 4,
        ..Limits::default()
    };
    let mut doc = Document::with_root("event", &few).unwrap();
    let root = doc.root().id();
    let detail = doc.append_element(root, "detail").unwrap();
    let err = doc.append_copy(detail, &src, a).unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::LimitExceeded(LimitKind::Nodes));
    let names: Vec<_> = doc.root().child_elements().map(|e| e.name()).collect();
    assert_eq!(names, ["detail"]);
    assert!(!doc.element(detail).unwrap().has_children());
    // Two nodes are free again.
    let x = doc.append_element(detail, "x").unwrap();
    doc.append_element(x, "y").unwrap();
}

#[test]
fn namespace_heavy_input_stays_linear() {
    // 32 declarations on each of 62 nested elements, then elements with 256
    // prefixed attributes resolved against them at depth 64: lookups stay
    // cheap.
    let l = Limits::default();
    let mut input = String::new();
    for _ in 0..62 {
        let _ = write!(input, "<a{}>", decls(32));
    }
    let attrs = repeat_with(256, |out, i| {
        let _ = write!(out, " p0:x{i}='v'");
    });
    let leaves = format!("<b{attrs}/>").repeat(500);
    let _ = write!(input, "<c>{leaves}</c>");
    input.push_str(&"</a>".repeat(62));
    assert!(accepts(&input, &l));
}

#[test]
fn long_equal_namespace_names_do_not_make_duplicate_checks_quadratic() {
    // Two prefixes bound to the same 512 KiB name, and 256 attributes
    // alternating between them on each of many elements.
    let l = Limits::default();
    let uri = "u".repeat(512 * 1024);
    let attrs = repeat_with(256, |out, i| {
        let prefix = if i.is_multiple_of(2) { "p" } else { "q" };
        let _ = write!(out, " {prefix}:x{i}='v'");
    });
    let input = format!(
        "<r xmlns:p='{uri}' xmlns:q='{uri}'>{}</r>",
        format!("<b{attrs}/>").repeat(500)
    );
    assert!(accepts(&input, &l));
    // And the same local name through both prefixes is still a duplicate.
    let dup = format!("<r xmlns:p='{uri}' xmlns:q='{uri}'><b p:x='1' q:x='2'/></r>");
    assert_eq!(
        Document::parse(&dup, &l).unwrap_err().kind(),
        XmlErrorKind::DuplicateAttribute
    );
}
