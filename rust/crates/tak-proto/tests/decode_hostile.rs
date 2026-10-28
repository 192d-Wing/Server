// SPDX-License-Identifier: Apache-2.0

//! QA additions for Phase 1a (plan sections 3.1, 3.4, 5.2, 8.1): hostile
//! protobuf input is rejected with a typed error and without a panic,
//! stack overflow or attacker-sized allocation (SI-10, SC-5):
//!
//! - deeply nested (unknown) groups, inside `TakMessage` and inside a
//!   sub-message, up to a million levels (prost recursion limit 100);
//! - unbalanced group markers;
//! - length prefixes that claim more than the buffer holds, up to
//!   `u64::MAX`, and over-long varints;
//! - the default 8 MiB caps (`MAX_MESSAGE_BYTES_DEFAULT`) at the limit and
//!   one byte past it, for `decode_tak_message` and for stream frames;
//! - `Debug` output of decode errors free of payload text (SI-11).

use std::error::Error;
use std::fmt::Write as _;

use tak_proto::framing::{FrameError, MAGIC, decode_mesh, decode_stream_frame, encode_mesh};
use tak_proto::{DecodeErrorKind, MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message};

const MAX: usize = MAX_MESSAGE_BYTES_DEFAULT;

/// Key of unknown field 99 as a start group (wire type 3): `0x9b 0x06`.
const START_GROUP_99: [u8; 2] = [0x9b, 0x06];
/// Key of unknown field 99 as an end group (wire type 4): `0x9c 0x06`.
const END_GROUP_99: [u8; 2] = [0x9c, 0x06];

fn nested_groups(depth: usize) -> Vec<u8> {
    let mut out = START_GROUP_99.repeat(depth);
    out.extend(END_GROUP_99.repeat(depth));
    out
}

fn kind(buf: &[u8]) -> Option<DecodeErrorKind> {
    decode_tak_message(buf, MAX).err().map(|e| e.kind())
}

#[test]
fn shallow_unknown_groups_are_skipped() {
    // protobuf-java and prost both skip a well-formed unknown group.
    for depth in [1, 2, 10, 50] {
        let msg = decode_tak_message(&nested_groups(depth), MAX)
            .unwrap_or_else(|e| panic!("depth {depth}: {e}"));
        assert_eq!(msg, tak_proto::v1::TakMessage::default(), "depth {depth}");
    }
}

#[test]
fn deeply_nested_groups_are_rejected_without_stack_overflow() {
    for depth in [101, 1_000, 100_000, 1_000_000] {
        assert_eq!(
            kind(&nested_groups(depth)),
            Some(DecodeErrorKind::Malformed),
            "depth {depth}"
        );
    }
    // Only start markers, never closed.
    assert_eq!(
        kind(&START_GROUP_99.repeat(1_000_000)),
        Some(DecodeErrorKind::Malformed)
    );
}

#[test]
fn recursion_limit_boundary_is_pinned() {
    // The deepest unknown-group nesting accepted at the top level. The
    // value follows from prost's recursion limit (100); pinning it makes a
    // change of limit visible.
    let deepest = (1..=200)
        .take_while(|&d| decode_tak_message(&nested_groups(d), MAX).is_ok())
        .last()
        .unwrap_or(0);
    assert_eq!(deepest, 100, "deepest accepted unknown-group nesting");
    assert_eq!(
        kind(&nested_groups(deepest + 1)),
        Some(DecodeErrorKind::Malformed)
    );
}

#[test]
fn deeply_nested_groups_inside_a_sub_message_are_rejected() {
    // TakMessage.cotEvent (2) -> CotEvent.detail (15) -> groups.
    let groups = nested_groups(10_000);
    let mut detail = vec![0x7a];
    push_varint(&mut detail, groups.len() as u64);
    detail.extend_from_slice(&groups);
    let mut msg = vec![0x12];
    push_varint(&mut msg, detail.len() as u64);
    msg.extend_from_slice(&detail);
    assert_eq!(kind(&msg), Some(DecodeErrorKind::Malformed));
}

#[test]
fn unbalanced_group_markers_are_rejected() {
    assert_eq!(kind(&END_GROUP_99), Some(DecodeErrorKind::Malformed));
    // Start of field 99, end of field 98.
    assert_eq!(
        kind(&[0x9b, 0x06, 0x94, 0x06]),
        Some(DecodeErrorKind::Malformed)
    );
    assert_eq!(kind(&START_GROUP_99), Some(DecodeErrorKind::Malformed));
}

fn push_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = u8::try_from(v & 0x7f).unwrap_or(0);
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

