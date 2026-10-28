// SPDX-License-Identifier: Apache-2.0

//! `tak_cot::xml` well-formedness, normalisation, namespaces, serializer
//! form and document edits (plan sections 1.5, 3.2, 8.1; parity-map K-30,
//! K-32, K-33, DEV-01b, C-01 to C-06, C-11).
//!
//! Serializer expectations here are the XML-default rules of the
//! `serialize` module; the X2X and P2X goldens (WU7, WU8) fix the
//! reference's exact form.

use std::fmt::Write as _;

use tak_cot::xml::{Document, NodeKind};
use tak_cot::{LimitKind, Limits, XmlError, XmlErrorKind};

fn parse(input: &str) -> Result<Document, XmlError> {
    Document::parse(input, &Limits::default())
}

fn kind(input: &str) -> Option<XmlErrorKind> {
    parse(input).err().map(|e| e.kind())
}

fn x2x(input: &str) -> Option<String> {
    String::from_utf8(parse(input).ok()?.to_xml_bytes()).ok()
}

/// The document without the declaration line the serializer writes.
fn body(input: &str) -> Option<String> {
    x2x(input)?
        .strip_prefix("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")
        .map(str::to_owned)
}

fn root_text(doc: &Document) -> Vec<String> {
    doc.root()
        .children()
        .filter_map(|n| match n {
            NodeKind::Text(t) => Some(t.to_owned()),
            _ => None,
        })
        .collect()
}

fn fragment(input: &str, path: &[&str]) -> Option<String> {
    let doc = parse(input).ok()?;
    let mut el = doc.root();
    for name in path {
        el = el.first_child_element(name)?;
    }
    let mut out = Vec::new();
    el.to_xml_fragment(&mut out);
    String::from_utf8(out).ok()
}

const COT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<event version="2.0" uid="ANDROID-1" type="a-f-G-U-C" how="m-g" time="2025-10-27T18:04:58Z" start="2025-10-27T18:04:58Z" stale="2025-10-27T18:10:58Z"><point lat="1.5" lon="-2.25" hae="9999999.0" ce="9999999.0" le="9999999.0"/><detail><contact callsign="A" endpoint="*:-1:stcp"/><__group name="Cyan" role="Team Member"/><remarks>hi &amp; bye</remarks></detail></event>"#;

// ---- accepted documents and the read API ----

#[test]
fn parses_a_cot_event() {
    let doc = parse(COT).unwrap();
    let root = doc.root();
    assert_eq!(root.name(), "event");
    assert_eq!(root.prefix(), None);
    assert_eq!(root.namespace_uri(), None);
    let names: Vec<_> = root.attributes().map(|a| a.name()).collect();
    assert_eq!(
        names,
        ["version", "uid", "type", "how", "time", "start", "stale"]
    );
    assert_eq!(root.attribute("uid"), Some("ANDROID-1"));
    assert_eq!(root.attribute("missing"), None);
    let detail = root.first_child_element("detail").unwrap();
    assert_eq!(detail.parent().unwrap().id(), root.id());
    assert!(root.parent().is_none());
    let children: Vec<_> = detail.child_elements().map(|e| e.name()).collect();
    assert_eq!(children, ["contact", "__group", "remarks"]);
    let remarks = detail.first_child_element("remarks").unwrap();
    assert_eq!(root_text_of(&remarks), "hi & bye");
    let decl = doc.declaration().unwrap();
    assert_eq!(decl.encoding(), Some("UTF-8"));
    assert_eq!(decl.standalone(), Some(true));
    assert_eq!(doc.element(detail.id()).unwrap().name(), "detail");
    assert!(doc.node(detail.id()).is_some());
}

fn root_text_of(el: &tak_cot::xml::ElementRef<'_>) -> String {
    el.children()
        .filter_map(|n| match n {
            NodeKind::Text(t) => Some(t),
            _ => None,
        })
        .collect()
}

