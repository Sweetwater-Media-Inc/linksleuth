# Clue MVP specification

## Purpose

Clue is a privacy-first graphical companion for LinkSleuth. Its first version
helps a person understand the structure of a URL before deciding what to do
with it. Clue explains observations and analysis limitations; it does not label
a URL safe or malicious.

This document describes the MVP implemented by the Clue browser prototype,
including Milestone 6.17 structural findings, not a production deployment or a
complete frontend design.

## Prototype status

Milestone 6.16 implements the workflow in `apps/clue` using a static Vite
application and the actual `linksleuth-wasm` adapter. The production build is
packaged with `wasm-bindgen` and verified in Chromium with runtime schema,
privacy, network, keyboard, summary, and responsive-layout tests. Detailed
evidence remains deliberately unavailable, and public hosting plus manual
assistive-technology review remain future work.

## MVP goals

- Analyze HTTP and HTTPS URLs locally with LinkSleuth's existing engine.
- Present structural observations in plain language before technical details.
- Explain whether each supported analysis stage completed, was unsupported, or
  reached a resource limit.
- Hide raw URL evidence and hostnames by default.
- Let the user deliberately reveal locally held technical evidence later.
- Produce a privacy-reduced summary suitable for deliberate copying or export.

## Primary workflow

1. The user opens Clue. Loading Clue and its static assets may contact the
   application's own host; this is separate from analysis.
2. The user pastes a URL into a clearly labelled input.
3. The user activates **Analyze URL**.
4. Clue analyzes the value locally without contacting the submitted URL or any
   discovered destination.
5. Clue displays fixed, understandable structural observations and occurrence
   counts.
6. Clue displays completeness for original-path analysis, percent decoding,
   destination traversal, and secondary encoding.
7. The user may deliberately reveal technical evidence in an expanded local
   view.
8. The user may generate a privacy-reduced summary that excludes raw evidence
   and the hostname unless a separate disclosure control is explicitly enabled.

Milestone 6.18 places a deterministic preliminary assessment before supporting
observations. Optional progressive context questions follow the preliminary
result and never block analysis. Assessment rules live in the independent Rust
assessment crate, not in JavaScript or the URL-analysis engine.

## User stories

- As a nontechnical user, I can paste a URL and understand what LinkSleuth
  structurally observed without learning decoder terminology first.
- As a privacy-conscious user, I can perform basic analysis without sending the
  submitted URL to a server.
- As a technical user, I can reveal provenance and evidence only when I choose.
- As a user of assistive technology, I can complete analysis and inspect every
  result using a keyboard and screen reader.
- As a user reviewing an incomplete result, I can see which stage was limited
  and why rather than mistaking missing observations for a clean result.
- As a user sharing results, I can create a reduced summary and review what it
  discloses before copying it.

## Initial interface sections

### URL input

- A visible label, concise example, and optional reminder that URLs may contain
  private tokens or document identifiers.
- A multiline-capable or horizontally scrollable control that does not place
  the submitted value in Clue's own page URL.
- No automatic analysis while the user is typing or pasting.
- No persistence, browser history integration, telemetry, or analytics event
  containing the value.

### Analyze action

- A clearly named **Analyze URL** button.
- Keyboard activation using standard button behavior.
- A local progress state if analysis takes perceptible time.
- Repeated activation replaces the previous result locally after confirming any
  currently revealed evidence will be cleared.

### Summary of observations

Each observation uses the consumer report's fixed title and explanation, stable
identifier, and occurrence count. Multiple occurrences of one category appear
as one observation with a count. Candidate destinations are described as
potential structural references, never verified redirects.

The primary interface does not display stable identifiers. It presents a
vertical list of fixed plain-language titles, explanations, and understated
occurrence counts. Identifiers remain in the validated browser contract for
integrations and tests.

If there are no observations, display:

> No supported structural observations were produced. This does not mean the
> URL is safe, and unsupported techniques may not have been examined.

Do not use a green safe badge, a red malicious badge, a threat score, or a
website preview.

### Analysis completeness

Display a concise overall status. The four detailed stages remain available in
the accessible **What Clue examined** disclosure:

- Original path
- Percent decoding
- Destination traversal
- Secondary encoding

Use the stage status and fixed completeness reasons. “Complete within supported
scope” means only that the bounded supported stage completed. An unsupported or
resource-limited stage remains prominent even when observations were produced.
Concrete ambiguity encountered in the submitted value is also visible outside
the collapsed disclosure.

## Milestone 6.17 supported structures

Clue can now present privacy-reduced observations for:

- Complete hostname-first references in a narrow destination-parameter
  allowlist, without inventing a scheme.
- Encoded query delimiters in those destination-bearing values.
- One deterministic compound-query interpretation with at most 16 inner
  parameters.
