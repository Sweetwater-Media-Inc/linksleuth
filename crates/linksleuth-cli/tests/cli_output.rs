use std::process::Command;

#[test]
fn cli_displays_normalization_findings() {
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/public/%2e%2e/admin")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Normalization Findings"));
    assert!(stdout.contains("Encoded parent-directory segment"));
    assert!(stdout.contains("/public/%2e%2e/admin"));
    assert!(stdout.contains("Normalized path: /admin"));
    assert!(stderr.is_empty());
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
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("LinkSleuth Shareable Summary"));
    assert!(stdout.contains("Hostname: [REDACTED]"));
    assert!(stdout.contains("Query parameters: 2"));
    assert!(stdout.contains("Note: This is a privacy-reduced summary, not a security verdict."));

    assert!(!stdout.contains("private.example.com"));
    assert!(!stdout.contains("/documents/123"));
    assert!(!stdout.contains("secret123"));
    assert!(!stdout.contains("person@example.com"));
    assert!(!stdout.contains("token="));
    assert!(stderr.is_empty());
}

#[test]
fn cli_shareable_mode_explains_findings_without_exposing_evidence() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "https://private.example.com/public/%2e%2e/admin?token=secret123&target=https%3A%2F%2Fother.example%2Fprivate",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let expected = concat!(
        "LinkSleuth Shareable Summary\n",
        "----------------------------\n",
        "Hostname: [REDACTED]\n",
        "Query parameters: 2\n",
        "Normalization findings: 1\n",
        "Potential embedded destinations: 1\n",
        "\n",
        "Observations\n",
        "\n",
        "1. Encoded parent-directory segment\n",
        "   The submitted URL contained encoded path components\n",
        "   that were normalized during parsing.\n",
        "\n",
        "2. Potential embedded destination\n",
        "   A complete HTTP/HTTPS URL was identified within\n",
        "   a query parameter. No redirect has been verified.\n",
        "\n",
        "Note: This is a privacy-reduced summary, not a security verdict.\n",
    );

    assert_eq!(stdout, expected);
    assert!(!stdout.contains("private.example.com"));
    assert!(!stdout.contains("other.example"));
    assert!(!stdout.contains("secret123"));
    assert!(!stdout.contains("/public/"));
    assert!(!stdout.contains("/private"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_shareable_mode_includes_hostname_only_when_explicitly_requested() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "--include-hostname",
            "https://private.example.com/documents/123?token=secret123#private-fragment",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Hostname: private.example.com"));
    assert!(!stdout.contains("[REDACTED]"));
    assert!(!stdout.contains("/documents/123"));
    assert!(!stdout.contains("secret123"));
    assert!(!stdout.contains("private-fragment"));
    assert!(!stdout.contains("token="));
    assert!(stderr.is_empty());
}

#[test]
fn cli_shareable_mode_reports_malformed_encoding_and_pass_limits_privately() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "https://private.example.com/?broken=secret%ZZvalue&deep=%2525252541",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("1. Malformed percent encoding"));
    assert!(stdout.contains("The original text was preserved during analysis."));
    assert!(stdout.contains("2. Decoding pass limit reached"));
    assert!(stdout.contains("Additional encoded sequences may remain."));
    assert!(!stdout.contains("private.example.com"));
    assert!(!stdout.contains("broken"));
    assert!(!stdout.contains("secret%ZZvalue"));
    assert!(!stdout.contains("deep"));
    assert!(!stdout.contains("%2525252541"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_shareable_mode_omits_observations_for_ordinary_urls() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--shareable",
            "https://private.example.com/products?category=books",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Normalization findings: 0"));
    assert!(stdout.contains("Potential embedded destinations: 0"));
    assert!(!stdout.contains("Observations"));
    assert!(!stdout.contains("private.example.com"));
    assert!(!stdout.contains("products"));
    assert!(!stdout.contains("category"));
    assert!(!stdout.contains("books"));
    assert!(stderr.is_empty());
}

#[test]
fn cli_default_mode_remains_detailed() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .arg("https://example.com/documents/123?token=secret123")
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("LinkSleuth URL Analysis"));
    assert!(stdout.contains("Hostname: example.com"));
    assert!(stdout.contains("Path:     /documents/123"));
    assert!(stdout.contains("Parameter: token"));

    assert!(!stdout.contains("LinkSleuth Shareable Summary"));
    assert!(stderr.is_empty());
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

#[test]
fn cli_rejects_hostname_disclosure_outside_shareable_mode() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args([
            "--include-hostname",
            "https://private.example.com/?token=secret123",
        ])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.is_empty());
    assert!(stderr.contains("Usage:"));
    assert!(!stderr.contains("private.example.com"));
    assert!(!stderr.contains("secret123"));
}

#[test]
fn cli_rejects_missing_hostname_option_url_without_echoing_arguments() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
        .args(["--shareable", "--include-hostname"])
        .output()
        .expect("Failed to execute LinkSleuth CLI");

    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.is_empty());
    assert_eq!(
        stderr,
        "Usage: linksleuth-cli [--shareable [--include-hostname]] <URL>\n"
    );
}

#[test]
fn cli_analysis_errors_do_not_echo_the_submitted_value() {
    for args in [
        vec!["--shareable", "not-a-url-secret123"],
        vec!["--shareable", "--include-hostname", "not-a-url-secret123"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_linksleuth-cli"))
            .args(args)
            .output()
            .expect("Failed to execute LinkSleuth CLI");

        assert_eq!(output.status.code(), Some(1));

        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        assert!(stdout.is_empty());
        assert_eq!(stderr, "Unable to analyze URL: the URL is invalid.\n");
        assert!(!stderr.contains("secret123"));
        assert!(!stderr.contains("not-a-url"));
    }
}
