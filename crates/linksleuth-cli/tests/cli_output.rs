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

#[test]
fn cli_shareable_mode_redacts_sensitive_url_evidence() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "https://private.example.com/documents/123?token=secret123&email=person@example.com",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("LinkSleuth Shareable Summary"));
    assert!(stdout.contains("Hostname: [REDACTED]"));
    assert!(stdout.contains("Query parameters: 2"));

    assert!(!stdout.contains("private.example.com"));
    assert!(!stdout.contains("/documents/123"));
    assert!(!stdout.contains("secret123"));
    assert!(!stdout.contains("person@example.com"));
    assert!(!stdout.contains("token="));
}

#[test]
fn cli_default_mode_remains_detailed() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/documents/123?token=secret123")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("LinkSleuth URL Analysis"));
    assert!(stdout.contains("Hostname: example.com"));
    assert!(stdout.contains("Path:     /documents/123"));
    assert!(stdout.contains("Parameter: token"));

    assert!(!stdout.contains("LinkSleuth Shareable Summary"));
}

#[test]
fn cli_rejects_extra_arguments_without_echoing_secrets() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "https://example.com/?token=secret123",
            "unexpected-private-value",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.is_empty());
    assert!(stderr.contains("Usage:"));

    assert!(!stderr.contains("secret123"));
    assert!(!stderr.contains("unexpected-private-value"));
    assert!(!stderr.contains("example.com"));
}
