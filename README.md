# LinkSleuth

**Understand a link before you click it.**

LinkSleuth is an open-source, privacy-first URL analysis engine written in Rust and maintained by Sweetwater Media Inc. It examines URL structure, reveals layers of percent encoding, and identifies possible embedded destinations without contacting the websites being analyzed.

> **Status:** Early development (v0.1.0). LinkSleuth does **not** determine whether a URL is safe or malicious.

## Principles

1. **Open source:** The core engine is licensed under Apache-2.0 and designed for reuse by others.
2. **Evidence over assumptions:** Observations and conclusions must be traceable to specific input and transformations.
3. **Passive inspection:** Never follow links, perform DNS lookups, contact suspicious destinations, or execute remote content during analysis.
4. **Privacy by design:** Prefer local analysis and data minimization. We do not sell user data or browsing activity.
5. **Accessible security:** Explain technical observations in language nontechnical users can understand.

## Current capabilities

- Parse HTTP and HTTPS URLs into scheme, hostname, path, port, query parameters, and fragment.
- Preserve original URL input alongside the URL parser's normalized interpretation, including changes to hostname case, default ports, and dot-segment paths.
- Preserve raw query parameter values alongside decoded equivalents.
- Retain repeated query parameter names in order, including empty values and flag-style parameters.
- Record whether each raw query parameter contained a literal equals sign (`has_equals`), distinguishing `flag` from `empty=`.
- Preserve distinct encoded forms such as `+`, `%20`, `%2B`, `%26`, and `%3D` for evidence review.
- Decode percent encoding in up to three passes, retaining each transformation.
- Preserve malformed percent sequences as warnings while decoding valid sequences.
- Record the decoding-attempt number and byte offset for each warning.
- Report whether further percent decoding may remain.
- Identify complete HTTP/HTTPS URLs embedded in decoded query parameter values.
- Produce preliminary structured findings for encoded parent-directory segments in original URL paths, showing original and normalized paths with an evidence-based explanation.
- Include normalization findings alongside decoded parameters and possible embedded destinations in the reusable `AnalysisReport`.
- Recognize forward and backslash separators in supported original-path evidence, while excluding query strings and fragments from these findings.
- Conservatively omit normalization findings for unsupported ambiguous scheme separators rather than asserting uncertain path evidence.
- Inspect URLs using a command-line tool.
- Run automated tests covering parsing, decoding, and discovery.

**Limitations:** Normalization findings currently recognize a limited set of encoded parent-directory patterns; raw-path extraction needs further edge-case hardening, and normalization findings are not yet displayed by the CLI. Embedded URLs are *possible destinations*, not verified redirects. The current engine does not resolve redirects, inspect webpage content, query reputation feeds, or determine whether a link is safe. The CLI currently displays raw URL information, which may include personal information or tokens; review output carefully before sharing it.

## Build and test

Requires Git and a Rust toolchain supporting the Rust 2024 edition.

```bash
git clone https://github.com/Sweetwater-Media-Inc/linksleuth.git
cd linksleuth
cargo build --workspace
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Inspect a link

```bash
cargo run -p linksleuth-cli -- \
  "https://example.com/?target=https%253A%252F%252Fother.example"
```

The CLI reports URL components, decoding steps, and any supported embedded URL discovery. No suspicious destination is contacted.

## Project structure

```text
crates/
  linksleuth-core/  Reusable Rust analysis library
  linksleuth-cli/   Command-line interface
```

The core library is independent of its CLI consumer. Planned consumers include **Clue**, a friendly web interface, and browser/email integrations.

## Roadmap and changes

Read [ROADMAP.md](ROADMAP.md) for priorities and [CHANGELOG.md](CHANGELOG.md) for completed changes.

## Security and privacy notes

A submitted URL may contain authentication tokens, email addresses, or private document identifiers. Avoid posting sensitive URLs publicly, and do not share unsanitized CLI output. Privacy-safe shareable reports are planned; they are not yet implemented.

Passive URL inspection can reveal syntax and embedded destinations, but **cannot guarantee a link is safe**. Continue using normal browser and email security protections.

## License and ownership

Copyright 2026 Sweetwater Media Inc.

Licensed under the [Apache License, Version 2.0](LICENSE).
