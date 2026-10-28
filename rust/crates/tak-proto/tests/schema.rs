// SPDX-License-Identifier: Apache-2.0

//! The generated types match the schemas in `src/takserver-protobuf`
//! (parity-map K-23, K-24) and `decode_tak_message` enforces its bounds
//! (K-25, K-26, DEV-01c, DEV-04).
//!
//! Expected names and field numbers are written from the `.proto` files; a
//! test failing here after an upstream merge means the wire contract changed.

use prost::Message as _;
use prost::Name;
use prost::encoding::{WireType, decode_key, decode_varint};
use tak_proto::v1::{
    Contact, CotEvent, Detail, Group, PrecisionLocation, Status, TakControl, TakMessage, Takv,
    Track,
};
use tak_proto::{
    DecodeError, DecodeErrorKind, MAX_MESSAGE_BYTES_DEFAULT, atakmap, cop, decode_tak_message,
    encode_tak_message, v1,
};

/// Asserts the protobuf full name of each generated message type, and that
/// its default value encodes to nothing (proto3).
macro_rules! assert_names {
    ($($ty:ty => $name:literal),+ $(,)?) => {{
        let mut n = 0_usize;
        $(
            assert_eq!(<$ty as Name>::full_name(), $name);
            assert!(<$ty>::default().encode_to_vec().is_empty(), "{}", $name);
            n += 1;
        )+
        n
    }};
}

#[test]
fn every_message_of_the_fifteen_schemas_is_reachable() {
    let n = assert_names!(
        // takmessage, takcontrol, cotevent, detail and its typed messages
        TakMessage => "atakmap.commoncommo.protobuf.v1.TakMessage",
        TakControl => "atakmap.commoncommo.protobuf.v1.TakControl",
        CotEvent => "atakmap.commoncommo.protobuf.v1.CotEvent",
        Detail => "atakmap.commoncommo.protobuf.v1.Detail",
        Contact => "atakmap.commoncommo.protobuf.v1.Contact",
        Group => "atakmap.commoncommo.protobuf.v1.Group",
        PrecisionLocation => "atakmap.commoncommo.protobuf.v1.PrecisionLocation",
        Status => "atakmap.commoncommo.protobuf.v1.Status",
        Takv => "atakmap.commoncommo.protobuf.v1.Takv",
        Track => "atakmap.commoncommo.protobuf.v1.Track",
        // message, missionannouncement
        v1::Message => "atakmap.commoncommo.protobuf.v1.Message",
        v1::MissionAnnouncement => "atakmap.commoncommo.protobuf.v1.MissionAnnouncement",
        // binarypayload
        cop::BinaryPayload => "gov.tak.cop.proto.v1.BinaryPayload",
        // fig
        atakmap::FederatedEvent => "com.atakmap.FederatedEvent",
        atakmap::GeoEvent => "com.atakmap.GeoEvent",
        atakmap::BinaryBlob => "com.atakmap.BinaryBlob",
        atakmap::ContactListEntry => "com.atakmap.ContactListEntry",
        atakmap::Empty => "com.atakmap.Empty",
        atakmap::Identity => "com.atakmap.Identity",
        atakmap::Subscription => "com.atakmap.Subscription",
        atakmap::ClientHealth => "com.atakmap.ClientHealth",
        atakmap::ServerHealth => "com.atakmap.ServerHealth",
        atakmap::Rol => "com.atakmap.ROL",
        atakmap::FederateGroups => "com.atakmap.FederateGroups",
        atakmap::TakServerVersion => "com.atakmap.TakServerVersion",
        atakmap::FederateProvenance => "com.atakmap.FederateProvenance",
        atakmap::FederateHops => "com.atakmap.FederateHops",
        atakmap::FederateGroupHopLimit => "com.atakmap.FederateGroupHopLimit",
        atakmap::FederateGroupHopLimits => "com.atakmap.FederateGroupHopLimits",
        atakmap::FederateTokenResponse => "com.atakmap.FederateTokenResponse",
        // streaminginput
        atakmap::ClientSubscription => "com.atakmap.ClientSubscription",
        atakmap::ClientVersion => "com.atakmap.ClientVersion",
        atakmap::ServerVersion => "com.atakmap.ServerVersion",
    );
    assert_eq!(n, 33);
}

