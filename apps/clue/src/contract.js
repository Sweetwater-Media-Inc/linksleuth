export const EXPECTED_WIRE_SCHEMA_VERSION = 2;
export const EXPECTED_CONSUMER_SCHEMA_VERSION = 1;
export const EXPECTED_ASSESSMENT_CONTRACT_VERSION = 1;
export const EXPECTED_DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION = 1;
export const EXPECTED_DESTINATION_DISCLOSURE_CONTRACT_VERSION = 1;

const DISCLOSURE_LEVELS = new Set(["hostnames", "detailed"]);
const DESTINATION_KINDS = new Set([
  "absolute-http",
  "absolute-https",
  "protocol-relative",
  "hostname-first",
]);
const DESTINATION_RELATIONSHIPS = new Set(["direct", "nested"]);

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

export function validateDestinationDisclosureResponse(value, expectedLevel) {
  if (!DISCLOSURE_LEVELS.has(expectedLevel)) {
    throw new ContractError();
  }
  assertRecordWithKeys(value, [
    "destination_disclosure_wire_schema_version",
    "disclosure",
    "error",
  ]);
  assertExactVersion(
    value.destination_disclosure_wire_schema_version,
    EXPECTED_DESTINATION_DISCLOSURE_WIRE_SCHEMA_VERSION,
  );

  const hasDisclosure = value.disclosure !== null;
  const hasError = value.error !== null;
  if (hasDisclosure === hasError) {
    throw new ContractError();
  }
  if (hasError) {
    assertRecordWithKeys(value.error, ["code", "message"]);
    assertNonEmptyString(value.error.code);
    assertNonEmptyString(value.error.message);
    return value;
  }

  validateDestinationDisclosure(value.disclosure, expectedLevel);
  return value;
}

function validateDestinationDisclosure(disclosure, expectedLevel) {
  assertRecordWithKeys(disclosure, [
    "contract_version",
    "level",
    "notice",
    "candidates",
    "ambiguities",
  ]);
  assertExactVersion(
    disclosure.contract_version,
    EXPECTED_DESTINATION_DISCLOSURE_CONTRACT_VERSION,
  );
  if (disclosure.level !== expectedLevel) {
    throw new ContractError();
  }
  assertNonEmptyString(disclosure.notice);
  assertArray(disclosure.candidates);
  assertArray(disclosure.ambiguities);
  if (disclosure.candidates.length > 64 || disclosure.ambiguities.length > 64) {
    throw new ContractError();
  }

  const candidateIdentifiers = new Set();
  for (const candidate of disclosure.candidates) {
    const keys = [
      "identifier",
      "hostname",
      "reference_kind",
      "relationship",
      "depth",
      "parent_candidate_identifier",
    ];
    if (expectedLevel === "detailed") {
      keys.push("detail");
    }
    assertRecordWithKeys(candidate, keys);
    assertNonEmptyString(candidate.identifier);
    assertNonEmptyString(candidate.hostname);
    if (
      candidateIdentifiers.has(candidate.identifier) ||
      !DESTINATION_KINDS.has(candidate.reference_kind) ||
      !DESTINATION_RELATIONSHIPS.has(candidate.relationship) ||
      !Number.isSafeInteger(candidate.depth) ||
      candidate.depth < 0 ||
      candidate.depth > 3 ||
      candidate.relationship !== (candidate.depth === 0 ? "direct" : "nested")
    ) {
      throw new ContractError();
    }
    candidateIdentifiers.add(candidate.identifier);
    assertNullableString(candidate.parent_candidate_identifier);
    if (expectedLevel === "detailed") {
      validateDestinationDetail(candidate.detail);
    }
  }

  for (const ambiguity of disclosure.ambiguities) {
    const keys = [
      "identifier",
      "explanation",
      "relationship",
      "depth",
      "parent_candidate_identifier",
    ];
    if (expectedLevel === "detailed") {
      keys.push("top_level_parameter_position", "source_parameter_position");
    }
    assertRecordWithKeys(ambiguity, keys);
    assertNonEmptyString(ambiguity.identifier);
    assertNonEmptyString(ambiguity.explanation);
    if (
      !DESTINATION_RELATIONSHIPS.has(ambiguity.relationship) ||
      !Number.isSafeInteger(ambiguity.depth) ||
      ambiguity.depth < 0 ||
      ambiguity.depth > 3 ||
      ambiguity.relationship !== (ambiguity.depth === 0 ? "direct" : "nested")
    ) {
      throw new ContractError();
    }
    assertNullableString(ambiguity.parent_candidate_identifier);
    if (expectedLevel === "detailed") {
      assertPositiveInteger(ambiguity.top_level_parameter_position);
      assertPositiveInteger(ambiguity.source_parameter_position);
    }
  }
}

function validateDestinationDetail(detail) {
  assertRecordWithKeys(detail, [
    "submitted_reference",
    "normalized_reference",
    "source_parameter",
    "top_level_parameter_position",
    "source_parameter_position",
    "transformations",
  ]);
  assertNonEmptyString(detail.submitted_reference);
  assertNonEmptyString(detail.normalized_reference);
  assertString(detail.source_parameter);
  assertPositiveInteger(detail.top_level_parameter_position);
  assertPositiveInteger(detail.source_parameter_position);
  assertArray(detail.transformations);
  if (detail.transformations.length > 8) {
    throw new ContractError();
  }
  for (const transformation of detail.transformations) {
    assertRecordWithKeys(transformation, ["identifier", "explanation"]);
    assertNonEmptyString(transformation.identifier);
    assertNonEmptyString(transformation.explanation);
  }
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
    "action_explanation",
  ]) {
    assertNonEmptyString(assessment[field]);
  }
  assertArray(assessment.next_steps);
  if (assessment.next_steps.length === 0 || assessment.next_steps.length > 3) {
    throw new ContractError();
  }
  const nextStepIdentifiers = new Set();
  for (const step of assessment.next_steps) {
    assertRecord(step);
    assertNonEmptyString(step.identifier);
    assertNonEmptyString(step.explanation);
    if (nextStepIdentifiers.has(step.identifier)) {
      throw new ContractError();
    }
    nextStepIdentifiers.add(step.identifier);
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

function assertRecordWithKeys(value, expectedKeys) {
  assertRecord(value);
  const actualKeys = Object.keys(value).sort();
  const sortedExpected = [...expectedKeys].sort();
  if (
    actualKeys.length !== sortedExpected.length ||
    actualKeys.some((key, index) => key !== sortedExpected[index])
  ) {
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

function assertString(value) {
  if (typeof value !== "string") {
    throw new ContractError();
  }
}

function assertNullableString(value) {
  if (value !== null && (typeof value !== "string" || value.length === 0)) {
    throw new ContractError();
  }
}

function assertPositiveInteger(value) {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new ContractError();
  }
}
