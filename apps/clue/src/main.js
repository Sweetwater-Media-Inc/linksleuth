import initWasm, {
  analyzeUrl,
  assessUrl,
  discloseDestinations,
} from "./generated/linksleuth_wasm.js";
import {
  ContractError,
  validateBrowserResponse,
  validateDestinationDisclosureResponse,
} from "./contract.js";

const INITIALIZATION_ERROR =
  "Clue could not start the local analyzer. Refresh the page and try again.";
const CONTRACT_ERROR =
  "Clue received an incompatible local analysis response. Refresh the page or update the application.";
const ANALYSIS_ERROR = "Clue could not complete local analysis. Try again.";
const CONTEXT_ERROR = "Clue could not update the assessment from those answers.";
const DISCLOSURE_ERROR =
  "Clue could not prepare that local destination disclosure. Hide the evidence and try again.";

const form = requiredElement("analysis-form");
const input = requiredElement("url-input");
const analyzeButton = requiredElement("analyze-button");
const analyzerStatus = requiredElement("analyzer-status");
const announcement = requiredElement("announcement");
const errorMessage = requiredElement("analysis-error");
const results = requiredElement("results");
const resultsHeading = requiredElement("results-heading");
const observationsContainer = requiredElement("observations");
const stagesContainer = requiredElement("completeness-stages");
const overallCompleteness = requiredElement("overall-completeness");
const coverageLimitations = requiredElement("coverage-limitations");
const coverageDetails = requiredElement("coverage-details");
const wireVersion = requiredElement("wire-version");
const consumerVersion = requiredElement("consumer-version");
const assessmentVersion = requiredElement("assessment-version");
const assessmentPanel = requiredElement("assessment");
const assessmentStatusLabel = requiredElement("assessment-status-label");
const assessmentHeading = requiredElement("assessment-heading");
const assessmentAction = requiredElement("assessment-action");
const assessmentExplanation = requiredElement("assessment-explanation");
const assessmentCategoryExplanation = requiredElement("assessment-category-explanation");
const assessmentNextSteps = requiredElement("assessment-next-steps");
const assessmentChange = requiredElement("assessment-change");
const assessmentReasonsHeading = requiredElement("assessment-reasons-heading");
const assessmentReasons = requiredElement("assessment-reasons");
const assessmentLimitations = requiredElement("assessment-limitations");
const contextSource = requiredElement("context-source");
const contextPurpose = requiredElement("context-purpose");
const contextExpected = requiredElement("context-expected");
const contextAction = requiredElement("context-action");
const showContextMoreButton = requiredElement("show-context-more");
const contextMore = requiredElement("context-more");
const clearContextButton = requiredElement("clear-context");
const contextStatus = requiredElement("context-status");
const supportingFindings = requiredElement("supporting-findings");
const observationsSummary = requiredElement("observations-summary");
const toggleDestinationHostnamesButton = requiredElement("toggle-destination-hostnames");
const destinationHostnames = requiredElement("destination-hostnames");
const destinationHostnameNotice = requiredElement("destination-hostname-notice");
const destinationHostnameList = requiredElement("destination-hostname-list");
const destinationAmbiguities = requiredElement("destination-ambiguities");
const toggleDestinationDetailsButton = requiredElement("toggle-destination-details");
const destinationDetails = requiredElement("destination-details");
const destinationDisclosureStatus = requiredElement("destination-disclosure-status");
const reviewSummaryButton = requiredElement("review-summary");
const summaryReview = requiredElement("summary-review");
const summaryHeading = requiredElement("summary-heading");
const summaryText = requiredElement("summary-text");
const copySummaryButton = requiredElement("copy-summary");
const closeSummaryButton = requiredElement("close-summary");
const copyStatus = requiredElement("copy-status");
const includeContextSummary = requiredElement("include-context-summary");

let currentReport = null;
let preliminaryReport = null;
let analyzedInput = null;
let contextRevision = 0;
const analyzerReady = initializeAnalyzer();

form.addEventListener("submit", handleAnalysis);
reviewSummaryButton.addEventListener("click", openSummaryReview);
copySummaryButton.addEventListener("click", copySummary);
closeSummaryButton.addEventListener("click", closeSummaryReview);
showContextMoreButton.addEventListener("click", toggleAdditionalContext);
clearContextButton.addEventListener("click", clearContext);
toggleDestinationHostnamesButton.addEventListener("click", toggleDestinationHostnames);
toggleDestinationDetailsButton.addEventListener("click", toggleDestinationDetails);
includeContextSummary.addEventListener("change", refreshSummary);
for (const control of [contextSource, contextPurpose, contextExpected, contextAction]) {
  control.addEventListener("change", updateAssessmentFromContext);
}

