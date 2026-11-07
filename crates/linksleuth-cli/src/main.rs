use linksleuth_core::parser::parse_url;

fn main() {
    let input = match std::env::args().nth(1) {
        Some(value) => value,
        None => {
            eprintln!("Usage: linksleuth-cli <URL>");
            std::process::exit(2);
        }
    };

    match parse_url(&input) {
        Ok(report) => {
            println!("LinkSleuth URL Analysis");
            println!("-----------------------");
            println!("Scheme:   {}", report.scheme);
            println!("Hostname: {}", report.hostname);
            println!("Port:     {:?}", report.port);
            println!("Path:     {}", report.path);
            println!("Fragment: {:?}", report.fragment);

            println!("\nQuery Parameters:");
            for parameter in &report.query_parameters {
                println!("  {} = {}", parameter.name, parameter.value);
            }
        }
        Err(error) => {
            eprintln!("Unable to analyze URL: {error:?}");
            std::process::exit(1);
        }
    }
}
