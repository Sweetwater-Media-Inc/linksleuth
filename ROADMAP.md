# LinkSleuth Development Roadmap

LinkSleuth is an open-source, evidence-driven, passive URL analysis engine written in Rust and maintained by Sweetwater Media Inc.

This is a living plan, not a promise of release dates. Completed items describe implemented features on the development branch, not necessarily production-ready functionality.

## Completed foundations

- [x] Create the public Apache-2.0 Rust repository under Sweetwater Media Inc.
- [x] Initialize the Cargo workspace and reusable core library.
- [x] Build a command-line interface.
- [x] Parse HTTP/HTTPS URLs and examine query parameters.
- [x] Preserve encoded and decoded query parameter values.
- [x] Implement bounded multi-pass percent decoding with transformation history.
- [x] Track decoding status and isolate parameter-specific errors.
- [x] Identify complete HTTP/HTTPS URLs embedded in query parameter values.
- [x] Add automated tests across parsing, decoding, and analysis.

## Next: Correctness and security hardening

- [x] Add tests for literal percent signs, malformed sequences, and UTF-8 decoding.
- [x] Record pass-aware warnings with byte offsets for each decoding attempt.
- [x] Add regression tests for empty values, repeated parameter names, and ambiguous encoded separators.
- [x] Preserve the presence of a literal equals sign for each query parameter (`flag` versus `empty=`).
- [x] Verify original URL preservation alongside normalization of hostnames, default ports, and encoded dot-segments.
- [x] Introduce a preliminary structured normalization finding for encoded parent-directory segments, preserving original and normalized paths and a cautious explanation.
- [x] Add original-path extraction regression tests for credentials, query/fragment boundaries, and backslash separators.
- [x] Test IPv6 authority syntax, encoded credentials, explicit ports, and conservatively skip ambiguous scheme-separator paths.
- [x] Integrate normalization findings into the reusable `AnalysisReport` with regression tests.
- [x] Add typed original-path coverage and conservative extraction across supported unusual authority spellings; distinguish unsupported extraction from examined paths with no finding.
- [ ] Continue researching unsupported unusual scheme/authority spellings without building a separate WHATWG parser.
- [x] Display normalization findings with original/normalized path evidence and explanations in the CLI.
- [x] Improve CLI normalization titles, show pass-aware decoding warnings, and add initial automated CLI output tests.
- [x] Refine percent-decoding status wording and add a regression test for malformed encoding alongside decoding completion.
- [x] Extend tests across CLI, Unicode, multi-pass decoding, and mixed valid/malformed sequences.
- [x] Add Unicode hostname, path and query tests, UTF-8 byte-offset tests, mixed hex-case decoding tests, and query/fragment boundary tests.
- [ ] Expand adversarial Unicode and unusual-URL regression coverage as new real-world cases arise.
- [x] Verify distinct raw and form-decoded values for `+`, `%20`, `%2B`, `%26`, and `%3D`.
- [x] Document raw versus form-decoded query representations and percent decoding semantics in the analysis API.
- [x] Preserve exact submitted query evidence separately from normalized/form-decoded values, with decoding-pass input/output and byte-offset provenance.
- [x] Separate percent-decoding completion/pass-limit status from original-path analysis coverage; avoid overstating completeness.
- [x] Enforce explicit limits on submitted URL bytes, query-parameter count, decoder input bytes and decoding passes.
- [ ] Evaluate separate warning-count caps and broader performance/fuzz testing.
- [x] Introduce a minimal privacy-reduced core report with redacted hostname and aggregate analysis counts, with tests for evidence preservation and example-sensitive-data exclusion.
- [x] Add safe-default `ShareableReportOptions` with explicit hostname disclosure, typed shareable finding categories, fixed explanatory text, and privacy regression tests.
- [ ] Continue reviewing metadata disclosure risks and expand privacy controls if justified; there is no general anonymity guarantee.
- [x] Integrate `--shareable` privacy-reduced CLI output using the core reporting module, with end-to-end regression tests.
- [x] Add privacy regression tests for CLI standard error, invalid arguments and analysis failures, and hostname opt-in.
- [ ] Evaluate safer URL input mechanisms to reduce shell-history and process-argument exposure.
- [x] Add GitHub Actions checks for formatting, tests, and Clippy on main pushes and pull requests.
- [ ] Confirm initial remote CI outcome and keep workflow maintained.

## Next: Expanded discovery and explainability