async function initializeAnalyzer() {
  try {
    await initWasm();
    analyzerStatus.textContent = "Local analyzer ready";
    document.documentElement.dataset.analyzer = "ready";
    return true;
  } catch {
    analyzerStatus.textContent = "Local analyzer unavailable";
    document.documentElement.dataset.analyzer = "unavailable";
    return false;
  }
}

async function handleAnalysis(event) {
  event.preventDefault();
  clearPreviousResult();

  if (input.value.trim().length === 0) {
    showError("Enter a URL to analyze.");
    return;
  }

  analyzeButton.disabled = true;
  analyzeButton.textContent = "Analyzing locally…";
  announcement.textContent = "Analyzing the link locally.";

  try {
    if (!(await analyzerReady)) {
      showError(INITIALIZATION_ERROR);
      return;
    }

    let parsed;
    try {
      parsed = JSON.parse(analyzeUrl(input.value));
      validateBrowserResponse(parsed);
    } catch (error) {
      if (error instanceof ContractError || error instanceof SyntaxError) {
        showError(CONTRACT_ERROR);
      } else {
        showError(ANALYSIS_ERROR);
      }
      return;
    }

    if (parsed.error !== null) {
      showError(parsed.error.message);
      return;
    }

    currentReport = parsed;
    preliminaryReport = parsed;
    analyzedInput = input.value;
    renderReport(parsed);
  } finally {
    analyzeButton.disabled = false;
    analyzeButton.textContent = "Analyze URL";
  }
}

function renderReport(response) {
  const report = response.report;
  wireVersion.textContent = String(response.wire_schema_version);
  consumerVersion.textContent = String(report.consumer_schema_version);
  assessmentVersion.textContent = String(report.assessment.contract_version);
  results.dataset.wireSchemaVersion = String(response.wire_schema_version);
  results.dataset.consumerSchemaVersion = String(report.consumer_schema_version);

  renderAssessment(report.assessment, false);
  renderObservations(report.observations);
  renderCompleteness(report.completeness);

  results.hidden = false;
  reviewSummaryButton.hidden = false;
  const count = report.observations.length;
  announcement.textContent =
    count === 1
      ? "Analysis complete. One structural observation was found."
      : `Analysis complete. ${count} structural observations were found.`;
  resultsHeading.focus({ preventScroll: true });
  resultsHeading.scrollIntoView({ block: "start", behavior: "instant" });
}

function renderAssessment(assessment, hasContext) {
  assessmentPanel.dataset.category = assessment.category_identifier;
  assessmentStatusLabel.textContent = hasContext
    ? "Assessment with optional context"
    : "Preliminary assessment";
  assessmentHeading.textContent = assessment.category_title;
  assessmentAction.textContent = assessment.action_title;
  assessmentExplanation.textContent = assessment.action_explanation;
  assessmentCategoryExplanation.textContent = assessment.category_explanation;
  assessmentNextSteps.replaceChildren();
  for (const step of assessment.next_steps) {
    const item = createElement("li");
    item.textContent = step.explanation;
    assessmentNextSteps.append(item);
  }
  assessmentChange.hidden = !hasContext;
  if (hasContext) {
    assessmentChange.textContent = contextChangeExplanation(assessment);
  }

  assessmentReasons.replaceChildren();
  const groups = [
    ["url-analysis", "From the URL analysis"],
    ["user-context", "From your optional answers"],
  ];
  const hasContextReasons = assessment.reasons.some(
    (reason) => reason.source === "user-context",
  );
  assessmentReasons.dataset.layout = hasContextReasons ? "split" : "full";
  assessmentReasonsHeading.textContent =
    assessment.category_identifier === "no-clear-structural-warning-signs"
      ? "What Clue considered"
      : "Why Clue is concerned";
  for (const [source, title] of groups) {
    const reasons = assessment.reasons.filter((reason) => reason.source === source);
    if (reasons.length === 0) {
      continue;
    }
    const group = createElement("section", "assessment-reason-group");
    const heading = createElement("h4");
    heading.textContent = title;
    const list = createElement("ul");
    for (const reason of reasons) {
      const item = createElement("li");
      item.textContent = reason.explanation;
      list.append(item);
    }
    group.append(heading, list);
    assessmentReasons.append(group);
  }
  if (assessment.reasons.length === 0) {
    const emptyReason = createElement("p", "assessment-reason-empty");
    emptyReason.textContent =
      "Clue did not identify a supported URL or context warning reason.";
    assessmentReasons.append(emptyReason);
  }

  assessmentLimitations.replaceChildren();
  for (const limitation of assessment.limitations) {
    const item = createElement("li");
    item.textContent = limitation.explanation;
    assessmentLimitations.append(item);
  }
}

