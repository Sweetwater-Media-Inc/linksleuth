//! Schema-versioned reports for application integrations.
//!
//! This module keeps privacy-reduced observations separate from deliberately
//! requested evidence. Its schema version is independent of the crate version;
//! incompatible consumer-contract changes require a schema-version change.
//!
//! ```
//! use linksleuth_core::analysis::analyze_url;
//! use linksleuth_core::consumer::{
//!     create_consumer_report, ConsumerReportOptions, CONSUMER_REPORT_SCHEMA_VERSION,
//! };
//!
//! let analysis = analyze_url(
//!     "https://shop.example/?target=https%3A%2F%2Fvendor.example%2Fproducts",
//! )?;
//! let private = create_consumer_report(&analysis, ConsumerReportOptions::default());
//! assert_eq!(private.schema_version(), CONSUMER_REPORT_SCHEMA_VERSION);
//! assert_eq!(private.hostname(), None);
//! assert!(private.evidence().is_empty());
//!
//! let detailed = create_consumer_report(
//!     &analysis,
//!     ConsumerReportOptions::new().with_detailed_evidence(),
//! );
//! assert!(!detailed.evidence().is_empty());
//! # Ok::<(), linksleuth_core::parser::UrlError>(())
//! ```

use std::fmt;

use crate::analysis::AnalysisReport;
use crate::decoder::{DecodeError, DecodingStatus};
use crate::destinations::{
    DestinationDiscoveryLimit, DestinationEncodingIssueKind, DestinationInterpretationIssueKind,
    DestinationReferenceKind, DestinationScheme, DestinationStructureKind,
    DestinationTransformation,
};
use crate::encodings::{Base64Alphabet, Base64Padding};
use crate::normalization::{
    NormalizationKind, OriginalPathAnalysis, OriginalPathUnsupportedReason,
};

/// Version of the consumer-report contract, independent of the crate version.
pub const CONSUMER_REPORT_SCHEMA_VERSION: u32 = 1;

/// Version of the explicit destination-disclosure contract.
///
/// This contract is independent of the privacy-reduced consumer report. It is
/// created only after a caller selects a disclosure level.
pub const DESTINATION_DISCLOSURE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumerReportOptions {
    include_hostname: bool,
    include_detailed_evidence: bool,
}

impl ConsumerReportOptions {
    pub const fn new() -> Self {
        Self {
            include_hostname: false,
            include_detailed_evidence: false,
        }
    }

    pub const fn with_hostname(mut self) -> Self {
        self.include_hostname = true;
        self
    }

    pub const fn with_detailed_evidence(mut self) -> Self {
        self.include_detailed_evidence = true;
        self
    }

    pub const fn includes_hostname(self) -> bool {
        self.include_hostname
    }

    pub const fn includes_detailed_evidence(self) -> bool {
        self.include_detailed_evidence
    }
}

