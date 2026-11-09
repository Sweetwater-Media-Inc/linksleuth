use std::char;

/// Maximum complete value accepted by a secondary destination decoder.
pub const MAX_SECONDARY_ENCODING_INPUT_BYTES: usize = 16 * 1024;

/// Maximum decoded text retained from a secondary destination decoder.
pub const MAX_SECONDARY_DECODED_BYTES: usize = 8 * 1024;

/// Maximum bytes searched for one semicolon-terminated HTML reference token.
pub const MAX_HTML_REFERENCE_TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64Alphabet {
    Standard,
    UrlSafe,
    /// The value uses only characters shared by both alphabets.
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64Padding {
    Padded,
    Unpadded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64DecodingResult {
    pub decoded: String,
    pub alphabet: Base64Alphabet,
    pub padding: Base64Padding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base64DecodingOutcome {
    NotApplicable,
    Decoded(Base64DecodingResult),
    Malformed,
    InvalidUtf8,
    InputTooLong,
    OutputTooLong,
}

/// Decodes one complete, strict Base64 value without searching substrings.
///
/// Standard and URL-safe alphabets are supported with canonical padded or
/// unpadded forms. Inputs without alphabet-specific characters are reported as
/// [`Base64Alphabet::Ambiguous`]. Invalid characters make the profile not
/// applicable; malformed padding, mixed alphabets, and non-zero trailing bits
/// are rejected.
pub fn decode_base64_complete(input: &str) -> Base64DecodingOutcome {
    if input.len() > MAX_SECONDARY_ENCODING_INPUT_BYTES {
        return Base64DecodingOutcome::InputTooLong;
    }

    if input.len() < 2 || !input.is_ascii() {
        return Base64DecodingOutcome::NotApplicable;
    }

    let has_standard = input.bytes().any(|byte| matches!(byte, b'+' | b'/'));
    let has_url_safe = input.bytes().any(|byte| matches!(byte, b'-' | b'_'));
    let has_padding = input.contains('=');

    if input.bytes().any(|byte| {
        !byte.is_ascii_alphanumeric() && !matches!(byte, b'+' | b'/' | b'-' | b'_' | b'=')
    }) {
        return Base64DecodingOutcome::NotApplicable;
    }

    if has_standard && has_url_safe {
        return Base64DecodingOutcome::Malformed;
    }

    let alphabet = if has_standard {
        Base64Alphabet::Standard
    } else if has_url_safe {
        Base64Alphabet::UrlSafe
    } else {
        Base64Alphabet::Ambiguous
    };
    let padding = if has_padding {
        Base64Padding::Padded
    } else {
        Base64Padding::Unpadded
    };

    let padding_count = input.bytes().rev().take_while(|byte| *byte == b'=').count();
    let data_len = input.len() - padding_count;
    let malformed = padding_count > 2
        || input[..data_len].contains('=')
        || (padding_count > 0 && !input.len().is_multiple_of(4))
        || (padding_count == 1 && data_len % 4 != 3)
        || (padding_count == 2 && data_len % 4 != 2)
        || (padding_count == 0 && data_len % 4 == 1);

    if malformed {
        return if has_padding {
            Base64DecodingOutcome::Malformed
        } else {
            Base64DecodingOutcome::NotApplicable
        };
    }

    let decoded_length = (data_len / 4) * 3
        + match data_len % 4 {
            0 => 0,
            2 => 1,
            3 => 2,
            _ => unreachable!("Base64 remainder was validated"),
        };
    if decoded_length > MAX_SECONDARY_DECODED_BYTES {
        return Base64DecodingOutcome::OutputTooLong;
    }

    let mut decoded = Vec::with_capacity(decoded_length);
    let mut buffer = 0_u32;
    let mut buffered_bits = 0_u8;

    for byte in input[..data_len].bytes() {
        let Some(value) = base64_value(byte, alphabet) else {
            return Base64DecodingOutcome::Malformed;
        };
        buffer = (buffer << 6) | u32::from(value);
        buffered_bits += 6;

        while buffered_bits >= 8 {
            buffered_bits -= 8;
            decoded.push(((buffer >> buffered_bits) & 0xff) as u8);
            buffer &= (1_u32 << buffered_bits) - 1;
        }
    }

    if buffer != 0 || decoded.len() != decoded_length {
        return if has_padding {
            Base64DecodingOutcome::Malformed
        } else {
            Base64DecodingOutcome::NotApplicable
        };
    }

    match String::from_utf8(decoded) {
        Ok(decoded) => Base64DecodingOutcome::Decoded(Base64DecodingResult {
            decoded,
            alphabet,
            padding,
        }),
        Err(_) => Base64DecodingOutcome::InvalidUtf8,
    }
}

fn base64_value(byte: u8, alphabet: Base64Alphabet) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' if alphabet != Base64Alphabet::UrlSafe => Some(62),
        b'/' if alphabet != Base64Alphabet::UrlSafe => Some(63),
        b'-' if alphabet == Base64Alphabet::UrlSafe => Some(62),
        b'_' if alphabet == Base64Alphabet::UrlSafe => Some(63),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlReferenceIssueKind {
    MalformedReference,
    UnsupportedNamedReference,
    InvalidUnicodeScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlReferenceDecodingStatus {
    Complete,
    Incomplete { issues: Vec<HtmlReferenceIssueKind> },
}

impl HtmlReferenceDecodingStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn issues(&self) -> &[HtmlReferenceIssueKind] {
        match self {
            Self::Complete => &[],
            Self::Incomplete { issues } => issues,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlReferenceDecodingResult {
    pub decoded: String,
    pub reference_count: usize,
    pub status: HtmlReferenceDecodingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlReferenceDecodingOutcome {
    NotApplicable,
    Decoded(HtmlReferenceDecodingResult),
    InputTooLong,
    OutputTooLong,
}

/// Decodes supported character references in one complete value.
///
/// Numeric decimal and hexadecimal references are supported, along with the
/// exact semicolon-terminated named set `&colon;`, `&sol;`, and `&amp;`.
/// Unsupported or malformed references are retained in `decoded` and reported
/// through the typed status rather than silently substituted.
pub fn decode_html_character_references_complete(input: &str) -> HtmlReferenceDecodingOutcome {
    if input.len() > MAX_SECONDARY_ENCODING_INPUT_BYTES {
        return HtmlReferenceDecodingOutcome::InputTooLong;
    }

    if !looks_like_html_reference(input) {
        return HtmlReferenceDecodingOutcome::NotApplicable;
    }

    let mut decoded = String::with_capacity(input.len());
    let mut issues = Vec::new();
    let mut reference_count = 0;
    let mut index = 0;

    while index < input.len() {
        let Some(relative_ampersand) = input[index..].find('&') else {
            decoded.push_str(&input[index..]);
            break;
        };
        let ampersand = index + relative_ampersand;
        decoded.push_str(&input[index..ampersand]);

        let reference_start = ampersand + 1;
        let semicolon = input.as_bytes()[reference_start..]
            .iter()
            .take(MAX_HTML_REFERENCE_TOKEN_BYTES)
            .position(|byte| *byte == b';')
            .map(|relative| reference_start + relative);

        let Some(semicolon) = semicolon else {
            if starts_supported_reference_without_semicolon(&input[ampersand..]) {
                add_unique(&mut issues, HtmlReferenceIssueKind::MalformedReference);
            }
            decoded.push('&');
            index = reference_start;
            continue;
        };

        let token = &input[reference_start..semicolon];
        match decode_html_reference_token(token) {
            Ok(Some(character)) => {
                decoded.push(character);
                reference_count += 1;
            }
            Ok(None) => {
                add_unique(
                    &mut issues,
                    HtmlReferenceIssueKind::UnsupportedNamedReference,
                );
                decoded.push_str(&input[ampersand..=semicolon]);
            }
            Err(issue) => {
                add_unique(&mut issues, issue);
                decoded.push_str(&input[ampersand..=semicolon]);
            }
        }
        index = semicolon + 1;

        if decoded.len() > MAX_SECONDARY_DECODED_BYTES {
            return HtmlReferenceDecodingOutcome::OutputTooLong;
        }
    }

    if decoded.len() > MAX_SECONDARY_DECODED_BYTES {
        return HtmlReferenceDecodingOutcome::OutputTooLong;
    }

    let status = if issues.is_empty() {
        HtmlReferenceDecodingStatus::Complete
    } else {
        HtmlReferenceDecodingStatus::Incomplete { issues }
    };

    HtmlReferenceDecodingOutcome::Decoded(HtmlReferenceDecodingResult {
        decoded,
        reference_count,
        status,
    })
}

fn looks_like_html_reference(input: &str) -> bool {
    input.contains("&#")
        || input.contains("&colon")
        || input.contains("&sol")
        || input.contains("&amp")
        || input.as_bytes().windows(2).any(|window| {
            window[0] == b'&' && window[1].is_ascii_alphabetic() && input.contains(';')
        })
}

fn starts_supported_reference_without_semicolon(input: &str) -> bool {
    input.starts_with("&#")
        || input.starts_with("&colon")
        || input.starts_with("&sol")
        || input.starts_with("&amp")
}

fn decode_html_reference_token(token: &str) -> Result<Option<char>, HtmlReferenceIssueKind> {
    match token {
        "colon" => Ok(Some(':')),
        "sol" => Ok(Some('/')),
        "amp" => Ok(Some('&')),
        _ if token.starts_with("#x") || token.starts_with("#X") => {
            decode_numeric_reference(&token[2..], 16)
        }
        _ if token.starts_with('#') => decode_numeric_reference(&token[1..], 10),
        _ if !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_alphanumeric()) => {
            Ok(None)
        }
        _ => Err(HtmlReferenceIssueKind::MalformedReference),
    }
}

fn decode_numeric_reference(
    digits: &str,
    radix: u32,
) -> Result<Option<char>, HtmlReferenceIssueKind> {
    if digits.is_empty() {
        return Err(HtmlReferenceIssueKind::MalformedReference);
    }

    let value = u32::from_str_radix(digits, radix)
        .map_err(|_| HtmlReferenceIssueKind::InvalidUnicodeScalar)?;
    char::from_u32(value)
        .map(Some)
        .ok_or(HtmlReferenceIssueKind::InvalidUnicodeScalar)
}

fn add_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_standard_padded_and_unpadded_base64() {
        let padded = decode_base64_complete("aHR0cHM6Ly9zdGFuZGFyZC5leGFtcGxlL/CfmIA=");
        let unpadded = decode_base64_complete("aHR0cHM6Ly9vdGhlci5leGFtcGxlLw");

        assert_eq!(
            padded,
            Base64DecodingOutcome::Decoded(Base64DecodingResult {
                decoded: "https://standard.example/😀".to_string(),
                alphabet: Base64Alphabet::Standard,
                padding: Base64Padding::Padded,
            })
        );
        assert_eq!(
            unpadded,
            Base64DecodingOutcome::Decoded(Base64DecodingResult {
                decoded: "https://other.example/".to_string(),
                alphabet: Base64Alphabet::Ambiguous,
                padding: Base64Padding::Unpadded,
            })
        );
    }

    #[test]
    fn decodes_url_safe_base64_and_preserves_variant() {
        let result = decode_base64_complete("aHR0cHM6Ly91cmxzYWZlLmV4YW1wbGUv8J-YgA");

        assert_eq!(
            result,
            Base64DecodingOutcome::Decoded(Base64DecodingResult {
                decoded: "https://urlsafe.example/😀".to_string(),
                alphabet: Base64Alphabet::UrlSafe,
                padding: Base64Padding::Unpadded,
            })
        );
    }

    #[test]
    fn decodes_short_complete_protocol_relative_reference() {
        assert_eq!(
            decode_base64_complete("Ly9h"),
            Base64DecodingOutcome::Decoded(Base64DecodingResult {
                decoded: "//a".to_string(),
                alphabet: Base64Alphabet::Ambiguous,
                padding: Base64Padding::Unpadded,
            })
        );
    }

    #[test]
    fn rejects_malformed_base64_profiles_and_invalid_utf8() {
        assert_eq!(
            decode_base64_complete("aHR0cHM6Ly9vdGhlci5leGFtcGxlLw==="),
            Base64DecodingOutcome::Malformed
        );
        assert_eq!(
            decode_base64_complete("aHR0cHM6Ly9vdGhlci5leGFtcGxlLw**"),
            Base64DecodingOutcome::NotApplicable
        );
        assert_eq!(
            decode_base64_complete("________"),
            Base64DecodingOutcome::InvalidUtf8
        );
    }

    #[test]
    fn ordinary_base64_text_does_not_imply_a_destination() {
        let result = decode_base64_complete("aGVsbG8gd29ybGQ=");
        let Base64DecodingOutcome::Decoded(result) = result else {
            panic!("expected valid Base64 text");
        };

        assert_eq!(result.decoded, "hello world");
    }

    #[test]
    fn decodes_decimal_hexadecimal_and_supported_named_references() {
        let decimal =
            decode_html_character_references_complete("https&#58;&#47;&#47;decimal.example&#47;");
        let hexadecimal =
            decode_html_character_references_complete("https&#x3A;&#x2F;&#X2f;hex.example&#x2f;");
        let named = decode_html_character_references_complete(
            "https&colon;&sol;&sol;named.example&sol;?a=1&amp;b=2",
        );

        for (result, expected) in [
            (decimal, "https://decimal.example/"),
            (hexadecimal, "https://hex.example/"),
            (named, "https://named.example/?a=1&b=2"),
        ] {
            let HtmlReferenceDecodingOutcome::Decoded(result) = result else {
                panic!("expected HTML-reference decoding");
            };
            assert_eq!(result.decoded, expected);
            assert!(result.status.is_complete());
        }
    }

    #[test]
    fn preserves_and_reports_malformed_unsupported_and_invalid_references() {
        let result = decode_html_character_references_complete(
            "https&colon;&sol;&sol;example.test&unknown;&#xD800;",
        );
        let HtmlReferenceDecodingOutcome::Decoded(result) = result else {
            panic!("expected HTML-reference decoding");
        };

        assert_eq!(result.decoded, "https://example.test&unknown;&#xD800;");
        assert_eq!(
            result.status.issues(),
            &[
                HtmlReferenceIssueKind::UnsupportedNamedReference,
                HtmlReferenceIssueKind::InvalidUnicodeScalar,
            ]
        );

        let missing_semicolon =
            decode_html_character_references_complete("https&colon&sol;&sol;example.test");
        let HtmlReferenceDecodingOutcome::Decoded(result) = missing_semicolon else {
            panic!("expected malformed HTML-reference coverage");
        };
        assert!(
            result
                .status
                .issues()
                .contains(&HtmlReferenceIssueKind::MalformedReference)
        );

        let unicode_token = decode_html_character_references_complete("&unknown雪;value");
        let HtmlReferenceDecodingOutcome::Decoded(result) = unicode_token else {
            panic!("expected malformed Unicode HTML-reference coverage");
        };
        assert_eq!(result.decoded, "&unknown雪;value");
        assert_eq!(
            result.status.issues(),
            &[HtmlReferenceIssueKind::MalformedReference]
        );
    }

    #[test]
    fn rejects_invalid_numeric_ranges_and_enforces_input_limit() {
        for input in ["&#x110000;", "&#55296;", "&#xZZ;"] {
            let HtmlReferenceDecodingOutcome::Decoded(result) =
                decode_html_character_references_complete(input)
            else {
                panic!("expected HTML-reference decoding status");
            };
            assert_eq!(
                result.status.issues(),
                &[HtmlReferenceIssueKind::InvalidUnicodeScalar]
            );
        }

        let oversized = format!("&colon;{}", "a".repeat(MAX_SECONDARY_ENCODING_INPUT_BYTES));
        assert_eq!(
            decode_html_character_references_complete(&oversized),
            HtmlReferenceDecodingOutcome::InputTooLong
        );

        let output_too_long = format!("&colon;{}", "a".repeat(MAX_SECONDARY_DECODED_BYTES + 1));
        assert_eq!(
            decode_html_character_references_complete(&output_too_long),
            HtmlReferenceDecodingOutcome::OutputTooLong
        );

        assert_eq!(
            decode_base64_complete(&"A".repeat(MAX_SECONDARY_ENCODING_INPUT_BYTES + 1)),
            Base64DecodingOutcome::InputTooLong
        );
        assert_eq!(
            decode_base64_complete(&"A".repeat(12 * 1024)),
            Base64DecodingOutcome::OutputTooLong
        );
    }
}
