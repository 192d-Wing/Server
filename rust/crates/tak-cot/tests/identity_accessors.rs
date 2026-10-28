// SPDX-License-Identifier: Apache-2.0

//! SEC-008 / parity-map C-06, D-18: `CotEvent::uid`, `type_` and `how` read
//! the same attribute as `to_tak_message` when the root carries the same
//! local name twice (one prefixed, one not), and serialization keeps both.
//! The reference's choice is fixed by the corpus goldens; these tests pin
//! that the TAK-RS accessors and the converter never disagree.

use std::error::Error;

use tak_cot::{CotEvent, Limits, ToProtoOptions, to_tak_message};

type Res<T> = Result<T, Box<dyn Error>>;

const DUP: &str = concat!(
    r#"<event version="2.0" xmlns:p="urn:syn:p" "#,
    r#"p:uid="VICTIM" uid="ATTACKER" "#,
    r#"p:type="a-h-G" type="a-f-G" "#,
    r#"p:how="h-e" how="m-g" "#,
    r#"time="2026-01-01T00:00:00Z" start="2026-01-01T00:00:00Z" stale="2026-01-01T00:05:00Z">"#,
    r#"<point lat="1" lon="2" hae="3" ce="4" le="5"/><detail/></event>"#
);

#[test]
fn identity_accessors_match_converter_on_duplicate_local_names() -> Res<()> {
    let ev = CotEvent::parse(DUP, &Limits::default())?;
    let msg = to_tak_message(&ev, &ToProtoOptions::default())?;
    let proto = msg.cot_event.ok_or("no cotEvent")?;

    assert_eq!(ev.uid(), Some("VICTIM"));
    assert_eq!(ev.type_(), Some("a-h-G"));
    assert_eq!(ev.how(), Some("h-e"));
    assert_eq!(ev.uid(), Some(proto.uid.as_str()));
    assert_eq!(ev.type_(), Some(proto.r#type.as_str()));
    assert_eq!(ev.how(), Some(proto.how.as_str()));
    Ok(())
}

#[test]
fn serialization_keeps_both_duplicate_identity_attributes() -> Res<()> {
    let ev = CotEvent::parse(DUP, &Limits::default())?;
    let xml = String::from_utf8(ev.to_xml_bytes())?;
    for needle in [
        r#"p:uid="VICTIM""#,
        r#" uid="ATTACKER""#,
        r#"p:type="a-h-G""#,
        r#" type="a-f-G""#,
    ] {
        assert!(xml.contains(needle), "{needle} missing from {xml}");
    }
    Ok(())
}