impl Default for ConsumerReportOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerDisclosureClass {
    PrivacyReduced,
    OriginalHostname,
    DetailedEvidence,
    OriginalHostnameAndDetailedEvidence,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerObservationKind {
    EncodedParentDirectoryNormalization,
    PotentialAbsoluteDestination,
    PotentialProtocolRelativeReference,
    PotentialHostnameFirstReference,
    NestedDestinationCandidate,
    Base64DerivedDestination,
    HtmlReferenceDerivedDestination,
    MultiplePercentDecodingTransformations,
    EncodedQueryDelimiter,
    CompoundQueryValue,
    AmbiguousHostnameFirstReference,
    MalformedPercentEncoding,
    PercentDecodingPassLimit,
    UnsupportedOriginalPathEvidence,
    IncompleteDestinationDiscovery,
    IncompleteEncodingInterpretation,
}

impl ConsumerObservationKind {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::EncodedParentDirectoryNormalization => "normalization.encoded-parent-directory",
            Self::PotentialAbsoluteDestination => "destination.absolute-candidate",
            Self::PotentialProtocolRelativeReference => "destination.protocol-relative-candidate",
            Self::PotentialHostnameFirstReference => "destination.hostname-first-candidate",
            Self::NestedDestinationCandidate => "destination.nested-candidate",
            Self::Base64DerivedDestination => "destination.base64-derived-candidate",
            Self::HtmlReferenceDerivedDestination => "destination.html-reference-derived-candidate",
            Self::MultiplePercentDecodingTransformations => {
                "encoding.multiple-percent-transformations"
            }
            Self::EncodedQueryDelimiter => "structure.encoded-query-delimiter",
            Self::CompoundQueryValue => "structure.compound-query-value",
            Self::AmbiguousHostnameFirstReference => "structure.hostname-first-ambiguous",
            Self::MalformedPercentEncoding => "decoding.malformed-percent-encoding",
            Self::PercentDecodingPassLimit => "decoding.percent-pass-limit",
            Self::UnsupportedOriginalPathEvidence => "coverage.original-path-unsupported",
            Self::IncompleteDestinationDiscovery => "coverage.destination-discovery-incomplete",
            Self::IncompleteEncodingInterpretation => "coverage.secondary-encoding-incomplete",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::EncodedParentDirectoryNormalization => "Encoded parent-directory normalization",
            Self::PotentialAbsoluteDestination => "This link contains another web address",
            Self::PotentialProtocolRelativeReference => {
                "This link contains a web address without a stated scheme"
            }
            Self::PotentialHostnameFirstReference => {
                "This link contains a hostname-first web address"
            }
            Self::NestedDestinationCandidate => "A web address appears inside another value",
            Self::Base64DerivedDestination => "An encoded value contains a web address",
            Self::HtmlReferenceDerivedDestination => "HTML-style encoding contains a web address",
            Self::MultiplePercentDecodingTransformations => "A value used multiple encoding layers",
            Self::EncodedQueryDelimiter => "Encoded query separators were found",
            Self::CompoundQueryValue => "A parameter contains its own query structure",
            Self::AmbiguousHostnameFirstReference => {
                "A hostname-like reference could not be classified confidently"
            }
            Self::MalformedPercentEncoding => "Malformed percent encoding",
            Self::PercentDecodingPassLimit => "Percent-decoding limit reached",
            Self::UnsupportedOriginalPathEvidence => "Original path evidence unsupported",
            Self::IncompleteDestinationDiscovery => "Destination discovery incomplete",
            Self::IncompleteEncodingInterpretation => "Encoding interpretation incomplete",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::EncodedParentDirectoryNormalization => {
                "The submitted path included encoded parent-directory syntax that the URL parser normalized. This structural observation does not establish intent."
            }
            Self::PotentialAbsoluteDestination => {
                "A complete URL was found inside this link. Some websites use embedded addresses to guide navigation, but Clue has not visited either website or verified a redirect."
            }
            Self::PotentialProtocolRelativeReference => {
                "A complete protocol-relative URL reference appeared in a supported query value. Its destination scheme and any navigation remain unverified."
            }
            Self::PotentialHostnameFirstReference => {
                "A complete hostname and path appeared in a supported destination parameter. No HTTP or HTTPS scheme was inferred, and navigation remains unverified."
            }
            Self::NestedDestinationCandidate => {
                "A potential destination appeared inside a query parameter of another candidate. This structure does not establish a redirect chain."
            }
            Self::Base64DerivedDestination => {
                "A complete destination reference appeared after bounded Base64 decoding. Encoding alone does not establish intent or navigation."
            }
            Self::HtmlReferenceDerivedDestination => {
                "A complete destination reference appeared after interpreting supported HTML character references. Navigation remains unverified."
            }
            Self::MultiplePercentDecodingTransformations => {
                "A query value changed during more than one bounded percent-decoding pass. Layered encoding is structural evidence and does not establish intent."
            }
            Self::EncodedQueryDelimiter => {
                "Percent decoding revealed query separators such as an ampersand, question mark, or equals sign. This does not prove the value is used for navigation."
            }
            Self::CompoundQueryValue => {
                "A supported destination parameter contained a bounded inner query structure. Clue examined only recognized inner parameter forms and did not follow them."
            }
            Self::AmbiguousHostnameFirstReference => {
                "A destination-bearing value began like a hostname but used an ambiguous authority and path boundary. Clue did not repair it or treat it as a verified destination."
            }
            Self::MalformedPercentEncoding => {
                "A query value contained malformed percent encoding. LinkSleuth preserved the submitted text rather than guessing a replacement."
            }
            Self::PercentDecodingPassLimit => {
                "A query value reached the bounded percent-decoding pass limit, so additional encoded content may remain."
            }
            Self::UnsupportedOriginalPathEvidence => {
                "The submitted URL spelling did not provide a confidently supported original-path boundary. No path-normalization conclusion is available."
            }
            Self::IncompleteDestinationDiscovery => {
                "A destination traversal resource limit was reached. Additional supported candidates may remain undiscovered."
            }
            Self::IncompleteEncodingInterpretation => {
                "A secondary encoding value was malformed, unsupported, or resource-limited. Additional supported evidence may remain uninterpreted."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerEvidenceReference {
    identifier: String,
}

impl ConsumerEvidenceReference {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerObservation {
    kind: ConsumerObservationKind,
    identifier: String,
    title: String,
    explanation: String,
    occurrence_count: usize,
    evidence_references: Vec<ConsumerEvidenceReference>,
}

impl ConsumerObservation {
    pub const fn kind(&self) -> ConsumerObservationKind {
        self.kind
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub const fn occurrence_count(&self) -> usize {
        self.occurrence_count
    }

    pub fn evidence_references(&self) -> &[ConsumerEvidenceReference] {
        &self.evidence_references
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerCompletenessStatus {
    CompleteWithinSupportedScope,
    PartiallyExamined,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerAnalysisStage {
    OriginalPath,
    PercentDecoding,
    DestinationTraversal,
    SecondaryEncoding,
}

impl ConsumerAnalysisStage {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::OriginalPath => "original-path",
            Self::PercentDecoding => "percent-decoding",
            Self::DestinationTraversal => "destination-traversal",
            Self::SecondaryEncoding => "secondary-encoding",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerStageStatus {
    CompleteWithinSupportedScope,
    PartiallyExamined,
    UnsupportedEvidence,
    ResourceLimited,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerCompletenessReason {
    identifier: String,
    explanation: String,
}

impl ConsumerCompletenessReason {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerStageCompleteness {
    stage: ConsumerAnalysisStage,
    status: ConsumerStageStatus,
    reasons: Vec<ConsumerCompletenessReason>,
}

impl ConsumerStageCompleteness {
    pub const fn stage(&self) -> ConsumerAnalysisStage {
        self.stage
    }

    pub const fn status(&self) -> ConsumerStageStatus {
        self.status
    }

    pub fn reasons(&self) -> &[ConsumerCompletenessReason] {
        &self.reasons
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerCompleteness {
    status: ConsumerCompletenessStatus,
    stages: Vec<ConsumerStageCompleteness>,
}

impl ConsumerCompleteness {
    pub const fn status(&self) -> ConsumerCompletenessStatus {
        self.status
    }

    pub fn stages(&self) -> &[ConsumerStageCompleteness] {
        &self.stages
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerEvidenceKind {
    Normalization,
    DestinationCandidate,
    QueryParameter,
    OriginalPathCoverage,
    EncodingIssue,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerDestinationKind {
    AbsoluteHttp,
    AbsoluteHttps,
    ProtocolRelative,
    HostnameFirst,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumerTransformation {
    PercentDecoding {
        passes: usize,
    },
    Base64Decoding {
        alphabet_identifier: String,
        padding_identifier: String,
    },
    HtmlCharacterReferenceDecoding {
        reference_count: usize,
    },
    CompoundQueryExtraction {
        parameter_position: usize,
    },
}

/// Amount of destination evidence deliberately requested by a consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationDisclosureLevel {
    Hostnames,
    Detailed,
}

impl DestinationDisclosureLevel {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Hostnames => "hostnames",
            Self::Detailed => "detailed",
        }
    }
}

/// Detail available only for the explicit detailed disclosure level.
#[derive(Clone, PartialEq, Eq)]
pub struct DestinationDisclosureDetail {
    submitted_reference_without_credentials_or_fragment: String,
    normalized_reference_without_credentials_or_fragment: String,
    source_parameter: String,
    top_level_parameter_position: usize,
    source_parameter_position: usize,
    transformations: Vec<ConsumerTransformation>,
}

impl DestinationDisclosureDetail {
    pub fn submitted_reference_without_credentials_or_fragment(&self) -> &str {
        &self.submitted_reference_without_credentials_or_fragment
    }

    pub fn normalized_reference_without_credentials_or_fragment(&self) -> &str {
        &self.normalized_reference_without_credentials_or_fragment
    }

    pub fn source_parameter(&self) -> &str {
        &self.source_parameter
    }

    pub const fn top_level_parameter_position(&self) -> usize {
        self.top_level_parameter_position
    }

    pub const fn source_parameter_position(&self) -> usize {
        self.source_parameter_position
    }

    pub fn transformations(&self) -> &[ConsumerTransformation] {
        &self.transformations
    }
}

impl fmt::Debug for DestinationDisclosureDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED DESTINATION DETAIL]")
    }
}

/// One supported destination candidate in source order.
#[derive(Clone, PartialEq, Eq)]
pub struct DestinationDisclosureCandidate {
    identifier: String,
    destination_kind: ConsumerDestinationKind,
    hostname: String,
    depth: usize,
    parent_candidate_identifier: Option<String>,
    detail: Option<DestinationDisclosureDetail>,
}

impl DestinationDisclosureCandidate {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub const fn destination_kind(&self) -> ConsumerDestinationKind {
        self.destination_kind
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }

    pub fn parent_candidate_identifier(&self) -> Option<&str> {
        self.parent_candidate_identifier.as_deref()
    }

    pub const fn detail(&self) -> Option<&DestinationDisclosureDetail> {
        self.detail.as_ref()
    }
}

impl fmt::Debug for DestinationDisclosureCandidate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DestinationDisclosureCandidate")
            .field("identifier", &self.identifier)
            .field("destination_kind", &self.destination_kind)
            .field("hostname", &"[REDACTED]")
            .field("depth", &self.depth)
            .field(
                "parent_candidate_identifier",
                &self.parent_candidate_identifier,
            )
            .field("has_detail", &self.detail.is_some())
            .finish()
    }
}

/// Supported destination-like material that was deliberately not classified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationDisclosureAmbiguity {
    identifier: String,
    explanation: String,
    depth: usize,
    parent_candidate_identifier: Option<String>,
    top_level_parameter_position: Option<usize>,
    source_parameter_position: Option<usize>,
}

impl DestinationDisclosureAmbiguity {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub const fn depth(&self) -> usize {
        self.depth
    }

    pub fn parent_candidate_identifier(&self) -> Option<&str> {
        self.parent_candidate_identifier.as_deref()
    }

    pub const fn top_level_parameter_position(&self) -> Option<usize> {
        self.top_level_parameter_position
    }

    pub const fn source_parameter_position(&self) -> Option<usize> {
        self.source_parameter_position
    }
}

/// Bounded, purpose-specific destination disclosure.
#[derive(Clone, PartialEq, Eq)]
pub struct DestinationDisclosure {
    schema_version: u32,
    level: DestinationDisclosureLevel,
    candidates: Vec<DestinationDisclosureCandidate>,
    ambiguities: Vec<DestinationDisclosureAmbiguity>,
}

impl DestinationDisclosure {
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub const fn level(&self) -> DestinationDisclosureLevel {
        self.level
    }

    pub fn candidates(&self) -> &[DestinationDisclosureCandidate] {
        &self.candidates
    }

    pub fn ambiguities(&self) -> &[DestinationDisclosureAmbiguity] {
        &self.ambiguities
    }
}

impl fmt::Debug for DestinationDisclosure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DestinationDisclosure")
            .field("schema_version", &self.schema_version)
            .field("level", &self.level)
            .field("candidate_count", &self.candidates.len())
            .field("ambiguity_count", &self.ambiguities.len())
            .finish()
    }
}

#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum ConsumerEvidenceDetail {
    Normalization {
        original_path: String,
        normalized_path: String,
    },
    DestinationCandidate {
        destination_kind: ConsumerDestinationKind,
        submitted_reference: String,
        normalized_reference: String,
        hostname: String,
        source_parameter: String,
        top_level_parameter_position: usize,
        source_parameter_position: usize,
        depth: usize,
        parent_evidence_identifier: Option<String>,
        transformations: Vec<ConsumerTransformation>,
    },
    QueryParameter {
        name: String,
        raw_value: String,
        position: usize,
    },
    OriginalPathCoverage {
        reason_identifier: String,
    },
    EncodingIssue {
        reason_identifier: String,
        top_level_parameter_position: usize,
        source_parameter_position: usize,
        depth: usize,
        parent_evidence_identifier: Option<String>,
    },
}

impl fmt::Debug for ConsumerEvidenceDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED DETAILED EVIDENCE]")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConsumerEvidence {
    identifier: String,
    kind: ConsumerEvidenceKind,
    detail: ConsumerEvidenceDetail,
}

impl ConsumerEvidence {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub const fn kind(&self) -> ConsumerEvidenceKind {
        self.kind
    }

    pub const fn detail(&self) -> &ConsumerEvidenceDetail {
        &self.detail
    }
}

impl fmt::Debug for ConsumerEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConsumerEvidence")
            .field("identifier", &self.identifier)
            .field("kind", &self.kind)
            .field("detail", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConsumerReport {
    schema_version: u32,
    disclosure: ConsumerDisclosureClass,
    hostname: Option<String>,
    completeness: ConsumerCompleteness,
    observations: Vec<ConsumerObservation>,
    evidence: Vec<ConsumerEvidence>,
}

impl ConsumerReport {
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub const fn disclosure(&self) -> ConsumerDisclosureClass {
        self.disclosure
    }

    pub fn hostname(&self) -> Option<&str> {
        self.hostname.as_deref()
    }

    pub const fn completeness(&self) -> &ConsumerCompleteness {
        &self.completeness
    }

    pub fn observations(&self) -> &[ConsumerObservation] {
        &self.observations
    }

    pub fn evidence(&self) -> &[ConsumerEvidence] {
        &self.evidence
    }
}

impl fmt::Debug for ConsumerReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConsumerReport")
            .field("schema_version", &self.schema_version)
            .field("disclosure", &self.disclosure)
            .field("hostname", &self.hostname.as_ref().map(|_| "[REDACTED]"))
            .field("completeness", &self.completeness)
            .field("observations", &self.observations)
            .field("evidence_count", &self.evidence.len())
            .finish()
    }
}

/// Creates a schema-versioned consumer report without changing the detailed analysis.
///
/// The default options disclose neither the original hostname nor detailed
/// evidence. These controls are independent and must be enabled deliberately.
pub fn create_consumer_report(
    report: &AnalysisReport,
    options: ConsumerReportOptions,
) -> ConsumerReport {
    let include_evidence = options.includes_detailed_evidence();
    let evidence = if include_evidence {
        build_evidence(report)
    } else {
        Vec::new()
    };
    let observations = build_observations(report, include_evidence);

    ConsumerReport {
        schema_version: CONSUMER_REPORT_SCHEMA_VERSION,
        disclosure: disclosure_class(options),
        hostname: options
            .includes_hostname()
            .then(|| report.url.hostname.clone()),
        completeness: build_completeness(report),
        observations,
        evidence,
    }
}

/// Creates an explicit, bounded destination disclosure in source order.
///
/// Hostname disclosure omits source names, candidate references, and
/// transformations. Detailed disclosure adds only destination-specific
/// evidence and removes literal fragment text from both candidate
/// representations. Neither level contains the submitted top-level URL.
pub fn create_destination_disclosure(
    report: &AnalysisReport,
    level: DestinationDisclosureLevel,
) -> DestinationDisclosure {
    let include_details = level == DestinationDisclosureLevel::Detailed;
    let candidates = report
        .destination_discovery
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| DestinationDisclosureCandidate {
            identifier: evidence_reference("candidate", index).identifier,
            destination_kind: consumer_destination_kind(candidate.reference.kind),
            hostname: candidate.reference.hostname.clone(),
            depth: candidate.provenance.depth,
            parent_candidate_identifier: candidate
                .provenance
                .parent_candidate_index
                .map(|parent| evidence_reference("candidate", parent).identifier),
            detail: include_details.then(|| DestinationDisclosureDetail {
                submitted_reference_without_credentials_or_fragment: privacy_scoped_reference(
                    &candidate.reference.submitted,
                ),
                normalized_reference_without_credentials_or_fragment: privacy_scoped_reference(
                    &candidate.reference.normalized,
                ),
                source_parameter: candidate.provenance.source_parameter.clone(),
                top_level_parameter_position: candidate.provenance.top_level_parameter_index + 1,
                source_parameter_position: candidate.provenance.source_parameter_index + 1,
                transformations: candidate
                    .provenance
                    .transformations
                    .iter()
                    .map(consumer_transformation)
                    .collect(),
            }),
        })
        .collect();

