use std::env;
use std::process::ExitCode;

use linksleuth_core::analysis::analyze_url;
use linksleuth_core::consumer::{
    ConsumerCompletenessStatus, ConsumerReport, ConsumerReportOptions, ConsumerStageStatus,
    create_consumer_report,
};
use linksleuth_core::parser::UrlError;

const SYNTHETIC_EXAMPLE: &str = concat!(
    "https://shop.example/public/%2e%2e/admin?target=",
    "https%3A%2F%2Fvendor.example%2Fproducts"
);

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let input = arguments
        .next()
        .unwrap_or_else(|| SYNTHETIC_EXAMPLE.to_string());

    if arguments.next().is_some() {
        eprintln!("Usage: linksleuth-consumer-app [HTTP_OR_HTTPS_URL]");
        return ExitCode::from(2);
    }

    match analyze_url(&input) {
        Ok(analysis) => {
            let report = create_consumer_report(&analysis, ConsumerReportOptions::default());
            print_report(&report);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Analysis error: {}", error_message(&error));
            ExitCode::from(2)
        }
    }
}

fn print_report(report: &ConsumerReport) {
    println!("LinkSleuth consumer API example");
    println!("Schema version: {}", report.schema_version());
    println!("Disclosure: privacy-reduced");
    println!();
    println!("Observations");

    if report.observations().is_empty() {
        println!("- No supported structural observations were produced.");
        println!("  This is not a declaration that the URL is safe.");
    } else {
        for observation in report.observations() {
            println!(
                "- {} [{}] — {} occurrence(s)",
                observation.title(),
                observation.identifier(),
                observation.occurrence_count()
            );
            println!("  {}", observation.explanation());
        }
    }

    println!();
    println!(
        "Analysis completeness: {}",
        completeness_status_label(report.completeness().status())
    );
    for stage in report.completeness().stages() {
        println!(
            "- {}: {}",
            stage.stage().identifier(),
            stage_status_label(stage.status())
        );
        for reason in stage.reasons() {
            println!("  {}: {}", reason.identifier(), reason.explanation());
        }
    }

    println!();
    println!("Detailed evidence and the original hostname are hidden by default.");
    println!("Results describe supported structural observations, not a safety verdict.");
}

fn completeness_status_label(status: ConsumerCompletenessStatus) -> &'static str {
    match status {
        ConsumerCompletenessStatus::CompleteWithinSupportedScope => {
            "complete within supported scope"
        }
        ConsumerCompletenessStatus::PartiallyExamined => "partially examined",
        _ => "unknown future status",
    }
}

fn stage_status_label(status: ConsumerStageStatus) -> &'static str {
    match status {
        ConsumerStageStatus::CompleteWithinSupportedScope => "complete within supported scope",
        ConsumerStageStatus::PartiallyExamined => "partially examined",
        ConsumerStageStatus::UnsupportedEvidence => "unsupported evidence",
        ConsumerStageStatus::ResourceLimited => "resource limited",
        _ => "unknown future status",
    }
}

fn error_message(error: &UrlError) -> &'static str {
    match error {
        UrlError::TooLong => "The URL exceeds the supported input length.",
        UrlError::TooManyQueryParameters => {
            "The URL contains more query parameters than the analyzer supports."
        }
        UrlError::UnsupportedScheme => "Only HTTP and HTTPS URLs are supported.",
        UrlError::Invalid(_) | UrlError::MissingHostname => "Enter a complete HTTP or HTTPS URL.",
    }
}
