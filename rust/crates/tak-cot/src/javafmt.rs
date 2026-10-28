// SPDX-License-Identifier: Apache-2.0

//! Number and `CoT` time text in the forms the reference server reads and
//! writes: `Double.toString`-style shortest digits, the `Double.valueOf` and
//! `Integer.parseInt` grammars, and `yyyy-MM-dd'T'HH:mm:ss'Z'` `CoT` times
//! (WU4, plan section 2.A).
//!
//! Every function here is total: it accepts any input, never panics and
//! never allocates more than a small multiple of its input. A parser returns
//! `None` where the reference throws, and the caller applies its default or
//! rejects the event (parity-map K-02, K-03, K-08).
//!
//! Derivation (D11): the layout rules of the `Double.toString`,
//! `Double.valueOf` and `Integer.parseInt` Javadoc; the IEEE 754 binary64
//! encoding; the Unicode Character Database (`Nd` digits); the Joda-Time
//! pattern and ISO formatter documentation (Apache-2.0); and black-box
//! observation of a JDK and Joda-Time on the corpus inputs. No reference code
//! was translated.
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation)
// - ASD STIG: APSC-DV-002530 / V-222606 (unverified)

use std::cmp::Ordering;

/// Decimal digit string and exponent of a positive finite double:
/// value = `d1.d2d3…` × 10^`exp`, with no leading or trailing zero digit.
struct Decimal {
    digits: Vec<u8>,
    exp: i64,
}

/// Formats `x` as the reference's `Double.toString` does.
///
/// The layout follows the `Double.toString` Javadoc: `NaN`, `Infinity`,
/// `-Infinity`, `0.0`, `-0.0`; plain notation with at least one fractional
/// digit for 10⁻³ ≤ |x| < 10⁷; otherwise `d.dddE±n` with at least one
/// fractional digit. The digits are the shortest that round-trip (from
/// `ryu`), and when that is a single digit the closest of the one- and
/// two-digit decimals that round-trip, as the current `Double.toString`
/// specification says (so `Double.MIN_VALUE` prints `4.9E-324`).
///
/// JDK 17, the reference's runtime, sometimes prints a longer digit string
/// than this; that difference is deviation DEV-03 (approved, gated on the
/// curated corpus set).
///
// PARITY: src/takserver-plugins/src/main/java/tak/server/proto/StreamingProtoBufHelper.java#proto2cot
//         (point and track values printed with JDK 17 Double.toString; DEV-03)
#[must_use]
pub fn double_to_string(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_owned();
    }
    if x.is_infinite() {
        return if x.is_sign_negative() {
            "-Infinity".to_owned()
        } else {
            "Infinity".to_owned()
        };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        };
    }
    let abs = x.abs();
    let mut buffer = ryu::Buffer::new();
    let Some(mut decimal) = decimal_parts(buffer.format_finite(abs)) else {
        // ryu always prints digits for a finite value; fall back to Rust's own
        // shortest form rather than panic.
        return format!("{x:?}");
    };
    if decimal.digits.len() == 1
        && let Some(two) = closest_two_digit(abs)
    {
        decimal = two;
    }

    let mut out = String::with_capacity(decimal.digits.len().saturating_add(8));
    if x.is_sign_negative() {
        out.push('-');
    }
    if (-3..7).contains(&decimal.exp) {
        push_plain(&mut out, &decimal);
    } else {
        push_scientific(&mut out, &decimal);
    }
    out
}

/// The closest two-significant-digit decimal to `abs`, if it parses back to
/// `abs`. Rust's `{:.1e}` rounds the exact binary value half-to-even, which
/// picks the even significand on a tie, as the specification requires.
fn closest_two_digit(abs: f64) -> Option<Decimal> {
    let text = format!("{abs:.1e}");
    let back: f64 = text.parse().ok()?;
    if back.to_bits() == abs.to_bits() {
        decimal_parts(&text)
    } else {
        None
    }
}

