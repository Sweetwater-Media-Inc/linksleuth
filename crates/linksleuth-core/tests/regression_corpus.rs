use linksleuth_core::analysis::analyze_url;
use linksleuth_core::destinations::{
    DestinationDiscoveryLimit, DestinationEncodingIssueKind, DestinationInterpretationIssueKind,
    DestinationReferenceKind, DestinationScheme, DestinationStructureKind,
    DestinationTransformation, MAX_COMPOUND_QUERY_PARAMETERS, MAX_DISCOVERED_DESTINATIONS,
};
use linksleuth_core::normalization::OriginalPathAnalysis;

const CORPUS: &str = include_str!("corpus/destination_cases.tsv");

#[derive(Debug, Default, PartialEq, Eq)]
struct CorpusMetrics {
    total_cases: usize,
    expected_candidate_positive_cases: usize,
    expected_candidate_negative_cases: usize,
    unexpected_candidate_findings: usize,
    missed_expected_candidate_findings: usize,
    incomplete_or_unsupported_cases: usize,
}

#[test]
fn destination_regression_corpus_matches_structured_expectations() {
    let mut metrics = CorpusMetrics::default();

    for line in CORPUS.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        metrics.total_cases += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        assert!(
            (9..=12).contains(&fields.len()),
            "invalid corpus row: {line}"
        );

        let id = fields[0];
        let input = expand_input(fields[1]);
        let expected_count = fields[2].parse::<usize>().unwrap();
        let expected_kinds = fields[3];
        let expected_transformations = fields[4];
        let expected_complete = fields[5].parse::<bool>().unwrap();
        let expected_reasons = fields[6];
        let expected_path_coverage = fields[7];
        let expected_normalization_count = fields[8].parse::<usize>().unwrap();
        let expected_structures = fields.get(9).copied().unwrap_or("*");
        let expected_sources = fields.get(10).copied().unwrap_or("*");
        let expected_submitted = fields.get(11).copied().unwrap_or("*");

        let report = analyze_url(&input)
            .unwrap_or_else(|error| panic!("corpus case {id} failed analysis with {error:?}"));
        let discovery = &report.destination_discovery;
        let expected_candidate_positive = expected_count > 0;
        let actual_candidate_positive = !discovery.candidates.is_empty();

        if expected_candidate_positive {
            metrics.expected_candidate_positive_cases += 1;
        } else {
            metrics.expected_candidate_negative_cases += 1;
        }
        if !expected_candidate_positive && actual_candidate_positive {
            metrics.unexpected_candidate_findings += 1;
        }
        if expected_candidate_positive && !actual_candidate_positive {
            metrics.missed_expected_candidate_findings += 1;
        }
        if !discovery.is_complete()
            || matches!(
                report.original_path_analysis,
                OriginalPathAnalysis::Unsupported { .. }
            )
        {
            metrics.incomplete_or_unsupported_cases += 1;
        }

        assert_eq!(
            discovery.candidates.len(),
            expected_count,
            "candidate count for {id}"
        );
        assert_eq!(
            discovery.is_complete(),
            expected_complete,
            "completion for {id}"
        );
        assert_eq!(
            reason_labels(discovery),
            expected_reasons,
            "completion reasons for {id}"
        );

        if expected_kinds != "*" && expected_kinds != "-" {
            assert_eq!(
                candidate_kind_labels(discovery),
                expected_kinds,
                "kinds for {id}"
            );
        }
        if expected_kinds == "-" {
            assert!(
                discovery.candidates.is_empty(),
                "unexpected candidate for {id}"
            );
        }

        if expected_transformations != "*" && expected_transformations != "-" {
            assert_eq!(
                transformation_labels(discovery),
                expected_transformations,
                "transformations for {id}"
            );
        }
        if expected_structures != "*" {
            assert_eq!(
                structure_labels(discovery),
                expected_structures,
                "structures for {id}"
            );
        }
        if expected_sources != "*" {
            assert_eq!(
                source_labels(discovery),
                expected_sources,
                "sources for {id}"
            );
        }
        if expected_submitted != "*" {
            assert_eq!(
                submitted_labels(discovery),
                expected_submitted,
                "submitted references for {id}"
            );
        }

        match expected_path_coverage {
            "examined" => assert_eq!(
                report.original_path_analysis,
                OriginalPathAnalysis::Examined,
                "path coverage for {id}"
            ),
            "unsupported" => assert!(
                matches!(
                    report.original_path_analysis,
                    OriginalPathAnalysis::Unsupported { .. }
                ),
                "path coverage for {id}"
            ),
            value => panic!("unsupported corpus path coverage {value}"),
        }
        assert_eq!(
            report.normalization_findings.len(),
            expected_normalization_count,
            "normalization findings for {id}"
        );
    }

    eprintln!(
        "corpus metrics: total={}, expected_positive={}, expected_negative={}, unexpected_findings={}, misses={}, incomplete_or_unsupported={}",
        metrics.total_cases,
        metrics.expected_candidate_positive_cases,
        metrics.expected_candidate_negative_cases,
        metrics.unexpected_candidate_findings,
        metrics.missed_expected_candidate_findings,
        metrics.incomplete_or_unsupported_cases,
    );

    assert_eq!(metrics.unexpected_candidate_findings, 0);
    assert_eq!(metrics.missed_expected_candidate_findings, 0);
    assert_eq!(
        metrics,
        CorpusMetrics {
            total_cases: 62,
            expected_candidate_positive_cases: 30,
            expected_candidate_negative_cases: 32,
            unexpected_candidate_findings: 0,
            missed_expected_candidate_findings: 0,
            incomplete_or_unsupported_cases: 18,
        }
    );
}

