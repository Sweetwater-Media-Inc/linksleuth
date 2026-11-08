#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingMethod {
    PercentEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingStep {
    pub method: DecodingMethod,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingResult {
    pub original: String,
    pub decoded: String,
    pub steps: Vec<DecodingStep>,
    pub status: DecodingStatus,
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
    MalformedPercentEncoding { pass: usize, offset: usize },
}

const MAX_INPUT_LENGTH: usize = 16 * 1024;
const MAX_DECODE_PASSES: usize = 3;

pub fn decode_percent(input: &str) -> Result<DecodingResult, DecodeError> {
    if input.len() > MAX_INPUT_LENGTH {
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
}