function contextChangeExplanation(assessment) {
  const preliminary = preliminaryReport?.report.assessment;
  if (preliminary === undefined || !assessment.context_changed_recommendation) {
    return "Your answers did not change Clue's assessment or practical guidance.";
  }

  if (assessment.category_identifier !== preliminary.category_identifier) {
    const change = `Your answers changed Clue's assessment from '${preliminary.category_title}' to '${assessment.category_title}'.`;
    const hasUnexpectedDocumentReason = assessment.reasons.some(
      (reason) => reason.identifier === "context.document-request-unexpected",
    );
    return hasUnexpectedDocumentReason
      ? `${change} The unexpected document-signing request contributed to the stronger recommendation.`
      : change;
  }

  if (assessment.action_identifier !== preliminary.action_identifier) {
    return `Your answers changed Clue's recommended action from '${preliminary.action_title}' to '${assessment.action_title}'.`;
  }

  return "Your answers changed Clue's practical next steps.";
}

function renderObservations(observations) {
  observationsContainer.replaceChildren();
  supportingFindings.open = false;
  observationsSummary.textContent = observationCountSummary(observations.length);

  if (observations.length === 0) {
    const emptyState = createElement("div", "empty-state");
    const title = createElement("h3", "empty-state-title");
    title.textContent = "No supported structural observations were found.";
    const explanation = createElement("p");
    explanation.textContent =
      "This does not mean the link is safe. LinkSleuth only examines supported URL structures.";
    emptyState.append(title, explanation);
    observationsContainer.append(emptyState);
    return;
  }

  for (const observation of observations) {
    const card = createElement("article", "observation-card");
    const heading = createElement("h3");
    heading.textContent = observation.title;
    const count = createElement("p", "occurrence-count");
    count.textContent =
      observation.occurrence_count === 1
        ? "Observed once"
        : `Observed ${observation.occurrence_count} times`;
    const explanation = createElement("p", "observation-explanation");
    explanation.textContent = observation.explanation;
    card.append(heading, count, explanation);
    observationsContainer.append(card);
  }
}

function observationCountSummary(count) {
  if (count === 0) {
    return "Clue identified no supported structural observations.";
  }
  if (count === 1) {
    return "Clue identified 1 structural observation.";
  }
  return `Clue identified ${count} structural observations.`;
}

async function toggleDestinationHostnames() {
  if (toggleDestinationHostnamesButton.getAttribute("aria-expanded") === "true") {
    clearDestinationDisclosure();
    destinationDisclosureStatus.textContent =
      "Potential destination hostnames and detailed evidence are hidden and removed from the page.";
    return;
  }

  const disclosure = await requestDestinationDisclosure(
    "hostnames",
    toggleDestinationHostnamesButton,
    "Revealing locally…",
  );
  if (disclosure === null) {
    return;
  }
  renderDestinationHostnames(disclosure);
  destinationHostnames.hidden = false;
  toggleDestinationHostnamesButton.setAttribute("aria-expanded", "true");
  toggleDestinationHostnamesButton.textContent = "Hide destination hostnames";
  destinationDisclosureStatus.textContent =
    disclosure.candidates.length === 1
      ? "One potential destination hostname revealed locally."
      : `${disclosure.candidates.length} potential destination hostnames revealed locally.`;
}

async function toggleDestinationDetails() {
  if (toggleDestinationDetailsButton.getAttribute("aria-expanded") === "true") {
    hideDestinationDetails();
    destinationDisclosureStatus.textContent =
      "Detailed destination evidence is hidden and removed from the page.";
    return;
  }

  const disclosure = await requestDestinationDisclosure(
    "detailed",
    toggleDestinationDetailsButton,
    "Preparing sensitive evidence…",
  );
  if (disclosure === null) {
    return;
  }
  renderDestinationDetails(disclosure);
  destinationDetails.hidden = false;
  toggleDestinationDetailsButton.setAttribute("aria-expanded", "true");
  toggleDestinationDetailsButton.textContent = "Hide detailed evidence";
  destinationDisclosureStatus.textContent =
    "Detailed destination evidence revealed locally. It may contain private link data.";
}