- Complete absolute or protocol-relative references occupying a supported
  inner parameter value.
- Multiple bounded percent-decoding transformations.
- Ambiguous hostname-first authority/path boundaries that were deliberately
  not promoted to candidates.

The compound parser does not scan arbitrary substrings, repeatedly interpret
HTML, or analyze fragments as destinations. The synthetic motivating example
therefore recognizes the complete inner protocol-relative value while leaving
the outer `hostname//path` spelling explicitly ambiguous.

## Milestone 6.18 assessment experience

The result order is a prominent recommended-action headline, assessment
category, explanation and evidence sufficiency, one to three practical next
steps, fixed reasons, optional context questions, supporting observations,
analysis coverage, and summary review. Strong warnings say **Do not open this
link**; ordinary and incomplete results remain visibly qualified. Categories
never use green safe styling, threat scores, or confirmed
malicious/legitimate language.

Context questions use finite native controls for source, claimed purpose,
expected contact, and requested action. They can be answered partially,
revised, cleared, or skipped. Reassessment reruns locally and does not mutate
the underlying observations. Context-derived summary reasons require a separate
explicit checkbox during review.

URL-derived reasons use the full card width before context is supplied. Once
context contributes reasons, the two sources use balanced columns when space
permits and stack on mobile. The UI states whether context changed the
category, only the recommended action, only practical guidance, or neither.

## Milestone 6.19 results layout

The result region is wider than the introductory input region but remains
bounded for readable line lengths. Above 60rem, the primary result uses an
approximately 55/45 assessment-and-context grid. The assessment remains first
in DOM and keyboard order. At 60rem and below, the same content stacks with the
assessment first and context second.

The optional context panel is always present after analysis. Source and purpose
questions are immediately visible, and one clearly named control reveals the
expected-contact and requested-action questions together. Changing an answer
updates the existing assessment in place without moving focus, replacing the
form, persisting data, or altering URL observations. Clearing answers restores
the preliminary assessment.

Assessment limitations remain visible in the primary card. Supporting URL
observations move to a collapsed native **Supporting URL findings** disclosure
that states the fixed observation count before expansion. The detailed
four-stage coverage view remains a separate disclosure below it. Strong,
caution, incomplete, and neutral outcomes use distinct restrained accents in
addition to explicit category text; no state is presented as a green safety
approval.

### Expandable technical details

- Hidden by default.
- Requires an explicit user action with disclosure-focused wording.
- Associates details with observation evidence references rather than merging
  raw evidence into fixed explanations.
- Reveals only locally held data and does not navigate to or preload candidate
  destinations.
- Uses text rendering, never HTML interpretation of evidence.
- Provides a clear action to hide and clear revealed evidence.

The initial browser adapter does not expose detailed evidence. This section is
the contract for a future opt-in extension, not an MVP implementation shortcut.

### Privacy-reduced report or export

- Generated from privacy-reduced report data, not by removing fields from a
  rendered detailed report.
- Excludes raw URLs, paths, parameter names and values, fragments, decoded
  payloads, destination hostnames, and the original hostname by default.
- Shows a review step before copying or downloading.
- States that observation categories and counts are metadata and are not
  guaranteed anonymous.
- Requires a separate deliberate control before including the original
  hostname. Hostname disclosure must not reveal nested hostnames.

### Privacy and limitations messaging

Place concise messaging near the input and results:

- Analysis runs locally and does not contact the analyzed URL.
- Candidate destinations are not loaded or verified.
- LinkSleuth recognizes bounded supported structures, not every concealment
  method.
- Results are observations, not a safe or malicious verdict.
- Revealed evidence and copied reports may contain sensitive information.

## Consumer report presentation model

| Consumer field | MVP presentation |
| --- | --- |
| Observation identifier | Stable internal/rendering key; available in technical context |
| Observation title | Primary plain-language heading |
| Observation explanation | Supporting fixed text |
| Occurrence count | “Observed once” or “Observed N times” |
| Evidence reference | Local association key; shown only when evidence is explicitly enabled |
| Completeness stage | One row per supported stage |
| Completeness status | Plain-language stage state, not a risk level |
| Completeness reason | Fixed explanation directly beneath the affected stage |

Presentation rules for important cases:

- **No observations:** show the explicit non-verdict message above.
- **Incomplete analysis:** lead with the affected stage and reason; do not imply
  undiscovered candidates were ruled out.
- **Unsupported original path:** say path normalization could not be concluded;
  do not render “no path finding.”
- **Several candidates:** use the aggregated occurrence count and avoid implying
  a redirect chain.
- **Repeated occurrences:** keep one category with its count; associate separate
  evidence references when evidence is enabled.
- **Evidence hidden:** do not display reference identifiers as if they were
  evidence values.
