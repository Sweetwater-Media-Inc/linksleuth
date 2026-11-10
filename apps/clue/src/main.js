import initWasm, { analyzeUrl } from "./generated/linksleuth_wasm.js";
import {
  ContractError,
  validateBrowserResponse,
} from "./contract.js";

const INITIALIZATION_ERROR =
  "Clue could not start the local analyzer. Refresh the page and try again.";
const CONTRACT_ERROR =
  "Clue received an incompatible local analysis response. Refresh the page or update the application.";
const ANALYSIS_ERROR = "Clue could not complete local analysis. Try again.";

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
const reviewSummaryButton = requiredElement("review-summary");
const summaryReview = requiredElement("summary-review");
const summaryHeading = requiredElement("summary-heading");
const summaryText = requiredElement("summary-text");
const copySummaryButton = requiredElement("copy-summary");
const closeSummaryButton = requiredElement("close-summary");
const copyStatus = requiredElement("copy-status");

let currentReport = null;
const analyzerReady = initializeAnalyzer();

form.addEventListener("submit", handleAnalysis);
reviewSummaryButton.addEventListener("click", openSummaryReview);
copySummaryButton.addEventListener("click", copySummary);
closeSummaryButton.addEventListener("click", closeSummaryReview);

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
  results.dataset.wireSchemaVersion = String(response.wire_schema_version);
  results.dataset.consumerSchemaVersion = String(report.consumer_schema_version);

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
  summaryText.value = buildSummary(currentReport);
  summaryReview.hidden = false;
  reviewSummaryButton.setAttribute("aria-expanded", "true");
  copyStatus.textContent = "";
  summaryHeading.focus();
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

function buildSummary(response) {
  const report = response.report;
  const lines = [
    "Clue privacy-reduced summary",
    "============================",
    `Browser wire schema: ${response.wire_schema_version}`,
    `Consumer report schema: ${report.consumer_schema_version}`,
    "",
    "Observations",
  ];

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

function clearPreviousResult() {
  currentReport = null;
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
