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
- Preserve exact submitted raw query parameter names and values alongside form-decoded equivalents, even where URL parsing normalizes Unicode or other characters.
- Retain repeated query parameter names in order, including empty values and flag-style parameters.
- Record whether each raw query parameter contained a literal equals sign (`has_equals`), distinguishing `flag` from `empty=`.
- Preserve distinct encoded forms such as `+`, `%20`, `%2B`, `%26`, and `%3D` for evidence review.
- Decode percent encoding in up to three passes, retaining each transformation.
- Preserve malformed percent sequences as warnings while decoding valid sequences.
- Record the decoding-attempt number and byte offset for each warning.
- Report whether further percent decoding may remain.
- Discover complete HTTP/HTTPS and protocol-relative references in supported query values, plus cautiously classified hostname-first references in a narrow destination-parameter allowlist. Bounded compound query interpretation can recover complete nested parameter values after supported percent/HTML decoding; ambiguous authority/path boundaries remain explicitly unresolved. Base64 and HTML character-reference discovery retains ordered transformations and source provenance. No arbitrary substring scanning or redirect verification.
- Produce preliminary structured findings for encoded parent-directory segments in original URL paths, showing original and normalized paths with an evidence-based explanation.
- Include normalization findings alongside decoded parameters and possible embedded destinations in the reusable `AnalysisReport`.
- Recognize forward and backslash separators in supported original-path evidence, while excluding query strings and fragments from these findings.
- Explicitly distinguish examined original-path evidence from unsupported or ambiguous extraction using typed coverage metadata, avoiding an unsupported-is-clean interpretation.
- Inspect URLs using a command-line tool that displays normalization findings alongside decoding results and possible embedded destinations.
- Report concrete structural evidence such as encoded query delimiters, compound query values, multiple percent transformations and ambiguous hostname-first references, even when a candidate cannot be confidently classified. Consumer and browser output remain privacy-reduced, with explicit partial-coverage reasons.
- Provide immediate, deterministic Clue link assessments through a separate `linksleuth-assessment` Rust crate, consuming the public privacy-reduced core consumer API. Clue presents qualified warning categories, clear action-first recommendations, and at most three fixed practical next steps without claiming a verified phishing verdict.
- Ask optional progressive questions *after* the initial analysis (source, claimed purpose, expected contact and requested action); reassess locally while preserving URL-derived observations and distinguishing reported circumstances from observed evidence. Context is never required for a preliminary result.
- Present a responsive Clue results workspace: side-by-side assessment (approximately 55%) and optional context questions (45%) on desktop, stacked in logical reading order on small screens. Supporting structural findings are collapsed by default while important limitations remain visible; users can expand details without changing the analysis.
- Provide a working, static Clue browser prototype under `apps/clue` using Vite, vanilla JavaScript and the actual packaged WASM adapter. Chromium runtime tests execute real local analysis and instrument requests; none were observed during analysis after initial app loading (local tests, not a deployment guarantee).
- Provide an independent Rust consumer example (`examples/consumer-app`) and a separate WebAssembly adapter (`crates/linksleuth-wasm`), with browser wire schema version 2 and independent assessment contract version 1. The core and adapter compiled for `wasm32-unknown-unknown`; browser runtime loading and JavaScript integration have since been verified locally in Chromium.
- Introduce a versioned, Rust-only `consumer` API (schema version 1) intended for Clue and third-party integrations. `create_consumer_report()` returns deterministic, fixed-text observations and four-stage analysis completeness, with no hostname or detailed evidence by default; hostname and evidence are independent explicit opt-ins.
- Generate a structured privacy-reduced `ShareableReport` in the core library: hostname redacted by default, aggregate counts including nested/protocol-relative and Base64/HTML-reference-derived candidates, typed discovery and encoding-coverage status, and fixed-text observations without URL-derived strings. An options-aware API allows explicit hostname inclusion.
- Use `--shareable` in the CLI to print a privacy-reduced report with readable findings, or `--shareable --include-hostname` to deliberately reveal the hostname. Tests cover privacy-sensitive output, error handling, opt-in disclosure, and detailed-mode compatibility.
- Bound input to 16 KiB, non-empty top-level query parameters to 256, decoder input to 16 KiB, and percent decoding to three passes. Secondary decoding is limited to 16 KiB input, 8 KiB output, and 32-byte HTML reference tokens; at most one Base64 or HTML stage follows percent decoding per examined value. Nested discovery stops at depth 3, 64 candidate occurrences, or 32 parameters per nested URL, and reports incomplete coverage when appropriate.
- Run automated tests covering parsing, decoding, Unicode, normalization coverage, resource limits, privacy-reduced reporting, and actual CLI output, including a machine-readable, 62-case synthetic destination regression corpus in `crates/linksleuth-core/tests/corpus/` with aggregate positive/negative expectation checks; GitHub Actions runs formatting, workspace tests, and Clippy on pushes and pull requests to main.

**Limitations:** Normalization findings currently recognize encoded parent-directory patterns only, and some unusual original-path spellings remain unsupported. Nested destination discovery examines complete candidate references in supported query values, including one bounded secondary Base64 or HTML-reference decoding stage. Hostname-first interpretation is ASCII-only, context-restricted, and rejects ambiguous constructs; compound query parsing uses one deterministic interpretation with at most 16 inner parameters. Fragments are not searched for destinations. It does not extract arbitrary substrings, alternate secondary decoders, support every HTML entity, or verify any redirect. Privacy-reduced reporting is not a fully audited or anonymous export; counts and finding categories may reveal metadata. Embedded URLs are *possible destinations*, not verified redirects. The current engine does not resolve redirects, inspect webpage content, query reputation feeds, or determine whether a link is safe. The CLI currently displays raw URL information, which may include personal information or tokens; review output carefully before sharing it.

