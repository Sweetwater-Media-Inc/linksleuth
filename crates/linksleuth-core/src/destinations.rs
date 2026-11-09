use std::collections::{HashSet, VecDeque};

use url::Url;

use crate::decoder::{DecodeError, DecodingResult, DecodingStatus, decode_percent};
use crate::parser::{UrlError, parse_url};

/// Maximum nested candidate depth. Directly embedded candidates have depth 0.
pub const MAX_DESTINATION_NESTING_DEPTH: usize = 3;

/// Maximum candidate occurrences retained in one analysis.
pub const MAX_DISCOVERED_DESTINATIONS: usize = 64;

/// Maximum parameters examined from each discovered candidate URL.
pub const MAX_NESTED_QUERY_PARAMETERS: usize = 32;

/// Legacy directly embedded absolute HTTP/HTTPS destination representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedDestination {
    pub url: String,
    pub hostname: String,
    pub source_parameter: String,
    pub decoding_passes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationScheme {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationReferenceKind {
    Absolute { scheme: DestinationScheme },
    ProtocolRelative,
}

/// A syntactically supported potential destination reference.
///
/// `submitted` is preserved separately from `normalized`. Protocol-relative
/// references remain protocol-relative in `normalized`; the HTTPS parsing
/// context used internally does not assert a destination scheme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationReference {
    pub kind: DestinationReferenceKind,
    pub submitted: String,
    pub normalized: String,
    pub hostname: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationProvenance {
    /// Zero-based position in the submitted URL's top-level query parameters.
    pub top_level_parameter_index: usize,
    /// Zero-based position in the immediate parent URL's query parameters.
    pub source_parameter_index: usize,
    /// Form-decoded source parameter name. This remains sensitive evidence.
    pub source_parameter: String,
    /// Direct candidates have depth 0; nested candidates have depth 1 or more.
    pub depth: usize,
    /// Index in `DestinationDiscovery::candidates` for a nested candidate's parent.
    pub parent_candidate_index: Option<usize>,
    /// Complete bounded decoding provenance for the source parameter value.
    pub decoding: DecodingResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDestination {
    pub reference: DestinationReference,
    pub provenance: DestinationProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationDiscoveryLimit {
    MaximumNestingDepth,
    MaximumCandidateCount,
    MaximumNestedParameterCount,
    DecoderInputLength,
    DecodingPassLimit,
    NestedUrlLength,
}

impl DestinationDiscoveryLimit {
    /// Fixed privacy-safe explanation suitable for detailed or shareable output.
    pub const fn explanation(self) -> &'static str {
        match self {
            Self::MaximumNestingDepth => {
                "Maximum destination nesting depth was reached; deeper parameters were not examined."
            }
            Self::MaximumCandidateCount => {
                "Maximum destination candidate count was reached; additional candidates may exist."
            }
            Self::MaximumNestedParameterCount => {
                "A nested URL exceeded the parameter examination limit; additional parameters were not examined."
            }
            Self::DecoderInputLength => {
                "A destination source exceeded the decoder input limit and was not examined."
            }
            Self::DecodingPassLimit => {
                "A destination source reached the decoding pass limit; additional encoding may remain."
            }
            Self::NestedUrlLength => {
                "A nested URL exceeded the supported URL length and was not examined further."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationDiscoveryStatus {
    Complete,
    Incomplete {
        limits: Vec<DestinationDiscoveryLimit>,
    },
}

impl DestinationDiscoveryStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn limits(&self) -> &[DestinationDiscoveryLimit] {
        match self {
            Self::Complete => &[],
            Self::Incomplete { limits } => limits,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationDiscovery {
    pub candidates: Vec<DiscoveredDestination>,
    pub status: DestinationDiscoveryStatus,
}

/// Borrowed top-level evidence used to seed bounded destination discovery.
pub struct DestinationDiscoverySeed<'a> {
    pub top_level_parameter_index: usize,
    pub source_parameter: &'a str,
    pub decoding: &'a Result<DecodingResult, DecodeError>,
}

/// Preserves the existing absolute HTTP/HTTPS discovery API.
pub fn discover_destination(
    value: &str,
    parameter_name: &str,
    decoding_passes: usize,
) -> Option<EmbeddedDestination> {
    let reference = discover_destination_reference(value)?;

    if !matches!(reference.kind, DestinationReferenceKind::Absolute { .. }) {
        return None;
    }

    Some(EmbeddedDestination {
        url: reference.normalized,
        hostname: reference.hostname,
        source_parameter: parameter_name.to_string(),
        decoding_passes,
    })
}

/// Classifies a complete absolute HTTP/HTTPS URL or protocol-relative reference.
///
/// The complete input value must be the reference; arbitrary substrings are not
/// searched. Backslash-based and parser-trimmed spellings are declined as
/// ambiguous. No network operation is performed.
pub fn discover_destination_reference(value: &str) -> Option<DestinationReference> {
    if value.trim() != value || value.contains('\\') {
        return None;
    }

    if value.starts_with("//") {
        return discover_protocol_relative_reference(value);
    }

    let parsed = Url::parse(value).ok()?;
    let kind = match parsed.scheme() {
        "http"
            if value
                .get(..7)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://")) =>
        {
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Http,
            }
        }
        "https"
            if value
                .get(..8)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://")) =>
        {
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Https,
            }
        }
        _ => return None,
    };
    let hostname = parsed.host_str()?.to_string();

    Some(DestinationReference {
        kind,
        submitted: value.to_string(),
        normalized: parsed.to_string(),
        hostname,
    })
}

fn discover_protocol_relative_reference(value: &str) -> Option<DestinationReference> {
    let authority_and_more = value.strip_prefix("//")?;
    if authority_and_more.is_empty()
        || authority_and_more.starts_with('/')
        || authority_and_more.starts_with('\\')
    {
        return None;
    }
    let explicit_port = protocol_relative_port(authority_and_more)?;

    // HTTPS is a parsing context only. The returned representation has the
    // scheme removed and remains explicitly protocol-relative.
    let parsing_context = format!("https:{value}");
    let parsed = Url::parse(&parsing_context).ok()?;
    let hostname = parsed.host_str()?.to_string();
    let mut normalized = parsed.as_str().strip_prefix("https:")?.to_string();

    // The URL parser elides an explicit port when it matches the parsing
    // context's default. A protocol-relative reference has no verified scheme,
    // so restore that port to keep normalization scheme-neutral.
    if parsed.port().is_none()
        && let Some(port) = explicit_port
    {
        let authority_end = normalized[2..]
            .find(['/', '?', '#'])
            .map_or(normalized.len(), |offset| offset + 2);
        normalized.insert_str(authority_end, &format!(":{port}"));
    }

    Some(DestinationReference {
        kind: DestinationReferenceKind::ProtocolRelative,
        submitted: value.to_string(),
        normalized,
        hostname,
    })
}

fn protocol_relative_port(authority_and_more: &str) -> Option<Option<u16>> {
    let authority_end = authority_and_more
        .find(['/', '?', '#'])
        .unwrap_or(authority_and_more.len());
    let authority = &authority_and_more[..authority_end];
    let host_and_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);

    let port_text = if let Some(bracket_end) = host_and_port.find(']') {
        match &host_and_port[bracket_end + 1..] {
            "" => return Some(None),
            suffix => suffix.strip_prefix(':')?,
        }
    } else {
        let Some((_, port)) = host_and_port.rsplit_once(':') else {
            return Some(None);
        };
        port
    };

    if port_text.is_empty() || !port_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    port_text.parse().ok().map(Some)
}

/// Discovers direct and nested destination candidates using iterative traversal.
///
/// Duplicate candidate occurrences are retained for provenance, while each
/// normalized reference is traversed at most once to prevent cycles and
/// duplicate work.
pub fn discover_destinations(seeds: &[DestinationDiscoverySeed<'_>]) -> DestinationDiscovery {
    let mut state = DiscoveryState::default();

    for seed in seeds {
        if state.candidate_limit_reached {
            break;
        }

        let decoding = match seed.decoding {
            Ok(decoding) => decoding,
            Err(DecodeError::InputTooLong) => {
                state.add_limit(DestinationDiscoveryLimit::DecoderInputLength);
                continue;
            }
            Err(DecodeError::InvalidUtf8) => continue,
        };

        state.observe_decoding_status(decoding);

        let Some(reference) = discover_destination_reference(&decoding.decoded) else {
            continue;
        };

        state.add_candidate(
            reference,
            DestinationProvenance {
                top_level_parameter_index: seed.top_level_parameter_index,
                source_parameter_index: seed.top_level_parameter_index,
                source_parameter: seed.source_parameter.to_string(),
                depth: 0,
                parent_candidate_index: None,
                decoding: decoding.clone(),
            },
        );
    }

    state.traverse_nested_candidates();
    state.finish()
}

#[derive(Debug)]
struct PendingTraversal {
    candidate_index: usize,
    parser_input: String,
}

#[derive(Default)]
struct DiscoveryState {
    candidates: Vec<DiscoveredDestination>,
    pending: VecDeque<PendingTraversal>,
    traversed: HashSet<(DestinationReferenceKind, String)>,
    limits: Vec<DestinationDiscoveryLimit>,
    candidate_limit_reached: bool,
}

impl DiscoveryState {
    fn add_limit(&mut self, limit: DestinationDiscoveryLimit) {
        if !self.limits.contains(&limit) {
            self.limits.push(limit);
        }
    }

    fn observe_decoding_status(&mut self, decoding: &DecodingResult) {
        if decoding.status == DecodingStatus::PassLimitReached {
            self.add_limit(DestinationDiscoveryLimit::DecodingPassLimit);
        }
    }

    fn add_candidate(
        &mut self,
        reference: DestinationReference,
        provenance: DestinationProvenance,
    ) -> Option<usize> {
        if self.candidates.len() >= MAX_DISCOVERED_DESTINATIONS {
            self.add_limit(DestinationDiscoveryLimit::MaximumCandidateCount);
            self.candidate_limit_reached = true;
            return None;
        }

        let candidate_index = self.candidates.len();
        let traversal_key = (reference.kind, reference.normalized.clone());
        let parser_input = match reference.kind {
            DestinationReferenceKind::Absolute { .. } => reference.submitted.clone(),
            DestinationReferenceKind::ProtocolRelative => {
                format!("https:{}", reference.submitted)
            }
        };

        self.candidates.push(DiscoveredDestination {
            reference,
            provenance,
        });

        if self.traversed.insert(traversal_key) {
            self.pending.push_back(PendingTraversal {
                candidate_index,
                parser_input,
            });
        }

        Some(candidate_index)
    }

    fn traverse_nested_candidates(&mut self) {
        while let Some(pending) = self.pending.pop_front() {
            if self.candidate_limit_reached {
                break;
            }

            let parsed = match parse_url(&pending.parser_input) {
                Ok(parsed) => parsed,
                Err(UrlError::TooLong) => {
                    self.add_limit(DestinationDiscoveryLimit::NestedUrlLength);
                    continue;
                }
                Err(UrlError::TooManyQueryParameters) => {
                    self.add_limit(DestinationDiscoveryLimit::MaximumNestedParameterCount);
                    continue;
                }
                Err(
                    UrlError::Invalid(_) | UrlError::UnsupportedScheme | UrlError::MissingHostname,
                ) => continue,
            };

            let parent = &self.candidates[pending.candidate_index];
            let next_depth = parent.provenance.depth + 1;
            let top_level_parameter_index = parent.provenance.top_level_parameter_index;

            if parent.provenance.depth >= MAX_DESTINATION_NESTING_DEPTH {
                if !parsed.query_parameters.is_empty() {
                    self.add_limit(DestinationDiscoveryLimit::MaximumNestingDepth);
                }
                continue;
            }

            let parameter_count = parsed.query_parameters.len();
            if parameter_count > MAX_NESTED_QUERY_PARAMETERS {
                self.add_limit(DestinationDiscoveryLimit::MaximumNestedParameterCount);
            }

            for (parameter_index, parameter) in parsed
                .query_parameters
                .iter()
                .take(MAX_NESTED_QUERY_PARAMETERS)
                .enumerate()
            {
                let decoding = match decode_percent(&parameter.raw_value) {
                    Ok(decoding) => decoding,
                    Err(DecodeError::InputTooLong) => {
                        self.add_limit(DestinationDiscoveryLimit::DecoderInputLength);
                        continue;
                    }
                    Err(DecodeError::InvalidUtf8) => continue,
                };

                self.observe_decoding_status(&decoding);

                let Some(reference) = discover_destination_reference(&decoding.decoded) else {
                    continue;
                };

                self.add_candidate(
                    reference,
                    DestinationProvenance {
                        top_level_parameter_index,
                        source_parameter_index: parameter_index,
                        source_parameter: parameter.name.clone(),
                        depth: next_depth,
                        parent_candidate_index: Some(pending.candidate_index),
                        decoding,
                    },
                );

                if self.candidate_limit_reached {
                    break;
                }
            }
        }
    }

    fn finish(self) -> DestinationDiscovery {
        let status = if self.limits.is_empty() {
            DestinationDiscoveryStatus::Complete
        } else {
            DestinationDiscoveryStatus::Incomplete {
                limits: self.limits,
            }
        };

        DestinationDiscovery {
            candidates: self.candidates,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_url;

    fn encode(value: &str) -> String {
        url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
    }

    fn discover(input: &str) -> DestinationDiscovery {
        let parsed = parse_url(input).unwrap();
        let decodings = parsed
            .query_parameters
            .iter()
            .map(|parameter| decode_percent(&parameter.raw_value))
            .collect::<Vec<_>>();
        let seeds = parsed
            .query_parameters
            .iter()
            .zip(&decodings)
            .enumerate()
            .map(|(index, (parameter, decoding))| DestinationDiscoverySeed {
                top_level_parameter_index: index,
                source_parameter: &parameter.name,
                decoding,
            })
            .collect::<Vec<_>>();

        discover_destinations(&seeds)
    }

    #[test]
    fn preserves_legacy_absolute_destination_api() {
        let result = discover_destination("https://other.example/path", "target", 2).unwrap();

        assert_eq!(result.url, "https://other.example/path");
        assert_eq!(result.hostname, "other.example");
        assert_eq!(result.source_parameter, "target");
        assert_eq!(result.decoding_passes, 2);
    }

    #[test]
    fn classifies_absolute_http_and_https_references() {
        let http = discover_destination_reference("http://one.example/path").unwrap();
        let https = discover_destination_reference("https://two.example/path").unwrap();

        assert_eq!(
            http.kind,
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Http,
            }
        );
        assert_eq!(
            https.kind,
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Https,
            }
        );
        assert_eq!(http.submitted, "http://one.example/path");
        assert_eq!(https.hostname, "two.example");
    }

    #[test]
    fn classifies_protocol_relative_ipv6_ports_credentials_and_unicode_hosts() {
        let ipv6 = discover_destination_reference("//[2001:db8::1]:8443/path").unwrap();
        let default_context_port =
            discover_destination_reference("//example.org:443/path").unwrap();
        let credentials = discover_destination_reference("//user:pass@example.org/path").unwrap();
        let unicode = discover_destination_reference("//例え.テスト/路径").unwrap();

        assert_eq!(ipv6.kind, DestinationReferenceKind::ProtocolRelative);
        assert_eq!(ipv6.hostname, "[2001:db8::1]");
        assert_eq!(ipv6.normalized, "//[2001:db8::1]:8443/path");
        assert_eq!(default_context_port.normalized, "//example.org:443/path");
        assert_eq!(credentials.hostname, "example.org");
        assert_eq!(unicode.hostname, "xn--r8jz45g.xn--zckzah");
        assert_eq!(unicode.submitted, "//例え.テスト/路径");
        assert!(unicode.normalized.starts_with("//xn--r8jz45g.xn--zckzah/"));
    }

    #[test]
    fn rejects_malformed_ambiguous_and_ordinary_relative_references() {
        for value in [
            "/local/path",
            "../relative/path",
            "//",
            "///ambiguous.example/path",
            r"//example.org\ambiguous",
            "//example.org:99999/path",
            "//example.org:/path",
            "//[2001:db8::1/path",
            "ftp://example.org/file",
            "prefix https://example.org/path",
        ] {
            assert!(
                discover_destination_reference(value).is_none(),
                "Expected unsupported reference: {value}"
            );
        }
    }

    #[test]
    fn discovers_single_and_multiple_nested_levels() {
        let final_url = "https://final.example/path";
        let middle_url = format!("https://middle.example/?next={}", encode(final_url));
        let first_url = format!("https://first.example/?middle={}", encode(&middle_url));
        let input = format!("https://top.example/?target={}", encode(&first_url));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 3);
        assert!(discovery.status.is_complete());
        assert_eq!(discovery.candidates[0].reference.hostname, "first.example");
        assert_eq!(discovery.candidates[0].provenance.depth, 0);
        assert_eq!(discovery.candidates[1].reference.hostname, "middle.example");
        assert_eq!(discovery.candidates[1].provenance.depth, 1);
        assert_eq!(
            discovery.candidates[1].provenance.parent_candidate_index,
            Some(0)
        );
        assert_eq!(discovery.candidates[2].reference.hostname, "final.example");
        assert_eq!(discovery.candidates[2].provenance.depth, 2);
        assert_eq!(
            discovery.candidates[2].provenance.parent_candidate_index,
            Some(1)
        );
    }

    #[test]
    fn preserves_repeated_parameter_positions_and_avoids_duplicate_traversal() {
        let child = "https://child.example/path";
        let repeated = format!("https://middle.example/?next={}", encode(child));
        let input = format!(
            "https://top.example/?target={}&target={}",
            encode(&repeated),
            encode(&repeated)
        );

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 3);
        assert_eq!(
            discovery.candidates[0].provenance.top_level_parameter_index,
            0
        );
        assert_eq!(
            discovery.candidates[1].provenance.top_level_parameter_index,
            1
        );
        assert_eq!(discovery.candidates[2].reference.hostname, "child.example");
        assert_eq!(
            discovery.candidates[2].provenance.parent_candidate_index,
            Some(0)
        );
        assert!(discovery.status.is_complete());
    }

    #[test]
    fn preserves_repeated_nested_parameter_names_and_positions() {
        let nested = format!(
            "https://middle.example/?next={}&next={}",
            encode("https://one.example/path"),
            encode("https://two.example/path")
        );
        let input = format!("https://top.example/?target={}", encode(&nested));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 3);
        assert_eq!(discovery.candidates[1].provenance.source_parameter, "next");
        assert_eq!(discovery.candidates[1].provenance.source_parameter_index, 0);
        assert_eq!(discovery.candidates[2].provenance.source_parameter, "next");
        assert_eq!(discovery.candidates[2].provenance.source_parameter_index, 1);
        assert_eq!(
            discovery.candidates[1].provenance.parent_candidate_index,
            Some(0)
        );
        assert_eq!(
            discovery.candidates[2].provenance.parent_candidate_index,
            Some(0)
        );
    }

    #[test]
    fn applies_percent_decoding_to_nested_parameter_values() {
        // Four layers here plus the top-level parent encoding leave two layers
        // for the nested source after the top-level three-pass decoder runs.
        let nested = "https%2525253A%2525252F%2525252Fdeep.example%2525252Fpath";
        let parent = format!("https://parent.example/?next={nested}");
        let input = format!("https://top.example/?target={}", encode(&parent));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 2);
        let nested_candidate = &discovery.candidates[1];
        assert_eq!(nested_candidate.reference.hostname, "deep.example");
        assert_eq!(nested_candidate.provenance.decoding.steps.len(), 2);
    }

    #[test]
    fn reports_maximum_nesting_depth_as_incomplete() {
        let mut nested = "https://level5.example/?tail=value".to_string();
        for level in (1..=4).rev() {
            nested = format!("https://level{level}.example/?next={}", encode(&nested));
        }
        let input = format!("https://top.example/?target={}", encode(&nested));

        let discovery = discover(&input);

        assert!(
            discovery
                .candidates
                .iter()
                .any(|candidate| candidate.provenance.depth == 3)
        );
        assert!(
            discovery
                .status
                .limits()
                .contains(&DestinationDiscoveryLimit::MaximumNestingDepth)
        );
    }

    #[test]
    fn reports_maximum_candidate_count_as_incomplete() {
        let query = (0..=MAX_DISCOVERED_DESTINATIONS)
            .map(|index| {
                format!(
                    "p{index}={}",
                    encode(&format!("https://host{index}.example/"))
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        let input = format!("https://top.example/?{query}");

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), MAX_DISCOVERED_DESTINATIONS);
        assert!(
            discovery
                .status
                .limits()
                .contains(&DestinationDiscoveryLimit::MaximumCandidateCount)
        );
    }

    #[test]
    fn reports_nested_parameter_limit_and_examines_only_bounded_prefix() {
        let nested_query = (0..=MAX_NESTED_QUERY_PARAMETERS)
            .map(|index| format!("p{index}=value"))
            .collect::<Vec<_>>()
            .join("&");
        let nested = format!("https://nested.example/?{nested_query}");
        let input = format!("https://top.example/?target={}", encode(&nested));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 1);
        assert!(
            discovery
                .status
                .limits()
                .contains(&DestinationDiscoveryLimit::MaximumNestedParameterCount)
        );
    }

    #[test]
    fn malformed_nested_values_are_not_guessed_as_destinations() {
        let parent = "https://parent.example/?next=//[invalid&other=/local/path";
        let input = format!("https://top.example/?target={}", encode(parent));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 1);
        assert!(discovery.status.is_complete());
    }

    #[test]
    fn reports_decoding_pass_limit_as_incomplete() {
        let discovery = discover("https://top.example/?target=%2525252541");

        assert!(discovery.candidates.is_empty());
        assert_eq!(
            discovery.status.limits(),
            &[DestinationDiscoveryLimit::DecodingPassLimit]
        );
    }

    #[test]
    fn ordinary_urls_have_complete_empty_discovery() {
        let discovery = discover("https://top.example/products?category=books");

        assert!(discovery.candidates.is_empty());
        assert!(discovery.status.is_complete());
    }
}
