#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizationKind {
    EncodedDotSegment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationFinding {
    pub kind: NormalizationKind,
    pub original_path: String,
    pub normalized_path: String,
    pub explanation: String,
}

use crate::parser::ParsedUrl;

pub fn analyze_normalization(parsed: &ParsedUrl) -> Vec<NormalizationFinding> {
    let mut findings = Vec::new();

    let Some(original_path) = extract_original_path(&parsed.original) else {
        return findings;
    };

    let contains_encoded_dot_segment = original_path.split('/').any(|segment| {
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

    findings
}

fn extract_original_path(input: &str) -> Option<&str> {
    let (_, remainder) = input.split_once("://")?;

    // Skip the authority (hostname, optional credentials and port).
    let path_start = remainder.find(['/', '?', '#']);

    match path_start {
        Some(index) if remainder.as_bytes()[index] == b'/' => {
            let path_and_more = &remainder[index..];
            let path_end = path_and_more
                .find(['?', '#'])
                .unwrap_or(path_and_more.len());

            Some(&path_and_more[..path_end])
        }
        _ => Some("/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_url;

    #[test]
    fn identifies_encoded_parent_directory_segment() {
        let parsed = parse_url("https://example.com/public/%2e%2e/admin").unwrap();

        let findings = analyze_normalization(&parsed);

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
}