#[test]
fn fig_enums_are_reachable_with_their_schema_names() {
    assert_eq!(atakmap::Crud::Delete.as_str_name(), "DELETE");
    assert_eq!(atakmap::Crud::Invalid as i32, 0);
    assert_eq!(atakmap::BinaryTypes::Image.as_str_name(), "IMAGE");
    assert_eq!(atakmap::BinaryTypes::Empty as i32, 0);
    // Nested enums are reachable through their message's module.
    let _ = atakmap::identity::ConnectionType::default();
    let _ = atakmap::client_health::ServingStatus::default();
    let _ = atakmap::server_health::ServingStatus::default();
}

#[test]
fn cross_package_reference_resolves() {
    // message.proto refers to gov.tak.cop.proto.v1.BinaryPayload.
    let msg = v1::Message {
        bloads: vec![cop::BinaryPayload {
            r#type: "t".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let back = v1::Message::decode(msg.encode_to_vec().as_slice()).unwrap();
    assert_eq!(back, msg);
}

/// Top-level `(field number, wire type)` keys of an encoded message, in order;
/// `None` if the bytes are not a well-formed field sequence.
fn keys(mut buf: &[u8]) -> Option<Vec<(u32, WireType)>> {
    let mut out = Vec::new();
    while !buf.is_empty() {
        let (tag, wire) = decode_key(&mut buf).ok()?;
        let skip = match wire {
            WireType::Varint => {
                decode_varint(&mut buf).ok()?;
                0
            }
            WireType::SixtyFourBit => 8,
            WireType::ThirtyTwoBit => 4,
            WireType::LengthDelimited => usize::try_from(decode_varint(&mut buf).ok()?).ok()?,
            _ => return None,
        };
        buf = buf.get(skip..)?;
        out.push((tag, wire));
    }
    Some(out)
}

/// The key of `msg` when it encodes to exactly one field, else `None`.
fn only_key(msg: &impl prost::Message) -> Option<(u32, WireType)> {
    match keys(&msg.encode_to_vec())?.as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

const LEN: WireType = WireType::LengthDelimited;
const VARINT: WireType = WireType::Varint;
const F64: WireType = WireType::SixtyFourBit;

fn s() -> String {
    "x".to_owned()
}

#[test]
fn tak_message_field_numbers() {
    let m = |f: fn(&mut TakMessage)| {
        let mut msg = TakMessage::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(m(|x| x.tak_control = Some(TakControl::default())), (1, LEN));
    assert_eq!(m(|x| x.cot_event = Some(CotEvent::default())), (2, LEN));
    assert_eq!(m(|x| x.submission_time = 1), (3, VARINT));
    assert_eq!(m(|x| x.creation_time = 1), (4, VARINT));

    let c = |f: fn(&mut TakControl)| {
        let mut msg = TakControl::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(c(|x| x.min_proto_version = 1), (1, VARINT));
    assert_eq!(c(|x| x.max_proto_version = 1), (2, VARINT));
}

#[test]
fn cot_event_field_numbers() {
    let m = |f: fn(&mut CotEvent)| {
        let mut msg = CotEvent::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(m(|x| x.r#type = s()), (1, LEN));
    assert_eq!(m(|x| x.access = s()), (2, LEN));
    assert_eq!(m(|x| x.qos = s()), (3, LEN));
    assert_eq!(m(|x| x.opex = s()), (4, LEN));
    assert_eq!(m(|x| x.uid = s()), (5, LEN));
    assert_eq!(m(|x| x.send_time = 1), (6, VARINT));
    assert_eq!(m(|x| x.start_time = 1), (7, VARINT));
    assert_eq!(m(|x| x.stale_time = 1), (8, VARINT));
    assert_eq!(m(|x| x.how = s()), (9, LEN));
    assert_eq!(m(|x| x.lat = 1.0), (10, F64));
    assert_eq!(m(|x| x.lon = 1.0), (11, F64));
    assert_eq!(m(|x| x.hae = 1.0), (12, F64));
    assert_eq!(m(|x| x.ce = 1.0), (13, F64));
    assert_eq!(m(|x| x.le = 1.0), (14, F64));
    assert_eq!(m(|x| x.detail = Some(Detail::default())), (15, LEN));
    assert_eq!(m(|x| x.caveat = s()), (16, LEN));
    assert_eq!(m(|x| x.releaseable_to = s()), (17, LEN));
}

#[test]
fn detail_and_typed_message_field_numbers() {
    let d = |f: fn(&mut Detail)| {
        let mut msg = Detail::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(d(|x| x.xml_detail = s()), (1, LEN));
    assert_eq!(d(|x| x.contact = Some(Contact::default())), (2, LEN));
    assert_eq!(d(|x| x.group = Some(Group::default())), (3, LEN));
    assert_eq!(
        d(|x| x.precision_location = Some(PrecisionLocation::default())),
        (4, LEN)
    );
    assert_eq!(d(|x| x.status = Some(Status::default())), (5, LEN));
    assert_eq!(d(|x| x.takv = Some(Takv::default())), (6, LEN));
    assert_eq!(d(|x| x.track = Some(Track::default())), (7, LEN));

    let contact = |f: fn(&mut Contact)| {
        let mut msg = Contact::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(contact(|x| x.endpoint = s()), (1, LEN));
    assert_eq!(contact(|x| x.callsign = s()), (2, LEN));

    let group = |f: fn(&mut Group)| {
        let mut msg = Group::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(group(|x| x.name = s()), (1, LEN));
    assert_eq!(group(|x| x.role = s()), (2, LEN));

    let pl = |f: fn(&mut PrecisionLocation)| {
        let mut msg = PrecisionLocation::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(pl(|x| x.geopointsrc = s()), (1, LEN));
    assert_eq!(pl(|x| x.altsrc = s()), (2, LEN));

    assert_eq!(only_key(&Status { battery: 1 }), Some((1, VARINT)));

    let takv = |f: fn(&mut Takv)| {
        let mut msg = Takv::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(takv(|x| x.device = s()), (1, LEN));
    assert_eq!(takv(|x| x.platform = s()), (2, LEN));
    assert_eq!(takv(|x| x.os = s()), (3, LEN));
    assert_eq!(takv(|x| x.version = s()), (4, LEN));

    let track = |f: fn(&mut Track)| {
        let mut msg = Track::default();
        f(&mut msg);
        only_key(&msg).unwrap()
    };
    assert_eq!(track(|x| x.speed = 1.0), (1, F64));
    assert_eq!(track(|x| x.course = 1.0), (2, F64));
}

#[test]
fn caveat_and_releaseable_to_are_written_after_detail() {
    // protobuf-java writes fields in field-number order, not declaration
    // order (cotevent.proto declares 16 and 17 before 5); so must we (K-24).
    let ev = CotEvent {
        r#type: "a-f-G".into(),
        uid: "u".into(),
        caveat: "c".into(),
        releaseable_to: "r".into(),
        detail: Some(Detail::default()),
        ..Default::default()
    };
    let bytes = ev.encode_to_vec();
    let tags: Vec<u32> = keys(&bytes).unwrap().into_iter().map(|(t, _)| t).collect();
    assert_eq!(tags, [1, 5, 15, 16, 17]);
    // Field 15 empty, then the two-byte keys of 16 and 17.
    assert!(
        bytes
            .windows(8)
            .any(|w| w == [0x7a, 0x00, 0x82, 0x01, 0x01, b'c', 0x8a, 0x01])
    );
}

#[test]
fn encode_tak_message_matches_prost_and_omits_defaults() {
    assert_eq!(encode_tak_message(&TakMessage::default()), Vec::<u8>::new());
    let msg = TakMessage {
        cot_event: Some(CotEvent {
            uid: "u".into(),
            lat: 1.5,
            ..Default::default()
        }),
        submission_time: 7,
        ..Default::default()
    };
    let bytes = encode_tak_message(&msg);
    assert_eq!(bytes, msg.encode_to_vec());
    assert_eq!(
        keys(&bytes)
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect::<Vec<_>>(),
        [2, 3]
    );
}

// ---- decode_tak_message ----

fn sample() -> TakMessage {
    TakMessage {
        cot_event: Some(CotEvent {
            r#type: "a-f-G-U-C".into(),
            uid: "ANDROID-1".into(),
            how: "m-g".into(),
            send_time: 1_700_000_000_123,
            start_time: 1_700_000_000_123,
            stale_time: 1_700_000_060_123,
            lat: 38.5,
            lon: -77.25,
            hae: 999_999.0,
            ce: 999_999.0,
            le: 999_999.0,
            detail: Some(Detail {
                xml_detail: "<remarks>hi</remarks>".into(),
                contact: Some(Contact {
                    endpoint: "*:-1:stcp".into(),
                    callsign: "ALPHA".into(),
                }),
                status: Some(Status { battery: 88 }),
                track: Some(Track {
                    speed: 1.25,
                    course: 270.0,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        submission_time: 1_700_000_000_200,
        creation_time: 1_700_000_000_150,
        ..Default::default()
    }
}

#[test]
fn decode_round_trips() {
    let msg = sample();
    let bytes = encode_tak_message(&msg);
    assert_eq!(
        decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT),
        Ok(msg)
    );
}

#[test]
fn decode_accepts_exactly_the_limit_and_rejects_one_more() {
    let bytes = encode_tak_message(&sample());
    assert!(decode_tak_message(&bytes, bytes.len()).is_ok());
    let err = decode_tak_message(&bytes, bytes.len() - 1).unwrap_err();
    assert_eq!(
        err,
        DecodeError::TooLarge {
            len: bytes.len(),
            max: bytes.len() - 1
        }
    );
    assert_eq!(err.kind(), DecodeErrorKind::TooLarge);
    assert_eq!(err.kind().as_str(), "proto.too_large");
}

#[test]
fn decode_rejects_oversize_before_looking_at_the_bytes() {
    // Not a valid message at all; the size check alone decides.
    let junk = vec![0xff_u8; 64];
    assert_eq!(
        decode_tak_message(&junk, 63).unwrap_err().kind(),
        DecodeErrorKind::TooLarge
    );
    assert_eq!(
        decode_tak_message(&junk, 64).unwrap_err().kind(),
        DecodeErrorKind::Malformed
    );
}

#[test]
fn default_limit_is_eight_mebibytes() {
    assert_eq!(MAX_MESSAGE_BYTES_DEFAULT, 8_388_608);
}

#[test]
fn decode_rejects_invalid_utf8_in_a_string_field() {
    // K-26: CotEvent.uid (5) holding 0xff. TakMessage.cotEvent is field 2.
    let cot = [0x2a, 0x01, 0xff];
    let mut bytes = vec![0x12, 3];
    bytes.extend_from_slice(&cot);
    let err = decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT).unwrap_err();
    assert_eq!(err.kind(), DecodeErrorKind::Malformed);
    assert_eq!(err.kind().as_str(), "proto.malformed");
}

#[test]
fn decode_rejects_a_known_field_with_the_wrong_wire_type() {
    // DEV-04: TakMessage.submissionTime (3, varint) sent length-delimited.
    let bytes = [0x1a, 0x01, 0x00];
    assert_eq!(
        decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT)
            .unwrap_err()
            .kind(),
        DecodeErrorKind::Malformed
    );
}

#[test]
fn decode_rejects_truncated_input() {
    let bytes = encode_tak_message(&sample());
    for cut in 1..bytes.len() {
        let prefix = bytes.get(..cut).unwrap();
        // Some prefixes end on a field boundary and decode to a smaller
        // message; none may panic, and none may equal the original.
        if let Ok(m) = decode_tak_message(prefix, MAX_MESSAGE_BYTES_DEFAULT) {
            assert_ne!(m, sample());
        }
    }
}

#[test]
fn decode_drops_unknown_fields() {
    // K-25: field 99 (varint) is skipped and not re-encoded.
    let mut bytes = encode_tak_message(&sample());
    bytes.extend_from_slice(&[0x98, 0x06, 0x01]);
    let msg = decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT).unwrap();
    assert_eq!(msg, sample());
    assert_eq!(encode_tak_message(&msg), encode_tak_message(&sample()));
}

#[test]
fn decode_error_display_has_no_payload_text() {
    // A valid uid followed by a string with invalid UTF-8.
    let mut cot = vec![0x2a, 6];
    cot.extend_from_slice(b"SECRET");
    cot.extend_from_slice(&[0x4a, 0x02, b'Z', 0xff]);
    let mut bytes = vec![0x12, u8::try_from(cot.len()).unwrap()];
    bytes.extend_from_slice(&cot);
    let err = decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT).unwrap_err();
    let mut text = err.to_string();
    let mut source = std::error::Error::source(&err);
    while let Some(s) = source {
        text.push_str(&s.to_string());
        source = s.source();
    }
    assert!(!text.contains("SECRET"), "{text}");
}
