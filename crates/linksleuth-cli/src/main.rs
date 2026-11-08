use linksleuth_core::analysis::analyze_url;
use linksleuth_core::decoder::DecodingStatus;
use linksleuth_core::reporting::create_shareable_report;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (shareable, input) = match args.as_slice() {
        [url] if url != "--shareable" => (false, url.as_str()),
        [flag, url] if flag == "--shareable" => (true, url.as_str()),
        _ => {
            eprintln!("Usage: linksleuth-cli [--shareable] <URL>");
            std::process::exit(2);
        }
    };

    match analyze_url(input) {
        Ok(report) => {
            if shareable {
                let shared = create_shareable_report(&report);

                println!("LinkSleuth Shareable Summary");
                println!("----------------------------");
                println!("Hostname: {}", shared.hostname);
                println!("Query parameters: {}", shared.query_parameter_count);
                println!(
                    "Normalization findings: {}",
                    shared.normalization_finding_count
                );
                println!(
                    "Potential embedded destinations: {}",
                    shared.embedded_destination_count
                );
                println!("Note: Privacy-reduced summary; metadata may still be sensitive.");

                return;
            }

            println!("LinkSleuth URL Analysis");
            println!("-----------------------");
            println!("Hostname: {}", report.url.hostname);
            println!("Path:     {}", report.url.path);

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
            eprintln!("Unable to analyze URL: {error:?}");
            std::process::exit(1);
        }
    }
}
