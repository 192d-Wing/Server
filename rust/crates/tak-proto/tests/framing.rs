// SPDX-License-Identifier: Apache-2.0

//! Stream and mesh frames (parity-map K-27, K-28, K-29, DEV-01c; plan
//! sections 3.1, 3.4, 8.1, 8.2).

use proptest::prelude::*;
use tak_proto::framing::{
    Frame, FrameError, FrameErrorKind, MAGIC, MAX_VARINT_BYTES, TAK_PROTOCOL_VERSION, decode_mesh,
    decode_stream_frame, encode_mesh, encode_stream_frame,
};
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message};

const MAX: usize = MAX_MESSAGE_BYTES_DEFAULT;

fn frame(msg: &[u8]) -> Result<Vec<u8>, FrameError> {
    let mut out = Vec::new();
    encode_stream_frame(msg, MAX, &mut out)?;
    Ok(out)
}

// ---- stream: encode ----

#[test]
fn stream_frame_layout() {
    assert_eq!(frame(b"abc").unwrap(), [MAGIC, 3, b'a', b'b', b'c']);
    assert_eq!(frame(b"").unwrap(), [MAGIC, 0]);
    // 300 = 0b1_0010_1100 -> 0xac 0x02.
    let big = vec![7_u8; 300];
    let f = frame(&big).unwrap();
    assert_eq!(f.get(..3), Some(&[MAGIC, 0xac, 0x02][..]));
    assert_eq!(f.len(), 303);
}

#[test]
fn stream_encode_appends_and_rejects_over_limit() {
    let mut out = vec![1, 2];
    encode_stream_frame(b"xy", 2, &mut out).unwrap();
    assert_eq!(out, [1, 2, MAGIC, 2, b'x', b'y']);

    let mut out = vec![9];
    let err = encode_stream_frame(b"xyz", 2, &mut out).unwrap_err();
    assert_eq!(err, FrameError::TooLarge { len: 3, max: 2 });
    assert_eq!(out, [9], "output untouched on error");
}

// ---- stream: decode ----

#[test]
fn stream_decode_full_frame() {
    let buf = frame(b"hello").unwrap();
    let f = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    assert_eq!(
        f,
        Frame {
            payload: 2..7,
            consumed: 7
        }
    );
    assert_eq!(f.payload_of(&buf), Some(&b"hello"[..]));
}

#[test]
fn stream_decode_back_to_back_frames() {
    let mut buf = frame(b"one").unwrap();
    buf.extend_from_slice(&frame(b"two!").unwrap());
    let first = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    assert_eq!(first.payload_of(&buf), Some(&b"one"[..]));
    let rest = buf.get(first.consumed..).unwrap();
    let second = decode_stream_frame(rest, MAX).unwrap().unwrap();
    assert_eq!(second.payload_of(rest), Some(&b"two!"[..]));
    assert_eq!(second.consumed, rest.len());
}

#[test]
fn stream_decode_every_strict_prefix_needs_more() {
    let buf = frame(&[0x55; 200]).unwrap();
    for cut in 0..buf.len() {
        assert_eq!(
            decode_stream_frame(buf.get(..cut).unwrap(), MAX),
            Ok(None),
            "prefix of {cut} bytes"
        );
    }
}

#[test]
fn stream_decode_zero_length_frame_is_an_empty_payload() {
    // parity-map D-07: what a connection does with it is Phase 3's decision.
    let buf = [MAGIC, 0, MAGIC];
    let f = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    assert_eq!(f.payload, 2..2);
    assert_eq!(f.consumed, 2);
}

#[test]
fn stream_decode_bad_magic() {
    let err = decode_stream_frame(&[0x3c, 1, 2], MAX).unwrap_err();
    assert_eq!(err, FrameError::BadMagic);
    assert_eq!(err.kind().as_str(), "frame.bad_magic");
}

