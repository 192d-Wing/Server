// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `stream_frame`: arbitrary bytes as a TAK Protocol stream
//! (`0xbf ‖ varint(len) ‖ payload`, repeated) and as a mesh datagram
//! (`0xbf ‖ varint(1) ‖ 0xbf ‖ payload`; plan section 1.8, WU9).
//!
//! Properties:
//! - decoding never panics; a decoded frame lies inside the buffer, its
//!   payload ends where the frame ends and is within `max_frame`;
//! - every proper prefix of a complete frame reads as "need more bytes";
//! - decode/encode consistency: re-encoding a decoded payload gives a frame
//!   that decodes to the same payload and is consumed whole, and encoding
//!   the whole input then decoding it gives the input back;
//! - a lower `max_frame` never accepts a frame the default rejects, and
//!   accepts exactly the frames whose payload fits it;
//! - mesh: a decoded payload is a suffix of the datagram and survives
//!   encode/decode; encoding the input as a mesh payload decodes back to it.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_proto::MAX_MESSAGE_BYTES_DEFAULT;
use tak_proto::framing::{
    Frame, FrameErrorKind, decode_mesh, decode_stream_frame, encode_mesh, encode_stream_frame,
};

/// A small limit, so the `TooLarge` path is reachable from short inputs.
const SMALL_MAX: usize = 64;

fuzz_target!(|data: &[u8]| {
    stream(data);
    whole_input_round_trips(data);
    mesh(data);
});

/// Walks `data` as a stream of frames under the default limit.
fn stream(data: &[u8]) {
    let max = MAX_MESSAGE_BYTES_DEFAULT;
    let mut buf = data;
    loop {
        let frame = match decode_stream_frame(buf, max) {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                small_limit_agrees(buf, None);
                return;
            }
            Err(err) => {
                assert!(!err.kind().as_str().is_empty());
                // An error under the default limit is an error under any
                // lower one too.
                assert!(decode_stream_frame(buf, SMALL_MAX).is_err());
                return;
            }
        };
        let Frame { payload, consumed } = frame.clone();
        assert!(consumed <= buf.len(), "frame runs past the buffer");
        assert!(consumed > payload.len(), "frame has no header");
        assert_eq!(payload.end, consumed);
        assert!(payload.len() <= max);
        let Some(bytes) = frame.payload_of(buf) else {
            panic!("payload range outside the buffer");
        };

        // Proper prefixes need more bytes.
        for cut in 0..consumed {
            let prefix = buf.get(..cut).unwrap_or_default();
            assert_eq!(
                decode_stream_frame(prefix, max),
                Ok(None),
                "prefix of {cut} bytes"
            );
        }

        // Re-encoding the payload gives an equivalent frame.
        let mut out = Vec::new();
        if let Err(err) = encode_stream_frame(bytes, max, &mut out) {
            panic!(
                "decoded payload does not re-encode: {}",
                err.kind().as_str()
            );
        }
        assert!(
            out.len() <= consumed,
            "re-encoded header longer than the original"
        );
        match decode_stream_frame(&out, max) {
            Ok(Some(again)) => {
                assert_eq!(again.consumed, out.len());
                assert_eq!(again.payload_of(&out), Some(bytes));
            }
            other => panic!("re-encoded frame does not decode: {other:?}"),
        }

        small_limit_agrees(buf, Some(&frame));

        let Some(rest) = buf.get(consumed..) else {
            panic!("consumed past the buffer");
        };
        buf = rest;
    }
}

/// Under `SMALL_MAX`, `buf` decodes to the same frame when its payload
/// fits, and to `TooLarge` (or "need more" before the length is known)
/// otherwise.
fn small_limit_agrees(buf: &[u8], full: Option<&Frame>) {
    let small = decode_stream_frame(buf, SMALL_MAX);
    match full {
        Some(frame) if frame.payload.len() <= SMALL_MAX => {
            assert_eq!(small, Ok(Some(frame.clone())));
        }
        Some(_) => match small {
            Err(err) => assert_eq!(err.kind(), FrameErrorKind::TooLarge),
            Ok(other) => panic!("small limit accepted an oversized frame: {other:?}"),
        },
        None => match small {
            Ok(None) => {}
            Err(err) => assert_eq!(err.kind(), FrameErrorKind::TooLarge),
            Ok(Some(_)) => panic!("small limit found a frame the default did not"),
        },
    }
}

/// Encoding the whole input as one frame decodes back to it.
fn whole_input_round_trips(data: &[u8]) {
    let mut out = Vec::new();
    if let Err(err) = encode_stream_frame(data, MAX_MESSAGE_BYTES_DEFAULT, &mut out) {
        panic!(
            "input under the limit does not encode: {}",
            err.kind().as_str()
        );
    }
    match decode_stream_frame(&out, MAX_MESSAGE_BYTES_DEFAULT) {
        Ok(Some(frame)) => {
            assert_eq!(frame.consumed, out.len());
            assert_eq!(frame.payload_of(&out), Some(data));
        }
        other => panic!("encoded frame does not decode: {other:?}"),
    }

    // Over the limit: rejected, output untouched.
    if let Some(short) = data.len().checked_sub(1) {
        let mut rejected = Vec::new();
        match encode_stream_frame(data, short, &mut rejected) {
            Err(err) => assert_eq!(err.kind(), FrameErrorKind::TooLarge),
            Ok(()) => panic!("encode accepted a payload over its limit"),
        }
        assert!(rejected.is_empty(), "rejected encode wrote output");
    }
}

/// Mesh datagram decode/encode consistency.
fn mesh(data: &[u8]) {
    match decode_mesh(data) {
        Ok(payload) => {
            assert!(data.ends_with(payload), "mesh payload is not a suffix");
            assert!(
                payload.len().saturating_add(3) <= data.len(),
                "mesh header too short"
            );
            let mut out = Vec::new();
            encode_mesh(payload, &mut out);
            assert_eq!(decode_mesh(&out), Ok(payload));
        }
        Err(err) => assert!(!err.kind().as_str().is_empty()),
    }
    let mut out = Vec::new();
    encode_mesh(data, &mut out);
    assert_eq!(decode_mesh(&out), Ok(data));
}
