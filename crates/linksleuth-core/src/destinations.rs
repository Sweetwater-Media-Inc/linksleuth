use std::collections::{HashSet, VecDeque};

use url::Url;

use crate::decoder::{DecodeError, DecodingResult, DecodingStatus, decode_percent};
use crate::encodings::{
    Base64Alphabet, Base64DecodingOutcome, Base64Padding, HtmlReferenceDecodingOutcome,
    HtmlReferenceIssueKind, decode_base64_complete, decode_html_character_references_complete,
};
use crate::parser::{UrlError, parse_url};

/// Maximum nested candidate depth. Directly embedded candidates have depth 0.
pub const MAX_DESTINATION_NESTING_DEPTH: usize = 3;

/// Maximum candidate occurrences retained in one analysis.
pub const MAX_DISCOVERED_DESTINATIONS: usize = 64;

/// Maximum parameters examined from each discovered candidate URL.
pub const MAX_NESTED_QUERY_PARAMETERS: usize = 32;

/// Maximum parameters examined from one supported compound query value.
pub const MAX_COMPOUND_QUERY_PARAMETERS: usize = 16;

/// Maximum compound-query interpretations considered for one source value.
///
/// LinkSleuth uses exactly one deterministic representation: the bounded
/// percent-decoded value, followed by at most one supported HTML-reference
/// interpretation when applicable.
pub const MAX_COMPOUND_QUERY_INTERPRETATIONS: usize = 1;

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
    Absolute {
        scheme: DestinationScheme,
    },
    ProtocolRelative,
    /// A complete hostname/path reference found in a destination-bearing
    /// parameter. No HTTP or HTTPS scheme is inferred.
    HostnameFirst,
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
    /// Ordered transformations that produced the classified reference.
    pub transformations: Vec<DestinationTransformation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationTransformation {
    PercentDecoding {
        passes: usize,
    },
    Base64Decoding {
        alphabet: Base64Alphabet,
        padding: Base64Padding,
    },
    HtmlCharacterReferenceDecoding {
        reference_count: usize,
    },
    CompoundQueryExtraction {
        parameter_position: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DestinationStructureKind {
    MultiplePercentDecodingTransformations,
    EncodedQueryDelimiter,
    CompoundQueryValue,
    AmbiguousHostnameFirstReference,
}

impl DestinationStructureKind {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::MultiplePercentDecodingTransformations => {
                "encoding.multiple-percent-transformations"
            }
            Self::EncodedQueryDelimiter => "structure.encoded-query-delimiter",
            Self::CompoundQueryValue => "structure.compound-query-value",
            Self::AmbiguousHostnameFirstReference => "structure.hostname-first-ambiguous",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationStructureObservation {
    pub kind: DestinationStructureKind,
    pub top_level_parameter_index: usize,
    pub source_parameter_index: usize,
    pub depth: usize,
    pub parent_candidate_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationInterpretationIssueKind {
    AmbiguousHostnameFirstReference,
}

impl DestinationInterpretationIssueKind {
    pub const fn explanation(self) -> &'static str {
        match self {
            Self::AmbiguousHostnameFirstReference => {
                "A destination-bearing value began with a valid hostname but used an ambiguous authority/path boundary, so it was not classified as a destination."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationInterpretationIssue {
    pub kind: DestinationInterpretationIssueKind,
    pub top_level_parameter_index: usize,
    pub source_parameter_index: usize,
    pub depth: usize,
    pub parent_candidate_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationInterpretationCoverage {
    Complete,
    Partial {
        issues: Vec<DestinationInterpretationIssueKind>,
    },
}

impl DestinationInterpretationCoverage {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn issues(&self) -> &[DestinationInterpretationIssueKind] {
        match self {
            Self::Complete => &[],
            Self::Partial { issues } => issues,
        }
    }
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
    MaximumCompoundParameterCount,
    DecoderInputLength,
    DecodingPassLimit,
    NestedUrlLength,
    SecondaryEncodingInputLength,
    SecondaryDecodedOutputLength,
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
            Self::MaximumCompoundParameterCount => {
                "A compound query value exceeded the inner-parameter examination limit; additional inner parameters were not examined."
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
            Self::SecondaryEncodingInputLength => {
                "A destination source exceeded the secondary-decoding input limit and was not examined."
            }
            Self::SecondaryDecodedOutputLength => {
                "A secondary-decoding result exceeded the supported output limit and was not retained."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationEncodingIssueKind {
    MalformedBase64,
    MalformedHtmlCharacterReference,
    UnsupportedHtmlCharacterReference,
    InvalidHtmlUnicodeScalar,
}

impl DestinationEncodingIssueKind {
    /// Fixed privacy-safe explanation suitable for detailed or shareable output.
    pub const fn explanation(self) -> &'static str {
        match self {
            Self::MalformedBase64 => {
                "A Base64-like destination source had malformed alphabet, padding, or trailing bits."
            }
            Self::MalformedHtmlCharacterReference => {
                "An HTML character reference was malformed or missing its required semicolon."
            }
            Self::UnsupportedHtmlCharacterReference => {
                "An HTML named character reference was outside the supported URL-syntax set."
            }
            Self::InvalidHtmlUnicodeScalar => {
                "An HTML numeric character reference did not represent a valid Unicode scalar value."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationEncodingIssue {
    pub kind: DestinationEncodingIssueKind,
    pub top_level_parameter_index: usize,
    pub source_parameter_index: usize,
    pub depth: usize,
    pub parent_candidate_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationEncodingCoverage {
    Complete,
    Incomplete {
        issues: Vec<DestinationEncodingIssueKind>,
    },
}

impl DestinationEncodingCoverage {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn issues(&self) -> &[DestinationEncodingIssueKind] {
        match self {
            Self::Complete => &[],
            Self::Incomplete { issues } => issues,
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
    pub encoding_coverage: DestinationEncodingCoverage,
    pub encoding_issues: Vec<DestinationEncodingIssue>,
    pub structure_observations: Vec<DestinationStructureObservation>,
    pub interpretation_coverage: DestinationInterpretationCoverage,
    pub interpretation_issues: Vec<DestinationInterpretationIssue>,
}

impl DestinationDiscovery {
    pub fn is_complete(&self) -> bool {
        self.status.is_complete()
            && self.encoding_coverage.is_complete()
            && self.interpretation_coverage.is_complete()
    }
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
    if value.trim() != value || value.contains('\\') || value.chars().any(char::is_control) {
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

        state.examine_source(
            decoding.clone(),
            CandidateSource {
                top_level_parameter_index: seed.top_level_parameter_index,
                source_parameter_index: seed.top_level_parameter_index,
                source_parameter: seed.source_parameter.to_string(),
                depth: 0,
                parent_candidate_index: None,
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

struct CandidateSource {
    top_level_parameter_index: usize,
    source_parameter_index: usize,
    source_parameter: String,
    depth: usize,
    parent_candidate_index: Option<usize>,
}

impl CandidateSource {
    fn is_destination_context(&self) -> bool {
        is_destination_parameter(&self.source_parameter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostnameFirstClassification {
    Supported,
    Ambiguous,
    NotApplicable,
}

#[derive(Default)]
struct DiscoveryState {
    candidates: Vec<DiscoveredDestination>,
    pending: VecDeque<PendingTraversal>,
    traversed: HashSet<(DestinationReferenceKind, String)>,
    limits: Vec<DestinationDiscoveryLimit>,
    encoding_issues: Vec<DestinationEncodingIssue>,
    structure_observations: Vec<DestinationStructureObservation>,
    interpretation_issues: Vec<DestinationInterpretationIssue>,
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

    fn examine_source(&mut self, decoding: DecodingResult, source: CandidateSource) {
        self.observe_decoding_status(&decoding);

        if decoding.steps.len() > 1 {
            self.add_structure_observation(
                DestinationStructureKind::MultiplePercentDecodingTransformations,
                &source,
            );
        }
        if source.is_destination_context() && reveals_encoded_query_delimiter(&decoding) {
            self.add_structure_observation(
                DestinationStructureKind::EncodedQueryDelimiter,
                &source,
            );
        }

        let mut transformations = Vec::new();
        if !decoding.steps.is_empty() {
            transformations.push(DestinationTransformation::PercentDecoding {
                passes: decoding.steps.len(),
            });
        }

        if let Some(reference) = discover_destination_reference(&decoding.decoded) {
            self.add_candidate_from_source(reference, decoding, transformations, source);
            return;
        }

        if source.is_destination_context() {
            match classify_hostname_first_reference(&decoding.decoded) {
                (HostnameFirstClassification::Supported, Some(reference)) => {
                    self.add_candidate_from_source(reference, decoding, transformations, source);
                    return;
                }
                (HostnameFirstClassification::Ambiguous, None) => {
                    self.add_hostname_first_ambiguity(&source);
                }
                _ => {}
            }
        }

        match decode_base64_complete(&decoding.decoded) {
            Base64DecodingOutcome::Decoded(base64) => {
                let Some(reference) = discover_destination_reference(&base64.decoded) else {
                    return;
                };
                transformations.push(DestinationTransformation::Base64Decoding {
                    alphabet: base64.alphabet,
                    padding: base64.padding,
                });
                self.add_candidate_from_source(reference, decoding, transformations, source);
                return;
            }
            Base64DecodingOutcome::Malformed => {
                self.add_encoding_issue(DestinationEncodingIssueKind::MalformedBase64, &source);
            }
            Base64DecodingOutcome::InputTooLong => {
                self.add_limit(DestinationDiscoveryLimit::SecondaryEncodingInputLength);
            }
            Base64DecodingOutcome::OutputTooLong => {
                self.add_limit(DestinationDiscoveryLimit::SecondaryDecodedOutputLength);
            }
            Base64DecodingOutcome::InvalidUtf8 => {}
            Base64DecodingOutcome::NotApplicable => {}
        }

        match decode_html_character_references_complete(&decoding.decoded) {
            HtmlReferenceDecodingOutcome::Decoded(html) => {
                if !html.status.is_complete() {
                    for issue in html.status.issues() {
                        self.add_encoding_issue(map_html_issue(*issue), &source);
                    }
                } else if let Some(reference) = discover_destination_reference(&html.decoded) {
                    transformations.push(
                        DestinationTransformation::HtmlCharacterReferenceDecoding {
                            reference_count: html.reference_count,
                        },
                    );
                    self.add_candidate_from_source(reference, decoding, transformations, source);
                    return;
                }
            }
            HtmlReferenceDecodingOutcome::InputTooLong => {
                self.add_limit(DestinationDiscoveryLimit::SecondaryEncodingInputLength);
            }
            HtmlReferenceDecodingOutcome::OutputTooLong => {
                self.add_limit(DestinationDiscoveryLimit::SecondaryDecodedOutputLength);
            }
            HtmlReferenceDecodingOutcome::NotApplicable => {}
        }

        if source.is_destination_context() {
            self.examine_compound_query_value(decoding, transformations, source);
        }
    }

    fn examine_compound_query_value(
        &mut self,
        decoding: DecodingResult,
        transformations: Vec<DestinationTransformation>,
        source: CandidateSource,
    ) {
        let mut interpreted = decoding.decoded.clone();
        let mut compound_transformations = transformations;

        if let HtmlReferenceDecodingOutcome::Decoded(html) =
            decode_html_character_references_complete(&interpreted)
        {
            if !html.status.is_complete() {
                return;
            }
            interpreted = html.decoded;
            compound_transformations.push(
                DestinationTransformation::HtmlCharacterReferenceDecoding {
                    reference_count: html.reference_count,
                },
            );
        }

        let Some((prefix, query)) = interpreted.split_once('?') else {
            return;
        };
        if prefix.is_empty() || query.is_empty() {
            return;
        }

        let prefix_classification = classify_hostname_first_reference(prefix).0;
        if !matches!(
            prefix_classification,
            HostnameFirstClassification::Supported | HostnameFirstClassification::Ambiguous
        ) && discover_destination_reference(prefix).is_none()
        {
            return;
        }

        self.add_structure_observation(DestinationStructureKind::CompoundQueryValue, &source);

        let parameter_count = query.split('&').filter(|part| !part.is_empty()).count();
        if parameter_count > MAX_COMPOUND_QUERY_PARAMETERS {
            self.add_limit(DestinationDiscoveryLimit::MaximumCompoundParameterCount);
        }

        for (position, part) in query
            .split('&')
            .filter(|part| !part.is_empty())
            .take(MAX_COMPOUND_QUERY_PARAMETERS)
            .enumerate()
        {
            let Some((raw_name, raw_value)) = part.split_once('=') else {
                continue;
            };
            let name = form_decode_component(raw_name);
            if !is_destination_parameter(&name) {
                continue;
            }

            let inner_decoding = match decode_percent(raw_value) {
                Ok(result) => result,
                Err(DecodeError::InputTooLong) => {
                    self.add_limit(DestinationDiscoveryLimit::DecoderInputLength);
                    continue;
                }
                Err(DecodeError::InvalidUtf8) => continue,
            };
            self.observe_decoding_status(&inner_decoding);

            let inner_source = CandidateSource {
                top_level_parameter_index: source.top_level_parameter_index,
                source_parameter_index: position,
                source_parameter: name,
                depth: source.depth + 1,
                parent_candidate_index: source.parent_candidate_index,
            };
            let reference =
                if let Some(reference) = discover_destination_reference(&inner_decoding.decoded) {
                    reference
                } else {
                    match classify_hostname_first_reference(&inner_decoding.decoded) {
                        (HostnameFirstClassification::Supported, Some(reference)) => reference,
                        (HostnameFirstClassification::Ambiguous, None) => {
                            self.add_hostname_first_ambiguity(&inner_source);
                            continue;
                        }
                        _ => continue,
                    }
                };
            let mut candidate_transformations = compound_transformations.clone();
            candidate_transformations.push(DestinationTransformation::CompoundQueryExtraction {
                parameter_position: position,
            });
            if !inner_decoding.steps.is_empty() {
                candidate_transformations.push(DestinationTransformation::PercentDecoding {
                    passes: inner_decoding.steps.len(),
                });
            }

            self.add_candidate_from_source(
                reference,
                decoding.clone(),
                candidate_transformations,
                inner_source,
            );

            if self.candidate_limit_reached {
                break;
            }
        }
    }

    fn add_candidate_from_source(
        &mut self,
        reference: DestinationReference,
        decoding: DecodingResult,
        transformations: Vec<DestinationTransformation>,
        source: CandidateSource,
    ) {
        self.add_candidate(
            reference,
            DestinationProvenance {
                top_level_parameter_index: source.top_level_parameter_index,
                source_parameter_index: source.source_parameter_index,
                source_parameter: source.source_parameter,
                depth: source.depth,
                parent_candidate_index: source.parent_candidate_index,
                decoding,
                transformations,
            },
        );
    }

    fn add_encoding_issue(&mut self, kind: DestinationEncodingIssueKind, source: &CandidateSource) {
        self.encoding_issues.push(DestinationEncodingIssue {
            kind,
            top_level_parameter_index: source.top_level_parameter_index,
            source_parameter_index: source.source_parameter_index,
            depth: source.depth,
            parent_candidate_index: source.parent_candidate_index,
        });
    }

    fn add_structure_observation(
        &mut self,
        kind: DestinationStructureKind,
        source: &CandidateSource,
    ) {
        let observation = DestinationStructureObservation {
            kind,
            top_level_parameter_index: source.top_level_parameter_index,
            source_parameter_index: source.source_parameter_index,
            depth: source.depth,
            parent_candidate_index: source.parent_candidate_index,
        };
        if !self.structure_observations.contains(&observation) {
            self.structure_observations.push(observation);
        }
    }

    fn add_hostname_first_ambiguity(&mut self, source: &CandidateSource) {
        self.add_structure_observation(
            DestinationStructureKind::AmbiguousHostnameFirstReference,
            source,
        );
        let issue = DestinationInterpretationIssue {
            kind: DestinationInterpretationIssueKind::AmbiguousHostnameFirstReference,
            top_level_parameter_index: source.top_level_parameter_index,
            source_parameter_index: source.source_parameter_index,
            depth: source.depth,
            parent_candidate_index: source.parent_candidate_index,
        };
        if !self.interpretation_issues.contains(&issue) {
            self.interpretation_issues.push(issue);
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
            DestinationReferenceKind::HostnameFirst => {
                format!("https://{}", reference.submitted)
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

                self.examine_source(
                    decoding,
                    CandidateSource {
                        top_level_parameter_index,
                        source_parameter_index: parameter_index,
                        source_parameter: parameter.name.clone(),
                        depth: next_depth,
                        parent_candidate_index: Some(pending.candidate_index),
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

        let mut coverage_issues = Vec::new();
        for issue in &self.encoding_issues {
            if !coverage_issues.contains(&issue.kind) {
                coverage_issues.push(issue.kind);
            }
        }
        let encoding_coverage = if coverage_issues.is_empty() {
            DestinationEncodingCoverage::Complete
        } else {
            DestinationEncodingCoverage::Incomplete {
                issues: coverage_issues,
            }
        };

        let mut interpretation_kinds = Vec::new();
        for issue in &self.interpretation_issues {
            if !interpretation_kinds.contains(&issue.kind) {
                interpretation_kinds.push(issue.kind);
            }
        }
        let interpretation_coverage = if interpretation_kinds.is_empty() {
            DestinationInterpretationCoverage::Complete
        } else {
            DestinationInterpretationCoverage::Partial {
                issues: interpretation_kinds,
            }
        };

        DestinationDiscovery {
            candidates: self.candidates,
            status,
            encoding_coverage,
            encoding_issues: self.encoding_issues,
            structure_observations: self.structure_observations,
            interpretation_coverage,
            interpretation_issues: self.interpretation_issues,
        }
    }
}

fn is_destination_parameter(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "target"
            | "url"
            | "redirect"
            | "redirect_uri"
            | "destination"
            | "dest"
            | "adurl"
            | "next"
            | "continue"
            | "return"
            | "return_to"
    )
}

fn form_decode_component(value: &str) -> String {
    url::form_urlencoded::parse(format!("{value}=").as_bytes())
        .next()
        .map(|(name, _)| name.into_owned())
        .unwrap_or_default()
}

fn reveals_encoded_query_delimiter(decoding: &DecodingResult) -> bool {
    if decoding.steps.is_empty() {
        return false;
    }
    let original = decoding.original.to_ascii_lowercase();
    (original.contains("%26") && decoding.decoded.contains('&'))
        || (original.contains("%3f") && decoding.decoded.contains('?'))
        || (original.contains("%3d") && decoding.decoded.contains('='))
}

fn classify_hostname_first_reference(
    value: &str,
) -> (HostnameFirstClassification, Option<DestinationReference>) {
    if value.trim() != value
        || value.contains(['\\', '@'])
        || value.chars().any(char::is_control)
        || value.starts_with('/')
        || value.contains("://")
    {
        return (HostnameFirstClassification::NotApplicable, None);
    }

    let Some(path_start) = value.find('/') else {
        return (HostnameFirstClassification::NotApplicable, None);
    };
    let authority = &value[..path_start];
    let path_and_more = &value[path_start..];
    if path_and_more.starts_with("//") {
        return if valid_hostname_first_authority(authority) {
            (HostnameFirstClassification::Ambiguous, None)
        } else {
            (HostnameFirstClassification::NotApplicable, None)
        };
    }
    if !valid_hostname_first_authority(authority) {
        return (HostnameFirstClassification::NotApplicable, None);
    }

    let parsing_context = format!("https://{value}");
    let Ok(parsed) = Url::parse(&parsing_context) else {
        return (HostnameFirstClassification::NotApplicable, None);
    };
    let Some(hostname) = parsed.host_str().map(str::to_string) else {
        return (HostnameFirstClassification::NotApplicable, None);
    };
    let mut normalized = parsed
        .as_str()
        .strip_prefix("https://")
        .unwrap_or(parsed.as_str())
        .to_string();
    if parsed.port().is_none()
        && let Some(port) = hostname_first_explicit_port(authority)
    {
        let authority_end = normalized.find(['/', '?', '#']).unwrap_or(normalized.len());
        normalized.insert_str(authority_end, &format!(":{port}"));
    }

    (
        HostnameFirstClassification::Supported,
        Some(DestinationReference {
            kind: DestinationReferenceKind::HostnameFirst,
            submitted: value.to_string(),
            normalized,
            hostname,
        }),
    )
}

fn valid_hostname_first_authority(authority: &str) -> bool {
    if authority.is_empty() || !authority.is_ascii() {
        return false;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    if let Some(port) = port
        && (port.is_empty()
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || port.parse::<u16>().is_err())
    {
        return false;
    }
    let host = host.strip_suffix('.').unwrap_or(host);
    let labels = host.split('.').collect::<Vec<_>>();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        })
}

fn hostname_first_explicit_port(authority: &str) -> Option<u16> {
    let (_, port) = authority.rsplit_once(':')?;
    port.parse().ok()
}

fn map_html_issue(issue: HtmlReferenceIssueKind) -> DestinationEncodingIssueKind {
    match issue {
        HtmlReferenceIssueKind::MalformedReference => {
            DestinationEncodingIssueKind::MalformedHtmlCharacterReference
        }
        HtmlReferenceIssueKind::UnsupportedNamedReference => {
            DestinationEncodingIssueKind::UnsupportedHtmlCharacterReference
        }
        HtmlReferenceIssueKind::InvalidUnicodeScalar => {
            DestinationEncodingIssueKind::InvalidHtmlUnicodeScalar
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
        assert!(discovery.encoding_coverage.is_complete());
    }

    #[test]
    fn discovers_padded_unpadded_and_url_safe_base64_candidates() {
        let input = concat!(
            "https://top.example/?padded=aHR0cHM6Ly9vdGhlci5leGFtcGxlLw==",
            "&unpadded=Ly9wcm90b2NvbC5leGFtcGxlL3BhdGg",
            "&urlsafe=aHR0cHM6Ly91cmxzYWZlLmV4YW1wbGUv8J-YgA"
        );

        let discovery = discover(input);

        assert_eq!(discovery.candidates.len(), 3);
        assert_eq!(discovery.candidates[0].reference.hostname, "other.example");
        assert_eq!(
            discovery.candidates[1].reference.kind,
            DestinationReferenceKind::ProtocolRelative
        );
        assert_eq!(
            discovery.candidates[2].reference.hostname,
            "urlsafe.example"
        );
        assert_eq!(
            discovery.candidates[0].provenance.transformations,
            vec![DestinationTransformation::Base64Decoding {
                alphabet: Base64Alphabet::Ambiguous,
                padding: Base64Padding::Padded,
            }]
        );
        assert_eq!(
            discovery.candidates[1].provenance.transformations,
            vec![DestinationTransformation::Base64Decoding {
                alphabet: Base64Alphabet::Ambiguous,
                padding: Base64Padding::Unpadded,
            }]
        );
        assert_eq!(
            discovery.candidates[2].provenance.transformations,
            vec![DestinationTransformation::Base64Decoding {
                alphabet: Base64Alphabet::UrlSafe,
                padding: Base64Padding::Unpadded,
            }]
        );
        assert!(discovery.is_complete());
    }

    #[test]
    fn discovers_decimal_hexadecimal_and_named_html_reference_candidates() {
        let decimal = encode("https&#58;&#47;&#47;decimal.example&#47;");
        let hexadecimal = encode("https&#x3A;&#x2F;&#X2f;hex.example&#x2f;");
        let named = encode("https&colon;&sol;&sol;named.example&sol;");
        let input =
            format!("https://top.example/?decimal={decimal}&hex={hexadecimal}&named={named}");

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 3);
        assert_eq!(
            discovery
                .candidates
                .iter()
                .map(|candidate| candidate.reference.hostname.as_str())
                .collect::<Vec<_>>(),
            ["decimal.example", "hex.example", "named.example"]
        );
        for candidate in &discovery.candidates {
            assert!(matches!(
                candidate.provenance.transformations.as_slice(),
                [
                    DestinationTransformation::PercentDecoding { passes: 1 },
                    DestinationTransformation::HtmlCharacterReferenceDecoding { .. }
                ]
            ));
        }
    }

    #[test]
    fn preserves_percent_then_base64_order_and_discovers_nested_encoded_candidates() {
        let encoded_parent =
            "aHR0cHM6Ly9taWRkbGUuZXhhbXBsZS8/bmV4dD1hSFIwY0hNNkx5OWphR2xzWkM1bGVHRnRjR3hsTHc9PQ==";
        let input = format!("https://top.example/?payload={}", encode(encoded_parent));

        let discovery = discover(&input);

        assert_eq!(discovery.candidates.len(), 2);
        assert_eq!(discovery.candidates[0].reference.hostname, "middle.example");
        assert_eq!(discovery.candidates[1].reference.hostname, "child.example");
        assert_eq!(discovery.candidates[1].provenance.depth, 1);
        assert_eq!(
            discovery.candidates[1].provenance.parent_candidate_index,
            Some(0)
        );
        assert!(matches!(
            discovery.candidates[0]
                .provenance
                .transformations
                .as_slice(),
            [
                DestinationTransformation::PercentDecoding { passes: 1 },
                DestinationTransformation::Base64Decoding { .. }
            ]
        ));
        assert!(matches!(
            discovery.candidates[1]
                .provenance
                .transformations
                .as_slice(),
            [DestinationTransformation::Base64Decoding { .. }]
        ));
    }

    #[test]
    fn reports_malformed_and_unsupported_secondary_encoding_coverage() {
        let unsupported_html = encode("https&colon;&sol;&sol;hidden.example&unknown;");
        let input = format!(
            "https://top.example/?base64=aHR0cHM6Ly9oaWRkZW4uZXhhbXBsZS8===&html={unsupported_html}"
        );

        let discovery = discover(&input);

        assert!(discovery.candidates.is_empty());
        assert!(discovery.status.is_complete());
        assert!(!discovery.encoding_coverage.is_complete());
        assert_eq!(
            discovery.encoding_coverage.issues(),
            &[
                DestinationEncodingIssueKind::MalformedBase64,
                DestinationEncodingIssueKind::UnsupportedHtmlCharacterReference,
            ]
        );
        assert_eq!(discovery.encoding_issues.len(), 2);
        assert!(!discovery.is_complete());
    }

    #[test]
    fn reports_secondary_decoded_output_limit_as_incomplete() {
        let input = format!("https://top.example/?payload={}", "A".repeat(12 * 1024));

        let discovery = discover(&input);

        assert!(discovery.candidates.is_empty());
        assert!(
            discovery
                .status
                .limits()
                .contains(&DestinationDiscoveryLimit::SecondaryDecodedOutputLength)
        );
        assert!(!discovery.is_complete());
    }

    #[test]
    fn benign_encoded_text_binary_and_ambiguous_strings_do_not_create_candidates() {
        let discovery = discover(concat!(
            "https://top.example/?text=aGVsbG8gd29ybGQ=",
            "&binary=________",
            "&identifier=ABCDEF12",
            "&relative=L2xvY2FsL3BhdGg="
        ));

        assert!(discovery.candidates.is_empty());
        assert!(discovery.is_complete());
    }

    #[test]
    fn rejects_control_characters_after_html_decoding() {
        let value = encode("https&colon;&sol;&sol;example.test&#10;path");
        let discovery = discover(&format!("https://top.example/?value={value}"));

        assert!(discovery.candidates.is_empty());
        assert!(discovery.is_complete());
    }

    #[test]
    fn recognizes_hostname_first_references_only_in_destination_contexts() {
        let supported = discover(
            "https://top.example/?target=tracking.example%2Fpath&label=ignored.example%2Fpath",
        );

        assert_eq!(supported.candidates.len(), 1);
        let candidate = &supported.candidates[0];
        assert_eq!(
            candidate.reference.kind,
            DestinationReferenceKind::HostnameFirst
        );
        assert_eq!(candidate.reference.submitted, "tracking.example/path");
        assert_eq!(candidate.reference.normalized, "tracking.example/path");
        assert_eq!(candidate.reference.hostname, "tracking.example");
        assert_eq!(candidate.provenance.source_parameter, "target");
    }

    #[test]
    fn preserves_trailing_dot_and_explicit_port_for_hostname_first_references() {
        let discovery = discover(
            "https://top.example/?target=tracking.example.%3A443%2Fpath&url=edge.example%3A8443%2Fnext",
        );

        assert_eq!(discovery.candidates.len(), 2);
        assert_eq!(
            discovery.candidates[0].reference.submitted,
            "tracking.example.:443/path"
        );
        assert_eq!(
            discovery.candidates[0].reference.normalized,
            "tracking.example.:443/path"
        );
        assert_eq!(
            discovery.candidates[0].reference.hostname,
            "tracking.example."
        );
        assert_eq!(
            discovery.candidates[1].reference.normalized,
            "edge.example:8443/next"
        );
    }

    #[test]
    fn rejects_malformed_hostname_first_values_and_records_double_slash_ambiguity() {
        let discovery = discover(concat!(
            "https://top.example/?target=tracking.example%2F%2Fpath",
            "&url=user%40example.test%2Fpath",
            "&redirect=bad_example.test%2Fpath",
            "&next=example.test%3A99999%2Fpath",
            "&dest=example.test%5Cpath"
        ));

        assert!(discovery.candidates.is_empty());
        assert_eq!(discovery.interpretation_issues.len(), 1);
        assert_eq!(
            discovery.interpretation_coverage.issues(),
            &[DestinationInterpretationIssueKind::AmbiguousHostnameFirstReference]
        );
        assert!(!discovery.is_complete());
    }

    #[test]
    fn compound_encoded_destination_example_is_useful_without_promoting_ambiguity() {
        let discovery = discover(concat!(
            "https://social.example/link/v2?aid=1988&scene=bio_url&target=",
            "tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=",
            "%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html",
            "#synthetic-fragment"
        ));

        assert_eq!(discovery.candidates.len(), 1);
        let candidate = &discovery.candidates[0];
        assert_eq!(
            candidate.reference.kind,
            DestinationReferenceKind::ProtocolRelative
        );
        assert_eq!(
            candidate.reference.submitted,
            "//storage.example/landing.html"
        );
        assert_eq!(
            candidate.reference.normalized,
            "//storage.example/landing.html"
        );
        assert_eq!(candidate.provenance.source_parameter, "adurl");
        assert_eq!(candidate.provenance.top_level_parameter_index, 2);
        assert_eq!(candidate.provenance.source_parameter_index, 1);
        assert_eq!(candidate.provenance.depth, 1);
        assert_eq!(
            candidate.provenance.transformations,
            vec![
                DestinationTransformation::PercentDecoding { passes: 1 },
                DestinationTransformation::HtmlCharacterReferenceDecoding { reference_count: 2 },
                DestinationTransformation::CompoundQueryExtraction {
                    parameter_position: 1,
                },
            ]
        );
        assert!(discovery.structure_observations.iter().any(|observation| {
            observation.kind == DestinationStructureKind::CompoundQueryValue
        }));
        assert!(discovery.structure_observations.iter().any(|observation| {
            observation.kind == DestinationStructureKind::EncodedQueryDelimiter
        }));
        assert!(discovery.structure_observations.iter().any(|observation| {
            observation.kind == DestinationStructureKind::AmbiguousHostnameFirstReference
        }));
        assert!(!discovery.is_complete());
    }

    #[test]
    fn compound_query_extraction_is_bounded() {
        let query = (0..=MAX_COMPOUND_QUERY_PARAMETERS)
            .map(|index| format!("adurl{index}=value"))
            .collect::<Vec<_>>()
            .join("&");
        let input = format!(
            "https://top.example/?target={}",
            encode(&format!("tracking.example//path?{query}"))
        );
        let discovery = discover(&input);

        assert!(
            discovery
                .status
                .limits()
                .contains(&DestinationDiscoveryLimit::MaximumCompoundParameterCount)
        );
    }

    #[test]
    fn compound_inner_destination_context_supports_hostname_first_references() {
        let discovery = discover(
            "https://top.example/?target=tracking.example%2F%2Fpath%3Fadurl%3Dinner.example%2Flanding",
        );

        assert_eq!(discovery.candidates.len(), 1);
        assert_eq!(
            discovery.candidates[0].reference.kind,
            DestinationReferenceKind::HostnameFirst
        );
        assert_eq!(
            discovery.candidates[0].reference.submitted,
            "inner.example/landing"
        );
        assert_eq!(discovery.candidates[0].provenance.source_parameter, "adurl");
    }
}