    let ambiguities = report
        .destination_discovery
        .interpretation_issues
        .iter()
        .enumerate()
        .map(|(index, issue)| DestinationDisclosureAmbiguity {
            identifier: format!("ambiguity-{}", index + 1),
            explanation: issue.kind.explanation().to_string(),
            depth: issue.depth,
            parent_candidate_identifier: issue
                .parent_candidate_index
                .map(|parent| evidence_reference("candidate", parent).identifier),
            top_level_parameter_position: include_details
                .then_some(issue.top_level_parameter_index + 1),
            source_parameter_position: include_details.then_some(issue.source_parameter_index + 1),
        })
        .collect();

    DestinationDisclosure {
        schema_version: DESTINATION_DISCLOSURE_SCHEMA_VERSION,
        level,
        candidates,
        ambiguities,
    }
}

fn consumer_destination_kind(kind: DestinationReferenceKind) -> ConsumerDestinationKind {
    match kind {
        DestinationReferenceKind::Absolute {
            scheme: DestinationScheme::Http,
        } => ConsumerDestinationKind::AbsoluteHttp,
        DestinationReferenceKind::Absolute {
            scheme: DestinationScheme::Https,
        } => ConsumerDestinationKind::AbsoluteHttps,
        DestinationReferenceKind::ProtocolRelative => ConsumerDestinationKind::ProtocolRelative,
        DestinationReferenceKind::HostnameFirst => ConsumerDestinationKind::HostnameFirst,
    }
}

fn privacy_scoped_reference(reference: &str) -> String {
    let without_fragment = reference
        .split_once('#')
        .map_or(reference, |(before, _)| before);
    let authority_start = if without_fragment.starts_with("//") {
        Some(2)
    } else {
        without_fragment.find("://").map(|separator| separator + 3)
    };
    let Some(authority_start) = authority_start else {
        return without_fragment.to_string();
    };
    let authority_end = without_fragment[authority_start..]
        .find(['/', '?'])
        .map_or(without_fragment.len(), |offset| authority_start + offset);
    let authority = &without_fragment[authority_start..authority_end];
    let Some(user_info_end) = authority.rfind('@') else {
        return without_fragment.to_string();
    };

    format!(
        "{}{}",
        &without_fragment[..authority_start],
        &without_fragment[authority_start + user_info_end + 1..]
    )
}

fn disclosure_class(options: ConsumerReportOptions) -> ConsumerDisclosureClass {
    match (
        options.includes_hostname(),
        options.includes_detailed_evidence(),
    ) {
        (false, false) => ConsumerDisclosureClass::PrivacyReduced,
        (true, false) => ConsumerDisclosureClass::OriginalHostname,
        (false, true) => ConsumerDisclosureClass::DetailedEvidence,
        (true, true) => ConsumerDisclosureClass::OriginalHostnameAndDetailedEvidence,
    }
}

fn build_observations(report: &AnalysisReport, include_evidence: bool) -> Vec<ConsumerObservation> {
    let candidates = &report.destination_discovery.candidates;
    let mut observations = Vec::new();

    let normalization_indices = report
        .normalization_findings
        .iter()
        .enumerate()
        .filter_map(|(index, finding)| {
            (finding.kind == NormalizationKind::EncodedDotSegment).then_some(index)
        })
        .collect::<Vec<_>>();
    push_observation(
        &mut observations,
        ConsumerObservationKind::EncodedParentDirectoryNormalization,
        normalization_indices.len(),
        evidence_references(include_evidence, "normalization", &normalization_indices),
    );

    let absolute_indices = candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            matches!(
                candidate.reference.kind,
                DestinationReferenceKind::Absolute { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    push_observation(
        &mut observations,
        ConsumerObservationKind::PotentialAbsoluteDestination,
        absolute_indices.len(),
        evidence_references(include_evidence, "candidate", &absolute_indices),
    );

    let protocol_indices = candidate_indices(report, |candidate| {
        candidate.reference.kind == DestinationReferenceKind::ProtocolRelative
    });
    push_observation(
        &mut observations,
        ConsumerObservationKind::PotentialProtocolRelativeReference,
        protocol_indices.len(),
        evidence_references(include_evidence, "candidate", &protocol_indices),
    );

    let hostname_first_indices = candidate_indices(report, |candidate| {
        candidate.reference.kind == DestinationReferenceKind::HostnameFirst
    });
    push_observation(
        &mut observations,
        ConsumerObservationKind::PotentialHostnameFirstReference,
        hostname_first_indices.len(),
        evidence_references(include_evidence, "candidate", &hostname_first_indices),
    );

    let nested_indices = candidate_indices(report, |candidate| candidate.provenance.depth > 0);
    push_observation(
        &mut observations,
        ConsumerObservationKind::NestedDestinationCandidate,
        nested_indices.len(),
        evidence_references(include_evidence, "candidate", &nested_indices),
    );

    let base64_indices = candidate_indices(report, |candidate| {
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
    });
    push_observation(
        &mut observations,
        ConsumerObservationKind::Base64DerivedDestination,
        base64_indices.len(),
        evidence_references(include_evidence, "candidate", &base64_indices),
    );

    let html_indices = candidate_indices(report, |candidate| {
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
    });
    push_observation(
        &mut observations,
        ConsumerObservationKind::HtmlReferenceDerivedDestination,
        html_indices.len(),
        evidence_references(include_evidence, "candidate", &html_indices),
    );

    for (structure_kind, consumer_kind) in [
        (
            DestinationStructureKind::MultiplePercentDecodingTransformations,
            ConsumerObservationKind::MultiplePercentDecodingTransformations,
        ),
        (
            DestinationStructureKind::EncodedQueryDelimiter,
            ConsumerObservationKind::EncodedQueryDelimiter,
        ),
        (
            DestinationStructureKind::CompoundQueryValue,
            ConsumerObservationKind::CompoundQueryValue,
        ),
        (
            DestinationStructureKind::AmbiguousHostnameFirstReference,
            ConsumerObservationKind::AmbiguousHostnameFirstReference,
        ),
    ] {
        let occurrence_count = report
            .destination_discovery
            .structure_observations
            .iter()
            .filter(|observation| observation.kind == structure_kind)
            .count();
        push_observation(
            &mut observations,
            consumer_kind,
            occurrence_count,
            Vec::new(),
        );
    }

    let malformed_percent_indices = report
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            parameter
                .decoding
                .as_ref()
                .is_ok_and(|decoding| !decoding.warnings.is_empty())
                .then_some(index)
        })
        .collect::<Vec<_>>();
    push_observation(
        &mut observations,
        ConsumerObservationKind::MalformedPercentEncoding,
        malformed_percent_indices.len(),
        evidence_references(include_evidence, "parameter", &malformed_percent_indices),
    );

    let pass_limit_indices = report
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            parameter
                .decoding
                .as_ref()
                .is_ok_and(|decoding| decoding.status == DecodingStatus::PassLimitReached)
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let nested_pass_limit = report
        .destination_discovery
        .status
        .limits()
        .contains(&DestinationDiscoveryLimit::DecodingPassLimit);
    let pass_limit_occurrence_count = pass_limit_indices.len().max(usize::from(nested_pass_limit));
    push_observation(
        &mut observations,
        ConsumerObservationKind::PercentDecodingPassLimit,
        pass_limit_occurrence_count,
        evidence_references(include_evidence, "parameter", &pass_limit_indices),
    );

    if matches!(
        report.original_path_analysis,
        OriginalPathAnalysis::Unsupported { .. }
    ) {
        push_observation(
            &mut observations,
            ConsumerObservationKind::UnsupportedOriginalPathEvidence,
            1,
            include_evidence
                .then(|| evidence_reference("path-coverage", 0))
                .into_iter()
                .collect(),
        );
    }

    let traversal_limit_count = report
        .destination_discovery
        .status
        .limits()
        .iter()
        .filter(|limit| is_traversal_limit(**limit))
        .count();
    push_observation(
        &mut observations,
        ConsumerObservationKind::IncompleteDestinationDiscovery,
        traversal_limit_count,
        Vec::new(),
    );

    let secondary_limit_count = report
        .destination_discovery
        .status
        .limits()
        .iter()
        .filter(|limit| is_secondary_limit(**limit))
        .count();
    let encoding_issue_indices =
        (0..report.destination_discovery.encoding_issues.len()).collect::<Vec<_>>();
    push_observation(
        &mut observations,
        ConsumerObservationKind::IncompleteEncodingInterpretation,
        secondary_limit_count + encoding_issue_indices.len(),
        evidence_references(include_evidence, "encoding-issue", &encoding_issue_indices),
    );

    observations
}

fn push_observation(
    observations: &mut Vec<ConsumerObservation>,
    kind: ConsumerObservationKind,
    occurrence_count: usize,
    evidence_references: Vec<ConsumerEvidenceReference>,
) {
    if occurrence_count == 0 {
        return;
    }

    observations.push(ConsumerObservation {
        kind,
        identifier: kind.identifier().to_string(),
        title: kind.title().to_string(),
        explanation: kind.explanation().to_string(),
        occurrence_count,
        evidence_references,
    });
}

fn candidate_indices(
    report: &AnalysisReport,
    predicate: impl Fn(&crate::destinations::DiscoveredDestination) -> bool,
) -> Vec<usize> {
    report
        .destination_discovery
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| predicate(candidate).then_some(index))
        .collect()
}

