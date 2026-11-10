//! Deterministic, local interpretation of privacy-reduced LinkSleuth reports.
//!
//! Assessment is deliberately separate from URL analysis. It consumes only
//! the public consumer report, groups correlated observations into evidence
//! families, and never fetches or verifies a destination.

use std::fmt;

use linksleuth_core::consumer::{
    ConsumerAnalysisStage, ConsumerObservationKind, ConsumerReport, ConsumerStageStatus,
};

pub const ASSESSMENT_CONTRACT_VERSION: u32 = 1;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkSource {
    Email,
    Message,
    SocialMedia,
    WebsiteOrSearch,
    QrCode,
    OtherOrUnknown,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimedPurpose {
    Document,
    AccountSignInOrVerification,
    DeliveryOrShipping,
    PaymentOrFinancialRequest,
    OrdinaryContent,
    SoftwareDownloadOrInstall,
    OtherOrUnknown,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedContact {
    Expected,
    Unexpected,
    Unsure,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestedAction {
    ViewContent,
    EnterAccountCredentials,
    ProvidePersonalInformation,
    MakePayment,
    DownloadOrInstallSoftware,
    ContactSomeone,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AssessmentContext {
    source: Option<LinkSource>,
    claimed_purpose: Option<ClaimedPurpose>,
    expected_contact: Option<ExpectedContact>,
    requested_action: Option<RequestedAction>,
}

impl AssessmentContext {
    pub const fn new() -> Self {
        Self {
            source: None,
            claimed_purpose: None,
            expected_contact: None,
            requested_action: None,
        }
    }

    pub const fn with_source(mut self, source: LinkSource) -> Self {
        self.source = Some(source);
        self
    }

    pub const fn with_claimed_purpose(mut self, purpose: ClaimedPurpose) -> Self {
        self.claimed_purpose = Some(purpose);
        self
    }

    pub const fn with_expected_contact(mut self, expected: ExpectedContact) -> Self {
        self.expected_contact = Some(expected);
        self
    }

    pub const fn with_requested_action(mut self, action: RequestedAction) -> Self {
        self.requested_action = Some(action);
        self
    }

    pub const fn source(self) -> Option<LinkSource> {
        self.source
    }

    pub const fn claimed_purpose(self) -> Option<ClaimedPurpose> {
        self.claimed_purpose
    }

    pub const fn expected_contact(self) -> Option<ExpectedContact> {
        self.expected_contact
    }

    pub const fn requested_action(self) -> Option<RequestedAction> {
        self.requested_action
    }

    pub const fn is_empty(self) -> bool {
        self.source.is_none()
            && self.claimed_purpose.is_none()
            && self.expected_contact.is_none()
            && self.requested_action.is_none()
    }

    pub const fn is_partial(self) -> bool {
        !self.is_empty()
            && (self.source.is_none()
                || self.claimed_purpose.is_none()
                || self.expected_contact.is_none()
                || self.requested_action.is_none())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentCategory {
    StrongWarningSigns,
    ProceedWithCaution,
    NoClearStructuralWarningSigns,
    UnableToAssessReliably,
}

impl AssessmentCategory {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::StrongWarningSigns => "strong-warning-signs",
            Self::ProceedWithCaution => "proceed-with-caution",
            Self::NoClearStructuralWarningSigns => "no-clear-structural-warning-signs",
            Self::UnableToAssessReliably => "unable-to-assess-reliably",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::StrongWarningSigns => "Strong warning signs",
            Self::ProceedWithCaution => "Proceed with caution",
            Self::NoClearStructuralWarningSigns => "No clear structural warning signs",
            Self::UnableToAssessReliably => "Unable to assess reliably",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::StrongWarningSigns => {
                "Several meaningfully different indicators support avoiding this link until the request is independently verified. These warning signs are consistent with phishing, but Clue has not confirmed malicious intent."
            }
            Self::ProceedWithCaution => {
                "One or more structural or contextual concerns warrant independent verification before relying on this link."
            }
            Self::NoClearStructuralWarningSigns => {
                "Clue found no meaningful warning signs within its supported checks. This does not establish that the website is trustworthy or safe."
            }
            Self::UnableToAssessReliably => {
                "Important supported analysis was incomplete or unavailable, so Clue cannot offer a reliable structural assessment."
            }
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedAction {
    DoNotOpenLink,
    ContinueCarefully,
    VerifyIndependently,
    VerifySenderIndependently,
    UseKnownWebsiteInstead,
    AvoidSensitiveActionUntilVerified,
    SeekAdditionalInformation,
}

impl RecommendedAction {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::DoNotOpenLink => "do-not-open-link",
            Self::ContinueCarefully => "continue-carefully",
            Self::VerifyIndependently => "verify-independently",
            Self::VerifySenderIndependently => "verify-sender-independently",
            Self::UseKnownWebsiteInstead => "use-known-website-instead",
            Self::AvoidSensitiveActionUntilVerified => "avoid-sensitive-action-until-verified",
            Self::SeekAdditionalInformation => "seek-additional-information",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::DoNotOpenLink => "Do not open this link",
            Self::ContinueCarefully => "No obvious structural warning signs found.",
            Self::VerifyIndependently
            | Self::VerifySenderIndependently
            | Self::UseKnownWebsiteInstead => "Verify this link before opening it.",
            Self::AvoidSensitiveActionUntilVerified => {
                "Do not continue until this request is verified."
            }
            Self::SeekAdditionalInformation => "Clue cannot reliably assess this link.",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::DoNotOpenLink => {
                "This request has several warning signs consistent with phishing. Avoid the link until you independently verify the request."
            }
            Self::ContinueCarefully => {
                "Clue did not find an obvious warning sign in the supported URL structures. This is not evidence that the link is safe or trustworthy."
            }
            Self::VerifyIndependently
            | Self::VerifySenderIndependently
            | Self::UseKnownWebsiteInstead => {
                "Clue found a reason to pause. Verify the request through a separate trusted source before relying on this link."
            }
            Self::AvoidSensitiveActionUntilVerified => {
                "The available evidence does not justify continuing with a sensitive request until it is independently verified."
            }
            Self::SeekAdditionalInformation => {
                "Clue could not reliably examine part of this link. Do not assume that missing findings mean it is safe."
            }
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedNextStep {
    AvoidOpeningLink,
    VerifyRequestIndependently,
    VerifySenderThroughTrustedChannel,
    VerifyDocumentRequestThroughTrustedChannel,
    UseKnownWebsiteOrApp,
    AvoidReplyingToEstablishTrust,
    AvoidCredentialsThroughLink,
    AvoidPaymentThroughLink,
    AvoidPersonalInformationThroughLink,
    AvoidSoftwareInstallThroughLink,
    ConfirmRequestMakesSense,
    TreatIncompleteAnalysisWithCaution,
}

impl RecommendedNextStep {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::AvoidOpeningLink => "avoid-opening-link",
            Self::VerifyRequestIndependently => "verify-request-independently",
            Self::VerifySenderThroughTrustedChannel => "verify-sender-through-trusted-channel",
            Self::VerifyDocumentRequestThroughTrustedChannel => {
                "verify-document-request-through-trusted-channel"
            }
            Self::UseKnownWebsiteOrApp => "use-known-website-or-app",
            Self::AvoidReplyingToEstablishTrust => "avoid-replying-to-establish-trust",
            Self::AvoidCredentialsThroughLink => "avoid-credentials-through-link",
            Self::AvoidPaymentThroughLink => "avoid-payment-through-link",
            Self::AvoidPersonalInformationThroughLink => "avoid-personal-information-through-link",
            Self::AvoidSoftwareInstallThroughLink => "avoid-software-install-through-link",
            Self::ConfirmRequestMakesSense => "confirm-request-makes-sense",
            Self::TreatIncompleteAnalysisWithCaution => "treat-incomplete-analysis-with-caution",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::AvoidOpeningLink => "Do not open this link.",
            Self::VerifyRequestIndependently => {
                "Verify the request using a source or contact method you already trust."
            }
            Self::VerifySenderThroughTrustedChannel => {
                "Contact the supposed sender using contact information you already trust."
            }
            Self::VerifyDocumentRequestThroughTrustedChannel => {
                "Contact the supposed sender through a trusted channel before opening or signing anything."
            }
            Self::UseKnownWebsiteOrApp => {
                "Open the service's known website or app yourself instead of using this link."
            }
            Self::AvoidReplyingToEstablishTrust => {
                "Do not rely on a reply to the same message to establish that the request is genuine."
            }
            Self::AvoidCredentialsThroughLink => {
                "Do not enter your password through this link. Open the service's website or app yourself."
            }
            Self::AvoidPaymentThroughLink => {
                "Do not provide payment information through this link until the request is independently verified."
            }
            Self::AvoidPersonalInformationThroughLink => {
                "Do not provide personal information through this link until the request is independently verified."
            }
            Self::AvoidSoftwareInstallThroughLink => {
                "Do not download or install software through this link until the request is independently verified."
            }
            Self::ConfirmRequestMakesSense => {
                "Confirm that the request and destination make sense before continuing."
            }
            Self::TreatIncompleteAnalysisWithCaution => {
                "Clue could not reliably examine part of this link. Verify the request independently rather than assuming the absence of findings means it is safe."
            }
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentReasonSource {
    UrlAnalysis,
    UserContext,
}

impl AssessmentReasonSource {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::UrlAnalysis => "url-analysis",
            Self::UserContext => "user-context",
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentReasonKind {
    EmbeddedOrNestedDestination,
    LayeredOrCompoundEncoding,
    AmbiguousDestinationInterpretation,
    AnalysisIncomplete,
    UnexpectedContact,
    UnexpectedDocumentRequest,
    UncertainContact,
    SensitiveClaimedPurpose,
    SensitiveRequestedAction,
}

impl AssessmentReasonKind {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::EmbeddedOrNestedDestination => "url.destination-transparency-reduced",
            Self::LayeredOrCompoundEncoding => "url.layered-or-compound-structure",
            Self::AmbiguousDestinationInterpretation => "url.destination-interpretation-ambiguous",
            Self::AnalysisIncomplete => "url.analysis-incomplete",
            Self::UnexpectedContact => "context.contact-unexpected",
            Self::UnexpectedDocumentRequest => "context.document-request-unexpected",
            Self::UncertainContact => "context.contact-uncertain",
            Self::SensitiveClaimedPurpose => "context.sensitive-purpose",
            Self::SensitiveRequestedAction => "context.sensitive-action-requested",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::EmbeddedOrNestedDestination => {
                "The link contains a supported embedded, nested, or intermediary address structure. Clue did not verify where navigation would lead."
            }
            Self::LayeredOrCompoundEncoding => {
                "Related encoding and compound-query observations reduce how directly the destination is presented. Encoding alone does not establish harmful intent."
            }
            Self::AmbiguousDestinationInterpretation => {
                "Part of the destination-like structure could not be classified confidently and was not repaired or promoted to a destination."
            }
            Self::AnalysisIncomplete => {
                "At least one supported analysis stage was unavailable or reached a limit. Missing findings must not be treated as reassurance."
            }
            Self::UnexpectedContact => {
                "You reported that the contact or request was unexpected. This is user-provided context, not independently verified evidence."
            }
            Self::UnexpectedDocumentRequest => {
                "You reported receiving an unexpected request to sign or view a document. Clue has not verified who sent it."
            }
            Self::UncertainContact => {
                "You reported being unsure whether the contact was expected. Independent verification can reduce that uncertainty."
            }
            Self::SensitiveClaimedPurpose => {
                "You reported a purpose involving an account, payment, software installation, or another sensitive interaction."
            }
            Self::SensitiveRequestedAction => {
                "You reported a request for credentials, personal information, payment, or software installation. Clue has not verified that request."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssessmentReason {
    kind: AssessmentReasonKind,
    source: AssessmentReasonSource,
}

impl AssessmentReason {
    pub const fn kind(self) -> AssessmentReasonKind {
        self.kind
    }

    pub const fn source(self) -> AssessmentReasonSource {
        self.source
    }

    pub const fn identifier(self) -> &'static str {
        self.kind.identifier()
    }

    pub const fn explanation(self) -> &'static str {
        self.kind.explanation()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssessmentLimitation {
    identifier: &'static str,
    explanation: &'static str,
}

impl AssessmentLimitation {
    pub const fn identifier(self) -> &'static str {
        self.identifier
    }

    pub const fn explanation(self) -> &'static str {
        self.explanation
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct LinkAssessment {
    contract_version: u32,
    category: AssessmentCategory,
    recommended_action: RecommendedAction,
    recommended_next_steps: Vec<RecommendedNextStep>,
    reasons: Vec<AssessmentReason>,
    limitations: Vec<AssessmentLimitation>,
    context_changed_recommendation: bool,
}

impl LinkAssessment {
    pub const fn contract_version(&self) -> u32 {
        self.contract_version
    }

    pub const fn category(&self) -> AssessmentCategory {
        self.category
    }

    pub const fn recommended_action(&self) -> RecommendedAction {
        self.recommended_action
    }

    pub fn recommended_next_steps(&self) -> &[RecommendedNextStep] {
        &self.recommended_next_steps
    }

    pub fn reasons(&self) -> &[AssessmentReason] {
        &self.reasons
    }

    pub fn limitations(&self) -> &[AssessmentLimitation] {
        &self.limitations
    }

    pub const fn context_changed_recommendation(&self) -> bool {
        self.context_changed_recommendation
    }
}

impl fmt::Debug for LinkAssessment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinkAssessment")
            .field("contract_version", &self.contract_version)
            .field("category", &self.category)
            .field("recommended_action", &self.recommended_action)
            .field(
                "recommended_next_step_identifiers",
                &self
                    .recommended_next_steps
                    .iter()
                    .map(|step| step.identifier())
                    .collect::<Vec<_>>(),
            )
            .field(
                "reason_identifiers",
                &self
                    .reasons
                    .iter()
                    .map(|reason| reason.identifier())
                    .collect::<Vec<_>>(),
            )
            .field(
                "limitation_identifiers",
                &self
                    .limitations
                    .iter()
                    .map(|limitation| limitation.identifier())
                    .collect::<Vec<_>>(),
            )
            .field(
                "context_changed_recommendation",
                &self.context_changed_recommendation,
            )
            .finish()
    }
}

#[derive(Default)]
struct EvidenceFamilies {
    destination_transparency: bool,
    layered_or_compound: bool,
    ambiguity: bool,
    analysis_incomplete: bool,
}

pub fn assess_link(report: &ConsumerReport, context: &AssessmentContext) -> LinkAssessment {
    let families = evidence_families(report);
    let preliminary = evaluate(&families, &AssessmentContext::default());
    let mut assessment = evaluate(&families, context);
    assessment.context_changed_recommendation = !context.is_empty()
        && (assessment.category != preliminary.category
            || assessment.recommended_action != preliminary.recommended_action
            || assessment.recommended_next_steps != preliminary.recommended_next_steps);
    assessment
}

fn evidence_families(report: &ConsumerReport) -> EvidenceFamilies {
    let mut families = EvidenceFamilies::default();

    for observation in report.observations() {
        match observation.kind() {
            ConsumerObservationKind::PotentialAbsoluteDestination
            | ConsumerObservationKind::PotentialProtocolRelativeReference
            | ConsumerObservationKind::PotentialHostnameFirstReference
            | ConsumerObservationKind::NestedDestinationCandidate => {
                families.destination_transparency = true;
            }
            ConsumerObservationKind::EncodedParentDirectoryNormalization
            | ConsumerObservationKind::Base64DerivedDestination
            | ConsumerObservationKind::HtmlReferenceDerivedDestination
            | ConsumerObservationKind::MultiplePercentDecodingTransformations
            | ConsumerObservationKind::EncodedQueryDelimiter
            | ConsumerObservationKind::CompoundQueryValue => {
                families.layered_or_compound = true;
            }
            ConsumerObservationKind::AmbiguousHostnameFirstReference => {
                families.ambiguity = true;
            }
            ConsumerObservationKind::UnsupportedOriginalPathEvidence
            | ConsumerObservationKind::IncompleteDestinationDiscovery
            | ConsumerObservationKind::IncompleteEncodingInterpretation
            | ConsumerObservationKind::PercentDecodingPassLimit => {
                families.analysis_incomplete = true;
            }
            ConsumerObservationKind::MalformedPercentEncoding => {}
            _ => {}
        }
    }

    for stage in report.completeness().stages() {
        match stage.status() {
            ConsumerStageStatus::ResourceLimited | ConsumerStageStatus::UnsupportedEvidence => {
                families.analysis_incomplete = true;
            }
            ConsumerStageStatus::PartiallyExamined
                if stage.stage() != ConsumerAnalysisStage::DestinationTraversal =>
            {
                families.analysis_incomplete = true;
            }
            _ => {}
        }
    }

    families
}

fn evaluate(families: &EvidenceFamilies, context: &AssessmentContext) -> LinkAssessment {
    let unexpected = context.expected_contact == Some(ExpectedContact::Unexpected);
    let uncertain = context.expected_contact == Some(ExpectedContact::Unsure);
    let sensitive_purpose = context.claimed_purpose.is_some_and(|purpose| {
        matches!(
            purpose,
            ClaimedPurpose::AccountSignInOrVerification
                | ClaimedPurpose::PaymentOrFinancialRequest
                | ClaimedPurpose::SoftwareDownloadOrInstall
        )
    });
    let sensitive_action = context.requested_action.is_some_and(|action| {
        matches!(
            action,
            RequestedAction::EnterAccountCredentials
                | RequestedAction::ProvidePersonalInformation
                | RequestedAction::MakePayment
                | RequestedAction::DownloadOrInstallSoftware
        )
    });

    let category = if families.analysis_incomplete {
        AssessmentCategory::UnableToAssessReliably
    } else if (unexpected && sensitive_action)
        || (unexpected
            && families.ambiguity
            && (families.destination_transparency || families.layered_or_compound))
        || (sensitive_action && families.ambiguity && families.layered_or_compound)
    {
        AssessmentCategory::StrongWarningSigns
    } else if families.destination_transparency
        || families.layered_or_compound
        || families.ambiguity
        || unexpected
        || uncertain
        || sensitive_purpose
        || sensitive_action
    {
        AssessmentCategory::ProceedWithCaution
    } else {
        AssessmentCategory::NoClearStructuralWarningSigns
    };

    let recommended_action = match category {
        AssessmentCategory::UnableToAssessReliably => RecommendedAction::SeekAdditionalInformation,
        AssessmentCategory::StrongWarningSigns => RecommendedAction::DoNotOpenLink,
        AssessmentCategory::ProceedWithCaution if sensitive_action => {
            RecommendedAction::AvoidSensitiveActionUntilVerified
        }
        AssessmentCategory::ProceedWithCaution if unexpected => {
            RecommendedAction::VerifySenderIndependently
        }
        AssessmentCategory::ProceedWithCaution if families.destination_transparency => {
            RecommendedAction::UseKnownWebsiteInstead
        }
        AssessmentCategory::ProceedWithCaution => RecommendedAction::VerifyIndependently,
        AssessmentCategory::NoClearStructuralWarningSigns => RecommendedAction::ContinueCarefully,
    };

    let recommended_next_steps = recommended_next_steps(families, context, category);

    let mut reasons = Vec::new();
    if families.destination_transparency {
        reasons.push(reason(
            AssessmentReasonKind::EmbeddedOrNestedDestination,
            AssessmentReasonSource::UrlAnalysis,
        ));
    }
    if families.layered_or_compound {
        reasons.push(reason(
            AssessmentReasonKind::LayeredOrCompoundEncoding,
            AssessmentReasonSource::UrlAnalysis,
        ));
    }
    if families.ambiguity {
        reasons.push(reason(
            AssessmentReasonKind::AmbiguousDestinationInterpretation,
            AssessmentReasonSource::UrlAnalysis,
        ));
    }
    if families.analysis_incomplete {
        reasons.push(reason(
            AssessmentReasonKind::AnalysisIncomplete,
            AssessmentReasonSource::UrlAnalysis,
        ));
    }
    if unexpected && context.claimed_purpose == Some(ClaimedPurpose::Document) {
        reasons.push(reason(
            AssessmentReasonKind::UnexpectedDocumentRequest,
            AssessmentReasonSource::UserContext,
        ));
    } else if unexpected {
        reasons.push(reason(
            AssessmentReasonKind::UnexpectedContact,
            AssessmentReasonSource::UserContext,
        ));
    } else if uncertain {
        reasons.push(reason(
            AssessmentReasonKind::UncertainContact,
            AssessmentReasonSource::UserContext,
        ));
    }
    if sensitive_purpose {
        reasons.push(reason(
            AssessmentReasonKind::SensitiveClaimedPurpose,
            AssessmentReasonSource::UserContext,
        ));
    }
    if sensitive_action {
        reasons.push(reason(
            AssessmentReasonKind::SensitiveRequestedAction,
            AssessmentReasonSource::UserContext,
        ));
    }

    let mut limitations = vec![AssessmentLimitation {
        identifier: "assessment.supported-evidence-only",
        explanation: "The assessment uses only supported local URL structures and optional answers. It does not verify identity, intent, content, or the final destination.",
    }];
    if context.is_empty() {
        limitations.push(AssessmentLimitation {
            identifier: "assessment.context-not-provided",
            explanation: "No optional context was supplied, so the recommendation is based only on supported URL evidence.",
        });
    } else if context.is_partial() {
        limitations.push(AssessmentLimitation {
            identifier: "assessment.context-partial",
            explanation: "Only some optional context questions were answered, so Clue did not assume the missing circumstances.",
        });
    }
    if families.ambiguity {
        limitations.push(AssessmentLimitation {
            identifier: "assessment.technical-ambiguity-unresolved",
            explanation: "User context cannot resolve the destination-like structure that Clue could not classify confidently.",
        });
    }
    if families.analysis_incomplete {
        limitations.push(AssessmentLimitation {
            identifier: "assessment.analysis-incomplete",
            explanation: "At least one supported analysis stage was incomplete; optional context cannot replace the missing technical evidence.",
        });
    }

    LinkAssessment {
        contract_version: ASSESSMENT_CONTRACT_VERSION,
        category,
        recommended_action,
        recommended_next_steps,
        reasons,
        limitations,
        context_changed_recommendation: false,
    }
}

fn recommended_next_steps(
    families: &EvidenceFamilies,
    context: &AssessmentContext,
    category: AssessmentCategory,
) -> Vec<RecommendedNextStep> {
    let mut steps = Vec::with_capacity(3);

    if category == AssessmentCategory::StrongWarningSigns {
        steps.push(RecommendedNextStep::AvoidOpeningLink);
    }

    if families.analysis_incomplete {
        steps.push(RecommendedNextStep::TreatIncompleteAnalysisWithCaution);
    }

    match context.requested_action {
        Some(RequestedAction::EnterAccountCredentials) => {
            steps.push(RecommendedNextStep::AvoidCredentialsThroughLink);
        }
        Some(RequestedAction::ProvidePersonalInformation) => {
            steps.push(RecommendedNextStep::AvoidPersonalInformationThroughLink);
        }
        Some(RequestedAction::MakePayment) => {
            steps.push(RecommendedNextStep::AvoidPaymentThroughLink);
        }
        Some(RequestedAction::DownloadOrInstallSoftware) => {
            steps.push(RecommendedNextStep::AvoidSoftwareInstallThroughLink);
        }
        _ => {}
    }

    if steps.len() < 3 {
        let verification_step = if context.expected_contact == Some(ExpectedContact::Unexpected)
            && context.claimed_purpose == Some(ClaimedPurpose::Document)
        {
            RecommendedNextStep::VerifyDocumentRequestThroughTrustedChannel
        } else if context.expected_contact == Some(ExpectedContact::Unexpected) {
            RecommendedNextStep::VerifySenderThroughTrustedChannel
        } else if families.destination_transparency {
            RecommendedNextStep::UseKnownWebsiteOrApp
        } else if category == AssessmentCategory::NoClearStructuralWarningSigns {
            RecommendedNextStep::ConfirmRequestMakesSense
        } else {
            RecommendedNextStep::VerifyRequestIndependently
        };
        if !steps.contains(&verification_step) {
            steps.push(verification_step);
        }
    }

    if category == AssessmentCategory::StrongWarningSigns
        && context.expected_contact == Some(ExpectedContact::Unexpected)
        && matches!(
            context.source,
            Some(LinkSource::Email | LinkSource::Message)
        )
        && steps.len() < 3
    {
        steps.push(RecommendedNextStep::AvoidReplyingToEstablishTrust);
    }

    steps
}

const fn reason(kind: AssessmentReasonKind, source: AssessmentReasonSource) -> AssessmentReason {
    AssessmentReason { kind, source }
}
