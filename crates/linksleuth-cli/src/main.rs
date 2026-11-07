use linksleuth_core::analysis::analyze_url;

fn main() {
    let input = match std::env::args().nth(1) {
        Some(value) => value,
        None => {
            eprintln!("Usage: linksleuth-cli <URL>");
            std::process::exit(2);
        }
    };

    match analyze_url(&input) {
        Ok(report) => {
            println!("LinkSleuth URL Analysis");
            println!("-----------------------");
            println!("Hostname: {}", report.url.hostname);
            println!("Path:     {}", report.url.path);

            for parameter in &report.parameters {
                println!("\nParameter: {}", parameter.name);

                match &parameter.decoding {
                    Ok(result) => {
                        if result.steps.is_empty() {
                            println!("  No percent encoding detected");
                        }

                        for (index, step) in result.steps.iter().enumerate() {
                            println!("  Pass {}: {}", index + 1, step.output);
                        }

                        println!("  Status: {:?}", result.status);
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
