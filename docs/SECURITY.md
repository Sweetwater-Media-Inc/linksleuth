# LinkSleuth Security and Privacy Design

This document describes current security invariants and a preliminary threat model. It is **not** a claim of formal audit, a vulnerability-disclosure contact page, or a guarantee that a link or report is safe.

## Trust boundaries and threats

The input URL is untrusted and may contain credentials, private hostnames, user identifiers, email addresses, document paths, fragments, API tokens, or attacker-controlled encodings. Analysis might reveal these in internal structures or detailed CLI output. Additional risks include misleading normalization, encoding layers, unexpectedly long inputs, false positives, privacy leakage through diagnostics, and accidental disclosure when copying results.

## Required invariants

1. **Passive operation:** The analysis engine does not perform DNS, HTTP, redirects, remote content loading, or code execution. A discovered URL is a **candidate**, not an observed redirect. No implicit network integrations.
2. **Evidence integrity:** Retain the submitted representation, including raw query text extracted from exact input, and distinguish it from parser normalization, form decoding, and iterative percent-decoding outputs. Record supported transformation steps, statuses, and malformed-percent observations; do not discard malformed input just to make it look valid.
3. **Bounded work:** Enforce 16 KiB submitted URL bytes, at most 256 non-empty top-level query parameters, at most 16 KiB percent-decoder input bytes, and at most three percent-decoding passes. For nested discovery, enforce maximum depth 3 (direct candidates at depth 0), 64 candidate occurrences, and 32 parameters examined per nested URL. Secondary Base64/HTML decoding is limited to 16 KiB input, 8 KiB output, and 32-byte HTML-reference tokens, with no repeated secondary-stage alternation; surface incomplete status when limits prevent full discovery. Counts are byte-based where specified, not Unicode character-based. Add explicit bounds to new extraction or decoding paths; account for CPU, memory, warning counts, and output amplification.
4. **Privacy by default:** Treat every URL-derived string as potentially sensitive, including a hostname. Default shareable reports redact hostnames, omit original/path/query/fragment/decoded/embedded-URL strings, and use fixed finding titles/explanations and aggregate counts. An opt-in hostname is an intentional disclosure and should be clearly identified as such.
5. **Defensive outputs:** Review `Debug`, `Display`, CLI stdout/stderr, error variants, logs, and future serialization for accidental evidence leakage. Do not interpolate untrusted URL values into shareable explanations. A detailed local report is **not** sanitized.
6. **Calibrated claims:** Do not label a link safe/malicious from parsing alone, or claim that malformed encoding, a normalized path, or an absolute/protocol-relative/nested candidate proves hostile intent or a verified redirect chain. Always distinguish an examined original path from unsupported extraction; an empty finding list alone is not a negative security result.

## Current reporting behaviour

- The `linksleuth-wasm` adapter provides a deliberately narrower browser JSON contract than the Rust consumer API: it uses default privacy-reduced options, omits hostname and detailed evidence fields, and emits fixed input/analysis error messages without reflecting submitted URL strings. Wire schema version 1 is independent of the Rust consumer schema version.
- The intended Clue page may load its own static assets, but analysis must not send submitted URLs, resolve analyzed hosts, fetch nested candidates, preload destinations, or log URL-bearing telemetry. Browser network instrumentation has **not** yet verified this invariant; current evidence is source/dependency review and target compilation.

- The new `consumer::create_consumer_report()` is privacy-reduced by default. Hostname inclusion and detailed evidence disclosure are **independent explicit opt-ins**. Detailed evidence can contain submitted, normalized, decoded or nested URL strings and must be treated as sensitive. Stable evidence references alone do not constitute permission to disclose evidence values.
- Consumer observations use fixed titles/explanations and stable identifiers; they are structural observations, not malicious/safe classifications. `Debug` output redacts opted-in hostname and evidence. The consumer report is not guaranteed anonymous or inherently safe to publish.
- Consumer completeness covers original-path evidence, percent decoding, destination traversal and secondary encoding independently. “Complete within supported scope” is never a guarantee of exhaustive detection or safety.

- `create_shareable_report()` uses default hostname redaction; the options-aware API can intentionally include the hostname.
- `ShareableReport` includes typed original-path coverage, aggregate counts for total/nested/protocol-relative/Base64/HTML-derived candidates, discovery and encoding-coverage status, and fixed-text finding categories. The supported types may evolve; never interpolate raw evidence into shareable text. This is **privacy-reduced**, **not anonymous**: counts and finding categories can still expose characteristics of a submitted URL.
- The `ShareableReport` `Debug` implementation masks its hostname even after explicit disclosure was enabled. Other report fields must remain safe by construction.
- Nested discovery preserves raw submitted references, source parameter names, decoding history, ordered Base64/HTML transformation provenance, and parent relationships in the **detailed** analysis only. Duplicate occurrences are retained, but descendants of duplicate normalized references attach to the first traversed occurrence; no traversal confirms a redirect.
- The CLI's `--shareable` mode is distinct from its detailed mode. `--shareable --include-hostname` reveals the hostname; it does not authorize raw URL, path, or token inclusion.
- Command-line arguments may appear in shell history, process listings, terminals, or surrounding tooling even if CLI **output** is redacted. A safer input mechanism remains future work. Do not test with genuine secrets.

## Engineering and testing expectations

- Use synthetic adversarial URLs with distinctive fake secrets for regression tests.
- Test both shareable stdout and stderr, failures, malformed input, opt-in disclosure, and `Debug` output; check whole rendered output when useful.
- Treat successful secondary decoding as a transformation, **not** evidence of maliciousness. A result is considered a candidate only if the entire decoded value passes conservative URL classification. Strict Base64 validation, limited named HTML references, malformed/unsupported encoding coverage, and separate resource-limit status reduce false claims; they do not guarantee absence of false positives.
- Render browser wire strings as text data, never unsanitized `innerHTML`; never place submitted URLs in the page address, browser telemetry, asset paths or automatic network requests. Explicit local evidence disclosure remains a future separate review.
- Re-evaluate privacy whenever adding fields, serialization, UI integrations, logging, network access, nested discovery mechanisms, or error details. Incomplete-analysis reasons must use fixed text, not URL-derived strings.
- Distinguish an unsupported analysis case from an affirmative negative finding where the API can express that distinction.
- The 42-case synthetic regression corpus currently has zero positive/negative expectation mismatches. These are corpus-specific assertions, not real-world false-positive measurements.
- Do not treat passing tests as a security proof. Escalate ambiguous behavior or potential disclosure rather than silently weakening the invariant.

See [ARCHITECTURE.md](ARCHITECTURE.md) for module responsibilities and [DEVELOPMENT.md](DEVELOPMENT.md) for the testing workflow.
