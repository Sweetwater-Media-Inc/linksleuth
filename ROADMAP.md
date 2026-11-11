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
- [x] Add automated, machine-readable synthetic destination regression corpus (expanded to 62 cases in Milestone 6.17) for legitimate, suspicious-looking, malformed, nested, ambiguous, bounded, and privacy-sensitive cases.
- [x] Expand the corpus with 15 additional benign and misleading-positive cases, and add aggregate expectation-mismatch metrics (Milestone 6.14).
- [ ] Continue expanding the corpus beyond 62 cases as new real-world-inspired, synthetic regression cases are identified.
- [ ] Introduce fuzz/property-based tests and evaluate false positives using the corpus and broader synthetic data.

## Milestone 6.17: real-world structural discovery and Clue clarity

- [x] Recognize narrowly scoped hostname-first references without assigning an unverified URL scheme.
- [x] Add one-interpretation bounded compound query-value extraction, including encoded HTML delimiters and complete protocol-relative inner destination values.
- [x] Add privacy-safe structural observations and interpretation ambiguity coverage; distinguish no candidate from no noteworthy supported evidence.
- [x] Expand the machine-readable synthetic corpus from 42 to 62 cases with a motivating-example structural equivalent, and maintain zero expectation mismatches in reported local tests.
- [x] Simplify Clue results: plain-language full-width findings, accessible expandable completeness details, and concise non-verdict messages.
- [ ] Continue evaluating ambiguous hostname/path constructs and novel compound forms only when classification is defensible and bounded.

## Milestone 6.18: context-aware assessment and decisive guidance

- [x] Add an independently testable Rust assessment crate consuming the public privacy-reduced core consumer interface, without changing the passive engine.
- [x] Provide immediate, qualified preliminary assessments and optional progressive context questions after analysis; keep all context local and allow revision/clearing.
- [x] Use deterministic evidence families, context attribution and explicit limitations instead of numeric phishing probabilities or conflating correlated findings.
- [x] Lead strong-warning results with **Do not open this link**, fixed explanations and at most three practical next steps; preserve distinct technical evidence, user-reported context and unverified inferences.
- [x] Add a separate 29-case synthetic assessment corpus and actual-WASM Chromium tests; local test reports show zero analysis-triggered requests, not a general deployment guarantee.
- [x] Maintain independently versioned consumer schema 1, assessment contract 1 and browser wire schema 2.
- [ ] Evaluate assessment calibration and false reassurance against broader labelled and synthetic examples, including phishing-like links with ordinary URL structure.
- [ ] Test Firefox/WebKit, manually review accessibility and validate production privacy/CSP behaviour before public launch.

## Milestone 6.19: Clue results layout and usability

- [x] Show recommendation and optional context questions side by side on desktop (roughly 55/45) and stack them on smaller screens with a logical reading order.
- [x] Increase desktop results space without stretching introductory/analyzer content; retain readable text and controls.
- [x] Expose the first two optional context questions immediately, with the remaining two available on demand and all context editable/clearable.
- [x] Collapse supporting structural findings by default, retain dynamic count and full evidence explanations, and keep important assessment limitations visible.
- [x] Verify initial and context-aware layouts at 390, 768, 1280 and 1920 pixels in local Chromium; reported 23 actual-WASM Chromium tests and no triggered analysis network requests.
- [ ] Manually verify screen-reader behaviour and test browser engines beyond Chromium before public release.

## Milestone 6.20: unified Clue and controlled destination evidence

- [x] Align navigation, left-aligned hero, URL analyzer, results and footer to the same responsive 88rem outer container.
- [x] Keep assessment/context side by side on desktop, with compact primary recommendation and full-width URL/context reasons below.
- [x] Preserve the privacy-reduced default browser response and introduce separately invoked, local-only candidate-hostname and detailed-evidence disclosure levels.
- [x] Render all revealed URL evidence as inert text, never a clickable link or preview; clear disclosed DOM nodes on hide and every new analysis.
- [x] Keep credentials and literal fragments out of displayed candidate representations, warn before revealing detail that query parameters may contain sensitive data, and exclude disclosed evidence from reviewed/copied summaries.
- [x] Independently version destination-disclosure contracts (core 1, browser 1) without changing default wire schema 2, consumer schema 1 or assessment contract 1.
- [x] Verify local Chromium disclosure/privacy/layout behaviour (reported: 182 Rust tests, 8 contract/privacy tests and 29 actual-WASM browser tests; no analysis-triggered network requests).
- [ ] Manually review hostname and detailed disclosures with privacy-safe synthetic examples, and perform accessibility, Firefox/WebKit and production CSP checks before launch.

