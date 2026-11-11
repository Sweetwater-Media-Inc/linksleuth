# Clue destination disclosure

## Purpose and boundary

Milestone 6.20 adds deliberate local inspection of destination evidence without
changing LinkSleuth's passive analysis. The normal browser response remains the
privacy-reduced browser wire schema 2 response. It contains no submitted or
candidate hostname, candidate reference, source parameter, decoded value, or
detailed evidence.

Destination disclosure uses the separate
`discloseDestinations(input, level)` WASM export. Its browser wire schema and
core disclosure contract are independently versioned at 1. The export reruns
the existing bounded local analysis and never fetches, resolves, previews, or
navigates to a submitted or candidate address.

## Disclosure levels

### Hostnames

The `hostnames` level returns, in source order:

- Stable candidate identity within this disclosure, such as `candidate-1`.
- Candidate hostname exactly as classified, including a meaningful trailing
  dot where retained.
- Reference kind: absolute HTTP, absolute HTTPS, protocol-relative, or
  hostname-first.
- Direct or structurally nested relationship, depth, and parent candidate
  identity where applicable.
- Fixed explanations for supported hostname-like material that was not
  confidently classified.

It does not return candidate references, parameter names, query strings,
fragments, credentials, tokens, transformations, source positions, or the
original submitted hostname.

### Detailed

The `detailed` level requires a separate action after hostname disclosure. It
adds only destination-specific evidence:

- The classified submitted candidate representation, with URL credentials and
  its literal fragment removed.
- The normalized candidate interpretation, with URL credentials and its
  literal fragment removed.
- Immediate source parameter name.
- One-based top-level and immediate source-parameter positions.
- Ordered bounded transformation explanations.
- Candidate classification, nesting depth, and parent identity.
- Source positions for unclassified ambiguity records.

Candidate query data can still contain private tokens. Detailed disclosure is
therefore sensitive even though URL credentials, fragments and the complete
top-level input are excluded. A submitted representation is the complete candidate value produced
by supported transformations; it is not a claim that navigation or a redirect
occurred. Protocol-relative and hostname-first references are not silently
rewritten as HTTPS.

Duplicate or normalized-equivalent candidates remain separate occurrences with
their own displayed identities and provenance, rather than being presented as
unique verified websites. Nested means structurally nested, not an observed
redirect chain.

## Threat model

The primary goal is to prevent accidental disclosure and navigation in a
local-only tool. It is not secrecy from the person controlling the browser or
their device.

| Surface | Control and remaining limitation |
| --- | --- |
| Default DOM | No destination values are requested or rendered. |
| Revealed DOM | Values are inserted only with `textContent`; no evidence is placed in `href`, `src`, HTML, embeds, previews, or navigation controls. |
| Hidden state | Closing disclosure removes generated nodes and frontend references. This does not prove erasure from JavaScript or WASM allocator memory. |
| Developer tools | Explicitly requested JSON and rendered text may be observable in memory or the DOM while open. Clue does not claim protection from local developer tools. |
| Clipboard | This milestone adds no destination-evidence copy action. The existing summary copy path is built only from the privacy-reduced response. Browser extensions or operating-system clipboard tools remain outside Clue's control. |
| Autofill | Revealed evidence is text, not an editable or autofill-enabled input. The submitted URL remains in the existing URL input until the user changes or closes the page. |
| Errors and logs | Disclosure arguments and URL-derived data are never reflected in errors. The application does not log responses. Browser, extension, or host crash reporting remains outside the application boundary. |
| Page source and attributes | Sensitive evidence is absent from static source and is not written to data attributes or navigation attributes. |
| Browser history | Analysis and disclosure do not change the page URL or write history state. |
| Screen sharing and screenshots | Open evidence is visible and can be captured. The UI warns before detailed disclosure; hiding cannot retract an earlier capture. |
| Summary export | Opening either level does not consent to export. Hostnames, references, parameters, transformations, and fragments remain excluded from the existing review-before-copy summary. |
| Network | Disclosure reruns local WASM analysis only. Browser request instrumentation covers analysis, reassessment, and disclosure calls in local Chromium. Static application assets may load during startup. |

The raw submitted URL necessarily exists in the URL input, JavaScript call
arguments, and WASM memory during local analysis. A disclosure response also
exists transiently in WASM serialization and JavaScript memory after the user
requests it. DOM removal narrows casual exposure and prevents stale evidence
from remaining on screen; it is not secure-memory wiping.

## Lifecycle

- Initial analysis requests only the privacy-reduced browser response.
- Hostname evidence is requested only after **Reveal destination hostnames**.
- Detailed evidence is requested only after **Reveal detailed evidence**.
- Hiding detailed evidence removes detailed nodes.
- Hiding hostnames also removes detailed nodes and resets both controls.
- Starting a new analysis removes every disclosed node before analyzing the new
  input.
- Context reassessment uses the existing privacy-reduced assessment API and
  never requests destination disclosure.
- Summary generation continues to read only the validated privacy-reduced
  analysis response.

## Versioning decision

Consumer report schema 1, assessment contract 1, and normal browser wire schema
2 remain unchanged. Destination disclosure is additive and independently
versioned:

- Core destination-disclosure contract: 1.
- Destination-disclosure browser wire schema: 1.

An incompatible change to either disclosure shape must advance its own version.
It must not weaken the normal browser response's privacy guarantees.

## Milestone 6.21 presentation levels

The evidence contract is unchanged, but hostname disclosure now presents two
language levels. The primary card says **Possible destination found**, makes
the exact classified hostname prominent, and explains:

> Clue identified this address inside the original link. It may be used as a
> destination, but Clue cannot confirm that opening the link would take you
> there.

The candidate identifier, precise reference-kind label, direct/nested
relationship, depth, and parent identifier are inside a per-candidate
**Technical details** disclosure. This disclosure does not request more data;
it reorganizes the already requested hostname-level response. The separate
**Reveal detailed evidence** action is still required before candidate
references, parameter provenance, source positions, or transformations enter
the DOM.

Hostnames remain inert text with meaningful spelling preserved. Candidate
order, identity, parent relationship, reference kind, and ambiguity records
are not altered by the common-language presentation.
