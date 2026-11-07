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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidPercentEncoding,
    InvalidUtf8,
    InputTooLong,
}

const MAX_INPUT_LENGTH: usize = 16 * 1024;
const MAX_DECODE_PASSES: usize = 3;

pub fn decode_percent(input: &str) -> Result<DecodingResult, DecodeError> {
    if input.len() > MAX_INPUT_LENGTH {
        return Err(DecodeError::InputTooLong);
    }

    let mut current = input.to_string();
    let mut steps = Vec::new();

    for _ in 0..MAX_DECODE_PASSES {
        let decoded = decode_once(&current)?;

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

    Ok(DecodingResult {
        original: input.to_string(),
        decoded: current,
        steps,
    })
}

fn decode_once(input: &str) -> Result<String, DecodeError> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(DecodeError::InvalidPercentEncoding);
            }

            let high = hex_value(bytes[index + 1]).ok_or(DecodeError::InvalidPercentEncoding)?;

            let low = hex_value(bytes[index + 2]).ok_or(DecodeError::InvalidPercentEncoding)?;

            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(output).map_err(|_| DecodeError::InvalidUtf8)
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
    fn rejects_invalid_encoding() {
        let result = decode_percent("hello%ZZ");

        assert_eq!(result, Err(DecodeError::InvalidPercentEncoding));
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
}
