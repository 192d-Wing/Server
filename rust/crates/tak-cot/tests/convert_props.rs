// SPDX-License-Identifier: Apache-2.0

//! Properties of the conversion (plan section 8.2): no panic on any input in
//! either direction; proto-to-XML output re-parses; the XML-to-proto result
//! does not change when the event is re-serialized first (the server
//! forwards re-serialized events, K-32); and proto-to-proto conversion is a
//! fixed point after one pass.
//!
//! Fixed case counts and a fixed RNG seed, so a run is reproducible; failure
//! files are not written into the source tree.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use tak_cot::{CotEvent, Limits, ToProtoOptions, from_tak_message, to_tak_message};
use tak_proto::encode_tak_message;
use tak_proto::v1::{
    Contact, CotEvent as ProtoEvent, Detail, Group, PrecisionLocation, Status, TakMessage, Takv,
    Track,
};

/// Seed recorded for reproducibility (plan section 8.2).
const SEED: u64 = 0x7a6b_2026_1027_0005;
const CASES: u32 = 512;

fn config() -> Config {
    Config {
        cases: CASES,
        rng_seed: RngSeed::Fixed(SEED),
        failure_persistence: None,
        ..Config::default()
    }
}

// ---- proto inputs ----

/// Strings with the characters escaping and XML validity care about,
/// including one XML 1.0 cannot carry (U+0000).
fn any_string() -> BoxedStrategy<String> {
    prop_oneof![
        4 => "[a-zA-Z0-9 ._:-]{0,8}",
        1 => "[a-z <>&\"'\t\n\r\u{0}\u{e9}\u{1F600}]{0,6}",
    ]
    .boxed()
}

fn any_double() -> BoxedStrategy<f64> {
    prop_oneof![
        Just(0.0),
        Just(-0.0),
        Just(-1.0),
        Just(999_999.0),
        Just(f64::NAN),
        Just(f64::INFINITY),
        any::<f64>(),
        any::<u64>().prop_map(f64::from_bits),
    ]
    .boxed()
}

fn any_time() -> BoxedStrategy<u64> {
    prop_oneof![
        Just(0_u64),
        Just(u64::MAX),
        Just(1 << 63),
        (0_u64..4_102_444_800_000),
        any::<u64>(),
    ]
    .boxed()
}

