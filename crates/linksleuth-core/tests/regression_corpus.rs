use linksleuth_core::analysis::analyze_url;
use linksleuth_core::destinations::{
    DestinationDiscoveryLimit, DestinationEncodingIssueKind, DestinationReferenceKind,
    DestinationScheme, DestinationTransformation, MAX_DISCOVERED_DESTINATIONS,
};
use linksleuth_core::normalization::OriginalPathAnalysis;

const CORPUS: &str = include_str!("corpus/destination_cases.tsv");

#[test]
fn destination_regression_corpus_matches_structured_expectations() {
    let mut case_count = 0;

    for line in CORPUS.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        case_count += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 9, "invalid corpus row: {line}");

        let id = fields[0];
        let input = expand_input(fields[1]);
        let expected_count = fields[2].parse::<usize>().unwrap();
        let expected_kinds = fields[3];
        let expected_transformations = fields[4];
        let expected_complete = fields[5].parse::<bool>().unwrap();
        let expected_reasons = fields[6];
        let expected_path_coverage = fields[7];
        let expected_normalization_count = fields[8].parse::<usize>().unwrap();

        let report = analyze_url(&input)
            .unwrap_or_else(|error| panic!("corpus case {id} failed analysis with {error:?}"));
        let discovery = &report.destination_discovery;

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

    assert_eq!(case_count, 27);
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

    if reasons.is_empty() {
        "none".to_string()
    } else {
        reasons.join(",")
    }
}
