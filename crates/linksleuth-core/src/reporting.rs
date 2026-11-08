#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareableReport {
    pub hostname: String,
    pub query_parameter_count: usize,
    pub normalization_finding_count: usize,
    pub embedded_destination_count: usize,
}

use crate::analysis::AnalysisReport;

pub fn create_shareable_report(report: &AnalysisReport) -> ShareableReport {
    ShareableReport {
        hostname: "[REDACTED]".to_string(),
        query_parameter_count: report.parameters.len(),
        normalization_finding_count: report.normalization_findings.len(),
        embedded_destination_count: report
            .parameters
            .iter()
            .filter(|parameter| parameter.destination.is_some())
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::analyze_url;

    #[test]
    fn shareable_report_excludes_sensitive_url_values() {
        let report = analyze_url(
            "https://example.com/private/document/123?token=secret123&email=person@example.com",
        )
        .unwrap();

        let shared = create_shareable_report(&report);

        assert_eq!(shared.hostname, "[REDACTED]");
        assert_eq!(shared.query_parameter_count, 2);
        assert_eq!(shared.normalization_finding_count, 0);
        assert_eq!(shared.embedded_destination_count, 0);
    }

    #[test]
    fn shareable_report_preserves_original_analysis() {
        let report =
            analyze_url("https://example.com/private/document/123?token=secret123").unwrap();

        let original_url = report.url.original.clone();
        let original_raw_value = report.url.query_parameters[0].raw_value.clone();
        let original_normalized_url = report.url.normalized.clone();

        let _shared = create_shareable_report(&report);

        assert_eq!(report.url.original, original_url);
        assert_eq!(report.url.normalized, original_normalized_url);
        assert_eq!(report.url.query_parameters[0].raw_value, original_raw_value);
    }

    #[test]
    fn shareable_report_debug_output_excludes_sensitive_values() {
        let report = analyze_url(
            "https://example.com/private/document/123?token=secret123&email=person@example.com",
        )
        .unwrap();

        let shared = create_shareable_report(&report);

        let output = format!("{shared:?}");

        assert!(!output.contains("secret123"));
        assert!(!output.contains("person@example.com"));
        assert!(!output.contains("/private/document/123"));
        assert!(!output.contains("token="));

        assert!(!output.contains("example.com"));
        assert!(output.contains("[REDACTED]"));
        assert!(output.contains("query_parameter_count: 2"));
    }
}
