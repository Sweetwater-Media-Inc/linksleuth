# LinkSleuth Security and Privacy Design

This document describes current security invariants and a preliminary threat model. It is **not** a claim of formal audit, a vulnerability-disclosure contact page, or a guarantee that a link or report is safe.

## Trust boundaries and threats

The input URL is untrusted and may contain credentials, private hostnames, user identifiers, email addresses, document paths, fragments, API tokens, or attacker-controlled encodings. Analysis might reveal these in internal structures or detailed CLI output. Additional risks include misleading normalization, encoding layers, unexpectedly long inputs, false positives, privacy leakage through diagnostics, and accidental disclosure when copying results.

## Required invariants

1. **Passive operation:** The analysis engine does not perform DNS, HTTP, redirects, remote content loading, or code execution. A discovered URL is a **candidate**, not an observed redirect. No implicit network integrations.
2. **Evidence integrity:** Retain the submitted representation and distinguish it from parser normalization and decoding outputs. Record supported transformation steps, statuses, and malformed-percent observations; do not discard malformed input just to make it look valid.
3. **Bounded work:** Keep input-length and decoding-pass bounds, and add explicit bounds for any new extraction or decoder. Account for CPU, memory, and output amplification.
4. **Privacy by default:** Treat every URL-derived string as potentially sensitive, including a hostname. Default shareable reports redact hostnames, omit original/path/query/fragment/decoded/embedded-URL strings, and use fixed finding titles/explanations and aggregate counts. An opt-in hostname is an intentional disclosure and should be clearly identified as such.
5. **Defensive outputs:** Review `Debug`, `Display`, CLI stdout/stderr, error variants, logs, and future serialization for accidental evidence leakage. Do not interpolate untrusted URL values into shareable explanations. A detailed local report is **not** sanitized.
6. **Calibrated claims:** Do not label a link safe/malicious from parsing alone, or claim that malformed encoding, a normalized path, or a candidate destination proves hostile intent.

## Current reporting behaviour

- `create_shareable_report()` uses default hostname redaction; the options-aware API can intentionally include the hostname.
- `ShareableReport` includes aggregate counts and four typed, fixed-text categories. This is **privacy-reduced**, **not anonymous**: counts and finding categories can still expose characteristics of a submitted URL.
- The `ShareableReport` `Debug` implementation masks its hostname even after explicit disclosure was enabled. Other report fields must remain safe by construction.
- The CLI's `--shareable` mode is distinct from its detailed mode. `--shareable --include-hostname` reveals the hostname; it does not authorize raw URL, path, or token inclusion.
- Command-line arguments may appear in shell history, process listings, terminals, or surrounding tooling even if CLI **output** is redacted. Do not test with genuine secrets.

## Engineering and testing expectations

- Use synthetic adversarial URLs with distinctive fake secrets for regression tests.
- Test both shareable stdout and stderr, failures, malformed input, opt-in disclosure, and `Debug` output; check whole rendered output when useful.
- Re-evaluate privacy whenever adding fields, serialization, UI integrations, logging, network access, or error details.
- Distinguish an unsupported analysis case from an affirmative negative finding where the API can express that distinction.
- Do not treat passing tests as a security proof. Escalate ambiguous behavior or potential disclosure rather than silently weakening the invariant.

See [ARCHITECTURE.md](ARCHITECTURE.md) for module responsibilities and [DEVELOPMENT.md](DEVELOPMENT.md) for the testing workflow.