/// Splits `d[.d…][e[-]n]` (the `ryu` and `{:e}` output forms of a positive
/// finite value) into significant digits and a scientific exponent.
fn decimal_parts(text: &str) -> Option<Decimal> {
    let (mantissa, exp_text) = match text.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (text, None),
    };
    let exp: i64 = match exp_text {
        Some(e) => e.parse().ok()?,
        None => 0,
    };
    let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all = int_part.bytes().chain(frac_part.bytes());
    let mut digits = Vec::with_capacity(mantissa.len());
    let mut leading_zeros: i64 = 0;
    for b in all {
        let d = b.checked_sub(b'0').filter(|d| *d <= 9)?;
        if digits.is_empty() && d == 0 {
            leading_zeros = leading_zeros.checked_add(1)?;
        } else {
            digits.push(d);
        }
    }
    while digits.last() == Some(&0) {
        digits.pop();
    }
    if digits.is_empty() {
        return None;
    }
    let int_len = i64::try_from(int_part.len()).ok()?;
    let exp = exp
        .checked_add(int_len)?
        .checked_sub(1)?
        .checked_sub(leading_zeros)?;
    Some(Decimal { digits, exp })
}

fn digit_char(d: u8) -> char {
    char::from(b'0'.saturating_add(d))
}

/// `123.45`, `0.00123`, `1000000.0`: used for exponents -3 to 6.
fn push_plain(out: &mut String, decimal: &Decimal) {
    if let Ok(exp) = usize::try_from(decimal.exp) {
        let int_len = exp.saturating_add(1);
        for i in 0..int_len {
            out.push(digit_char(decimal.digits.get(i).copied().unwrap_or(0)));
        }
        out.push('.');
        let frac = decimal.digits.get(int_len..).unwrap_or(&[]);
        if frac.is_empty() {
            out.push('0');
        } else {
            out.extend(frac.iter().copied().map(digit_char));
        }
    } else {
        out.push_str("0.");
        let zeros = decimal.exp.unsigned_abs().saturating_sub(1);
        for _ in 0..zeros {
            out.push('0');
        }
        out.extend(decimal.digits.iter().copied().map(digit_char));
    }
}

/// `1.0E7`, `4.9E-324`, `1.2345E-4`.
fn push_scientific(out: &mut String, decimal: &Decimal) {
    let mut digits = decimal.digits.iter().copied().map(digit_char);
    out.push(digits.next().unwrap_or('0'));
    out.push('.');
    let before = out.len();
    out.extend(digits);
    if out.len() == before {
        out.push('0');
    }
    out.push('E');
    out.push_str(&decimal.exp.to_string());
}

/// Parses `text` with the `Double.valueOf(String)` grammar; `None` where the
/// reference throws `NumberFormatException` (the caller applies its default,
/// as `parseDoubleOrDefault` does).
///
/// Accepted, after trimming every leading and trailing character up to
/// U+0020: an optional sign, then `NaN`, `Infinity`, a decimal literal
/// (ASCII digits, optional `.`, optional exponent) or a hexadecimal
/// floating-point literal (`0x`, hex digits, a required binary exponent), each
/// number optionally followed by one of `f`, `F`, `d`, `D`. The value is
/// correctly rounded to the nearest double (ties to even), overflowing to
/// infinity and underflowing to zero; the suffix does not change it. Only
/// ASCII digits are accepted, case matters for `NaN` and `Infinity`.
///
// PARITY: src/takserver-plugins/src/main/java/tak/server/util/NumericUtil.java#parseDoubleOrDefault
//         (grammar from the JDK 17 Double#valueOf(String) Javadoc)
#[must_use]
pub fn parse_double(text: &str) -> Option<f64> {
    let trimmed = text.trim_matches(|c: char| c <= '\u{20}');
    let (negative, body) = match trimmed.as_bytes().first() {
        Some(b'-') => (true, trimmed.get(1..)?),
        Some(b'+') => (false, trimmed.get(1..)?),
        _ => (false, trimmed),
    };
    let magnitude = match body {
        "NaN" => return Some(f64::NAN),
        "Infinity" => f64::INFINITY,
        _ => {
            let bytes = body.as_bytes();
            let bytes = match bytes.last() {
                Some(b'f' | b'F' | b'd' | b'D') => bytes.get(..bytes.len().saturating_sub(1))?,
                _ => bytes,
            };
            match bytes {
                [b'0', b'x' | b'X', rest @ ..] => parse_hex(rest)?,
                _ => parse_decimal(bytes)?,
            }
        }
    };
    Some(if negative { -magnitude } else { magnitude })
}

