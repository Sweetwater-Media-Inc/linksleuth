use linksleuth_assessment::{
    AssessmentCategory, AssessmentContext, ClaimedPurpose, ExpectedContact, LinkSource,
    RecommendedAction, RecommendedNextStep, RequestedAction, assess_link,
};
use linksleuth_core::analysis::analyze_url;
use linksleuth_core::consumer::{ConsumerReportOptions, create_consumer_report};

const CORPUS: &str = include_str!("corpus/assessment_cases.tsv");

#[test]
fn assessment_corpus_matches_expected_outcomes() {
    let mut count = 0;

    for line in CORPUS.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        count += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 10, "invalid assessment corpus row: {line}");

        let id = fields[0];
        let input = expand_input(fields[1]);
        let context = context(fields[2], fields[3], fields[4], fields[5]);
        let analysis = analyze_url(&input)
            .unwrap_or_else(|error| panic!("assessment case {id} failed: {error:?}"));
        let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
        let assessment = assess_link(&report, &context);

        assert_eq!(
            assessment.category().identifier(),
            fields[6],
            "category for {id}"
        );
        assert_eq!(
            assessment.recommended_action().identifier(),
            fields[7],
            "recommendation for {id}"
        );
        let reasons = assessment
            .reasons()
            .iter()
            .map(|reason| reason.identifier())
            .collect::<Vec<_>>()
            .join(",");
        let expected_reasons = if fields[8] == "-" { "" } else { fields[8] };
        assert_eq!(reasons, expected_reasons, "reasons for {id}");
        assert_eq!(
            assessment.context_changed_recommendation(),
            fields[9].parse::<bool>().unwrap(),
            "context change for {id}"
        );
    }

    assert_eq!(count, 29);
}

#[test]
fn changing_and_clearing_context_is_deterministic_and_does_not_mutate_evidence() {
    let analysis =
        analyze_url("https://top.example/?target=https%3A%2F%2Fdestination.example%2Fcontent")
            .unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
    let observation_snapshot = report.observations().to_vec();

    let preliminary = assess_link(&report, &AssessmentContext::default());
    let sensitive = assess_link(
        &report,
        &AssessmentContext::new()
            .with_expected_contact(ExpectedContact::Expected)
            .with_requested_action(RequestedAction::EnterAccountCredentials),
    );
    let cleared = assess_link(&report, &AssessmentContext::default());

    assert_eq!(
        preliminary.category(),
        AssessmentCategory::ProceedWithCaution
    );
    assert_eq!(sensitive.category(), AssessmentCategory::ProceedWithCaution);
    assert_eq!(
        preliminary.recommended_action(),
        RecommendedAction::UseKnownWebsiteInstead
    );
    assert_eq!(
        sensitive.recommended_action(),
        RecommendedAction::AvoidSensitiveActionUntilVerified
    );
    assert!(sensitive.context_changed_recommendation());
    assert_eq!(cleared, preliminary);
    assert_eq!(report.observations(), observation_snapshot);
}

#[test]
fn fixed_assessment_text_avoids_certainty_and_source_evidence() {
    let analysis = analyze_url(
        "https://private.example/path?token=fake-secret&target=https%3A%2F%2Fhidden.example%2Fprivate",
    )
    .unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
    let assessment = assess_link(
        &report,
        &AssessmentContext::new()
            .with_source(LinkSource::Email)
            .with_expected_contact(ExpectedContact::Unexpected),
    );
    let rendered = format!(
        "{} {} {} {} {} {} {} {:?}",
        assessment.category().title(),
        assessment.category().explanation(),
        assessment.recommended_action().title(),
        assessment.recommended_action().explanation(),
        assessment
            .recommended_next_steps()
            .iter()
            .map(|step| step.identifier())
            .collect::<Vec<_>>()
            .join(" "),
        assessment
            .recommended_next_steps()
            .iter()
            .map(|step| step.explanation())
            .collect::<Vec<_>>()
            .join(" "),
        assessment
            .reasons()
            .iter()
            .map(|reason| reason.explanation())
            .collect::<Vec<_>>()
            .join(" "),
        assessment
    );
    let lowercase = rendered.to_lowercase();

    for sensitive in [
        "private.example",
        "hidden.example",
        "fake-secret",
        "token",
        "/private",
    ] {
        assert!(!rendered.contains(sensitive));
    }
    assert!(!lowercase.contains("confirmed phishing"));
    assert!(!lowercase.contains("proven malicious"));
    assert!(!lowercase.contains("guaranteed safe"));
}