### Clue destination evidence and unified layout (Milestone 6.20)

Clue now aligns header navigation, hero, analyzer, results and footer to one responsive 88rem outer container. The desktop assessment and optional context questions remain side by side, with concise primary guidance and a full-width explanation of reasons beneath. Supporting observations stay expandable.

**Potential destination evidence is opt-in.** The normal browser result stays privacy-reduced. Users must separately request candidate hostnames, then explicitly confirm before revealing detailed provenance and decoded candidate references. Neither disclosure navigates to or verifies a destination; evidence is inert text, not a link. Credentials and literal fragments are omitted from disclosed candidate representations, but **detailed query strings can still include sensitive tokens or identifiers**. Hiding a panel removes rendered nodes, not every copy in browser memory. Revealing evidence does not add it to the privacy-reduced copy/share summary. See [destination disclosure and threat model](docs/CLUE_DESTINATION_DISCLOSURE.md).

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

The default CLI mode reports URL components, decoding steps, percent-decoding completion or pass-limit conditions, malformed-sequence observations, direct and nested potential destinations with provenance, and supported normalization findings with original/normalized path evidence. It also distinguishes unsupported original-path analysis from an examined path with no finding. For privacy-reduced output, run `cargo run -p linksleuth-cli -- --shareable 'https://example.com/?token=example'`; this prints a redacted hostname, aggregate counts, and fixed-text observations instead of original URL evidence. Add `--include-hostname` after `--shareable` only when you deliberately want to disclose the hostname. No suspicious destination is contacted.

## Project structure

```text
crates/
  linksleuth-core/  Reusable Rust analysis library
  linksleuth-cli/   Command-line interface
```

The core library is independent of its CLI consumer. See [Clue MVP specification](docs/CLUE_MVP.md) and [integration readiness assessment](docs/CLUE_INTEGRATION_READINESS.md) for the proposed local-first browser workflow and verified WASM build boundary. Applications should prefer the additive, schema-versioned `linksleuth_core::consumer` API over coupling to internal analysis structures. The Rust consumer schema version 1, Clue assessment contract version 1, and browser adapter JSON wire schema version 2 are independently versioned and separate from the crate's development version. Clue runs its packaged WASM adapter in Chromium and leads with an actionable, context-aware assessment; users can optionally refine the recommendation with local context questions before reviewing plain-language observations, expandable coverage, and privacy-reduced summary review/copy. Firefox, WebKit, and production deployment remain unverified. Planned consumers include **Clue**, a friendly web interface, and browser/email integrations.

The Clue results view uses a bounded wider desktop layout for the assessment and optional questions. The first two context prompts are immediately available, with two additional prompts revealed on demand. URL-derived supporting observations remain available in a native expandable section; the assessment is actionable without expanding it. Local Playwright tests cover desktop/mobile responsiveness and browser network behaviour. The interface has not received manual screen-reader or production-hosting validation.

## Try Clue in your browser

```bash
cd apps/clue
npm ci
npm run dev
```

Open the local address shown by Vite (normally `http://127.0.0.1:5173/`). The dev command builds and packages the actual Rust WASM adapter; initial setup requires the Rust `wasm32-unknown-unknown` target and matching `wasm-bindgen-cli 0.2.130`. See [apps/clue/README.md](apps/clue/README.md) for setup, production preview and browser test commands. Submitted URL analysis is local and passive; the app loads its own assets, but browser tests observed no requests initiated by analysis.

## Try the independent consumer and WASM build

```bash
cargo run -p linksleuth-consumer-app
cargo test -p linksleuth-consumer-app
rustup target add wasm32-unknown-unknown
cargo check -p linksleuth-core --target wasm32-unknown-unknown
cargo build -p linksleuth-wasm --target wasm32-unknown-unknown
```

The example uses privacy-reduced observations by default. WASM compilation alone is not browser validation; Clue now additionally has browser packaging and Chromium runtime tests, though production deployment and additional browser engines remain unverified.

## Roadmap and changes

Read [ROADMAP.md](ROADMAP.md) for priorities and [CHANGELOG.md](CHANGELOG.md) for completed changes. Contributor and agent guidance lives in [AGENTS.md](AGENTS.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/SECURITY.md](docs/SECURITY.md), and [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Security and privacy notes

A submitted URL may contain authentication tokens, email addresses, or private document identifiers. Avoid posting sensitive URLs publicly, and do not share unsanitized CLI output. `ShareableReport` omits original URL evidence and emits only redacted hostname by default, aggregate counts, and fixed finding categories; opt-in hostname disclosure is available through reporting options or the CLI's `--shareable --include-hostname`. Reports are not guaranteed anonymous: even counts and finding kinds can be sensitive. Command-line URLs can also remain in shell history or process listings. The consumer API also defaults to privacy-reduced observations; explicitly requested evidence may contain sensitive URL data and must be handled accordingly. Clue's local assessment is not a verified phishing or safety verdict; optional context is user-reported and unverified. Copied summaries exclude context-derived reasons by default, although even standardized next steps may reveal context categories, and therefore remain privacy-reduced rather than anonymous. Review `docs/SECURITY.md` for limitations.

Passive URL inspection can reveal syntax and embedded destinations, but **cannot guarantee a link is safe**. Continue using normal browser and email security protections.

## License and ownership

Copyright 2026 Sweetwater Media Inc.

Licensed under the [Apache License, Version 2.0](LICENSE).
