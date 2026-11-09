use std::fmt;

use crate::analysis::AnalysisReport;
use crate::decoder::DecodingStatus;
use crate::destinations::{
    DestinationDiscoveryStatus, DestinationEncodingCoverage, DestinationReferenceKind,
    DestinationTransformation,
};
use crate::normalization::{NormalizationKind, OriginalPathAnalysis};

#[derive(Clone, PartialEq, Eq)]
pub struct ShareableReport {
    pub hostname: String,
    pub query_parameter_count: usize,
    pub original_path_analysis: OriginalPathAnalysis,
    pub normalization_finding_count: usize,
    pub embedded_destination_count: usize,
    pub nested_destination_count: usize,
    pub protocol_relative_destination_count: usize,
    pub base64_destination_count: usize,
    pub html_character_reference_destination_count: usize,
    pub destination_encoding_issue_count: usize,
    pub destination_discovery_status: DestinationDiscoveryStatus,
    pub destination_encoding_coverage: DestinationEncodingCoverage,
    pub findings: Vec<ShareableFinding>,
}

// Keep accidental diagnostic logging privacy-reduced, including when a caller
// explicitly requested the hostname for presentation.
impl fmt::Debug for ShareableReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ShareableReport")
            .field("hostname", &"[REDACTED]")
            .field("query_parameter_count", &self.query_parameter_count)
            .field("original_path_analysis", &self.original_path_analysis)
            .field(
                "normalization_finding_count",
                &self.normalization_finding_count,
            )
            .field(
                "embedded_destination_count",
                &self.embedded_destination_count,
            )
            .field("nested_destination_count", &self.nested_destination_count)
            .field(
                "protocol_relative_destination_count",
                &self.protocol_relative_destination_count,
            )
            .field("base64_destination_count", &self.base64_destination_count)
            .field(
                "html_character_reference_destination_count",
                &self.html_character_reference_destination_count,
            )
            .field(
                "destination_encoding_issue_count",
                &self.destination_encoding_issue_count,
            )
            .field(
                "destination_discovery_status",
                &self.destination_discovery_status,
            )
            .field(
                "destination_encoding_coverage",
                &self.destination_encoding_coverage,
            )
            .field("findings", &self.findings)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareableReportOptions {
    pub include_hostname: bool,
}

impl ShareableReportOptions {
    pub const fn new() -> Self {
        Self {
            include_hostname: false,
        }
    }

    pub const fn with_hostname(mut self) -> Self {
        self.include_hostname = true;
        self
    }
}

impl Default for ShareableReportOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareableFindingKind {
    EncodedParentDirectorySegment,
    PotentialEmbeddedDestination,
    PotentialProtocolRelativeReference,
    PotentialBase64EncodedDestination,
    PotentialHtmlCharacterReferenceEncodedDestination,
    MalformedPercentEncoding,
    DecodingPassLimitReached,
    DestinationDiscoveryIncomplete,
    DestinationEncodingAnalysisIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareableFinding {
    pub kind: ShareableFindingKind,
}

impl ShareableFinding {
    pub const fn new(kind: ShareableFindingKind) -> Self {
        Self { kind }
    }

    pub const fn title(self) -> &'static str {
        match self.kind {
            ShareableFindingKind::EncodedParentDirectorySegment => {
                "Encoded parent-directory segment"
            }
            ShareableFindingKind::PotentialEmbeddedDestination => "Potential embedded destination",
            ShareableFindingKind::PotentialProtocolRelativeReference => {
                "Potential protocol-relative reference"
            }
            ShareableFindingKind::PotentialBase64EncodedDestination => {
                "Potential Base64-encoded destination"
            }
            ShareableFindingKind::PotentialHtmlCharacterReferenceEncodedDestination => {
                "Potential HTML-reference-encoded destination"
            }
            ShareableFindingKind::MalformedPercentEncoding => "Malformed percent encoding",
            ShareableFindingKind::DecodingPassLimitReached => "Decoding pass limit reached",
            ShareableFindingKind::DestinationDiscoveryIncomplete => {
                "Destination discovery incomplete"
            }
            ShareableFindingKind::DestinationEncodingAnalysisIncomplete => {
                "Destination encoding analysis incomplete"
            }
        }
    }