fn evidence_references(
    include_evidence: bool,
    prefix: &str,
    indices: &[usize],
) -> Vec<ConsumerEvidenceReference> {
    if !include_evidence {
        return Vec::new();
    }

    indices
        .iter()
        .map(|index| evidence_reference(prefix, *index))
        .collect()
}

fn evidence_reference(prefix: &str, index: usize) -> ConsumerEvidenceReference {
    ConsumerEvidenceReference {
        identifier: format!("{prefix}-{}", index + 1),
    }
}

fn build_evidence(report: &AnalysisReport) -> Vec<ConsumerEvidence> {
    let mut evidence = Vec::new();

    for (index, finding) in report.normalization_findings.iter().enumerate() {
        evidence.push(ConsumerEvidence {
            identifier: evidence_reference("normalization", index).identifier,
            kind: ConsumerEvidenceKind::Normalization,
            detail: ConsumerEvidenceDetail::Normalization {
                original_path: finding.original_path.clone(),
                normalized_path: finding.normalized_path.clone(),
            },
        });
    }

    for (index, candidate) in report.destination_discovery.candidates.iter().enumerate() {
        let destination_kind = consumer_destination_kind(candidate.reference.kind);
        let parent_evidence_identifier = candidate
            .provenance
            .parent_candidate_index
            .map(|parent| evidence_reference("candidate", parent).identifier);
        let transformations = candidate
            .provenance
            .transformations
            .iter()
            .map(consumer_transformation)
            .collect();

        evidence.push(ConsumerEvidence {
            identifier: evidence_reference("candidate", index).identifier,
            kind: ConsumerEvidenceKind::DestinationCandidate,
            detail: ConsumerEvidenceDetail::DestinationCandidate {
                destination_kind,
                submitted_reference: candidate.reference.submitted.clone(),
                normalized_reference: candidate.reference.normalized.clone(),
                hostname: candidate.reference.hostname.clone(),
                source_parameter: candidate.provenance.source_parameter.clone(),
                top_level_parameter_position: candidate.provenance.top_level_parameter_index + 1,
                source_parameter_position: candidate.provenance.source_parameter_index + 1,
                depth: candidate.provenance.depth,
                parent_evidence_identifier,
                transformations,
            },
        });
    }

    for (index, parameter) in report.parameters.iter().enumerate() {
        let has_relevant_evidence = match &parameter.decoding {
            Ok(decoding) => {
                !decoding.warnings.is_empty() || decoding.status == DecodingStatus::PassLimitReached
            }
            Err(_) => false,
        };
        if !has_relevant_evidence {
            continue;
        }

        evidence.push(ConsumerEvidence {
            identifier: evidence_reference("parameter", index).identifier,
            kind: ConsumerEvidenceKind::QueryParameter,
            detail: ConsumerEvidenceDetail::QueryParameter {
                name: parameter.name.clone(),
                raw_value: report.url.query_parameters[index].raw_value.clone(),
                position: index + 1,
            },
        });
    }

    if let OriginalPathAnalysis::Unsupported { reason } = report.original_path_analysis {
        evidence.push(ConsumerEvidence {
            identifier: evidence_reference("path-coverage", 0).identifier,
            kind: ConsumerEvidenceKind::OriginalPathCoverage,
            detail: ConsumerEvidenceDetail::OriginalPathCoverage {
                reason_identifier: original_path_reason_identifier(reason).to_string(),
            },
        });
    }

    for (index, issue) in report
        .destination_discovery
        .encoding_issues
        .iter()
        .enumerate()
    {
        evidence.push(ConsumerEvidence {
            identifier: evidence_reference("encoding-issue", index).identifier,
            kind: ConsumerEvidenceKind::EncodingIssue,
            detail: ConsumerEvidenceDetail::EncodingIssue {
                reason_identifier: encoding_issue_identifier(issue.kind).to_string(),
                top_level_parameter_position: issue.top_level_parameter_index + 1,
                source_parameter_position: issue.source_parameter_index + 1,
                depth: issue.depth,
                parent_evidence_identifier: issue
                    .parent_candidate_index
                    .map(|parent| evidence_reference("candidate", parent).identifier),
            },
        });
    }

    evidence
}

