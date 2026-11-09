use crate::decoder::{DecodeError, DecodingResult, decode_percent};
use crate::destinations::{EmbeddedDestination, discover_destination};
use crate::normalization::{
    NormalizationFinding, OriginalPathAnalysis, analyze_normalization_with_coverage,
};
use crate::parser::{ParsedUrl, UrlError, parse_url};

#[derive(Debug)]
pub struct ParameterAnalysis {
    /// Form-decoded query parameter name supplied by [`ParsedUrl`].
    pub name: String,
    /// Bounded iterative percent decoding of the parameter's exact raw value.
    pub decoding: Result<DecodingResult, DecodeError>,
    /// Passive candidate discovered from the final iterative decoding output.
    pub destination: Option<EmbeddedDestination>,
}

#[derive(Debug)]
pub struct AnalysisReport {
    pub url: ParsedUrl,
    pub parameters: Vec<ParameterAnalysis>,
    /// Whether original path evidence was examined or could not be interpreted.
    pub original_path_analysis: OriginalPathAnalysis,
    /// Findings derived only from confidently extracted original path evidence.
    pub normalization_findings: Vec<NormalizationFinding>,
}

pub fn analyze_url(input: &str) -> Result<AnalysisReport, UrlError> {
    let url = parse_url(input)?;
    let normalization = analyze_normalization_with_coverage(&url);

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
        original_path_analysis: normalization.original_path_analysis,
        normalization_findings: normalization.findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::{DecodingStatus, DecodingWarning};
    use crate::normalization::{OriginalPathAnalysis, OriginalPathUnsupportedReason};

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
        assert_eq!(
            report.original_path_analysis,
            OriginalPathAnalysis::Examined
        );

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
        assert_eq!(
            report.original_path_analysis,
            OriginalPathAnalysis::Examined
        );
        assert_eq!(report.parameters.len(), 1);
    }

    #[test]
    fn distinguishes_unsupported_path_analysis_from_no_finding() {
        let report =
            analyze_url("https:////example.com/public/%2e%2e/admin?category=books").unwrap();

        assert_eq!(
            report.original_path_analysis,
            OriginalPathAnalysis::Unsupported {
                reason: OriginalPathUnsupportedReason::AmbiguousAuthorityBoundary,
            }
        );
        assert!(report.normalization_findings.is_empty());
        assert_eq!(report.parameters.len(), 1);
    }

    #[test]
    fn keeps_raw_form_and_iteratively_decoded_query_evidence_distinct() {
        let report =
            analyze_url("https://example.com/路径?value=雪+storm&nested=%25E9%259B%25AA%252Bstorm")
                .unwrap();

        let raw_parameters = &report.url.query_parameters;
        assert_eq!(raw_parameters[0].raw_value, "雪+storm");
        assert_eq!(raw_parameters[0].value, "雪 storm");
        assert_eq!(
            report.parameters[0].decoding.as_ref().unwrap().decoded,
            "雪+storm"
        );

        assert_eq!(raw_parameters[1].raw_value, "%25E9%259B%25AA%252Bstorm");
        assert_eq!(raw_parameters[1].value, "%E9%9B%AA%2Bstorm");
        let nested = report.parameters[1].decoding.as_ref().unwrap();
        assert_eq!(nested.decoded, "雪+storm");
        assert_eq!(nested.steps.len(), 2);
    }

    #[test]
    fn preserves_mixed_decoding_warning_provenance_through_analysis() {
        let report = analyze_url("https://example.com/?value=%E2%98%83%20snow%ZZ").unwrap();

        let result = report.parameters[0].decoding.as_ref().unwrap();
        assert_eq!(result.decoded, "☃ snow%ZZ");
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].input, "%E2%98%83%20snow%ZZ");
        assert_eq!(result.steps[0].output, "☃ snow%ZZ");
        assert_eq!(
            result.warnings,
            vec![
                DecodingWarning::MalformedPercentEncoding {
                    pass: 1,
                    offset: 16,
                },
                DecodingWarning::MalformedPercentEncoding { pass: 2, offset: 8 },
            ]
        );
    }
}