    pub const fn explanation_lines(self) -> &'static [&'static str] {
        match self.kind {
            ShareableFindingKind::EncodedParentDirectorySegment => &[
                "The submitted URL contained encoded path components",
                "that were normalized during parsing.",
            ],
            ShareableFindingKind::PotentialEmbeddedDestination => &[
                "A complete HTTP/HTTPS URL was identified within",
                "a query parameter. No redirect has been verified.",
            ],
            ShareableFindingKind::PotentialProtocolRelativeReference => &[
                "A protocol-relative URL reference was identified within",
                "a query parameter. No scheme or redirect has been verified.",
            ],
            ShareableFindingKind::PotentialBase64EncodedDestination => &[
                "A complete destination reference was identified after bounded",
                "Base64 decoding. Encoding does not establish intent or navigation.",
            ],
            ShareableFindingKind::PotentialHtmlCharacterReferenceEncodedDestination => &[
                "A complete destination reference was identified after interpreting",
                "supported HTML references. No navigation has been verified.",
            ],
            ShareableFindingKind::MalformedPercentEncoding => &[
                "One or more query parameters contained malformed percent",
                "encoding. The original text was preserved during analysis.",
            ],
            ShareableFindingKind::DecodingPassLimitReached => &[
                "One or more query parameters reached the decoding pass limit.",
                "Additional encoded sequences may remain.",
            ],
            ShareableFindingKind::DestinationDiscoveryIncomplete => &[
                "One or more destination discovery limits were reached.",
                "Additional potential destinations may remain undiscovered.",
            ],
            ShareableFindingKind::DestinationEncodingAnalysisIncomplete => &[
                "One or more supported encoding interpretations were malformed",
                "or unsupported. Additional destination evidence may remain.",
            ],
        }
    }
}

/// Creates a privacy-reduced report using safe defaults.
///
/// This function retains its original signature and redacts the hostname.
pub fn create_shareable_report(report: &AnalysisReport) -> ShareableReport {
    create_shareable_report_with_options(report, ShareableReportOptions::default())
}