/// Pieces an `xmlDetail` is assembled from: typed-name elements (eligible
/// and not), other elements, namespaces, text, comments, and fragments that
/// make the whole malformed.
fn xml_piece() -> BoxedStrategy<String> {
    prop_oneof![
        Just(r#"<contact callsign="X"/>"#.to_owned()),
        Just(r#"<contact callsign="Y" endpoint=""/>"#.to_owned()),
        Just(r#"<contact callsign="Z" phone="1"/>"#.to_owned()),
        Just(r#"<__group name="a" role="b"/>"#.to_owned()),
        Just(r#"<precisionlocation geopointsrc="g" altsrc="a"/>"#.to_owned()),
        Just(r#"<status battery="5"/>"#.to_owned()),
        Just(r#"<status battery="x"/>"#.to_owned()),
        Just(r#"<takv device="d" platform="p" os="o" version="v"/>"#.to_owned()),
        Just(r#"<track speed="1.5" course="nope"/>"#.to_owned()),
        Just("<remarks>hi &amp; bye</remarks>".to_owned()),
        Just(r#"<x:contact xmlns:x="urn:x" callsign="N"/>"#.to_owned()),
        Just(r#"<link uid="a"><b c="d"/></link>"#.to_owned()),
        Just(" text ".to_owned()),
        Just("<!-- c -->".to_owned()),
        Just("<![CDATA[x]]>".to_owned()),
        Just("<a>".to_owned()),
        Just("</detail>".to_owned()),
        Just("&bad;".to_owned()),
        Just("<y:a/>".to_owned()),
        any_string(),
    ]
    .boxed()
}

fn any_xml_detail() -> BoxedStrategy<String> {
    prop_oneof![
        Just(String::new()),
        prop::collection::vec(xml_piece(), 1..6).prop_map(|v| v.concat()),
    ]
    .boxed()
}

fn any_detail() -> BoxedStrategy<Detail> {
    (
        any_xml_detail(),
        proptest::option::of((any_string(), any_string())),
        proptest::option::of((any_string(), any_string())),
        proptest::option::of((any_string(), any_string())),
        proptest::option::of(prop_oneof![Just(u32::MAX), Just(1 << 31), any::<u32>()]),
        proptest::option::of((any_string(), any_string())),
        proptest::option::of((any_double(), any_double())),
    )
        .prop_map(|(xml, contact, group, pl, battery, takv, track)| Detail {
            xml_detail: xml,
            contact: contact.map(|(endpoint, callsign)| Contact { endpoint, callsign }),
            group: group.map(|(name, role)| Group { name, role }),
            precision_location: pl.map(|(geopointsrc, altsrc)| PrecisionLocation {
                geopointsrc,
                altsrc,
            }),
            status: battery.map(|battery| Status { battery }),
            takv: takv.map(|(device, os)| Takv {
                device,
                platform: "ATAK-CIV".into(),
                os,
                version: "4.10".into(),
            }),
            track: track.map(|(speed, course)| Track { speed, course }),
        })
        .boxed()
}

fn any_message() -> BoxedStrategy<TakMessage> {
    let strings = (
        any_string(),
        any_string(),
        any_string(),
        any_string(),
        any_string(),
    );
    let times = (any_time(), any_time(), any_time());
    let point = (
        any_double(),
        any_double(),
        any_double(),
        any_double(),
        any_double(),
    );
    (
        proptest::option::of((strings, times, point, proptest::option::of(any_detail()))),
        any_time(),
        any_time(),
    )
        .prop_map(|(event, submission, creation)| TakMessage {
            tak_control: None,
            cot_event: event.map(
                |(
                    (uid, t, how, access, caveat),
                    (send, start, stale),
                    (lat, lon, hae, ce, le),
                    detail,
                )| {
                    ProtoEvent {
                        r#type: t,
                        access,
                        qos: String::new(),
                        opex: String::new(),
                        caveat,
                        releaseable_to: String::new(),
                        uid,
                        send_time: send,
                        start_time: start,
                        stale_time: stale,
                        how,
                        lat,
                        lon,
                        hae,
                        ce,
                        le,
                        detail,
                    }
                },
            ),
            submission_time: submission,
            creation_time: creation,
        })
        .boxed()
}

// ---- XML inputs ----

fn attr_value() -> BoxedStrategy<String> {
    prop_oneof![
        Just(String::new()),
        Just("2025-10-27T18:04:58Z".to_owned()),
        Just("2025-10-27T18:04:58.123Z".to_owned()),
        Just("292278994-08-17T07:12:56Z".to_owned()),
        Just("-1".to_owned()),
        Just("1.5".to_owned()),
        Just("NaN".to_owned()),
        Just("0x1p3".to_owned()),
        "[a-z0-9 .:+-]{0,8}",
    ]
    .boxed()
}

fn escape(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
}

/// One element with attributes picked from `names`.
fn element(
    names: &'static [&'static str],
    attr_names: &'static [&'static str],
) -> BoxedStrategy<String> {
    (
        prop::sample::select(names),
        prop::collection::vec((prop::sample::select(attr_names), attr_value()), 0..5),
        prop::bool::ANY,
    )
        .prop_map(|(name, attrs, with_child)| {
            let mut seen = Vec::new();
            let mut out = format!("<{name}");
            for (a, v) in attrs {
                if seen.contains(&a) {
                    continue;
                }
                seen.push(a);
                let _ = write!(out, r#" {a}="{}""#, escape(&v));
            }
            if with_child {
                let _ = write!(out, "><x>t</x></{name}>");
            } else {
                out.push_str("/>");
            }
            out
        })
        .boxed()
}

const DETAIL_NAMES: &[&str] = &[
    "contact",
    "__group",
    "precisionlocation",
    "status",
    "takv",
    "track",
    "remarks",
    "link",
    "n:contact",
];
const DETAIL_ATTRS: &[&str] = &[
    "callsign",
    "endpoint",
    "name",
    "role",
    "geopointsrc",
    "altsrc",
    "battery",
    "device",
    "platform",
    "os",
    "version",
    "speed",
    "course",
    "n:callsign",
    "phone",
];
const EVENT_ATTRS: &[&str] = &[
    "uid",
    "type",
    "how",
    "time",
    "start",
    "stale",
    "access",
    "caveat",
    "releasableTo",
    "n:uid",
];
const POINT_ATTRS: &[&str] = &["lat", "lon", "hae", "ce", "le", "n:lat"];

fn any_event_xml() -> BoxedStrategy<String> {
    (
        prop::collection::vec((prop::sample::select(EVENT_ATTRS), attr_value()), 0..8),
        prop::collection::vec(element(&["point"], POINT_ATTRS), 0..3),
        proptest::option::of(prop::collection::vec(
            prop_oneof![
                4 => element(DETAIL_NAMES, DETAIL_ATTRS),
                1 => Just(" text <!-- c -->".to_owned()),
            ],
            0..8,
        )),
    )
        .prop_map(|(attrs, points, detail)| {
            let mut seen = Vec::new();
            let mut out = String::from(r#"<event xmlns:n="urn:n""#);
            for (a, v) in attrs {
                if seen.contains(&a) {
                    continue;
                }
                seen.push(a);
                let _ = write!(out, r#" {a}="{}""#, escape(&v));
            }
            out.push('>');
            for p in points {
                out.push_str(&p);
            }
            if let Some(children) = detail {
                out.push_str("<detail>");
                for c in children {
                    out.push_str(&c);
                }
                out.push_str("</detail>");
            }
            out.push_str("</event>");
            out
        })
        .boxed()
}

/// The encoded X2P result, or `None` when the conversion rejects.
fn x2p_bytes(ev: &CotEvent) -> Option<Vec<u8>> {
    to_tak_message(ev, &ToProtoOptions::default())
        .ok()
        .map(|m| encode_tak_message(&m))
}

/// The encoded P2P result, or `None` when either step rejects.
fn p2p(msg: &TakMessage) -> Option<TakMessage> {
    let ev = from_tak_message(msg, &Limits::default()).ok()?;
    to_tak_message(&ev, &ToProtoOptions::default()).ok()
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn prop_from_tak_message_never_panics(msg in any_message()) {
        let _ = from_tak_message(&msg, &Limits::default());
        let tiny = Limits {
            max_nodes: 8,
            max_xml_detail_bytes: 16,
            max_depth: 3,
            ..Limits::default()
        };
        let _ = from_tak_message(&msg, &tiny);
    }

    #[test]
    fn prop_to_tak_message_never_panics_on_any_text(input in "[<>a-z=\" /:]{0,64}") {
        if let Ok(ev) = CotEvent::parse(&input, &Limits::default()) {
            let _ = to_tak_message(&ev, &ToProtoOptions::default());
        }
    }

    #[test]
    fn prop_x2p_unchanged_by_reserialization(xml in any_event_xml()) {
        let ev = CotEvent::parse(&xml, &Limits::default()).unwrap();
        let first = x2p_bytes(&ev);
        let again = String::from_utf8(ev.to_xml_bytes()).unwrap();
        let ev2 = CotEvent::parse(&again, &Limits::default()).unwrap();
        prop_assert_eq!(first, x2p_bytes(&ev2));
    }

    #[test]
    fn prop_x2p_survives_tiny_xml_detail_cap(xml in any_event_xml(), cap in 0_usize..64) {
        let limits = Limits { max_xml_detail_bytes: cap, ..Limits::default() };
        let ev = CotEvent::parse(&xml, &limits).unwrap();
        if let Ok(msg) = to_tak_message(&ev, &ToProtoOptions::default()) {
            let len = msg.cot_event.and_then(|e| e.detail).map_or(0, |d| d.xml_detail.len());
            prop_assert!(len <= cap);
        }
    }

    #[test]
    fn prop_p2x_output_reparses(msg in any_message()) {
        if let Ok(ev) = from_tak_message(&msg, &Limits::default()) {
            let xml = String::from_utf8(ev.to_xml_bytes()).unwrap();
            let reparsed = CotEvent::parse(&xml, &Limits::default());
            prop_assert!(reparsed.is_ok());
        }
    }

    #[test]
    fn prop_p2p_is_a_fixed_point_after_one_pass(msg in any_message()) {
        if let Some(once) = p2p(&msg) {
            let twice = p2p(&once);
            prop_assert_eq!(
                twice.map(|m| encode_tak_message(&m)),
                Some(encode_tak_message(&once))
            );
        }
    }
}
