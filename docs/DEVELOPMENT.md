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
- Include ordinary, malformed, adversarial, edge-case, and privacy cases appropriate to the change. Maintain the synthetic destination corpus with explicit expected candidate presence, incomplete status and aggregate mismatch reporting; synthetic results do not measure real-world phishing performance. Test both successful and rejected CLI options.
- Preserve existing behavior and APIs unless the milestone explicitly permits a change; call out migration concerns.
- For privacy-related output, check full stdout/stderr, `Debug` representation, sensitive markers, and opt-in boundaries. For the versioned `consumer` API, test default disclosure, independent hostname/evidence opt-ins, stable observation IDs, stage-level completeness and evidence-reference privacy.
- When changing URL parsing or normalization, test exact submitted raw evidence, explicit examined-versus-unsupported path coverage, and the documented byte/parameter/pass boundaries. Avoid broad privacy guarantees.

## Integration verification

- For the independent consumer example, run `cargo build -p linksleuth-consumer-app` and `cargo test -p linksleuth-consumer-app`; it should only use public core consumer APIs.
- For the browser adapter, run `cargo test -p linksleuth-wasm`, `cargo check -p linksleuth-core --target wasm32-unknown-unknown`, `cargo check -p linksleuth-wasm --target wasm32-unknown-unknown`, and `cargo build -p linksleuth-wasm --target wasm32-unknown-unknown` when the target is installed. Target compilation must not be reported as browser-runtime validation.
- Future Clue integration must validate wasm-bindgen packaging, both schema versions, safe DOM rendering, fixed error messages and browser network activity before claiming local-browser behavior.
- See [CLUE_MVP.md](CLUE_MVP.md) and [CLUE_INTEGRATION_READINESS.md](CLUE_INTEGRATION_READINESS.md).

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