- [x] Recognize protocol-relative candidate URLs, with typed classification and conservative authority validation.
- [x] Discover candidate URLs nested within candidate query parameters, with typed parent/position/decoding provenance and bounded breadth-first traversal.
- [ ] Consider additional nested parameter structures without arbitrary substring guessing.
- [x] Add bounded whole-value Base64 and limited HTML character-reference decoding with strict validation, explicit transformation order, and conservative candidate classification.
- [ ] Evaluate additional encodings or named HTML references only with measurable value and false-positive evidence.
- [x] Distinguish absolute and protocol-relative candidates from verified redirects; do not claim navigation or a final destination.
- [x] Introduce initial fixed-text, privacy-reduced structured findings for four supported observations.
- [x] Extend detailed candidate provenance and privacy-reduced reporting to include nested/protocol-relative findings and incomplete discovery status.
- [ ] Expand detailed findings beyond current supported candidate and normalization categories.
- [x] Add automated, machine-readable synthetic destination regression corpus (expanded to 42 cases in Milestone 6.14) for legitimate, suspicious-looking, malformed, nested, ambiguous, bounded, and privacy-sensitive cases.
- [x] Expand the corpus with 15 additional benign and misleading-positive cases, and add aggregate expectation-mismatch metrics (Milestone 6.14).
- [ ] Continue expanding the corpus as new real-world-inspired, synthetic regression cases are identified.
- [ ] Introduce fuzz/property-based tests and evaluate false positives using the corpus and broader synthetic data.

## Development documentation

- [x] Add `docs/CLUE_MVP.md` and `docs/CLUE_INTEGRATION_READINESS.md`, including the confirmed WASM compilation boundary and remaining browser tasks.

- [x] Document Milestone 6.14 consumer schema, disclosure boundary, stage completeness and corpus-based quality metrics.

- [x] Document bounded Base64 and HTML-reference decoding, typed coverage, provenance, limitations, and regression corpus from Milestone 6.13.

- [x] Document Milestone 6.12 candidate discovery, limits, status and privacy boundaries.

- [x] Add repository-level `AGENTS.md` and architecture, security, and development guidance in `docs/`.
- [ ] Keep guidance in sync with future architectural or privacy changes.

## Next: API stabilization and detection quality

- [x] Audit public API layering and add a schema-versioned, encapsulated consumer interface while retaining legacy detailed/shareable APIs.
- [x] Exercise the public consumer API from an independent Cargo workspace package (Milestone 6.15).
- [ ] Establish a consumer-contract compatibility policy before promoting a stable version-1 product.
- [x] Add four-stage consumer completeness with explicit coverage/reason identifiers and status scoped to supported techniques.
- [x] Measure expected-positive/negative corpus mismatches without claiming real-world detection accuracy.
- [ ] Add more diverse synthetic and adversarial samples to evaluate detection quality beyond the current corpus.
- [x] Verify `wasm32-unknown-unknown` compilation, add an isolated privacy-reduced WASM adapter with versioned JSON wire output, and define Clue's local-first MVP (Milestone 6.15).
- [ ] Package the WASM adapter with wasm-bindgen and verify JavaScript/browser-runtime integration.

## Future: Clue companion web interface

- [x] Compile the reusable core and browser adapter for `wasm32-unknown-unknown` (compilation only; browser execution remains unverified).
- [ ] Build a minimal static Clue prototype and exercise the WASM output in an actual browser.
- [ ] Instrument browser-runtime tests to confirm analysis does not request submitted or discovered destinations.
- [ ] Build a simple accessible interface to paste and inspect links.
- [ ] Prefer local processing so submitted URLs remain on the user's device.
- [ ] Show understandable observations with optional technical evidence details.
- [ ] Offer redacted, shareable reports.
- [ ] Publish explicit privacy and security documentation.

## Longer-term possibilities

- [ ] Browser extensions with manual right-click inspection.
- [ ] Email-client integrations, subject to each client's extension capabilities.
- [ ] Optional, privacy-conscious reputation intelligence with separately disclosed network activity.
- [ ] Developer-facing API or library integration examples.
- [ ] Sustainable business offerings that do not compromise the free open-source engine or user privacy.

## Non-goals for the initial engine

- No active navigation, link following, DNS resolution, or webpage fetching.
- No claim that an embedded URL proves a redirect or that a parsed URL is safe.
- No collection or sale of user browsing histories or submitted URLs.
- No requirement for user accounts, subscriptions, or remote services to perform basic analysis.

See [README.md](README.md) for usage and [CHANGELOG.md](CHANGELOG.md) for implemented changes.