#[test]
fn stream_decode_rejects_a_sixth_varint_byte() {
    // Five continuation bytes: the varint would need a sixth.
    let buf = [MAGIC, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
    let err = decode_stream_frame(&buf, MAX).unwrap_err();
    assert_eq!(err, FrameError::VarintTooLong);
    assert_eq!(err.kind().as_str(), "frame.varint_too_long");
    // It is rejected as soon as the fifth byte is seen.
    assert_eq!(
        decode_stream_frame(buf.get(..6).unwrap(), MAX),
        Err(FrameError::VarintTooLong)
    );
    // Four continuation bytes are just an incomplete varint.
    assert_eq!(decode_stream_frame(buf.get(..5).unwrap(), MAX), Ok(None));
}

#[test]
fn stream_decode_rejects_a_varint_above_u32() {
    // 2^32 in five bytes.
    let buf = [MAGIC, 0x80, 0x80, 0x80, 0x80, 0x10];
    let err = decode_stream_frame(&buf, usize::MAX).unwrap_err();
    assert_eq!(err, FrameError::VarintOverflow);
    assert_eq!(err.kind().as_str(), "frame.varint_overflow");
    // u32::MAX itself is a valid length (then limited by max_frame).
    let buf = [MAGIC, 0xff, 0xff, 0xff, 0xff, 0x0f];
    assert_eq!(decode_stream_frame(&buf, usize::MAX), Ok(None));
}

#[test]
fn stream_decode_checks_the_limit_before_the_payload_arrives() {
    // Declares 1 MiB + 1 with no payload yet: rejected now, not buffered.
    let mut buf = Vec::new();
    encode_stream_frame(&vec![0; 1_048_577], usize::MAX, &mut buf).unwrap();
    let header = buf.get(..4).unwrap();
    let err = decode_stream_frame(header, 1_048_576).unwrap_err();
    assert_eq!(
        err,
        FrameError::TooLarge {
            len: 1_048_577,
            max: 1_048_576
        }
    );
    assert_eq!(err.kind(), FrameErrorKind::TooLarge);
    assert_eq!(err.kind().as_str(), "frame.too_large");
    // Exactly at the limit is fine.
    assert!(decode_stream_frame(&buf, 1_048_577).unwrap().is_some());
}

#[test]
fn stream_decode_accepts_a_non_minimal_length() {
    // 3 encoded in two bytes; protobuf varint decoders accept this.
    let buf = [MAGIC, 0x83, 0x00, b'a', b'b', b'c'];
    let f = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    assert_eq!(f.payload_of(&buf), Some(&b"abc"[..]));
}

#[test]
fn stream_payload_decodes_as_a_tak_message() {
    let msg = tak_proto::v1::TakMessage {
        creation_time: 42,
        ..Default::default()
    };
    let buf = frame(&tak_proto::encode_tak_message(&msg)).unwrap();
    let f = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    let payload = f.payload_of(&buf).unwrap();
    assert_eq!(decode_tak_message(payload, MAX), Ok(msg));
}

// ---- mesh ----

#[test]
fn mesh_layout_and_round_trip() {
    let mut out = Vec::new();
    encode_mesh(b"msg", &mut out);
    assert_eq!(out, [MAGIC, 0x01, MAGIC, b'm', b's', b'g']);
    assert_eq!(decode_mesh(&out), Ok(&b"msg"[..]));
    assert_eq!(TAK_PROTOCOL_VERSION, 1);

    let mut empty = Vec::new();
    encode_mesh(b"", &mut empty);
    assert_eq!(decode_mesh(&empty), Ok(&b""[..]));
}

#[test]
fn mesh_rejects_a_wrong_version() {
    let err = decode_mesh(&[MAGIC, 0x02, MAGIC, 0x0a]).unwrap_err();
    assert_eq!(err, FrameError::UnsupportedVersion);
    assert_eq!(err.kind().as_str(), "frame.unsupported_version");
    assert_eq!(
        decode_mesh(&[MAGIC, 0x00, MAGIC]),
        Err(FrameError::UnsupportedVersion)
    );
}

#[test]
fn mesh_accepts_a_non_minimal_version() {
    assert_eq!(decode_mesh(&[MAGIC, 0x81, 0x00, MAGIC, 7]), Ok(&[7_u8][..]));
}

#[test]
fn mesh_rejects_a_missing_second_magic() {
    assert_eq!(
        decode_mesh(&[MAGIC, 0x01, 0x0a, 0x00]),
        Err(FrameError::BadMagic)
    );
}

#[test]
fn mesh_rejects_bad_first_byte() {
    // '<' (an XML datagram) is the caller's other branch, not a mesh frame.
    assert_eq!(decode_mesh(b"<event/>"), Err(FrameError::BadMagic));
}

#[test]
fn mesh_rejects_truncated_headers() {
    for d in [&[][..], &[MAGIC], &[MAGIC, 0x81], &[MAGIC, 0x01]] {
        let err = decode_mesh(d).unwrap_err();
        assert_eq!(err, FrameError::Truncated, "{d:?}");
        assert_eq!(err.kind().as_str(), "frame.truncated");
    }
}

#[test]
fn mesh_rejects_bad_version_varints() {
    assert_eq!(
        decode_mesh(&[MAGIC, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01, MAGIC]),
        Err(FrameError::VarintTooLong)
    );
    assert_eq!(
        decode_mesh(&[MAGIC, 0x80, 0x80, 0x80, 0x80, 0x10, MAGIC]),
        Err(FrameError::VarintOverflow)
    );
}

#[test]
fn errors_do_not_echo_input() {
    let all = [
        FrameError::BadMagic,
        FrameError::VarintTooLong,
        FrameError::VarintOverflow,
        FrameError::TooLarge { len: 9, max: 8 },
        FrameError::Truncated,
        FrameError::UnsupportedVersion,
    ];
    for e in all {
        let text = e.to_string();
        assert!(text.starts_with("TAK protocol") || text.starts_with("unsupported TAK"));
        assert!(!text.contains("0x"), "{text}");
    }
    assert_eq!(MAX_VARINT_BYTES, 5);
}

// ---- properties (plan section 8.2) ----

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn prop_stream_decode_never_panics(buf in prop::collection::vec(any::<u8>(), 0..64), max in 0_usize..128) {
        if let Ok(Some(f)) = decode_stream_frame(&buf, max) {
            prop_assert!(f.consumed <= buf.len());
            prop_assert!(f.payload.end == f.consumed);
            prop_assert!(f.payload.len() <= max);
            prop_assert!(f.payload_of(&buf).is_some());
        }
    }

    #[test]
    fn prop_magic_prefixed_stream_decode_never_panics(tail in prop::collection::vec(any::<u8>(), 0..16)) {
        let mut buf = vec![MAGIC];
        buf.extend_from_slice(&tail);
        let _ = decode_stream_frame(&buf, MAX);
    }

    #[test]
    fn prop_stream_round_trip(msg in prop::collection::vec(any::<u8>(), 0..512), extra in prop::collection::vec(any::<u8>(), 0..8)) {
        let mut buf = Vec::new();
        encode_stream_frame(&msg, MAX, &mut buf).unwrap();
        let frame_len = buf.len();
        buf.extend_from_slice(&extra);
        let f = decode_stream_frame(&buf, MAX).unwrap().unwrap();
        prop_assert_eq!(f.consumed, frame_len);
        prop_assert_eq!(f.payload_of(&buf), Some(&msg[..]));
    }

    #[test]
    fn prop_mesh_decode_never_panics(buf in prop::collection::vec(any::<u8>(), 0..32)) {
        if let Ok(payload) = decode_mesh(&buf) {
            prop_assert!(payload.len() + 3 <= buf.len());
        }
    }

    #[test]
    fn prop_mesh_round_trip(msg in prop::collection::vec(any::<u8>(), 0..512)) {
        let mut out = Vec::new();
        encode_mesh(&msg, &mut out);
        prop_assert_eq!(decode_mesh(&out), Ok(&msg[..]));
    }

    #[test]
    fn prop_decode_tak_message_never_panics(buf in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = decode_tak_message(&buf, MAX);
        if let Ok(msg) = decode_tak_message(&buf, MAX) {
            // Whatever decodes re-encodes stably (bytes, not values: a
            // decoded double may be NaN, which is not equal to itself).
            let again = tak_proto::encode_tak_message(&msg);
            let back = decode_tak_message(&again, MAX).unwrap();
            prop_assert_eq!(tak_proto::encode_tak_message(&back), again);
        }
    }
}
