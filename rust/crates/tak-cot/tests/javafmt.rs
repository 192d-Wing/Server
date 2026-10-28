// SPDX-License-Identifier: Apache-2.0

//! `tak_cot::javafmt`: Java-compatible numbers and `CoT` times (parity-map
//! K-14, K-34, K-35, K-36, C-07, DEV-03; plan sections 2.A WU4, 8.1, 8.2).
//!
//! The tables at the end hold the corpus number inputs
//! (`rust/parity/corpus/cases/numbers/`) with the results a current JDK and
//! Joda-Time 2.14 give for them, observed black-box. The gated comparison
//! with the reference's own JDK 17 and Joda-Time 2.13.1 output is the golden
//! test (WU8); these tables keep the grammar fixed in the meantime.

use proptest::prelude::*;
use tak_cot::javafmt::{
    cot_time_to_string, double_to_string, parse_cot_time, parse_double, parse_int,
};

// ---- double_to_string ----

#[test]
fn double_to_string_specials() {
    assert_eq!(double_to_string(f64::NAN), "NaN");
    assert_eq!(double_to_string(-f64::NAN), "NaN");
    assert_eq!(
        double_to_string(f64::from_bits(0x7ff0_0000_0000_0001)),
        "NaN"
    );
    assert_eq!(double_to_string(f64::INFINITY), "Infinity");
    assert_eq!(double_to_string(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(double_to_string(0.0), "0.0");
    assert_eq!(double_to_string(-0.0), "-0.0");
}

#[test]
fn double_to_string_plain_range() {
    for (x, s) in [
        (1.0, "1.0"),
        (-1.0, "-1.0"),
        (100.0, "100.0"),
        (0.1, "0.1"),
        (0.5, "0.5"),
        (std::f64::consts::PI, "3.141592653589793"),
        (-45.123_456, "-45.123456"),
        (-140.654_321, "-140.654321"),
        (999_999.0, "999999.0"),
        (1_234_567.0, "1234567.0"),
        (9_999_999.0, "9999999.0"),
        (9_999_999.999_999_998, "9999999.999999998"),
        (0.001, "0.001"),
        (0.002, "0.002"),
        (0.012_345, "0.012345"),
        (9_999_999.0 / 10_000_000_000.0, "9.999999E-4"),
    ] {
        assert_eq!(double_to_string(x), s, "{x:e}");
    }
}

#[test]
fn double_to_string_scientific_range() {
    for (x, s) in [
        (1.0e7, "1.0E7"),
        (-1.0e7, "-1.0E7"),
        (1.234_567_8e7, "1.2345678E7"),
        (1.0e21, "1.0E21"),
        (1.0e23, "1.0E23"),
        (123_456_789.0, "1.23456789E8"),
        (1.0e-4, "1.0E-4"),
        (1.234_5e-4, "1.2345E-4"),
        (f64::MAX, "1.7976931348623157E308"),
        (-f64::MAX, "-1.7976931348623157E308"),
        (f64::MIN_POSITIVE, "2.2250738585072014E-308"),
        (
            f64::from_bits(0x000f_ffff_ffff_ffff),
            "2.225073858507201E-308",
        ),
        (2.0e-323, "2.0E-323"),
    ] {
        assert_eq!(double_to_string(x), s, "{x:e}");
    }
}

#[test]
fn double_to_string_boundaries_around_one_thousandth() {
    // The double nearest 0.001 is just above it and prints plain; the one
    // below prints in scientific notation.
    let nearest = 0.001_f64;
    assert_eq!(double_to_string(nearest), "0.001");
    let below = f64::from_bits(nearest.to_bits() - 1);
    assert_eq!(double_to_string(below), "9.999999999999998E-4");
    let above = f64::from_bits(nearest.to_bits() + 1);
    assert_eq!(double_to_string(above), "0.0010000000000000002");
}

#[test]
fn double_to_string_boundaries_around_ten_million() {
    let ten_million = 1.0e7_f64;
    let below = f64::from_bits(ten_million.to_bits() - 1);
    assert_eq!(double_to_string(below), "9999999.999999998");
    let above = f64::from_bits(ten_million.to_bits() + 1);
    assert_eq!(double_to_string(above), "1.0000000000000002E7");
}

#[test]
fn double_to_string_prefers_a_closer_two_digit_form_over_one_digit() {
    // The shortest digit string of Double.MIN_VALUE is `5E-324`, but `4.9E-324`
    // also round-trips and is closer to the exact value; Java prints that.
    assert_eq!(double_to_string(f64::from_bits(1)), "4.9E-324");
    assert_eq!(double_to_string(-f64::from_bits(1)), "-4.9E-324");
    assert_eq!(double_to_string(f64::from_bits(2)), "9.9E-324");
    assert_eq!(double_to_string(f64::from_bits(3)), "1.5E-323");
    // One digit stays one digit when no two-digit decimal is closer.
    assert_eq!(double_to_string(2.0), "2.0");
    assert_eq!(double_to_string(9.0e-4), "9.0E-4");
}

// ---- parse_double ----

fn bits(text: &str) -> Option<u64> {
    parse_double(text).map(f64::to_bits)
}

#[test]
fn parse_double_corpus_strings() {
    for (text, expected) in DOUBLE_STRINGS {
        assert_eq!(bits(text), *expected, "{text:?}");
    }
}

#[test]
fn parse_double_suffixes_do_not_change_the_value() {
    for suffix in ["", "d", "D", "f", "F"] {
        assert_eq!(parse_double(&format!("0.1{suffix}")), Some(0.1), "{suffix}");
        assert_eq!(parse_double(&format!("1e3{suffix}")), Some(1000.0));
        assert_eq!(parse_double(&format!("0x1p3{suffix}")), Some(8.0));
    }
    for bad in [
        "1.5dd",
        "1.5 d",
        "1.5fd",
        "NaNd",
        "Infinityf",
        "1.5e",
        "d",
        "1.5L",
    ] {
        assert_eq!(parse_double(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_double_trims_only_characters_up_to_space() {
    assert_eq!(
        parse_double(" \t\n\r\u{b}\u{c}\u{0}\u{1f}2\u{1} "),
        Some(2.0)
    );
    for bad in ["\u{a0}5", "5\u{a0}", "1\u{2009}", "\u{feff}1", "1 2"] {
        assert_eq!(parse_double(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_double_special_words_are_case_sensitive() {
    assert!(parse_double("NaN").is_some_and(f64::is_nan));
    assert!(parse_double("-NaN").is_some_and(f64::is_nan));
    assert!(parse_double(" +NaN ").is_some_and(f64::is_nan));
    assert_eq!(parse_double("Infinity"), Some(f64::INFINITY));
    assert_eq!(parse_double("-Infinity"), Some(f64::NEG_INFINITY));
    for bad in [
        "nan", "NAN", "inf", "-inf", "Inf", "infinity", "INFINITY", "+-1", "--1",
    ] {
        assert_eq!(parse_double(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_double_hex_literals() {
    for (text, value) in [
        ("0x1p3", 8.0),
        ("0X1P3", 8.0),
        ("0x1.8p1", 3.0),
        ("0x.8p1", 1.0),
        ("0x1.p0", 1.0),
        ("-0x1p-1", -0.5),
        ("0x0p0", 0.0),
        ("0x1.fffffffffffffp1023", f64::MAX),
        ("0x1p-1022", f64::MIN_POSITIVE),
        ("0x1p-1074", f64::from_bits(1)),
        ("0x1p1024", f64::INFINITY),
        ("0x1p-1076", 0.0),
        // Exactly half of the smallest subnormal ties to even (zero); a bit
        // more rounds up.
        ("0x1p-1075", 0.0),
        ("0x1.0000000000001p-1075", f64::from_bits(1)),
        ("0x1.00000000000000000000001p-1075", f64::from_bits(1)),
        // 2^53 + 1 is a tie between 2^53 and 2^53 + 2: even wins.
        ("0x20000000000001p0", 9_007_199_254_740_992.0),
        ("0x20000000000003p0", 9_007_199_254_740_996.0),
        ("0x1p99999999999999999999", f64::INFINITY),
        ("0x0p99999999999999999999", 0.0),
    ] {
        assert_eq!(bits(text), Some(value.to_bits()), "{text}");
    }
    for bad in [
        "0x1", "0x1p", "0xp1", "0x.p1", "0x1.8", "0x1g", "0x1p1.5", "0x1p+", "0x",
    ] {
        assert_eq!(parse_double(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_double_is_correctly_rounded() {
    // Halfway between 1 and the next double, plus or minus a little.
    assert_eq!(
        parse_double("1.00000000000000011102230246251565404236316680908203125"),
        Some(1.0)
    );
    assert_eq!(
        parse_double("1.00000000000000011102230246251565404236316680908203126"),
        Some(f64::from_bits(1.0_f64.to_bits() + 1))
    );
    assert_eq!(parse_double("2.4703282292062327e-324"), Some(0.0));
    assert_eq!(
        parse_double("2.4703282292062328e-324"),
        Some(f64::from_bits(1))
    );
    assert_eq!(parse_double("1.7976931348623158e308"), Some(f64::MAX));
    assert_eq!(parse_double("1.7976931348623159e308"), Some(f64::INFINITY));
    assert_eq!(
        parse_double("9007199254740993"),
        Some(9_007_199_254_740_992.0)
    );
}

#[test]
fn parse_double_long_inputs_and_huge_exponents() {
    let ones = "1".repeat(400);
    assert_eq!(parse_double(&ones), Some(f64::INFINITY));
    assert_eq!(
        parse_double(&format!("{ones}e-399")),
        parse_double("1.1111111111111111")
    );
    let tiny = format!("0.{}5", "0".repeat(323));
    assert_eq!(parse_double(&tiny), Some(f64::from_bits(1)));
    // Many leading fraction zeros cancelled by a large exponent.
    let shifted = format!("0.{}1e100005", "0".repeat(100_000));
    assert_eq!(parse_double(&shifted), Some(1.0e4));
    assert_eq!(parse_double("1e2147483647"), Some(f64::INFINITY));
    assert_eq!(parse_double("1e-2147483648"), Some(0.0));
    assert_eq!(parse_double("1e99999999999999999999"), Some(f64::INFINITY));
    assert_eq!(parse_double("0e99999999999"), Some(0.0));
    assert_eq!(bits("-0e5"), Some((-0.0_f64).to_bits()));
}

// ---- parse_int ----

#[test]
fn parse_int_corpus_strings() {
    for (text, expected) in INT_STRINGS {
        assert_eq!(parse_int(text), *expected, "{text:?}");
    }
}

#[test]
fn parse_int_range_and_sign() {
    assert_eq!(parse_int("2147483647"), Some(i32::MAX));
    assert_eq!(parse_int("+2147483647"), Some(i32::MAX));
    assert_eq!(parse_int("-2147483648"), Some(i32::MIN));
    assert_eq!(parse_int("-0000000000002147483648"), Some(i32::MIN));
    assert_eq!(parse_int("2147483648"), None);
    assert_eq!(parse_int("+2147483648"), None);
    assert_eq!(parse_int("-2147483649"), None);
    assert_eq!(parse_int(&"9".repeat(10_000)), None);
    for bad in [
        "", "+", "-", "+-1", "-+1", " 1", "1 ", "1_000", "0x10", "1.0", "1e3",
    ] {
        assert_eq!(parse_int(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_int_accepts_unicode_decimal_digits_of_the_bmp() {
    // Arabic-Indic, extended Arabic-Indic, NKo, Devanagari, fullwidth,
    // Meetei Mayek, Ol Chiki: each `Nd` run is accepted, mixed freely.
    assert_eq!(parse_int("\u{661}\u{662}\u{663}"), Some(123));
    assert_eq!(parse_int("-\u{6f9}\u{6f0}"), Some(-90));
    assert_eq!(parse_int("\u{7c7}"), Some(7));
    assert_eq!(parse_int("\u{966}\u{96f}"), Some(9));
    assert_eq!(parse_int("+\u{ff11}\u{ff12}"), Some(12));
    assert_eq!(parse_int("\u{abf5}5"), Some(55));
    assert_eq!(parse_int("\u{1c59}"), Some(9));
    assert_eq!(
        parse_int(
            "\u{ff12}\u{ff11}\u{ff14}\u{ff17}\u{ff14}\u{ff18}\u{ff13}\u{ff16}\u{ff14}\u{ff17}"
        ),
        Some(i32::MAX)
    );
}

#[test]
fn parse_int_rejects_other_numeric_characters() {
    for bad in [
        "\u{b2}",    // superscript two (No)
        "\u{2460}",  // circled digit one (No)
        "\u{216b}",  // Roman numeral twelve (Nl)
        "\u{19da}",  // New Tai Lue tham digit one (No)
        "\u{1d7ce}", // mathematical bold digit zero: Nd, but outside the BMP
        "\u{104a0}", // Osmanya digit zero: Nd, outside the BMP
        "\u{65f}",   // just below the Arabic-Indic run
        "\u{66a}",   // just above it
        "\u{ff1a}",  // just above the fullwidth run
    ] {
        assert_eq!(parse_int(bad), None, "{bad:?}");
    }
}

// ---- cot_time_to_string ----

#[test]
fn cot_time_to_string_corpus_values() {
    for (ms, expected) in TIMES_MS {
        assert_eq!(cot_time_to_string(*ms), *expected, "{ms}");
    }
}

#[test]
fn cot_time_to_string_years_and_negative_millis() {
    assert_eq!(cot_time_to_string(0), "1970-01-01T00:00:00Z");
    assert_eq!(cot_time_to_string(-1), "1969-12-31T23:59:59Z");
    assert_eq!(
        cot_time_to_string(-62_167_219_200_000),
        "0000-01-01T00:00:00Z"
    );
    assert_eq!(
        cot_time_to_string(-62_167_219_200_001),
        "-0001-12-31T23:59:59Z"
    );
    assert_eq!(
        cot_time_to_string(253_402_300_800_000),
        "10000-01-01T00:00:00Z"
    );
    assert_eq!(cot_time_to_string(i64::MAX), "292278994-08-17T07:12:55Z");
    assert_eq!(cot_time_to_string(i64::MIN), "-292275055-05-16T16:47:04Z");
}

// ---- parse_cot_time ----

#[test]
fn parse_cot_time_corpus_strings() {
    for (text, expected) in TIME_STRINGS {
        assert_eq!(parse_cot_time(text), *expected, "{text:?}");
    }
}

#[test]
fn parse_cot_time_rejects_where_the_reference_wraps_around() {
    // Java returns these wrapped values; TAK-RS rejects anything whose local
    // time or instant is outside the i64 millisecond range (impl notes WU4,
    // deviation candidate).
    for (text, java) in JAVA_WRAPS {
        assert_eq!(parse_cot_time(text), None, "{text:?} (Java: {java})");
    }
}

#[test]
fn parse_cot_time_grammars() {
    let base = 1_793_102_400_000; // 2026-10-27T12:00:00Z
    // Without a fraction: `Z` only.
    assert_eq!(parse_cot_time("2026-10-27T12:00:00Z"), Some(base));
    assert_eq!(parse_cot_time("2026-10-27t12:00:00z"), Some(base));
    assert_eq!(parse_cot_time("2026-10-27T12:00:00+00:00"), None);
    assert_eq!(parse_cot_time("2026-10-27T12:00:00"), None);
    // With a fraction: 1 to 9 digits, truncated, and a required offset.
    assert_eq!(parse_cot_time("2026-10-27T12:00:00.5Z"), Some(base + 500));
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.999999999Z"),
        Some(base + 999)
    );
    assert_eq!(parse_cot_time("2026-10-27T12:00:00.1234567890Z"), None);
    assert_eq!(parse_cot_time("2026-10-27T12:00:00.Z"), None);
    assert_eq!(parse_cot_time("2026-10-27T12:00:00.000"), None);
    assert_eq!(parse_cot_time("2026-10-27T12:00:00,000Z"), None);
    // Offsets.
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.0+01"),
        Some(base - 3_600_000)
    );
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.0-0130"),
        Some(base + 5_400_000)
    );
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.0+01:30:15,5"),
        Some(base - 5_415_500)
    );
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.0+013015500"),
        Some(base - 5_415_500)
    );
    assert_eq!(
        parse_cot_time("2026-10-27T12:00:00.0+23:59:59.999"),
        Some(base - 86_399_999)
    );
    for bad in [
        "2026-10-27T12:00:00.0+24:00",
        "2026-10-27T12:00:00.0+1",
        "2026-10-27T12:00:00.0+01:3",
        "2026-10-27T12:00:00.0+01:",
        "2026-10-27T12:00:00.0+0130:15",
        "2026-10-27T12:00:00.0+01:30:15.1234",
        "2026-10-27T12:00:00.0+01:60",
        "2026-10-27T12:00:00.0+00:00Z",
        "2026-10-27T12:00:00.0UTC",
    ] {
        assert_eq!(parse_cot_time(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_cot_time_field_widths_and_ranges() {
    assert_eq!(parse_cot_time("2026-1-7T2:3:4Z"), Some(1_767_751_384_000));
    assert_eq!(
        parse_cot_time("+2026-10-27T12:00:00Z"),
        Some(1_793_102_400_000)
    );
    assert_eq!(
        parse_cot_time("000002026-10-27T12:00:00Z"),
        Some(1_793_102_400_000)
    );
    assert_eq!(parse_cot_time("0-1-1T0:0:0Z"), Some(-62_167_219_200_000));
    assert_eq!(parse_cot_time("-0-1-1T0:0:0Z"), Some(-62_167_219_200_000));
    assert_eq!(
        parse_cot_time("2024-02-29T00:00:00Z"),
        Some(1_709_164_800_000)
    );
    assert_eq!(
        parse_cot_time("0-02-29T00:00:00Z"),
        Some(-62_162_121_600_000)
    );
    for bad in [
        "0000000002026-10-27T12:00:00Z",
        "2026-010-27T12:00:00Z",
        "2026-10-27T012:00:00Z",
        "2026-+1-27T12:00:00Z",
        "++2026-10-27T12:00:00Z",
        "2023-02-29T00:00:00Z",
        "1900-02-29T00:00:00Z",
        "-1-02-29T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-00-10T00:00:00Z",
        "2026-10-00T00:00:00Z",
        "2026-10-27T24:00:00Z",
        "2026-10-27T12:60:00Z",
        "2026-10-27T12:00:60Z",
        "\u{662}\u{660}\u{662}\u{666}-10-27T12:00:00Z",
        " 2026-10-27T12:00:00Z",
        "2026-10-27T12:00:00Z ",
    ] {
        assert_eq!(parse_cot_time(bad), None, "{bad:?}");
    }
}

#[test]
fn parse_cot_time_extremes_of_the_i64_range() {
    assert_eq!(
        parse_cot_time("292278994-08-17T07:12:55.807Z"),
        Some(i64::MAX)
    );
    assert_eq!(
        parse_cot_time("-292275055-05-16T16:47:04.192Z"),
        Some(i64::MIN)
    );
    assert_eq!(parse_cot_time("292278994-08-17T07:12:55.807-00:01"), None);
    assert_eq!(parse_cot_time("999999999-01-01T00:00:00Z"), None);
    assert_eq!(parse_cot_time("-999999999-01-01T00:00:00Z"), None);
}

// ---- properties (plan 8.2) ----

proptest! {
    #![proptest_config(ProptestConfig { cases: 512, ..ProptestConfig::default() })]

    #[test]
    fn double_round_trips_bit_exactly(raw in any::<u64>()) {
        let x = f64::from_bits(raw);
        let text = double_to_string(x);
        let back = parse_double(&text).unwrap();
        if x.is_nan() {
            prop_assert!(back.is_nan());
        } else {
            prop_assert_eq!(back.to_bits(), x.to_bits(), "{}", text);
        }
    }

    #[test]
    fn double_round_trips_for_cot_like_values(x in -1.0e7_f64..1.0e7) {
        let back = parse_double(&double_to_string(x)).unwrap();
        prop_assert_eq!(back.to_bits(), x.to_bits());
    }

    #[test]
    fn hex_rendering_parses_back(raw in any::<u64>()) {
        let x = f64::from_bits(raw);
        prop_assume!(x.is_finite());
        let exp = i64::try_from((raw >> 52) & 0x7ff).unwrap();
        let fraction = raw & ((1_u64 << 52) - 1);
        let sign = if x.is_sign_negative() { "-" } else { "" };
        let text = if exp == 0 {
            format!("{sign}0x0.{fraction:013x}p-1022")
        } else {
            format!("{sign}0x1.{fraction:013x}p{}", exp - 1023)
        };
        prop_assert_eq!(parse_double(&text).map(f64::to_bits), Some(raw), "{}", text);
    }

    #[test]
    fn parse_double_never_panics(text in "\\PC{0,64}") {
        let _ = parse_double(&text);
    }

    #[test]
    fn parse_double_never_panics_on_grammar_like_input(
        text in "[ \t+-]?(0[xX])?[0-9a-fA-F.]{0,30}([eEpP][+-]?[0-9]{0,25})?[fFdD]?[ ]?"
    ) {
        let _ = parse_double(&text);
    }

    #[test]
    fn parse_int_round_trips(v in any::<i32>()) {
        prop_assert_eq!(parse_int(&v.to_string()), Some(v));
        prop_assert_eq!(parse_int(&format!("+{v}")), (v >= 0).then_some(v));
    }

    #[test]
    fn parse_int_matches_the_ascii_grammar(text in "[+-]?[0-9]{0,12}[ a]?") {
        prop_assert_eq!(parse_int(&text), text.parse::<i32>().ok());
    }

    #[test]
    fn parse_int_never_panics(text in "\\PC{0,40}") {
        let _ = parse_int(&text);
    }

    #[test]
    fn cot_time_round_trips_to_the_second(ms in any::<i64>()) {
        let text = cot_time_to_string(ms);
        prop_assert!(text.ends_with('Z'));
        let second = ms.div_euclid(1_000).checked_mul(1_000);
        prop_assert_eq!(parse_cot_time(&text), second, "{}", text);
    }

    #[test]
    fn cot_time_with_millis_round_trips_exactly(
        ms in (i64::MIN + 1_000)..=i64::MAX
    ) {
        let text = cot_time_to_string(ms);
        let with_millis = format!(
            "{}.{:03}Z",
            text.trim_end_matches('Z'),
            ms.rem_euclid(1_000)
        );
        prop_assert_eq!(parse_cot_time(&with_millis), Some(ms), "{}", with_millis);
    }

    #[test]
    fn parse_cot_time_never_panics(text in "\\PC{0,48}") {
        let _ = parse_cot_time(&text);
    }

    #[test]
    fn parse_cot_time_never_panics_on_grammar_like_input(
        text in "[+-]?[0-9]{1,10}-[0-9]{1,3}-[0-9]{1,3}[Tt][0-9]{1,3}:[0-9]{1,3}:[0-9]{1,3}(\\.[0-9]{0,10})?([Zz]|[+-][0-9:.,]{0,12})?"
    ) {
        let _ = parse_cot_time(&text);
    }
}

// ---- corpus tables (inputs from rust/parity/corpus/cases/numbers/) ----

const INT_STRINGS: &[(&str, Option<i32>)] = &[
    ("0", Some(0)),
    ("1", Some(1)),
    ("-1", Some(-1)),
    ("+1", Some(1)),
    ("-0", Some(0)),
    ("+0", Some(0)),
    ("2147483647", Some(2_147_483_647)),
    ("2147483648", None),
    ("-2147483648", Some(-2_147_483_648)),
    ("-2147483649", None),
    ("4294967295", None),
    ("99999999999999999999", None),
    (" 1", None),
    ("1 ", None),
    ("\t1", None),
    ("", None),
    ("-", None),
    ("+", None),
    ("1.0", None),
    ("1e3", None),
    ("0x10", None),
    ("010", Some(10)),
    ("00000000000000000000001", Some(1)),
    ("abc", None),
    ("--1", None),
    ("+-1", None),
    ("1-", None),
    ("1_0", None),
    ("\u{661}\u{662}\u{663}", Some(123)),
    ("\u{ff11}\u{ff12}", Some(12)),
    ("\u{7c1}", Some(1)),
    ("\u{966}\u{967}", Some(1)),
    ("\u{ff11}a", None),
    ("87", Some(87)),
    ("100", Some(100)),
    ("255", Some(255)),
    ("65535", Some(65_535)),
    ("-2147483647", Some(-2_147_483_647)),
    ("2147483646", Some(2_147_483_646)),
    ("  ", None),
];

const DOUBLE_STRINGS: &[(&str, Option<u64>)] = &[
    ("0", Some(0x0000_0000_0000_0000)),
    ("1", Some(0x3ff0_0000_0000_0000)),
    ("-1", Some(0xbff0_0000_0000_0000)),
    ("+1", Some(0x3ff0_0000_0000_0000)),
    ("1.0", Some(0x3ff0_0000_0000_0000)),
    ("-1.0", Some(0xbff0_0000_0000_0000)),
    ("0.0", Some(0x0000_0000_0000_0000)),
    ("-0.0", Some(0x8000_0000_0000_0000)),
    ("-0", Some(0x8000_0000_0000_0000)),
    ("+0.0", Some(0x0000_0000_0000_0000)),
    (" 2 ", Some(0x4000_0000_0000_0000)),
    ("\t3\t", Some(0x4008_0000_0000_0000)),
    ("\u{b}4\u{c}", Some(0x4010_0000_0000_0000)),
    ("\u{a0}5", None),
    ("1.5d", Some(0x3ff8_0000_0000_0000)),
    ("1.5D", Some(0x3ff8_0000_0000_0000)),
    ("1.5f", Some(0x3ff8_0000_0000_0000)),
    ("1.5F", Some(0x3ff8_0000_0000_0000)),
    ("1.5dd", None),
    ("1.5 d", None),
    ("1e10", Some(0x4202_a05f_2000_0000)),
    ("1E10", Some(0x4202_a05f_2000_0000)),
    ("1e+10", Some(0x4202_a05f_2000_0000)),
    ("1e-10", Some(0x3ddb_7cdf_d9d7_bdbb)),
    ("1e", None),
    ("e1", None),
    ("1ee1", None),
    ("1e1.5", None),
    (".5", Some(0x3fe0_0000_0000_0000)),
    ("5.", Some(0x4014_0000_0000_0000)),
    (".", None),
    ("", None),
    (" ", None),
    ("NaN", Some(0x7ff8_0000_0000_0000)),
    ("-NaN", Some(0x7ff8_0000_0000_0000)),
    ("+NaN", Some(0x7ff8_0000_0000_0000)),
    ("nan", None),
    ("NAN", None),
    ("NaNd", None),
    ("Infinity", Some(0x7ff0_0000_0000_0000)),
    ("-Infinity", Some(0xfff0_0000_0000_0000)),
    ("+Infinity", Some(0x7ff0_0000_0000_0000)),
    ("Infinityd", None),
    ("inf", None),
    ("-inf", None),
    ("INFINITY", None),
    ("infinity", None),
    ("Inf", None),
    ("0x1p3", Some(0x4020_0000_0000_0000)),
    ("0X1P3", Some(0x4020_0000_0000_0000)),
    ("0x1.8p1", Some(0x4008_0000_0000_0000)),
    ("0x.8p1", Some(0x3ff0_0000_0000_0000)),
    ("0x1p", None),
    ("0x1", None),
    ("0x1p3d", Some(0x4020_0000_0000_0000)),
    ("-0x1p-1074", Some(0x8000_0000_0000_0001)),
    ("0x1p-1075", Some(0x0000_0000_0000_0000)),
    ("0x1.0000000000001p-1075", Some(0x0000_0000_0000_0001)),
    ("0x1.fffffffffffffp1023", Some(0x7fef_ffff_ffff_ffff)),
    ("0x1p1024", Some(0x7ff0_0000_0000_0000)),
    ("1e400", Some(0x7ff0_0000_0000_0000)),
    ("-1e400", Some(0xfff0_0000_0000_0000)),
    ("1e-400", Some(0x0000_0000_0000_0000)),
    ("4.9e-324", Some(0x0000_0000_0000_0001)),
    ("2.4703282292062327e-324", Some(0x0000_0000_0000_0000)),
    ("2.4703282292062328e-324", Some(0x0000_0000_0000_0001)),
    ("2.2250738585072014E-308", Some(0x0010_0000_0000_0000)),
    ("2.2250738585072011e-308", Some(0x000f_ffff_ffff_ffff)),
    ("1.7976931348623157e308", Some(0x7fef_ffff_ffff_ffff)),
    ("1.7976931348623158e308", Some(0x7fef_ffff_ffff_ffff)),
    ("1.7976931348623159e308", Some(0x7ff0_0000_0000_0000)),
    ("9007199254740993", Some(0x4340_0000_0000_0000)),
    ("9007199254740995", Some(0x4340_0000_0000_0002)),
    ("0.1", Some(0x3fb9_9999_9999_999a)),
    ("0.30000000000000004", Some(0x3fd3_3333_3333_3334)),
    ("3.141592653589793", Some(0x4009_21fb_5444_2d18)),
    ("-45.123456", Some(0xc046_8fcd_67fd_3f5b)),
    ("-140.654321", Some(0xc061_94f0_3298_02c1)),
    ("9999999.0", Some(0x4163_12cf_e000_0000)),
    ("999999", Some(0x412e_847e_0000_0000)),
    ("1e2147483647", Some(0x7ff0_0000_0000_0000)),
    ("1e-2147483648", Some(0x0000_0000_0000_0000)),
    ("1e99999999999999999999", Some(0x7ff0_0000_0000_0000)),
    ("0e99999999999", Some(0x0000_0000_0000_0000)),
    ("00012", Some(0x4028_0000_0000_0000)),
    ("1_000", None),
    ("1,5", None),
    ("1.5.5", None),
    ("abc", None),
    ("--1", None),
    ("+-1", None),
    ("1 2", None),
    ("\u{661}\u{662}", None),
    ("\u{ff11}", None),
    ("1\u{2009}", None),
    ("-.5e-1", Some(0xbfa9_9999_9999_999a)),
    ("+.5E+1", Some(0x4014_0000_0000_0000)),
    ("1.0E-3", Some(0x3f50_624d_d2f1_a9fc)),
];

const TIMES_MS: &[(i64, &str)] = &[
    (0, "1970-01-01T00:00:00Z"),
    (1, "1970-01-01T00:00:00Z"),
    (999, "1970-01-01T00:00:00Z"),
    (1_000, "1970-01-01T00:00:01Z"),
    (1_001, "1970-01-01T00:00:01Z"),
    (-1, "1969-12-31T23:59:59Z"),
    (-999, "1969-12-31T23:59:59Z"),
    (-1_000, "1969-12-31T23:59:59Z"),
    (-1_001, "1969-12-31T23:59:58Z"),
    (59_999, "1970-01-01T00:00:59Z"),
    (60_000, "1970-01-01T00:01:00Z"),
    (86_399_999, "1970-01-01T23:59:59Z"),
    (86_400_000, "1970-01-02T00:00:00Z"),
    (1_761_588_298_123, "2025-10-27T18:04:58Z"),
    (1_761_588_298_999, "2025-10-27T18:04:58Z"),
    (1_793_102_400_000, "2026-10-27T12:00:00Z"),
    (1_793_102_400_123, "2026-10-27T12:00:00Z"),
    (i64::MAX, "292278994-08-17T07:12:55Z"),
    (i64::MIN, "-292275055-05-16T16:47:04Z"),
    (-9_223_372_036_854_775_807, "-292275055-05-16T16:47:04Z"),
    (9_223_372_036_854_774_808, "292278994-08-17T07:12:54Z"),
    (253_402_300_799_999, "9999-12-31T23:59:59Z"),
    (253_402_300_800_000, "10000-01-01T00:00:00Z"),
    (-62_135_596_800_000, "0001-01-01T00:00:00Z"),
    (-62_135_596_800_001, "0000-12-31T23:59:59Z"),
    (-62_167_219_200_000, "0000-01-01T00:00:00Z"),
    (-62_167_305_600_000, "-0001-12-31T00:00:00Z"),
    (-30_610_224_001_000, "0999-12-31T23:59:59Z"),
    (-30_610_224_000_000, "1000-01-01T00:00:00Z"),
    (1_709_208_000_000, "2024-02-29T12:00:00Z"),
    (951_782_400_000, "2000-02-29T00:00:00Z"),
    (-2_203_891_200_000, "1900-03-01T00:00:00Z"),
    (2_147_483_647_000, "2038-01-19T03:14:07Z"),
    (2_147_483_648_000, "2038-01-19T03:14:08Z"),
    (4_294_967_295_000, "2106-02-07T06:28:15Z"),
    (4_294_967_296_000, "2106-02-07T06:28:16Z"),
    (-12_220_243_200_000, "1582-10-04T00:00:00Z"),
    (-12_219_292_800_000, "1582-10-15T00:00:00Z"),
    (1_000_000_000_000_000, "33658-09-27T01:46:40Z"),
    (-1_000_000_000_000_000, "-29719-04-05T22:13:20Z"),
    (10_000_000_000_000_000, "318857-05-20T17:46:40Z"),
    (-10_000_000_000_000_000, "-314918-08-13T06:13:20Z"),
    (100_000_000_000_000_000, "3170843-11-07T09:46:40Z"),
    (1_793_102_700_000, "2026-10-27T12:05:00Z"),
    (1_798_761_599_999, "2026-12-31T23:59:59Z"),
    (1_798_761_600_000, "2027-01-01T00:00:00Z"),
    (2_147_483_648, "1970-01-25T20:31:23Z"),
    (2_147_483_647, "1970-01-25T20:31:23Z"),
    (4_294_967_296, "1970-02-19T17:02:47Z"),
    (-2_147_483_648, "1969-12-07T03:28:36Z"),
    (9_007_199_254_740_992, "287396-10-12T08:59:00Z"),
    (-9_007_199_254_740_992, "-283457-03-21T15:00:59Z"),
    (9_223_372_036_854_775_000, "292278994-08-17T07:12:55Z"),
    (9_223_371_985_593_600_000, "292278993-01-01T00:00:00Z"),
    (9_223_372_017_129_600_000, "292278994-01-01T00:00:00Z"),
    (-9_223_372_017_043_200_000, "-292275054-01-01T00:00:00Z"),
    (-9_223_372_017_129_600_000, "-292275055-12-31T00:00:00Z"),
    (4_611_686_018_427_387_904, "146140482-04-24T15:36:27Z"),
    (-4_611_686_018_427_387_904, "-146136543-09-08T08:23:32Z"),
    (1_772_323_207_257, "2026-03-01T00:00:07Z"),
];

const TIME_STRINGS: &[(&str, Option<i64>)] = &[
    ("2026-10-27T12:00:00Z", Some(1_793_102_400_000)),
    ("2026-10-27T12:00:00.1Z", Some(1_793_102_400_100)),
    ("2026-10-27T12:00:00.12Z", Some(1_793_102_400_120)),
    ("2026-10-27T12:00:00.123Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.1234Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.12345Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123456Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.1234567Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.12345678Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123456789Z", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123+00:00", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123-05:30", Some(1_793_122_200_123)),
    ("2026-10-27T12:00:00.123+14:00", Some(1_793_052_000_123)),
    ("2026-10-27T12:00:00.123+0530", Some(1_793_082_600_123)),
    ("2026-10-27T12:00:00.123+05", Some(1_793_084_400_123)),
    ("2026-10-27T12:00:00.123", None),
    ("2026-10-27T12:00:00", None),
    ("2026-10-27T12:00:00+00:00", None),
    ("2026-10-27t12:00:00Z", Some(1_793_102_400_000)),
    ("2026-10-27T12:00:00z", Some(1_793_102_400_000)),
    ("2026-10-27t12:00:00.500z", Some(1_793_102_400_500)),
    ("2026-1-7T2:3:4Z", Some(1_767_751_384_000)),
    ("2026-1-7T2:3:4.5Z", Some(1_767_751_384_500)),
    ("2026-13-01T00:00:00Z", None),
    ("2026-02-30T00:00:00.000Z", None),
    ("2026-10-27T24:00:00Z", None),
    ("2026-10-27T23:59:60.000Z", None),
    ("0999-01-01T00:00:00Z", Some(-30_641_760_000_000)),
    ("999-01-01T00:00:00.000Z", Some(-30_641_760_000_000)),
    ("0000-01-01T00:00:00.000Z", Some(-62_167_219_200_000)),
    ("10000-01-01T00:00:00Z", Some(253_402_300_800_000)),
    ("10000-01-01T00:00:00.000Z", Some(253_402_300_800_000)),
    ("-0001-01-01T00:00:00.000Z", Some(-62_198_755_200_000)),
    ("yesterday", None),
    ("", None),
    ("1793102400000", None),
    ("2026-10-27 12:00:00Z", None),
    ("2026-10-27T12:00Z", None),
    ("2026-10-27T12:00:00.Z", None),
    ("2026-10-27T12:00:00.123456789012Z", None),
    ("2026-10-27T12.5Z", None),
    (" 2026-10-27T12:00:00Z", None),
    ("1969-12-31T23:59:59.999Z", Some(-1)),
    ("1970-01-01T00:00:00Z", Some(0)),
    ("1970-01-01T00:00:00.000Z", Some(0)),
    ("1969-12-31T23:59:59Z", Some(-1_000)),
    ("2038-01-19T03:14:08Z", Some(2_147_483_648_000)),
    ("2024-02-29T00:00:00Z", Some(1_709_164_800_000)),
    ("2026-10-27T12:00:00.123-00:00", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123+00", Some(1_793_102_400_123)),
    ("2026-10-27T12:00:00.123+24:00", None),
    ("2026-10-27T12:00:00.123+23:59", Some(1_793_016_060_123)),
    ("2026-10-27T12:00:00.123+05:30:15", Some(1_793_082_585_123)),
    ("2026-10-27T12:00:00.000GMT", None),
    ("2026-10-27T12:00:00.000UTC", None),
    ("2026-10-27T12:00:00,123Z", None),
    ("2026-W44-2T12:00:00.000Z", None),
    ("2026-300T12:00:00.000Z", None),
    ("2026-10-27T12:30.5Z", None),
    ("2026-10-27T12:00:00.999999999Z", Some(1_793_102_400_999)),
    ("2026-10-27T12:00:00.0005Z", Some(1_793_102_400_000)),
    ("2026-10-27T12:00:00Z ", None),
    ("2026-10-27T12:00:00ZZ", None),
    ("2026-10-27T12:00:00+0000", None),
    ("\u{662}\u{660}\u{662}\u{666}-10-27T12:00:00Z", None),
    ("\u{662}\u{660}\u{662}\u{666}-10-27T12:00:00.000Z", None),
    ("292278993-12-31T23:59:59Z", Some(9_223_372_017_129_599_000)),
    ("292278994-01-01T00:00:00Z", Some(9_223_372_017_129_600_000)),
    ("+2026-10-27T12:00:00Z", Some(1_793_102_400_000)),
    ("+2026-10-27T12:00:00.000Z", Some(1_793_102_400_000)),
    ("2026-10-27T12:00:00.000+5:30", None),
    ("2026-10-27T12:00:00.000 Z", None),
    ("2026-10-27", None),
    ("2026-10-27.000", None),
    ("T12:00:00.000Z", None),
    ("2026-10-27T12:00:00.000Zgarbage", None),
    ("99999-12-31T23:59:59.999Z", Some(3_093_527_980_799_999)),
    ("2026-00-10T00:00:00Z", None),
    ("2026-10-00T00:00:00Z", None),
    ("2026-10-27T12:60:00Z", None),
    ("2026-10-27T-1:00:00Z", None),
    ("2026-10-27T12:00:00.-1Z", None),
    ("2026-10-27T12:00:00.1234567890123456789Z", None),
    ("2026-10-27T12:00:00.000+14:01", Some(1_793_051_940_000)),
    ("2026-10-27T12:00:00.000-18:00", Some(1_793_167_200_000)),
    ("2026-10-27T12:00:00.000-18:01", Some(1_793_167_260_000)),
    ("292278993-01-01T00:00:00Z", Some(9_223_371_985_593_600_000)),
    ("292278994-08-17T07:12:55Z", Some(9_223_372_036_854_775_000)),
    ("292278995-01-01T00:00:00Z", None),
    ("292278999-01-01T00:00:00Z", None),
    ("300000000-01-01T00:00:00Z", None),
    ("999999999-01-01T00:00:00Z", None),
    (
        "-292275054-01-01T00:00:00Z",
        Some(-9_223_372_017_043_200_000),
    ),
    (
        "-292275055-05-16T16:47:05Z",
        Some(-9_223_372_036_854_775_000),
    ),
    ("-292275056-01-01T00:00:00Z", None),
    ("-999999999-01-01T00:00:00Z", None),
    ("-1000000000-01-01T00:00:00Z", None),
    ("1000000000-01-01T00:00:00Z", None),
    ("+999999999-01-01T00:00:00Z", None),
    ("++2026-10-27T12:00:00Z", None),
    ("-+2026-10-27T12:00:00Z", None),
    ("+-2026-10-27T12:00:00Z", None),
    ("-2026-10-27T12:00:00Z", Some(-126_075_700_800_000)),
    ("-0-10-27T12:00:00Z", Some(-62_141_256_000_000)),
    ("0-1-1T0:0:0Z", Some(-62_167_219_200_000)),
    ("2026-010-27T12:00:00Z", None),
    ("2026-10-027T12:00:00Z", None),
    ("2026-10-27T012:00:00Z", None),
    ("2026-10-27T12:000:00Z", None),
    ("2026-10-27T12:00:000Z", None),
    ("2026-+1-27T12:00:00Z", None),
    (
        "2026-10-27T12:00:00.000+05:30:15.123",
        Some(1_793_082_584_877),
    ),
    (
        "2026-10-27T12:00:00.000+05:30:15,1",
        Some(1_793_082_584_900),
    ),
    ("2026-10-27T12:00:00.000+05:30:15.1234", None),
    ("2026-10-27T12:00:00.000+05:30:15.", None),
    ("2026-10-27T12:00:00.000+053015", Some(1_793_082_585_000)),
    ("2026-10-27T12:00:00.000+053015123", Some(1_793_082_584_877)),
    ("2026-10-27T12:00:00.000+0530151", Some(1_793_082_584_900)),
    ("2026-10-27T12:00:00.000+05301", None),
    ("2026-10-27T12:00:00.000+053", None),
    ("2026-10-27T12:00:00.000+05:3", None),
    ("2026-10-27T12:00:00.000+05:", None),
    ("2026-10-27T12:00:00.000+05:30:", None),
    ("2026-10-27T12:00:00.000+05:3015", None),
    ("2026-10-27T12:00:00.000+05:60", None),
    ("2026-10-27T12:00:00.000+05:30:60", None),
    ("2026-10-27T12:00:00.000+", None),
    ("2026-10-27T12:00:00.000-", None),
    ("2026-10-27T12:00:00.000+5", None),
    (
        "2026-10-27T12:00:00.000+23:59:59.999",
        Some(1_793_016_000_001),
    ),
    (
        "2026-10-27T12:00:00.000-23:59:59.999",
        Some(1_793_188_799_999),
    ),
    ("2026-10-27T12:00:00.000+0530:15", None),
    ("2026-10-27T12:00:00.000+05.30", None),
    ("2026-10-27T12:00:00.000+05:30.5", None),
    ("2026-10-27T12:00:00.000+05:30:15:1", None),
    ("2026-10-27T12:00:00.1234567890Z", None),
    ("2026-10-27T12:00:00.000ZZ", None),
    ("2026-10-27T12:00:00.000", None),
    ("2026-10-27T12:00:00.000z", Some(1_793_102_400_000)),
    ("-292275055-05-16T16:47:04.192Z", Some(i64::MIN)),
    ("292278994-08-17T07:12:55.807Z", Some(i64::MAX)),
    (
        "292278994-08-17T07:12:55.807+00:01",
        Some(9_223_372_036_854_715_807),
    ),
    (
        "-292275055-05-16T16:47:04.192-00:01",
        Some(-9_223_372_036_854_715_808),
    ),
    ("2023-02-29T00:00:00Z", None),
    ("2024-02-29T00:00:00.0Z", Some(1_709_164_800_000)),
    ("1900-02-29T00:00:00Z", None),
    ("2000-02-29T00:00:00Z", Some(951_782_400_000)),
    ("0-02-29T00:00:00Z", Some(-62_162_121_600_000)),
    ("-4-02-29T00:00:00Z", Some(-62_288_352_000_000)),
    ("-1-02-29T00:00:00Z", None),
    ("2026-04-31T00:00:00Z", None),
    ("2026-12-31T23:59:59Z", Some(1_798_761_599_000)),
    ("2026-12-32T00:00:00Z", None),
    ("1.5", None),
    (".", None),
    ("2026-10-27T12:00:00.5Z.", None),
    ("2026.10-27T12:00:00Z", None),
    ("2026-10-27T12:00:00.000+00:00Z", None),
    ("0000000002026-10-27T12:00:00Z", None),
    ("000002026-10-27T12:00:00Z", Some(1_793_102_400_000)),
    ("2026-10-27T1:2:3.4+00", Some(1_793_062_923_400)),
];

const JAVA_WRAPS: &[(&str, i64)] = &[
    ("292278994-08-17T07:12:56Z", -9_223_372_036_854_775_000),
    ("292278994-12-31T23:59:59Z", -9_223_372_017_043_201_000),
    ("-292275055-01-01T00:00:00Z", 9_223_372_017_129_600_000),
    ("-292275055-05-16T16:47:04Z", 9_223_372_028_854_024_000),
    ("-292275055-05-16T16:47:04.191Z", 9_223_372_028_854_024_191),
    ("292278994-08-17T07:12:55.808Z", -9_223_372_036_802_824_192),
    (
        "292278994-08-17T08:12:55.807+01:00",
        -9_223_372_036_802_824_193,
    ),
    (
        "-292275055-05-16T15:47:04.192-01:00",
        9_223_372_028_854_024_192,
    ),
];