#[test]
fn declaration_is_optional_and_all_forms_parse() {
    assert!(parse("<a/>").unwrap().declaration().is_none());
    for ok in [
        "<?xml version='1.0'?><a/>",
        "<?xml version=\"1.0\" encoding='utf-8'?><a/>",
        "<?xml version = \"1.0\"  standalone = 'no' ?>\n<a/>",
        "<?xml version=\"1.0\" encoding=\"ISO-8859-1\" standalone=\"yes\"?><a/>",
    ] {
        assert!(parse(ok).is_ok(), "{ok}");
    }
}

#[test]
fn byte_order_mark_is_accepted_once() {
    assert!(parse("\u{FEFF}<?xml version=\"1.0\"?><a/>").is_ok());
    assert_eq!(
        kind("\u{FEFF}\u{FEFF}<a/>"),
        Some(XmlErrorKind::ContentOutsideRoot)
    );
    // Offsets count the mark.
    let err = parse("\u{FEFF}<a>&bogus;</a>").unwrap_err();
    assert_eq!(err.offset(), Some(6));
}

#[test]
fn top_level_comments_and_pis_are_kept() {
    let doc = parse("<!--c1--><?p d?>\n<a/>\n<!--c2-->").unwrap();
    let kinds: Vec<_> = doc
        .top_level()
        .map(|n| match n {
            NodeKind::Comment(c) => format!("comment:{c}"),
            NodeKind::ProcessingInstruction { target, data } => format!("pi:{target}:{data}"),
            NodeKind::Element(e) => format!("element:{}", e.name()),
            NodeKind::Text(_) | NodeKind::CData(_) => "other".to_owned(),
        })
        .collect();
    assert_eq!(kinds, ["comment:c1", "pi:p:d", "element:a", "comment:c2"]);
}

// ---- refused: DOCTYPE and entities (K-30) ----

#[test]
fn doctype_is_refused_anywhere() {
    for input in [
        "<!DOCTYPE a><a/>",
        "<?xml version=\"1.0\"?><!DOCTYPE a SYSTEM \"file:///etc/passwd\"><a/>",
        "<!DOCTYPE a [<!ENTITY x \"y\">]><a>&x;</a>",
        "<!doctype a><a/>",
        "<a/><!DOCTYPE a>",
        "<!DOCTYPE a [",
    ] {
        assert_eq!(kind(input), Some(XmlErrorKind::Doctype), "{input}");
    }
}

#[test]
fn billion_laughs_is_refused_at_the_doctype() {
    let mut input = String::from("<?xml version=\"1.0\"?><!DOCTYPE lolz [<!ENTITY lol \"lol\">");
    for i in 1..10 {
        let prev = if i == 1 {
            "lol".to_owned()
        } else {
            format!("lol{}", i - 1)
        };
        let _ = write!(
            input,
            "<!ENTITY lol{i} \"{}\">",
            format!("&{prev};").repeat(10)
        );
    }
    input.push_str("]><lolz>&lol9;</lolz>");
    let err = parse(&input).unwrap_err();
    assert_eq!(err.kind(), XmlErrorKind::Doctype);
    assert_eq!(err.offset(), Some(21));
}

