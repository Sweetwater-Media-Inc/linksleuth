#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationKind {
    EncodedDotSegment,
}

impl NormalizationKind {
    pub fn title(&self) -> &'static str {
        match self {
            Self::EncodedDotSegment => "Encoded parent-directory segment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationFinding {
    pub kind: NormalizationKind,
    pub original_path: String,
    pub normalized_path: String,
    pub explanation: String,
}

use crate::parser::ParsedUrl;

/// Whether original path evidence was confidently examined.
///
/// `Examined` does not mean the URL was benign; it only means the original
/// path boundary could be interpreted for the supported normalization checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalPathAnalysis {
    Examined,
    Unsupported {
        reason: OriginalPathUnsupportedReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginalPathUnsupportedReason {
    /// The submitted scheme boundary was changed or not recognizable.
    UnrecognizedSchemeBoundary,
    /// Extra authority separators made the original path boundary ambiguous.
    AmbiguousAuthorityBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationAnalysis {
    /// Coverage metadata for the original path extraction attempt.
    pub original_path_analysis: OriginalPathAnalysis,
    /// Findings based only on confidently extracted original path evidence.
    pub findings: Vec<NormalizationFinding>,
}

/// Retains the original findings-only API for existing consumers.
///
/// Consumers that need to distinguish an examined path from unsupported
/// extraction should call [`analyze_normalization_with_coverage`].
pub fn analyze_normalization(parsed: &ParsedUrl) -> Vec<NormalizationFinding> {
    analyze_normalization_with_coverage(parsed).findings
}

/// Analyzes supported original path evidence and reports extraction coverage.
///
/// Unsupported extraction produces no normalization findings, but is distinct
/// from an examined path that produced no finding.
pub fn analyze_normalization_with_coverage(parsed: &ParsedUrl) -> NormalizationAnalysis {
    let mut findings = Vec::new();

    let original_path = match extract_original_path(parsed) {
        Ok(original_path) => original_path,
        Err(reason) => {
            return NormalizationAnalysis {
                original_path_analysis: OriginalPathAnalysis::Unsupported { reason },
                findings,
            };
        }
    };

    let contains_encoded_dot_segment = original_path.split(['/', '\\']).any(|segment| {
        let lower = segment.to_ascii_lowercase();

        matches!(lower.as_str(), "%2e%2e" | "%2e." | ".%2e")
    });

    if contains_encoded_dot_segment && original_path != parsed.path {
        findings.push(NormalizationFinding {
            kind: NormalizationKind::EncodedDotSegment,
            original_path: original_path.to_string(),
            normalized_path: parsed.path.clone(),
            explanation: String::from(
                "The submitted URL contains an encoded parent-directory \
                 segment. The URL parser normalized the path. \
                 This does not establish malicious intent.",
            ),
        });
    }

    NormalizationAnalysis {
        original_path_analysis: OriginalPathAnalysis::Examined,
        findings,
    }
}

fn extract_original_path(parsed: &ParsedUrl) -> Result<&str, OriginalPathUnsupportedReason> {
    let input = parsed.original.as_str();
    let scheme_length = parsed.scheme.len();
    let Some(submitted_scheme) = input.get(..scheme_length) else {
        return Err(OriginalPathUnsupportedReason::UnrecognizedSchemeBoundary);
    };

    if !submitted_scheme.eq_ignore_ascii_case(&parsed.scheme)
        || input.as_bytes().get(scheme_length) != Some(&b':')
    {
        return Err(OriginalPathUnsupportedReason::UnrecognizedSchemeBoundary);
    }

    let after_scheme = &input[scheme_length + 1..];
    let leading_separator_count = after_scheme
        .as_bytes()
        .iter()
        .take_while(|byte| matches!(byte, b'/' | b'\\'))
        .count();

    // The URL parser accepts zero, one, or two authority separators for HTTP
    // and HTTPS. Three or more can be normalized in ways that make the
    // original authority/path boundary ambiguous, so decline to infer it.
    if leading_separator_count > 2 {
        return Err(OriginalPathUnsupportedReason::AmbiguousAuthorityBoundary);
    }

    let authority_and_more = &after_scheme[leading_separator_count..];

    // Preserve original separators while recognizing both
    // forward slashes and backslashes in HTTP/HTTPS URLs.
    let path_start = authority_and_more.find(['/', '\\', '?', '#']);

    match path_start {
        Some(index) if matches!(authority_and_more.as_bytes()[index], b'/' | b'\\') => {
            let path_and_more = &authority_and_more[index..];

            let path_end = path_and_more
                .find(['?', '#'])
                .unwrap_or(path_and_more.len());

            Ok(&path_and_more[..path_end])
        }
        _ => Ok("/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_url;

    #[test]
    fn identifies_encoded_parent_directory_segment() {
        let parsed = parse_url("https://example.com/public/%2e%2e/admin").unwrap();

        let analysis = analyze_normalization_with_coverage(&parsed);
        let findings = analysis.findings;

        assert_eq!(
            analysis.original_path_analysis,
            OriginalPathAnalysis::Examined
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, NormalizationKind::EncodedDotSegment);
        assert_eq!(findings[0].original_path, "/public/%2e%2e/admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn ignores_encoded_dot_segments_in_query_parameters() {
        let parsed = parse_url("https://example.com/admin?note=%2e%2e").unwrap();

        let findings = analyze_normalization(&parsed);

        assert!(findings.is_empty());
    }

    #[test]
    fn recognizes_mixed_encoded_dot_segments() {
        for segment in ["%2e%2e", "%2e.", ".%2e", "%2E%2E"] {
            let input = format!("https://example.com/public/{segment}/admin");
            let parsed = parse_url(&input).unwrap();

            let findings = analyze_normalization(&parsed);

            assert_eq!(
                findings.len(),
                1,
                "Expected a normalization finding for {segment}"
            );

            assert_eq!(findings[0].kind, NormalizationKind::EncodedDotSegment);

            assert_eq!(findings[0].normalized_path, "/admin");
        }
    }

    #[test]
    fn handles_credentials_in_url_authority() {
        let parsed = parse_url("https://user:password@example.com/public/%2e%2e/admin").unwrap();

        let findings = analyze_normalization(&parsed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].original_path, "/public/%2e%2e/admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn handles_ipv4_authority_with_port() {
        let parsed = parse_url("https://127.0.0.1:8443/public/%2e%2e/admin").unwrap();

        let analysis = analyze_normalization_with_coverage(&parsed);

        assert_eq!(
            analysis.original_path_analysis,
            OriginalPathAnalysis::Examined
        );
        assert_eq!(analysis.findings.len(), 1);
        assert_eq!(analysis.findings[0].original_path, "/public/%2e%2e/admin");
        assert_eq!(analysis.findings[0].normalized_path, "/admin");
    }

    #[test]
    fn ignores_encoded_dot_segments_in_fragments() {
        let parsed = parse_url("https://example.com/admin#section/%2e%2e/test").unwrap();

        let findings = analyze_normalization(&parsed);

        assert!(findings.is_empty());
    }

    #[test]
    fn handles_query_before_fragment_without_path() {
        let parsed =
            parse_url("https://example.com?redirect=/public/%2e%2e/admin#section").unwrap();

        let findings = analyze_normalization(&parsed);

        assert!(findings.is_empty());
    }

    #[test]
    fn preserves_backslash_path_evidence() {
        let input = r"https://example.com\public\%2e%2e\admin";

        let parsed = parse_url(input).unwrap();
        let findings = analyze_normalization(&parsed);

        assert_eq!(parsed.path, "/admin");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].original_path, r"\public\%2e%2e\admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn ignores_backslash_dot_segments_in_query() {
        let parsed = parse_url(r"https://example.com/admin?note=\public\%2e%2e\admin").unwrap();

        let findings = analyze_normalization(&parsed);

        assert!(findings.is_empty());
    }

    #[test]
    fn handles_ipv6_authority_with_port() {
        let parsed = parse_url("https://[2001:db8::1]:8443/public/%2e%2e/admin").unwrap();

        let findings = analyze_normalization(&parsed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].original_path, "/public/%2e%2e/admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn handles_encoded_credentials_in_authority() {
        let parsed =
            parse_url("https://user%2Fname:pass%40word@example.com/public/%2e%2e/admin").unwrap();

        let findings = analyze_normalization(&parsed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].original_path, "/public/%2e%2e/admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn handles_backslash_after_port() {
        let parsed = parse_url(r"https://example.com:8443\public\%2e%2e\admin").unwrap();

        let findings = analyze_normalization(&parsed);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].original_path, r"\public\%2e%2e\admin");
        assert_eq!(findings[0].normalized_path, "/admin");
    }

    #[test]
    fn examines_unusual_supported_http_authority_spellings() {
        for (input, expected_path) in [
            (
                r"https:\\example.com\public\%2e%2e\admin",
                r"\public\%2e%2e\admin",
            ),
            (
                "https:/example.com/public/%2e%2e/admin",
                "/public/%2e%2e/admin",
            ),
            (
                "https:example.com/public/%2e%2e/admin",
                "/public/%2e%2e/admin",
            ),
        ] {
            let parsed = parse_url(input).unwrap();
            let analysis = analyze_normalization_with_coverage(&parsed);

            assert_eq!(
                analysis.original_path_analysis,
                OriginalPathAnalysis::Examined,
                "Expected supported extraction for {input}"
            );
            assert_eq!(analysis.findings.len(), 1, "Expected finding for {input}");
            assert_eq!(analysis.findings[0].original_path, expected_path);
            assert_eq!(analysis.findings[0].normalized_path, "/admin");
        }
    }

    #[test]
    fn treats_query_or_fragment_before_a_path_like_suffix_as_examined_root_path() {
        for input in [
            "https://example.com?next=/public/%2e%2e/admin",
            "https://example.com#next=/public/%2e%2e/admin",
        ] {
            let parsed = parse_url(input).unwrap();
            let analysis = analyze_normalization_with_coverage(&parsed);

            assert_eq!(
                analysis.original_path_analysis,
                OriginalPathAnalysis::Examined
            );
            assert!(analysis.findings.is_empty());
        }
    }

    #[test]
    fn avoids_findings_for_ambiguous_scheme_separators() {
        let input = "https:////example.com/public/%2e%2e/admin";
        let parsed = parse_url(input).unwrap();

        assert_eq!(parsed.original, input);
        assert_eq!(parsed.path, "/admin");

        let analysis = analyze_normalization_with_coverage(&parsed);

        assert_eq!(
            analysis.original_path_analysis,
            OriginalPathAnalysis::Unsupported {
                reason: OriginalPathUnsupportedReason::AmbiguousAuthorityBoundary,
            }
        );
        assert!(analysis.findings.is_empty());

        // The compatibility API still returns no unsupported evidence as a finding.
        assert!(analyze_normalization(&parsed).is_empty());
    }

    #[test]
    fn exposes_unrecognized_scheme_boundaries_as_unsupported() {
        let parsed = parse_url(" https://example.com/public/%2e%2e/admin").unwrap();

        let analysis = analyze_normalization_with_coverage(&parsed);

        assert_eq!(
            analysis.original_path_analysis,
            OriginalPathAnalysis::Unsupported {
                reason: OriginalPathUnsupportedReason::UnrecognizedSchemeBoundary,
            }
        );
        assert!(analysis.findings.is_empty());
    }

    #[test]
    fn normalization_kind_has_readable_title() {
        assert_eq!(
            NormalizationKind::EncodedDotSegment.title(),
            "Encoded parent-directory segment"
        );
    }
}
