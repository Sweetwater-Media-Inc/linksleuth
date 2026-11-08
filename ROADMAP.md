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
- [ ] Harden original-path extraction against unusual authority and delimiter syntax, then integrate normalization findings with the main analysis report and CLI.
- [ ] Add additional Unicode and unusual query-syntax edge cases.
- [x] Verify distinct raw and form-decoded values for `+`, `%20`, `%2B`, `%26`, and `%3D`.
- [ ] Document percent decoding versus form-style `+` decoding throughout the analysis API.
- [ ] Extend evidence provenance from exact raw input through all transformations (pass-aware warnings now implemented).
- [ ] Refine decoding status and limitations to avoid overstating completeness.
- [ ] Apply strict bounds to all relevant analysis stages.
- [ ] Design privacy-safe redaction for reports and diagnostic output.
- [ ] Add CI checks for formatting, tests, and Clippy.

## Next: Expanded discovery and explainability

- [ ] Recognize protocol-relative embedded URLs such as `//example.com/path`.
- [ ] Explore URLs nested in additional parameter structures or text, with explicit source provenance.
- [ ] Add carefully bounded Base64 and HTML-entity analysis.
- [ ] Distinguish observed decoded text from candidate destinations and verified redirects.
- [ ] Create structured findings with evidence, limitations, and plain-language explanations.
- [ ] Add a regression corpus of synthetic suspicious and legitimate URLs.
- [ ] Introduce fuzz/property-based tests and evaluate false positives.

## Future: Clue companion web interface

- [ ] Compile the reusable analysis engine for WebAssembly where practical.
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
