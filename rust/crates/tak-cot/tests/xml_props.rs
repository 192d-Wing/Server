// SPDX-License-Identifier: Apache-2.0

//! Properties of `tak_cot::xml` (plan section 8.2): no panic on any input,
//! generated well-formed documents parse, serializer output re-parses and is
//! a fixed point (X2X idempotence), attribute values survive the round trip,
//! and every element fragment is a document on its own.
//!
//! Fixed case counts and a fixed RNG seed, so a run is reproducible; failure
//! files are not written into the source tree.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use tak_cot::Limits;
use tak_cot::xml::{Document, ElementRef, NodeKind};

/// Seed recorded for reproducibility (plan section 8.2).
const SEED: u64 = 0x7a6b_2026_1027_0003;
const CASES: u32 = 512;

fn config() -> Config {
    Config {
        cases: CASES,
        rng_seed: RngSeed::Fixed(SEED),
        failure_persistence: None,
        ..Config::default()
    }
}

/// A generated document, rendered to valid XML 1.0 by `render`.
#[derive(Clone, Debug)]
enum Gen {
    Element {
        name: String,
        /// Bind prefix `p` to this name and use `p:` on the element and on
        /// odd-numbered attributes.
        prefix_ns: Option<String>,
        default_ns: Option<String>,
        attrs: Vec<(String, String)>,
        children: Vec<Gen>,
    },
    Text(String),
    CData(String),
    Comment(String),
    Pi(String, String),
}

fn name() -> impl Strategy<Value = String> {
    // `xmlns...` would be a namespace declaration, not an attribute.
    "[a-zA-Z_\u{e9}][a-zA-Z0-9._\u{e9}-]{0,6}".prop_filter("not xmlns", |n| !n.starts_with("xmlns"))
}

fn any_text() -> impl Strategy<Value = String> {
    // Every character class the escapes and normalisation treat specially.
    "[a-z <>&\"'\t\n\r\u{e9}\u{1F600}\u{FFFD}]{0,10}"
}

fn leaf() -> impl Strategy<Value = Gen> {
    prop_oneof![
        any_text().prop_map(Gen::Text),
        "[a-z <>&\"'\t\n\u{e9}]{0,8}".prop_map(Gen::CData),
        "[a-z <>&\t\n\u{e9}]{0,8}".prop_map(Gen::Comment),
        ("[a-wyz][a-z0-9]{0,4}", "[a-z <>&\"']{0,8}").prop_map(|(t, d)| Gen::Pi(t, d)),
    ]
}

fn element(children: impl Strategy<Value = Vec<Gen>>) -> impl Strategy<Value = Gen> {
    (
        name(),
        prop::option::of("urn:[a-z]{1,4}"),
        prop::option::weighted(0.2, prop::option::of("urn:[a-z]{1,4}")),
        prop::collection::btree_map(name(), any_text(), 0..5),
        children,
    )
        .prop_map(
            |(name, prefix_ns, default_ns, attrs, children)| Gen::Element {
                name,
                prefix_ns,
                // `Some(None)` renders `xmlns=""`.
                default_ns: default_ns.map(Option::unwrap_or_default),
                attrs: attrs.into_iter().collect(),
                children,
            },
        )
}

fn document() -> impl Strategy<Value = Gen> {
    element(Just(Vec::new()).boxed()).prop_recursive(6, 48, 6, |inner| {
        element(prop::collection::vec(prop_oneof![leaf(), inner], 0..6).boxed())
    })
}