fn consumer_transformation(transformation: &DestinationTransformation) -> ConsumerTransformation {
    match transformation {
        DestinationTransformation::PercentDecoding { passes } => {
            ConsumerTransformation::PercentDecoding { passes: *passes }
        }
        DestinationTransformation::Base64Decoding { alphabet, padding } => {
            ConsumerTransformation::Base64Decoding {
                alphabet_identifier: match alphabet {
                    Base64Alphabet::Standard => "standard",
                    Base64Alphabet::UrlSafe => "url-safe",
                    Base64Alphabet::Ambiguous => "ambiguous",
                }
                .to_string(),
                padding_identifier: match padding {
                    Base64Padding::Padded => "padded",
                    Base64Padding::Unpadded => "unpadded",
                }
                .to_string(),
            }
        }
        DestinationTransformation::HtmlCharacterReferenceDecoding { reference_count } => {
            ConsumerTransformation::HtmlCharacterReferenceDecoding {
                reference_count: *reference_count,
            }
        }
        DestinationTransformation::CompoundQueryExtraction { parameter_position } => {
            ConsumerTransformation::CompoundQueryExtraction {
                parameter_position: *parameter_position,
            }
        }
    }
}

fn build_completeness(report: &AnalysisReport) -> ConsumerCompleteness {
    let original_path = match report.original_path_analysis {
        OriginalPathAnalysis::Examined => complete_stage(ConsumerAnalysisStage::OriginalPath),
        OriginalPathAnalysis::Unsupported { reason } => ConsumerStageCompleteness {
            stage: ConsumerAnalysisStage::OriginalPath,
            status: ConsumerStageStatus::UnsupportedEvidence,
            reasons: vec![make_reason(
                original_path_reason_identifier(reason),
                "The submitted URL spelling did not provide a confidently supported original-path boundary.",
            )],
        },
    };

    let mut percent_reasons = Vec::new();
    let mut percent_resource_limited = false;
    let mut percent_unsupported = false;
    for parameter in &report.parameters {
        match &parameter.decoding {
            Ok(decoding) if decoding.status == DecodingStatus::PassLimitReached => {
                percent_resource_limited = true;
                add_reason(
                    &mut percent_reasons,
                    "percent-decoding-pass-limit",
                    "The bounded percent-decoding pass limit was reached for at least one query value.",
                );
            }
            Err(DecodeError::InputTooLong) => {
                percent_resource_limited = true;
                add_reason(
                    &mut percent_reasons,
                    "percent-decoding-input-limit",
                    "At least one query value exceeded the percent-decoder input limit.",
                );
            }
            Err(DecodeError::InvalidUtf8) => {
                percent_unsupported = true;
                add_reason(
                    &mut percent_reasons,
                    "percent-decoding-invalid-utf8",
                    "At least one percent-decoded query value was not valid UTF-8 and could not be interpreted as supported text.",
                );
            }
            Ok(_) => {}
        }
    }
    for limit in report.destination_discovery.status.limits() {
        match limit {
            DestinationDiscoveryLimit::DecoderInputLength => {
                percent_resource_limited = true;
                add_reason(
                    &mut percent_reasons,
                    destination_limit_identifier(*limit),
                    limit.explanation(),
                );
            }
            DestinationDiscoveryLimit::DecodingPassLimit => {
                percent_resource_limited = true;
                add_reason(
                    &mut percent_reasons,
                    destination_limit_identifier(*limit),
                    limit.explanation(),
                );
            }
            _ => {}
        }
    }
    let percent_decoding = ConsumerStageCompleteness {
        stage: ConsumerAnalysisStage::PercentDecoding,
        status: if percent_resource_limited {
            ConsumerStageStatus::ResourceLimited
        } else if percent_unsupported {
            ConsumerStageStatus::UnsupportedEvidence
        } else {
            ConsumerStageStatus::CompleteWithinSupportedScope
        },
        reasons: percent_reasons,
    };

    let mut traversal_reasons = Vec::new();
    let mut secondary_reasons = Vec::new();
    for limit in report.destination_discovery.status.limits() {
        let (target, identifier) = if is_traversal_limit(*limit) {
            (&mut traversal_reasons, destination_limit_identifier(*limit))
        } else if is_secondary_limit(*limit) {
            (&mut secondary_reasons, destination_limit_identifier(*limit))
        } else {
            continue;
        };
        add_reason(target, identifier, limit.explanation());
    }
    let destination_traversal = ConsumerStageCompleteness {
        stage: ConsumerAnalysisStage::DestinationTraversal,
        status: if !traversal_reasons.is_empty() {
            ConsumerStageStatus::ResourceLimited
        } else if report
            .destination_discovery
            .interpretation_coverage
            .is_complete()
        {
            ConsumerStageStatus::CompleteWithinSupportedScope
        } else {
            ConsumerStageStatus::PartiallyExamined
        },
        reasons: {
            for issue in report
                .destination_discovery
                .interpretation_coverage
                .issues()
            {
                add_reason(
                    &mut traversal_reasons,
                    interpretation_issue_identifier(*issue),
                    issue.explanation(),
                );
            }
            traversal_reasons
        },
    };

    for issue in report.destination_discovery.encoding_coverage.issues() {
        add_reason(
            &mut secondary_reasons,
            encoding_issue_identifier(*issue),
            issue.explanation(),
        );
    }
    let secondary_has_resource_limit = report
        .destination_discovery
        .status
        .limits()
        .iter()
        .any(|limit| is_secondary_limit(*limit));
    let secondary_encoding = ConsumerStageCompleteness {
        stage: ConsumerAnalysisStage::SecondaryEncoding,
        status: if secondary_has_resource_limit {
            ConsumerStageStatus::ResourceLimited
        } else if secondary_reasons.is_empty() {
            ConsumerStageStatus::CompleteWithinSupportedScope
        } else {
            ConsumerStageStatus::PartiallyExamined
        },
        reasons: secondary_reasons,
    };

    let stages = vec![
        original_path,
        percent_decoding,
        destination_traversal,
        secondary_encoding,
    ];
    let status = if stages
        .iter()
        .all(|stage| stage.status == ConsumerStageStatus::CompleteWithinSupportedScope)
    {
        ConsumerCompletenessStatus::CompleteWithinSupportedScope
    } else {
        ConsumerCompletenessStatus::PartiallyExamined
    };

    ConsumerCompleteness { status, stages }
}

