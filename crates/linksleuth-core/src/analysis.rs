use crate::decoder::{DecodeError, DecodingResult, decode_percent};
use crate::destinations::{EmbeddedDestination, discover_destination};
use crate::normalization::{NormalizationFinding, analyze_normalization};
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
    pub normalization_findings: Vec<NormalizationFinding>,
}

pub fn analyze_url(input: &str) -> Result<AnalysisReport, UrlError> {
    let url = parse_url(input)?;
    let normalization_findings = analyze_normalization(&url);

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

    Ok(AnalysisReport {
        url,
        parameters,
        normalization_findings,
    })
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

    #[test]
    fn includes_normalization_findings_in_report() {
        let report = analyze_url(
            "https://example.com/public/%2e%2e/admin?target=https%3A%2F%2Fother.example",
        )
        .unwrap();

        assert_eq!(report.normalization_findings.len(), 1);

        let finding = &report.normalization_findings[0];

        assert_eq!(
            finding.kind,
            crate::normalization::NormalizationKind::EncodedDotSegment
        );
        assert_eq!(finding.original_path, "/public/%2e%2e/admin");
        assert_eq!(finding.normalized_path, "/admin");

        // Normalization analysis must not interfere with query analysis.
        assert_eq!(report.parameters.len(), 1);
        assert_eq!(
            report.parameters[0].destination.as_ref().unwrap().hostname,
            "other.example"
        );
    }

    #[test]
    fn returns_empty_normalization_findings_for_ordinary_url() {
        let report = analyze_url("https://example.com/products?category=books").unwrap();

        assert!(report.normalization_findings.is_empty());
        assert_eq!(report.parameters.len(), 1);
    }
}