/// Escapes for rendering, varied so the parser sees references of every
/// form and raw line ends.
fn render_chars(out: &mut String, s: &str, in_attr: bool, salt: usize) {
    for (i, c) in s.chars().enumerate() {
        let alt = (i + salt).is_multiple_of(2);
        match c {
            '<' => out.push_str(if alt { "&lt;" } else { "&#60;" }),
            '&' => out.push_str(if alt { "&amp;" } else { "&#x26;" }),
            '>' => out.push_str(if alt { "&gt;" } else { ">" }),
            '"' if in_attr => out.push_str("&quot;"),
            '\'' if in_attr => out.push_str(if alt { "&apos;" } else { "&#39;" }),
            // In attributes white space other than a space must be a
            // reference to survive normalisation; in text a raw CR would
            // be normalised away, so it is always a reference.
            '\t' if in_attr => out.push_str("&#9;"),
            '\n' if in_attr => out.push_str(if alt { "&#10;" } else { "&#xA;" }),
            '\r' => out.push_str("&#13;"),
            c => out.push(c),
        }
    }
}

fn render(g: &Gen, out: &mut String, salt: usize) {
    match g {
        Gen::Element {
            name,
            prefix_ns,
            default_ns,
            attrs,
            children,
        } => {
            let qname = if prefix_ns.is_some() {
                format!("p:{name}")
            } else {
                name.clone()
            };
            let _ = write!(out, "<{qname}");
            if let Some(ns) = default_ns {
                let _ = write!(out, " xmlns='{ns}'");
            }
            if let Some(ns) = prefix_ns {
                let _ = write!(out, " xmlns:p=\"{ns}\"");
            }
            for (i, (an, av)) in attrs.iter().enumerate() {
                let prefix = if prefix_ns.is_some() && !i.is_multiple_of(2) {
                    "p:"
                } else {
                    ""
                };
                let _ = write!(out, "\n {prefix}{an} = \"");
                render_chars(out, av, true, salt + i);
                out.push('"');
            }
            if children.is_empty() && salt.is_multiple_of(2) {
                out.push_str("/>");
                return;
            }
            out.push('>');
            for (i, c) in children.iter().enumerate() {
                render(c, out, salt + i + 1);
            }
            let _ = write!(out, "</{qname}\r\n>");
        }
        Gen::Text(t) => render_chars(out, t, false, salt),
        Gen::CData(t) => {
            let _ = write!(out, "<![CDATA[{t}]]>");
        }
        Gen::Comment(t) => {
            let _ = write!(out, "<!--{t}-->");
        }
        Gen::Pi(target, data) => {
            let _ = write!(out, "<?{target} {data}?>");
        }
    }
}

fn rendered(g: &Gen) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<!--top-->");
    render(g, &mut out, 0);
    out.push_str("\n<?end?>\n");
    out
}

fn reserialize(bytes: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(bytes).ok()?;
    Some(
        Document::parse(text, &Limits::default())
            .ok()?
            .to_xml_bytes(),
    )
}

/// Every element of the document, root first, without recursion.
fn elements(doc: &Document) -> Vec<ElementRef<'_>> {
    let mut out = vec![doc.root()];
    let mut i = 0;
    while let Some(el) = out.get(i).copied() {
        out.extend(el.child_elements());
        i += 1;
    }
    out
}

fn mutate(seed: &str, edits: &[(usize, u8)]) -> Option<String> {
    let mut bytes = seed.as_bytes().to_vec();
    for &(pos, b) in edits {
        let len = bytes.len();
        if len == 0 {
            break;
        }
        match b % 3 {
            0 => {
                *bytes.get_mut(pos % len)? = b;
            }
            1 => {
                bytes.remove(pos % len);
            }
            _ => bytes.insert(pos % (len + 1), b),
        }
    }
    String::from_utf8(bytes).ok()
}

const SEED_DOC: &str = "<?xml version='1.0'?><!--c--><event uid='a&amp;b' x:y='1' xmlns:x='urn:x'><point lat='1.0'/><detail><contact callsign='A'/>t&#10;<![CDATA[d]]><?p q?></detail></event>";