/// Largest exponent magnitude tracked; anything larger already makes every
/// input under this cap overflow or underflow, and saturating keeps the
/// arithmetic in range.
const EXP_CAP: i64 = 1_000_000_000_000_000;

/// Splits `digits[.digits]` at the first byte that is not part of it.
/// Returns (integer digits, fraction digits, rest).
fn split_mantissa(bytes: &[u8], is_digit: fn(&u8) -> bool) -> (&[u8], &[u8], &[u8]) {
    let int_len = bytes.iter().take_while(|b| is_digit(b)).count();
    let (int_part, rest) = bytes.split_at(int_len);
    match rest.split_first() {
        Some((b'.', after)) => {
            let frac_len = after.iter().take_while(|b| is_digit(b)).count();
            let (frac_part, rest) = after.split_at(frac_len);
            (int_part, frac_part, rest)
        }
        _ => (int_part, &[], rest),
    }
}

/// Parses `[+-]digits` (ASCII) to the end of `bytes`, saturating at
/// `EXP_CAP`. `None` if there is no digit or anything else follows.
fn parse_exponent(bytes: &[u8]) -> Option<i64> {
    let (negative, digits) = match bytes.split_first() {
        Some((b'-', rest)) => (true, rest),
        Some((b'+', rest)) => (false, rest),
        _ => (false, bytes),
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let value = digits.iter().fold(0_i64, |acc, b| {
        acc.saturating_mul(10)
            .saturating_add(i64::from(b.saturating_sub(b'0')))
            .min(EXP_CAP)
    });
    Some(if negative {
        value.saturating_neg()
    } else {
        value
    })
}

/// Decimal literal: `digits`, `digits.`, `.digits` or `digits.digits`, then
/// an optional `e`/`E` exponent. Returns the non-negative value.
fn parse_decimal(bytes: &[u8]) -> Option<f64> {
    let (int_part, frac_part, rest) = split_mantissa(bytes, u8::is_ascii_digit);
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let exp = match rest.split_first() {
        None => 0,
        Some((b'e' | b'E', exp_bytes)) => parse_exponent(exp_bytes)?,
        Some(_) => return None,
    };

    // Significant digits and the power of ten of the first one, computed
    // here so that the string handed to the correctly rounded `str::parse`
    // carries a small exponent whatever the input's digit count.
    let all = || int_part.iter().chain(frac_part.iter());
    let Some(first_nonzero) = all().position(|b| *b != b'0') else {
        return Some(0.0);
    };
    let int_len = i64::try_from(int_part.len()).ok()?;
    let position = i64::try_from(first_nonzero).ok()?;
    let lead_exp = int_len
        .checked_sub(1)?
        .checked_sub(position)?
        .saturating_add(exp);
    if lead_exp > 310 {
        return Some(f64::INFINITY);
    }
    if lead_exp < -330 {
        return Some(0.0);
    }
    let mut normalized = String::with_capacity(all().count().saturating_add(16));
    normalized.push_str("0.");
    normalized.extend(all().skip(first_nonzero).map(|b| char::from(*b)));
    normalized.push('e');
    normalized.push_str(&lead_exp.checked_add(1)?.to_string());
    normalized.parse().ok()
}

/// Hexadecimal literal after `0x`: hex digits with an optional `.` (at least
/// one digit), then `p`/`P` and a decimal exponent. Returns the non-negative
/// value rounded to nearest, ties to even, with gradual underflow.
fn parse_hex(bytes: &[u8]) -> Option<f64> {
    let (int_part, frac_part, rest) = split_mantissa(bytes, u8::is_ascii_hexdigit);
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let exp = match rest.split_first() {
        Some((b'p' | b'P', exp_bytes)) => parse_exponent(exp_bytes)?,
        _ => return None,
    };

    // Accumulate up to 64 significant bits; later non-zero digits only set
    // `sticky`. value = mantissa × 2^shift (+ something below the last bit).
    let mut mantissa: u64 = 0;
    let mut shift: i64 = exp;
    let mut sticky = false;
    for (index, b) in int_part.iter().chain(frac_part.iter()).enumerate() {
        let in_fraction = index >= int_part.len();
        let digit = u64::from(char::from(*b).to_digit(16)?);
        if mantissa == 0 && digit == 0 {
            if in_fraction {
                shift = shift.saturating_sub(4);
            }
        } else if mantissa.leading_zeros() >= 4 {
            mantissa = mantissa.checked_shl(4)? | digit;
            if in_fraction {
                shift = shift.saturating_sub(4);
            }
        } else {
            sticky |= digit != 0;
            if !in_fraction {
                shift = shift.saturating_add(4);
            }
        }
    }
    if mantissa == 0 {
        return Some(0.0);
    }
    Some(round_binary(mantissa, shift, sticky))
}

/// Rounds `mantissa × 2^shift` (plus a non-zero remainder below the last bit
/// when `sticky`) to the nearest double, ties to even. `mantissa` is non-zero.
fn round_binary(mantissa: u64, shift: i64, sticky: bool) -> f64 {
    let lz = mantissa.leading_zeros();
    let normalized = mantissa.checked_shl(lz).unwrap_or(0);
    // Exponent of the top bit: value = normalized / 2^63 × 2^top.
    let top = shift.saturating_add(63).saturating_sub(i64::from(lz));
    if top > 1023 {
        return f64::INFINITY;
    }
    // Bits kept: 53 for a normal result, fewer for a subnormal one.
    let kept: i64 = if top >= -1022 {
        53
    } else {
        top.saturating_add(1075)
    };
    if kept < 0 {
        return 0.0;
    }
    let drop = u32::try_from(64_i64.saturating_sub(kept)).unwrap_or(64);
    let quotient = normalized.checked_shr(drop).unwrap_or(0);
    let remainder = normalized.wrapping_sub(quotient.checked_shl(drop).unwrap_or(0));
    let half = 1_u64.checked_shl(drop.saturating_sub(1)).unwrap_or(0);
    let round_up = match remainder.cmp(&half) {
        Ordering::Greater => true,
        Ordering::Equal => sticky || quotient & 1 == 1,
        Ordering::Less => false,
    };
    let quotient = if round_up {
        quotient.saturating_add(1)
    } else {
        quotient
    };
    if kept < 53 {
        // Subnormal: the significand is the encoding; a carry into bit 52
        // yields the smallest normal number, which is also correct.
        return f64::from_bits(quotient);
    }
    // Normal: `quotient` is in [2^52, 2^53]; a carry to 2^53 bumps the exponent.
    let (quotient, top) = if quotient >> 53 == 1 {
        (quotient >> 1, top.saturating_add(1))
    } else {
        (quotient, top)
    };
    let Ok(biased) = u64::try_from(top.saturating_add(1023)) else {
        return 0.0;
    };
    if biased >= 0x7ff {
        return f64::INFINITY;
    }
    f64::from_bits((biased << 52) | (quotient & ((1_u64 << 52) - 1)))
}

/// Code points of the digit zero of every Unicode `Nd` (decimal digit) run in
/// the Basic Multilingual Plane; each run is ten consecutive code points. The
/// BMP set is the same in Unicode 13.0 (JDK 17) and later versions.
const BMP_DIGIT_ZEROS: [u32; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// Decimal value of `c` as `Character.digit(char, 10)` gives it: any BMP
/// `Nd` character. Characters outside the BMP are two UTF-16 units in Java
/// and never a digit there.
fn decimal_digit_value(c: char) -> Option<u32> {
    let code = u32::from(c);
    let index = BMP_DIGIT_ZEROS.partition_point(|zero| *zero <= code);
    let zero = BMP_DIGIT_ZEROS.get(index.checked_sub(1)?)?;
    let value = code.checked_sub(*zero)?;
    (value < 10).then_some(value)
}

/// Parses `text` with the `Integer.parseInt(String)` grammar; `None` where
/// the reference throws `NumberFormatException`.
///
/// An optional `+` or `-`, then one or more decimal digits, nothing else (no
/// whitespace), within the `i32` range. A digit is any Unicode `Nd` character
/// of the Basic Multilingual Plane (`١٢٣` is 123, `１２` is 12), as the
/// reference's `Character.digit` accepts.
///
// PARITY: src/takserver-plugins/src/main/java/tak/server/util/NumericUtil.java#parseIntOrDefault
//         (grammar from the JDK 17 Integer#parseInt(String) Javadoc)
#[must_use]
pub fn parse_int(text: &str) -> Option<i32> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if digits.is_empty() {
        return None;
    }
    // Accumulate the magnitude; the bound is 2^31 for a negative result.
    let limit: i64 = if negative {
        i64::from(i32::MAX).checked_add(1)?
    } else {
        i64::from(i32::MAX)
    };
    let mut magnitude: i64 = 0;
    for c in digits.chars() {
        let digit = decimal_digit_value(c)?;
        magnitude = magnitude
            .checked_mul(10)?
            .checked_add(i64::from(digit))
            .filter(|m| *m <= limit)?;
    }
    let value = if negative {
        magnitude.checked_neg()?
    } else {
        magnitude
    };
    i32::try_from(value).ok()
}

const MILLIS_PER_SECOND: i64 = 1_000;
const SECONDS_PER_DAY: i64 = 86_400;

/// Formats milliseconds since the epoch as a `CoT` time,
/// `yyyy-MM-dd'T'HH:mm:ss'Z'` in UTC, dropping the milliseconds (parity-map
/// K-14).
///
/// Calendar: proleptic Gregorian with a year 0, so 1 BCE is `0000` and 2 BCE
/// is `-0001`. The year has at least four digits, a `-` when negative and no
/// sign otherwise (`10000-01-01T00:00:00Z`). Every `i64` is printable, from
/// `-292275055-05-16T16:47:04Z` to `292278994-08-17T07:12:55Z`. Negative
/// values round toward the past (`-1` is `1969-12-31T23:59:59Z`).
///
// PARITY: src/takserver-plugins/src/main/java/com/bbn/marti/remote/util/DateUtil.java#toCotTime
#[must_use]
pub fn cot_time_to_string(ms: i64) -> String {
    // Cannot fail: `i64` milliseconds are about ±1.1e11 days, far inside the
    // range `civil_from_days` handles.
    format_cot_time(ms).unwrap_or_default()
}

fn format_cot_time(ms: i64) -> Option<String> {
    let seconds = ms.div_euclid(MILLIS_PER_SECOND);
    let days = seconds.div_euclid(SECONDS_PER_DAY);
    let second_of_day = seconds.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days)?;
    let hour = second_of_day.div_euclid(3_600);
    let minute = second_of_day.rem_euclid(3_600).div_euclid(60);
    let second = second_of_day.rem_euclid(60);
    let sign = if year < 0 { "-" } else { "" };
    Some(format!(
        "{sign}{:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
        year.unsigned_abs()
    ))
}