/// Creates a privacy-reduced report with explicit disclosure options.
///
/// URL paths, query names and values, fragments, decoded values, and embedded
/// destination evidence are never copied into the returned report. The source
/// hostname is included only when `include_hostname` is explicitly enabled.
pub fn create_shareable_report_with_options(
    report: &AnalysisReport,
    options: ShareableReportOptions,
) -> ShareableReport {
    let embedded_destination_count = report.destination_discovery.candidates.len();
    let nested_destination_count = report
        .destination_discovery
        .candidates
        .iter()
        .filter(|candidate| candidate.provenance.depth > 0)
        .count();
    let protocol_relative_destination_count = report
        .destination_discovery
        .candidates
        .iter()
        .filter(|candidate| candidate.reference.kind == DestinationReferenceKind::ProtocolRelative)
        .count();
    let base64_destination_count = report
        .destination_discovery
        .candidates
        .iter()
        .filter(|candidate| {
            candidate
                .provenance
                .transformations
                .iter()
                .any(|transformation| {
                    matches!(
                        transformation,
                        DestinationTransformation::Base64Decoding { .. }
                    )
                })
        })
        .count();
    let html_character_reference_destination_count = report
        .destination_discovery
        .candidates
        .iter()
        .filter(|candidate| {
            candidate
                .provenance
                .transformations
                .iter()
                .any(|transformation| {
                    matches!(
                        transformation,
                        DestinationTransformation::HtmlCharacterReferenceDecoding { .. }
                    )
                })
        })
        .count();
    let has_malformed_percent_encoding = report.parameters.iter().any(|parameter| {
        parameter
            .decoding
            .as_ref()
            .is_ok_and(|decoding| !decoding.warnings.is_empty())
    });
    let has_decoding_pass_limit = report.parameters.iter().any(|parameter| {
        parameter
            .decoding
            .as_ref()
            .is_ok_and(|decoding| decoding.status == DecodingStatus::PassLimitReached)
    });
    let mut findings = Vec::new();

    if report
        .normalization_findings
        .iter()
        .any(|finding| finding.kind == NormalizationKind::EncodedDotSegment)
    {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::EncodedParentDirectorySegment,
        ));
    }

    if embedded_destination_count > protocol_relative_destination_count {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::PotentialEmbeddedDestination,
        ));
    }

    if protocol_relative_destination_count > 0 {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::PotentialProtocolRelativeReference,
        ));
    }

    if base64_destination_count > 0 {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::PotentialBase64EncodedDestination,
        ));
    }

    if html_character_reference_destination_count > 0 {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::PotentialHtmlCharacterReferenceEncodedDestination,
        ));
    }

    if has_malformed_percent_encoding {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::MalformedPercentEncoding,
        ));
    }

    if has_decoding_pass_limit {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::DecodingPassLimitReached,
        ));
    }

    if !report.destination_discovery.status.is_complete() {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::DestinationDiscoveryIncomplete,
        ));
    }

    if !report.destination_discovery.encoding_coverage.is_complete() {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::DestinationEncodingAnalysisIncomplete,
        ));
    }

    ShareableReport {
        hostname: if options.include_hostname {
            report.url.hostname.clone()
        } else {
            "[REDACTED]".to_string()
        },
        query_parameter_count: report.parameters.len(),
        original_path_analysis: report.original_path_analysis,
        normalization_finding_count: report.normalization_findings.len(),
        embedded_destination_count,
        nested_destination_count,
        protocol_relative_destination_count,
        base64_destination_count,
        html_character_reference_destination_count,
        destination_encoding_issue_count: report.destination_discovery.encoding_issues.len(),
        destination_discovery_status: report.destination_discovery.status.clone(),
        destination_encoding_coverage: report.destination_discovery.encoding_coverage.clone(),
        findings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::analyze_url;

    #[test]
    fn shareable_options_are_private_by_default() {
        let options = ShareableReportOptions::default();

        assert!(!options.include_hostname);
    }

    #[test]
    fn backward_compatible_report_function_uses_safe_defaults() {
        let report = analyze_url(
            "https://private.example.com/private/document/123?token=secret123&email=person@example.com",
        )
        .unwrap();

        let shared = create_shareable_report(&report);

        assert_eq!(shared.hostname, "[REDACTED]");
        assert_eq!(shared.query_parameter_count, 2);
        assert_eq!(
            shared.original_path_analysis,
            OriginalPathAnalysis::Examined
        );
        assert_eq!(shared.normalization_finding_count, 0);
        assert_eq!(shared.embedded_destination_count, 0);
        assert_eq!(shared.nested_destination_count, 0);
        assert_eq!(shared.protocol_relative_destination_count, 0);
        assert_eq!(shared.base64_destination_count, 0);
        assert_eq!(shared.html_character_reference_destination_count, 0);
        assert_eq!(shared.destination_encoding_issue_count, 0);
        assert!(shared.destination_discovery_status.is_complete());
        assert!(shared.destination_encoding_coverage.is_complete());
        assert!(shared.findings.is_empty());
    }

    #[test]
    fn options_aware_report_includes_only_explicit_hostname_evidence() {
        let report = analyze_url(
            "https://private.example.com/private/document/123?token=secret123#private-fragment",
        )
        .unwrap();

        let shared = create_shareable_report_with_options(
            &report,
            ShareableReportOptions::new().with_hostname(),
        );

        assert_eq!(shared.hostname, "private.example.com");
        assert_eq!(shared.query_parameter_count, 1);
        assert!(shared.findings.is_empty());

        let debug_output = format!("{shared:?}");
        assert!(!debug_output.contains("private.example.com"));
        assert!(!debug_output.contains("/private/document/123"));
        assert!(!debug_output.contains("secret123"));
        assert!(!debug_output.contains("private-fragment"));
        assert!(debug_output.contains("[REDACTED]"));
    }

    #[test]
    fn shareable_report_creates_all_supported_finding_kinds() {
        let report = analyze_url(
            "https://private.example.com/public/%2e%2e/admin?target=https%3A%2F%2Fother.example%2Fprivate&broken=secret%ZZvalue&deep=%2525252541",
        )
        .unwrap();

        let shared = create_shareable_report(&report);

        assert_eq!(
            shared.findings,
            vec![
                ShareableFinding::new(ShareableFindingKind::EncodedParentDirectorySegment),
                ShareableFinding::new(ShareableFindingKind::PotentialEmbeddedDestination),
                ShareableFinding::new(ShareableFindingKind::MalformedPercentEncoding),
                ShareableFinding::new(ShareableFindingKind::DecodingPassLimitReached),
                ShareableFinding::new(ShareableFindingKind::DestinationDiscoveryIncomplete),
            ]
        );
        assert_eq!(
            shared.findings[0].title(),
            "Encoded parent-directory segment"
        );
        assert_eq!(
            shared.findings[2].explanation_lines(),
            &[
                "One or more query parameters contained malformed percent",
                "encoding. The original text was preserved during analysis.",
            ]
        );
    }

    #[test]
    fn shareable_report_has_no_findings_for_an_ordinary_url() {
        let report = analyze_url("https://example.com/products?category=books").unwrap();

        let shared = create_shareable_report(&report);

        assert!(shared.findings.is_empty());
        assert_eq!(
            shared.original_path_analysis,
            OriginalPathAnalysis::Examined
        );
        assert_eq!(shared.normalization_finding_count, 0);
        assert_eq!(shared.embedded_destination_count, 0);
    }

    #[test]
    fn shareable_report_counts_multiple_destinations_but_uses_one_fixed_finding() {
        let report = analyze_url(
            "https://example.com/?first=https%3A%2F%2Fone.example&second=https%3A%2F%2Ftwo.example",
        )
        .unwrap();

        let shared = create_shareable_report(&report);

        assert_eq!(shared.embedded_destination_count, 2);
        assert_eq!(
            shared.findings,
            vec![ShareableFinding::new(
                ShareableFindingKind::PotentialEmbeddedDestination
            )]
        );
    }

    #[test]
    fn shareable_report_summarizes_nested_protocol_relative_discovery_without_evidence() {
        let report = analyze_url(
            "https://private.example.com/?target=https%3A%2F%2Fmiddle.example%2F%3Fnext%3D%252F%252Fnested.example%252Fprivate",
        )
        .unwrap();

        let shared = create_shareable_report_with_options(
            &report,
            ShareableReportOptions::new().with_hostname(),
        );

        assert_eq!(shared.hostname, "private.example.com");
        assert_eq!(shared.embedded_destination_count, 2);
        assert_eq!(shared.nested_destination_count, 1);
        assert_eq!(shared.protocol_relative_destination_count, 1);
        assert!(shared.destination_discovery_status.is_complete());
        assert_eq!(
            shared.findings,
            vec![
                ShareableFinding::new(ShareableFindingKind::PotentialEmbeddedDestination),
                ShareableFinding::new(ShareableFindingKind::PotentialProtocolRelativeReference),
            ]
        );

        let output = format!("{shared:?}");
        assert!(!output.contains("private.example.com"));
        assert!(!output.contains("middle.example"));
        assert!(!output.contains("nested.example"));
        assert!(!output.contains("/private"));
    }

    #[test]
    fn shareable_report_summarizes_encoded_candidates_without_evidence() {
        let html = url::form_urlencoded::byte_serialize(
            "https&colon;&sol;&sol;html-secret.example&sol;private".as_bytes(),
        )
        .collect::<String>();
        let input = format!(
            "https://private.example.com/?payload=aHR0cHM6Ly9iYXNlNjQtc2VjcmV0LmV4YW1wbGUvcHJpdmF0ZQ==&markup={html}"
        );
        let report = analyze_url(&input).unwrap();

        let shared = create_shareable_report_with_options(
            &report,
            ShareableReportOptions::new().with_hostname(),
        );

        assert_eq!(shared.hostname, "private.example.com");
        assert_eq!(shared.embedded_destination_count, 2);
        assert_eq!(shared.base64_destination_count, 1);
        assert_eq!(shared.html_character_reference_destination_count, 1);
        assert_eq!(shared.destination_encoding_issue_count, 0);
        assert!(shared.destination_encoding_coverage.is_complete());
        assert!(shared.findings.contains(&ShareableFinding::new(
            ShareableFindingKind::PotentialBase64EncodedDestination
        )));
        assert!(shared.findings.contains(&ShareableFinding::new(
            ShareableFindingKind::PotentialHtmlCharacterReferenceEncodedDestination
        )));

        let fields = format!(
            "{}:{}:{}:{:?}:{:?}",
            shared.base64_destination_count,
            shared.html_character_reference_destination_count,
            shared.destination_encoding_issue_count,
            shared.destination_encoding_coverage,
            shared.findings,
        );
        let debug = format!("{shared:?}");
        for output in [&fields, &debug] {
            assert!(!output.contains("base64-secret.example"));
            assert!(!output.contains("html-secret.example"));
            assert!(!output.contains("private"));
            assert!(!output.contains("payload"));
            assert!(!output.contains("markup"));
            assert!(!output.contains("aHR0cHM6"));
        }
    }

    #[test]
    fn shareable_report_exposes_only_fixed_encoding_coverage_metadata() {
        let html = url::form_urlencoded::byte_serialize(
            "https&colon;&sol;&sol;hidden.example&unsupported;".as_bytes(),
        )
        .collect::<String>();
        let report = analyze_url(&format!("https://private.example.com/?secret={html}")).unwrap();

        let shared = create_shareable_report(&report);

        assert_eq!(shared.destination_encoding_issue_count, 1);
        assert!(!shared.destination_encoding_coverage.is_complete());
        assert!(shared.findings.contains(&ShareableFinding::new(
            ShareableFindingKind::DestinationEncodingAnalysisIncomplete
        )));
        let output = format!("{shared:?}");
        assert!(!output.contains("private.example.com"));
        assert!(!output.contains("hidden.example"));
        assert!(!output.contains("unsupported"));
        assert!(!output.contains("secret"));
    }

    #[test]
    fn shareable_report_preserves_original_analysis() {
        let report =
            analyze_url("https://example.com/private/document/123?token=secret123").unwrap();

        let original_url = report.url.original.clone();
        let original_raw_value = report.url.query_parameters[0].raw_value.clone();
        let original_normalized_url = report.url.normalized.clone();

        let _shared = create_shareable_report_with_options(
            &report,
            ShareableReportOptions::new().with_hostname(),
        );

        assert_eq!(report.url.original, original_url);
        assert_eq!(report.url.normalized, original_normalized_url);
        assert_eq!(report.url.query_parameters[0].raw_value, original_raw_value);
    }

    #[test]
    fn shareable_report_preserves_unsupported_path_coverage_without_raw_evidence() {
        let report =
            analyze_url("https:////private.example.com/public/%2e%2e/admin?token=secret123")
                .unwrap();

        let shared = create_shareable_report(&report);

        assert!(matches!(
            shared.original_path_analysis,
            OriginalPathAnalysis::Unsupported { .. }
        ));
        assert_eq!(shared.normalization_finding_count, 0);
        assert!(shared.findings.is_empty());

        let output = format!("{shared:?}");
        assert!(!output.contains("private.example.com"));
        assert!(!output.contains("/public/"));
        assert!(!output.contains("secret123"));
    }

    #[test]
    fn default_report_fields_and_debug_output_exclude_sensitive_url_evidence() {
        let report = analyze_url(
            "https://private.example.com/public/%2e%2e/admin?target=https%3A%2F%2Fother.example%2Fprivate&token=secret123&email=person@example.com",
        )
        .unwrap();

        let shared = create_shareable_report(&report);
        let field_output = format!(
            "{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}",
            shared.hostname,
            shared.query_parameter_count,
            shared.original_path_analysis,
            shared.normalization_finding_count,
            shared.embedded_destination_count,
            shared.nested_destination_count,
            shared.protocol_relative_destination_count,
            shared.base64_destination_count,
            shared.html_character_reference_destination_count,
            shared.destination_encoding_issue_count,
            shared.destination_discovery_status,
            shared.destination_encoding_coverage,
            shared.findings,
        );
        let debug_output = format!("{shared:?}");

        for output in [&field_output, &debug_output] {
            assert!(!output.contains("private.example.com"));
            assert!(!output.contains("other.example"));
            assert!(!output.contains("secret123"));
            assert!(!output.contains("person@example.com"));
            assert!(!output.contains("/public/"));
            assert!(!output.contains("/private"));
            assert!(!output.contains("token="));
        }

        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("EncodedParentDirectorySegment"));
        assert!(debug_output.contains("PotentialEmbeddedDestination"));
    }
}
