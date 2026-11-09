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
- Include ordinary, malformed, adversarial, edge-case, and privacy cases appropriate to the change. Test both successful and rejected CLI options.
- Preserve existing behavior and APIs unless the milestone explicitly permits a change; call out migration concerns.
- For privacy-related output, check full stdout/stderr, `Debug` representation, sensitive markers, and opt-in boundaries.
- When changing URL parsing or normalization, test exact submitted raw evidence, explicit examined-versus-unsupported path coverage, and the documented byte/parameter/pass boundaries. Avoid broad privacy guarantees.

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
