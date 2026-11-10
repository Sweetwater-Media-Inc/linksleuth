import assert from "node:assert/strict";
import test from "node:test";

import {
  ContractError,
  EXPECTED_ASSESSMENT_CONTRACT_VERSION,
  EXPECTED_CONSUMER_SCHEMA_VERSION,
  EXPECTED_WIRE_SCHEMA_VERSION,
  validateBrowserResponse,
} from "../src/contract.js";

function validResponse() {
  return {
    wire_schema_version: EXPECTED_WIRE_SCHEMA_VERSION,
    report: {
      consumer_schema_version: EXPECTED_CONSUMER_SCHEMA_VERSION,
      disclosure: "privacy-reduced",
      observations: [
        {
          identifier: "destination.absolute-candidate",
          title: "Potential embedded destination",
          explanation: "Fixed explanation.",
          occurrence_count: 1,
          evidence_references: [],
        },
      ],
      completeness: {
        status: "complete-within-supported-scope",
        stages: [
          "original-path",
          "percent-decoding",
          "destination-traversal",
          "secondary-encoding",
        ].map((identifier) => ({
          identifier,
          status: "complete-within-supported-scope",
          reasons: [],
        })),
      },
      assessment: {
        contract_version: EXPECTED_ASSESSMENT_CONTRACT_VERSION,
        category_identifier: "proceed-with-caution",
        category_title: "Proceed with caution",
        category_explanation: "Fixed explanation.",
        action_identifier: "verify-independently",
        action_title: "Verify independently.",
        action_explanation: "Fixed recommendation explanation.",
        next_steps: [
          {
            identifier: "verify-request-independently",
            explanation: "Fixed next step.",
          },
        ],
        reasons: [
          {
            identifier: "url.example-reason",
            source: "url-analysis",
            explanation: "Fixed reason.",
          },
        ],
        limitations: [
          {
            identifier: "assessment.example-limitation",
            explanation: "Fixed limitation.",
          },
        ],
        context_changed_recommendation: false,
      },
    },
    error: null,
  };
}

test("accepts the documented success and error envelopes", () => {
  const success = validResponse();
  assert.equal(validateBrowserResponse(success), success);

  const failure = {
    wire_schema_version: 2,
    report: null,
    error: { code: "invalid-url", message: "Enter a complete HTTP or HTTPS URL." },
  };
  assert.equal(validateBrowserResponse(failure), failure);
});

test("rejects unknown wire and consumer schema versions", () => {
  const unknownWire = validResponse();
  unknownWire.wire_schema_version = 3;
  assert.throws(() => validateBrowserResponse(unknownWire), ContractError);

  const unknownConsumer = validResponse();
  unknownConsumer.report.consumer_schema_version = 2;
  assert.throws(() => validateBrowserResponse(unknownConsumer), ContractError);

  const unknownAssessment = validResponse();
  unknownAssessment.report.assessment.contract_version = 2;
  assert.throws(() => validateBrowserResponse(unknownAssessment), ContractError);
});

test("rejects missing fields, evidence disclosure, and incomplete stage sets", () => {
  const missingTitle = validResponse();
  delete missingTitle.report.observations[0].title;
  assert.throws(() => validateBrowserResponse(missingTitle), ContractError);

  const disclosedEvidence = validResponse();
  disclosedEvidence.report.observations[0].evidence_references.push("candidate-1");
  assert.throws(() => validateBrowserResponse(disclosedEvidence), ContractError);

  const missingStage = validResponse();
  missingStage.report.completeness.stages.pop();
  assert.throws(() => validateBrowserResponse(missingStage), ContractError);

  const missingAssessment = validResponse();
  delete missingAssessment.report.assessment;
  assert.throws(() => validateBrowserResponse(missingAssessment), ContractError);

  const rawContextSource = validResponse();
  rawContextSource.report.assessment.reasons[0].source = "private-email-value";
  assert.throws(() => validateBrowserResponse(rawContextSource), ContractError);

  const missingNextSteps = validResponse();
  missingNextSteps.report.assessment.next_steps = [];
  assert.throws(() => validateBrowserResponse(missingNextSteps), ContractError);

  const tooManyNextSteps = validResponse();
  tooManyNextSteps.report.assessment.next_steps = Array.from({ length: 4 }, (_, index) => ({
    identifier: `step-${index}`,
    explanation: "Fixed next step.",
  }));
  assert.throws(() => validateBrowserResponse(tooManyNextSteps), ContractError);

  const duplicateNextSteps = validResponse();
  duplicateNextSteps.report.assessment.next_steps.push({
    ...duplicateNextSteps.report.assessment.next_steps[0],
  });
  assert.throws(() => validateBrowserResponse(duplicateNextSteps), ContractError);
});
