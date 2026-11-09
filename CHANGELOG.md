# Changelog

All notable changes to LinkSleuth are documented here. This project uses the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

### Added
- Milestone 6.14: additive `consumer` module with schema-versioned `ConsumerReport` contract (schema 1, independent of crate version), private fields/accessors, non-exhaustive extensible enums, stable observation identifiers, deterministic ordering and aggregated occurrence counts.
- Consumer-facing four-stage completeness model (original path, percent decoding, destination traversal, secondary encoding) with structured stage statuses and fixed explanations; completeness is only within supported scope.
- Independent opt-in controls for original hostname and detailed evidence, with evidence references separated from actual evidence values and redacted diagnostic `Debug` output.
- Synthetic regression corpus expanded from 27 to 42 cases; test harness reports 21 expected positive, 21 expected negative, zero unexpected candidates, zero missed candidates, and seven incomplete/unsupported cases against corpus expectations (144 total reported local tests).
- Milestone 6.13: bounded whole-value Base64 decoding (standard, URL-safe, padded/unpadded; ambiguous alphabet classification) with strict padding, trailing-bit and UTF-8 validation, plus conservative HTML character-reference decoding (decimal, hexadecimal, and exactly `&colon;`, `&sol;`, `&amp;`).
- Typed secondary encoding results, coverage issues, and ordered `DestinationTransformation` provenance integrated into existing breadth-first nested destination discovery. One secondary decoding stage may follow bounded percent decoding per value.
- Secondary decoding limits: 16 KiB encoded input, 8 KiB decoded output, and 32-byte HTML-reference token scanning; explicit coverage and limit status keep unexamined or malformed inputs distinct from negative findings.
- Shareable counts and fixed-text findings for Base64- and HTML-reference-derived candidates; detailed CLI outputs transformation order and characteristics.
- Machine-readable 27-case synthetic TSV regression corpus exercised by automated integration tests, covering valid, benign, malformed, ambiguous, nested, bounded, and privacy-sensitive scenarios (132 tests reported passing locally).
- Milestone 6.12: typed absolute HTTP/HTTPS and protocol-relative reference classification, with scheme-neutral handling of protocol-relative references and conservative rejection of ambiguous spellings.
- Bounded breadth-first nested destination discovery with provenance (parameter positions, parent candidate, depth, decoding steps), duplicate occurrence retention, and deduplicated traversal work.
- Typed destination-discovery completeness status and fixed limit explanations; limits are nesting depth 3, 64 candidate occurrences, and 32 parameters per nested URL, reusing existing URL/decoder limits.
- Privacy-reduced report counts and fixed-text findings for protocol-relative references and incomplete discovery; detailed CLI now shows direct/nested relationships and decoding evidence.
- Regression tests covering nested and protocol-relative references, IPv6/ports, malformed inputs, traversal limits, provenance, CLI output and privacy isolation (109 total tests in the reported local verification).
- Milestone 6.11: typed original-path analysis coverage (`OriginalPathAnalysis`, `OriginalPathUnsupportedReason`, `NormalizationAnalysis`, and `analyze_normalization_with_coverage()`) exposed through analysis and privacy-reduced reports.
- Public limits for submitted URL size (16 KiB), non-empty query parameters (256), decoder input (16 KiB), and decoding passes (three), with boundary and Unicode-byte-length tests.
- GitHub Actions Rust CI workflow to check formatting, workspace tests, and Clippy on pushes and pull requests targeting `main`.
- Additional tests for URL authority spelling, supported versus unsupported original path extraction, IDN/Unicode inputs, encoding and raw/form-decoded provenance, resource limits, and reporting/CLI compatibility.
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
- Milestone 6.14 preserves existing detailed analysis, shareable report API, and CLI output unchanged; the new consumer API is the recommended integration surface for Clue and future consumers. No serialization/wire format was introduced and real-world false-positive rates have not been measured.
- Milestone 6.13: `DestinationDiscovery` carries encoding coverage and issues alongside resource-limit status; use `DestinationDiscovery::is_complete()` to consider both. Several public report/provenance structs gained fields and enums gained variants, potentially affecting downstream struct literals or exhaustive matches.
- Destination classification now conservatively rejects control characters in candidate references.
- Milestone 6.12: `AnalysisReport` includes `destination_discovery`. `ShareableReport` adds nested/protocol-relative counts and discovery status, while the legacy direct absolute destination API remains available. Added public fields and enum variants may affect downstream struct literals or exhaustive matches.
- Milestone 6.11: raw query values now reflect exact submitted input rather than URL-parser-normalized representations; normalized and form-decoded versions remain distinct.
- Ambiguous original-path evidence now carries explicit unsupported coverage instead of being silently represented by an empty normalization finding list.
- URL inputs with more than 256 non-empty query parameters now fail with `UrlError::TooManyQueryParameters`. Public report struct additions and this new error variant may require updates to downstream consumers.
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
