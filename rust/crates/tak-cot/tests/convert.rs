// SPDX-License-Identifier: Apache-2.0

//! One test per "Ported" and "Kept quirk" row of the Phase 1a parity map,
//! sections A to C, named after the row ID, plus the deviation rows the
//! conversion touches (DEV-02, DEV-05) and C-08 / C-12.
//!
//! Expectations are the behaviour statements of the parity map (black-box
//! descriptions of the reference). They check values and structure, not
//! serializer bytes: the exact XML form is fixed by the WU8 goldens.

use std::error::Error;

use tak_cot::xml::{Document, ElementRef, NodeKind};
use tak_cot::{
    ConvertError, ConvertErrorKind, CotEvent, Limits, TimeField, ToProtoOptions, from_tak_message,
    to_tak_message,
};
use tak_proto::framing::{
    MAGIC, decode_mesh, decode_stream_frame, encode_mesh, encode_stream_frame,
};
use tak_proto::v1::{
    Contact, CotEvent as ProtoEvent, Detail, Group, PrecisionLocation, Status, TakControl,
    TakMessage, Takv, Track,
};
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message};

type Res<T> = Result<T, Box<dyn Error>>;

/// XML to proto with no caller times.
fn x2p(xml: &str) -> Res<TakMessage> {
    let ev = CotEvent::parse(xml, &Limits::default())?;
    Ok(to_tak_message(&ev, &ToProtoOptions::default())?)
}

/// XML to proto, returning only the `CotEvent`.
fn x2p_event(xml: &str) -> Res<ProtoEvent> {
    x2p(xml)?.cot_event.ok_or_else(|| "no cotEvent".into())
}

/// XML to proto `Detail` of an event whose `<detail>` holds `inner`.
fn x2p_detail(inner: &str) -> Res<Detail> {
    let xml = format!(
        r#"<event version="2.0" uid="u" type="a-f-G" how="m-g" time="2025-10-27T18:04:58Z" start="2025-10-27T18:04:58Z" stale="2025-10-27T18:05:58Z"><point lat="1" lon="2" hae="3" ce="4" le="5"/><detail>{inner}</detail></event>"#
    );
    x2p_event(&xml)?.detail.ok_or_else(|| "no detail".into())
}

/// A message with only `detail` set in its event.
fn with_detail(detail: Detail) -> TakMessage {
    TakMessage {
        cot_event: Some(ProtoEvent {
            uid: "u".into(),
            detail: Some(detail),
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    }
}

/// Proto to proto (the path a proto client's message takes to another
/// proto client).
fn p2p(msg: &TakMessage) -> Res<TakMessage> {
    let ev = from_tak_message(msg, &Limits::default())?;
    Ok(to_tak_message(&ev, &ToProtoOptions::default())?)
}

fn p2p_event(msg: &TakMessage) -> Res<ProtoEvent> {
    p2p(msg)?.cot_event.ok_or_else(|| "no cotEvent".into())
}

fn child<'a>(el: ElementRef<'a>, name: &str) -> Res<ElementRef<'a>> {
    el.first_child_element(name)
        .ok_or_else(|| format!("no <{name}>").into())
}

/// (name, value) pairs of an element's attributes, in order.
fn attrs(el: ElementRef<'_>) -> Vec<(String, String)> {
    el.attributes()
        .map(|a| (a.name().to_owned(), a.value().to_owned()))
        .collect()
}

fn child_names(el: ElementRef<'_>) -> Vec<String> {
    el.child_elements().map(|e| e.name().to_owned()).collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
        .collect()
}

const T0: &str = "2025-10-27T18:04:58Z";
const T0_MS: u64 = 1_761_588_298_000;

// ---------------------------------------------------------------------------
// A. XML -> proto
// ---------------------------------------------------------------------------

