use linksleuth_core::analysis::analyze_url;
use linksleuth_core::decoder::DecodingStatus;
use linksleuth_core::destinations::{
    DestinationReferenceKind, DestinationScheme, DestinationTransformation,
};
use linksleuth_core::encodings::{Base64Alphabet, Base64Padding};
use linksleuth_core::normalization::OriginalPathAnalysis;
use linksleuth_core::parser::UrlError;
use linksleuth_core::reporting::{
    ShareableReportOptions, create_shareable_report, create_shareable_report_with_options,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (shareable_options, input) = match args.as_slice() {
        [url] if !url.starts_with("--") => (None, url.as_str()),
        [flag, url] if flag == "--shareable" && !url.starts_with("--") => {
            (Some(ShareableReportOptions::default()), url.as_str())
        }
        [shareable, include_hostname, url]
            if shareable == "--shareable"
                && include_hostname == "--include-hostname"
                && !url.starts_with("--") =>
        {
            (
                Some(ShareableReportOptions::new().with_hostname()),
                url.as_str(),
            )
        }
        _ => {
            eprintln!("Usage: linksleuth-cli [--shareable [--include-hostname]] <URL>");
            std::process::exit(2);
        }
    };

    match analyze_url(input) {
        Ok(report) => {
            if let Some(options) = shareable_options {
                let shared = if options.include_hostname {
                    create_shareable_report_with_options(&report, options)
                } else {
                    create_shareable_report(&report)
                };

                println!("LinkSleuth Shareable Summary");
                println!("----------------------------");
                println!("Hostname: {}", shared.hostname);
                println!("Query parameters: {}", shared.query_parameter_count);
                match shared.original_path_analysis {
                    OriginalPathAnalysis::Examined => println!(
                        "Normalization findings: {}",
                        shared.normalization_finding_count
                    ),
                    OriginalPathAnalysis::Unsupported { .. } => {
                        println!("Normalization findings: unavailable")
                    }
                }
                println!(
                    "Potential embedded destinations: {}",
                    shared.embedded_destination_count
                );
                if shared.nested_destination_count > 0 {
                    println!("Nested candidates: {}", shared.nested_destination_count);
                }
                if shared.protocol_relative_destination_count > 0 {
                    println!(
                        "Protocol-relative references: {}",
                        shared.protocol_relative_destination_count
                    );
                }
                if shared.base64_destination_count > 0 {
                    println!(
                        "Base64-encoded candidates: {}",
                        shared.base64_destination_count
                    );
                }
                if shared.html_character_reference_destination_count > 0 {
                    println!(
                        "HTML-reference-encoded candidates: {}",
                        shared.html_character_reference_destination_count
                    );
                }
                if !shared.destination_discovery_status.is_complete()
                    || !shared.destination_encoding_coverage.is_complete()
                {
                    println!("Destination discovery: incomplete");
                    for limit in shared.destination_discovery_status.limits() {
                        println!("  Note: {}", limit.explanation());
                    }
                    for issue in shared.destination_encoding_coverage.issues() {
                        println!("  Note: {}", issue.explanation());
                    }
                }

                if !shared.findings.is_empty() {
                    println!("\nObservations");

                    for (index, finding) in shared.findings.iter().enumerate() {
                        println!("\n{}. {}", index + 1, finding.title());

                        for line in finding.explanation_lines() {
                            println!("   {line}");
                        }
                    }
                }

                println!("\nNote: This is a privacy-reduced summary, not a security verdict.");

                return;
            }

            println!("LinkSleuth URL Analysis");
            println!("-----------------------");
            println!("Hostname: {}", report.url.hostname);
            println!("Path:     {}", report.url.path);

            if matches!(
                report.original_path_analysis,
                OriginalPathAnalysis::Unsupported { .. }
            ) {
                println!("\nNormalization Analysis");
                println!("----------------------");
                println!(
                    "Original path evidence could not be interpreted confidently; no normalization conclusion is available."
                );
            }

            if !report.normalization_findings.is_empty() {
                println!("\nNormalization Findings");
                println!("----------------------");

                for finding in &report.normalization_findings {
                    println!("\nFinding: {}", finding.kind.title());
                    println!("  Original path:   {}", finding.original_path);
                    println!("  Normalized path: {}", finding.normalized_path);
                    println!("  Explanation:     {}", finding.explanation);
                }
            }

            for parameter in &report.parameters {
                println!("\nParameter: {}", parameter.name);

                match &parameter.decoding {
                    Ok(result) => {
                        if result.steps.is_empty() {
                            println!("  No valid percent-encoded sequences decoded");
                        }

                        for (index, step) in result.steps.iter().enumerate() {
                            println!("  Pass {}: {}", index + 1, step.output);
                        }

                        match result.status {
                            DecodingStatus::Complete => {
                                println!("  Percent decoding: No further valid sequences detected");
                            }
                            DecodingStatus::PassLimitReached => {
                                println!(
                                    "  Percent decoding: Pass limit reached; additional encoded sequences remain"
                                );
                            }
                        }

                        for warning in &result.warnings {
                            match warning {
                                linksleuth_core::decoder::DecodingWarning::MalformedPercentEncoding {
                                    pass,
                                    offset,
                                } => {
                                    println!(
                                        "  Decoding observation: Invalid percent sequence at byte offset {} during pass {}.",
                                        offset, pass
                                    );
                                    println!("  Note: The original text was preserved.");
                                }
                            }
                        }
                    }
                    Err(error) => {
                        println!("  Decoding error: {error:?}");
                    }
                }
            }

            if !report.destination_discovery.candidates.is_empty()
                || !report.destination_discovery.is_complete()
            {
                println!("\nPotential Destinations");
                println!("----------------------");

                for (index, candidate) in report.destination_discovery.candidates.iter().enumerate()
                {
                    println!("\nPotential destination {}", index + 1);
                    println!(
                        "  Type: {}",
                        destination_kind_label(candidate.reference.kind)
                    );
                    println!(
                        "  Relationship: {}",
                        if candidate.provenance.depth == 0 {
                            "Directly embedded candidate"
                        } else {
                            "Nested candidate"
                        }
                    );
                    println!("  Submitted reference: {}", candidate.reference.submitted);
                    println!("  Normalized reference: {}", candidate.reference.normalized);
                    println!("  Candidate hostname: {}", candidate.reference.hostname);
                    println!(
                        "  Top-level parameter position: {}",
                        candidate.provenance.top_level_parameter_index + 1
                    );
                    println!(
                        "  Source parameter: {}",
                        candidate.provenance.source_parameter
                    );
                    println!(
                        "  Source parameter position: {}",
                        candidate.provenance.source_parameter_index + 1
                    );
                    println!("  Discovery depth: {}", candidate.provenance.depth);
                    if let Some(parent) = candidate.provenance.parent_candidate_index {
                        println!("  Parent candidate: {}", parent + 1);
                    }
                    if candidate.provenance.transformations.is_empty() {
                        println!("  Transformations: None");
                    } else {
                        println!("  Transformations:");
                        for (step, transformation) in
                            candidate.provenance.transformations.iter().enumerate()
                        {
                            match transformation {
                                DestinationTransformation::PercentDecoding { passes } => println!(
                                    "    {}. Percent decoding ({} {})",
                                    step + 1,
                                    passes,
                                    if *passes == 1 { "pass" } else { "passes" }
                                ),
                                DestinationTransformation::Base64Decoding { alphabet, padding } => {
                                    println!(
                                        "    {}. Base64 decoding ({}, {})",
                                        step + 1,
                                        base64_alphabet_label(*alphabet),
                                        base64_padding_label(*padding)
                                    )
                                }
                                DestinationTransformation::HtmlCharacterReferenceDecoding {
                                    reference_count,
                                } => println!(
                                    "    {}. HTML character-reference decoding ({} {})",
                                    step + 1,
                                    reference_count,
                                    if *reference_count == 1 {
                                        "reference"
                                    } else {
                                        "references"
                                    }
                                ),
                                DestinationTransformation::CompoundQueryExtraction {
                                    parameter_position,
                                } => println!(
                                    "    {}. Compound query extraction (inner parameter {})",
                                    step + 1,
                                    parameter_position + 1
                                ),
                            }
                        }
                    }
                    println!(
                        "  Decoding passes: {}",
                        candidate.provenance.decoding.steps.len()
                    );
                    for (pass, step) in candidate.provenance.decoding.steps.iter().enumerate() {
                        println!("  Discovery pass {} input: {}", pass + 1, step.input);
                        println!("  Discovery pass {} output: {}", pass + 1, step.output);
                    }
                    println!(
                        "  Note: Potential destination evidence only; no redirect was verified."
                    );
                }

                for issue in &report.destination_discovery.encoding_issues {
                    println!("\nEncoding analysis incomplete");
                    println!(
                        "  Top-level parameter position: {}",
                        issue.top_level_parameter_index + 1
                    );
                    println!(
                        "  Source parameter position: {}",
                        issue.source_parameter_index + 1
                    );
                    println!("  Discovery depth: {}", issue.depth);
                    if let Some(parent) = issue.parent_candidate_index {
                        println!("  Parent candidate: {}", parent + 1);
                    }
                    println!("  Reason: {}", issue.kind.explanation());
                }

                for observation in &report.destination_discovery.structure_observations {
                    println!(
                        "\nStructural observation: {}",
                        observation.kind.identifier()
                    );
                    println!(
                        "  Top-level parameter position: {}",
                        observation.top_level_parameter_index + 1
                    );
                    println!("  Discovery depth: {}", observation.depth);
                }

                for issue in &report.destination_discovery.interpretation_issues {
                    println!("\nDestination interpretation incomplete");
                    println!("  Reason: {}", issue.kind.explanation());
                }

                if !report.destination_discovery.status.is_complete() {
                    println!("\nDestination discovery incomplete");
                    for limit in report.destination_discovery.status.limits() {
                        println!("  {}", limit.explanation());
                    }
                }
            }
        }
        Err(error) => {
            let reason = match error {
                UrlError::TooLong => "the URL exceeds the supported length",
                UrlError::TooManyQueryParameters => "the URL contains too many query parameters",
                UrlError::Invalid(_) => "the URL is invalid",
                UrlError::UnsupportedScheme => "the URL scheme is not supported",
                UrlError::MissingHostname => "the URL has no hostname",
            };
            eprintln!("Unable to analyze URL: {reason}.");
            std::process::exit(1);
        }
    }
}

fn destination_kind_label(kind: DestinationReferenceKind) -> &'static str {
    match kind {
        DestinationReferenceKind::Absolute {
            scheme: DestinationScheme::Http,
        } => "Absolute HTTP candidate URL",
        DestinationReferenceKind::Absolute {
            scheme: DestinationScheme::Https,
        } => "Absolute HTTPS candidate URL",
        DestinationReferenceKind::ProtocolRelative => "Protocol-relative reference",
        DestinationReferenceKind::HostnameFirst => "Hostname-first reference",
    }
}

fn base64_alphabet_label(alphabet: Base64Alphabet) -> &'static str {
    match alphabet {
        Base64Alphabet::Standard => "standard alphabet",
        Base64Alphabet::UrlSafe => "URL-safe alphabet",
        Base64Alphabet::Ambiguous => "alphabet shared by standard and URL-safe Base64",
    }
}

fn base64_padding_label(padding: Base64Padding) -> &'static str {
    match padding {
        Base64Padding::Padded => "padded",
        Base64Padding::Unpadded => "unpadded",
    }
}