async function requestDestinationDisclosure(level, button, busyLabel) {
  if (analyzedInput === null) {
    destinationDisclosureStatus.textContent = DISCLOSURE_ERROR;
    return null;
  }
  const restingLabel = button.textContent;
  button.disabled = true;
  button.textContent = busyLabel;
  try {
    const parsed = JSON.parse(discloseDestinations(analyzedInput, level));
    validateDestinationDisclosureResponse(parsed, level);
    if (parsed.error !== null) {
      destinationDisclosureStatus.textContent = DISCLOSURE_ERROR;
      return null;
    }
    return parsed.disclosure;
  } catch {
    destinationDisclosureStatus.textContent = DISCLOSURE_ERROR;
    return null;
  } finally {
    button.disabled = false;
    button.textContent = restingLabel;
  }
}

function renderDestinationHostnames(disclosure) {
  destinationHostnameList.replaceChildren();
  destinationAmbiguities.replaceChildren();
  destinationHostnameNotice.textContent = disclosure.notice;

  if (disclosure.candidates.length === 0) {
    const empty = createElement("p", "destination-empty");
    empty.textContent =
      "Clue couldn't identify a complete destination address from the supported structures it examined.";
    destinationHostnameList.append(empty);
  } else {
    for (const candidate of disclosure.candidates) {
      const card = createElement("article", "destination-card");
      const identity = createElement("p", "destination-candidate-id");
      identity.textContent = candidate.identifier;
      const heading = createElement("h4");
      const hostname = createElement("code", "destination-hostname");
      hostname.textContent = candidate.hostname;
      heading.append(hostname);
      const metadata = createElement("dl", "destination-metadata");
      appendDefinition(metadata, "Reference type", destinationKindLabel(candidate.reference_kind));
      appendDefinition(
        metadata,
        "Relationship",
        candidate.relationship === "direct"
          ? "Direct candidate"
          : `Nested candidate at depth ${candidate.depth}`,
      );
      if (candidate.parent_candidate_identifier !== null) {
        appendDefinition(metadata, "Structurally nested in", candidate.parent_candidate_identifier);
      }
      const caution = createElement("p", "destination-caution");
      caution.textContent = "Not visited or verified. This is inert text, not a link.";
      card.append(identity, heading, metadata, caution);
      destinationHostnameList.append(card);
    }
  }

  if (disclosure.ambiguities.length > 0) {
    const heading = createElement("h4");
    heading.textContent = "Unclassified address-like material";
    const intro = createElement("p");
    intro.textContent =
      "Part of this link resembles another address, but Clue couldn't identify it confidently.";
    destinationAmbiguities.append(heading, intro);
    for (const ambiguity of disclosure.ambiguities) {
      const item = createElement("p", "destination-ambiguity");
      item.textContent = ambiguity.explanation;
      destinationAmbiguities.append(item);
    }
  }
}