fn complete_stage(stage: ConsumerAnalysisStage) -> ConsumerStageCompleteness {
    ConsumerStageCompleteness {
        stage,
        status: ConsumerStageStatus::CompleteWithinSupportedScope,
        reasons: Vec::new(),
    }
}

fn make_reason(identifier: &str, explanation: &str) -> ConsumerCompletenessReason {
    ConsumerCompletenessReason {
        identifier: identifier.to_string(),
        explanation: explanation.to_string(),
    }
}

fn add_reason(reasons: &mut Vec<ConsumerCompletenessReason>, identifier: &str, explanation: &str) {
    if reasons.iter().any(|reason| reason.identifier == identifier) {
        return;
    }
    reasons.push(make_reason(identifier, explanation));
}

fn is_traversal_limit(limit: DestinationDiscoveryLimit) -> bool {
    matches!(
        limit,
        DestinationDiscoveryLimit::MaximumNestingDepth
            | DestinationDiscoveryLimit::MaximumCandidateCount
            | DestinationDiscoveryLimit::MaximumNestedParameterCount
            | DestinationDiscoveryLimit::MaximumCompoundParameterCount
            | DestinationDiscoveryLimit::NestedUrlLength
    )
}

fn is_secondary_limit(limit: DestinationDiscoveryLimit) -> bool {
    matches!(
        limit,
        DestinationDiscoveryLimit::SecondaryEncodingInputLength
            | DestinationDiscoveryLimit::SecondaryDecodedOutputLength
    )
}

fn destination_limit_identifier(limit: DestinationDiscoveryLimit) -> &'static str {
    match limit {
        DestinationDiscoveryLimit::MaximumNestingDepth => "destination-maximum-depth",
        DestinationDiscoveryLimit::MaximumCandidateCount => "destination-maximum-candidates",
        DestinationDiscoveryLimit::MaximumNestedParameterCount => {
            "destination-maximum-nested-parameters"
        }
        DestinationDiscoveryLimit::MaximumCompoundParameterCount => {
            "destination-maximum-compound-parameters"
        }
        DestinationDiscoveryLimit::DecoderInputLength => "percent-decoding-input-limit",
        DestinationDiscoveryLimit::DecodingPassLimit => "percent-decoding-pass-limit",
        DestinationDiscoveryLimit::NestedUrlLength => "destination-nested-url-limit",
        DestinationDiscoveryLimit::SecondaryEncodingInputLength => "secondary-encoding-input-limit",
        DestinationDiscoveryLimit::SecondaryDecodedOutputLength => {
            "secondary-encoding-output-limit"
        }
    }
}

fn interpretation_issue_identifier(issue: DestinationInterpretationIssueKind) -> &'static str {
    match issue {
        DestinationInterpretationIssueKind::AmbiguousHostnameFirstReference => {
            "destination-hostname-first-ambiguous"
        }
    }
}

fn original_path_reason_identifier(reason: OriginalPathUnsupportedReason) -> &'static str {
    match reason {
        OriginalPathUnsupportedReason::UnrecognizedSchemeBoundary => {
            "original-path-unrecognized-scheme-boundary"
        }
        OriginalPathUnsupportedReason::AmbiguousAuthorityBoundary => {
            "original-path-ambiguous-authority-boundary"
        }
    }
}