#[test]
fn only_predefined_entities_and_char_refs() {
    let doc = parse("<a b='&lt;&gt;&amp;&apos;&quot;&#65;&#x42;'>&lt;&#x1F600;</a>").unwrap();
    assert_eq!(doc.root().attribute("b"), Some("<>&'\"AB"));
    assert_eq!(root_text(&doc), ["<\u{1F600}"]);
    assert_eq!(kind("<a>&nbsp;</a>"), Some(XmlErrorKind::UndefinedEntity));
    assert_eq!(kind("<a b='&nbsp;'/>"), Some(XmlErrorKind::UndefinedEntity));
    assert_eq!(kind("<a>&#0;</a>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(kind("<a b='&#0;'/>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(kind("<a>&#xFFFE;</a>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(kind("<a>&#xD800;</a>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(
        kind("<a>&#x110000;</a>"),
        Some(XmlErrorKind::InvalidCharRef)
    );
    assert_eq!(kind("<a>&#1;</a>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(kind("<a>&#xg;</a>"), Some(XmlErrorKind::InvalidCharRef));
    assert_eq!(kind("<a>& b</a>"), Some(XmlErrorKind::Syntax));
    assert_eq!(kind("<a>&lt</a>"), Some(XmlErrorKind::Syntax));
    assert_eq!(kind("<a b='&'/>"), Some(XmlErrorKind::Syntax));
}

// ---- characters and names ----

#[test]
fn non_xml_chars_are_refused() {
    for input in [
        "<a>\u{1}</a>",
        "<a>\u{0}</a>",
        "<a b='\u{1F}'/>",
        "<a><!--\u{2}--></a>",
        "<a><![CDATA[\u{3}]]></a>",
        "<a><?p \u{4}?></a>",
        "<a>\u{FFFF}</a>",
    ] {
        assert_eq!(kind(input), Some(XmlErrorKind::InvalidChar), "{input:?}");
    }
    assert!(parse("<a b='\u{FFFD}\u{10FFFF}'>\u{D7FF}\u{E000}</a>").is_ok());
}

#[test]
fn invalid_names_are_refused() {
    for input in [
        "<1a/>",
        "<a 1b='x'/>",
        "<a:b:c/>",
        "<a :b='x'/>",
        "<a b:='x'/>",
        "<-a/>",
        "<a/b/>",
        "<a / >",
        "<xmlns:a/>",
        "<a><?a:b x?></a>",
        "<a><??></a>",
    ] {
        assert_eq!(kind(input), Some(XmlErrorKind::InvalidName), "{input}");
    }
    assert!(parse("<\u{e9}t\u{e9} _x.y-z='1'/>").is_ok());
}

// ---- namespaces (K-33, C-06) ----

#[test]
fn prefixes_must_be_declared() {
    assert_eq!(kind("<x:a/>"), Some(XmlErrorKind::UndeclaredPrefix));
    assert_eq!(kind("<a x:b='1'/>"), Some(XmlErrorKind::UndeclaredPrefix));
    assert_eq!(kind("<a xmlns:x='u'/><!-- -->"), None);
    assert_eq!(
        kind("<a><b xmlns:x='u'/><x:c/></a>"),
        Some(XmlErrorKind::UndeclaredPrefix)
    );
    // Declarations apply to the element's own name and attributes, in any
    // order, and `xml` is always bound.
    assert!(parse("<x:a x:b='1' xmlns:x='urn:x' xml:lang='en'/>").is_ok());
}

#[test]
fn namespace_resolution() {
    let doc = parse("<a xmlns='urn:d' xmlns:x='urn:x'><b x:c='1' d='2'/><x:e/><f xmlns=''/></a>")
        .unwrap();
    let a = doc.root();
    assert_eq!(a.namespace_uri(), Some("urn:d"));
    let decls: Vec<_> = a.namespace_decls().map(|d| (d.prefix(), d.uri())).collect();
    assert_eq!(decls, [(None, "urn:d"), (Some("x"), "urn:x")]);
    // Declarations are not attributes.
    assert_eq!(a.attributes().len(), 0);
    let b = a.first_child_element("b").unwrap();
    assert_eq!(b.namespace_uri(), Some("urn:d"));
    let attrs: Vec<_> = b
        .attributes()
        .map(|at| {
            (
                at.name(),
                at.prefix(),
                at.local_name(),
                at.namespace_uri(),
                at.value(),
            )
        })
        .collect();
    assert_eq!(
        attrs,
        [
            ("x:c", Some("x"), "c", Some("urn:x"), "1"),
            ("d", None, "d", None, "2")
        ]
    );
    // Local-name lookups ignore the prefix (C-06 expectation).
    assert_eq!(b.attribute("c"), Some("1"));
    let e = a.first_child_element("e").unwrap();
    assert_eq!(
        (e.name(), e.prefix(), e.local_name()),
        ("x:e", Some("x"), "e")
    );
    assert_eq!(e.namespace_uri(), Some("urn:x"));
    assert_eq!(a.first_child_element("f").unwrap().namespace_uri(), None);
}

#[test]
fn forbidden_namespace_declarations() {
    for input in [
        "<a xmlns:x=''/>",
        "<a xmlns:xmlns='urn:x'/>",
        "<a xmlns:xml='urn:x'/>",
        "<a xmlns:x='http://www.w3.org/XML/1998/namespace'/>",
        "<a xmlns:x='http://www.w3.org/2000/xmlns/'/>",
        "<a xmlns='http://www.w3.org/2000/xmlns/'/>",
    ] {
        assert_eq!(
            kind(input),
            Some(XmlErrorKind::InvalidNamespaceDecl),
            "{input}"
        );
    }
    assert!(parse("<a xmlns:xml='http://www.w3.org/XML/1998/namespace'/>").is_ok());
}

#[test]
fn duplicate_attributes() {
    assert_eq!(
        kind("<a b='1' b='2'/>"),
        Some(XmlErrorKind::DuplicateAttribute)
    );
    assert_eq!(
        kind("<a xmlns:x='u' xmlns:x='v'/>"),
        Some(XmlErrorKind::DuplicateAttribute)
    );
    // Same namespace and local name through two prefixes.
    assert_eq!(
        kind("<a xmlns:p='u' xmlns:q='u' p:b='1' q:b='2'/>"),
        Some(XmlErrorKind::DuplicateAttribute)
    );
    assert!(parse("<a xmlns:p='u' xmlns:q='v' p:b='1' q:b='2' b='3'/>").is_ok());
}

// ---- structure ----

#[test]
fn structural_errors() {
    assert_eq!(kind("<a><b></a>"), Some(XmlErrorKind::MismatchedEndTag));
    assert_eq!(kind("<a></b>"), Some(XmlErrorKind::MismatchedEndTag));
    assert_eq!(kind("</a>"), Some(XmlErrorKind::MismatchedEndTag));
    assert_eq!(kind("<a></a></a>"), Some(XmlErrorKind::MismatchedEndTag));
    assert_eq!(kind("<a><b>"), Some(XmlErrorKind::UnexpectedEof));
    assert_eq!(kind("<a/><b/>"), Some(XmlErrorKind::MultipleRoots));
    assert_eq!(kind("<a/>text"), Some(XmlErrorKind::ContentOutsideRoot));
    assert_eq!(kind("text<a/>"), Some(XmlErrorKind::ContentOutsideRoot));
    assert_eq!(kind("<a/>&amp;"), Some(XmlErrorKind::ContentOutsideRoot));
    assert_eq!(
        kind("<![CDATA[x]]><a/>"),
        Some(XmlErrorKind::ContentOutsideRoot)
    );
    assert_eq!(kind(""), Some(XmlErrorKind::NoRoot));
    assert_eq!(kind("  <!-- only -->  "), Some(XmlErrorKind::NoRoot));
    assert!(parse("\n\t <a></a >\r\n").is_ok());
}

#[test]
fn syntax_errors() {
    for input in [
        "<a b='1'c='2'/>",
        "<a b/>",
        "<a b=c/>",
        "<a b='<'/>",
        "<a b=\"1' />",
        "<a>]]></a>",
        "<a><!-- x -- y --></a>",
        "<a><!-- x ---></a>",
        "<a",
        "<a><!-- x",
        "<a><![CDATA[x</a>",
    ] {
        assert_eq!(kind(input), Some(XmlErrorKind::Syntax), "{input}");
    }
}

#[test]
fn xml_declaration_rules() {
    assert_eq!(
        kind("<?xml version=\"1.1\"?><a/>"),
        Some(XmlErrorKind::UnsupportedVersion)
    );
    assert_eq!(
        kind("<?xml version='1.9'?><a/>"),
        Some(XmlErrorKind::UnsupportedVersion)
    );
    for input in [
        "<?xml version='2.0'?><a/>",
        "<?xml?><a/>",
        "<?xml encoding='UTF-8'?><a/>",
        "<?xml version='1.0' standalone='yes' encoding='UTF-8'?><a/>",
        "<?xml version='1.0' standalone='maybe'?><a/>",
        "<?xml version='1.0' encoding='8bit'?><a/>",
        "<?xml version='1.0'encoding='UTF-8'?><a/>",
        "<?xml version='1.0' foo='bar'?><a/>",
        " <?xml version='1.0'?><a/>",
        "<!-- c --><?xml version='1.0'?><a/>",
        "<a/><?xml version='1.0'?>",
    ] {
        assert_eq!(kind(input), Some(XmlErrorKind::XmlDeclaration), "{input}");
    }
    assert_eq!(
        kind("<a><?XmL x?></a>"),
        Some(XmlErrorKind::ReservedPiTarget)
    );
    assert!(parse("<a><?xml-stylesheet href='s'?></a>").is_ok());
}

// ---- normalisation ----

#[test]
fn line_ends_are_normalised() {
    let doc = parse("<a b='x'>1\r\n2\r3\n4<!--c\r\nd--><![CDATA[e\rf]]><?p g\r\nh?></a>").unwrap();
    let parts: Vec<_> = doc
        .root()
        .children()
        .map(|n| match n {
            NodeKind::Text(t) | NodeKind::Comment(t) | NodeKind::CData(t) => t.to_owned(),
            NodeKind::ProcessingInstruction { data, .. } => data.to_owned(),
            NodeKind::Element(_) => String::new(),
        })
        .collect();
    assert_eq!(parts, ["1\n2\n3\n4", "c\nd", "e\nf", "g\nh"]);
}

#[test]
fn attribute_values_are_normalised() {
    let doc = parse("<a b='x\ty\nz\r\nw\rv' c='&#9;&#10;&#13;' d='  two  spaces  '/>").unwrap();
    let a = doc.root();
    assert_eq!(a.attribute("b"), Some("x y z w v"));
    // Character references keep the character (`&#10;` preserved).
    assert_eq!(a.attribute("c"), Some("\t\n\r"));
    // CDATA-type attributes keep their spaces (no DTD, no collapsing).
    assert_eq!(a.attribute("d"), Some("  two  spaces  "));
}

#[test]
fn adjacent_text_and_references_merge() {
    let doc = parse("<a>x&amp;y&#10;z<b/>w</a>").unwrap();
    assert_eq!(root_text(&doc), ["x&y\nz", "w"]);
    let doc = parse("<a>  <b/>\n</a>").unwrap();
    assert_eq!(root_text(&doc), ["  ", "\n"]);
}

// ---- serializer (K-32; C-01 to C-05, C-11) ----

#[test]
fn serializer_writes_declaration_and_round_trips_cot() {
    let out = x2x(COT).unwrap();
    assert!(out.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<event version=\"2.0\""));
    assert!(out.contains("<point lat=\"1.5\" lon=\"-2.25\""));
    assert!(out.contains("<contact callsign=\"A\" endpoint=\"*:-1:stcp\"/>"));
    assert!(out.contains("<remarks>hi &amp; bye</remarks>"));
    assert_eq!(x2x(&out).unwrap(), out);
}

#[test]
fn serializer_escapes() {
    assert_eq!(
        body("<a b='&lt;&gt;&amp;&quot;&apos;&#9;&#10;&#13; '>&lt;&gt;&amp;\"'\t\n&#13;</a>")
            .unwrap(),
        "<a b=\"&lt;&gt;&amp;&quot;'&#9;&#10;&#13; \">&lt;&gt;&amp;\"'\t\n&#13;</a>"
    );
    // Non-ASCII stays UTF-8.
    assert_eq!(
        body("<a b='&#xe9;'>&#x1F600;</a>").unwrap(),
        "<a b=\"\u{e9}\">\u{1F600}</a>"
    );
}

#[test]
fn serializer_node_forms() {
    assert_eq!(body("<a></a>").unwrap(), "<a/>");
    assert_eq!(body("<a> </a>").unwrap(), "<a> </a>");
    assert_eq!(
        body("<!--x--><?p  d ?><a><![CDATA[<&>]]><!--c--><?q?></a><!--y-->").unwrap(),
        "<!--x--><?p d ?><a><![CDATA[<&>]]><!--c--><?q?></a><!--y-->"
    );
    // Attribute order is kept (C-11).
    assert_eq!(
        body("<a z='1' a='2' m='3'/>").unwrap(),
        "<a z=\"1\" a=\"2\" m=\"3\"/>"
    );
    // The input declaration does not change the written one.
    assert!(
        x2x("<?xml version='1.0' encoding='ISO-8859-1'?><a/>")
            .unwrap()
            .starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<a/>")
    );
}

#[test]
fn serializer_namespaces() {
    let input = "<x:a xmlns:x='urn:x' xmlns='urn:d'><b x:c='1'/></x:a>";
    assert_eq!(
        body(input).unwrap(),
        "<x:a xmlns:x=\"urn:x\" xmlns=\"urn:d\"><b x:c=\"1\"/></x:a>"
    );
}

#[test]
fn fragments_declare_what_they_use() {
    let input = "<event xmlns:x='urn:x' xmlns='urn:d'><detail><x:a x:b='1'><c/></x:a><e xmlns=''/><f xml:lang='en'/></detail></event>";
    assert_eq!(
        fragment(input, &["detail", "a"]).unwrap(),
        "<x:a xmlns:x=\"urn:x\" x:b=\"1\"><c xmlns=\"urn:d\"/></x:a>"
    );
    assert_eq!(
        fragment(input, &["detail", "e"]).unwrap(),
        "<e xmlns=\"\"/>"
    );
    assert_eq!(
        fragment(input, &["detail", "f"]).unwrap(),
        "<f xmlns=\"urn:d\" xml:lang=\"en\"/>"
    );
    // Without namespaces a fragment is just the element.
    assert_eq!(
        fragment(COT, &["detail", "contact"]).unwrap(),
        "<contact callsign=\"A\" endpoint=\"*:-1:stcp\"/>"
    );
    // Every fragment re-parses on its own.
    for path in [&["detail", "a"][..], &["detail", "e"], &["detail", "f"]] {
        assert!(parse(&fragment(input, path).unwrap()).is_ok());
    }
}

#[test]
fn bounded_fragment() {
    let doc = parse(COT).unwrap();
    let contact = doc
        .root()
        .first_child_element("detail")
        .and_then(|d| d.first_child_element("contact"))
        .unwrap();
    let mut out = b"prefix".to_vec();
    contact.try_to_xml_fragment(&mut out, 1024).unwrap();
    assert_eq!(
        out,
        b"prefix<contact callsign=\"A\" endpoint=\"*:-1:stcp\"/>"
    );
    let mut out = b"prefix".to_vec();
    let err = contact.try_to_xml_fragment(&mut out, 10).unwrap_err();
    assert_eq!(
        err.kind(),
        XmlErrorKind::LimitExceeded(LimitKind::XmlDetailBytes)
    );
    assert_eq!(err.offset(), None);
    assert_eq!(out, b"prefix");
}

// ---- errors (SI-11) ----

#[test]
fn errors_never_quote_input() {
    for input in [
        "<secret><b></secret>",
        "<secret attr='&secret;'/>",
        "<secret xmlns:secret=''/>",
        "<secret:x/>",
    ] {
        let err = parse(input).unwrap_err();
        let text = format!("{err} {err:?}");
        assert!(!text.contains("secret"), "{text}");
        assert!(text.contains(err.kind().as_str()), "{text}");
    }
    let err = parse("<a>").unwrap_err();
    assert_eq!(
        err.to_string(),
        "unexpected end of input inside an element (xml.unexpected_eof) at byte 3"
    );
}

#[test]
fn error_codes_are_stable() {
    let codes = [
        (XmlErrorKind::Syntax, "xml.syntax"),
        (XmlErrorKind::Doctype, "xml.doctype"),
        (XmlErrorKind::XmlDeclaration, "xml.declaration"),
        (XmlErrorKind::UnsupportedVersion, "xml.unsupported_version"),
        (XmlErrorKind::UndefinedEntity, "xml.undefined_entity"),
        (XmlErrorKind::InvalidCharRef, "xml.invalid_char_ref"),
        (XmlErrorKind::InvalidChar, "xml.invalid_char"),
        (XmlErrorKind::InvalidName, "xml.invalid_name"),
        (XmlErrorKind::ReservedPiTarget, "xml.reserved_pi_target"),
        (XmlErrorKind::UndeclaredPrefix, "xml.undeclared_prefix"),
        (
            XmlErrorKind::InvalidNamespaceDecl,
            "xml.invalid_namespace_decl",
        ),
        (XmlErrorKind::DuplicateAttribute, "xml.duplicate_attribute"),
        (XmlErrorKind::MismatchedEndTag, "xml.mismatched_end_tag"),
        (XmlErrorKind::UnexpectedEof, "xml.unexpected_eof"),
        (XmlErrorKind::NoRoot, "xml.no_root"),
        (XmlErrorKind::MultipleRoots, "xml.multiple_roots"),
        (XmlErrorKind::ContentOutsideRoot, "xml.content_outside_root"),
        (XmlErrorKind::InvalidNode, "xml.invalid_node"),
        (
            XmlErrorKind::LimitExceeded(LimitKind::InputBytes),
            "xml.limit.input_bytes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::Depth),
            "xml.limit.depth",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::Nodes),
            "xml.limit.nodes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::Attributes),
            "xml.limit.attributes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::NamespaceDecls),
            "xml.limit.namespace_decls",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::NameBytes),
            "xml.limit.name_bytes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::AttrValueBytes),
            "xml.limit.attr_value_bytes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::TextBytes),
            "xml.limit.text_bytes",
        ),
        (
            XmlErrorKind::LimitExceeded(LimitKind::XmlDetailBytes),
            "xml.limit.xml_detail_bytes",
        ),
    ];
    for (kind, code) in codes {
        assert_eq!(kind.as_str(), code);
    }
}

#[test]
fn error_offsets_point_at_the_construct() {
    assert_eq!(parse("<a>\u{1}</a>").unwrap_err().offset(), Some(3));
    assert_eq!(parse("<a b='xy\u{1}'/>").unwrap_err().offset(), Some(8));
    assert_eq!(
        parse("<a><b/><!DOCTYPE a></a>").unwrap_err().offset(),
        Some(7)
    );
    assert_eq!(parse("<a b='1' 2='3'/>").unwrap_err().offset(), Some(9));
}

// ---- documents are plain data ----

#[test]
fn document_is_send_sync_clone_and_debug_is_content_free() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Document>();
    let doc = parse(COT).unwrap();
    let copy = doc.clone();
    assert_eq!(copy.to_xml_bytes(), doc.to_xml_bytes());
    let debug = format!("{doc:?} {:?}", doc.root());
    assert!(!debug.contains("ANDROID"), "{debug}");
}

// ---- edits (used by the proto -> XML conversion) ----

#[test]
fn build_a_document() {
    let limits = Limits::default();
    let mut doc = Document::with_root("event", &limits).unwrap();
    let root = doc.root().id();
    doc.push_attribute(root, "uid", "u\"1\n").unwrap();
    let point = doc.append_element(root, "point").unwrap();
    doc.push_attribute(point, "lat", "1.0").unwrap();
    let detail = doc.append_element(root, "detail").unwrap();
    let remarks = doc.append_element(detail, "remarks").unwrap();
    doc.append_text(remarks, "a<b").unwrap();
    let empty = doc.append_element(detail, "empty").unwrap();
    doc.append_text(empty, "").unwrap();
    let out = String::from_utf8(doc.to_xml_bytes()).unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<event uid=\"u&quot;1&#10;\"><point lat=\"1.0\"/><detail><remarks>a&lt;b</remarks><empty></empty></detail></event>"
    );
    assert_eq!(parse(&out).unwrap().root().attribute("uid"), Some("u\"1\n"));
}

