import initWasm, { analyzeUrl, assessUrl } from "./generated/linksleuth_wasm.js";
import {
  ContractError,
  validateBrowserResponse,
} from "./contract.js";

const INITIALIZATION_ERROR =
  "Clue could not start the local analyzer. Refresh the page and try again.";
const CONTRACT_ERROR =
  "Clue received an incompatible local analysis response. Refresh the page or update the application.";
const ANALYSIS_ERROR = "Clue could not complete local analysis. Try again.";
const CONTEXT_ERROR = "Clue could not update the assessment from those answers.";

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
const assessmentChange = requiredElement("assessment-change");
const assessmentReasons = requiredElement("assessment-reasons");
const assessmentLimitations = requiredElement("assessment-limitations");
const openContextButton = requiredElement("open-context");
const contextQuestions = requiredElement("context-questions");
const contextSource = requiredElement("context-source");
const contextPurpose = requiredElement("context-purpose");
const contextExpected = requiredElement("context-expected");
const contextAction = requiredElement("context-action");
const clearContextButton = requiredElement("clear-context");
const closeContextButton = requiredElement("close-context");
const contextStatus = requiredElement("context-status");
const contextSteps = [...document.querySelectorAll(".context-step")];
const contextNextButtons = [...document.querySelectorAll(".context-next")];
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
let contextRevision = 0;
const analyzerReady = initializeAnalyzer();

form.addEventListener("submit", handleAnalysis);
reviewSummaryButton.addEventListener("click", openSummaryReview);
copySummaryButton.addEventListener("click", copySummary);
closeSummaryButton.addEventListener("click", closeSummaryReview);
openContextButton.addEventListener("click", openContextQuestions);
clearContextButton.addEventListener("click", clearContext);
closeContextButton.addEventListener("click", closeContextQuestions);
includeContextSummary.addEventListener("change", refreshSummary);
for (const control of [contextSource, contextPurpose, contextExpected, contextAction]) {
  control.addEventListener("change", updateAssessmentFromContext);
}
contextNextButtons.forEach((button, index) => {
  button.addEventListener("click", () => revealContextStep(index + 1));
});

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
  resultsHeading.focus();
}

function renderAssessment(assessment, hasContext) {
  assessmentPanel.dataset.category = assessment.category_identifier;
  assessmentStatusLabel.textContent = hasContext
    ? "Assessment with optional context"
    : "Preliminary assessment";
  assessmentHeading.textContent = assessment.category_title;
  assessmentAction.textContent = assessment.action_title;
  assessmentExplanation.textContent = assessment.category_explanation;
  assessmentChange.hidden = !hasContext;
  if (hasContext) {
    assessmentChange.textContent = assessment.context_changed_recommendation
      ? "Your answers changed Clue's category or recommended action."
      : "Your answers did not change the category or recommended action.";
  }

  assessmentReasons.replaceChildren();
  const groups = [
    ["url-analysis", "From the URL analysis"],
    ["user-context", "From your optional answers"],
  ];
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

  assessmentLimitations.replaceChildren();
  for (const limitation of assessment.limitations) {
    const item = createElement("li");
    item.textContent = limitation.explanation;
    assessmentLimitations.append(item);
  }
}

function renderObservations(observations) {
  observationsContainer.replaceChildren();

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
    "Assessment",
    report.assessment.category_title,
    report.assessment.category_explanation,
    `Recommended action: ${report.assessment.action_title}`,
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

function openContextQuestions() {
  contextQuestions.hidden = false;
  openContextButton.setAttribute("aria-expanded", "true");
  contextSource.focus();
}

function closeContextQuestions() {
  contextQuestions.hidden = true;
  openContextButton.setAttribute("aria-expanded", "false");
  openContextButton.focus();
}

function revealContextStep(index) {
  const step = contextSteps[index];
  if (!step) {
    return;
  }
  step.hidden = false;
  step.querySelector("select")?.focus();
}

async function updateAssessmentFromContext() {
  if (preliminaryReport === null) {
    return;
  }
  const revision = ++contextRevision;
  contextStatus.textContent = "Updating the assessment locally…";
  const context = assessmentContext();

  try {
    const parsed = JSON.parse(assessUrl(input.value, JSON.stringify(context)));
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
  contextSteps.forEach((step, index) => {
    step.hidden = index !== 0;
  });
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
  contextQuestions.hidden = true;
  openContextButton.setAttribute("aria-expanded", "false");
  contextStatus.textContent = "";
  for (const control of [contextSource, contextPurpose, contextExpected, contextAction]) {
    control.value = "";
  }
  contextSteps.forEach((step, index) => {
    step.hidden = index !== 0;
  });
}

function showError(message) {
  currentReport = null;
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