fn encoding_issue_identifier(issue: DestinationEncodingIssueKind) -> &'static str {
    match issue {
        DestinationEncodingIssueKind::MalformedBase64 => "secondary-malformed-base64",
        DestinationEncodingIssueKind::MalformedHtmlCharacterReference => {
            "secondary-malformed-html-reference"
        }
        DestinationEncodingIssueKind::UnsupportedHtmlCharacterReference => {
            "secondary-unsupported-html-reference"
        }
        DestinationEncodingIssueKind::InvalidHtmlUnicodeScalar => {
            "secondary-invalid-html-unicode-scalar"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::analyze_url;
    use crate::destinations::MAX_DISCOVERED_DESTINATIONS;
    use crate::reporting::create_shareable_report;

    fn observation(report: &ConsumerReport, kind: ConsumerObservationKind) -> &ConsumerObservation {
        report
            .observations()
            .iter()
            .find(|observation| observation.kind() == kind)
            .expect("expected consumer observation")
    }

    fn complex_analysis() -> AnalysisReport {
        analyze_url(concat!(
            "https://private.example.com/private/path?token=fake-secret-123",
            "&target=https%3A%2F%2Fdestination.example%2Fprivate",
            "&payload=aHR0cHM6Ly9lbmNvZGVkLmV4YW1wbGUvcHJpdmF0ZQ%3D%3D"
        ))
        .unwrap()
    }

    #[test]
    fn consumer_schema_version_and_private_defaults_are_stable() {
        let analysis = complex_analysis();
        let report = create_consumer_report(&analysis, ConsumerReportOptions::default());

        assert_eq!(CONSUMER_REPORT_SCHEMA_VERSION, 1);
        assert_eq!(report.schema_version(), CONSUMER_REPORT_SCHEMA_VERSION);
        assert_eq!(report.disclosure(), ConsumerDisclosureClass::PrivacyReduced);
        assert_eq!(report.hostname(), None);
        assert!(report.evidence().is_empty());
        assert!(
            report
                .observations()
                .iter()
                .all(|observation| observation.evidence_references().is_empty())
        );

        let fields = format!(
            "{}:{:?}:{:?}:{:?}",
            report.schema_version(),
            report.disclosure(),
            report.completeness(),
            report.observations(),
        );
        let debug = format!("{report:?}");
        for output in [&fields, &debug] {
            assert!(!output.contains("private.example.com"));
            assert!(!output.contains("destination.example"));
            assert!(!output.contains("encoded.example"));
            assert!(!output.contains("fake-secret-123"));
            assert!(!output.contains("/private"));
            assert!(!output.contains("token"));
            assert!(!output.contains("aHR0cHM6"));
        }
    }

    #[test]
    fn hostname_and_detailed_evidence_controls_are_independent() {
        let analysis = complex_analysis();
        let hostname_only =
            create_consumer_report(&analysis, ConsumerReportOptions::new().with_hostname());
        assert_eq!(
            hostname_only.disclosure(),
            ConsumerDisclosureClass::OriginalHostname
        );
        assert_eq!(hostname_only.hostname(), Some("private.example.com"));
        assert!(hostname_only.evidence().is_empty());
        assert!(!format!("{hostname_only:?}").contains("private.example.com"));

        let evidence_only = create_consumer_report(
            &analysis,
            ConsumerReportOptions::new().with_detailed_evidence(),
        );
        assert_eq!(
            evidence_only.disclosure(),
            ConsumerDisclosureClass::DetailedEvidence
        );
        assert_eq!(evidence_only.hostname(), None);
        assert!(!evidence_only.evidence().is_empty());
        assert!(
            evidence_only
                .observations()
                .iter()
                .any(|observation| !observation.evidence_references().is_empty())
        );

        let combined = create_consumer_report(
            &analysis,
            ConsumerReportOptions::new()
                .with_hostname()
                .with_detailed_evidence(),
        );
        assert_eq!(
            combined.disclosure(),
            ConsumerDisclosureClass::OriginalHostnameAndDetailedEvidence
        );
        assert_eq!(combined.hostname(), Some("private.example.com"));
        assert!(!combined.evidence().is_empty());
    }

    #[test]
    fn destination_hostname_disclosure_is_narrow_and_redacted_in_debug() {
        let analysis = analyze_url(concat!(
            "https://private.example/?token=fake-secret&target=",
            "https%3A%2F%2Fdestination.example.%2Fprivate%3Fcode%3Dfake-token%23private-fragment"
        ))
        .unwrap();
        let disclosure =
            create_destination_disclosure(&analysis, DestinationDisclosureLevel::Hostnames);

        assert_eq!(
            disclosure.schema_version(),
            DESTINATION_DISCLOSURE_SCHEMA_VERSION
        );
        assert_eq!(disclosure.level(), DestinationDisclosureLevel::Hostnames);
        assert_eq!(disclosure.candidates().len(), 1);
        let candidate = &disclosure.candidates()[0];
        assert_eq!(candidate.identifier(), "candidate-1");
        assert_eq!(candidate.hostname(), "destination.example.");
        assert_eq!(
            candidate.destination_kind(),
            ConsumerDestinationKind::AbsoluteHttps
        );
        assert_eq!(candidate.depth(), 0);
        assert_eq!(candidate.parent_candidate_identifier(), None);
        assert_eq!(candidate.detail(), None);

        let debug = format!("{disclosure:?}:{candidate:?}");
        for private in [
            "private.example",
            "destination.example",
            "fake-secret",
            "fake-token",
            "private-fragment",
            "target",
        ] {
            assert!(!debug.contains(private));
        }
    }

    #[test]
    fn detailed_destination_disclosure_is_scoped_and_removes_credentials_and_fragments() {
        let analysis = analyze_url(concat!(
            "https://top.example/?target=",
            "https%3A%2F%2Ffake-user%3Afake-password%40destination.example%2Fprivate%3Fcode%3Dfake-token",
            "%26next%3Dhttps%253A%252F%252Fnested.example%252Flanding%23private-fragment"
        ))
        .unwrap();
        let disclosure =
            create_destination_disclosure(&analysis, DestinationDisclosureLevel::Detailed);

        assert_eq!(disclosure.level(), DestinationDisclosureLevel::Detailed);
        assert_eq!(disclosure.candidates().len(), 2);
        let direct = &disclosure.candidates()[0];
        let detail = direct.detail().expect("detailed evidence");
        assert_eq!(detail.source_parameter(), "target");
        assert_eq!(detail.top_level_parameter_position(), 1);
        assert_eq!(detail.source_parameter_position(), 1);
        assert!(
            detail
                .submitted_reference_without_credentials_or_fragment()
                .contains("fake-token")
        );
        assert!(
            !detail
                .submitted_reference_without_credentials_or_fragment()
                .contains("private-fragment")
        );
        assert!(
            !detail
                .normalized_reference_without_credentials_or_fragment()
                .contains("private-fragment")
        );
        assert!(
            !detail
                .submitted_reference_without_credentials_or_fragment()
                .contains("fake-user")
        );
        assert!(
            !detail
                .normalized_reference_without_credentials_or_fragment()
                .contains("fake-password")
        );
        assert!(!detail.transformations().is_empty());

        let nested = &disclosure.candidates()[1];
        assert_eq!(nested.depth(), 1);
        assert_eq!(nested.parent_candidate_identifier(), Some("candidate-1"));
        assert_eq!(nested.detail().unwrap().source_parameter(), "next");
        assert!(!format!("{detail:?}").contains("fake-token"));
    }

    #[test]
    fn destination_disclosure_keeps_ambiguity_separate_from_candidates() {
        let analysis = analyze_url(concat!(
            "https://social.example/?target=",
            "tracking.example%2F%2Fpath%3Fadurl%3D%2F%2Fstorage.example%2Flanding"
        ))
        .unwrap();
        let hostnames =
            create_destination_disclosure(&analysis, DestinationDisclosureLevel::Hostnames);

        assert_eq!(hostnames.candidates().len(), 1);
        assert_eq!(hostnames.ambiguities().len(), 1);
        let ambiguity = &hostnames.ambiguities()[0];
        assert_eq!(ambiguity.identifier(), "ambiguity-1");
        assert!(
            ambiguity
                .explanation()
                .contains("not classified as a destination")
        );
        assert_eq!(ambiguity.top_level_parameter_position(), None);
        assert_eq!(ambiguity.source_parameter_position(), None);

        let detailed =
            create_destination_disclosure(&analysis, DestinationDisclosureLevel::Detailed);
        assert_eq!(
            detailed.ambiguities()[0].top_level_parameter_position(),
            Some(1)
        );
        assert_eq!(
            detailed.ambiguities()[0].source_parameter_position(),
            Some(1)
        );
    }

    #[test]
    fn explicit_evidence_uses_stable_references_and_redacted_debug() {
        let analysis = complex_analysis();
        let report = create_consumer_report(
            &analysis,
            ConsumerReportOptions::new().with_detailed_evidence(),
        );

        let absolute = observation(
            &report,
            ConsumerObservationKind::PotentialAbsoluteDestination,
        );
        assert_eq!(absolute.occurrence_count(), 2);
        assert_eq!(
            absolute
                .evidence_references()
                .iter()
                .map(ConsumerEvidenceReference::identifier)
                .collect::<Vec<_>>(),
            ["candidate-1", "candidate-2"]
        );

        let candidate = report
            .evidence()
            .iter()
            .find(|evidence| evidence.identifier() == "candidate-1")
            .unwrap();
        let ConsumerEvidenceDetail::DestinationCandidate {
            hostname,
            source_parameter,
            ..
        } = candidate.detail()
        else {
            panic!("expected destination evidence");
        };
        assert_eq!(hostname, "destination.example");
        assert_eq!(source_parameter, "target");

        let evidence_debug = format!("{candidate:?}");
        let report_debug = format!("{report:?}");
        for output in [&evidence_debug, &report_debug] {
            assert!(!output.contains("destination.example"));
            assert!(!output.contains("target"));
            assert!(!output.contains("/private"));
        }
        assert!(evidence_debug.contains("[REDACTED]"));
    }

    #[test]
    fn observation_order_identifiers_and_explanations_are_deterministic() {
        let input = concat!(
            "https://top.example/public/%2e%2e/admin?broken=value%ZZ",
            "&deep=%2525252541",
            "&absolute=https%3A%2F%2Fabsolute.example%2F%3Fnext%3D%252F%252Fnested.example%252F",
            "&base64=aHR0cHM6Ly9lbmNvZGVkLmV4YW1wbGUv"
        );
        let analysis = analyze_url(input).unwrap();
        let first = create_consumer_report(&analysis, ConsumerReportOptions::default());
        let second = create_consumer_report(&analysis, ConsumerReportOptions::default());

        assert_eq!(first, second);
        let identifiers = first
            .observations()
            .iter()
            .map(ConsumerObservation::identifier)
            .collect::<Vec<_>>();
        assert_eq!(
            identifiers,
            [
                "normalization.encoded-parent-directory",
                "destination.absolute-candidate",
                "destination.protocol-relative-candidate",
                "destination.nested-candidate",
                "destination.base64-derived-candidate",
                "encoding.multiple-percent-transformations",
                "decoding.malformed-percent-encoding",
                "decoding.percent-pass-limit",
            ]
        );

        for observation in first.observations() {
            assert_eq!(observation.identifier(), observation.kind().identifier());
            assert_eq!(observation.title(), observation.kind().title());
            assert_eq!(observation.explanation(), observation.kind().explanation());
        }
    }

    #[test]
    fn duplicate_candidate_occurrences_are_aggregated_with_distinct_evidence() {
        let analysis = analyze_url(concat!(
            "https://top.example/?target=https%3A%2F%2Frepeated.example%2F",
            "&target=https%3A%2F%2Frepeated.example%2F"
        ))
        .unwrap();
        let report = create_consumer_report(
            &analysis,
            ConsumerReportOptions::new().with_detailed_evidence(),
        );
        let observation = observation(
            &report,
            ConsumerObservationKind::PotentialAbsoluteDestination,
        );

        assert_eq!(observation.occurrence_count(), 2);
        assert_eq!(observation.evidence_references().len(), 2);
        assert_ne!(
            observation.evidence_references()[0].identifier(),
            observation.evidence_references()[1].identifier()
        );
    }

    #[test]
    fn completeness_separates_unsupported_resource_and_encoding_limits() {
        let unsupported = analyze_url("https:////top.example/path?category=books").unwrap();
        let unsupported_report =
            create_consumer_report(&unsupported, ConsumerReportOptions::default());
        assert_eq!(
            unsupported_report.completeness().status(),
            ConsumerCompletenessStatus::PartiallyExamined
        );
        let original_stage = unsupported_report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::OriginalPath)
            .unwrap();
        assert_eq!(
            original_stage.status(),
            ConsumerStageStatus::UnsupportedEvidence
        );
        assert_eq!(original_stage.reasons().len(), 1);
        assert_eq!(
            observation(
                &unsupported_report,
                ConsumerObservationKind::UnsupportedOriginalPathEvidence
            )
            .occurrence_count(),
            1
        );

        let query = (0..=MAX_DISCOVERED_DESTINATIONS)
            .map(|index| {
                format!(
                    "p{index}={}",
                    url::form_urlencoded::byte_serialize(
                        format!("https://host{index}.example/").as_bytes()
                    )
                    .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        let limited = analyze_url(&format!("https://top.example/?{query}")).unwrap();
        let limited_report = create_consumer_report(&limited, ConsumerReportOptions::default());
        let traversal_stage = limited_report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::DestinationTraversal)
            .unwrap();
        assert_eq!(
            traversal_stage.status(),
            ConsumerStageStatus::ResourceLimited
        );
        assert_eq!(
            observation(
                &limited_report,
                ConsumerObservationKind::IncompleteDestinationDiscovery
            )
            .occurrence_count(),
            1
        );

        let html = url::form_urlencoded::byte_serialize(
            "https&colon;&sol;&sol;hidden.example&unsupported;".as_bytes(),
        )
        .collect::<String>();
        let encoding = analyze_url(&format!("https://top.example/?value={html}")).unwrap();
        let encoding_report = create_consumer_report(&encoding, ConsumerReportOptions::default());
        let encoding_stage = encoding_report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::SecondaryEncoding)
            .unwrap();
        assert_eq!(
            encoding_stage.status(),
            ConsumerStageStatus::PartiallyExamined
        );
        assert_eq!(
            observation(
                &encoding_report,
                ConsumerObservationKind::IncompleteEncodingInterpretation
            )
            .occurrence_count(),
            1
        );
    }

    #[test]
    fn complete_means_complete_only_within_supported_scope() {
        let analysis = analyze_url("https://top.example/products?category=books").unwrap();
        let report = create_consumer_report(&analysis, ConsumerReportOptions::default());

        assert!(report.observations().is_empty());
        assert_eq!(
            report.completeness().status(),
            ConsumerCompletenessStatus::CompleteWithinSupportedScope
        );
        assert!(report.completeness().stages().iter().all(|stage| {
            stage.status() == ConsumerStageStatus::CompleteWithinSupportedScope
                && stage.reasons().is_empty()
        }));
    }

    #[test]
    fn nested_percent_limit_is_reported_in_its_own_completeness_stage() {
        let analysis = analyze_url(concat!(
            "https://top.example/?payload=",
            "aHR0cHM6Ly9taWRkbGUuZXhhbXBsZS8/ZGVlcD0lMjUyNTI1MjU0MQ=="
        ))
        .unwrap();
        assert!(analysis.parameters.iter().all(|parameter| {
            parameter
                .decoding
                .as_ref()
                .is_ok_and(|decoding| decoding.status != DecodingStatus::PassLimitReached)
        }));

        let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
        assert_eq!(
            observation(&report, ConsumerObservationKind::PercentDecodingPassLimit)
                .occurrence_count(),
            1
        );
        let percent_stage = report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::PercentDecoding)
            .unwrap();
        assert_eq!(percent_stage.status(), ConsumerStageStatus::ResourceLimited);
        assert_eq!(
            percent_stage.reasons()[0].identifier(),
            "percent-decoding-pass-limit"
        );
        let traversal_stage = report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::DestinationTraversal)
            .unwrap();
        assert_eq!(
            traversal_stage.status(),
            ConsumerStageStatus::CompleteWithinSupportedScope
        );
    }

    #[test]
    fn fixed_consumer_text_contains_no_verdict_claims() {
        let kinds = [
            ConsumerObservationKind::EncodedParentDirectoryNormalization,
            ConsumerObservationKind::PotentialAbsoluteDestination,
            ConsumerObservationKind::PotentialProtocolRelativeReference,
            ConsumerObservationKind::PotentialHostnameFirstReference,
            ConsumerObservationKind::NestedDestinationCandidate,
            ConsumerObservationKind::Base64DerivedDestination,
            ConsumerObservationKind::HtmlReferenceDerivedDestination,
            ConsumerObservationKind::MultiplePercentDecodingTransformations,
            ConsumerObservationKind::EncodedQueryDelimiter,
            ConsumerObservationKind::CompoundQueryValue,
            ConsumerObservationKind::AmbiguousHostnameFirstReference,
            ConsumerObservationKind::MalformedPercentEncoding,
            ConsumerObservationKind::PercentDecodingPassLimit,
            ConsumerObservationKind::UnsupportedOriginalPathEvidence,
            ConsumerObservationKind::IncompleteDestinationDiscovery,
            ConsumerObservationKind::IncompleteEncodingInterpretation,
        ];

        for kind in kinds {
            let text = format!("{} {}", kind.title(), kind.explanation()).to_lowercase();
            assert!(!text.contains("this link is safe"));
            assert!(!text.contains("this link is malicious"));
            assert!(!text.contains("redirects to"));
            assert!(!text.contains("final destination"));
        }
    }

    #[test]
    fn motivating_compound_example_has_privacy_safe_observations_and_partial_coverage() {
        let private_values = [
            "social.example",
            "tracking.example",
            "storage.example",
            "synthetic-fragment",
            "fake-id",
            "adurl",
        ];
        let analysis = analyze_url(concat!(
            "https://social.example/link/v2?aid=1988&scene=bio_url&target=",
            "tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=",
            "%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html",
            "#synthetic-fragment"
        ))
        .unwrap();
        let report = create_consumer_report(&analysis, ConsumerReportOptions::default());

        for kind in [
            ConsumerObservationKind::PotentialProtocolRelativeReference,
            ConsumerObservationKind::NestedDestinationCandidate,
            ConsumerObservationKind::HtmlReferenceDerivedDestination,
            ConsumerObservationKind::EncodedQueryDelimiter,
            ConsumerObservationKind::CompoundQueryValue,
            ConsumerObservationKind::AmbiguousHostnameFirstReference,
        ] {
            assert_eq!(observation(&report, kind).occurrence_count(), 1);
        }
        let traversal = report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.stage() == ConsumerAnalysisStage::DestinationTraversal)
            .unwrap();
        assert_eq!(traversal.status(), ConsumerStageStatus::PartiallyExamined);
        assert_eq!(
            traversal.reasons()[0].identifier(),
            "destination-hostname-first-ambiguous"
        );

        let debug = format!("{report:?}");
        let rendered = report
            .observations()
            .iter()
            .map(|item| format!("{} {}", item.title(), item.explanation()))
            .collect::<String>();
        for secret in private_values {
            assert!(!debug.contains(secret));
            assert!(!rendered.contains(secret));
        }
    }

    #[test]
    fn existing_shareable_report_remains_independent() {
        let analysis = complex_analysis();
        let before = create_shareable_report(&analysis);
        let _consumer = create_consumer_report(
            &analysis,
            ConsumerReportOptions::new()
                .with_hostname()
                .with_detailed_evidence(),
        );
        let after = create_shareable_report(&analysis);

        assert_eq!(before, after);
        assert_eq!(before.hostname, "[REDACTED]");
    }

    #[test]
    fn analysis_source_has_no_network_client_usage() {
        let source = [
            include_str!("analysis.rs"),
            include_str!("consumer.rs"),
            include_str!("decoder.rs"),
            include_str!("destinations.rs"),
            include_str!("encodings.rs"),
            include_str!("normalization.rs"),
            include_str!("parser.rs"),
            include_str!("reporting.rs"),
        ]
        .join("\n");

        for forbidden in [
            concat!("req", "west::"),
            concat!("hy", "per::"),
            concat!("std::net::", "TcpStream"),
            concat!("std::net::", "UdpSocket"),
            concat!("TcpStream", "::connect"),
            concat!("UdpSocket", "::bind"),
        ] {
            assert!(
                !source.contains(forbidden),
                "network API found: {forbidden}"
            );
        }
    }
}
