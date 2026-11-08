# Changelog

All notable changes to LinkSleuth are documented here. This project uses the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

### Added
- Rust workspace with reusable `linksleuth-core` library and `linksleuth-cli` executable.
- HTTP/HTTPS URL parsing, including query parameters and fragments.
- Preservation of encoded and decoded query parameter values.
- Bounded multi-pass percent decoding with an evidence trail and completion status.
- Per-parameter decoding results and error isolation.
- Passive discovery of complete HTTP/HTTPS URLs embedded in query parameter values.
- Automated tests for parsing, decoding, integration, and destination discovery.
- Regression tests for ambiguous query encoding, repeated parameter names, and empty or flag-style query values (Milestone 6.2).
- Regression tests confirming that original URLs are retained alongside normalized interpretations, including encoded dot-segments and default HTTPS ports.
- `normalization` core module with `NormalizationKind`, `NormalizationFinding`, and `analyze_normalization` to identify encoded parent-directory segments in URL paths and retain original versus normalized path evidence.
- Tests for mixed-case and mixed encoded-dot forms and for excluding query-only dot segments.
- Original-path extraction tests for URL credentials, missing explicit paths, fragments, and backslashes; backslash path separators are now preserved as evidence.
- Regression tests for IPv6 authorities and ports, encoded credentials, backslashes after ports, and ambiguous scheme separators. Normalization detection now declines uncertain original-path extraction for the tested ambiguous scheme case.
- Two CLI integration tests that execute the binary and verify normalization finding text and decoding warning output.
- CLI regression test distinguishing malformed percent sequences from completion of percent decoding.
- Initial `reporting` module exposing `ShareableReport` and `create_shareable_report()`, with hostname redacted and aggregate query-parameter, normalization-finding, and candidate-destination counts.
- Three reporting tests covering sensitive-value exclusion from summary and `Debug` output, and preservation of original analysis evidence.
- CLI `--shareable` option to present a privacy-reduced report using the reusable core reporting module; three new CLI integration tests cover shareable output, unchanged detailed mode, and non-echoing argument errors.
- Milestone 6.10: typed `ShareableFindingKind`/`ShareableFinding` categories for normalization, candidate embedded destinations, malformed percent encoding, and pass limits, using fixed titles and explanations.
- `ShareableReportOptions` and `create_shareable_report_with_options()` for explicit hostname disclosure while preserving the safe default API.
- Expanded CLI tests for exact privacy-reduced output, no URL-derived evidence in shareable stdout/stderr, hostname opt-in, invalid arguments, and unchanged detailed output.

### Changed
- Milestone 6.10: shareable CLI output now renders structured observations; the explicit `--shareable --include-hostname` combination can reveal the normalized hostname. Diagnostic `Debug` formatting still masks it. CLI analysis errors use fixed messages rather than echoing URL input.
- Clarified CLI percent-decoding wording: no successfully decoded sequences, no further valid sequences detected, and pass-limit conditions are shown separately from malformed-sequence observations.
- The CLI displays human-readable normalization finding titles and malformed percent-sequence observations with decoding pass and byte offset.
- The CLI now displays structured normalization findings, original and normalized paths, and cautious explanations when supported URL path transformations are detected.
- `analyze_url()` now includes structured normalization findings in `AnalysisReport` alongside existing parameter decoding and candidate destination results; integration tests cover both a detected normalization and an ordinary URL.
- Query parameter parsing now records the presence of a literal `=` separator, preserving the difference between `flag` and `empty=` without confusing encoded `%3D` with a separator.
- Percent decoding now preserves malformed or literal percent sequences and continues decoding valid sequences.
- Decoding warnings include their decoding pass and byte offset for evidence provenance.
- Tests cover literal percent signs, mixed valid/malformed input, and warnings from later decoding passes.

### Documentation
- Project README, changelog, and roadmap.
- Repository-wide `AGENTS.md` plus `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, and `docs/DEVELOPMENT.md` to preserve engineering constraints and review practices.

## Releases

No tagged releases yet. The crates currently use development version `0.1.0`. The entries above describe work on the main development branch, not a published release.
