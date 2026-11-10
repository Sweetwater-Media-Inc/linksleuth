# Future intelligence and investigation planning

This document records possible future capabilities. Nothing described here is
implemented by LinkSleuth core, the assessment crate, the WASM adapter, or the
Clue browser application.

## Voluntary intelligence and abuse reporting

A future service could let a user deliberately submit evidence about a
concerning link. It must be separate from basic local analysis and follow these
principles:

- Explicit opt-in; never upload automatically.
- Separate directly observed URL evidence from user-reported context.
- Minimize and redact personal information before storage or review.
- Define access controls, retention periods, deletion, and audit records.
- Require corroboration before making abuse allegations.
- Require human review before contacting hosting, messaging, domain, or other
  providers.
- Never publish accusations based only on automated structural findings.

## Isolated active destination investigation

Any future system that follows URLs or inspects remote content must run outside
the passive engine and outside Clue's local browser-analysis environment. Its
design review should include:

- Disposable isolated virtual machines or microVMs; a container alone is not a
  sufficient hostile-browser boundary.
- No host filesystem, credential, clipboard, shared browser-profile, or
  authentication-token access.
- Restrictive egress policy that blocks private, loopback, link-local, and
  cloud-metadata destinations.
- Address resolution and policy revalidation after every redirect.
- Bounded redirect depth, execution time, downloads, response sizes, and
  retained artifacts.
- Controlled storage for screenshots, metadata, and fetched content.
- Investigator authorization and complete audit trails.
- Legal, privacy, retention, abuse-handling, and provider-reporting review.
- Human review of findings before conclusions or external reports.

Milestone 6.18 adds no fetching, DNS resolution, navigation, browser automation,
malware execution, reputation lookup, or intelligence submission.