#[test]
fn assessment_source_has_no_network_or_platform_dependencies() {
    let source = include_str!("../src/lib.rs");

    for forbidden in [
        "reqwest",
        "hyper::",
        "TcpStream",
        "UdpSocket",
        "std::fs",
        "fetch(",
        "XMLHttpRequest",
        "window.",
        "window.document",
    ] {
        assert!(
            !source.contains(forbidden),
            "found forbidden source: {forbidden}"
        );
    }
}

#[test]
fn context_can_reduce_missing_context_uncertainty_but_not_technical_ambiguity() {
    let ordinary_analysis = analyze_url("https://ordinary.example/content").unwrap();
    let ordinary_report =
        create_consumer_report(&ordinary_analysis, ConsumerReportOptions::default());
    let preliminary = assess_link(&ordinary_report, &AssessmentContext::default());
    let expected = assess_link(
        &ordinary_report,
        &AssessmentContext::new().with_expected_contact(ExpectedContact::Expected),
    );
    assert!(
        preliminary
            .limitations()
            .iter()
            .any(|item| item.identifier() == "assessment.context-not-provided")
    );
    assert!(
        expected
            .limitations()
            .iter()
            .all(|item| item.identifier() != "assessment.context-not-provided")
    );
    assert!(
        expected
            .limitations()
            .iter()
            .any(|item| item.identifier() == "assessment.context-partial")
    );
    assert_eq!(expected.category(), preliminary.category());

    let ambiguous_analysis =
        analyze_url("https://top.example/?target=tracking.example%2F%2Fpath").unwrap();
    let ambiguous_report =
        create_consumer_report(&ambiguous_analysis, ConsumerReportOptions::default());
    let contextual = assess_link(
        &ambiguous_report,
        &AssessmentContext::new().with_expected_contact(ExpectedContact::Expected),
    );
    assert!(
        contextual
            .limitations()
            .iter()
            .any(|item| { item.identifier() == "assessment.technical-ambiguity-unresolved" })
    );
    assert_eq!(
        contextual.category(),
        AssessmentCategory::ProceedWithCaution
    );
}

#[test]
fn motivating_context_produces_decisive_bounded_steps_in_stable_order() {
    let analysis = analyze_url(
        "https://social.example/link/v2?aid=1988&scene=bio_url&target=tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html#synthetic-fragment",
    )
    .unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
    let assessment = assess_link(
        &report,
        &AssessmentContext::new()
            .with_source(LinkSource::Email)
            .with_claimed_purpose(ClaimedPurpose::Document)
            .with_expected_contact(ExpectedContact::Unexpected)
            .with_requested_action(RequestedAction::ViewContent),
    );

    assert_eq!(
        assessment.category(),
        AssessmentCategory::StrongWarningSigns
    );
    assert_eq!(
        assessment.recommended_action(),
        RecommendedAction::DoNotOpenLink
    );
    assert_eq!(
        assessment.recommended_next_steps(),
        [
            RecommendedNextStep::AvoidOpeningLink,
            RecommendedNextStep::VerifyDocumentRequestThroughTrustedChannel,
            RecommendedNextStep::AvoidReplyingToEstablishTrust,
        ]
    );
    assert_eq!(assessment.recommended_next_steps().len(), 3);
    assert!(
        assessment
            .recommended_action()
            .explanation()
            .contains("consistent with phishing")
    );
    assert!(
        assessment
            .recommended_action()
            .explanation()
            .contains("independently verify")
    );
}

