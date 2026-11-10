# Clue browser prototype

Clue is LinkSleuth's first working browser interface. It loads the real
`linksleuth-wasm` adapter, analyzes an entered HTTP/HTTPS URL locally, and
renders the adapter's privacy-reduced observations and stage-specific
completeness.

Clue does not fetch the entered URL, load discovered destinations, use a remote
analysis service, or decide whether a URL is safe or malicious.

Milestone 6.17 adds plain-language structural observations for bounded
hostname-first and compound-query forms. The primary result view omits internal
identifiers and schema badges, uses full-width observation rows, and keeps the
four technical coverage stages in an accessible **What Clue examined**
disclosure. Encountered limitations remain visible without expanding it.

Milestone 6.18 adds an immediate deterministic assessment and actionable
recommendation before observations. A short optional progressive context flow
can update that guidance locally. Answers are finite categories, remain in page
memory, are never persisted or transmitted, and can be revised or cleared.
The final refinement makes the recommended action the primary headline,
reserves **Do not open this link** for strong-warning outcomes, and provides a
bounded list of fixed practical next steps. The category and evidence limits
remain visible, and URL-derived versus user-reported reasons stay separate.

Milestone 6.19 reorganizes results for faster comparison. At desktop widths,
the assessment and optional context form use an approximately 55/45 two-column
layout inside a wider, bounded results region. The first two context questions
are immediately visible; one explicit control reveals the remaining two.
Tablet and mobile widths stack the assessment first and context second.
Supporting structural findings move into a collapsed native disclosure with a
fixed observation count, while assessment limitations remain visible.

## Tool versions

The checked-in lockfiles and build script currently target:

- Rust/Cargo with the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli 0.2.130`, matching the Rust crate exactly
- Node.js 22.12 or newer; locally verified with Node.js 24.13.0
- npm 11.6.2
- Vite 8.3.7
- Playwright 1.65.0
- Playwright Chromium 158.0.8127.18, build 1254

## First-time setup

From the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
cd apps/clue
npm install
npx playwright install chromium
```

Use `npm ci` instead of `npm install` when reproducing from
`package-lock.json`. On a minimal Linux CI image, Playwright may require:

```bash
npx playwright install --with-deps chromium
```

## Run locally

```bash
cd apps/clue
npm run dev
```

Open the printed address, normally:

```text
http://127.0.0.1:5173/
```

`npm run dev` first builds the Rust adapter in release mode and runs
`wasm-bindgen` before starting Vite. No manual artifact copying is required.

## Production build and preview

```bash
cd apps/clue
npm run build
npm run preview
```

The preview address is:

```text
http://127.0.0.1:4173/
```

The production build writes `dist/`. Rust-generated browser bindings are
written to `src/generated/`. Both directories, along with `node_modules/` and
Playwright output, are ignored and must be rebuilt from source.

A Rust WASM target build alone is not browser packaging. The `build:wasm`
script performs both steps:

1. `cargo build -p linksleuth-wasm --release --target wasm32-unknown-unknown`
2. `wasm-bindgen --target web` to produce browser-loadable JavaScript and WASM

## Tests

Run the complete frontend verification:

```bash
cd apps/clue
npm test
```

This builds production assets, runs Node contract/privacy tests, starts the
production preview server, and runs Playwright against the real generated WASM
in Chromium. Browser tests instrument requests and fail if analysis starts any
request, including requests to synthetic submitted or candidate hostnames.
Responsive assertions cover 390px mobile, 768px tablet, 1280px desktop, and
1920px large-desktop layouts.

Individual commands are available when iterating:

```bash
npm run build:wasm
npm run build
npm run test:contract
npm run test:browser
```

## Browser contract

Clue accepts only browser wire schema version 2, consumer report schema version
1, and assessment contract version 1. Runtime validation checks success/error envelope shape,
privacy-reduced disclosure, observations, occurrence counts, all four
completeness stages, fixed reasons, and the absence of evidence references.
The validator also requires the fixed action explanation and one to three
typed practical next steps.

An unknown version or malformed response produces a fixed compatibility error.
The raw response is never rendered as a fallback.

Wire schema 2 adds the required privacy-reduced assessment object. Consumer
schema 1 remains unchanged. The final action-explanation and next-step fields
are additive, so wire schema 2 and assessment contract 1 remain unchanged.
Version information remains available under **About this result**.

## Privacy boundary

- The entered URL remains in the input element and in memory for the current
  page only.
- It is not placed in Clue's page URL, storage, cookies, telemetry, logs, or
  analysis-triggered requests.
- Results contain no original hostname, destination hostname, path, parameters,
  decoded payload, or detailed evidence.
- Summary text is derived only from the validated privacy-reduced response and
  is copied only after separate Review and Copy actions.
- Context-derived reasons are omitted from summaries unless the user explicitly
  enables their inclusion during review.
- Context uses bounded structured choices only and is not persisted, logged, or
  transmitted.
- Report categories and counts remain metadata; the summary is not guaranteed
  anonymous.
- Detailed evidence disclosure is intentionally unavailable.

The page includes a tested meta Content Security Policy suitable for the local
static build. Production hosting should also send a reviewed CSP as an HTTP
header and re-run browser tests against the deployed configuration; this
prototype does not claim production hardening.

## Known limitations

- Automated runtime coverage currently uses Chromium only.
- There is no detailed-evidence interface, hostname disclosure, persistence, or
  analysis history.
- Clipboard behavior depends on browser permission; a selected textarea is the
  fallback.
- Browser tests verify local static assets and analysis network behavior, not a
  public hosting environment.
- Accessibility uses semantic HTML and automated interaction coverage, but a
  manual assistive-technology audit is still required before public release.
