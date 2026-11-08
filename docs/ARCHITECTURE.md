# LinkSleuth Architecture

## Purpose and status

LinkSleuth is a Rust 2024 workspace for **passive** HTTP/HTTPS URL inspection. The reusable engine preserves original and normalized evidence, performs bounded percent decoding, identifies candidate embedded HTTP/HTTPS URLs, and produces cautious observations. It does **not** visit destinations or determine safety. The intended future **Clue** interface and browser/email integrations are consumers, not part of the current engine.

## Workspace

- `crates/linksleuth-core`: reusable library; exports `analysis`, `parser`, `decoder`, `destinations`, `normalization`, and `reporting` from `src/lib.rs`.
- `crates/linksleuth-cli`: executable entry point in `src/main.rs`; integrates the library and formats detailed or privacy-reduced output.
- `crates/linksleuth-cli/tests/cli_output.rs`: integration tests that execute the compiled CLI.
- Root `Cargo.toml`: workspace membership, Rust 2024 edition, Apache-2.0 license, development version 0.1.0.

## Data flow

1. `parser::parse_url(input)` limits submitted URL length, accepts HTTP/HTTPS, parses via the `url` crate, and retains `ParsedUrl.original` **alongside** `ParsedUrl.normalized`. Query parameters retain raw names/values and form-decoded representations, repeated parameters, and whether a literal equals sign was present.
2. `normalization::analyze_normalization(&parsed)` compares supported original path evidence with the normalized path. Its current typed finding recognizes encoded parent-directory segments. Uncertain original-path extraction can produce **no finding**, which is not proof the URL lacks other normalization changes.
3. `decoder::decode_percent(raw_value)` applies bounded iterative percent decoding, retaining transformation steps, status, and warnings with pass and byte-offset provenance. This is distinct from form-style `+` decoding performed when parsing query parameters.
4. `destinations::discover_destination(decoded, parameter_name, passes)` identifies **possible** complete HTTP/HTTPS URLs in decoded parameter values. It does not test whether any server redirects.
5. `analysis::analyze_url(input)` composes the above into `AnalysisReport`: parsed URL, parameter-level decoding/candidate destinations, and normalization findings. These structures are detailed evidence and can contain secrets.
6. `reporting::create_shareable_report(&analysis)` produces a new, privacy-reduced `ShareableReport` without modifying the source. `create_shareable_report_with_options(..., ShareableReportOptions)` supports an **explicit** hostname opt-in. The report contains aggregate counts and fixed, typed `ShareableFindingKind` categories: encoded parent-directory segment, possible embedded destination, malformed percent encoding, and decoding pass limit. One category may summarize several events. Explanations are static, not assembled from URL text.
7. The CLI prints detailed evidence in default mode, a redacted-hostname report with `--shareable`, or opt-in hostname disclosure with `--shareable --include-hostname`.

## Important separations

- **Parsing versus normalization:** parser-normalized paths must not replace original URL evidence.
- **Form parsing versus percent decoding:** raw, form-decoded, and successive percent-decoded representations have different semantics.
- **Observed candidate versus verified action:** an embedded destination candidate does not demonstrate network navigation or redirection.
- **Evidence versus presentation:** shareable reporting consumes analysis but must never mutate it or copy sensitive source strings into observations. A report's `Debug` implementation deliberately redacts the hostname even when its explicit display field includes an opted-in value.
- **Core versus consumers:** keep display-oriented formatting and CLI argument handling outside the engine.

## Current limitations and future work

Path evidence extraction does not cover every unusual URL spelling; absence of a normalization finding is not a comprehensive assertion. Candidate destination detection recognizes complete HTTP/HTTPS URLs rather than every nested encoding or redirect mechanism. Pass/size limits are deliberate; larger inputs and deeper analysis require a bounds review. Shareable findings and counts are metadata, not an anonymous export. CI, additional fuzz/property testing, and richer discovery are roadmap items, not assumed capabilities.

See [SECURITY.md](SECURITY.md) for threat/privacy constraints, [DEVELOPMENT.md](DEVELOPMENT.md) for workflow, and [../ROADMAP.md](../ROADMAP.md) for priorities.