- **Evidence enabled:** clearly mark the view as sensitive and local, while
  retaining fixed observation text separately.

## Browser error contract

Errors use fixed text and never echo the input or raw Rust error formatting.

| State | Code | Message |
| --- | --- | --- |
| Empty input | `empty-input` | Enter a URL to analyze. |
| Invalid URL or missing hostname | `invalid-url` | Enter a complete HTTP or HTTPS URL. |
| Unsupported scheme | `unsupported-scheme` | Only HTTP and HTTPS URLs are supported. |
| Input too long | `input-too-long` | The URL exceeds the supported input length. |
| Too many query parameters | `too-many-query-parameters` | The URL contains more query parameters than the analyzer supports. |
| Unexpected local failure | `analysis-failure` | LinkSleuth could not prepare the local analysis result. |

Errors are announced in an accessible status region and focus moves only when
needed to help the user recover. The original value may remain in the input for
local correction, but it is not copied into the error message or logs.

## Functional requirements

- Invoke the versioned privacy-reduced browser boundary for analysis.
- Parse and validate the adapter's wire schema version before rendering.
- Render observations and all completeness stages.
- Represent zero observations and incomplete stages truthfully.
- Keep detailed evidence and hostname disclosure off by default.
- Keep input, results, and any revealed evidence in memory only for the session.
- Clear previous result state before displaying a new result.
- Produce privacy-reduced export from structured report data.

## Nonfunctional requirements

- Analysis remains passive, bounded, deterministic, and local-first.
- No analyzed URL values in telemetry, analytics, crash reports, page URLs,
  storage, service-worker caches, or network request metadata.
- No DNS, HTTP, preview, reputation, or redirect requests during analysis.
- URL-derived strings are inserted with safe text APIs, not `innerHTML`.
- The interface remains useful at common mobile and desktop sizes.
- Loading and parsing failures degrade to fixed error states.
- The browser bundle validates the expected consumer and wire schema versions.

## Privacy assumptions

- The application and its own static assets may be delivered over the network.
- Basic analysis runs in the user's browser after those assets load.
- LinkSleuth does not contact the analyzed URL, candidate destinations, DNS, or
  reputation services.
- Browser extensions, clipboard managers, operating-system features, and the
  user's device are outside Clue's control and may observe pasted data.
- Privacy-reduced output still contains category and count metadata and is not
  guaranteed anonymous.
- Detailed evidence is sensitive even when it never leaves the device.

## Accessibility considerations

- Associate every input with a visible label and instructions.
- Use native controls and logical heading order.
- Provide complete keyboard operation and visible focus indicators.
- Announce analysis completion and errors without repeatedly interrupting the
  user.
- Do not rely on color, icons, animation, or spatial position alone.
- Keep text readable at 200% zoom and support reflow without horizontal page
  scrolling.
- Use `aria-expanded` and an accessible name for technical-detail controls.
- Avoid danger/success color semantics that imply an unsupported verdict.
- Respect reduced-motion and high-contrast preferences.

## Out of scope

- Accounts, subscriptions, dashboards, or persistent history
- Remote analysis APIs or telemetry
- DNS, HTTP fetching, redirect following, previews, or reputation services
- Threat scores and safe/malicious classifications
- Browser extensions and email-client integration
- New decoder or detection techniques
- Automatic evidence or hostname disclosure
- Complex navigation beyond the single analysis workflow

## Acceptance criteria

- A valid synthetic URL can be analyzed locally and produces versioned report
  data without a network request to that URL.
- Observation title, explanation, identifier, and count render correctly.
- All completeness stages and applicable fixed reasons render correctly.
- Zero observations render the explicit non-verdict state.
- Unsupported and resource-limited stages cannot be mistaken for completion.
- Default output contains no original hostname or detailed evidence.
- Invalid inputs produce the fixed error contract without reflecting input.
- No candidate is loaded, previewed, or linked as an automatic action.
- Keyboard and screen-reader testing covers input, analysis, results, details,
  errors, and export review.
- Privacy-reduced export is generated from reduced structured data and reviewed
  before copying.

## Recommended implementation sequence

1. Package and load the tested WebAssembly adapter.
2. Validate wire and consumer schema versions.
3. Build the URL input, analyze action, and fixed error states.
4. Render privacy-reduced observations and completeness without evidence.
5. Add explicit zero-observation and incomplete-analysis messages.
6. Add privacy-reduced export with a disclosure review.
7. Design and security-review a separate opt-in evidence boundary.
8. Complete keyboard, screen-reader, zoom/reflow, and privacy testing.
9. Maintain and expand browser-runtime coverage beyond the initial Chromium
   module-loading, analysis, privacy, network, and responsive-layout tests.