function renderDestinationDetails(disclosure) {
  destinationDetails.replaceChildren();
  const warning = createElement("p", "detailed-evidence-warning");
  warning.textContent = disclosure.notice;
  destinationDetails.append(warning);

  if (disclosure.candidates.length === 0) {
    const empty = createElement("p", "destination-empty");
    empty.textContent = "No supported candidate detail is available.";
    destinationDetails.append(empty);
  }

  for (const candidate of disclosure.candidates) {
    const card = createElement("article", "destination-detail-card");
    const heading = createElement("h4");
    heading.textContent = `${candidate.identifier}: ${candidate.hostname}`;
    const metadata = createElement("dl", "destination-metadata destination-detail-metadata");
    appendDefinition(metadata, "Classification", destinationKindLabel(candidate.reference_kind));
    appendDefinition(
      metadata,
      "Provenance",
      candidate.relationship === "direct"
        ? "Direct candidate from the submitted link"
        : `Structurally nested at depth ${candidate.depth}`,
    );
    if (candidate.parent_candidate_identifier !== null) {
      appendDefinition(metadata, "Parent candidate", candidate.parent_candidate_identifier);
    }
    appendDefinition(metadata, "Source parameter", candidate.detail.source_parameter || "Unnamed");
    appendDefinition(
      metadata,
      "Source positions",
      `Top-level parameter ${candidate.detail.top_level_parameter_position}; immediate source parameter ${candidate.detail.source_parameter_position}`,
    );

    const submittedLabel = createElement("p", "destination-value-label");
    submittedLabel.textContent =
      "Classified submitted representation (credentials and fragment omitted)";
    const submitted = createElement("code", "destination-reference");
    submitted.textContent = candidate.detail.submitted_reference;
    const normalizedLabel = createElement("p", "destination-value-label");
    normalizedLabel.textContent =
      "Normalized interpretation (credentials and fragment omitted)";
    const normalized = createElement("code", "destination-reference");
    normalized.textContent = candidate.detail.normalized_reference;

    const transformationHeading = createElement("h5");
    transformationHeading.textContent = "Supported transformation sequence";
    const transformationList = createElement("ol", "destination-transformations");
    if (candidate.detail.transformations.length === 0) {
      const item = createElement("li");
      item.textContent = "No decoding transformation was required.";
      transformationList.append(item);
    } else {
      for (const transformation of candidate.detail.transformations) {
        const item = createElement("li");
        item.textContent = transformation.explanation;
        transformationList.append(item);
      }
    }

    card.append(
      heading,
      metadata,
      submittedLabel,
      submitted,
      normalizedLabel,
      normalized,
      transformationHeading,
      transformationList,
    );
    destinationDetails.append(card);
  }

  if (disclosure.ambiguities.length > 0) {
    const ambiguitySection = createElement("section", "destination-detail-ambiguities");
    const heading = createElement("h4");
    heading.textContent = "Ambiguity and incomplete interpretation";
    ambiguitySection.append(heading);
    for (const ambiguity of disclosure.ambiguities) {
      const item = createElement("p");
      item.textContent = `${ambiguity.explanation} Source positions: top-level parameter ${ambiguity.top_level_parameter_position}, immediate source parameter ${ambiguity.source_parameter_position}.`;
      ambiguitySection.append(item);
    }
    destinationDetails.append(ambiguitySection);
  }
}

function clearDestinationDisclosure() {
  hideDestinationDetails();
  destinationHostnameList.replaceChildren();
  destinationAmbiguities.replaceChildren();
  destinationHostnameNotice.textContent = "";
  destinationHostnames.hidden = true;
  toggleDestinationHostnamesButton.setAttribute("aria-expanded", "false");
  toggleDestinationHostnamesButton.textContent = "Reveal destination hostnames";
}

function hideDestinationDetails() {
  destinationDetails.replaceChildren();
  destinationDetails.hidden = true;
  toggleDestinationDetailsButton.setAttribute("aria-expanded", "false");
  toggleDestinationDetailsButton.textContent = "Reveal detailed evidence";
}

function destinationKindLabel(kind) {
  const labels = {
    "absolute-http": "Absolute HTTP reference",
    "absolute-https": "Absolute HTTPS reference",
    "protocol-relative": "Protocol-relative reference; no scheme was verified",
    "hostname-first": "Hostname-first reference; no scheme was inferred",
  };
  return labels[kind] ?? "Unknown supported reference";
}

function appendDefinition(list, termText, valueText) {
  const term = createElement("dt");
  term.textContent = termText;
  const value = createElement("dd");
  value.textContent = valueText;
  list.append(term, value);
}

function renderCompleteness(completeness) {
  overallCompleteness.textContent = completenessStatusLabel(completeness.status);
  stagesContainer.replaceChildren();
  coverageLimitations.replaceChildren();
  coverageLimitations.hidden = true;
  coverageDetails.open = false;

  for (const stage of completeness.stages) {
    const item = createElement("article", "stage-card");
    if (stage.status !== "complete-within-supported-scope") {
      item.classList.add("stage-limited");
    }

    const headingRow = createElement("div", "stage-heading");
    const heading = createElement("h4");
    heading.textContent = stageLabel(stage.identifier);
    const status = createElement("span", "stage-status");
    status.textContent = stageStatusLabel(stage.status);
    headingRow.append(heading, status);
    item.append(headingRow);

    for (const reason of stage.reasons) {
      const reasonText = createElement("p", "stage-reason");
      reasonText.textContent = reason.explanation;
      item.append(reasonText);
    }

    stagesContainer.append(item);

    if (stage.status !== "complete-within-supported-scope") {
      coverageLimitations.hidden = false;
      const limitation = createElement("article", "coverage-limitation");
      const limitationTitle = createElement("h4");
      limitationTitle.textContent = `${stageLabel(stage.identifier)} was not fully examined`;
      limitation.append(limitationTitle);
      for (const reason of stage.reasons) {
        const reasonText = createElement("p");
        reasonText.textContent = reason.explanation;
        limitation.append(reasonText);
      }
      coverageLimitations.append(limitation);
    }
  }
}