#[test]
fn length_prefixes_beyond_the_buffer_are_rejected_without_allocating() {
    // TakMessage.cotEvent (2, length-delimited) claiming far more than
    // follows; prost checks the remaining length before reading.
    for claimed in [16_u64, u64::from(u32::MAX), u64::MAX >> 1, u64::MAX] {
        let mut buf = vec![0x12];
        push_varint(&mut buf, claimed);
        buf.extend_from_slice(b"abc");
        assert_eq!(
            kind(&buf),
            Some(DecodeErrorKind::Malformed),
            "claimed {claimed}"
        );
    }
    // A string field (CotEvent.uid, 5) inside cotEvent claiming 2^62 bytes.
    let mut inner = vec![0x2a];
    push_varint(&mut inner, 1 << 62);
    let mut buf = vec![0x12];
    push_varint(&mut buf, inner.len() as u64);
    buf.extend_from_slice(&inner);
    assert_eq!(kind(&buf), Some(DecodeErrorKind::Malformed));
}

#[test]
fn over_long_varints_are_rejected() {
    // TakMessage.submissionTime (3, varint) with 11 varint bytes.
    let mut buf = vec![0x18];
    buf.extend_from_slice(&[0xff; 10]);
    buf.push(0x01);
    assert_eq!(kind(&buf), Some(DecodeErrorKind::Malformed));
    // A key varint that never terminates.
    assert_eq!(kind(&[0xff; 64]), Some(DecodeErrorKind::Malformed));
}

#[test]
fn decode_default_cap_at_8_mib_and_one_byte_more() {
    // A valid message of exactly MAX bytes: cotEvent.uid holding the rest.
    let header_len = 1 + 4 + 1 + 4; // tag, varint(len) x2 at 4 bytes each
    let uid_len = MAX - header_len;
    let mut inner = vec![0x2a];
    push_varint(&mut inner, uid_len as u64);
    inner.resize(inner.len() + uid_len, b'u');
    let mut buf = vec![0x12];
    push_varint(&mut buf, inner.len() as u64);
    buf.extend_from_slice(&inner);
    assert_eq!(buf.len(), MAX);
    let msg = decode_tak_message(&buf, MAX).unwrap();
    assert_eq!(msg.cot_event.unwrap().uid.len(), uid_len);

    buf.push(0);
    let err = decode_tak_message(&buf, MAX).unwrap_err();
    assert_eq!(err.kind(), DecodeErrorKind::TooLarge);
    assert_eq!(err.kind().as_str(), "proto.too_large");
}

#[test]
fn stream_frame_default_cap_at_8_mib_and_one_byte_more() {
    let mut header = vec![MAGIC];
    push_varint(&mut header, MAX as u64);
    let mut buf = header.clone();
    buf.resize(header.len() + MAX, 0);
    let frame = decode_stream_frame(&buf, MAX).unwrap().unwrap();
    assert_eq!(frame.payload.len(), MAX);
    assert_eq!(frame.consumed, buf.len());

    // MAX + 1 is refused from the header alone, before any payload.
    let mut header = vec![MAGIC];
    push_varint(&mut header, MAX as u64 + 1);
    assert_eq!(
        decode_stream_frame(&header, MAX),
        Err(FrameError::TooLarge {
            len: MAX + 1,
            max: MAX
        })
    );
    // The largest length a 5-byte varint can carry is refused too.
    assert_eq!(
        decode_stream_frame(&[MAGIC, 0xff, 0xff, 0xff, 0xff, 0x0f], MAX)
            .unwrap_err()
            .kind()
            .as_str(),
        "frame.too_large"
    );
}

#[test]
fn oversized_mesh_payload_is_refused_by_the_message_cap() {
    // Mesh datagrams carry no length; the payload is bounded by the
    // TakMessage decode cap (and in Phase 3 by the datagram size).
    let payload = vec![0_u8; MAX + 1];
    let mut datagram = Vec::new();
    encode_mesh(&payload, &mut datagram);
    let got = decode_mesh(&datagram).unwrap();
    assert_eq!(got.len(), MAX + 1);
    assert_eq!(
        decode_tak_message(got, MAX).unwrap_err().kind(),
        DecodeErrorKind::TooLarge
    );
}

#[test]
fn decode_error_debug_has_no_payload_text() {
    // cotEvent { uid: "Qz7Mark", type: "Z\xff" }: fails on the second
    // string, after the first was read.
    let mut cot = vec![0x2a, 7];
    cot.extend_from_slice(b"Qz7Mark");
    cot.extend_from_slice(&[0x0a, 0x02, b'Z', 0xff]);
    let mut buf = vec![0x12];
    push_varint(&mut buf, cot.len() as u64);
    buf.extend_from_slice(&cot);
    let err = decode_tak_message(&buf, MAX).unwrap_err();
    let mut text = format!("{err} {err:?}");
    let mut source = err.source();
    while let Some(s) = source {
        let _ = write!(text, " | {s} {s:?}");
        source = s.source();
    }
    assert!(!text.contains("Qz7Mark"), "{text}");
}
