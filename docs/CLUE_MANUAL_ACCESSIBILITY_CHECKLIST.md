# Clue manual accessibility checklist

## Status and setup

This is a future-test plan. Milestone 6.21 did not perform a manual
screen-reader audit. Record the date, operating system, browser, assistive
technology and version, viewport, input method, and any browser extensions for
each run. Use only synthetic URLs and fictional context.

Test at least one desktop screen reader/browser pairing and one mobile screen
reader/browser pairing. Repeat keyboard and zoom checks in current Chromium,
Firefox, and WebKit-family browsers where practical.

## Keyboard-only workflow

- Load Clue and use the skip link. Expect focus to move to the analysis region
  with a visible indicator.
- Reach the URL input, Analyze button, all selects, context expansion, clear
  action, findings disclosures, destination controls, summary review, and copy
  controls using Tab and Shift+Tab only. Expect a logical order and visible
  focus at every step.
- Submit from the URL input with Enter. Expect focus to move once to **Clue's
  assessment**, without trapping focus or unexpectedly scrolling away from the
  result.
- Open and close native details controls with Enter or Space. Expect their
  expanded state and content visibility to agree.
- Reveal and hide hostname and detailed evidence. Expect the invoking buttons
  to remain operable, labels and expanded states to update, and hidden
  generated evidence to be removed.
- Open and close summary review. Expect focus to enter at its heading and
  return to **Review summary** when closed.
- Shift+Tab back through the page and return to the URL input. Expect no focus
  loss or keyboard trap.

## Screen-reader workflow

- Navigate by landmarks and headings. Expect one page heading followed by a
  coherent analyzer/result hierarchy; no heading should depend on visual
  position alone.
- Inspect the URL input, every select, and every button. Expect a concise,
  unique accessible name and the help/error association for the input.
- Submit an empty and then invalid value. Expect the fixed error to be
  announced without echoing the submitted text.
- Analyze an ordinary synthetic URL. Expect the action-first headline,
  qualified category, next step, and limitations to be understandable in DOM
  order. It must not sound like a safety approval.
- Answer the first two context questions, reveal the additional two, and
  create a strong-warning scenario. Expect question sequencing to be clear,
  status changes to be announced, and **Do not open this link** to be
  unmistakable without relying on color.
- Revise and clear context. Expect updates to be announced without repeatedly
  moving focus; clearing must restore the preliminary assessment.
- Navigate collapsed supporting findings. Expect their count/summary before
  optional observation cards.
- Reveal a hostname. Expect **Possible destination found**, the exact hostname,
  the non-verification explanation, and inert-text caution before technical
  provenance.
- Open **Technical details**, then separately reveal detailed evidence. Expect
  expanded/collapsed states to be announced and technical labels to match the
  visible data.
- Hide both levels. Expect stale evidence to be absent from subsequent virtual
  cursor navigation.
- Review the summary and its optional context-sharing checkbox. Expect the
  opt-in consequence to be clear before copying.

## Visual adaptation and motion

- At 200% and 400% browser zoom, and with the viewport near 390 CSS pixels,
  expect reflow into one column, readable controls, no two-dimensional
  scrolling, and no clipped hostname or technical value.
- Increase operating-system text size where supported. Expect labels, status
  messages, warnings, and controls to remain visible and usable.
- Enable reduced motion. Expect no essential information to depend on motion;
  result focus and disclosure state changes must remain understandable.
- Check high-contrast/forced-colors mode where available. Expect action labels,
  borders, focus, and disclosure controls to remain perceivable without their
  normal colors.

Record pass/fail, exact reproduction steps, severity, screenshots only with
synthetic data, and whether the issue blocks task completion. Automated axe
results are supporting evidence, not a substitute for this checklist.