/// Days since 1970-01-01 to a proleptic Gregorian (year, month, day).
///
/// Counts in 400-year eras of 146 097 days, each starting on a 1 March so
/// that the leap day is the last day of the counted year.
fn civil_from_days(days: i64) -> Option<(i64, i64, i64)> {
    // 0000-03-01 is 719 468 days before 1970-01-01.
    let shifted = days.checked_add(719_468)?;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    // Year of the era: remove the leap days before `day_of_era`.
    let year_of_era = day_of_era
        .checked_sub(day_of_era.div_euclid(1_460))?
        .checked_add(day_of_era.div_euclid(36_524))?
        .checked_sub(day_of_era.div_euclid(146_096))?
        .div_euclid(365);
    let day_of_year = day_of_era.checked_sub(days_before_year_of_era(year_of_era)?)?;
    // Months from March: 31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 28/29.
    let month_from_march = day_of_year.checked_mul(5)?.checked_add(2)?.div_euclid(153);
    let day = day_of_year
        .checked_sub(days_before_month_from_march(month_from_march)?)?
        .checked_add(1)?;
    let month = if month_from_march < 10 {
        month_from_march.checked_add(3)?
    } else {
        month_from_march.checked_sub(9)?
    };
    let year = era
        .checked_mul(400)?
        .checked_add(year_of_era)?
        .checked_add(i64::from(month <= 2))?;
    Some((year, month, day))
}

