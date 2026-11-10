# Clue integration readiness assessment

## Scope

This assessment records Milestone 6.15 and 6.16 evidence for using LinkSleuth
from an independent Rust package and the first Clue browser prototype. It does
not claim that the application is production-hardened or that the consumer API
is permanently frozen.

## Independent Rust integration

`examples/consumer-app` is a separate workspace package with a normal Cargo
dependency on `linksleuth-core`. It calls only public APIs:

- `analysis::analyze_url()`
- `consumer::create_consumer_report()`
- `ConsumerReportOptions`
- Public consumer report accessors

It does not inspect decoder internals, `DestinationDiscovery` fields, private
consumer fields, or report construction helpers. Its default output includes
schema version, fixed observations, occurrence counts, explanations, and all
completeness stages. It does not print raw evidence or the hostname.

Reproduce it with:

```bash
cargo run -p linksleuth-consumer-app
cargo test -p linksleuth-consumer-app
```

Supplying a URL as a command-line argument is convenient for a synthetic
example but can expose it through shell history or process listings. This is
not the intended Clue input mechanism.

## WebAssembly compatibility evidence

### Confirmed compatible by compilation

- `linksleuth-core` compiles for `wasm32-unknown-unknown` with the installed
  Rust toolchain.
- The direct runtime dependency is `url`; its resolved form-urlencoded, IDNA,
  ICU, and percent-encoding dependency graph also compiled for the target.
- Core source uses owned strings, vectors, bounded parsing/decoding, and pure
  computation. It has no filesystem, system-clock, thread, socket, DNS, HTTP,
  or platform-randomness imports.
- `linksleuth-wasm` compiles as an `rlib` and `cdylib` for the same target.
- Browser-specific dependencies are isolated in `linksleuth-wasm`; the core
  manifest remains platform-neutral.

Verification commands:

```bash
rustup target list --installed
cargo check -p linksleuth-core --target wasm32-unknown-unknown
cargo check -p linksleuth-wasm --target wasm32-unknown-unknown
```

### Confirmed outside the browser target boundary

- `linksleuth-cli` depends on process arguments and terminal standard output;
  it is not the browser interface.
- `linksleuth-consumer-app` is a native demonstration package and is not part
  of the browser bundle.

These are architectural separations, not defects in the reusable core.

### Verified in the Clue prototype

- `wasm-bindgen 0.2.130` packages the release adapter for `web` output.
- Vite imports and initializes the generated bindings and WASM binary.
- Chromium executes `analyzeUrl(input)` through the actual adapter.
- Runtime validation accepts browser wire schema 1 and consumer schema 1 and
  rejects unknown or incomplete contracts.
- Playwright exercises observations, no-observation output, incomplete stages,
  fixed errors, keyboard interaction, summary review/copy, and mobile layout.
- Request instrumentation confirms that analysis produces no request after the
  application's own local assets have loaded and does not contact synthetic
  submitted or candidate hostnames.

### Not yet verified

- Runtime behavior across supported browsers
- Browser memory and performance characteristics for maximum-size inputs
- Public-host Content Security Policy and deployment configuration
- Manual assistive-technology behavior
- The future explicit technical-evidence disclosure flow

Successful target compilation demonstrates portability but remains distinct
from browser execution. Clue's Chromium suite now supplies browser-runtime
evidence; other browsers and public deployment remain unverified.

## Browser adapter decision

Milestone 6.15 introduces `crates/linksleuth-wasm` because both the core and its
dependency graph compiled successfully for `wasm32-unknown-unknown`, and a
minimal boundary could be implemented without changing the engine.

The adapter:

- Accepts a URL string.
- Calls the existing `analyze_url()` and `create_consumer_report()` functions.
- Always uses default `ConsumerReportOptions`.
- Maps the report to a small browser model rather than serializing
  `AnalysisReport` or arbitrary consumer internals.
- Returns a JSON envelope with browser wire schema version 1 and the independent
  consumer schema version.
- Includes fixed observations, occurrence counts, empty evidence-reference
  lists, completeness stages, and fixed reasons.
- Omits hostname and evidence fields entirely.
- Maps invalid input to fixed error codes and messages.
- Performs no network operations.

On the WebAssembly target, `wasm-bindgen` exposes `analyzeUrl(input)`, which
returns the JSON envelope as a string. JSON is used as a narrow explicit wire
schema; frontend code must parse it as data and render via `textContent` or an
equivalent safe framework binding.

The browser wire schema version is separate from consumer schema version 1.
Wire-incompatible changes require advancing `BROWSER_WIRE_SCHEMA_VERSION`.
Consumer-contract changes continue to follow the core schema version.

## Dependency assessment

- `linksleuth-core`: unchanged; still depends only on `url` directly.
- `linksleuth-wasm`: adds `serde` for the adapter's owned wire structures,
  `serde_json` for explicit JSON output, and target-specific `wasm-bindgen` for
  the exported WebAssembly function.
- No dependency adds HTTP, DNS, filesystem, telemetry, clocks, threads, or
  randomness to analysis.

These dependencies are confined to the adapter. A future frontend packaging
milestone may add tooling around `wasm-bindgen`, but it should not move browser
dependencies into `linksleuth-core`.

## Local-first privacy boundary

The intended frontend loads its application assets, then performs analysis in
the browser. Analysis must not:

- Send the submitted URL to a server.
- Place it in the page URL, analytics, logs, or telemetry.
- Resolve or fetch the submitted hostname.
- Load, preview, preconnect to, or navigate to candidate destinations.
- Interpolate URL-derived evidence into HTML.
- Reveal hostname or detailed evidence by default.

The initial adapter is narrower than the full consumer API: it provides no
hostname or evidence opt-in. Adding those capabilities requires a separately
reviewed local disclosure design and explicit UI controls.

## Test boundaries

Native tests cover mapping, JSON shape, privacy defaults, fixed errors,
zero-observation presentation data, incomplete stages, and absence of network
client imports. Existing workspace tests continue to cover core behavior and
CLI compatibility.

Node tests cover the untrusted JSON contract and prohibited frontend source
patterns. Playwright runs the production build with generated WASM in Chromium
and instruments network requests during analysis. These tests establish local
Chromium behavior, not behavior in every browser or a deployed hosting
environment.
