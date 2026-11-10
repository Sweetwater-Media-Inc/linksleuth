# Clue context-aware assessment

## Purpose and boundary

Milestone 6.18 adds deterministic decision support above LinkSleuth's passive
URL analysis. `linksleuth-core` continues to report structure, provenance, and
coverage without assigning safety or maliciousness. The independent
`linksleuth-assessment` crate consumes only the public privacy-reduced
`ConsumerReport` and optional typed context.

An assessment is qualified guidance, not a phishing verdict, destination
verification, reputation lookup, or statistical confidence estimate.

## Assessment contract

Assessment contract version 1 exposes:

- One stable category identifier, fixed title, and fixed explanation.
- One stable recommended-action identifier, fixed headline, and fixed
  explanation.
- One to three typed, fixed-text practical next steps in deterministic order.
- Supporting reasons separated into URL-analysis and user-context sources.
- Fixed limitations.
- Whether context changed the category, recommended action, or practical steps
  relative to the empty-context preliminary assessment.

Categories are:

- `strong-warning-signs`
- `proceed-with-caution`
- `no-clear-structural-warning-signs`
- `unable-to-assess-reliably`

`strong-warning-signs` uses the direct recommended-action headline **Do not
open this link**. That is precautionary guidance, not a claim of confirmed
phishing. The other primary headlines are **Verify this link before opening
it**, **No obvious structural warning signs found**, and **Clue cannot reliably
assess this link**. No category means confirmed scam, confirmed legitimacy,
guaranteed safety, or a verified redirect.

## Evidence families and deterministic rules

Correlated observations are reduced to four Boolean URL evidence families:

1. Destination transparency: complete embedded, nested, hostname-first, or
   protocol-relative candidate observations.
2. Layered or compound structure: supported normalization, encoding, delimiter,
   or compound-query observations.
3. Destination ambiguity: a supported destination-like spelling that could not
   be classified confidently.
4. Incomplete analysis: resource-limited, unsupported, or materially partial
   supported stages.

Several observations from one transformation path therefore contribute once
to a family. There is no point total or threat percentage.

Incomplete supported analysis produces `unable-to-assess-reliably`. Strong
warning signs require meaningfully independent evidence, such as unexpected
contact plus a sensitive requested action, or unexpected contact combined with
both destination ambiguity and a separate URL-structure family. Otherwise a
single URL concern, uncertainty, unexpected contact, sensitive purpose, or
sensitive requested action produces qualified caution. An ordinary URL with no
such reasons receives `no-clear-structural-warning-signs` plus an explicit
non-safety limitation.

The synthetic motivating case combines an embedded candidate, compound
encoded structure, unresolved hostname-first ambiguity, and an unexpected
document-signing context. That combination produces `strong-warning-signs`,
`do-not-open-link`, and fixed steps to avoid opening, contact the supposed
sender through a previously trusted channel, and avoid relying on a reply to
the same message. The rule does not assert that the message came from its
claimed sender, that navigation will redirect, or that content is malicious.

Next steps are selected from typed identifiers. Credential, payment, personal
information, software-installation, unexpected document, known-site, trusted
sender, and incomplete-analysis guidance are tailored only when the typed
context or URL evidence supports them. Ordering is deterministic and the
collection is capped at three entries.

Context may change the recommended action without changing the category. For
example, an embedded candidate initially recommends using a known website; a
reported credential request retains `proceed-with-caution` but changes the
action to avoiding sensitive entry until independently verified. Saying a
message was expected never removes a technical warning or ambiguity.

## Optional context

The model accepts four optional bounded enums:

- Source: email, message, social media, website/search, QR code, or
  other/unknown.
- Claimed purpose: document, account access, delivery, payment, ordinary
  content, software, or other/unknown.
- Expected contact: expected, unexpected, or unsure.
- Requested action: view, credentials, personal information, payment,
  software, contact someone, or other/unknown.

No free-text organization field is implemented. Without an authoritative,
maintained mapping, a user-provided brand name cannot establish that a hostname
is related to an organization. Omitting that field also avoids storing or
exporting private organization text.

## Browser lifecycle and privacy

The WASM adapter does not retain an opaque global analysis session. Initial
analysis and reassessment both rerun the same bounded local analysis, then apply
the Rust assessment rules. This is deterministic, avoids hidden lifecycle
state, and leaves the observations and completeness model unchanged.

Browser wire schema 2 requires the assessment object and supports the
`assessUrl(input, contextJson)` local export. Consumer schema remains 1 and the
assessment contract is independently versioned at 1. Context JSON is limited
to 1 KiB, rejects unknown fields and enum values, and contains no free text.
The decisive headline, explanation, and typed next-step fields are additive to
wire schema 2 and assessment contract 1, so neither version advances. Existing
consumers that ignore unknown JSON fields remain compatible; Clue's validator
requires them for this application build.

The normal summary includes the recommendation headline and explanation,
assessment category, practical next steps, URL-derived reasons, limitations,
observations, and completeness. Context-derived reasons are excluded unless
the user explicitly checks the inclusion control during summary review. The
original URL, hostnames, decoded evidence, and raw context are never included.

## Bounds and limitations

- Four context fields, each an optional finite enum.
- Context JSON maximum: 1 KiB.
- Four URL evidence families and four derived context predicates.
- One non-recursive rule evaluation with fixed-size reason and limitation
  collections and at most three recommended next steps.
- No network, filesystem, storage, telemetry, model inference, reputation
  service, domain-brand mapping, or active navigation.

The system cannot verify a sender, organization, message claim, website
content, final destination, or user answer. Context can reduce the specific
uncertainty caused by missing answers, but cannot resolve technical ambiguity
or incomplete URL analysis.

## Milestone 6.19 presentation boundary

Milestone 6.19 changes only how the existing assessment contract is presented.
The browser places assessment and optional context side by side on desktop,
keeps the first two questions visible, and updates guidance in place. Important
typed limitations remain visible; structural observations are available in a
separate collapsed disclosure. Categories, action rules, next-step selection,
reason ordering, contract version 1, and all privacy semantics are unchanged.
