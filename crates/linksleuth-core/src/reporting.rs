use std::fmt;

use crate::analysis::AnalysisReport;
use crate::decoder::DecodingStatus;
use crate::normalization::NormalizationKind;

#[derive(Clone, PartialEq, Eq)]
pub struct ShareableReport {
    pub hostname: String,
    pub query_parameter_count: usize,
    pub normalization_finding_count: usize,
    pub embedded_destination_count: usize,
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
            .field(
                "normalization_finding_count",
                &self.normalization_finding_count,
            )
            .field(
                "embedded_destination_count",
                &self.embedded_destination_count,
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
    MalformedPercentEncoding,
    DecodingPassLimitReached,
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
            ShareableFindingKind::MalformedPercentEncoding => "Malformed percent encoding",
            ShareableFindingKind::DecodingPassLimitReached => "Decoding pass limit reached",
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
            ShareableFindingKind::MalformedPercentEncoding => &[
                "One or more query parameters contained malformed percent",
                "encoding. The original text was preserved during analysis.",
            ],
            ShareableFindingKind::DecodingPassLimitReached => &[
                "One or more query parameters reached the decoding pass limit.",
                "Additional encoded sequences may remain.",
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
    let embedded_destination_count = report
        .parameters
        .iter()
        .filter(|parameter| parameter.destination.is_some())
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

    if embedded_destination_count > 0 {
        findings.push(ShareableFinding::new(
            ShareableFindingKind::PotentialEmbeddedDestination,
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

    ShareableReport {
        hostname: if options.include_hostname {
            report.url.hostname.clone()
        } else {
            "[REDACTED]".to_string()
        },
        query_parameter_count: report.parameters.len(),
        normalization_finding_count: report.normalization_findings.len(),
        embedded_destination_count,
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
        assert_eq!(shared.normalization_finding_count, 0);
        assert_eq!(shared.embedded_destination_count, 0);
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
    fn default_report_fields_and_debug_output_exclude_sensitive_url_evidence() {
        let report = analyze_url(
            "https://private.example.com/public/%2e%2e/admin?target=https%3A%2F%2Fother.example%2Fprivate&token=secret123&email=person@example.com",
        )
        .unwrap();

        let shared = create_shareable_report(&report);
        let field_output = format!(
            "{}:{}:{}:{}:{:?}",
            shared.hostname,
            shared.query_parameter_count,
            shared.normalization_finding_count,
            shared.embedded_destination_count,
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
