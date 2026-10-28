// SPDX-License-Identifier: Apache-2.0

//! `encode_tak_message` writes the bytes protobuf-java writes (parity-map
//! K-24, C-08): prost's encoding, except that a `double` holding `-0.0` is
//! written, because protobuf-java's proto3 code omits a double only when its
//! bit pattern is zero.
//!
//! The `-0.0` expectation comes from a black-box run of protobuf-java's own
//! proto3 code (`DoubleValue` with `-0.0` encodes as
//! `09 00 00 00 00 00 00 00 80`; `0.0` encodes as nothing). The golden case
//! `x2p-point-negative-zero` confirms it for the reference.
//!
//! Fixed case count and RNG seed, so a run is reproducible.

use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use prost::Message as _;
use prost::encoding::{WireType, decode_key, decode_varint};
use tak_proto::v1::{
    Contact, CotEvent, Detail, Group, PrecisionLocation, Status, TakControl, TakMessage, Takv,
    Track,
};
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message};

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

const NEG_ZERO: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0x80];

#[test]
fn c08_negative_zero_point_double_is_written() {
    let msg = TakMessage {
        cot_event: Some(CotEvent {
            lat: -0.0,
            ..Default::default()
        }),
        ..Default::default()
    };
    // TakMessage.cotEvent (2, len 9) { lat (10, fixed64) = -0.0 }
    let mut expected = vec![0x12, 9, 0x51];
    expected.extend_from_slice(&NEG_ZERO);
    assert_eq!(encode_tak_message(&msg), expected);
    // prost alone omits it, which is why the hand encoder exists.
    assert_eq!(msg.encode_to_vec(), [0x12, 0]);
}

