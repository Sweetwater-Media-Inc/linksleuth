# LinkSleuth WebAssembly adapter

This crate is the minimal browser boundary for future Clue integration. It
accepts a URL string, runs the existing passive core analysis, and returns a
versioned JSON envelope containing only privacy-reduced observations and
stage-specific completeness.

The adapter intentionally has no hostname or detailed-evidence option. Those
controls require a future, explicitly designed local disclosure interface.
Consumers must parse the returned JSON as data and render strings using safe
DOM text APIs such as `textContent`; never interpolate values with `innerHTML`.

Check the native model and its tests:

```bash
cargo test -p linksleuth-wasm
```

Check the WebAssembly build:

```bash
rustup target add wasm32-unknown-unknown
cargo check -p linksleuth-wasm --target wasm32-unknown-unknown
```

On WebAssembly, `analyzeUrl(input)` returns the JSON envelope as a string. This
crate has compilation coverage, not browser-runtime execution coverage. It does
not fetch the submitted URL, resolve DNS, follow redirects, or load discovered
destinations.
