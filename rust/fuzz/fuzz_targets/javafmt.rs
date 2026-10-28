// SPDX-License-Identifier: Apache-2.0
//! Fuzz target `javafmt`: the Java-compatible number and `CoT` time text
//! forms (plan section 1.8, WU9).
//!
//! Properties:
//! - `parse_double`, `parse_int` and `parse_cot_time` never panic or time
//!   out on any text;
//! - doubles: `parse_double(double_to_string(x))` gives `x` back bit for bit
//!   for every non-NaN `x` (finite, infinite, `-0.0`), and NaN for NaN; this
//!   is checked for the first eight input bytes as an `f64` and for every
//!   value `parse_double` reads from the text;
//! - ints: whatever `parse_int` reads prints and parses back to itself;
//! - times: `parse_cot_time(cot_time_to_string(ms))` is `ms` truncated to
//!   the second (toward the past), for the first eight input bytes as an
//!   `i64` and for every value `parse_cot_time` reads from the text.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SA-11(8) (Dynamic Code Analysis)
#![no_main]

use libfuzzer_sys::fuzz_target;
use tak_cot::javafmt::{
    cot_time_to_string, double_to_string, parse_cot_time, parse_double, parse_int,
};

fuzz_target!(|data: &[u8]| {
    if let Some(head) = data.first_chunk::<8>() {
        double_round_trips(f64::from_le_bytes(*head));
        cot_time_round_trips(i64::from_le_bytes(*head));
    }

    // The corpus seeds are text, one number or time per input.
    let text = String::from_utf8_lossy(data);
    if let Some(x) = parse_double(&text) {
        double_round_trips(x);
    }
    if let Some(v) = parse_int(&text) {
        assert_eq!(
            parse_int(&v.to_string()),
            Some(v),
            "int {v} does not round-trip"
        );
    }
    if let Some(ms) = parse_cot_time(&text) {
        cot_time_round_trips(ms);
    }
});

fn double_round_trips(x: f64) {
    let text = double_to_string(x);
    let Some(back) = parse_double(&text) else {
        panic!("double_to_string output {text:?} does not parse");
    };
    if x.is_nan() {
        assert!(back.is_nan(), "NaN printed as {text:?}");
    } else {
        assert_eq!(back.to_bits(), x.to_bits(), "{text:?} does not round-trip");
    }
}

fn cot_time_round_trips(ms: i64) {
    let text = cot_time_to_string(ms);
    assert!(text.ends_with('Z'), "{text:?} is not a UTC time");
    let second = ms.div_euclid(1_000).checked_mul(1_000);
    assert_eq!(
        parse_cot_time(&text),
        second,
        "{text:?} does not round-trip"
    );
}
