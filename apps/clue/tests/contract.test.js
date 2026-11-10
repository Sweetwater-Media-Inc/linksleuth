import assert from "node:assert/strict";
import test from "node:test";

import {
  ContractError,
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
    },
    error: null,
  };
}

test("accepts the documented success and error envelopes", () => {
  const success = validResponse();
  assert.equal(validateBrowserResponse(success), success);

  const failure = {
    wire_schema_version: 1,
    report: null,
    error: { code: "invalid-url", message: "Enter a complete HTTP or HTTPS URL." },
  };
  assert.equal(validateBrowserResponse(failure), failure);
});

test("rejects unknown wire and consumer schema versions", () => {
  const unknownWire = validResponse();
  unknownWire.wire_schema_version = 2;
  assert.throws(() => validateBrowserResponse(unknownWire), ContractError);

  const unknownConsumer = validResponse();
  unknownConsumer.report.consumer_schema_version = 2;
  assert.throws(() => validateBrowserResponse(unknownConsumer), ContractError);
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
});