function openSummaryReview() {
  if (currentReport === null) {
    return;
  }
  includeContextSummary.checked = false;
  summaryText.value = buildSummary(currentReport, false);
  summaryReview.hidden = false;
  reviewSummaryButton.setAttribute("aria-expanded", "true");
  copyStatus.textContent = "";
  summaryHeading.focus();
}

function refreshSummary() {
  if (currentReport !== null && !summaryReview.hidden) {
    summaryText.value = buildSummary(currentReport, includeContextSummary.checked);
  }
}

async function copySummary() {
  try {
    await navigator.clipboard.writeText(summaryText.value);
    copyStatus.textContent = "Summary copied to the clipboard.";
  } catch {
    summaryText.focus();
    summaryText.select();
    copyStatus.textContent =
      "Clipboard access is unavailable. The summary is selected for manual copying.";
  }
}

function closeSummaryReview() {
  summaryReview.hidden = true;
  reviewSummaryButton.setAttribute("aria-expanded", "false");
  copyStatus.textContent = "";
  reviewSummaryButton.focus();
}

function buildSummary(response, includeContextReasons) {
  const report = response.report;
  const lines = [
    "Clue privacy-reduced summary",
    "============================",
    `Browser wire schema: ${response.wire_schema_version}`,
    `Consumer report schema: ${report.consumer_schema_version}`,
    `Assessment contract: ${report.assessment.contract_version}`,
    "",
    "Recommended action",
    report.assessment.action_title,
    report.assessment.action_explanation,
    "",
    `Assessment category: ${report.assessment.category_title}`,
    report.assessment.category_explanation,
    "",
    "What you should do",
    ...report.assessment.next_steps.map((step) => `- ${step.explanation}`),
    "",
    "Assessment reasons",
  ];
  const assessmentReasons = report.assessment.reasons.filter(
    (reason) => reason.source === "url-analysis" || includeContextReasons,
  );
  if (assessmentReasons.length === 0) {
    lines.push("No supporting warning reasons were produced.");
  } else {
    for (const reason of assessmentReasons) {
      lines.push(`- ${reason.explanation}`);
    }
  }
  if (
    !includeContextReasons &&
    report.assessment.reasons.some((reason) => reason.source === "user-context")
  ) {
    lines.push("- Optional user-context reasons were excluded from this summary.");
  }
  lines.push("", "Assessment limitations");
  for (const limitation of report.assessment.limitations) {
    lines.push(`- ${limitation.explanation}`);
  }
  lines.push("", "Observations");

  if (report.observations.length === 0) {
    lines.push(
      "No supported structural observations were found.",
      "This does not mean the link is safe. LinkSleuth only examines supported URL structures.",
    );
  } else {
    report.observations.forEach((observation, index) => {
      lines.push(
        `${index + 1}. ${observation.title} — ${occurrenceSummary(observation.occurrence_count)}`,
        `   ${observation.explanation}`,
      );
    });
  }

  lines.push("", "Analysis completeness");
  for (const stage of report.completeness.stages) {
    lines.push(`- ${stageLabel(stage.identifier)}: ${stageStatusLabel(stage.status)}`);
    for (const reason of stage.reasons) {
      lines.push(`  ${reason.explanation}`);
    }
  }

  lines.push(
    "",
    "This is structural analysis, not a security verdict.",
    "The summary contains observation metadata and is not guaranteed anonymous.",
  );
  return lines.join("\n");
}

function toggleAdditionalContext() {
  const shouldShow = contextMore.hidden;
  contextMore.hidden = !shouldShow;
  showContextMoreButton.setAttribute("aria-expanded", String(shouldShow));
  showContextMoreButton.textContent = shouldShow
    ? "Hide two additional questions"
    : "Show two more optional questions";
}

