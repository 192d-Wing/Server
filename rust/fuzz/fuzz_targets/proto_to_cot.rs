// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `proto_to_cot`: arbitrary bytes decoded as a `TakMessage`,
//! then converted to `CoT` XML (P2X; plan section 1.8, WU9). The nested
//! `xmlDetail` string is untrusted XML inside an untrusted message.
//!
//! Properties, all under `Limits::default()`:
//! - decoding and conversion never panic, time out or exhaust memory;
//! - P2X re-parse: every P2X output is valid UTF-8 that parses again;
//! - X2X idempotence on that output;
//! - X2P of the P2X event never panics, and of the re-parsed event either
//!   rejects or produces a message that encodes and decodes.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_cot::{CotEvent, Limits, ToProtoOptions, from_tak_message, to_tak_message};
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message};

fuzz_target!(|data: &[u8]| {
    let Ok(msg) = decode_tak_message(data, MAX_MESSAGE_BYTES_DEFAULT) else {
        return;
    };
    let limits = Limits::default();
    let ev = match from_tak_message(&msg, &limits) {
        Ok(ev) => ev,
        Err(err) => {
            assert!(!err.kind().as_str().is_empty());
            let _ = err.to_string();
            return;
        }
    };

    // P2X output re-parses and is an X2X fixed point.
    let once = ev.to_xml_bytes();
    let Ok(once_text) = std::str::from_utf8(&once) else {
        panic!("P2X wrote invalid UTF-8");
    };
    let reparsed = match CotEvent::parse(once_text, &limits) {
        Ok(reparsed) => reparsed,
        Err(err) => panic!("P2X output does not re-parse: {}", err.kind().as_str()),
    };
    assert_eq!(
        reparsed.to_xml_bytes(),
        once,
        "X2X not idempotent on P2X output"
    );

    // X2P from the P2X event (numeric cache) and from the re-parsed one.
    let _ = to_tak_message(&ev, &ToProtoOptions::default());
    if let Ok(x2p) = to_tak_message(&reparsed, &ToProtoOptions::default()) {
        let bytes = encode_tak_message(&x2p);
        if let Err(err) = decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT.max(bytes.len())) {
            panic!("X2P output does not decode: {}", err.kind().as_str());
        }
    }
});