#[test]
fn practical_guidance_is_specific_without_promoting_ordinary_links_to_strong_warning() {
    let analysis = analyze_url("https://ordinary.example/account").unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
    let expected_credentials = assess_link(
        &report,
        &AssessmentContext::new()
            .with_source(LinkSource::Email)
            .with_claimed_purpose(ClaimedPurpose::AccountSignInOrVerification)
            .with_expected_contact(ExpectedContact::Expected)
            .with_requested_action(RequestedAction::EnterAccountCredentials),
    );
    assert_eq!(
        expected_credentials.category(),
        AssessmentCategory::ProceedWithCaution
    );
    assert_eq!(
        expected_credentials.recommended_next_steps(),
        [
            RecommendedNextStep::AvoidCredentialsThroughLink,
            RecommendedNextStep::VerifyRequestIndependently,
        ]
    );
    assert!(
        !expected_credentials
            .recommended_action()
            .explanation()
            .contains("phishing")
    );

    let ordinary = assess_link(&report, &AssessmentContext::default());
    assert_eq!(
        ordinary.category(),
        AssessmentCategory::NoClearStructuralWarningSigns
    );
    assert_eq!(
        ordinary.recommended_next_steps(),
        [RecommendedNextStep::ConfirmRequestMakesSense]
    );
    assert!(!ordinary.category().explanation().contains("phishing"));
}

#[test]
fn evidence_families_and_next_steps_are_deduplicated() {
    let analysis = analyze_url(
        "https://top.example/?first=https%3A%2F%2Fone.example%2F&second=https%3A%2F%2Ftwo.example%2F",
    )
    .unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
    let assessment = assess_link(&report, &AssessmentContext::default());

    assert_eq!(
        assessment
            .reasons()
            .iter()
            .filter(|reason| reason.identifier() == "url.destination-transparency-reduced")
            .count(),
        1
    );
    assert_eq!(
        assessment.recommended_next_steps(),
        [RecommendedNextStep::UseKnownWebsiteOrApp]
    );
}

fn context(source: &str, purpose: &str, expected: &str, action: &str) -> AssessmentContext {
    let mut context = AssessmentContext::new();
    if source != "-" {
        context = context.with_source(match source {
            "email" => LinkSource::Email,
            "message" => LinkSource::Message,
            "social" => LinkSource::SocialMedia,
            "website" => LinkSource::WebsiteOrSearch,
            "qr" => LinkSource::QrCode,
            "other" => LinkSource::OtherOrUnknown,
            _ => panic!("unknown source {source}"),
        });
    }
    if purpose != "-" {
        context = context.with_claimed_purpose(match purpose {
            "document" => ClaimedPurpose::Document,
            "account" => ClaimedPurpose::AccountSignInOrVerification,
            "delivery" => ClaimedPurpose::DeliveryOrShipping,
            "payment" => ClaimedPurpose::PaymentOrFinancialRequest,
            "ordinary" => ClaimedPurpose::OrdinaryContent,
            "software" => ClaimedPurpose::SoftwareDownloadOrInstall,
            "other" => ClaimedPurpose::OtherOrUnknown,
            _ => panic!("unknown purpose {purpose}"),
        });
    }
    if expected != "-" {
        context = context.with_expected_contact(match expected {
            "expected" => ExpectedContact::Expected,
            "unexpected" => ExpectedContact::Unexpected,
            "unsure" => ExpectedContact::Unsure,
            _ => panic!("unknown expected value {expected}"),
        });
    }
    if action != "-" {
        context = context.with_requested_action(match action {
            "view" => RequestedAction::ViewContent,
            "credentials" => RequestedAction::EnterAccountCredentials,
            "personal" => RequestedAction::ProvidePersonalInformation,
            "payment" => RequestedAction::MakePayment,
            "software" => RequestedAction::DownloadOrInstallSoftware,
            "contact" => RequestedAction::ContactSomeone,
            "other" => RequestedAction::OtherOrUnknown,
            _ => panic!("unknown action {action}"),
        });
    }
    context
}

fn expand_input(input: &str) -> String {
    if input != "@generate-depth-limit" {
        return input.to_string();
    }

    let mut nested = "https://level5.example/?tail=value".to_string();
    for level in (1..=4).rev() {
        nested = format!(
            "https://level{level}.example/?next={}",
            url::form_urlencoded::byte_serialize(nested.as_bytes()).collect::<String>()
        );
    }
    format!(
        "https://top.example/?target={}",
        url::form_urlencoded::byte_serialize(nested.as_bytes()).collect::<String>()
    )
}
