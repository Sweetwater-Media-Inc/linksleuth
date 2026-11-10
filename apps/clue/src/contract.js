export const EXPECTED_WIRE_SCHEMA_VERSION = 2;
export const EXPECTED_CONSUMER_SCHEMA_VERSION = 1;
export const EXPECTED_ASSESSMENT_CONTRACT_VERSION = 1;

const OVERALL_STATUSES = new Set([
  "complete-within-supported-scope",
  "partially-examined",
]);
const STAGE_STATUSES = new Set([
  "complete-within-supported-scope",
  "partially-examined",
  "unsupported-evidence",
  "resource-limited",
]);
const REQUIRED_STAGES = new Set([
  "original-path",
  "percent-decoding",
  "destination-traversal",
  "secondary-encoding",
]);

export class ContractError extends Error {
  constructor() {
    super("The local analysis response did not match Clue's supported schema.");
    this.name = "ContractError";
  }
}

export function validateBrowserResponse(value) {
  assertRecord(value);
  assertExactVersion(value.wire_schema_version, EXPECTED_WIRE_SCHEMA_VERSION);

  const hasReport = value.report !== null;
  const hasError = value.error !== null;
  if (hasReport === hasError) {
    throw new ContractError();
  }

  if (hasError) {
    validateError(value.error);
    return value;
  }

  validateReport(value.report);
  return value;
}

function validateReport(report) {
  assertRecord(report);
  assertExactVersion(
    report.consumer_schema_version,
    EXPECTED_CONSUMER_SCHEMA_VERSION,
  );
  if (report.disclosure !== "privacy-reduced") {
    throw new ContractError();
  }

  assertArray(report.observations);
  for (const observation of report.observations) {
    assertRecord(observation);
    assertNonEmptyString(observation.identifier);
    assertNonEmptyString(observation.title);
    assertNonEmptyString(observation.explanation);
    if (
      !Number.isSafeInteger(observation.occurrence_count) ||
      observation.occurrence_count < 1
    ) {
      throw new ContractError();
    }
    assertArray(observation.evidence_references);
    if (observation.evidence_references.length !== 0) {
      throw new ContractError();
    }
  }

  validateCompleteness(report.completeness);
  validateAssessment(report.assessment);
}

function validateAssessment(assessment) {
  assertRecord(assessment);
  assertExactVersion(
    assessment.contract_version,
    EXPECTED_ASSESSMENT_CONTRACT_VERSION,
  );
  for (const field of [
    "category_identifier",
    "category_title",
    "category_explanation",
    "action_identifier",
    "action_title",
  ]) {
    assertNonEmptyString(assessment[field]);
  }
  assertArray(assessment.reasons);
  for (const reason of assessment.reasons) {
    assertRecord(reason);
    assertNonEmptyString(reason.identifier);
    if (reason.source !== "url-analysis" && reason.source !== "user-context") {
      throw new ContractError();
    }
    assertNonEmptyString(reason.explanation);
  }
  assertArray(assessment.limitations);
  for (const limitation of assessment.limitations) {
    assertRecord(limitation);
    assertNonEmptyString(limitation.identifier);
    assertNonEmptyString(limitation.explanation);
  }
  if (typeof assessment.context_changed_recommendation !== "boolean") {
    throw new ContractError();
  }
}

function validateCompleteness(completeness) {
  assertRecord(completeness);
  if (!OVERALL_STATUSES.has(completeness.status)) {
    throw new ContractError();
  }
  assertArray(completeness.stages);
  if (completeness.stages.length !== REQUIRED_STAGES.size) {
    throw new ContractError();
  }

  const stageIdentifiers = new Set();
  for (const stage of completeness.stages) {
    assertRecord(stage);
    if (!REQUIRED_STAGES.has(stage.identifier) || stageIdentifiers.has(stage.identifier)) {
      throw new ContractError();
    }
    stageIdentifiers.add(stage.identifier);
    if (!STAGE_STATUSES.has(stage.status)) {
      throw new ContractError();
    }
    assertArray(stage.reasons);
    for (const reason of stage.reasons) {
      assertRecord(reason);
      assertNonEmptyString(reason.identifier);
      assertNonEmptyString(reason.explanation);
    }
  }

  if (stageIdentifiers.size !== REQUIRED_STAGES.size) {
    throw new ContractError();
  }
}

function validateError(error) {
  assertRecord(error);
  assertNonEmptyString(error.code);
  assertNonEmptyString(error.message);
}

function assertExactVersion(actual, expected) {
  if (!Number.isSafeInteger(actual) || actual !== expected) {
    throw new ContractError();
  }
}

function assertRecord(value) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new ContractError();
  }
}

function assertArray(value) {
  if (!Array.isArray(value)) {
    throw new ContractError();
  }
}

function assertNonEmptyString(value) {
  if (typeof value !== "string" || value.length === 0) {
    throw new ContractError();
  }
}
