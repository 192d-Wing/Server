// SPDX-License-Identifier: Apache-2.0

//! XML 1.0 `Char`, `Name` and `QName` checks, which quick-xml does not
//! perform.
//!
//! Derived from the XML 1.0 (Fifth Edition) productions `Char` [2], `S` [3],
//! `NameStartChar` [4], `NameChar` [4a] and `Name` [5], and from Namespaces
//! in XML 1.0 (Third Edition) `NCName` [4] and `QName` [7]. Whether the
//! reference's parser uses the Fifth Edition name tables or the older
//! Appendix B tables is capture-determined (parity-map C-10).
//
// COMPLIANCE MAPPING:
// - NIST 800-53: SI-10 (Information Input Validation)

/// XML 1.0 `Char`: tab, line feed, carriage return, and every scalar value
/// from U+0020 except U+FFFE and U+FFFF (Rust `char` already excludes
/// surrogates).
pub(crate) const fn is_xml_char(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
    )
}

/// XML 1.0 `S` (white space).
pub(crate) const fn is_xml_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

const fn is_name_start_char(c: char) -> bool {
    matches!(
        c,
        ':' | 'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{C0}'..='\u{D6}'
            | '\u{D8}'..='\u{F6}'
            | '\u{F8}'..='\u{2FF}'
            | '\u{370}'..='\u{37D}'
            | '\u{37F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

const fn is_name_char(c: char) -> bool {
    is_name_start_char(c)
        || matches!(
            c,
            '-' | '.' | '0'..='9' | '\u{B7}' | '\u{300}'..='\u{36F}' | '\u{203F}'..='\u{2040}'
        )
}

/// XML 1.0 `Name`.
pub(crate) fn is_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if is_name_start_char(first) => chars.all(is_name_char),
        _ => false,
    }
}

/// Namespaces `NCName`: a `Name` without a colon.
pub(crate) fn is_ncname(s: &str) -> bool {
    !s.contains(':') && is_name(s)
}

/// Splits a Namespaces `QName` into `(prefix, local part)`, or returns
/// `None` when `s` is not a `QName` (empty part, more than one colon, or a
/// part that is not an `NCName`).
pub(crate) fn split_qname(s: &str) -> Option<(Option<&str>, &str)> {
    match s.split_once(':') {
        Some((prefix, local)) if is_ncname(prefix) && is_ncname(local) => {
            Some((Some(prefix), local))
        }
        None if is_name(s) => Some((None, s)),
        _ => None,
    }
}

/// Byte index of the first character of `s` that is not an XML 1.0 `Char`.
pub(crate) fn first_invalid_char(s: &str) -> Option<usize> {
    s.char_indices()
        .find_map(|(i, c)| (!is_xml_char(c)).then_some(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chars() {
        assert!(is_xml_char('\t') && is_xml_char('\n') && is_xml_char('\r'));
        assert!(!is_xml_char('\0') && !is_xml_char('\u{1}') && !is_xml_char('\u{1F}'));
        assert!(is_xml_char('\u{D7FF}') && is_xml_char('\u{E000}') && is_xml_char('\u{FFFD}'));
        assert!(!is_xml_char('\u{FFFE}') && !is_xml_char('\u{FFFF}'));
        assert!(is_xml_char('\u{10000}') && is_xml_char('\u{10FFFF}'));
        assert_eq!(first_invalid_char("ab\u{1}c"), Some(2));
        assert_eq!(first_invalid_char("\u{e9}\u{FFFF}"), Some(2));
        assert_eq!(first_invalid_char("ok"), None);
    }

    #[test]
    fn names() {
        for ok in [
            "a",
            "_a",
            "a-b.c",
            "x1",
            "\u{e9}t\u{e9}",
            "a\u{B7}",
            ":",
            "a:b:c",
        ] {
            assert!(is_name(ok), "{ok}");
        }
        for bad in ["", "1a", "-a", ".a", "a b", "a\u{D7}", "\u{300}a"] {
            assert!(!is_name(bad), "{bad}");
        }
        assert!(is_ncname("a.b") && !is_ncname("a:b"));
    }

    #[test]
    fn qnames() {
        assert_eq!(split_qname("a"), Some((None, "a")));
        assert_eq!(split_qname("p:a"), Some((Some("p"), "a")));
        for bad in [":a", "a:", "a:b:c", ":", "1:a", "a:1", ""] {
            assert_eq!(split_qname(bad), None, "{bad}");
        }
    }
}
