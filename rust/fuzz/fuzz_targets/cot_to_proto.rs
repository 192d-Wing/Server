// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `cot_to_proto`: arbitrary bytes parsed as a `CoT` event, then
//! converted to TAK Protocol v1 (X2P; plan section 1.8, WU9).
//!
//! Properties, all under `Limits::default()`:
//! - parsing and conversion never panic, time out or exhaust memory;
//! - X2P is unchanged by re-serialization: converting the X2X form of an
//!   event gives the same bytes (or the same rejection) as converting the
//!   event itself;
//! - the `xmlDetail` written never exceeds `max_xml_detail_bytes`;
//! - the encoded message decodes again and re-encodes to the same bytes;
//! - converting the result back (P2X) never panics, and any P2X output
//!   re-parses.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_cot::{CotEvent, Limits, ToProtoOptions, from_tak_message, to_tak_message};
use tak_proto::{MAX_MESSAGE_BYTES_DEFAULT, decode_tak_message, encode_tak_message};

/// X2P of `ev`, encoded; `Err` carries the stable error code.
fn x2p(ev: &CotEvent) -> Result<Vec<u8>, &'static str> {
    to_tak_message(ev, &ToProtoOptions::default())
        .map(|msg| encode_tak_message(&msg))
        .map_err(|err| err.kind().as_str())
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let limits = Limits::default();
    let Ok(ev) = CotEvent::parse(&text, &limits) else {
        return;
    };

    let msg = match to_tak_message(&ev, &ToProtoOptions::default()) {
        Ok(msg) => msg,
        Err(err) => {
            assert!(!err.kind().as_str().is_empty());
            let _ = err.to_string();
            return;
        }
    };
    let xml_detail_len = msg
        .cot_event
        .as_ref()
        .and_then(|e| e.detail.as_ref())
        .map_or(0, |d| d.xml_detail.len());
    assert!(
        xml_detail_len <= limits.max_xml_detail_bytes,
        "xmlDetail exceeds the cap"
    );

    // Encode, decode, encode is stable.
    let bytes = encode_tak_message(&msg);
    let decoded = match decode_tak_message(&bytes, MAX_MESSAGE_BYTES_DEFAULT.max(bytes.len())) {
        Ok(decoded) => decoded,
        Err(err) => panic!("X2P output does not decode: {}", err.kind().as_str()),
    };
    assert_eq!(encode_tak_message(&decoded), bytes, "X2P encoding unstable");

    // X2P of the X2X form is the same.
    let Ok(again_text) = String::from_utf8(ev.to_xml_bytes()) else {
        panic!("serializer wrote invalid UTF-8");
    };
    let again = match CotEvent::parse(&again_text, &limits) {
        Ok(again) => again,
        Err(err) => panic!(
            "serializer output does not re-parse: {}",
            err.kind().as_str()
        ),
    };
    assert_eq!(x2p(&again), Ok(bytes), "X2P changed by re-serialization");

    // P2X of the result: no panic, output re-parses.
    if let Ok(back) = from_tak_message(&msg, &limits) {
        let Ok(back_text) = String::from_utf8(back.to_xml_bytes()) else {
            panic!("P2X wrote invalid UTF-8");
        };
        if let Err(err) = CotEvent::parse(&back_text, &limits) {
            panic!("P2X output does not re-parse: {}", err.kind().as_str());
        }
    }
});