/// Days in the first `year_of_era` years of an era (from 1 March).
fn days_before_year_of_era(year_of_era: i64) -> Option<i64> {
    year_of_era
        .checked_mul(365)?
        .checked_add(year_of_era.div_euclid(4))?
        .checked_sub(year_of_era.div_euclid(100))
}

/// Days in a March-based year before month `month_from_march` (0 = March).
fn days_before_month_from_march(month_from_march: i64) -> Option<i64> {
    Some(
        month_from_march
            .checked_mul(153)?
            .checked_add(2)?
            .div_euclid(5),
    )
}

/// Proleptic Gregorian (year, month 1–12, day) to days since 1970-01-01.
/// `None` only on arithmetic overflow.
fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    let (year, month_from_march) = if month <= 2 {
        (year.checked_sub(1)?, month.checked_add(9)?)
    } else {
        (year, month.checked_sub(3)?)
    };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_era = days_before_year_of_era(year_of_era)?
        .checked_add(days_before_month_from_march(month_from_march)?)?
        .checked_add(day)?
        .checked_sub(1)?;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
}

fn is_leap_year(year: i64) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Byte cursor over an ASCII grammar; every read is bounds-checked.
struct Cursor<'a> {
    rest: &'a [u8],
}

impl Cursor<'_> {
    fn peek(&self) -> Option<u8> {
        self.rest.first().copied()
    }

    fn bump(&mut self) {
        self.rest = self.rest.get(1..).unwrap_or(&[]);
    }

    /// Consumes `expected`, ASCII case-insensitively.
    fn literal(&mut self, expected: u8) -> Option<()> {
        self.peek()
            .filter(|b| b.eq_ignore_ascii_case(&expected))
            .map(|_| self.bump())
    }

    /// Counts the ASCII digits at the cursor, up to `max`, without consuming.
    fn digit_run(&self, max: usize) -> usize {
        self.rest
            .iter()
            .take(max)
            .take_while(|b| b.is_ascii_digit())
            .count()
    }

    /// Consumes `count` ASCII digits (already counted) as a number.
    fn take_digits(&mut self, count: usize) -> i64 {
        let mut value: i64 = 0;
        for _ in 0..count {
            let digit = self.peek().map_or(0, |b| b.saturating_sub(b'0'));
            value = value.saturating_mul(10).saturating_add(i64::from(digit));
            self.bump();
        }
        value
    }

    /// One to `max` ASCII digits.
    fn number(&mut self, max: usize) -> Option<i64> {
        let count = self.digit_run(max);
        (count > 0).then(|| self.take_digits(count))
    }
}

