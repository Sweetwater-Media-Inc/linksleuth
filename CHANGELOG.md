# Changelog

All notable changes to LinkSleuth are documented here. This project uses the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [Unreleased]

### Added
- Rust workspace with reusable `linksleuth-core` library and `linksleuth-cli` executable.
- HTTP/HTTPS URL parsing, including query parameters and fragments.
- Preservation of encoded and decoded query parameter values.
- Bounded multi-pass percent decoding with an evidence trail and completion status.
- Per-parameter decoding results and error isolation.
- Passive discovery of complete HTTP/HTTPS URLs embedded in query parameter values.
- Automated tests for parsing, decoding, integration, and destination discovery.

### Changed
- Percent decoding now preserves malformed or literal percent sequences and continues decoding valid sequences.
- Decoding warnings include their decoding pass and byte offset for evidence provenance.
- Tests cover literal percent signs, mixed valid/malformed input, and warnings from later decoding passes.

### Documentation
- Project README, changelog, and roadmap.

## Releases

No tagged releases yet. The crates currently use development version `0.1.0`. The entries above describe work on the main development branch, not a published release.