#[test]
fn edit_errors() {
    let limits = Limits::default();
    assert_eq!(
        Document::with_root("x:event", &limits).unwrap_err().kind(),
        XmlErrorKind::InvalidName
    );
    let mut doc = Document::with_root("event", &limits).unwrap();
    let root = doc.root().id();
    let text = doc.append_text(root, "t").unwrap();
    assert_eq!(
        doc.append_element(text, "a").unwrap_err().kind(),
        XmlErrorKind::InvalidNode
    );
    assert_eq!(
        doc.push_attribute(text, "a", "1").unwrap_err().kind(),
        XmlErrorKind::InvalidNode
    );
    assert_eq!(
        doc.append_text(text, "a").unwrap_err().kind(),
        XmlErrorKind::InvalidNode
    );
    assert_eq!(
        doc.append_element(root, "1a").unwrap_err().kind(),
        XmlErrorKind::InvalidName
    );
    assert_eq!(
        doc.push_attribute(root, "a:b", "1").unwrap_err().kind(),
        XmlErrorKind::InvalidName
    );
    doc.push_attribute(root, "a", "1").unwrap();
    assert_eq!(
        doc.push_attribute(root, "a", "2").unwrap_err().kind(),
        XmlErrorKind::DuplicateAttribute
    );
    assert_eq!(
        doc.push_attribute(root, "b", "\u{0}").unwrap_err().kind(),
        XmlErrorKind::InvalidChar
    );
    assert_eq!(
        doc.append_text(root, "\u{1}").unwrap_err().kind(),
        XmlErrorKind::InvalidChar
    );
    assert_eq!(
        doc.detach(root).unwrap_err().kind(),
        XmlErrorKind::InvalidNode
    );
    let err = doc.detach(root).unwrap_err();
    assert_eq!(err.offset(), None);
}