/// Parses a `CoT` time to milliseconds since the epoch; `None` where the
/// reference throws (the conversion then fails, parity-map K-02).
///
/// Two grammars, chosen as the reference chooses them, by whether the text
/// contains a `.`. Both start with
/// `year-month-dayThour:minute:second`, where the year is one to nine ASCII
/// digits with an optional `+` or `-`, every other field one or two digits,
/// and `T` is matched in either case.
///
/// - Without a `.`: then `Z` or `z`, and nothing else.
/// - With a `.`: then `.`, one to nine fraction digits (truncated to
///   milliseconds), and an offset: `Z`/`z`, or `+`/`-` with a two-digit hour
///   (at most 23) followed optionally by minutes, seconds and up to three
///   fraction-of-second digits, either all separated (`+05:30:15.123`, `,`
///   allowed for `.`) or all run together (`+053015123`).
///
/// Fields must form a real proleptic Gregorian date and time (month 1–12,
/// day within the month, hour 0–23, minute and second 0–59). No surrounding
/// whitespace is allowed.
///
/// The local time and the result must both fit in `i64` milliseconds. The
/// reference wraps around instead for the few months of years ±292 million
/// at the edges of that range; that difference is recorded in the impl notes
/// as a deviation candidate.
///
// PARITY: src/takserver-plugins/src/main/java/com/bbn/marti/remote/util/DateUtil.java#millisFromCotTimeStr
#[must_use]
pub fn parse_cot_time(text: &str) -> Option<i64> {
    let with_fraction = text.contains('.');
    let mut cursor = Cursor {
        rest: text.as_bytes(),
    };

    let year = parse_year(&mut cursor)?;
    cursor.literal(b'-')?;
    let month = cursor.number(2)?;
    cursor.literal(b'-')?;
    let day = cursor.number(2)?;
    cursor.literal(b'T')?;
    let hour = cursor.number(2)?;
    cursor.literal(b':')?;
    let minute = cursor.number(2)?;
    cursor.literal(b':')?;
    let second = cursor.number(2)?;

    let (millis, offset_ms) = if with_fraction {
        cursor.literal(b'.')?;
        let count = cursor.digit_run(9);
        if count == 0 {
            return None;
        }
        let millis = fraction_millis(cursor.rest.get(..count)?);
        cursor.take_digits(count);
        (millis, parse_offset(&mut cursor)?)
    } else {
        cursor.literal(b'Z')?;
        (0, 0)
    };
    if !cursor.rest.is_empty() {
        return None;
    }

    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    let days = days_from_civil(year, month, day)?;
    let second_of_day = hour
        .checked_mul(3_600)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)?;
    // In i128 so that only the local time and the result are range-checked,
    // not intermediate sums (`-292275055-05-16T16:47:04.192Z` is `i64::MIN`).
    let local = i128::from(days)
        .checked_mul(i128::from(SECONDS_PER_DAY))?
        .checked_add(i128::from(second_of_day))?
        .checked_mul(i128::from(MILLIS_PER_SECOND))?
        .checked_add(i128::from(millis))?;
    let local = i64::try_from(local).ok()?;
    local.checked_sub(offset_ms)
}