proptest! {
    #![proptest_config(config())]

    #[test]
    fn prop_parse_never_panics_on_arbitrary_text(input in "\\PC{0,200}") {
        let _ = Document::parse(&input, &Limits::default());
    }

    #[test]
    fn prop_parse_never_panics_on_markup_soup(
        input in "[<>/?!=&#;:'\"a-c x\\[\\]\\-\r\n\t\u{0}\u{1}\u{FEFF}\u{e9}]{0,120}"
    ) {
        if let Ok(doc) = Document::parse(&input, &Limits::default()) {
            // Anything accepted must also serialize and re-parse.
            prop_assert!(reserialize(&doc.to_xml_bytes()).is_some());
        }
    }

    #[test]
    fn prop_parse_never_panics_on_mutated_cot(
        edits in prop::collection::vec((any::<usize>(), any::<u8>()), 1..8)
    ) {
        if let Some(input) = mutate(SEED_DOC, &edits)
            && let Ok(doc) = Document::parse(&input, &Limits::default())
        {
            let once = doc.to_xml_bytes();
            prop_assert_eq!(reserialize(&once), Some(once.clone()));
        }
    }

    #[test]
    fn prop_parse_never_panics_under_tiny_limits(
        input in "[<>/a-c ='&;#x0-9]{0,60}",
        depth in 0_u16..4, nodes in 0_u32..6, attrs in 0_u16..3, name in 0_u16..4,
        value in 0_u32..4, text in 0_u32..4, size in 0_usize..64,
    ) {
        let limits = Limits {
            max_input_bytes: size,
            max_depth: depth,
            max_nodes: nodes,
            max_attributes: attrs,
            max_namespace_decls: attrs,
            max_name_bytes: name,
            max_attr_value_bytes: value,
            max_text_bytes: text,
            max_xml_detail_bytes: size,
        };
        let _ = Document::parse(&input, &limits);
    }

    #[test]
    fn prop_generated_documents_parse_and_x2x_is_idempotent(g in document()) {
        let input = rendered(&g);
        let doc = Document::parse(&input, &Limits::default());
        prop_assert!(doc.is_ok(), "{:?} on {:?}", doc.err(), input);
        let Ok(doc) = doc else { return Ok(()) };
        let once = doc.to_xml_bytes();
        let twice = reserialize(&once);
        prop_assert_eq!(twice.as_deref(), Some(once.as_slice()), "input {:?}", input);
    }

    #[test]
    fn prop_attribute_values_survive_round_trip(g in document()) {
        let Gen::Element { attrs, .. } = &g else { return Ok(()) };
        let input = rendered(&g);
        let doc = Document::parse(&input, &Limits::default()).unwrap();
        let parsed: Vec<_> = doc
            .root()
            .attributes()
            .map(|a| (a.local_name().to_owned(), a.value().to_owned()))
            .collect();
        prop_assert_eq!(&parsed, attrs);
        let again = Document::parse(
            std::str::from_utf8(&doc.to_xml_bytes()).unwrap(),
            &Limits::default(),
        )
        .unwrap();
        let reparsed: Vec<_> = again
            .root()
            .attributes()
            .map(|a| (a.local_name().to_owned(), a.value().to_owned()))
            .collect();
        prop_assert_eq!(&reparsed, attrs);
    }

    #[test]
    fn prop_every_fragment_is_a_document(g in document()) {
        let input = rendered(&g);
        let doc = Document::parse(&input, &Limits::default()).unwrap();
        for el in elements(&doc) {
            let mut out = Vec::new();
            el.to_xml_fragment(&mut out);
            let text = std::str::from_utf8(&out).unwrap();
            let frag = Document::parse(text, &Limits::default());
            prop_assert!(frag.is_ok(), "{:?} on {:?}", frag.err(), text);
            let frag = frag.unwrap();
            prop_assert_eq!(frag.root().local_name(), el.local_name());
            prop_assert_eq!(frag.root().namespace_uri(), el.namespace_uri());
            // Text content is unchanged.
            let texts = |e: ElementRef<'_>| -> String {
                e.children()
                    .filter_map(|n| match n {
                        NodeKind::Text(t) | NodeKind::CData(t) => Some(t),
                        _ => None,
                    })
                    .collect()
            };
            prop_assert_eq!(texts(frag.root()), texts(el));
        }
    }
}