#[test]
fn append_copy_and_detach() {
    let limits = Limits::default();
    let src = parse(
        "<detail><contact callsign='B'/><x:a xmlns:x='urn:x'><x:b>t</x:b></x:a><g xmlns='urn:g'><h/></g></detail>",
    )
    .unwrap();
    let mut doc = Document::with_root("event", &limits).unwrap();
    let root = doc.root().id();
    let detail = doc.append_element(root, "detail").unwrap();
    let contact = doc.append_element(detail, "contact").unwrap();
    doc.push_attribute(contact, "callsign", "A").unwrap();
    doc.append_element(detail, "track").unwrap();

    // The xmlDetail override: drop the typed contact, append the copies.
    doc.detach(contact).unwrap();
    assert_eq!(
        doc.detach(contact).unwrap_err().kind(),
        XmlErrorKind::InvalidNode
    );
    for child in src
        .root()
        .child_elements()
        .map(|e| e.id())
        .collect::<Vec<_>>()
    {
        doc.append_copy(detail, &src, child).unwrap();
    }
    // A built element under a copied default-namespace element.
    let g = doc
        .root()
        .first_child_element("detail")
        .and_then(|d| d.first_child_element("g"))
        .unwrap()
        .id();
    doc.append_element(g, "plain").unwrap();

    let out = String::from_utf8(doc.to_xml_bytes()).unwrap();
    assert_eq!(
        out,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<event><detail><track/><contact callsign=\"B\"/><x:a xmlns:x=\"urn:x\"><x:b>t</x:b></x:a><g xmlns=\"urn:g\"><h/><plain xmlns=\"\"/></g></detail></event>"
    );
    let reparsed = parse(&out).unwrap();
    let plain = reparsed
        .root()
        .first_child_element("detail")
        .and_then(|d| d.first_child_element("g"))
        .and_then(|g| g.first_child_element("plain"))
        .unwrap();
    assert_eq!(plain.namespace_uri(), None);

    // Detaching the first and last children relinks the siblings.
    let detail_ref = doc.element(detail).unwrap();
    let ids: Vec<_> = detail_ref.child_elements().map(|e| e.id()).collect();
    doc.detach(*ids.first().unwrap()).unwrap();
    doc.detach(*ids.last().unwrap()).unwrap();
    let names: Vec<_> = doc
        .element(detail)
        .unwrap()
        .child_elements()
        .map(|e| e.name())
        .collect();
    assert_eq!(names, ["contact", "x:a"]);
}