/// Year: optional `+`/`-` directly followed by a digit, then one to nine
/// ASCII digits.
fn parse_year(cursor: &mut Cursor<'_>) -> Option<i64> {
    let negative = match cursor.peek() {
        Some(sign @ (b'-' | b'+')) => {
            if !cursor.rest.get(1).is_some_and(u8::is_ascii_digit) {
                return None;
            }
            cursor.bump();
            sign == b'-'
        }
        _ => false,
    };
    let magnitude = cursor.number(9)?;
    if negative {
        magnitude.checked_neg()
    } else {
        Some(magnitude)
    }
}

/// Milliseconds from fraction-of-second digits: the first three digits, the
/// rest truncated (`5` is 500, `0005` is 0).
fn fraction_millis(digits: &[u8]) -> i64 {
    [100_i64, 10, 1]
        .into_iter()
        .zip(digits.iter())
        .map(|(scale, b)| scale.saturating_mul(i64::from(b.saturating_sub(b'0'))))
        .fold(0, i64::saturating_add)
}

/// ISO offset after the fraction: `Z`/`z`, or `±hh[[:]mm[[:]ss[[.,]f{1,3}]]]`
/// with separators either everywhere or nowhere. Parsing stops quietly at
/// the first byte that does not continue the offset; the caller then
/// requires the end of input. Returns the offset in milliseconds.
fn parse_offset(cursor: &mut Cursor<'_>) -> Option<i64> {
    if cursor.literal(b'Z').is_some() {
        return Some(0);
    }
    let negative = match cursor.peek() {
        Some(b'+') => false,
        Some(b'-') => true,
        _ => return None,
    };
    cursor.bump();
    if cursor.digit_run(2) < 2 {
        return None;
    }
    let hours = cursor.take_digits(2);
    if hours > 23 {
        return None;
    }
    let mut offset = hours.checked_mul(3_600_000)?;
    let magnitude = 'fields: {
        let separated = match cursor.peek() {
            Some(b':') => {
                cursor.bump();
                true
            }
            Some(b) if b.is_ascii_digit() => false,
            _ => break 'fields offset,
        };
        // Minutes, then seconds: two digits each, at most 59.
        for (index, unit) in [60_000_i64, 1_000].into_iter().enumerate() {
            if index > 0 {
                if cursor.peek().is_none() {
                    break 'fields offset;
                }
                if separated {
                    if cursor.peek() != Some(b':') {
                        break 'fields offset;
                    }
                    cursor.bump();
                }
            }
            match cursor.digit_run(2) {
                0 if !separated => break 'fields offset,
                2 => {}
                _ => return None,
            }
            let value = cursor.take_digits(2);
            if value > 59 {
                return None;
            }
            offset = offset.checked_add(value.checked_mul(unit)?)?;
        }
        // Fraction of a second: one to three digits.
        if cursor.peek().is_none() {
            break 'fields offset;
        }
        if separated {
            if !matches!(cursor.peek(), Some(b'.' | b',')) {
                break 'fields offset;
            }
            cursor.bump();
        }
        let count = cursor.digit_run(3);
        if count == 0 {
            if separated {
                return None;
            }
            break 'fields offset;
        }
        let fraction = cursor.rest.get(..count).map_or(0, fraction_millis);
        cursor.take_digits(count);
        offset.checked_add(fraction)?
    };
    if negative {
        magnitude.checked_neg()
    } else {
        Some(magnitude)
    }
}
