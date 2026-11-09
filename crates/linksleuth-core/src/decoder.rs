#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingMethod {
    PercentEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingStep {
    pub method: DecodingMethod,
    /// Exact input to this decoding pass.
    pub input: String,
    /// Output produced by this decoding pass and input to the next pass.
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingResult {
    /// Exact string passed to [`decode_percent`].
    pub original: String,
    /// Final output after bounded iterative decoding.
    pub decoded: String,
    /// Ordered input/output provenance for passes that changed the text.
    pub steps: Vec<DecodingStep>,
    pub status: DecodingStatus,
    /// Malformed percent observations with one-based pass and byte offset.
    pub warnings: Vec<DecodingWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidUtf8,
    InputTooLong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingStatus {
    Complete,
    PassLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingWarning {
    /// A malformed percent sequence observed at a UTF-8 byte offset.
    MalformedPercentEncoding { pass: usize, offset: usize },
}

/// Maximum decoder input size, measured in UTF-8 bytes.
pub const MAX_DECODER_INPUT_BYTES: usize = 16 * 1024;

/// Maximum number of iterative percent-decoding passes.
pub const MAX_DECODE_PASSES: usize = 3;

/// Iteratively percent-decodes exact raw query value text.
///
/// This is deliberately distinct from HTML form decoding: `+` remains `+`
/// here, while the parser's form-decoded query representation treats it as a
/// space. Each successful transformation is retained in `steps`. Work is
/// bounded by [`MAX_DECODER_INPUT_BYTES`] and [`MAX_DECODE_PASSES`], and
/// percent decoding cannot expand the input byte length.
pub fn decode_percent(input: &str) -> Result<DecodingResult, DecodeError> {
    if input.len() > MAX_DECODER_INPUT_BYTES {
        return Err(DecodeError::InputTooLong);
    }

    let mut current = input.to_string();
    let mut steps = Vec::new();
    let mut warnings = Vec::new();

    for pass in 1..=MAX_DECODE_PASSES {
        let (decoded, pass_warnings) = decode_once(&current, pass)?;

        warnings.extend(pass_warnings);

        if decoded == current {
            break;
        }

        steps.push(DecodingStep {
            method: DecodingMethod::PercentEncoding,
            input: current,
            output: decoded.clone(),
        });

        current = decoded;
    }

    let status = if contains_percent_encoding(&current) {
        DecodingStatus::PassLimitReached
    } else {
        DecodingStatus::Complete
    };

    Ok(DecodingResult {
        original: input.to_string(),
        decoded: current,
        steps,
        status,
        warnings,
    })
}

fn decode_once(input: &str, pass: usize) -> Result<(String, Vec<DecodingWarning>), DecodeError> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut warnings = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 < bytes.len()
                && let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }

            warnings.push(DecodingWarning::MalformedPercentEncoding {
                pass,
                offset: index,
            });

            output.push(b'%');
            index += 1;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    let decoded = String::from_utf8(output).map_err(|_| DecodeError::InvalidUtf8)?;

    Ok((decoded, warnings))
}

fn contains_percent_encoding(input: &str) -> bool {
    input.as_bytes().windows(3).any(|window| {
        window[0] == b'%' && hex_value(window[1]).is_some() && hex_value(window[2]).is_some()
    })
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_single_layer() {
        let result = decode_percent("https%3A%2F%2Fexample.com").unwrap();

        assert_eq!(result.decoded, "https://example.com");
        assert_eq!(result.steps.len(), 1);
    }

    #[test]
    fn decodes_multiple_layers() {
        let result = decode_percent("https%253A%252F%252Fexample.com").unwrap();

        assert_eq!(result.decoded, "https://example.com");
        assert_eq!(result.steps.len(), 2);

        assert_eq!(result.steps[0].output, "https%3A%2F%2Fexample.com");
    }

    #[test]
    fn preserves_plain_text() {
        let result = decode_percent("hello world").unwrap();

        assert_eq!(result.decoded, "hello world");
        assert!(result.steps.is_empty());
    }

    #[test]
    fn reports_invalid_encoding() {
        let result = decode_percent("hello%ZZ").unwrap();

        assert_eq!(
            result.warnings,
            vec![DecodingWarning::MalformedPercentEncoding { pass: 1, offset: 5 }]
        );
    }

    #[test]
    fn rejects_invalid_utf8() {
        let result = decode_percent("%FF");

        assert_eq!(result, Err(DecodeError::InvalidUtf8));
    }

    #[test]
    fn decodes_utf8_characters() {
        let result = decode_percent("caf%C3%A9").unwrap();

        assert_eq!(result.decoded, "café");
    }

    #[test]
    fn decodes_multibyte_utf8_with_mixed_hex_case() {
        let result = decode_percent("%e9%9B%aa-%E2%98%83-%4a%4A").unwrap();

        assert_eq!(result.decoded, "雪-☃-JJ");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].input, "%e9%9B%aa-%E2%98%83-%4a%4A");
        assert_eq!(result.steps[0].output, "雪-☃-JJ");
    }

    #[test]
    fn decodes_encoded_query_delimiters_without_form_decoding_plus() {
        let result = decode_percent("a%26b%3Dc%3Fnext+value").unwrap();

        assert_eq!(result.decoded, "a&b=c?next+value");
    }

    #[test]
    fn respects_pass_limit() {
        let result = decode_percent("%2525252541").unwrap();

        assert_eq!(result.steps.len(), 3);
        assert_eq!(result.decoded, "%2541");
    }

    #[test]
    fn rejects_oversized_input() {
        let input = "a".repeat(20_000);

        assert_eq!(decode_percent(&input), Err(DecodeError::InputTooLong));
    }

    #[test]
    fn accepts_decoder_input_at_byte_limit_and_rejects_next_byte() {
        let at_limit = "a".repeat(MAX_DECODER_INPUT_BYTES);
        let over_limit = format!("{at_limit}a");

        assert!(decode_percent(&at_limit).is_ok());
        assert_eq!(decode_percent(&over_limit), Err(DecodeError::InputTooLong));
    }

    #[test]
    fn decoder_limit_counts_unicode_bytes() {
        let input = "雪".repeat((MAX_DECODER_INPUT_BYTES / 3) + 1);

        assert!(input.chars().count() < MAX_DECODER_INPUT_BYTES);
        assert!(input.len() > MAX_DECODER_INPUT_BYTES);
        assert_eq!(decode_percent(&input), Err(DecodeError::InputTooLong));
    }

    #[test]
    fn reports_complete_decoding() {
        let result = decode_percent("https%3A%2F%2Fexample.com").unwrap();

        assert_eq!(result.status, DecodingStatus::Complete);
    }

    #[test]
    fn reports_pass_limit_reached() {
        let result = decode_percent("%2525252541").unwrap();

        assert_eq!(result.status, DecodingStatus::PassLimitReached);
    }

    #[test]
    fn preserves_literal_percent_after_decoding() {
        let result = decode_percent("50%25").unwrap();

        assert_eq!(result.decoded, "50%");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.status, DecodingStatus::Complete);
    }

    #[test]
    fn decodes_valid_sequences_alongside_literal_percent() {
        let result = decode_percent("Save%20%25%20today").unwrap();

        assert_eq!(result.decoded, "Save % today");
        assert_eq!(result.steps.len(), 1);
    }

    #[test]
    fn preserves_malformed_percent_text_without_discarding_evidence() {
        let result = decode_percent("hello%ZZworld").unwrap();

        assert_eq!(result.decoded, "hello%ZZworld");
        assert!(result.steps.is_empty());
    }

    #[test]
    fn decodes_valid_sequences_despite_malformed_encoding() {
        let result = decode_percent("hello%ZZ%20world").unwrap();

        assert_eq!(result.decoded, "hello%ZZ world");
        assert_eq!(result.steps.len(), 1);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn identifies_warning_from_later_decoding_pass() {
        let result = decode_percent("50%25").unwrap();

        assert_eq!(result.decoded, "50%");

        assert_eq!(
            result.warnings,
            vec![DecodingWarning::MalformedPercentEncoding { pass: 2, offset: 2 }]
        );
    }

    #[test]
    fn warning_offsets_count_utf8_bytes() {
        let result = decode_percent("雪%ZZ").unwrap();

        assert_eq!(
            result.warnings,
            vec![DecodingWarning::MalformedPercentEncoding { pass: 1, offset: 3 }]
        );
    }

    #[test]
    fn decoding_steps_do_not_expand_byte_length() {
        let result = decode_percent("%2525252541").unwrap();

        assert_eq!(result.steps.len(), MAX_DECODE_PASSES);
        assert!(
            result
                .steps
                .iter()
                .all(|step| step.output.len() <= step.input.len())
        );
        assert_eq!(result.status, DecodingStatus::PassLimitReached);
    }
}
