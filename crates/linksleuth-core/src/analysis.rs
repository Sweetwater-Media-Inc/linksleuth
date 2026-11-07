use crate::decoder::{DecodeError, DecodingResult, decode_percent};
use crate::parser::{ParsedUrl, UrlError, parse_url};

#[derive(Debug)]
pub struct ParameterAnalysis {
    pub name: String,
    pub decoding: Result<DecodingResult, DecodeError>,
}

#[derive(Debug)]
pub struct AnalysisReport {
    pub url: ParsedUrl,
    pub parameters: Vec<ParameterAnalysis>,
}

pub fn analyze_url(input: &str) -> Result<AnalysisReport, UrlError> {
    let url = parse_url(input)?;

    let parameters = url
        .query_parameters
        .iter()
        .map(|parameter| ParameterAnalysis {
            name: parameter.name.clone(),
            decoding: decode_percent(&parameter.raw_value),
        })
        .collect();

    Ok(AnalysisReport { url, parameters })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::DecodingStatus;

    #[test]
    fn analyzes_nested_encoding() {
        let report =
            analyze_url("https://example.com/?target=https%253A%252F%252Fother.example").unwrap();

        let result = report.parameters[0].decoding.as_ref().unwrap();

        assert_eq!(result.decoded, "https://other.example");
        assert_eq!(result.steps.len(), 2);
        assert_eq!(result.status, DecodingStatus::Complete);
    }

    #[test]
    fn preserves_decoding_errors() {
        let report = analyze_url("https://example.com/?broken=%ZZ&valid=hello%20world").unwrap();

        assert_eq!(report.parameters.len(), 2);
        assert!(report.parameters[0].decoding.is_err());
        assert!(report.parameters[1].decoding.is_ok());
    }
}
