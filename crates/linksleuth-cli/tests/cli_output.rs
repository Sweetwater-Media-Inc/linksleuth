use std::process::Command;

#[test]
fn cli_displays_normalization_findings() {
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/public/%2e%2e/admin")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Normalization Findings"));
    assert!(stdout.contains("Encoded parent-directory segment"));
    assert!(stdout.contains("/public/%2e%2e/admin"));
    assert!(stdout.contains("Normalized path: /admin"));
}

#[test]
fn cli_displays_decoding_warnings() {
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/?message=hello%ZZworld")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Decoding observation"));
    assert!(stdout.contains("byte offset 5"));
    assert!(stdout.contains("pass 1"));
}

#[test]
fn cli_distinguishes_malformed_encoding_from_decoding_completion() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/?message=hello%ZZworld")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("No valid percent-encoded sequences decoded"));

    assert!(stdout.contains("Percent decoding: No further valid sequences detected"));

    assert!(stdout.contains("Invalid percent sequence at byte offset 5 during pass 1"));

    assert!(!stdout.contains("Status: Complete"));
}
