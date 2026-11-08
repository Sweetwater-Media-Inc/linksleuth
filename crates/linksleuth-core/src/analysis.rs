use crate::decoder::{DecodeError, DecodingResult, decode_percent};
use crate::destinations::{EmbeddedDestination, discover_destination};
use crate::parser::{ParsedUrl, UrlError, parse_url};

#[derive(Debug)]
pub struct ParameterAnalysis {
    pub name: String,
    pub decoding: Result<DecodingResult, DecodeError>,
    pub destination: Option<EmbeddedDestination>,
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
        .map(|parameter| {
            let decoding = decode_percent(&parameter.raw_value);

            let destination = match &decoding {
                Ok(result) => {
                    discover_destination(&result.decoded, &parameter.name, result.steps.len())
                }
                Err(_) => None,
            };

            ParameterAnalysis {
                name: parameter.name.clone(),
                decoding,
                destination,
            }
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
    fn preserves_decoding_warnings() {
        let report = analyze_url("https://example.com/?broken=%ZZ&valid=hello%20world").unwrap();

        assert_eq!(report.parameters.len(), 2);

        let broken = report.parameters[0].decoding.as_ref().unwrap();

        assert_eq!(broken.decoded, "%ZZ");
        assert_eq!(broken.warnings.len(), 1);

        let valid = report.parameters[1].decoding.as_ref().unwrap();

        assert_eq!(valid.decoded, "hello world");
        assert!(valid.warnings.is_empty());
    }

    #[test]
    fn discovers_embedded_destination() {
        let report =
            analyze_url("https://example.com/?target=https%253A%252F%252Fother.example").unwrap();

        let destination = report.parameters[0].destination.as_ref().unwrap();

        assert_eq!(destination.hostname, "other.example");
        assert_eq!(destination.decoding_passes, 2);
    }
}
