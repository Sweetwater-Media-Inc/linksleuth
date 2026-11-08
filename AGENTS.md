# Instructions for Coding Agents

These instructions apply to work throughout the LinkSleuth repository. Read the relevant sections of [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/SECURITY.md](docs/SECURITY.md), and [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) before changing code. The README and ROADMAP describe public capabilities and priorities, not permanent implementation mandates.

## Non-negotiable boundaries

- LinkSleuth's current analysis is strictly passive. Do not fetch URLs, follow redirects, resolve DNS, make network requests during analysis, or execute content from an analyzed URL. Do not add reputation lookups as a side effect.
- Treat all URLs, hostnames, paths, credentials, query names/values, fragments, and decoded text as potentially sensitive and untrusted.
- Preserve original evidence and normalization/decoding provenance in the analysis layer. Never silently substitute normalized interpretations for submitted input.
- Keep privacy-reduced reporting separate from raw analysis evidence. Shareable output must not accidentally include raw source data through fields, formatting, Debug, errors, or CLI output. Hostname inclusion must be an explicit opt-in.
- A candidate embedded HTTP/HTTPS URL is **not** a verified redirect. Normalization and malformed encoding are observations, not proof of malicious intent. Do not issue “safe” or “malicious” verdicts without a separately defined, evidence-backed capability.
- Preserve existing size and decoding-pass bounds; consider limits before expanding parsing or extraction.
- Keep `linksleuth-core` reusable and independent from CLI presentation and the planned Clue frontend.

## Implementation practice

- Inspect affected code and existing tests first. Prefer focused additions to rewrites, unnecessary dependencies, or speculative abstractions.
- Implement the **full requested acceptance criteria**, not just an illustrative output example. Explicitly audit each requirement before declaring completion.
- Add or update tests for normal, malformed, adversarial, and privacy-sensitive cases as appropriate. Use synthetic URLs and dummy secrets; never use live credentials.
- Preserve public API compatibility unless an intentional breaking change is agreed upon.
- Review error paths and diagnostic output for sensitive-value leakage.
- Run `cargo fmt --all --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check` before handoff.

## Handoff and Git

- Unless explicitly directed, **do not commit, push, create branches, or edit README.md, CHANGELOG.md, or ROADMAP.md**. Documentation changes requested as part of a task are an exception.
- Report changed files, new APIs, tests and commands run, requirements completed/partial/missing, tradeoffs, and unresolved privacy/security concerns.
- Never describe unrun checks as passing or claim that an implementation is secure solely because tests pass.
- If a requirement conflicts with the passive/privacy invariants, stop that portion and surface the conflict for review.

These are durable project rules. Milestone-specific acceptance criteria belong in the task prompt, not in this file.
