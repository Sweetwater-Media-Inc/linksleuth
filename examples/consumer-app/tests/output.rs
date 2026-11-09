use std::process::Command;

#[test]
fn default_output_uses_only_privacy_reduced_consumer_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-consumer-app"))
        .arg(concat!(
            "https://private.example/private/path?token=fake-secret-123",
            "&target=https%3A%2F%2Fdestination.example%2Fprivate"
        ))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Schema version: 1"));
    assert!(stdout.contains("destination.absolute-candidate"));
    assert!(stdout.contains("1 occurrence(s)"));
    assert!(stdout.contains("original-path: complete within supported scope"));
    assert!(stdout.contains("Detailed evidence and the original hostname are hidden by default."));
    for sensitive in [
        "private.example",
        "destination.example",
        "fake-secret-123",
        "token",
        "/private",
    ] {
        assert!(!stdout.contains(sensitive));
        assert!(!stderr.contains(sensitive));
    }
    assert!(stderr.is_empty());
}

#[test]
fn zero_observations_are_not_presented_as_a_safety_verdict() {
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-consumer-app"))
        .arg("https://ordinary.example/products?category=books")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("No supported structural observations were produced."));
    assert!(stdout.contains("This is not a declaration that the URL is safe."));
    assert!(!stdout.contains("ordinary.example"));
    assert!(!stdout.contains("products"));
    assert!(!stdout.contains("category"));
    assert!(!stdout.contains("books"));
}

#[test]
fn errors_are_fixed_and_do_not_reflect_input() {
    let secret = "fake-secret-value";
    let output = Command::new(env!("CARGO_BIN_EXE_linksleuth-consumer-app"))
        .arg(format!("not a URL {secret}"))
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.is_empty());
    assert_eq!(
        stderr,
        "Analysis error: Enter a complete HTTP or HTTPS URL.\n"
    );
    assert!(!stderr.contains(secret));
}
