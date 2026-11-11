# LinkSleuth Development Workflow

## Working model

We develop in reviewed, meaningful batches. Feature scope and acceptance criteria are defined before implementation; Codex can implement the batch locally; a human reviews requirements, security/privacy boundaries, and verification results before committing. Prefer self-contained, maintainable changes over one-function handoffs or broad rewrites.

## Before implementation

- Read [../AGENTS.md](../AGENTS.md), [ARCHITECTURE.md](ARCHITECTURE.md), and [SECURITY.md](SECURITY.md).
- Inspect existing code and tests. Identify public APIs, invariants, size/pass limits, and potential privacy impacts.
- Convert the **entire** task into explicit acceptance criteria. Examples are illustrative; they do not replace requirements.
- Use synthetic URLs. Do not use real credentials, private links, or customer data in tests or issue reports.
- Avoid unrequested dependencies and changes to parser/decoder behavior when implementing presentation-only features.

## Coding and testing

- Prefer typed Rust data structures, explicit states, bounded loops, and clear error handling. Keep raw evidence separate from fixed shareable messages.
- Add unit tests in the affected core module and executable CLI integration tests under `crates/linksleuth-cli/tests/` when changing user-visible behavior.
- Include ordinary, malformed, adversarial, edge-case, and privacy cases appropriate to the change. Use fully synthetic equivalents of real-world motivating URLs; never commit original links containing tokens, identifiable fragments, or private identifiers. Maintain the synthetic destination corpus with explicit expected candidate presence, incomplete status and aggregate mismatch reporting; synthetic results do not measure real-world phishing performance. Test both successful and rejected CLI options.
- Preserve existing behavior and APIs unless the milestone explicitly permits a change; call out migration concerns.
- For privacy-related output, check full stdout/stderr, `Debug` representation, sensitive markers, and opt-in boundaries. For Clue's separate assessment crate, test deterministic warning categories, URL-versus-context attribution, evidence-family grouping, recommendation wording, fixed next-step identifiers, incomplete-analysis safeguards and context clearing. For the versioned `consumer` API, test default disclosure, independent hostname/evidence opt-ins, stable observation IDs, stage-level completeness and evidence-reference privacy.
- When changing URL parsing or normalization, test exact submitted raw evidence, explicit examined-versus-unsupported path coverage, and the documented byte/parameter/pass boundaries. Avoid broad privacy guarantees.

## Integration verification

- For the independent consumer example, run `cargo build -p linksleuth-consumer-app` and `cargo test -p linksleuth-consumer-app`; it should only use public core consumer APIs.
- For assessment integration, run `cargo test -p linksleuth-assessment` and `cargo check -p linksleuth-assessment --target wasm32-unknown-unknown`. Maintain the separate 29-case synthetic assessment corpus alongside the 62-case destination corpus; neither measures real-world phishing accuracy.
- For the browser adapter, run `cargo test -p linksleuth-wasm`, `cargo check -p linksleuth-core --target wasm32-unknown-unknown`, `cargo check -p linksleuth-wasm --target wasm32-unknown-unknown`, and `cargo build -p linksleuth-wasm --target wasm32-unknown-unknown` when the target is installed. Target compilation must not be reported as browser-runtime validation.
- Clue now packages WASM for the browser and tests actual Chromium execution with Playwright request interception; this confirms only the tested local browser environment. Maintain schema validation, safe DOM rendering, fixed errors, and zero analysis-triggered network requests as regression invariants. Retain regression tests for bounded compound queries, hostname-first ambiguity, privacy-safe structural findings, truthful incomplete coverage, initial assessment and reassessment, optional user context, action-first warning copy, keyboard/mobile behaviour, and privacy-reduced summary opt-ins. Browser wire schema is 2, distinct from consumer schema 1 and assessment contract 1. Cross-browser, deployment CSP and manual accessibility checks remain future gates.
- From `apps/clue`, use `npm ci`, `npm run dev`, `npm run build`, and `npm test` with the matching Rust WASM target and wasm-bindgen CLI. `npm test` runs production build, contract/privacy checks and real-WASM Chromium tests. See [../apps/clue/README.md](../apps/clue/README.md) for pinned versions and first-time setup.
- See [CLUE_MVP.md](CLUE_MVP.md) and [CLUE_INTEGRATION_READINESS.md](CLUE_INTEGRATION_READINESS.md).

### Clue layout regression expectations

When changing Clue presentation, test initial and context-aware results using the real production WASM adapter. Verify the assessment/context desktop grid and stacked mobile reading order, native supporting-findings disclosure, important visible limitations, keyboard/focus/scroll continuity across context updates, summary privacy and zero analysis-triggered requests. Milestone 6.19 locally reported 23 actual-WASM Chromium tests (19 desktop and four mobile) across 390, 768, 1280 and 1920px visual inspection, alongside 176 Rust and five Node contract/privacy tests. Manual screen-reader testing and non-Chromium browser validation remain outstanding.

### Destination disclosure regression requirements

Maintain separate default-output and explicit-disclosure tests. Verify that normal `analyzeUrl` / assessment responses never contain candidate hostnames, full references, credentials or fragments; that `discloseDestinations(input, level)` exposes only its permitted fields; and that detailed evidence requires a distinct user confirmation. Test inert text rendering (no `href`/`src`/navigation), valid provenance, bounded output, fixed private errors, no leaks in the review/copy summary, removal of disclosed nodes on hide/new analysis and zero analysis/disclosure-triggered network requests in actual-WASM browser testing. Use only synthetic URLs with fabricated tokens and identifiers. Milestone 6.20 reported 182 passing Rust tests, 8 frontend contract/privacy tests and 29 actual-WASM Chromium tests; remote CI, Firefox/WebKit, deployed CSP and manual screen-reader status must be verified separately.

## Required checks

Run from workspace root:

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

If checks fail, fix failures or clearly report blockers. State the actual test count from the run, never an assumed count. GitHub Actions CI now runs formatting, workspace tests, and Clippy on pushes and pull requests to main. Confirm each remote run's actual status; a committed workflow is not proof of a passing run.

## Delivery and Git responsibilities

- Codex: implement, test, and report locally. **Do not commit, push, or edit project-level README/CHANGELOG/ROADMAP unless explicitly asked.**
- Review: compare delivered code and tests with *every* acceptance criterion. Check for disclosures and unintended core changes, not just green tests.
- Human developer: review and commit/push approved code.
- Documentation steward: update README, CHANGELOG, ROADMAP and architectural guidance after the feature is committed. Changes to those files are reviewed and pulled into the local checkout before new work starts.

Before a new implementation after remote documentation changes, run `git pull --ff-only` with a clean worktree.

## Handoff format

Give a compact report with changed files, implemented requirements versus partial/missing items, new/changed public APIs, test commands and results, noteworthy choices, compatibility impacts, and remaining privacy/security risks. Do not claim full completion without checking against the request.
