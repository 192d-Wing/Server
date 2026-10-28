// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `cot_parse`: arbitrary bytes into the bounded XML parser and
//! the `CoT` event view (plan section 1.8, WU9).
//!
//! Properties, all under `Limits::default()`:
//! - parsing never panics, times out or exhausts memory;
//! - X2X idempotence: whatever parses serializes to text that parses again
//!   and serializes to the same bytes;
//! - every element fragment (the `xmlDetail` form) that fits the limit is a
//!   document of its own with the same local name;
//! - error codes are stable, non-empty strings.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_cot::xml::{Document, ElementRef, NodeKind};
use tak_cot::{CotEvent, Limits};

/// Most elements whose fragment is re-parsed per input; keeps one input
/// well under the 1 s libFuzzer timeout.
const MAX_FRAGMENTS: usize = 256;

fuzz_target!(|data: &[u8]| {
    // Lossy, so invalid UTF-8 from the mutator still reaches the parser.
    let text = String::from_utf8_lossy(data);
    let limits = Limits::default();

    let event = match CotEvent::parse(&text, &limits) {
        Ok(event) => event,
        Err(err) => {
            assert!(!err.kind().as_str().is_empty());
            let _ = err.to_string();
            return;
        }
    };
    let doc = event.document();
    let _ = (event.uid(), event.type_(), event.how());
    let once = event.to_xml_bytes();
    x2x_is_idempotent(&once, &limits);
    fragments_are_documents(doc, &limits);
});

/// Serialized output re-parses and is a fixed point of parse + serialize.
fn x2x_is_idempotent(once: &[u8], limits: &Limits) {
    let Ok(once_text) = std::str::from_utf8(once) else {
        panic!("serializer wrote invalid UTF-8");
    };
    let reparsed = match Document::parse(once_text, limits) {
        Ok(doc) => doc,
        Err(err) => panic!(
            "serializer output does not re-parse: {}",
            err.kind().as_str()
        ),
    };
    assert_eq!(reparsed.to_xml_bytes(), once, "X2X is not idempotent");
}

/// Every element's fragment, when within `max_input_bytes`, parses on its
/// own with the same local name and namespace.
fn fragments_are_documents(doc: &Document, limits: &Limits) {
    let mut stack: Vec<ElementRef<'_>> = vec![doc.root()];
    let mut done = 0_usize;
    while let Some(el) = stack.pop() {
        if done >= MAX_FRAGMENTS {
            break;
        }
        done = done.saturating_add(1);
        let mut out = Vec::new();
        if el
            .try_to_xml_fragment(&mut out, limits.max_input_bytes)
            .is_ok()
        {
            let Ok(text) = std::str::from_utf8(&out) else {
                panic!("fragment is invalid UTF-8");
            };
            let frag = match Document::parse(text, limits) {
                Ok(frag) => frag,
                Err(err) => panic!("fragment does not parse: {}", err.kind().as_str()),
            };
            assert_eq!(frag.root().local_name(), el.local_name());
            assert_eq!(frag.root().namespace_uri(), el.namespace_uri());
        } else {
            assert!(out.is_empty(), "a rejected fragment left output behind");
        }
        for child in el.children() {
            if let NodeKind::Element(child) = child {
                stack.push(child);
            }
        }
    }
}
