// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `proto_roundtrip`: arbitrary bytes decoded as a `TakMessage`
//! and taken through the server's proto-in, proto-out path (P2P = P2X then
//! X2P; plan section 1.8, WU9).
//!
//! Properties, all under `Limits::default()`:
//! - no panic, timeout or memory exhaustion anywhere on the path;
//! - P2P is a fixed point after one pass: a second P2P pass of the first
//!   pass's output gives the same encoded bytes (compared as bytes, since a
//!   decoded double may be NaN);
//! - the encoded P2P output decodes and re-encodes to the same bytes.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_cot::{Limits, ToProtoOptions, from_tak_message, to_tak_message};
use tak_proto::v1::TakMessage;
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message};

/// One P2P pass; `None` when either conversion rejects.
fn p2p(msg: &TakMessage, limits: &Limits) -> Option<TakMessage> {
    let ev = from_tak_message(msg, limits).ok()?;
    to_tak_message(&ev, &ToProtoOptions::default()).ok()
}

fuzz_target!(|data: &[u8]| {
    let Ok(msg) = decode_tak_message(data, MAX_MESSAGE_BYTES_DEFAULT) else {
        return;
    };
    let limits = Limits::default();
    let Some(once) = p2p(&msg, &limits) else {
        return;
    };
    let once_bytes = encode_tak_message(&once);

    let decoded =
        match decode_tak_message(&once_bytes, MAX_MESSAGE_BYTES_DEFAULT.max(once_bytes.len())) {
            Ok(decoded) => decoded,
            Err(err) => panic!("P2P output does not decode: {}", err.kind().as_str()),
        };
    assert_eq!(
        encode_tak_message(&decoded),
        once_bytes,
        "P2P encoding unstable"
    );

    let twice = p2p(&once, &limits).map(|m| encode_tak_message(&m));
    assert_eq!(
        twice,
        Some(once_bytes),
        "P2P is not a fixed point after one pass"
    );
});
