# LinkSleuth Architecture

## Purpose and status

LinkSleuth is a Rust 2024 workspace for **passive** HTTP/HTTPS URL inspection. The reusable engine preserves original and normalized evidence, performs bounded percent decoding, identifies candidate embedded HTTP/HTTPS URLs, and produces cautious observations. It does **not** visit destinations or determine safety. The intended future **Clue** interface and browser/email integrations are consumers, not part of the current engine.

## Workspace

- `crates/linksleuth-core`: reusable library; exports `analysis`, `parser`, `decoder`, `destinations`, `normalization`, and `reporting` from `src/lib.rs`.
- `crates/linksleuth-cli`: executable entry point in `src/main.rs`; integrates the library and formats detailed or privacy-reduced output.
- `crates/linksleuth-cli/tests/cli_output.rs`: integration tests that execute the compiled CLI.
- `crates/linksleuth-core/src/encodings.rs`: bounded, strict complete-value Base64 decoding and conservative supported HTML character-reference decoding.
- `crates/linksleuth-core/tests/corpus/destination_cases.tsv` and `tests/regression_corpus.rs`: machine-readable synthetic destination corpus with automated expectations.
- Root `Cargo.toml`: workspace membership, Rust 2024 edition, Apache-2.0 license, development version 0.1.0.

## Data flow

1. `parser::parse_url(input)` accepts HTTP/HTTPS, parses via the `url` crate, and retains `ParsedUrl.original` **alongside** `ParsedUrl.normalized`. Query parameter raw text is derived from the exact submitted input, not reconstructed from the parser-normalized URL; form-decoded values are stored separately. Repetitions, flags, and literal equals-sign presence are retained. The submitted URL is bounded to 16 KiB and non-empty parameter count to 256.
2. `normalization::analyze_normalization_with_coverage(&parsed)` returns `NormalizationAnalysis`, which distinguishes `OriginalPathAnalysis::Examined` from `OriginalPathAnalysis::Unsupported` (with a typed reason), alongside supported normalization findings. This matters because an unsupported path must not be mistaken for a clean finding result. The original findings-only `analyze_normalization()` interface remains available. Coverage includes conservative interpretation of selected zero-to-two-separator authority spellings; ambiguous cases are explicitly unsupported.
3. `decoder::decode_percent(raw_value)` applies up to three percent-decoding passes with a 16 KiB decoder-input limit, retaining transformation step inputs/outputs, status, and warnings with pass and UTF-8 byte-offset provenance. This is distinct from form-style `+` decoding performed when parsing query parameters.
4. `destinations::discover_destination_reference(value)` classifies supported complete HTTP/HTTPS URLs and protocol-relative URL references without contacting them. `discover_destinations(seeds)` performs iterative breadth-first nested candidate discovery. `DestinationProvenance` retains top-level and immediate-parent parameter positions, source names, decoding history, depth (direct = 0), and parent candidate index. A value is percent-decoded first, classified directly, and, if no candidate is found, may undergo **at most one** bounded Base64 **or** supported HTML character-reference stage before final classification. `DestinationTransformation` records ordered stages and variants; `encodings` exposes typed outcomes for malformed/unsupported values. Candidate occurrences remain distinct evidence; normalized-reference traversal deduplication prevents repeated nested work, so descendants attach to the first traversed occurrence. The original `discover_destination()` interface remains for legacy directly embedded absolute URLs. None of these results establish redirects.
5. `analysis::analyze_url(input)` composes the above into `AnalysisReport`: parsed URL, parameter-level decoding/legacy direct candidates, `destination_discovery` including separate resource-limit status and encoding-coverage metadata, normalization findings, and `original_path_analysis` coverage. Use `DestinationDiscovery::is_complete()` to consider both forms of incompleteness. These structures are detailed evidence and can contain secrets.
6. `reporting::create_shareable_report(&analysis)` produces a new, privacy-reduced `ShareableReport` without modifying the source. `create_shareable_report_with_options(..., ShareableReportOptions)` supports an **explicit** hostname opt-in. The report contains original-path coverage metadata, aggregate candidate counts (including nested, protocol-relative, Base64-derived and HTML-reference-derived), discovery and encoding-coverage statuses, and fixed typed `ShareableFindingKind` categories. These include original normalization/decoding findings, potential encoded destinations, and incomplete discovery; no source strings are copied into shareable observations. One category may summarize several events. Explanations are static, not assembled from URL text.
7. The CLI prints detailed evidence in default mode, a redacted-hostname report with `--shareable`, or opt-in hostname disclosure with `--shareable --include-hostname`.

## Important separations

- **Parsing versus normalization:** parser-normalized paths must not replace original URL evidence.
- **Form parsing versus percent decoding:** raw, form-decoded, and successive percent-decoded representations have different semantics.
- **Observed candidate versus verified action:** an embedded or nested destination candidate does not demonstrate navigation, a redirect chain, or a verified final destination. Protocol-relative references have no verified scheme.
- **Evidence versus presentation:** shareable reporting consumes analysis but must never mutate it or copy sensitive source strings into observations. A report's `Debug` implementation deliberately redacts the hostname even when its explicit display field includes an opted-in value.
- **Core versus consumers:** keep display-oriented formatting and CLI argument handling outside the engine.

## Current limitations and future work

Path evidence extraction does not cover every unusual URL spelling (including ambiguous authority boundaries and certain parser-trimmed inputs). Check the typed coverage before interpreting an empty normalization finding set. Candidate destination detection recognizes complete HTTP/HTTPS or protocol-relative references in supported query values and follows nested query values only. It does not scan arbitrary substrings or chain secondary Base64/HTML decoders. Discovery is bounded to depth 3, 64 recorded candidates, and 32 parameters per nested URL. Secondary encoded input is limited to 16 KiB, decoded output to 8 KiB, and HTML reference tokens to 32 bytes (in addition to URL and percent-decoder bounds). Limit hits are explicitly reported as incomplete. URL, query-count, decoder-input, and pass limits are deliberate; larger inputs and deeper analysis require a bounds review. Shareable findings and counts are metadata, not an anonymous export. Rust CI is configured for main pushes and pull requests; a passing remote workflow must be verified separately. Fuzz/property testing and richer discovery remain roadmap items.

See [SECURITY.md](SECURITY.md) for threat/privacy constraints, [DEVELOPMENT.md](DEVELOPMENT.md) for workflow, and [../ROADMAP.md](../ROADMAP.md) for priorities.
