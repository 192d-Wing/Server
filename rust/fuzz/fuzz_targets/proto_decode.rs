// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `proto_decode`: arbitrary bytes decoded as a TAK Protocol v1
//! `TakMessage` (plan section 1.8, WU9).
//!
//! Properties:
//! - decoding never panics, times out or exhausts memory (prost recursion
//!   limit, `MAX_MESSAGE_BYTES_DEFAULT`);
//! - the length limit is enforced before decoding: one byte under the input
//!   length always rejects with `proto.too_large`;
//! - whatever decodes re-encodes, and the re-encoding decodes and encodes to
//!   the same bytes (compared as bytes: a decoded double may be NaN).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_proto::{
    DecodeErrorKind, MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message,
};

fuzz_target!(|data: &[u8]| {
    if let Some(short) = data.len().checked_sub(1) {
        match decode_tak_message(data, short) {
            Err(err) => assert_eq!(err.kind(), DecodeErrorKind::TooLarge),
            Ok(_) => panic!("decode accepted an input over its limit"),
        }
    }

    let msg = match decode_tak_message(data, MAX_MESSAGE_BYTES_DEFAULT) {
        Ok(msg) => msg,
        Err(err) => {
            assert!(!err.kind().as_str().is_empty());
            let _ = err.to_string();
            return;
        }
    };
    let once = encode_tak_message(&msg);
    let back = match decode_tak_message(&once, MAX_MESSAGE_BYTES_DEFAULT.max(once.len())) {
        Ok(back) => back,
        Err(err) => panic!(
            "re-encoded message does not decode: {}",
            err.kind().as_str()
        ),
    };
    assert_eq!(encode_tak_message(&back), once, "encode/decode not stable");
});
