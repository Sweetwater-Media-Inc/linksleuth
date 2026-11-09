# LinkSleuth consumer application example

This small workspace application demonstrates how an independent Rust package
can use only LinkSleuth's public consumer API. It prints fixed observations and
stage-specific completeness without printing raw URL evidence, the original
hostname, or detailed evidence.

Run the built-in synthetic example:

```bash
cargo run -p linksleuth-consumer-app
```

Or supply an HTTP/HTTPS URL:

```bash
cargo run -p linksleuth-consumer-app -- \
  'https://shop.example/?target=https%3A%2F%2Fvendor.example%2Fproducts'
```

Command-line arguments can be retained in shell history and process listings.
Use synthetic inputs for demonstrations and do not assume the example sanitizes
how the URL reaches the process. The displayed report is privacy-reduced, not a
safety verdict or a guarantee of anonymity.