#[test]
fn k01_root_type_uid_how_copied_and_missing_left_empty() {
    let ev = x2p_event(r#"<event type="a-f-G-U-C" uid="ANDROID-1" how="m-g"/>"#).unwrap();
    assert_eq!(
        (ev.r#type.as_str(), ev.uid.as_str(), ev.how.as_str()),
        ("a-f-G-U-C", "ANDROID-1", "m-g")
    );

    // Missing attributes leave the fields empty; a message is still produced.
    let msg = x2p(r"<event/>").unwrap();
    let ev = msg.cot_event.unwrap();
    assert!(ev.r#type.is_empty() && ev.uid.is_empty() && ev.how.is_empty());

    // Any root element name is accepted (no CoT schema).
    assert_eq!(x2p_event(r#"<foo uid="x"/>"#).unwrap().uid, "x");
}

#[test]
fn k02_times_to_millis_missing_zero_unparseable_rejects() {
    let ev = x2p_event(&format!(
        r#"<event time="{T0}" start="2025-10-27T18:04:58.123Z" stale="2025-10-27T18:04:58.5+01:00"/>"#
    ))
    .unwrap();
    assert_eq!(ev.send_time, T0_MS);
    assert_eq!(ev.start_time, T0_MS + 123);
    assert_eq!(ev.stale_time, T0_MS + 500 - 3_600_000);

    // Missing -> 0.
    let ev = x2p_event(r#"<event uid="u"/>"#).unwrap();
    assert_eq!((ev.send_time, ev.start_time, ev.stale_time), (0, 0, 0));

    // Unparseable -> the whole conversion fails, naming the attribute.
    for (attr, field) in [
        ("time", TimeField::Time),
        ("start", TimeField::Start),
        ("stale", TimeField::Stale),
    ] {
        let xml = format!(r#"<event {attr}="not a time"/>"#);
        let ev = CotEvent::parse(&xml, &Limits::default()).unwrap();
        let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
        assert_eq!(err, ConvertError::Time(field));
        assert_eq!(err.kind(), ConvertErrorKind::Time(field));
    }
    // An ISO time without an offset is unreadable too.
    let ev = CotEvent::parse(
        r#"<event time="2025-10-27T18:04:58.1"/>"#,
        &Limits::default(),
    )
    .unwrap();
    assert!(to_tak_message(&ev, &ToProtoOptions::default()).is_err());

    // A time before 1970 is a negative Java long, sent as its unsigned
    // 64-bit pattern.
    let ev = x2p_event(r#"<event time="1969-12-31T23:59:59Z"/>"#).unwrap();
    assert_eq!(ev.send_time, (-1000_i64).cast_unsigned());
}

#[test]
fn dev05_out_of_range_time_rejected_like_any_bad_time() {
    // The reference's time library wraps around for these (a meaningless
    // value is sent); TAK-RS rejects the event (candidate DEV-05).
    for t in [
        "292278994-08-17T07:12:56Z",
        "-292275055-01-01T00:00:00Z",
        "292278994-12-31T23:59:59.999Z",
    ] {
        let xml = format!(r#"<event uid="u" time="{T0}" start="{T0}" stale="{t}"/>"#);
        let ev = CotEvent::parse(&xml, &Limits::default()).unwrap();
        assert_eq!(
            to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err(),
            ConvertError::Time(TimeField::Stale),
            "{t}"
        );
    }
    // The last printable instant still converts.
    let ev = x2p_event(r#"<event time="292278994-08-17T07:12:55Z"/>"#).unwrap();
    assert_eq!(ev.send_time, (i64::MAX - 807).cast_unsigned());
}

#[test]
fn k03_first_point_only_and_its_defaults() {
    // No point: all five are 0, not 999999.
    let ev = x2p_event(r#"<event uid="u"/>"#).unwrap();
    assert_eq!([ev.lat, ev.lon, ev.hae, ev.ce, ev.le], [0.0; 5]);

    // Only the first point counts.
    let ev = x2p_event(
        r#"<event><point lat="1.5" lon="-2.25" hae="3" ce="4" le="5"/><point lat="9" lon="9" hae="9" ce="9" le="9"/></event>"#,
    )
    .unwrap();
    assert_eq!(
        [ev.lat, ev.lon, ev.hae, ev.ce, ev.le],
        [1.5, -2.25, 3.0, 4.0, 5.0]
    );

    // Missing attribute -> 0 for all five.
    let ev = x2p_event(r"<event><point/></event>").unwrap();
    assert_eq!([ev.lat, ev.lon, ev.hae, ev.ce, ev.le], [0.0; 5]);

    // Unparseable: lat/lon/hae -> 0, ce/le -> 999999.
    let ev =
        x2p_event(r#"<event><point lat="x" lon="" hae="inf" ce="abc" le=" "/></event>"#).unwrap();
    assert_eq!(
        [ev.lat, ev.lon, ev.hae, ev.ce, ev.le],
        [0.0, 0.0, 0.0, 999_999.0, 999_999.0]
    );

    // Java grammar: trimming, suffixes, hex, NaN and Infinity parse.
    let ev = x2p_event(
        r#"<event><point lat=" 2 " lon="1.5d" hae="0x1p3" ce="NaN" le="-Infinity"/></event>"#,
    )
    .unwrap();
    assert_eq!([ev.lat, ev.lon, ev.hae], [2.0, 1.5, 8.0]);
    assert!(ev.ce.is_nan());
    assert_eq!(ev.le, f64::NEG_INFINITY);

    // A prefixed point and prefixed attributes match by local name (C-06
    // expectation).
    let ev = x2p_event(
        r#"<event xmlns:x="urn:x"><x:point x:lat="7" lon="8" hae="9" ce="1" le="2"/></event>"#,
    )
    .unwrap();
    assert_eq!([ev.lat, ev.lon], [7.0, 8.0]);
}

#[test]
fn k04_optional_root_attributes_and_releasable_to_spelling() {
    let ev = x2p_event(
        r#"<event caveat="C" releasableTo="USA" opex="o" qos="1-r-c" access="Unclassified"/>"#,
    )
    .unwrap();
    assert_eq!(ev.caveat, "C");
    assert_eq!(ev.releaseable_to, "USA");
    assert_eq!(ev.opex, "o");
    assert_eq!(ev.qos, "1-r-c");
    assert_eq!(ev.access, "Unclassified");

    // The proto spelling is not read from XML.
    let ev = x2p_event(r#"<event releaseableTo="USA"/>"#).unwrap();
    assert_eq!(ev.releaseable_to, "");

    // Present but empty: copied (an empty proto3 string is not written).
    let ev = x2p_event(r#"<event access="" caveat=""/>"#).unwrap();
    assert!(ev.access.is_empty() && ev.caveat.is_empty());
}

#[test]
fn k05_detail_message_whenever_a_detail_element_exists() {
    let ev = x2p_event(r#"<event uid="u"><detail/></event>"#).unwrap();
    assert_eq!(ev.detail, Some(Detail::default()));
    // Encodes as field 15 with length 0.
    let bytes = encode_tak_message(&TakMessage {
        cot_event: Some(ev),
        ..TakMessage::default()
    });
    assert!(bytes.ends_with(&[0x7a, 0]));

    // Only ignorable content: still an empty Detail.
    let ev = x2p_event(r"<event><detail> text <!-- c --><?pi x?></detail></event>").unwrap();
    assert_eq!(ev.detail, Some(Detail::default()));

    // No detail element: no Detail.
    assert_eq!(x2p_event(r#"<event uid="u"/>"#).unwrap().detail, None);
}

#[test]
fn k06_contact_exact_attribute_set_and_empty_endpoint() {
    let d = x2p_detail(r#"<contact callsign="A"/>"#).unwrap();
    assert_eq!(
        d.contact,
        Some(Contact {
            callsign: "A".into(),
            endpoint: String::new()
        })
    );
    assert_eq!(d.xml_detail, "");

    let d = x2p_detail(r#"<contact endpoint="*:-1:stcp" callsign="A"/>"#).unwrap();
    assert_eq!(d.contact.unwrap().endpoint, "*:-1:stcp");

    // An empty endpoint is dropped from both the message and xmlDetail.
    let d = x2p_detail(r#"<contact callsign="A" endpoint=""/>"#).unwrap();
    assert_eq!(
        d.contact,
        Some(Contact {
            callsign: "A".into(),
            endpoint: String::new()
        })
    );
    assert_eq!(d.xml_detail, "");

    // Not eligible: an extra attribute, no callsign, no attributes.
    for inner in [
        r#"<contact callsign="A" endpoint="e" phone="1"/>"#,
        r#"<contact callsign="A" phone="1"/>"#,
        r#"<contact endpoint="e"/>"#,
        r"<contact/>",
    ] {
        let d = x2p_detail(inner).unwrap();
        assert_eq!(d.contact, None, "{inner}");
        assert!(d.xml_detail.starts_with("<contact"), "{inner}");
    }

    // A namespace declaration is not an attribute (C-06 expectation).
    let d = x2p_detail(r#"<contact xmlns:x="urn:x" callsign="A"/>"#).unwrap();
    assert!(d.contact.is_some());
}

#[test]
fn k07_group_precisionlocation_takv_need_their_exact_attribute_sets() {
    let d = x2p_detail(
        r#"<__group role="Team Member" name="Cyan"/><precisionlocation altsrc="DTED0" geopointsrc="GPS"/><takv os="34" version="4.10" device="Pixel" platform="ATAK-CIV"/>"#,
    )
    .unwrap();
    assert_eq!(
        d.group,
        Some(Group {
            name: "Cyan".into(),
            role: "Team Member".into()
        })
    );
    assert_eq!(
        d.precision_location,
        Some(PrecisionLocation {
            geopointsrc: "GPS".into(),
            altsrc: "DTED0".into()
        })
    );
    assert_eq!(
        d.takv,
        Some(Takv {
            device: "Pixel".into(),
            platform: "ATAK-CIV".into(),
            os: "34".into(),
            version: "4.10".into()
        })
    );
    assert_eq!(d.xml_detail, "");

    // One missing or one extra attribute: the element stays in xmlDetail.
    let d = x2p_detail(
        r#"<__group name="Cyan"/><precisionlocation geopointsrc="GPS" altsrc="x" extra="1"/><takv device="d" platform="p" os="o"/>"#,
    )
    .unwrap();
    assert_eq!((d.group, d.precision_location, d.takv), (None, None, None));
    let doc = Document::parse(&format!("<d>{}</d>", d.xml_detail), &Limits::default()).unwrap();
    assert_eq!(
        child_names(doc.root()),
        ["__group", "precisionlocation", "takv"]
    );

    // Empty values are still values.
    let d = x2p_detail(r#"<__group name="" role=""/>"#).unwrap();
    assert_eq!(d.group, Some(Group::default()));
}

#[test]
fn k08_status_and_track_parse_with_zero_defaults() {
    let d = x2p_detail(r#"<status battery="87"/><track course="270.5" speed="3.25"/>"#).unwrap();
    assert_eq!(d.status, Some(Status { battery: 87 }));
    assert_eq!(
        d.track,
        Some(Track {
            speed: 3.25,
            course: 270.5
        })
    );

    // Unparseable -> 0; `Integer.parseInt` takes no fraction or spaces.
    let d = x2p_detail(r#"<status battery="87.5"/><track speed="fast" course=""/>"#).unwrap();
    assert_eq!(d.status, Some(Status { battery: 0 }));
    assert_eq!(d.track, Some(Track::default()));
    let d = x2p_detail(r#"<status battery=" 1"/>"#).unwrap();
    assert_eq!(d.status, Some(Status { battery: 0 }));

    // A negative battery is sent as its unsigned 32-bit pattern.
    let d = x2p_detail(r#"<status battery="-1"/>"#).unwrap();
    assert_eq!(d.status, Some(Status { battery: u32::MAX }));

    // Exact attribute sets.
    let d = x2p_detail(r#"<status battery="1" charging="true"/><track speed="1"/>"#).unwrap();
    assert_eq!((d.status, d.track), (None, None));
}

#[test]
fn k09_only_the_first_element_of_each_name_is_tried() {
    // First ineligible: the second, eligible one is not tried; both stay.
    let d = x2p_detail(r#"<contact callsign="A" x="1"/><contact callsign="B"/>"#).unwrap();
    assert_eq!(d.contact, None);
    let doc = Document::parse(&format!("<d>{}</d>", d.xml_detail), &Limits::default()).unwrap();
    let kept: Vec<_> = doc
        .root()
        .child_elements()
        .map(|e| e.attribute("callsign").unwrap_or_default().to_owned())
        .collect();
    assert_eq!(kept, ["A", "B"]);

    // First eligible: it converts; the second stays in xmlDetail.
    let d = x2p_detail(r#"<remarks>r</remarks><contact callsign="A"/><contact callsign="B"/>"#)
        .unwrap();
    assert_eq!(d.contact.unwrap().callsign, "A");
    let doc = Document::parse(&format!("<d>{}</d>", d.xml_detail), &Limits::default()).unwrap();
    assert_eq!(child_names(doc.root()), ["remarks", "contact"]);
    assert_eq!(
        child(doc.root(), "contact").unwrap().attribute("callsign"),
        Some("B")
    );
}

#[test]
fn k10_remaining_child_elements_concatenated_in_document_order() {
    let d = x2p_detail(
        "\n  <remarks>hi</remarks>\n  text<!-- c --><contact callsign=\"A\"/><link uid=\"x\"><x>1</x></link>\n",
    )
    .unwrap();
    assert!(d.contact.is_some());
    // Each remaining element is written alone; text, white space and
    // comments directly under detail are dropped.
    let doc = Document::parse(&format!("<d>{}</d>", d.xml_detail), &Limits::default()).unwrap();
    assert_eq!(child_names(doc.root()), ["remarks", "link"]);
    assert!(
        doc.root()
            .children()
            .all(|n| matches!(n, NodeKind::Element(_)))
    );
    assert!(d.xml_detail.starts_with("<remarks>hi</remarks>"));

    // Nested content inside a kept element is kept.
    let link = child(doc.root(), "link").unwrap();
    assert_eq!(child_names(link), ["x"]);

    // No remaining element: xmlDetail unset.
    let d = x2p_detail(r#"<contact callsign="A"/> trailing"#).unwrap();
    assert_eq!(d.xml_detail, "");
}

#[test]
fn k10_xml_detail_output_is_capped() {
    // A namespace declared on `event` is re-declared by every fragment
    // that uses it, so xmlDetail can outgrow the input. It is capped by
    // max_xml_detail_bytes.
    let uri = "u".repeat(1000);
    let children = "<x:a/>".repeat(50);
    let xml = format!(r#"<event xmlns:x="{uri}"><detail>{children}</detail></event>"#);
    let limits = Limits {
        max_xml_detail_bytes: 20_000,
        ..Limits::default()
    };
    let ev = CotEvent::parse(&xml, &limits).unwrap();
    let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
    assert_eq!(err, ConvertError::XmlDetailTooLarge);
    assert_eq!(err.kind().as_str(), "convert.xml_detail_too_large");

    // Under the default cap it converts, and every fragment carries the
    // declaration.
    let ev = CotEvent::parse(&xml, &Limits::default()).unwrap();
    let msg = to_tak_message(&ev, &ToProtoOptions::default()).unwrap();
    let xml_detail = msg.cot_event.unwrap().detail.unwrap().xml_detail;
    assert_eq!(xml_detail.matches(uri.as_str()).count(), 50);
}

#[test]
fn k11_creation_and_submission_only_when_positive() {
    let mut ev = CotEvent::parse(r#"<event uid="u"/>"#, &Limits::default()).unwrap();
    let msg = to_tak_message(&ev, &ToProtoOptions::default()).unwrap();
    assert_eq!((msg.creation_time, msg.submission_time), (0, 0));

    let opts = ToProtoOptions {
        creation_ms: Some(1_000),
        submission_ms: Some(2_000),
    };
    let msg = to_tak_message(&ev, &opts).unwrap();
    assert_eq!((msg.creation_time, msg.submission_time), (1_000, 2_000));

    for v in [0, -1, -5, i64::MIN] {
        let opts = ToProtoOptions {
            creation_ms: Some(v),
            submission_ms: Some(v),
        };
        let msg = to_tak_message(&ev, &opts).unwrap();
        assert_eq!((msg.creation_time, msg.submission_time), (0, 0), "{v}");
    }

    // The event's own times apply unless the options override them.
    ev.set_creation_ms(Some(7));
    ev.set_submission_ms(Some(8));
    let msg = to_tak_message(&ev, &ToProtoOptions::default()).unwrap();
    assert_eq!((msg.creation_time, msg.submission_time), (7, 8));
    let opts = ToProtoOptions {
        creation_ms: Some(70),
        submission_ms: None,
    };
    let msg = to_tak_message(&ev, &opts).unwrap();
    assert_eq!((msg.creation_time, msg.submission_time), (70, 8));
    assert!(msg.tak_control.is_none());
}

#[test]
fn k12_failures_return_typed_errors_without_input_text() {
    let secret = "SECRET-CALLSIGN";
    let ev = CotEvent::parse(
        &format!(r#"<event uid="{secret}" time="{secret}"/>"#),
        &Limits::default(),
    )
    .unwrap();
    let err = to_tak_message(&ev, &ToProtoOptions::default()).unwrap_err();
    assert_eq!(err.kind().as_str(), "convert.time.time");
    let text = format!("{err} {err:?}");
    assert!(!text.contains(secret), "{text}");

    let msg = with_detail(Detail {
        xml_detail: format!("<{secret}>"),
        ..Detail::default()
    });
    let err = from_tak_message(&msg, &Limits::default()).unwrap_err();
    assert_eq!(err.kind().as_str(), "convert.xml_detail");
    let mut text = format!("{err} {err:?}");
    let mut source = err.source();
    while let Some(s) = source {
        text.push_str(&s.to_string());
        source = s.source();
    }
    assert!(!text.contains(secret), "{text}");

    // Parse errors keep the XML code.
    let err = CotEvent::parse("<!DOCTYPE x><event/>", &Limits::default()).unwrap_err();
    assert_eq!(err.kind().as_str(), "xml.doctype");
}

// ---------------------------------------------------------------------------
// B. proto -> XML
// ---------------------------------------------------------------------------

fn full_event() -> ProtoEvent {
    ProtoEvent {
        r#type: "a-f-G-U-C".into(),
        access: "Unclassified".into(),
        qos: "1-r-c".into(),
        opex: "e".into(),
        caveat: "C".into(),
        releaseable_to: "USA".into(),
        uid: "ANDROID-1".into(),
        send_time: 1_761_588_298_123,
        start_time: 1_761_588_298_123,
        stale_time: 1_761_588_358_999,
        how: "m-g".into(),
        lat: 38.5,
        lon: -77.25,
        hae: 999_999.0,
        ce: 9.9,
        le: 1.0e7,
        detail: None,
    }
}

#[test]
fn k13_event_attributes_in_fixed_order() {
    let msg = TakMessage {
        cot_event: Some(full_event()),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let root = ev.document().root();
    assert_eq!(root.name(), "event");
    assert_eq!(
        attrs(root),
        pairs(&[
            ("version", "2.0"),
            ("uid", "ANDROID-1"),
            ("type", "a-f-G-U-C"),
            ("how", "m-g"),
            ("time", "2025-10-27T18:04:58Z"),
            ("start", "2025-10-27T18:04:58Z"),
            ("stale", "2025-10-27T18:05:58Z"),
            ("caveat", "C"),
            ("releasableTo", "USA"),
            ("opex", "e"),
            ("qos", "1-r-c"),
            ("access", "Unclassified"),
        ])
    );
    assert_eq!(ev.uid(), Some("ANDROID-1"));
    assert_eq!(ev.type_(), Some("a-f-G-U-C"));
    assert_eq!(ev.how(), Some("m-g"));

    // Empty strings: the first seven are written anyway, the rest omitted.
    let ev = from_tak_message(
        &TakMessage {
            cot_event: Some(ProtoEvent::default()),
            ..TakMessage::default()
        },
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(
        attrs(ev.document().root()),
        pairs(&[
            ("version", "2.0"),
            ("uid", ""),
            ("type", ""),
            ("how", ""),
            ("time", "1970-01-01T00:00:00Z"),
            ("start", "1970-01-01T00:00:00Z"),
            ("stale", "1970-01-01T00:00:00Z"),
        ])
    );
}

#[test]
fn k14_times_printed_in_whole_seconds() {
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            send_time: 1_761_588_298_999,
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    assert_eq!(ev.document().root().attribute("time"), Some(T0));
    let xml = String::from_utf8(ev.to_xml_bytes()).unwrap();
    assert!(xml.contains(r#"time="2025-10-27T18:04:58Z""#), "{xml}");
}

#[test]
fn k15_point_always_written_in_double_to_string_form() {
    let msg = TakMessage {
        cot_event: Some(full_event()),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let point = child(ev.document().root(), "point").unwrap();
    assert_eq!(
        attrs(point),
        pairs(&[
            ("lat", "38.5"),
            ("lon", "-77.25"),
            ("hae", "999999.0"),
            ("ce", "9.9"),
            ("le", "1.0E7"),
        ])
    );

    // Zeros and specials.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            lat: -0.0,
            lon: f64::NAN,
            hae: f64::INFINITY,
            ce: 0.001,
            le: 1.0e-4,
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let point = child(ev.document().root(), "point").unwrap();
    assert_eq!(
        attrs(point),
        pairs(&[
            ("lat", "-0.0"),
            ("lon", "NaN"),
            ("hae", "Infinity"),
            ("ce", "0.001"),
            ("le", "1.0E-4"),
        ])
    );
}

#[test]
fn k17_detail_element_always_written() {
    let msg = TakMessage {
        cot_event: Some(full_event()),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let root = ev.document().root();
    assert_eq!(child_names(root), ["point", "detail"]);
    assert!(!child(root, "detail").unwrap().has_children());

    // P2P therefore always yields at least an empty Detail.
    assert_eq!(p2p_event(&msg).unwrap().detail, Some(Detail::default()));
}

fn all_typed() -> Detail {
    Detail {
        xml_detail: String::new(),
        contact: Some(Contact {
            endpoint: "*:-1:stcp".into(),
            callsign: "A".into(),
        }),
        group: Some(Group {
            name: "Cyan".into(),
            role: "Team Member".into(),
        }),
        precision_location: Some(PrecisionLocation {
            geopointsrc: "GPS".into(),
            altsrc: "DTED0".into(),
        }),
        status: Some(Status { battery: 87 }),
        takv: Some(Takv {
            device: "Pixel".into(),
            platform: "ATAK-CIV".into(),
            os: "34".into(),
            version: "4.10".into(),
        }),
        track: Some(Track {
            speed: 3.25,
            course: 270.0,
        }),
    }
}

#[test]
fn k18_typed_elements_first_in_fixed_order_with_fixed_attributes() {
    let mut detail = all_typed();
    detail.xml_detail = "<remarks>hi</remarks>".into();
    let ev = from_tak_message(&with_detail(detail), &Limits::default()).unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    assert_eq!(
        child_names(detail),
        [
            "contact",
            "__group",
            "precisionlocation",
            "status",
            "takv",
            "track",
            "remarks"
        ]
    );
    let got: Vec<_> = detail.child_elements().map(attrs).collect();
    assert_eq!(
        got,
        [
            pairs(&[("callsign", "A"), ("endpoint", "*:-1:stcp")]),
            pairs(&[("name", "Cyan"), ("role", "Team Member")]),
            pairs(&[("geopointsrc", "GPS"), ("altsrc", "DTED0")]),
            pairs(&[("battery", "87")]),
            pairs(&[
                ("device", "Pixel"),
                ("platform", "ATAK-CIV"),
                ("os", "34"),
                ("version", "4.10")
            ]),
            pairs(&[("speed", "3.25"), ("course", "270.0")]),
            vec![],
        ]
    );

    // `callsign` always, `endpoint` only when non-empty; empty messages
    // still produce their element with empty values.
    let ev = from_tak_message(
        &with_detail(Detail {
            contact: Some(Contact::default()),
            group: Some(Group::default()),
            ..Detail::default()
        }),
        &Limits::default(),
    )
    .unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    let got: Vec<_> = detail.child_elements().map(attrs).collect();
    assert_eq!(
        got,
        [
            pairs(&[("callsign", "")]),
            pairs(&[("name", ""), ("role", "")])
        ]
    );

    // The typed round trip is exact.
    let back = p2p_event(&with_detail(all_typed())).unwrap();
    assert_eq!(back.detail, Some(all_typed()));
}

#[test]
fn k19_unsigned_fields_rendered_as_java_signed_values() {
    let msg = with_detail(Detail {
        status: Some(Status { battery: u32::MAX }),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let status = child(child(ev.document().root(), "detail").unwrap(), "status").unwrap();
    assert_eq!(status.attribute("battery"), Some("-1"));

    let msg = with_detail(Detail {
        status: Some(Status { battery: 1 << 31 }),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let status = child(child(ev.document().root(), "detail").unwrap(), "status").unwrap();
    assert_eq!(status.attribute("battery"), Some("-2147483648"));

    // sendTime >= 2^63 is a negative long: a date before 1970.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            send_time: u64::MAX,
            start_time: 1 << 63,
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let root = ev.document().root();
    assert_eq!(root.attribute("time"), Some("1969-12-31T23:59:59Z"));
    assert_eq!(root.attribute("start"), Some("-292275055-05-16T16:47:04Z"));
}

#[test]
fn k20_xml_detail_overrides_typed_and_moves_to_the_end() {
    // The plan's example: typed contact A, xmlDetail contact B.
    let msg = with_detail(Detail {
        xml_detail: r#"<contact callsign="B"/>"#.into(),
        contact: Some(Contact {
            callsign: "A".into(),
            endpoint: String::new(),
        }),
        group: Some(Group {
            name: "Cyan".into(),
            role: "r".into(),
        }),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    assert_eq!(child_names(detail), ["__group", "contact"]);
    assert_eq!(
        child(detail, "contact").unwrap().attribute("callsign"),
        Some("B")
    );

    // Two xmlDetail contacts: only the last survives.
    let msg = with_detail(Detail {
        xml_detail: r#"<contact callsign="B"/><remarks>r</remarks><contact callsign="C"/>"#.into(),
        contact: Some(Contact {
            callsign: "A".into(),
            endpoint: String::new(),
        }),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    assert_eq!(child_names(detail), ["remarks", "contact"]);
    assert_eq!(
        child(detail, "contact").unwrap().attribute("callsign"),
        Some("C")
    );

    // Non-typed names are never removed; text between elements is dropped.
    let msg = with_detail(Detail {
        xml_detail: "<remarks>1</remarks> text <remarks>2</remarks><!-- c -->".into(),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    assert_eq!(child_names(detail), ["remarks", "remarks"]);
    assert!(detail.children().all(|n| matches!(n, NodeKind::Element(_))));

    // Text-only xmlDetail adds nothing.
    let msg = with_detail(Detail {
        xml_detail: "just text".into(),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    assert!(
        !child(ev.document().root(), "detail")
            .unwrap()
            .has_children()
    );

    // A prefixed typed name overrides by local name.
    let msg = with_detail(Detail {
        xml_detail: r#"<x:contact xmlns:x="urn:x" callsign="B"/>"#.into(),
        contact: Some(Contact {
            callsign: "A".into(),
            endpoint: String::new(),
        }),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let detail = child(ev.document().root(), "detail").unwrap();
    assert_eq!(child_names(detail), ["x:contact"]);
}

#[test]
fn k21_malformed_xml_detail_rejects_the_event() {
    for bad in [
        "<a>",
        "</detail><detail>",
        "<x:a/>",
        "<!DOCTYPE a><a/>",
        "<a>&ent;</a>",
        "<?xml version=\"1.0\"?><a/>",
        "<a b='1' b='2'/>",
        "&#0;",
    ] {
        let msg = with_detail(Detail {
            xml_detail: bad.into(),
            ..Detail::default()
        });
        let err = from_tak_message(&msg, &Limits::default()).unwrap_err();
        assert_eq!(err.kind(), ConvertErrorKind::XmlDetail, "{bad}");
        assert!(err.xml_error().is_some(), "{bad}");
    }

    // Over max_xml_detail_bytes: rejected before parsing.
    let limits = Limits {
        max_xml_detail_bytes: 10,
        ..Limits::default()
    };
    let msg = with_detail(Detail {
        xml_detail: "<remarks>12345</remarks>".into(),
        ..Detail::default()
    });
    let err = from_tak_message(&msg, &limits).unwrap_err();
    assert_eq!(err.kind(), ConvertErrorKind::XmlDetail);
    assert_eq!(
        err.xml_error().map(|e| e.kind().as_str()),
        Some("xml.limit.xml_detail_bytes")
    );

    // xmlDetail elements count against the event's node limit.
    let limits = Limits {
        max_nodes: 10,
        ..Limits::default()
    };
    let msg = with_detail(Detail {
        xml_detail: "<a/>".repeat(8),
        ..Detail::default()
    });
    let err = from_tak_message(&msg, &limits).unwrap_err();
    assert_eq!(err.kind(), ConvertErrorKind::XmlDetail);
}

#[test]
fn k16_numeric_cache_preferred_with_minus_one_as_unset() {
    // P2X shows whole seconds; P2P keeps the milliseconds.
    let msg = TakMessage {
        cot_event: Some(full_event()),
        ..TakMessage::default()
    };
    let back = p2p_event(&msg).unwrap();
    assert_eq!(back.send_time, 1_761_588_298_123);
    assert_eq!(back.stale_time, 1_761_588_358_999);
    assert_eq!(back.ce, 9.9);

    // sendTime u64::MAX is -1 ("unset"): the document string is parsed
    // instead, 1969-12-31T23:59:59Z = -1000 ms.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            send_time: u64::MAX,
            start_time: u64::MAX - 1,
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let back = p2p_event(&msg).unwrap();
    assert_eq!(back.send_time, u64::MAX - 999);
    assert_eq!(back.start_time, u64::MAX - 1);

    // Doubles not representable by the printed text survive via the cache;
    // -1.0 falls back to the text, which gives the same value.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            lat: 0.1 + 0.2,
            lon: -1.0,
            detail: Some(Detail {
                status: Some(Status { battery: u32::MAX }),
                track: Some(Track {
                    speed: -1.0,
                    course: f64::NAN,
                }),
                ..Detail::default()
            }),
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let back = p2p_event(&msg).unwrap();
    assert_eq!(back.lat.to_bits(), (0.1_f64 + 0.2).to_bits());
    assert_eq!(back.lon, -1.0);
    let detail = back.detail.unwrap();
    assert_eq!(detail.status, Some(Status { battery: u32::MAX }));
    let track = detail.track.unwrap();
    assert_eq!(track.speed, -1.0);
    assert!(track.course.is_nan());

    // Battery, speed and course from the cache win even when an xmlDetail
    // element replaced the typed one.
    let msg = with_detail(Detail {
        xml_detail: r#"<status battery="5"/><track speed="7" course="8"/>"#.into(),
        status: Some(Status { battery: 90 }),
        track: Some(Track {
            speed: 1.5,
            course: 2.5,
        }),
        ..Detail::default()
    });
    let detail = p2p_event(&msg).unwrap().detail.unwrap();
    assert_eq!(detail.status, Some(Status { battery: 90 }));
    assert_eq!(
        detail.track,
        Some(Track {
            speed: 1.5,
            course: 2.5
        })
    );

    // No Status/Track in the proto: the xmlDetail values are parsed.
    let msg = with_detail(Detail {
        xml_detail: r#"<status battery="5"/><track speed="7" course="8"/>"#.into(),
        ..Detail::default()
    });
    let detail = p2p_event(&msg).unwrap().detail.unwrap();
    assert_eq!(detail.status, Some(Status { battery: 5 }));
    assert_eq!(
        detail.track,
        Some(Track {
            speed: 7.0,
            course: 8.0
        })
    );

    // An event parsed from XML has no cache.
    let ev = x2p_event(&format!(
        r#"<event time="{T0}"><point lat="1" lon="2" hae="3" ce="4" le="5"/></event>"#
    ))
    .unwrap();
    assert_eq!(ev.send_time, T0_MS);
}

#[test]
fn k22_creation_and_submission_carried_when_positive_signed() {
    let msg = TakMessage {
        cot_event: Some(full_event()),
        submission_time: 5,
        creation_time: 6,
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    assert_eq!((ev.creation_ms(), ev.submission_ms()), (Some(6), Some(5)));
    let back = p2p(&msg).unwrap();
    assert_eq!((back.creation_time, back.submission_time), (6, 5));

    // >= 2^63 is a negative long: not carried.
    let msg = TakMessage {
        cot_event: Some(full_event()),
        submission_time: u64::MAX,
        creation_time: 1 << 63,
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    assert_eq!((ev.creation_ms(), ev.submission_ms()), (None, None));
    let back = p2p(&msg).unwrap();
    assert_eq!((back.creation_time, back.submission_time), (0, 0));
}

#[test]
fn c12_message_without_cot_event_uses_the_default_event() {
    let msg = TakMessage {
        tak_control: Some(TakControl {
            min_proto_version: 1,
            max_proto_version: 1,
        }),
        ..TakMessage::default()
    };
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let root = ev.document().root();
    assert_eq!(root.attribute("uid"), Some(""));
    assert_eq!(root.attribute("time"), Some("1970-01-01T00:00:00Z"));
    assert_eq!(child_names(root), ["point", "detail"]);
    // The control message is not carried.
    let back = p2p(&msg).unwrap();
    assert_eq!(back.tak_control, None);
    assert_eq!(
        back.cot_event,
        Some(ProtoEvent {
            detail: Some(Detail::default()),
            ..ProtoEvent::default()
        })
    );
}

#[test]
fn dev02_xml_detail_is_read_as_utf8() {
    let msg = with_detail(Detail {
        xml_detail: "<remarks>caf\u{e9} \u{1F600}</remarks>".into(),
        ..Detail::default()
    });
    let ev = from_tak_message(&msg, &Limits::default()).unwrap();
    let remarks = child(child(ev.document().root(), "detail").unwrap(), "remarks").unwrap();
    let text: String = remarks
        .children()
        .filter_map(|n| match n {
            NodeKind::Text(t) => Some(t),
            _ => None,
        })
        .collect();
    assert_eq!(text, "caf\u{e9} \u{1F600}");
    let back = p2p_event(&msg).unwrap();
    assert!(
        back.detail
            .unwrap()
            .xml_detail
            .contains("caf\u{e9} \u{1F600}")
    );
}

#[test]
fn p2x_strings_xml_cannot_carry_fail_the_build() {
    // Capture-determined (see impl notes): the reference writes such a
    // value unchecked; TAK-RS refuses to build an unparseable document.
    let msg = TakMessage {
        cot_event: Some(ProtoEvent {
            uid: "a\u{0}b".into(),
            ..ProtoEvent::default()
        }),
        ..TakMessage::default()
    };
    let err = from_tak_message(&msg, &Limits::default()).unwrap_err();
    assert_eq!(err.kind(), ConvertErrorKind::Build);
    assert_eq!(err.kind().as_str(), "convert.build");
}

// ---------------------------------------------------------------------------
// C. Wire codec and framing (tak-proto, exercised through its public API)
// ---------------------------------------------------------------------------

#[test]
fn k23_schema_types_reachable_by_package() {
    // The three packages the conversion and later phases use resolve, and
    // their defaults encode to nothing (proto3). Names and tags are checked
    // in tak-proto's schema tests.
    assert_eq!(encode_tak_message(&TakMessage::default()), Vec::<u8>::new());
    let _payload = tak_proto::cop::BinaryPayload::default();
    let _event = tak_proto::atakmap::FederatedEvent::default();
}

#[test]
fn k24_x2p_encoding_in_tag_order_with_defaults_omitted() {
    let xml = format!(
        r#"<event uid="u" type="t" how="h" time="{T0}" start="{T0}" stale="{T0}" caveat="c" releasableTo="r"><point lat="1" lon="0" hae="0" ce="0" le="0"/><detail/></event>"#
    );
    let msg = x2p(&xml).unwrap();
    let bytes = encode_tak_message(&msg);
    // cotEvent's last three fields are detail (15), caveat (16) and
    // releaseableTo (17): ... 7a 00 82 01 01 'c' 8a 01 01 'r'.
    assert!(bytes.ends_with(&[0x7a, 0, 0x82, 1, 1, b'c', 0x8a, 1, 1, b'r']));
}

#[test]
fn c08_negative_zero_from_xml_is_written() {
    let msg = x2p(r#"<event><point lat="-0.0" lon="0" hae="0" ce="0" le="0"/></event>"#).unwrap();
    let bytes = encode_tak_message(&msg);
    // cotEvent { lat (10, fixed64) = -0.0 }
    assert_eq!(bytes, [0x12, 9, 0x51, 0, 0, 0, 0, 0, 0, 0, 0x80]);
}

#[test]
fn k25_unknown_fields_are_not_forwarded() {
    // TakMessage { cotEvent { uid "u", field 99 varint 1 } }
    let input = [0x12, 6, 0x2a, 1, b'u', 0x98, 0x06, 1];
    let msg = decode_tak_message(&input, MAX_MESSAGE_BYTES_DEFAULT).unwrap();
    let out = encode_tak_message(&p2p(&msg).unwrap());
    let back = decode_tak_message(&out, MAX_MESSAGE_BYTES_DEFAULT).unwrap();
    assert_eq!(back.cot_event.unwrap().uid, "u");
    assert!(!out.windows(2).any(|w| w == [0x98, 0x06]));
}

#[test]
fn k26_invalid_utf8_string_rejected() {
    // TakMessage { cotEvent { uid = FF } }
    let input = [0x12, 3, 0x2a, 1, 0xff];
    assert!(decode_tak_message(&input, MAX_MESSAGE_BYTES_DEFAULT).is_err());
}

#[test]
fn k27_k28_stream_frame_round_trip() {
    let msg = x2p(r#"<event uid="u"/>"#).unwrap();
    let payload = encode_tak_message(&msg);
    let mut frame = Vec::new();
    encode_stream_frame(&payload, MAX_MESSAGE_BYTES_DEFAULT, &mut frame).unwrap();
    assert_eq!(frame.first(), Some(&MAGIC));
    assert_eq!(frame.get(1), Some(&u8::try_from(payload.len()).unwrap()));
    let decoded = decode_stream_frame(&frame, MAX_MESSAGE_BYTES_DEFAULT)
        .unwrap()
        .unwrap();
    assert_eq!(decoded.payload_of(&frame), Some(payload.as_slice()));
    // A truncated frame needs more bytes.
    let short = frame.get(..frame.len() - 1).unwrap();
    assert!(
        decode_stream_frame(short, MAX_MESSAGE_BYTES_DEFAULT)
            .unwrap()
            .is_none()
    );
}

#[test]
fn k29_mesh_datagram_round_trip() {
    let payload = encode_tak_message(&x2p(r#"<event uid="u"/>"#).unwrap());
    let mut datagram = Vec::new();
    encode_mesh(&payload, &mut datagram);
    assert_eq!(datagram.get(..3), Some(&[MAGIC, 1, MAGIC][..]));
    assert_eq!(decode_mesh(&datagram).unwrap(), payload.as_slice());
}
