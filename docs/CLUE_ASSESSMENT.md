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

## Milestone 6.20 presentation and disclosure boundary

Milestone 6.20 moves detailed assessment reasons below the primary
assessment/context grid so they can use the full aligned page width. This does
not change reason generation, ordering, attribution, limitations, categories,
actions, next steps, context values or decision rules.

Destination disclosure is independent of assessment. Reassessment continues to
consume only the privacy-reduced consumer report and never requests or uses a
candidate hostname, reference, source parameter or transformation. Opening or
closing either destination-disclosure level cannot change a recommendation.
Assessment contract version 1 remains unchanged.

## Milestone 6.21 rule inventory and quality audit

The assessment engine was audited against its complete finite rule set. The
identifiers below document implementation predicates; they do not add fields
to assessment contract 1.

| Rule or predicate | Inputs | Family / trigger | Category effect | Recommended-action effect | Relevant limitations | Main excessive-warning risk | Main false-reassurance risk |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `family.destination-transparency` | Absolute, protocol-relative, hostname-first, or nested destination observations | Any supported embedded candidate | At least caution | Known website or app, unless a higher-priority context action applies | Supported evidence only | Legitimate redirectors and analytics links receive caution | Candidate presence does not prove navigation or danger |
| `family.layered-or-compound` | Supported normalization, Base64/HTML derivation, repeated percent transformation, encoded delimiter, or compound query | Any matching observation | At least caution | Independent verification unless another action has priority | Supported evidence only | Ordinary complex application links receive caution | Encoding is correlated structure, not intent |
| `family.ambiguity` | Ambiguous hostname-first observation | Any unresolved supported ambiguity | At least caution | Independent verification unless another action has priority | Technical ambiguity unresolved | Benign malformed or unconventional syntax receives caution | Clue cannot safely repair or classify the possible address |
| `family.analysis-incomplete` | Unsupported original path, incomplete discovery/encoding, percent limit, resource-limited/unsupported stage, or material non-traversal partial stage | Any incomplete supported analysis | `unable-to-assess-reliably`, with highest precedence | Seek additional information | Analysis incomplete | A limit can dominate otherwise ordinary structure | Missing evidence must never become a clean result |
| `context.unexpected` | `expected_contact=unexpected` | Reported unexpected contact | At least caution | Verify sender, unless sensitive/strong/incomplete rules take precedence | Supported evidence only | An unexpected benign message receives caution | User answers are unverified and may be mistaken |
| `context.unsure` | `expected_contact=unsure` | Reported uncertainty | At least caution | Independent verification | Supported evidence only | Uncertainty alone causes caution | Does not establish sender identity |
| `context.sensitive-purpose` | Account, payment, or software purpose | Any matching purpose | At least caution | Independent verification unless another action has priority | Supported evidence only | Expected routine workflows still receive caution | A claimed purpose cannot establish legitimacy |
| `context.sensitive-action` | Credentials, personal data, payment, or software action | Any matching requested action | At least caution | Avoid the sensitive action until verified | Supported evidence only | Legitimate requested actions remain cautious | A simple-looking URL cannot validate a sensitive request |
| `strong.unexpected-sensitive-action` | Unexpected contact plus sensitive action | Both predicates true | `strong-warning-signs` | Do not open | Supported evidence only | Incorrect user context can over-warn | Independent context signals justify direct avoidance despite a simple URL |
| `strong.unexpected-ambiguous-structure` | Unexpected contact, ambiguity, plus destination or layered structure | All predicates true | `strong-warning-signs` | Do not open | Technical ambiguity unresolved | Complex benign links in unexpected messages can over-warn | Ambiguity is not treated as a verified redirect |
| `strong.sensitive-ambiguous-layered` | Sensitive action, ambiguity, and layered structure | All predicates true | `strong-warning-signs` | Do not open | Technical ambiguity unresolved | Legitimate complex sign-in flows can over-warn | Sensitive context plus independent structural concerns warrants avoidance |
| `default.no-clear-warning` | No URL family, unexpected/unsure status, sensitive purpose, or sensitive action | No warning predicate true | `no-clear-structural-warning-signs` | Continue carefully | Supported evidence only; missing/partial context as applicable | None inherent | The qualified wording must not be read as a safety approval |

Recommended-action precedence is: incomplete analysis, strong warning,
sensitive action, unexpected contact, destination transparency, then general
caution. Next steps are separately deduplicated and capped at three. An
unexpected document request selects document-specific trusted-channel advice;
sensitive requested actions select action-specific advice. These choices do
not remove technical limitations.

The audit found no contradictory outcomes, unreachable categories, hidden
suppression of technical warnings by context, or escalation caused solely by
duplicate correlated observations. Incomplete analysis correctly has higher
precedence than otherwise clean structure. Expected contact does not suppress
a URL family, resolve ambiguity, or establish safety. Ordinary URLs do not
receive a strong warning unless the user reports both unexpected contact and a
sensitive requested action.

Two calibration risks remain. First, legitimate tracking, redirector, and
deeply encoded application links can produce qualified caution; this is the
main excessive-warning risk. Second, a structurally simple deceptive link can
receive the no-clear category when no meaningful context is supplied; fixed
non-safety wording and optional context mitigate but cannot eliminate this
false-reassurance risk. No assessment rule was changed in Milestone 6.21
because the expanded corpus exposed no reproducible contradiction or unsafe
precedence defect.

## Milestone 6.21 quality corpus

`assessment_quality_cases.tsv` adds 36 synthetic, reviewable scenarios to the
existing 29 regression cases. It records observations, typed context, expected
category and action, principal reasons, important limitations, a quality focus
(`false-reassurance`, `excessive-warning`, or `neither`), and a written
rationale. The cases cover ordinary, suspicious-looking, ambiguous,
incomplete, and adversarial structures. They use reserved synthetic domains
and fabricated identifiers.

The corpus test establishes **regression consistency** with documented
judgments. The rationales are a **rule-quality judgment** based on evidence the
tool actually has. They do not measure real-world phishing detection, and the
corpus pass result must not be presented as an accuracy rate.

The synthetic motivating fixture still begins at qualified caution. Reporting
an unexpected email and document purpose produces the direct **Do not open
this link** guidance with separately attributed URL and context reasons.
Changing the report to expected contact returns it to qualified caution, not a
safety approval; clearing context deterministically restores the preliminary
assessment.