async function updateAssessmentFromContext() {
  if (preliminaryReport === null) {
    return;
  }
  const revision = ++contextRevision;
  contextStatus.textContent = "Updating the assessment locally…";
  const context = assessmentContext();

  try {
    const parsed = JSON.parse(assessUrl(analyzedInput, JSON.stringify(context)));
    validateBrowserResponse(parsed);
    if (revision !== contextRevision) {
      return;
    }
    if (parsed.error !== null) {
      contextStatus.textContent = CONTEXT_ERROR;
      return;
    }
    currentReport = parsed;
    const hasContext = Object.keys(context).length > 0;
    renderAssessment(parsed.report.assessment, hasContext);
    contextStatus.textContent = parsed.report.assessment.context_changed_recommendation
      ? "Assessment updated. Your answers changed the guidance."
      : "Assessment updated. The guidance did not change.";
    announcement.textContent = contextStatus.textContent;
  } catch {
    if (revision === contextRevision) {
      contextStatus.textContent = CONTEXT_ERROR;
    }
  }
}

function assessmentContext() {
  const entries = [
    ["source", contextSource.value],
    ["claimed_purpose", contextPurpose.value],
    ["expected_contact", contextExpected.value],
    ["requested_action", contextAction.value],
  ];
  return Object.fromEntries(entries.filter(([, value]) => value !== ""));
}

function clearContext() {
  contextRevision += 1;
  for (const control of [contextSource, contextPurpose, contextExpected, contextAction]) {
    control.value = "";
  }
  if (preliminaryReport !== null) {
    currentReport = preliminaryReport;
    renderAssessment(preliminaryReport.report.assessment, false);
  }
  contextStatus.textContent = "Optional answers cleared. Showing the preliminary assessment.";
  announcement.textContent = contextStatus.textContent;
}

function clearPreviousResult() {
  currentReport = null;
  preliminaryReport = null;
  analyzedInput = null;
  contextRevision += 1;
  errorMessage.hidden = true;
  errorMessage.textContent = "";
  results.hidden = true;
  observationsContainer.replaceChildren();
  stagesContainer.replaceChildren();
  coverageLimitations.replaceChildren();
  coverageLimitations.hidden = true;
  coverageDetails.open = false;
  summaryReview.hidden = true;
  summaryText.value = "";
  copyStatus.textContent = "";
  reviewSummaryButton.setAttribute("aria-expanded", "false");
  includeContextSummary.checked = false;
  contextMore.hidden = true;
  showContextMoreButton.setAttribute("aria-expanded", "false");
  showContextMoreButton.textContent = "Show two more optional questions";
  contextStatus.textContent = "";
  for (const control of [contextSource, contextPurpose, contextExpected, contextAction]) {
    control.value = "";
  }
  supportingFindings.open = false;
  observationsSummary.textContent = "";
  clearDestinationDisclosure();
  destinationDisclosureStatus.textContent = "";
}

function showError(message) {
  currentReport = null;
  preliminaryReport = null;
  analyzedInput = null;
  clearDestinationDisclosure();
  results.hidden = true;
  summaryReview.hidden = true;
  errorMessage.textContent = message;
  errorMessage.hidden = false;
  announcement.textContent = `Analysis could not be completed. ${message}`;
}

function stageLabel(identifier) {
  const labels = {
    "original-path": "Original path",
    "percent-decoding": "Percent decoding",
    "destination-traversal": "Destination traversal",
    "secondary-encoding": "Secondary encoding",
  };
  return labels[identifier] ?? "Unknown stage";
}

function stageStatusLabel(status) {
  const labels = {
    "complete-within-supported-scope": "Examined within supported scope",
    "partially-examined": "Partially examined",
    "unsupported-evidence": "Unsupported evidence",
    "resource-limited": "Analysis limit reached",
  };
  return labels[status] ?? "Unknown status";
}

function completenessStatusLabel(status) {
  return status === "complete-within-supported-scope"
    ? "Clue completed its supported checks."
    : "Clue could not fully examine every supported structure in this link.";
}

function occurrenceSummary(count) {
  return count === 1 ? "observed once" : `observed ${count} times`;
}

function createElement(tagName, className) {
  const element = document.createElement(tagName);
  if (className) {
    element.className = className;
  }
  return element;
}

function requiredElement(id) {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error("Clue could not initialize its local interface.");
  }
  return element;
}
