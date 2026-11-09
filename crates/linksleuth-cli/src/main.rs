use linksleuth_core::analysis::analyze_url;
use linksleuth_core::decoder::DecodingStatus;
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

                if let Some(destination) = &parameter.destination {
                    println!("  Embedded URL: {}", destination.url);
                    println!("  Embedded hostname: {}", destination.hostname);
                    println!("  Decoding passes: {}", destination.decoding_passes);
                    println!("  Note: Potential destination only; no redirect verified.");
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