#[test]
fn c08_every_negative_zero_double_is_written() {
    let msg = TakMessage {
        cot_event: Some(CotEvent {
            lat: -0.0,
            lon: -0.0,
            hae: -0.0,
            ce: -0.0,
            le: -0.0,
            detail: Some(Detail {
                track: Some(Track {
                    speed: -0.0,
                    course: -0.0,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let bytes = encode_tak_message(&msg);
    let mut track = vec![0x09];
    track.extend_from_slice(&NEG_ZERO);
    track.push(0x11);
    track.extend_from_slice(&NEG_ZERO);
    let mut detail = vec![0x3a, 18];
    detail.extend_from_slice(&track);
    let mut event = Vec::new();
    for key in [0x51, 0x59, 0x61, 0x69, 0x71] {
        event.push(key);
        event.extend_from_slice(&NEG_ZERO);
    }
    event.extend_from_slice(&[0x7a, 20]);
    event.extend_from_slice(&detail);
    let mut expected = vec![0x12, u8::try_from(event.len()).unwrap()];
    expected.extend_from_slice(&event);
    assert_eq!(bytes, expected);

    // The decoded values keep their sign.
    let back = decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT).unwrap();
    let ev = back.cot_event.unwrap();
    assert!(ev.lat.is_sign_negative() && ev.le.is_sign_negative());
    let t = ev.detail.unwrap().track.unwrap();
    assert!(t.speed.is_sign_negative() && t.course.is_sign_negative());
}

#[test]
fn c08_positive_zero_is_omitted_and_empty_messages_are_kept() {
    let msg = TakMessage {
        cot_event: Some(CotEvent {
            lat: 0.0,
            detail: Some(Detail {
                track: Some(Track::default()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    // cotEvent { detail { track {} } }
    assert_eq!(encode_tak_message(&msg), [0x12, 4, 0x7a, 2, 0x3a, 0]);
    assert_eq!(encode_tak_message(&msg), msg.encode_to_vec());
}

#[test]
fn k24_fields_in_tag_order_caveat_and_releaseable_to_after_detail() {
    let msg = TakMessage {
        tak_control: Some(TakControl {
            min_proto_version: 1,
            max_proto_version: 1,
        }),
        cot_event: Some(CotEvent {
            r#type: "t".into(),
            access: "a".into(),
            qos: "q".into(),
            opex: "o".into(),
            caveat: "c".into(),
            releaseable_to: "r".into(),
            uid: "u".into(),
            send_time: 1,
            start_time: 2,
            stale_time: 3,
            how: "h".into(),
            lat: 1.0,
            lon: 2.0,
            hae: 3.0,
            ce: 4.0,
            le: 5.0,
            detail: Some(Detail::default()),
        }),
        submission_time: u64::MAX,
        creation_time: 9,
    };
    let bytes = encode_tak_message(&msg);
    assert_eq!(bytes, msg.encode_to_vec());
    // Event field order 1..=17 (15 before 16 and 17).
    let ev = msg.cot_event.as_ref().unwrap().encode_to_vec();
    assert_eq!(tags(&ev), Some((1..=17).collect::<Vec<u32>>()));
}

/// The field numbers of `buf`, in order.
fn tags(mut buf: &[u8]) -> Option<Vec<u32>> {
    let mut out = Vec::new();
    while !buf.is_empty() {
        let (tag, wire) = decode_key(&mut buf).ok()?;
        let skip = match wire {
            WireType::Varint => {
                decode_varint(&mut buf).ok()?;
                0
            }
            WireType::SixtyFourBit => 8,
            WireType::LengthDelimited => usize::try_from(decode_varint(&mut buf).ok()?).ok()?,
            _ => return None,
        };
        buf = buf.get(skip..)?;
        out.push(tag);
    }
    Some(out)
}

fn small_string() -> impl Strategy<Value = String> {
    prop_oneof![Just(String::new()), "[a-z\u{e9}]{1,6}"]
}

fn any_double() -> impl Strategy<Value = f64> {
    prop_oneof![
        Just(0.0),
        Just(-0.0),
        Just(f64::NAN),
        Just(-1.0),
        any::<f64>(),
        any::<u64>().prop_map(f64::from_bits),
    ]
}

fn any_detail() -> impl Strategy<Value = Detail> {
    (
        small_string(),
        proptest::option::of((small_string(), small_string())),
        proptest::option::of((small_string(), small_string())),
        proptest::option::of((small_string(), small_string())),
        proptest::option::of(any::<u32>()),
        proptest::option::of((small_string(), small_string())),
        proptest::option::of((any_double(), any_double())),
    )
        .prop_map(|(xml, contact, group, pl, status, takv, track)| Detail {
            xml_detail: xml,
            contact: contact.map(|(endpoint, callsign)| Contact { endpoint, callsign }),
            group: group.map(|(name, role)| Group { name, role }),
            precision_location: pl.map(|(geopointsrc, altsrc)| PrecisionLocation {
                geopointsrc,
                altsrc,
            }),
            status: status.map(|battery| Status { battery }),
            takv: takv.map(|(device, os)| Takv {
                device,
                platform: "p".into(),
                os,
                version: String::new(),
            }),
            track: track.map(|(speed, course)| Track { speed, course }),
        })
}

fn any_event() -> impl Strategy<Value = CotEvent> {
    (
        (small_string(), small_string(), small_string()),
        (any::<u64>(), prop_oneof![Just(0_u64), any::<u64>()]),
        (
            any_double(),
            any_double(),
            any_double(),
            any_double(),
            any_double(),
        ),
        proptest::option::of(any_detail()),
        (small_string(), small_string()),
    )
        .prop_map(
            |((t, uid, how), (send, stale), (lat, lon, hae, ce, le), detail, (caveat, rel))| {
                CotEvent {
                    r#type: t,
                    uid,
                    how,
                    send_time: send,
                    stale_time: stale,
                    lat,
                    lon,
                    hae,
                    ce,
                    le,
                    detail,
                    caveat,
                    releaseable_to: rel,
                    ..Default::default()
                }
            },
        )
}

fn any_message() -> impl Strategy<Value = TakMessage> {
    (
        proptest::option::of((any::<u32>(), any::<u32>())),
        proptest::option::of(any_event()),
        any::<u64>(),
        prop_oneof![Just(0_u64), any::<u64>()],
    )
        .prop_map(|(control, ev, submission, creation)| TakMessage {
            tak_control: control.map(|(min_proto_version, max_proto_version)| TakControl {
                min_proto_version,
                max_proto_version,
            }),
            cot_event: ev,
            submission_time: submission,
            creation_time: creation,
        })
}

/// The message with every `-0.0` double replaced by `0.0` (the only values
/// where the two encoders differ).
fn without_negative_zero(mut msg: TakMessage) -> TakMessage {
    let fix = |d: &mut f64| {
        if d.to_bits() == (-0.0_f64).to_bits() {
            *d = 0.0;
        }
    };
    if let Some(ev) = &mut msg.cot_event {
        for d in [
            &mut ev.lat,
            &mut ev.lon,
            &mut ev.hae,
            &mut ev.ce,
            &mut ev.le,
        ] {
            fix(d);
        }
        if let Some(t) = ev.detail.as_mut().and_then(|d| d.track.as_mut()) {
            fix(&mut t.speed);
            fix(&mut t.course);
        }
    }
    msg
}

proptest! {
    #![proptest_config(config())]

    /// Without `-0.0` the hand encoder is byte-identical to prost.
    #[test]
    fn prop_encode_matches_prost_without_negative_zero(msg in any_message()) {
        let msg = without_negative_zero(msg);
        prop_assert_eq!(encode_tak_message(&msg), msg.encode_to_vec());
    }

    /// Decoding the output and encoding again gives the same bytes, and the
    /// bytes decode under prost (the length prefixes are right).
    #[test]
    fn prop_encode_decode_encode_is_stable(msg in any_message()) {
        let bytes = encode_tak_message(&msg);
        let back = decode_tak_message(&bytes, usize::MAX).unwrap();
        prop_assert_eq!(encode_tak_message(&back), bytes);
    }
}
