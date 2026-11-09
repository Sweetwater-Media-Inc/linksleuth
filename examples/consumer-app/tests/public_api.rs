use linksleuth_core::analysis::analyze_url;
use linksleuth_core::consumer::{
    CONSUMER_REPORT_SCHEMA_VERSION, ConsumerAnalysisStage, ConsumerCompletenessStatus,
    ConsumerDisclosureClass, ConsumerReportOptions, ConsumerStageStatus, create_consumer_report,
};

#[test]
fn independent_consumer_can_use_the_public_contract() {
    let analysis = analyze_url(concat!(
        "https://private.example/public/%2e%2e/admin?token=fake-secret",
        "&target=https%3A%2F%2Fdestination.example%2Fprivate"
    ))
    .unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());

    assert_eq!(report.schema_version(), CONSUMER_REPORT_SCHEMA_VERSION);
    assert_eq!(report.disclosure(), ConsumerDisclosureClass::PrivacyReduced);
    assert_eq!(report.hostname(), None);
    assert!(report.evidence().is_empty());

    let observations = report.observations();
    assert_eq!(observations.len(), 2);
    assert_eq!(
        observations[0].identifier(),
        "normalization.encoded-parent-directory"
    );
    assert_eq!(
        observations[0].title(),
        "Encoded parent-directory normalization"
    );
    assert!(
        observations[0]
            .explanation()
            .contains("does not establish intent")
    );
    assert_eq!(observations[0].occurrence_count(), 1);
    assert!(observations[0].evidence_references().is_empty());

    assert_eq!(
        report.completeness().status(),
        ConsumerCompletenessStatus::CompleteWithinSupportedScope
    );
    assert_eq!(report.completeness().stages().len(), 4);
    assert_eq!(
        report.completeness().stages()[0].stage(),
        ConsumerAnalysisStage::OriginalPath
    );
    assert_eq!(
        report.completeness().stages()[0].status(),
        ConsumerStageStatus::CompleteWithinSupportedScope
    );
}

#[test]
fn independent_disclosure_controls_remain_explicit_and_separate() {
    let analysis = analyze_url(concat!(
        "https://private.example/?target=",
        "https%3A%2F%2Fnested.example%2Fprivate"
    ))
    .unwrap();

    let hostname = create_consumer_report(&analysis, ConsumerReportOptions::new().with_hostname());
    assert_eq!(hostname.hostname(), Some("private.example"));
    assert!(hostname.evidence().is_empty());
    assert!(
        hostname
            .observations()
            .iter()
            .all(|observation| observation.evidence_references().is_empty())
    );

    let evidence = create_consumer_report(
        &analysis,
        ConsumerReportOptions::new().with_detailed_evidence(),
    );
    assert_eq!(evidence.hostname(), None);
    assert!(!evidence.evidence().is_empty());
    assert!(
        evidence
            .observations()
            .iter()
            .any(|observation| !observation.evidence_references().is_empty())
    );
}

#[test]
fn independent_consumer_can_distinguish_unsupported_analysis() {
    let analysis = analyze_url("https:////private.example/path?category=books").unwrap();
    let report = create_consumer_report(&analysis, ConsumerReportOptions::default());

    assert_eq!(
        report.completeness().status(),
        ConsumerCompletenessStatus::PartiallyExamined
    );
    let original_path = report
        .completeness()
        .stages()
        .iter()
        .find(|stage| stage.stage() == ConsumerAnalysisStage::OriginalPath)
        .unwrap();
    assert_eq!(
        original_path.status(),
        ConsumerStageStatus::UnsupportedEvidence
    );
    assert!(!original_path.reasons().is_empty());
}
