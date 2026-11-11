//! Minimal browser-facing adapter for privacy-reduced LinkSleuth reports.
//!
//! The adapter exposes a deliberately small, versioned JSON schema. It never
//! exports `AnalysisReport`, original hostnames, or detailed evidence. Browser
//! applications must treat returned strings as data and render them with safe
//! text APIs rather than interpolating them as HTML.

use linksleuth_assessment::{
    AssessmentContext, ClaimedPurpose, ExpectedContact, LinkAssessment, LinkSource,
    RequestedAction, assess_link,
};
use linksleuth_core::analysis::analyze_url;
use linksleuth_core::consumer::{
    ConsumerCompletenessStatus, ConsumerDestinationKind, ConsumerReport, ConsumerReportOptions,
    ConsumerStageStatus, ConsumerTransformation, DestinationDisclosure, DestinationDisclosureLevel,
    create_consumer_report, create_destination_disclosure,
};
use linksleuth_core::parser::UrlError;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Version of the browser JSON envelope, independent of the consumer schema.
pub const BROWSER_WIRE_SCHEMA_VERSION: u32 = 2;

/// Version of the separate explicit destination-disclosure browser response.
pub const DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION: u32 = 1;

/// Maximum serialized optional context accepted from the browser.
pub const MAX_ASSESSMENT_CONTEXT_BYTES: usize = 1024;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserErrorCode {
    EmptyInput,
    InvalidUrl,
    UnsupportedScheme,
    InputTooLong,
    TooManyQueryParameters,
    InvalidContext,
    InvalidDisclosureLevel,
    AnalysisFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserError {
    code: BrowserErrorCode,
    message: &'static str,
}

impl BrowserError {
    pub const fn code(&self) -> BrowserErrorCode {
        self.code
    }

    pub const fn message(&self) -> &'static str {
        self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserObservation {
    identifier: String,
    title: String,
    explanation: String,
    occurrence_count: usize,
    evidence_references: Vec<String>,
}

impl BrowserObservation {
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

    pub fn evidence_references(&self) -> &[String] {
        &self.evidence_references
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserCompletenessReason {
    identifier: String,
    explanation: String,
}

impl BrowserCompletenessReason {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserCompletenessStage {
    identifier: String,
    status: String,
    reasons: Vec<BrowserCompletenessReason>,
}

impl BrowserCompletenessStage {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn reasons(&self) -> &[BrowserCompletenessReason] {
        &self.reasons
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserCompleteness {
    status: String,
    stages: Vec<BrowserCompletenessStage>,
}

impl BrowserCompleteness {
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn stages(&self) -> &[BrowserCompletenessStage] {
        &self.stages
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserReport {
    consumer_schema_version: u32,
    disclosure: &'static str,
    observations: Vec<BrowserObservation>,
    completeness: BrowserCompleteness,
    assessment: BrowserAssessment,
}

impl BrowserReport {
    pub const fn consumer_schema_version(&self) -> u32 {
        self.consumer_schema_version
    }

    pub const fn disclosure(&self) -> &'static str {
        self.disclosure
    }

    pub fn observations(&self) -> &[BrowserObservation] {
        &self.observations
    }

    pub const fn completeness(&self) -> &BrowserCompleteness {
        &self.completeness
    }

    pub const fn assessment(&self) -> &BrowserAssessment {
        &self.assessment
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserAssessmentReason {
    identifier: String,
    source: String,
    explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserAssessmentLimitation {
    identifier: String,
    explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserAssessmentNextStep {
    identifier: String,
    explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserAssessment {
    contract_version: u32,
    category_identifier: String,
    category_title: String,
    category_explanation: String,
    action_identifier: String,
    action_title: String,
    action_explanation: String,
    next_steps: Vec<BrowserAssessmentNextStep>,
    reasons: Vec<BrowserAssessmentReason>,
    limitations: Vec<BrowserAssessmentLimitation>,
    context_changed_recommendation: bool,
}

impl BrowserAssessment {
    pub fn category_identifier(&self) -> &str {
        &self.category_identifier
    }

    pub fn action_identifier(&self) -> &str {
        &self.action_identifier
    }

    pub const fn context_changed_recommendation(&self) -> bool {
        self.context_changed_recommendation
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserAssessmentContext {
    source: Option<BrowserLinkSource>,
    claimed_purpose: Option<BrowserClaimedPurpose>,
    expected_contact: Option<BrowserExpectedContact>,
    requested_action: Option<BrowserRequestedAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BrowserLinkSource {
    Email,
    Message,
    SocialMedia,
    WebsiteOrSearch,
    QrCode,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BrowserClaimedPurpose {
    Document,
    AccountSignInOrVerification,
    DeliveryOrShipping,
    PaymentOrFinancialRequest,
    OrdinaryContent,
    SoftwareDownloadOrInstall,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BrowserExpectedContact {
    Expected,
    Unexpected,
    Unsure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BrowserRequestedAction {
    ViewContent,
    EnterAccountCredentials,
    ProvidePersonalInformation,
    MakePayment,
    DownloadOrInstallSoftware,
    ContactSomeone,
    OtherOrUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserResponse {
    wire_schema_version: u32,
    report: Option<BrowserReport>,
    error: Option<BrowserError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationTransformation {
    identifier: &'static str,
    explanation: String,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationDetail {
    submitted_reference: String,
    normalized_reference: String,
    source_parameter: String,
    top_level_parameter_position: usize,
    source_parameter_position: usize,
    transformations: Vec<BrowserDestinationTransformation>,
}

impl std::fmt::Debug for BrowserDestinationDetail {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("[REDACTED DESTINATION DETAIL]")
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationCandidate {
    identifier: String,
    hostname: String,
    reference_kind: &'static str,
    relationship: &'static str,
    depth: usize,
    parent_candidate_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<BrowserDestinationDetail>,
}

impl std::fmt::Debug for BrowserDestinationCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserDestinationCandidate")
            .field("identifier", &self.identifier)
            .field("hostname", &"[REDACTED]")
            .field("reference_kind", &self.reference_kind)
            .field("relationship", &self.relationship)
            .field("depth", &self.depth)
            .field(
                "parent_candidate_identifier",
                &self.parent_candidate_identifier,
            )
            .field("has_detail", &self.detail.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationAmbiguity {
    identifier: String,
    explanation: String,
    relationship: &'static str,
    depth: usize,
    parent_candidate_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_level_parameter_position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_parameter_position: Option<usize>,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationDisclosure {
    contract_version: u32,
    level: &'static str,
    notice: &'static str,
    candidates: Vec<BrowserDestinationCandidate>,
    ambiguities: Vec<BrowserDestinationAmbiguity>,
}

impl std::fmt::Debug for BrowserDestinationDisclosure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BrowserDestinationDisclosure")
            .field("contract_version", &self.contract_version)
            .field("level", &self.level)
            .field("candidate_count", &self.candidates.len())
            .field("ambiguity_count", &self.ambiguities.len())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserDestinationDisclosureResponse {
    destination_disclosure_wire_schema_version: u32,
    disclosure: Option<BrowserDestinationDisclosure>,
    error: Option<BrowserError>,
}

impl BrowserResponse {
    pub const fn wire_schema_version(&self) -> u32 {
        self.wire_schema_version
    }

    pub const fn report(&self) -> Option<&BrowserReport> {
        self.report.as_ref()
    }

    pub const fn error(&self) -> Option<&BrowserError> {
        self.error.as_ref()
    }

    fn success(report: BrowserReport) -> Self {
        Self {
            wire_schema_version: BROWSER_WIRE_SCHEMA_VERSION,
            report: Some(report),
            error: None,
        }
    }

    fn failure(error: BrowserError) -> Self {
        Self {
            wire_schema_version: BROWSER_WIRE_SCHEMA_VERSION,
            report: None,
            error: Some(error),
        }
    }
}

/// Runs local passive analysis and returns only the privacy-reduced browser model.
pub fn analyze_for_browser(input: &str) -> BrowserResponse {
    analyze_for_browser_with_context(input, &AssessmentContext::default())
}

fn analyze_for_browser_with_context(input: &str, context: &AssessmentContext) -> BrowserResponse {
    if input.trim().is_empty() {
        return BrowserResponse::failure(error(
            BrowserErrorCode::EmptyInput,
            "Enter a URL to analyze.",
        ));
    }

    match analyze_url(input) {
        Ok(analysis) => {
            let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
            let assessment = assess_link(&report, context);
            BrowserResponse::success(browser_report(&report, &assessment))
        }
        Err(source) => BrowserResponse::failure(browser_error(&source)),
    }
}

/// Returns the versioned browser response as JSON containing no raw evidence.
pub fn analyze_for_browser_json(input: &str) -> String {
    let response = analyze_for_browser(input);
    serde_json::to_string(&response).unwrap_or_else(|_| fixed_serialization_failure().to_string())
}

/// Reassesses the same URL locally with optional typed context.
pub fn assess_for_browser_json(input: &str, context_json: &str) -> String {
    if context_json.len() > MAX_ASSESSMENT_CONTEXT_BYTES {
        return serialize_response(BrowserResponse::failure(error(
            BrowserErrorCode::InvalidContext,
            "Clue could not use the optional context answers.",
        )));
    }
    let context = match serde_json::from_str::<BrowserAssessmentContext>(context_json) {
        Ok(context) => assessment_context(context),
        Err(_) => {
            return serialize_response(BrowserResponse::failure(error(
                BrowserErrorCode::InvalidContext,
                "Clue could not use the optional context answers.",
            )));
        }
    };
    serialize_response(analyze_for_browser_with_context(input, &context))
}

/// Returns a separately versioned, explicitly requested destination disclosure.
///
/// The normal browser analysis response never calls this path. Each invocation
/// reruns the same passive local analysis and returns only the selected bounded
/// disclosure level.
pub fn disclose_destinations_for_browser_json(input: &str, level: &str) -> String {
    let level = match level {
        "hostnames" => DestinationDisclosureLevel::Hostnames,
        "detailed" => DestinationDisclosureLevel::Detailed,
        _ => {
            return serialize_destination_disclosure_response(
                BrowserDestinationDisclosureResponse {
                    destination_disclosure_wire_schema_version:
                        DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION,
                    disclosure: None,
                    error: Some(error(
                        BrowserErrorCode::InvalidDisclosureLevel,
                        "Clue could not use that destination disclosure level.",
                    )),
                },
            );
        }
    };

    let response = if input.trim().is_empty() {
        BrowserDestinationDisclosureResponse {
            destination_disclosure_wire_schema_version: DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION,
            disclosure: None,
            error: Some(error(
                BrowserErrorCode::EmptyInput,
                "Enter a URL to analyze.",
            )),
        }
    } else {
        match analyze_url(input) {
            Ok(analysis) => BrowserDestinationDisclosureResponse {
                destination_disclosure_wire_schema_version:
                    DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION,
                disclosure: Some(browser_destination_disclosure(
                    &create_destination_disclosure(&analysis, level),
                )),
                error: None,
            },
            Err(source) => BrowserDestinationDisclosureResponse {
                destination_disclosure_wire_schema_version:
                    DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION,
                disclosure: None,
                error: Some(browser_error(&source)),
            },
        }
    };

    serialize_destination_disclosure_response(response)
}

/// WebAssembly entry point. Browser runtime execution is not covered by native tests.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = analyzeUrl)]
pub fn analyze_url_wasm(input: &str) -> String {
    analyze_for_browser_json(input)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = assessUrl)]
pub fn assess_url_wasm(input: &str, context_json: &str) -> String {
    assess_for_browser_json(input, context_json)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = discloseDestinations)]
pub fn disclose_destinations_wasm(input: &str, level: &str) -> String {
    disclose_destinations_for_browser_json(input, level)
}

fn browser_destination_disclosure(
    disclosure: &DestinationDisclosure,
) -> BrowserDestinationDisclosure {
    BrowserDestinationDisclosure {
        contract_version: disclosure.schema_version(),
        level: disclosure.level().identifier(),
        notice: match disclosure.level() {
            DestinationDisclosureLevel::Hostnames => {
                "Clue identified addresses inside the link. These are possible destinations, not verified websites the link will open."
            }
            DestinationDisclosureLevel::Detailed => {
                "Detailed information can include private data from the original link. These are possible destinations, not verified websites the link will open."
            }
        },
        candidates: disclosure
            .candidates()
            .iter()
            .map(|candidate| BrowserDestinationCandidate {
                identifier: candidate.identifier().to_string(),
                hostname: candidate.hostname().to_string(),
                reference_kind: destination_kind_identifier(candidate.destination_kind()),
                relationship: relationship_identifier(candidate.depth()),
                depth: candidate.depth(),
                parent_candidate_identifier: candidate
                    .parent_candidate_identifier()
                    .map(str::to_string),
                detail: candidate.detail().map(|detail| BrowserDestinationDetail {
                    submitted_reference: detail
                        .submitted_reference_without_credentials_or_fragment()
                        .to_string(),
                    normalized_reference: detail
                        .normalized_reference_without_credentials_or_fragment()
                        .to_string(),
                    source_parameter: detail.source_parameter().to_string(),
                    top_level_parameter_position: detail.top_level_parameter_position(),
                    source_parameter_position: detail.source_parameter_position(),
                    transformations: detail
                        .transformations()
                        .iter()
                        .map(browser_destination_transformation)
                        .collect(),
                }),
            })
            .collect(),
        ambiguities: disclosure
            .ambiguities()
            .iter()
            .map(|ambiguity| BrowserDestinationAmbiguity {
                identifier: ambiguity.identifier().to_string(),
                explanation: ambiguity.explanation().to_string(),
                relationship: relationship_identifier(ambiguity.depth()),
                depth: ambiguity.depth(),
                parent_candidate_identifier: ambiguity
                    .parent_candidate_identifier()
                    .map(str::to_string),
                top_level_parameter_position: ambiguity.top_level_parameter_position(),
                source_parameter_position: ambiguity.source_parameter_position(),
            })
            .collect(),
    }
}

fn destination_kind_identifier(kind: ConsumerDestinationKind) -> &'static str {
    match kind {
        ConsumerDestinationKind::AbsoluteHttp => "absolute-http",
        ConsumerDestinationKind::AbsoluteHttps => "absolute-https",
        ConsumerDestinationKind::ProtocolRelative => "protocol-relative",
        ConsumerDestinationKind::HostnameFirst => "hostname-first",
        _ => "unknown-future-kind",
    }
}

const fn relationship_identifier(depth: usize) -> &'static str {
    if depth == 0 { "direct" } else { "nested" }
}

fn browser_destination_transformation(
    transformation: &ConsumerTransformation,
) -> BrowserDestinationTransformation {
    match transformation {
        ConsumerTransformation::PercentDecoding { passes } => BrowserDestinationTransformation {
            identifier: "percent-decoding",
            explanation: format!("Applied {passes} bounded percent-decoding pass(es)."),
        },
        ConsumerTransformation::Base64Decoding {
            alphabet_identifier,
            padding_identifier,
        } => BrowserDestinationTransformation {
            identifier: "base64-decoding",
            explanation: format!(
                "Applied bounded Base64 decoding ({alphabet_identifier}, {padding_identifier})."
            ),
        },
        ConsumerTransformation::HtmlCharacterReferenceDecoding { reference_count } => {
            BrowserDestinationTransformation {
                identifier: "html-character-reference-decoding",
                explanation: format!(
                    "Interpreted {reference_count} supported HTML character reference(s)."
                ),
            }
        }
        ConsumerTransformation::CompoundQueryExtraction { parameter_position } => {
            BrowserDestinationTransformation {
                identifier: "compound-query-extraction",
                explanation: format!(
                    "Extracted supported inner query parameter {parameter_position}."
                ),
            }
        }
        _ => BrowserDestinationTransformation {
            identifier: "unknown-future-transformation",
            explanation: "A supported transformation was applied.".to_string(),
        },
    }
}

fn browser_report(report: &ConsumerReport, assessment: &LinkAssessment) -> BrowserReport {
    let observations = report
        .observations()
        .iter()
        .map(|observation| BrowserObservation {
            identifier: observation.identifier().to_string(),
            title: observation.title().to_string(),
            explanation: observation.explanation().to_string(),
            occurrence_count: observation.occurrence_count(),
            evidence_references: observation
                .evidence_references()
                .iter()
                .map(|reference| reference.identifier().to_string())
                .collect(),
        })
        .collect();
    let stages = report
        .completeness()
        .stages()
        .iter()
        .map(|stage| BrowserCompletenessStage {
            identifier: stage.stage().identifier().to_string(),
            status: stage_status_identifier(stage.status()).to_string(),
            reasons: stage
                .reasons()
                .iter()
                .map(|reason| BrowserCompletenessReason {
                    identifier: reason.identifier().to_string(),
                    explanation: reason.explanation().to_string(),
                })
                .collect(),
        })
        .collect();

    BrowserReport {
        consumer_schema_version: report.schema_version(),
        disclosure: "privacy-reduced",
        observations,
        completeness: BrowserCompleteness {
            status: completeness_status_identifier(report.completeness().status()).to_string(),
            stages,
        },
        assessment: browser_assessment(assessment),
    }
}

fn browser_assessment(assessment: &LinkAssessment) -> BrowserAssessment {
    BrowserAssessment {
        contract_version: assessment.contract_version(),
        category_identifier: assessment.category().identifier().to_string(),
        category_title: assessment.category().title().to_string(),
        category_explanation: assessment.category().explanation().to_string(),
        action_identifier: assessment.recommended_action().identifier().to_string(),
        action_title: assessment.recommended_action().title().to_string(),
        action_explanation: assessment.recommended_action().explanation().to_string(),
        next_steps: assessment
            .recommended_next_steps()
            .iter()
            .map(|step| BrowserAssessmentNextStep {
                identifier: step.identifier().to_string(),
                explanation: step.explanation().to_string(),
            })
            .collect(),
        reasons: assessment
            .reasons()
            .iter()
            .map(|reason| BrowserAssessmentReason {
                identifier: reason.identifier().to_string(),
                source: reason.source().identifier().to_string(),
                explanation: reason.explanation().to_string(),
            })
            .collect(),
        limitations: assessment
            .limitations()
            .iter()
            .map(|limitation| BrowserAssessmentLimitation {
                identifier: limitation.identifier().to_string(),
                explanation: limitation.explanation().to_string(),
            })
            .collect(),
        context_changed_recommendation: assessment.context_changed_recommendation(),
    }
}

fn assessment_context(context: BrowserAssessmentContext) -> AssessmentContext {
    let mut result = AssessmentContext::new();
    if let Some(source) = context.source {
        result = result.with_source(match source {
            BrowserLinkSource::Email => LinkSource::Email,
            BrowserLinkSource::Message => LinkSource::Message,
            BrowserLinkSource::SocialMedia => LinkSource::SocialMedia,
            BrowserLinkSource::WebsiteOrSearch => LinkSource::WebsiteOrSearch,
            BrowserLinkSource::QrCode => LinkSource::QrCode,
            BrowserLinkSource::OtherOrUnknown => LinkSource::OtherOrUnknown,
        });
    }
    if let Some(purpose) = context.claimed_purpose {
        result = result.with_claimed_purpose(match purpose {
            BrowserClaimedPurpose::Document => ClaimedPurpose::Document,
            BrowserClaimedPurpose::AccountSignInOrVerification => {
                ClaimedPurpose::AccountSignInOrVerification
            }
            BrowserClaimedPurpose::DeliveryOrShipping => ClaimedPurpose::DeliveryOrShipping,
            BrowserClaimedPurpose::PaymentOrFinancialRequest => {
                ClaimedPurpose::PaymentOrFinancialRequest
            }
            BrowserClaimedPurpose::OrdinaryContent => ClaimedPurpose::OrdinaryContent,
            BrowserClaimedPurpose::SoftwareDownloadOrInstall => {
                ClaimedPurpose::SoftwareDownloadOrInstall
            }
            BrowserClaimedPurpose::OtherOrUnknown => ClaimedPurpose::OtherOrUnknown,
        });
    }
    if let Some(expected) = context.expected_contact {
        result = result.with_expected_contact(match expected {
            BrowserExpectedContact::Expected => ExpectedContact::Expected,
            BrowserExpectedContact::Unexpected => ExpectedContact::Unexpected,
            BrowserExpectedContact::Unsure => ExpectedContact::Unsure,
        });
    }
    if let Some(action) = context.requested_action {
        result = result.with_requested_action(match action {
            BrowserRequestedAction::ViewContent => RequestedAction::ViewContent,
            BrowserRequestedAction::EnterAccountCredentials => {
                RequestedAction::EnterAccountCredentials
            }
            BrowserRequestedAction::ProvidePersonalInformation => {
                RequestedAction::ProvidePersonalInformation
            }
            BrowserRequestedAction::MakePayment => RequestedAction::MakePayment,
            BrowserRequestedAction::DownloadOrInstallSoftware => {
                RequestedAction::DownloadOrInstallSoftware
            }
            BrowserRequestedAction::ContactSomeone => RequestedAction::ContactSomeone,
            BrowserRequestedAction::OtherOrUnknown => RequestedAction::OtherOrUnknown,
        });
    }
    result
}

fn serialize_response(response: BrowserResponse) -> String {
    serde_json::to_string(&response).unwrap_or_else(|_| fixed_serialization_failure().to_string())
}

fn serialize_destination_disclosure_response(
    response: BrowserDestinationDisclosureResponse,
) -> String {
    serde_json::to_string(&response)
        .unwrap_or_else(|_| fixed_destination_serialization_failure().to_string())
}

fn completeness_status_identifier(status: ConsumerCompletenessStatus) -> &'static str {
    match status {
        ConsumerCompletenessStatus::CompleteWithinSupportedScope => {
            "complete-within-supported-scope"
        }
        ConsumerCompletenessStatus::PartiallyExamined => "partially-examined",
        _ => "unknown-future-status",
    }
}

fn stage_status_identifier(status: ConsumerStageStatus) -> &'static str {
    match status {
        ConsumerStageStatus::CompleteWithinSupportedScope => "complete-within-supported-scope",
        ConsumerStageStatus::PartiallyExamined => "partially-examined",
        ConsumerStageStatus::UnsupportedEvidence => "unsupported-evidence",
        ConsumerStageStatus::ResourceLimited => "resource-limited",
        _ => "unknown-future-status",
    }
}

fn browser_error(source: &UrlError) -> BrowserError {
    match source {
        UrlError::TooLong => error(
            BrowserErrorCode::InputTooLong,
            "The URL exceeds the supported input length.",
        ),
        UrlError::TooManyQueryParameters => error(
            BrowserErrorCode::TooManyQueryParameters,
            "The URL contains more query parameters than the analyzer supports.",
        ),
        UrlError::UnsupportedScheme => error(
            BrowserErrorCode::UnsupportedScheme,
            "Only HTTP and HTTPS URLs are supported.",
        ),
        UrlError::Invalid(_) | UrlError::MissingHostname => error(
            BrowserErrorCode::InvalidUrl,
            "Enter a complete HTTP or HTTPS URL.",
        ),
    }
}

const fn error(code: BrowserErrorCode, message: &'static str) -> BrowserError {
    BrowserError { code, message }
}

const fn fixed_serialization_failure() -> &'static str {
    concat!(
        r#"{"wire_schema_version":2,"report":null,"error":{"code":"analysis-failure","#,
        r#""message":"LinkSleuth could not prepare the local analysis result."}}"#
    )
}

const fn fixed_destination_serialization_failure() -> &'static str {
    concat!(
        r#"{"destination_disclosure_wire_schema_version":1,"disclosure":null,"error":{"code":"analysis-failure","#,
        r#""message":"Clue could not prepare the local destination disclosure."}}"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use linksleuth_core::consumer::CONSUMER_REPORT_SCHEMA_VERSION;
    use serde_json::Value;

    #[test]
    fn browser_report_is_versioned_and_privacy_reduced() {
        let input = concat!(
            "https://private.example/private/path?token=fake-secret-123",
            "&target=https%3A%2F%2Fdestination.example%2Fprivate"
        );
        let response = analyze_for_browser(input);
        let report = response.report().unwrap();

        assert_eq!(response.wire_schema_version(), BROWSER_WIRE_SCHEMA_VERSION);
        assert_eq!(
            report.consumer_schema_version(),
            CONSUMER_REPORT_SCHEMA_VERSION
        );
        assert_eq!(report.disclosure(), "privacy-reduced");
        assert!(response.error().is_none());
        assert_eq!(report.observations().len(), 1);
        assert_eq!(
            report.observations()[0].identifier(),
            "destination.absolute-candidate"
        );
        assert_eq!(report.observations()[0].occurrence_count(), 1);
        assert!(report.observations()[0].evidence_references().is_empty());
        assert_eq!(report.completeness().stages().len(), 4);

        let json = analyze_for_browser_json(input);
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["wire_schema_version"], 2);
        assert_eq!(value["report"]["consumer_schema_version"], 1);
        assert_eq!(value["report"]["assessment"]["contract_version"], 1);
        assert_eq!(
            value["report"]["assessment"]["action_title"],
            "Verify this link before opening it."
        );
        assert_eq!(
            value["report"]["assessment"]["next_steps"][0]["identifier"],
            "use-known-website-or-app"
        );
        assert!(value["report"].get("hostname").is_none());
        assert!(value["report"].get("evidence").is_none());
        for sensitive in [
            "private.example",
            "destination.example",
            "fake-secret-123",
            "token",
            "/private",
        ] {
            assert!(!json.contains(sensitive));
        }
    }

    #[test]
    fn hostname_disclosure_contains_only_bounded_candidate_identity() {
        let input = concat!(
            "https://private.example/?email=fake.person%40example.test&target=",
            "https%3A%2F%2Fdestination.example.%2Fprivate%3Ftoken%3Dfake-token%23private-fragment"
        );
        let json = disclose_destinations_for_browser_json(input, "hostnames");
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["destination_disclosure_wire_schema_version"], 1);
        assert_eq!(value["disclosure"]["contract_version"], 1);
        assert_eq!(value["disclosure"]["level"], "hostnames");
        assert_eq!(
            value["disclosure"]["candidates"].as_array().unwrap().len(),
            1
        );
        let candidate = &value["disclosure"]["candidates"][0];
        assert_eq!(candidate["identifier"], "candidate-1");
        assert_eq!(candidate["hostname"], "destination.example.");
        assert_eq!(candidate["reference_kind"], "absolute-https");
        assert_eq!(candidate["relationship"], "direct");
        assert!(candidate.get("detail").is_none());
        for private in [
            "private.example",
            "fake.person",
            "fake-token",
            "private-fragment",
            "source_parameter",
            "submitted_reference",
            "normalized_reference",
        ] {
            assert!(!json.contains(private));
        }
    }

    #[test]
    fn detailed_disclosure_is_separate_fragment_reduced_and_provenance_preserving() {
        let input = concat!(
            "https://top.example/?target=",
            "https%3A%2F%2Ffake-user%3Afake-password%40destination.example%2Fprivate%3Ftoken%3Dfake-token",
            "%26next%3D%252F%252Fnested.example%252Flanding%23private-fragment"
        );
        let default_json = analyze_for_browser_json(input);
        let detailed_json = disclose_destinations_for_browser_json(input, "detailed");
        let value: Value = serde_json::from_str(&detailed_json).unwrap();
        let candidates = value["disclosure"]["candidates"].as_array().unwrap();

        assert!(!default_json.contains("destination.example"));
        assert!(!default_json.contains("nested.example"));
        assert!(!default_json.contains("fake-token"));
        assert_eq!(value["disclosure"]["level"], "detailed");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0]["detail"]["source_parameter"], "target");
        assert_eq!(candidates[0]["detail"]["top_level_parameter_position"], 1);
        assert_eq!(candidates[1]["relationship"], "nested");
        assert_eq!(candidates[1]["depth"], 1);
        assert_eq!(candidates[1]["parent_candidate_identifier"], "candidate-1");
        assert!(
            candidates[0]["detail"]["transformations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["identifier"] == "percent-decoding")
        );
        assert!(detailed_json.contains("fake-token"));
        assert!(!detailed_json.contains("private-fragment"));
        assert!(!detailed_json.contains("fake-user"));
        assert!(!detailed_json.contains("fake-password"));
        assert!(!detailed_json.contains("https://top.example"));
    }

    #[test]
    fn disclosure_ambiguity_is_not_promoted_and_invalid_arguments_are_private() {
        let input = concat!(
            "https://social.example/?target=",
            "tracking.example%2F%2Fpath%3Fadurl%3D%2F%2Fstorage.example%2Flanding"
        );
        let hostname_json = disclose_destinations_for_browser_json(input, "hostnames");
        let hostname_value: Value = serde_json::from_str(&hostname_json).unwrap();
        let ambiguities = hostname_value["disclosure"]["ambiguities"]
            .as_array()
            .unwrap();
        assert_eq!(ambiguities.len(), 1);
        assert!(
            ambiguities[0]["explanation"]
                .as_str()
                .unwrap()
                .contains("not classified as a destination")
        );
        assert!(ambiguities[0].get("top_level_parameter_position").is_none());
        assert!(!hostname_json.contains("tracking.example"));

        let invalid = disclose_destinations_for_browser_json(
            "https://private.example/?token=fake-secret",
            "everything-fake-secret",
        );
        let invalid_value: Value = serde_json::from_str(&invalid).unwrap();
        assert_eq!(invalid_value["disclosure"], Value::Null);
        assert_eq!(invalid_value["error"]["code"], "invalid-disclosure-level");
        assert!(!invalid.contains("private.example"));
        assert!(!invalid.contains("fake-secret"));
        assert!(!invalid.contains("everything-fake-secret"));
    }

    #[test]
    fn browser_report_preserves_stage_specific_incompleteness() {
        let response = analyze_for_browser("https:////private.example/path?category=books");
        let report = response.report().unwrap();

        assert_eq!(report.completeness().status(), "partially-examined");
        let original_path = report
            .completeness()
            .stages()
            .iter()
            .find(|stage| stage.identifier() == "original-path")
            .unwrap();
        assert_eq!(original_path.status(), "unsupported-evidence");
        assert_eq!(
            original_path.reasons()[0].identifier(),
            "original-path-ambiguous-authority-boundary"
        );
        assert!(!original_path.reasons()[0].explanation().is_empty());
    }

    #[test]
    fn compound_example_exposes_only_fixed_structural_findings() {
        let input = concat!(
            "https://social.example/link/v2?aid=1988&scene=bio_url&target=",
            "tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=",
            "%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html",
            "#synthetic-fragment"
        );
        let json = analyze_for_browser_json(input);
        let value: Value = serde_json::from_str(&json).unwrap();
        let identifiers = value["report"]["observations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|observation| observation["identifier"].as_str().unwrap())
            .collect::<Vec<_>>();

        for expected in [
            "destination.protocol-relative-candidate",
            "structure.encoded-query-delimiter",
            "structure.compound-query-value",
            "structure.hostname-first-ambiguous",
        ] {
            assert!(identifiers.contains(&expected));
        }
        let traversal = value["report"]["completeness"]["stages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|stage| stage["identifier"] == "destination-traversal")
            .unwrap();
        assert_eq!(traversal["status"], "partially-examined");
        assert_eq!(
            traversal["reasons"][0]["identifier"],
            "destination-hostname-first-ambiguous"
        );
        for sensitive in [
            "social.example",
            "tracking.example",
            "storage.example",
            "fake-id",
            "adurl",
            "synthetic-fragment",
        ] {
            assert!(!json.contains(sensitive));
        }
    }

    #[test]
    fn no_observations_do_not_add_a_safety_verdict() {
        let json = analyze_for_browser_json("https://ordinary.example/products?category=books");
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["report"]["observations"], Value::Array(Vec::new()));
        assert!(value["report"].get("verdict").is_none());
        assert!(!json.to_lowercase().contains("malicious"));
        assert!(!json.to_lowercase().contains("url is safe"));
        assert!(!json.contains("ordinary.example"));
        assert!(!json.contains("products"));
        assert!(!json.contains("books"));
    }

    #[test]
    fn browser_errors_are_fixed_and_do_not_reflect_input() {
        let cases = [
            ("", BrowserErrorCode::EmptyInput, "Enter a URL to analyze."),
            (
                "not a URL fake-secret-123",
                BrowserErrorCode::InvalidUrl,
                "Enter a complete HTTP or HTTPS URL.",
            ),
            (
                "ftp://fake-secret-123.example/private",
                BrowserErrorCode::UnsupportedScheme,
                "Only HTTP and HTTPS URLs are supported.",
            ),
        ];

        for (input, expected_code, expected_message) in cases {
            let response = analyze_for_browser(input);
            let error = response.error().unwrap();
            assert_eq!(error.code(), expected_code);
            assert_eq!(error.message(), expected_message);
            assert!(response.report().is_none());

            let json = analyze_for_browser_json(input);
            assert!(!json.contains("fake-secret-123"));
            assert!(!json.contains("/private"));
        }
    }

    #[test]
    fn browser_errors_cover_supported_resource_limits() {
        let oversized = format!("https://example.com/{}", "x".repeat(16 * 1024));
        let oversized_response = analyze_for_browser(&oversized);
        let oversized_error = oversized_response.error().unwrap();
        assert_eq!(oversized_error.code(), BrowserErrorCode::InputTooLong);
        assert_eq!(
            oversized_error.message(),
            "The URL exceeds the supported input length."
        );
        assert!(!analyze_for_browser_json(&oversized).contains(&"x".repeat(64)));

        let parameters = (0..257)
            .map(|index| format!("p{index}=value"))
            .collect::<Vec<_>>()
            .join("&");
        let excessive_parameters = format!("https://example.com/?{parameters}");
        let parameter_response = analyze_for_browser(&excessive_parameters);
        let parameter_error = parameter_response.error().unwrap();
        assert_eq!(
            parameter_error.code(),
            BrowserErrorCode::TooManyQueryParameters
        );
        assert_eq!(
            parameter_error.message(),
            "The URL contains more query parameters than the analyzer supports."
        );

        let fallback: Value = serde_json::from_str(fixed_serialization_failure()).unwrap();
        assert_eq!(fallback["error"]["code"], "analysis-failure");
        assert_eq!(fallback["report"], Value::Null);
    }

    #[test]
    fn adapter_source_has_no_network_client_usage() {
        let source = concat!(include_str!("lib.rs"), include_str!("../Cargo.toml"));
        for forbidden in [
            concat!("req", "west"),
            concat!("hy", "per"),
            concat!("std::net::", "TcpStream"),
            concat!("std::net::", "UdpSocket"),
            concat!("web_sys::", "Request"),
            concat!("web_sys::", "WebSocket"),
        ] {
            assert!(
                !source.contains(forbidden),
                "network API found: {forbidden}"
            );
        }
    }

    #[test]
    fn context_reassessment_changes_guidance_without_changing_observations() {
        let input = "https://top.example/?target=https%3A%2F%2Fdestination.example%2Fcontent";
        let preliminary: Value = serde_json::from_str(&analyze_for_browser_json(input)).unwrap();
        let contextual: Value = serde_json::from_str(&assess_for_browser_json(
            input,
            r#"{"source":"email","claimed_purpose":"ordinary-content","expected_contact":"expected","requested_action":"enter-account-credentials"}"#,
        ))
        .unwrap();

        assert_eq!(
            preliminary["report"]["assessment"]["category_identifier"],
            "proceed-with-caution"
        );
        assert_eq!(
            preliminary["report"]["assessment"]["action_identifier"],
            "use-known-website-instead"
        );
        assert_eq!(
            contextual["report"]["assessment"]["category_identifier"],
            "proceed-with-caution"
        );
        assert_eq!(
            contextual["report"]["assessment"]["action_identifier"],
            "avoid-sensitive-action-until-verified"
        );
        assert_eq!(
            preliminary["report"]["observations"],
            contextual["report"]["observations"]
        );
        assert_eq!(
            preliminary["report"]["completeness"],
            contextual["report"]["completeness"]
        );
        assert_eq!(
            contextual["report"]["assessment"]["context_changed_recommendation"],
            true
        );
    }

    #[test]
    fn unexpected_document_context_produces_decisive_private_guidance() {
        let input = concat!(
            "https://social.example/link/v2?target=",
            "tracking.example%2F%2Fpath%3Fadurl%3D%2F%2Fstorage.example%2Flanding"
        );
        let json = assess_for_browser_json(
            input,
            r#"{"source":"email","claimed_purpose":"document","expected_contact":"unexpected","requested_action":"view-content"}"#,
        );
        let value: Value = serde_json::from_str(&json).unwrap();
        let assessment = &value["report"]["assessment"];

        assert_eq!(assessment["category_identifier"], "strong-warning-signs");
        assert_eq!(assessment["action_identifier"], "do-not-open-link");
        assert_eq!(assessment["action_title"], "Do not open this link");
        assert_eq!(assessment["next_steps"].as_array().unwrap().len(), 3);
        assert_eq!(
            assessment["next_steps"][1]["identifier"],
            "verify-document-request-through-trusted-channel"
        );
        assert!(json.contains("consistent with phishing"));
        for sensitive in [
            "social.example",
            "tracking.example",
            "storage.example",
            "adurl",
        ] {
            assert!(!json.contains(sensitive));
        }
    }

    #[test]
    fn invalid_or_oversized_context_returns_a_fixed_private_error() {
        for context in [
            r#"{"source":"private-secret-value"}"#.to_string(),
            "x".repeat(MAX_ASSESSMENT_CONTEXT_BYTES + 1),
        ] {
            let json = assess_for_browser_json("https://private.example/path", &context);
            let value: Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["error"]["code"], "invalid-context");
            assert_eq!(
                value["error"]["message"],
                "Clue could not use the optional context answers."
            );
            assert!(!json.contains("private.example"));
            assert!(!json.contains("private-secret-value"));
        }
    }
}
