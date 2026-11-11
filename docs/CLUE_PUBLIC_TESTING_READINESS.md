# Clue public-testing readiness assessment

## Decision

Clue is suitable for continued local evaluation and a small, supervised
usability study using synthetic links. It is **not ready for public launch**.
Broader public testing remains blocked on deployed-environment privacy and CSP
verification, manual assistive-technology testing, nontechnical usability
evidence, and completion of WebKit verification in a working environment.

Passing synthetic tests establishes deterministic behavior against documented
expectations. It does not establish real-world phishing-detection accuracy,
website safety, or production security.

## Status summary

| Area | Status | Evidence and boundary |
| --- | --- | --- |
| Passive local analysis | **Verified locally** | Rust and actual-WASM browser tests preserve no DNS, HTTP destination requests, redirect following, reputation lookup, telemetry, or persistence. |
| Core discovery regression | **Verified locally** | The existing 62-case destination corpus remains in the workspace test suite. Parsing semantics were not changed in Milestone 6.21. |
| Assessment consistency | **Verified locally** | Existing 29 cases plus 36 separate quality scenarios pass documented category, action, reason, limitation, and principal-observation expectations. Exhaustive bounded context combinations are deterministic and all four categories are reachable. |
| Assessment quality | **Partially verified** | A rule audit found no contradiction, unsafe precedence, unreachable category, context suppression, or duplicate-evidence escalation. Judgments remain synthetic and have not been calibrated against representative real messages or user decisions. |
| Chromium | **Verified locally** | Production WASM, desktop/mobile workflows, privacy, disclosure, accessibility, and responsive tests pass in the installed Playwright Chromium. |
| Firefox | **Verified locally** | The focused production-WASM workflow covers initialization, ordinary/preliminary behavior through the broader suite's shared contracts, motivating fixture, context update/clear, both disclosure levels, hide, privacy-reduced summary, and request instrumentation in installed Playwright Firefox. |
| WebKit | **Blocked in this environment** | Installed Playwright WebKit 27.2 launches and loads a `data:` document, but every localhost HTTP navigation fails before application code with `page.goto: WebKit encountered an internal error`, including a plain Python directory server. This is an environment/engine blocker, not a demonstrated Clue pass or failure. |
| Automated accessibility | **Verified locally within automation limits** | `@axe-core/playwright` 4.14.0 (MPL-2.0, development-only) reports zero violations in six Chromium states: initial, preliminary, strong warning, hostname disclosure, expanded technical/detailed disclosure, and 390px stacked results. Interaction tests cover labels, keyboard controls, disclosure states, focus movement, live status, and error alerts. |
| Manual accessibility | **Not verified** | No manual screen-reader, forced-colors, high zoom, or assistive-technology session was performed. A reproducible checklist is available in `CLUE_MANUAL_ACCESSIBILITY_CHECKLIST.md`. |
| Privacy-reduced default | **Verified locally** | Synthetic markers are absent from default results/errors; evidence is excluded from summaries; context reasons require summary opt-in; disclosure DOM is generated only after explicit actions and removed on hide/new analysis. |
| Local CSP | **Partially verified** | The static build has a restrictive meta CSP and local browser tests pass. A meta policy cannot replace production response headers and the deployed policy is not known. |
| Production hosting | **Not verified** | No public host, HTTP security headers, caching policy, TLS configuration, asset integrity, monitoring boundary, or deployed privacy behavior was inspected. |
| Nontechnical usability | **Not verified** | No volunteers were recruited or tested. `CLUE_USABILITY_TESTING_PROTOCOL.md` defines a later synthetic-data study. |
| Public launch | **Blocked** | Manual accessibility, usability, deployed security/privacy review, WebKit verification, and release governance remain outstanding. |

## Implemented user-facing capability

Clue accepts an HTTP/HTTPS URL, performs bounded analysis in local WASM, and
shows an action-first qualified assessment with optional structured context.
It separates URL-derived and context-derived reasons, retains limitations,
keeps supporting observations collapsed, and provides a review-before-copy
privacy-reduced summary.

Potential destination hostnames and detailed evidence remain separate explicit
disclosures. Hostname cards now lead with ordinary-language meaning and place
candidate identity/reference/nesting metadata behind **Technical details**.
Detailed references, parameters, positions, and transformations still require
the separate detailed-evidence action. All evidence is rendered as inert text.

## Assessment quality findings

The finite rule inventory is documented in `CLUE_ASSESSMENT.md`. Incomplete
analysis has highest precedence. Strong warnings require independent context
and/or structural predicates rather than a numerical score. Correlated
observations collapse into four Boolean evidence families. Expected contact
does not erase technical evidence, and clearing context restores the same
preliminary assessment.

No rule change was justified by the 36 added scenarios. The main known
excessive-warning risk is caution for legitimate analytics, redirector, or
complex encoded links. The main false-reassurance risk is that structurally
simple deceptive links can receive the qualified no-clear category when useful
context is absent. Fixed language says that this is not safety evidence, but
users may still overgeneralize it. Synthetic corpus success is regression
consistency and a documented rule-quality judgment—not phishing accuracy.

## Privacy and disclosure limitations

The submitted URL necessarily exists in the input, JavaScript argument, and
WASM memory during analysis. Explicit disclosure responses exist transiently
in serialization and JavaScript memory. Removing DOM nodes is not secure
memory erasure. Browser extensions, developer tools, crash reporting, screen
capture, operating-system facilities, and a compromised host are outside the
application boundary.

Hostname disclosure intentionally reveals classified hostnames and technical
metadata. Detailed disclosure can reveal candidate query data and parameter
names. The UI warns about screen sharing and requires separate actions, but it
cannot retract evidence already observed or captured. The summary is
privacy-reduced, not guaranteed anonymous.

The no-request assertions cover locally instrumented application workflows.
They do not prove behavior of browser extensions, deployment infrastructure,
or future third-party assets. Startup still loads Clue's own local assets.

## Accessibility findings and limitations

Automated checks found and corrected one WCAG 2.5.3 label-in-name issue: the
brand link's custom accessible name omitted visible “Powered by LinkSleuth”
text. No axe violations remain in the six tested states. Automated analysis
cannot establish announcement quality, comprehension, focus usability in all
assistive technologies, 400% reflow, forced-colors behavior, or real
screen-reader reading order. Those remain manual blockers.

## Outstanding blockers and recommended sequence

1. Run the manual accessibility checklist with named desktop and mobile
   assistive-technology/browser combinations; remediate task-blocking findings.
2. Complete WebKit production-WASM testing on another host or CI image where a
   basic localhost HTTP page loads. Re-run `npm run test:cross-browser`; do not
   infer Safari support from Chromium or Firefox.
3. Conduct the five-to-ten-person usability protocol with synthetic fixtures.
   Treat safety-approval interpretations, missed strong warnings, and verified-
   redirect assumptions as release-blocking comprehension failures.
4. Choose a candidate static host and review TLS, cache rules, MIME types,
   source maps, logging, third-party services, privacy policy, and security
   headers. Send a reviewed CSP as an HTTP response header and rerun all browser
   request/privacy tests against the deployed origin.
5. Obtain focused security/privacy review of the compiled client, disclosure
   lifecycle, supply-chain dependencies, and deployment configuration.
6. Define incident handling, feedback intake, data-retention promises, version
   labeling, and rollback before inviting broader unsupervised use.

The next milestone should concentrate on manual accessibility and supervised
usability evidence, a working WebKit verification environment, and a concrete
deployment-hardening plan. It should not broaden URL parsing semantics or add
reputation/network features merely to prepare a launch.