## Milestone 6.21: assessment quality and public-testing readiness

- [x] Audit the deterministic assessment-rule inventory and test false-reassurance / excessive-warning scenarios without changing unsupported claims into security verdicts.
- [x] Add 36 synthetic assessment quality cases (65 total assessment scenarios) while retaining 62 destination regression cases; separate synthetic consistency from unmeasured real-world phishing accuracy.
- [x] Improve hostname disclosure wording for ordinary users while retaining precise technical provenance under an expandable section.
- [x] Extend actual-WASM browser verification to Firefox; document WebKit's localhost navigation failure as an environment blocker, not a verified application result.
- [x] Add six automated axe accessibility states with no reported violations, plus manual accessibility and supervised synthetic-data usability testing plans.
- [x] Produce [Clue public-testing readiness](docs/CLUE_PUBLIC_TESTING_READINESS.md): suitable for local and supervised synthetic-data evaluation, not public launch.
- [ ] Complete manual screen-reader testing, 5–10 participant nontechnical usability study, WebKit validation in a working environment, deployed CSP/security-header validation and production privacy/security review.

## Development documentation

- [x] Document Clue's actual Chromium/WASM prototype, browser test boundary and remaining deployment gaps (Milestone 6.16).

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
- [x] Package the WASM adapter with wasm-bindgen and verify JavaScript/browser-runtime integration in local Chromium (Milestone 6.16); Firefox/WebKit and deployment validation remain open.

## Future: Clue companion web interface

- [x] Compile the reusable core, assessment crate and browser adapter for `wasm32-unknown-unknown`; execute the packaged adapter locally in Chromium.
- [x] Build a responsive static Clue prototype with actual WASM packaging and Chromium runtime analysis (Milestone 6.16).
- [x] Instrument Chromium browser-runtime tests to confirm no analysis-triggered network requests in the tested local context.
- [ ] Expand real browser tests to Firefox and WebKit, and verify local-first behaviour under a production hosting configuration.
- [x] Manually inspect Milestone 6.17 Clue results using the original motivating URL locally without navigating to submitted or candidate destinations; do not commit the sensitive original URL.
- [ ] Manually validate the refined Milestone 6.18 strong-warning UI against the motivating URL with optional context, without uploading or navigating to the URL.
- [ ] Complete manual screen-reader/accessibility testing, maximum-size input performance checks, and production CSP validation.
- [ ] Confirm remote Clue CI run on main.
- [x] Build an initial accessible-by-design interface for URL input, observations, completeness and privacy-reduced summary; manual assistive-technology review remains pending.
- [x] Analyze submitted URLs locally through real WASM in Chromium, with network-request assertions against analysis.
- [x] Show plain-language structured observations and completeness using the privacy-reduced browser response.
- [x] Add immediate local context-aware assessment with actionable warnings and typed next steps (Milestone 6.18).
- [ ] Design and security-review explicit local technical-evidence disclosure before implementation.
- [x] Offer a review-first privacy-reduced summary with an explicit copy action; counts/categories may reveal metadata.
- [ ] Publish explicit privacy and security documentation.

## Longer-term possibilities

- [ ] Browser extensions with manual right-click inspection.
- [ ] Email-client integrations, subject to each client's extension capabilities.
- [ ] Optional, privacy-conscious reputation intelligence with separately disclosed network activity.
- [ ] Explore explicitly opted-in, privacy-minimized reports of suspicious URLs with corroboration and human-reviewed hosting-provider abuse submissions; see [docs/FUTURE_INTELLIGENCE.md](docs/FUTURE_INTELLIGENCE.md).
- [ ] Design a separate, authorized active investigation service using disposable hardened isolation, strict outbound-network controls and evidence retention safeguards; never add active navigation to LinkSleuth or default Clue analysis.
- [ ] Developer-facing API or library integration examples.
- [ ] Sustainable business offerings that do not compromise the free open-source engine or user privacy.

## Non-goals for the initial engine

- No active navigation, link following, DNS resolution, or webpage fetching.
- No claim that an embedded URL proves a redirect or that a parsed URL is safe.
- No collection or sale of user browsing histories or submitted URLs.
- No requirement for user accounts, subscriptions, or remote services to perform basic analysis.

See [README.md](README.md) for usage and [CHANGELOG.md](CHANGELOG.md) for implemented changes.