fn expand_input(input: &str) -> String {
    match input {
        "@generate-depth-limit" => {
            let mut nested = "https://level5.example/?tail=value".to_string();
            for level in (1..=4).rev() {
                nested = format!(
                    "https://level{level}.example/?next={}",
                    percent_encode(&nested)
                );
            }
            format!("https://top.example/?target={}", percent_encode(&nested))
        }
        "@generate-candidate-limit" => {
            let query = (0..=MAX_DISCOVERED_DESTINATIONS)
                .map(|index| {
                    format!(
                        "p{index}={}",
                        percent_encode(&format!("https://host{index}.example/"))
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            format!("https://top.example/?{query}")
        }
        "@generate-secondary-output-limit" => {
            format!("https://top.example/?payload={}", "A".repeat(12 * 1024))
        }
        "@generate-compound-parameter-limit" => {
            let query = (0..=MAX_COMPOUND_QUERY_PARAMETERS)
                .map(|index| format!("p{index}=value"))
                .collect::<Vec<_>>()
                .join("&");
            format!(
                "https://top.example/?target={}",
                percent_encode(&format!("tracking.example//path?{query}"))
            )
        }
        input => input.to_string(),
    }
}

fn percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn candidate_kind_labels(
    discovery: &linksleuth_core::destinations::DestinationDiscovery,
) -> String {
    discovery
        .candidates
        .iter()
        .map(|candidate| match candidate.reference.kind {
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Http,
            } => "http",
            DestinationReferenceKind::Absolute {
                scheme: DestinationScheme::Https,
            } => "https",
            DestinationReferenceKind::ProtocolRelative => "protocol",
            DestinationReferenceKind::HostnameFirst => "hostname-first",
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn transformation_labels(
    discovery: &linksleuth_core::destinations::DestinationDiscovery,
) -> String {
    discovery
        .candidates
        .iter()
        .map(|candidate| {
            if candidate.provenance.transformations.is_empty() {
                return "none".to_string();
            }

            candidate
                .provenance
                .transformations
                .iter()
                .map(|transformation| match transformation {
                    DestinationTransformation::PercentDecoding { .. } => "percent",
                    DestinationTransformation::Base64Decoding { .. } => "base64",
                    DestinationTransformation::HtmlCharacterReferenceDecoding { .. } => "html",
                    DestinationTransformation::CompoundQueryExtraction { .. } => "compound",
                })
                .collect::<Vec<_>>()
                .join("+")
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn reason_labels(discovery: &linksleuth_core::destinations::DestinationDiscovery) -> String {
    let mut reasons = discovery
        .status
        .limits()
        .iter()
        .map(|limit| match limit {
            DestinationDiscoveryLimit::MaximumNestingDepth => "maximum-depth",
            DestinationDiscoveryLimit::MaximumCandidateCount => "maximum-candidates",
            DestinationDiscoveryLimit::MaximumNestedParameterCount => "maximum-nested-parameters",
            DestinationDiscoveryLimit::MaximumCompoundParameterCount => {
                "maximum-compound-parameters"
            }
            DestinationDiscoveryLimit::DecoderInputLength => "percent-input-limit",
            DestinationDiscoveryLimit::DecodingPassLimit => "percent-pass-limit",
            DestinationDiscoveryLimit::NestedUrlLength => "nested-url-limit",
            DestinationDiscoveryLimit::SecondaryEncodingInputLength => "secondary-input-limit",
            DestinationDiscoveryLimit::SecondaryDecodedOutputLength => "secondary-output-limit",
        })
        .collect::<Vec<_>>();
    reasons.extend(
        discovery
            .encoding_coverage
            .issues()
            .iter()
            .map(|issue| match issue {
                DestinationEncodingIssueKind::MalformedBase64 => "malformed-base64",
                DestinationEncodingIssueKind::MalformedHtmlCharacterReference => "malformed-html",
                DestinationEncodingIssueKind::UnsupportedHtmlCharacterReference => {
                    "unsupported-html"
                }
                DestinationEncodingIssueKind::InvalidHtmlUnicodeScalar => "invalid-html-scalar",
            }),
    );
    reasons.extend(
        discovery
            .interpretation_coverage
            .issues()
            .iter()
            .map(|issue| match issue {
                DestinationInterpretationIssueKind::AmbiguousHostnameFirstReference => {
                    "ambiguous-hostname-first"
                }
            }),
    );

    if reasons.is_empty() {
        "none".to_string()
    } else {
        reasons.join(",")
    }
}

fn structure_labels(discovery: &linksleuth_core::destinations::DestinationDiscovery) -> String {
    if discovery.structure_observations.is_empty() {
        return "-".to_string();
    }
    discovery
        .structure_observations
        .iter()
        .map(|observation| match observation.kind {
            DestinationStructureKind::MultiplePercentDecodingTransformations => "multi-percent",
            DestinationStructureKind::EncodedQueryDelimiter => "encoded-delimiter",
            DestinationStructureKind::CompoundQueryValue => "compound",
            DestinationStructureKind::AmbiguousHostnameFirstReference => "ambiguous-hostname-first",
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn source_labels(discovery: &linksleuth_core::destinations::DestinationDiscovery) -> String {
    if discovery.candidates.is_empty() {
        return "-".to_string();
    }
    discovery
        .candidates
        .iter()
        .map(|candidate| candidate.provenance.source_parameter.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn submitted_labels(discovery: &linksleuth_core::destinations::DestinationDiscovery) -> String {
    if discovery.candidates.is_empty() {
        return "-".to_string();
    }
    discovery
        .candidates
        .iter()
        .map(|candidate| candidate.reference.submitted.as_str())
        .collect::<Vec<_>>()
        .join(",")
}
