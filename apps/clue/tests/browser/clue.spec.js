import { expect, test } from "@playwright/test";

const SENSITIVE_URL =
  "https://private.example/private/path?token=fake-secret-123&target=https%3A%2F%2Fdestination.example%2Fprivate";
const COMPOUND_EXAMPLE =
  "https://social.example/link/v2?aid=1988&scene=bio_url&target=tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html#synthetic-fragment";

async function openClue(page) {
  const requests = [];
  const externalRequests = [];

  page.on("request", (request) => requests.push(request.url()));
  await page.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    if (url.hostname !== "127.0.0.1" && url.hostname !== "localhost") {
      externalRequests.push(url.href);
      await route.abort();
      return;
    }
    await route.continue();
  });

  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  expect(externalRequests).toEqual([]);
  return { requests, externalRequests };
}

async function analyze(page, url) {
  await page.getByLabel("URL to analyze").fill(url);
  await page.getByRole("button", { name: "Analyze URL" }).click();
}

async function expandSupportingFindings(page) {
  const findings = page.locator("#supporting-findings");
  await expect(findings).not.toHaveAttribute("open", "");
  await findings.locator("summary").click();
  await expect(findings).toHaveAttribute("open", "");
}

test("loads the actual WASM adapter and retains accepted schema versions outside the primary header", async ({
  page,
}) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);

  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeVisible();
  await expect(page.locator(".results-header")).not.toContainText("Wire 1");
  await expect(page.locator(".results-header")).not.toContainText("Report 1");
  await page.getByText("About this result").click();
  await expect(page.locator(".result-about")).toContainText("browser contract 2");
  await expect(page.locator(".result-about")).toContainText("consumer report 1");
  await expect(page.locator(".result-about")).toContainText("assessment contract 1");
  await expect(page.locator("#results")).toHaveAttribute("data-wire-schema-version", "2");
  await expect(page.locator("#results")).toHaveAttribute(
    "data-consumer-schema-version",
    "1",
  );
});

test("places the assessment and visible context questions side by side on desktop", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await openClue(page);
  await analyze(page, SENSITIVE_URL);

  const { assessmentBox, contextBox } = await page.evaluate(() => ({
    assessmentBox: document.querySelector("#assessment").getBoundingClientRect().toJSON(),
    contextBox: document.querySelector(".context-panel").getBoundingClientRect().toJSON(),
  }));
  expect(Math.abs(assessmentBox.y - contextBox.y)).toBeLessThan(2);
  expect(contextBox.x).toBeGreaterThan(assessmentBox.x + assessmentBox.width);
  expect(assessmentBox.width).toBeGreaterThan(contextBox.width);

  const alignedEdges = await page.evaluate(() => {
    const selectors = [".header-inner", ".intro", ".analyzer-panel", "#results", "footer"];
    return selectors.map((selector) => {
      const box = document.querySelector(selector).getBoundingClientRect();
      return { selector, left: box.left, right: box.right };
    });
  });
  for (const edge of alignedEdges.slice(1)) {
    expect(Math.abs(edge.left - alignedEdges[0].left), edge.selector).toBeLessThan(2);
    expect(Math.abs(edge.right - alignedEdges[0].right), edge.selector).toBeLessThan(2);
  }

  await expect(page.getByLabel("Where did the link come from?")).toBeVisible();
  await expect(page.getByLabel("What was it supposed to be for?")).toBeVisible();
  await expect(page.getByRole("button", { name: "Show two more optional questions" })).toBeVisible();
  await expect(page.getByLabel("Were you expecting this contact or request?")).toBeHidden();

  const primaryViewportState = await page.evaluate(() => {
    const action = document.querySelector("#assessment-action").getBoundingClientRect();
    const purpose = document.querySelector("#context-purpose").getBoundingClientRect();
    return {
      actionVisible: action.top >= 0 && action.bottom <= window.innerHeight,
      purposeVisible: purpose.top >= 0 && purpose.bottom <= window.innerHeight,
    };
  });
  expect(primaryViewportState).toEqual({ actionVisible: true, purposeVisible: true });

  const reasonBox = await page.locator(".assessment-reason-section").boundingBox();
  const gridBox = await page.locator(".results-primary-grid").boundingBox();
  expect(Math.abs(reasonBox.x - gridBox.x)).toBeLessThan(2);
  expect(Math.abs(reasonBox.width - gridBox.width)).toBeLessThan(2);
});

test("renders encoded destinations and performs no analysis-triggered requests", async ({
  page,
}) => {
  const network = await openClue(page);
  const assetRequestCount = network.requests.length;

  await analyze(page, SENSITIVE_URL);
  await expect(page.locator("#supporting-findings")).not.toHaveAttribute("open", "");
  await expect(page.locator("#observations-summary")).toHaveText(
    "Clue identified 1 structural observation.",
  );
  await expandSupportingFindings(page);
  await expect(
    page.getByRole("heading", { name: "This link contains another web address" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Verify this link before opening it." }),
  ).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText("Proceed with caution");
  await expect(page.locator("#assessment-action")).toHaveText(
    "Verify this link before opening it.",
  );
  await expect(page.getByRole("heading", { name: "What you should do" })).toBeVisible();
  await expect(page.locator("#assessment-next-steps")).toContainText(
    "Open the service's known website or app yourself",
  );
  await expect(page.getByText("Observed once")).toBeVisible();
  const cardWidth = await page.locator(".observation-card").first().evaluate(
    (element) => element.getBoundingClientRect().width,
  );
  const listWidth = await page.locator("#observations").evaluate(
    (element) => element.getBoundingClientRect().width,
  );
  expect(cardWidth).toBeGreaterThan(listWidth * 0.9);
  await page.waitForTimeout(100);

  expect(network.externalRequests).toEqual([]);
  expect(network.requests.slice(assetRequestCount)).toEqual([]);
  expect(network.requests.some((url) => url.includes("private.example"))).toBe(false);
  expect(network.requests.some((url) => url.includes("destination.example"))).toBe(false);
});

test("shows the explicit non-verdict state when there are no observations", async ({ page }) => {
  await openClue(page);
  await analyze(page, "https://ordinary.example/products?category=books");

  await expect(page.locator("#observations-summary")).toHaveText(
    "Clue identified no supported structural observations.",
  );
  await expect(page.locator("#supporting-findings")).not.toHaveAttribute("open", "");
  await expandSupportingFindings(page);
  await expect(
    page.getByText("No supported structural observations were found."),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "No obvious structural warning signs found." }),
  ).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText(
    "No clear structural warning signs",
  );
  await expect(page.locator("#assessment")).toContainText(
    "This does not establish that the website is trustworthy or safe.",
  );
  await expect(
    page.getByText(
      "This does not mean the link is safe. LinkSleuth only examines supported URL structures.",
    ),
  ).toBeVisible();
  await expect(page.locator("#results")).not.toContainText("Safe");
  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await expect(page.locator("#destination-hostname-list")).toContainText(
    "couldn't identify a complete destination address",
  );
});

test("keeps unsupported original-path analysis separate from completed stages", async ({ page }) => {
  await openClue(page);
  await analyze(page, "https:////private.example/path?category=books");

  await expect(
    page.getByText("Clue could not fully examine every supported structure in this link."),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Clue cannot reliably assess this link." }),
  ).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText("Unable to assess reliably");
  await expect(page.locator(".assessment-category-explanation")).toContainText(
    "incomplete or unavailable",
  );
  await expect(page.locator("#assessment-limitations")).toBeVisible();
  await expect(page.locator("#assessment-limitations")).toContainText(
    "At least one supported analysis stage was incomplete",
  );
  const limitations = page.locator("#coverage-limitations");
  await expect(limitations).toContainText("Original path was not fully examined");
  await expect(limitations).toContainText(
    "The submitted URL spelling did not provide a confidently supported original-path boundary.",
  );
  await page.getByText("What Clue examined").click();
  const stages = page.locator("#completeness-stages");
  await expect(stages).toContainText("Original path");
  await expect(stages).toContainText("Unsupported evidence");
  await expect(stages).toContainText("Percent decoding");
  await expect(stages).toContainText("Examined within supported scope");
});

test("uses fixed errors for empty and invalid input without reflecting sensitive values", async ({
  page,
}) => {
  await openClue(page);
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await expect(page.getByRole("alert")).toHaveText("Enter a URL to analyze.");

  const secret = "fake-secret-invalid-987";
  await analyze(page, `not a URL ${secret}`);
  await expect(page.getByRole("alert")).toHaveText(
    "Enter a complete HTTP or HTTPS URL.",
  );
  await expect(page.getByRole("alert")).not.toContainText(secret);
  await expect(page.locator("#results")).toBeHidden();
});

test("privacy-reduced results and explicit copy workflow exclude raw URL evidence", async ({
  context,
  page,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"], {
    origin: "http://127.0.0.1:4173",
  });
  await openClue(page);
  await analyze(page, SENSITIVE_URL);

  const resultsText = await page.locator("#results").textContent();
  for (const sensitive of [
    "private.example",
    "destination.example",
    "fake-secret-123",
    "/private/path",
  ]) {
    expect(resultsText).not.toContain(sensitive);
  }

  await page.getByRole("button", { name: "Review summary" }).click();
  const summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).toContain("Clue privacy-reduced summary");
  expect(summary).toContain("Assessment contract: 1");
  expect(summary).toContain("Proceed with caution");
  expect(summary).toContain("Recommended action\nVerify this link before opening it.");
  expect(summary).toContain("What you should do");
  expect(summary).toContain("This link contains another web address — observed once");
  expect(summary).toContain("This is structural analysis, not a security verdict.");
  expect(summary).toContain("not guaranteed anonymous");
  for (const sensitive of [
    "private.example",
    "destination.example",
    "fake-secret-123",
    "token",
    "/private/path",
  ]) {
    expect(summary).not.toContain(sensitive);
  }

  await page.getByRole("button", { name: "Copy summary" }).click();
  await expect(page.locator("#copy-status")).toHaveText("Summary copied to the clipboard.");
  const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
  expect(clipboardText).toBe(summary);
});

test("reveals only hostname-level destination evidence after an explicit local action", async ({
  page,
}) => {
  const network = await openClue(page);
  await analyze(page, COMPOUND_EXAMPLE);
  const requestCount = network.requests.length;

  await expect(page.locator("#destination-hostnames")).toBeHidden();
  await expect(page.locator("#destination-hostname-list")).toBeEmpty();
  await expect(page.locator("#destination-details")).toBeEmpty();
  const defaultText = await page.locator("#results").textContent();
  for (const privateValue of [
    "storage.example",
    "tracking.example",
    "fake-id",
    "adurl",
    "synthetic-fragment",
  ]) {
    expect(defaultText).not.toContain(privateValue);
  }

  const reveal = page.locator("#toggle-destination-hostnames");
  await reveal.click();
  await expect(reveal).toHaveAttribute("aria-expanded", "true");
  await expect(reveal).toHaveText("Hide destination hostnames");
  await expect(page.locator("#destination-hostname-list")).toContainText("storage.example");
  await expect(page.locator("#destination-hostname-list")).toContainText(
    "Protocol-relative reference; no scheme was verified",
  );
  await expect(page.locator("#destination-ambiguities")).toContainText(
    "couldn't identify it confidently",
  );
  const hostnameText = await page.locator("#destination-hostnames").textContent();
  for (const privateValue of ["tracking.example", "fake-id", "adurl", "synthetic-fragment"]) {
    expect(hostnameText).not.toContain(privateValue);
  }
  await expect(page.locator("#destination-details")).toBeHidden();
  await expect(page.locator("#destination-details")).toBeEmpty();
  await expect(page.locator("#destination-hostnames a")).toHaveCount(0);
  await expect(page.locator("#destination-hostnames [href], #destination-hostnames [src]")).toHaveCount(0);
  await page.waitForTimeout(100);
  expect(network.externalRequests).toEqual([]);
  expect(network.requests.slice(requestCount)).toEqual([]);
});

test("requires a second action for detailed evidence and removes sensitive nodes when hidden", async ({
  page,
}) => {
  const url =
    "https://outer.example/?email=fake.person%40example.test&target=" +
    "https%3A%2F%2Fdestination.example.%2Fprivate%3Ftoken%3Dfake-token%23private-fragment";
  await openClue(page);
  await analyze(page, url);

  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await expect(page.locator("#destination-hostname-list")).toContainText(
    "destination.example.",
  );
  await expect(page.locator("#destination-hostnames")).not.toContainText("fake-token");
  await expect(page.locator("#destination-hostnames")).not.toContainText("fake.person");
  await expect(page.locator("#destination-details")).toBeEmpty();

  const detailButton = page.locator("#toggle-destination-details");
  await detailButton.click();
  await expect(detailButton).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("#destination-details")).toContainText("fake-token");
  await expect(page.locator("#destination-details")).toContainText("Source parameter");
  await expect(page.locator("#destination-details")).toContainText("target");
  await expect(page.locator("#destination-details")).toContainText(
    "Classified submitted representation (credentials and fragment omitted)",
  );
  await expect(page.locator("#destination-details")).not.toContainText("private-fragment");
  await expect(page.locator("#destination-details a")).toHaveCount(0);
  await expect(page.locator("#destination-details [href], #destination-details [src]")).toHaveCount(0);

  await page.getByRole("button", { name: "Hide detailed evidence" }).click();
  await expect(page.locator("#destination-details")).toBeHidden();
  await expect(page.locator("#destination-details")).toBeEmpty();
  await expect(page.locator("#results")).not.toContainText("fake-token");

  await page.getByRole("button", { name: "Hide destination hostnames" }).click();
  await expect(page.locator("#destination-hostnames")).toBeHidden();
  await expect(page.locator("#destination-hostname-list")).toBeEmpty();
  await expect(page.locator("#results")).not.toContainText("destination.example.");
});

test("disclosure never changes summaries and resets for a new analysis", async ({ page }) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);
  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await page.getByRole("button", { name: "Reveal detailed evidence" }).click();
  await expect(page.locator("#destination-details")).toContainText("destination.example");

  await page.getByRole("button", { name: "Review summary" }).click();
  const summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).not.toContain("destination.example");
  expect(summary).not.toContain("fake-secret-123");
  await page.getByRole("button", { name: "Close summary" }).click();

  await analyze(page, "https://ordinary.example/products?category=books");
  await expect(page.getByRole("button", { name: "Reveal destination hostnames" })).toHaveAttribute(
    "aria-expanded",
    "false",
  );
  await expect(page.locator("#destination-hostnames")).toBeHidden();
  await expect(page.locator("#destination-hostname-list")).toBeEmpty();
  await expect(page.locator("#destination-details")).toBeEmpty();
  await expect(page.locator("#results")).not.toContainText("destination.example");
});

test("context reassessment preserves the closed disclosure boundary", async ({ page }) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("document");

  await expect(page.locator("#destination-hostnames")).toBeHidden();
  await expect(page.locator("#destination-hostname-list")).toBeEmpty();
  await expect(page.locator("#destination-details")).toBeEmpty();
  await expect(page.locator("#results")).not.toContainText("destination.example");
});

test("supports keyboard submission and accessible summary controls", async ({ page }) => {
  await openClue(page);
  const input = page.getByLabel("URL to analyze");
  await input.fill(
    "https://keyboard.example/?target=https%3A%2F%2Fcandidate.example%2F",
  );
  await input.press("Enter");

  const hostnameDisclosure = page.locator("#toggle-destination-hostnames");
  await hostnameDisclosure.focus();
  await page.keyboard.press("Enter");
  await expect(hostnameDisclosure).toHaveAttribute("aria-expanded", "true");
  const detailedDisclosure = page.locator("#toggle-destination-details");
  await detailedDisclosure.focus();
  await page.keyboard.press("Enter");
  await expect(detailedDisclosure).toHaveAttribute("aria-expanded", "true");

  const findingsSummary = page.locator("#supporting-findings > summary");
  await findingsSummary.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("#supporting-findings")).toHaveAttribute("open", "");
  await expect(
    page.getByRole("heading", { name: "This link contains another web address" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Review summary" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { name: "Review before copying" })).toBeFocused();
  await expect(page.getByRole("button", { name: "Copy summary" })).toBeVisible();
  await page.getByRole("button", { name: "Close summary" }).click();
  await expect(page.getByRole("button", { name: "Review summary" })).toBeFocused();
});

test("shows a fixed local error when the WASM asset cannot initialize", async ({ page }) => {
  await page.route("**/*.wasm", (route) => route.abort());
  await page.goto("/");
  await expect(page.getByText("Local analyzer unavailable")).toBeVisible();

  await page.getByLabel("URL to analyze").fill("https://private.example/fake-secret");
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await expect(page.getByRole("alert")).toHaveText(
    "Clue could not start the local analyzer. Refresh the page and try again.",
  );
  await expect(page.getByRole("alert")).not.toContainText("private.example");
  await expect(page.getByRole("alert")).not.toContainText("fake-secret");
});

test("explains the motivating compound structure without disclosing its evidence or making requests", async ({
  page,
}) => {
  const network = await openClue(page);
  const assetRequestCount = network.requests.length;
  await analyze(page, COMPOUND_EXAMPLE);
  await expect(page.locator("#supporting-findings")).not.toHaveAttribute("open", "");
  await expect(page.locator("#observations-summary")).toHaveText(
    "Clue identified 6 structural observations.",
  );
  await expandSupportingFindings(page);
  await expect(page.locator(".observation-card")).toHaveCount(6);
  for (const card of await page.locator(".observation-card").all()) {
    await expect(card).toBeVisible();
  }

  for (const title of [
    "This link contains a web address without a stated scheme",
    "Encoded query separators were found",
    "A parameter contains its own query structure",
    "A hostname-like reference could not be classified confidently",
  ]) {
    await expect(page.getByRole("heading", { name: title })).toBeVisible();
  }
  await expect(
    page.getByRole("heading", { name: "Verify this link before opening it." }),
  ).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText("Proceed with caution");
  await expect(page.locator("#assessment-reasons")).toHaveAttribute("data-layout", "full");
  const cautionStyle = await page.locator("#assessment").evaluate((element) => ({
    border: getComputedStyle(element).borderLeftColor,
    background: getComputedStyle(element).backgroundColor,
  }));
  const preliminaryReasonsWidth = await page.locator(".assessment-reason-group").first().evaluate(
    (element) => element.getBoundingClientRect().width,
  );
  const preliminaryReasonAreaWidth = await page.locator("#assessment-reasons").evaluate(
    (element) => element.getBoundingClientRect().width,
  );
  expect(preliminaryReasonsWidth).toBeGreaterThan(preliminaryReasonAreaWidth * 0.9);
  await expect(page.locator("#coverage-limitations")).toContainText(
    "Destination traversal was not fully examined",
  );
  const resultText = await page.locator("#results").textContent();
  for (const sensitive of [
    "social.example",
    "tracking.example",
    "storage.example",
    "fake-id",
    "adurl",
    "synthetic-fragment",
    "destination.protocol-relative-candidate",
  ]) {
    expect(resultText).not.toContain(sensitive);
  }
  await expect(page.getByLabel("Where did the link come from?")).toBeVisible();
  await expect(page.getByLabel("What was it supposed to be for?")).toBeVisible();
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("document");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
  await page
    .getByLabel("Were you expecting this contact or request?")
    .selectOption("unexpected");
  await expect(page.getByRole("heading", { name: "Do not open this link" })).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText("Strong warning signs");
  const strongStyle = await page.locator("#assessment").evaluate((element) => ({
    border: getComputedStyle(element).borderLeftColor,
    background: getComputedStyle(element).backgroundColor,
  }));
  expect(strongStyle.border).not.toBe(cautionStyle.border);
  expect(strongStyle.background).not.toBe(cautionStyle.background);
  expect(strongStyle.border).toBe("rgb(161, 40, 40)");
  await expect(page.locator("#assessment-explanation")).toContainText(
    "warning signs consistent with phishing",
  );
  await expect(page.locator("#assessment-next-steps")).toContainText(
    "Contact the supposed sender through a trusted channel before opening or signing anything.",
  );
  await expect(page.locator("#assessment-next-steps")).toContainText(
    "Do not rely on a reply to the same message",
  );
  await expect(page.locator("#assessment-change")).toContainText(
    "changed Clue's assessment from 'Proceed with caution' to 'Strong warning signs'",
  );
  await expect(page.locator("#assessment-change")).toContainText(
    "unexpected document-signing request contributed",
  );
  await expect(page.locator("#assessment-reasons")).toHaveAttribute("data-layout", "split");
  const contextReasonWidths = await page.locator(".assessment-reason-group").evaluateAll(
    (elements) => elements.map((element) => element.getBoundingClientRect().width),
  );
  expect(contextReasonWidths).toHaveLength(2);
  expect(Math.abs(contextReasonWidths[0] - contextReasonWidths[1])).toBeLessThan(2);
  expect(contextReasonWidths[0]).toBeGreaterThan(preliminaryReasonAreaWidth * 0.4);
  await expect(page.locator("#assessment-limitations")).toContainText(
    "User context cannot resolve the destination-like structure",
  );
  await page.waitForTimeout(100);
  expect(network.externalRequests).toEqual([]);
  expect(network.requests.slice(assetRequestCount)).toEqual([]);
});

test("offers optional progressive context and updates guidance without changing URL evidence", async ({
  page,
}) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);

  const observationsBefore = await page.locator("#observations").textContent();
  await expect(page.locator("#assessment-status-label")).toHaveText(
    "Preliminary assessment",
  );
  await expect(page.getByLabel("Where did the link come from?")).toBeVisible();
  await expect(page.getByLabel("What was it supposed to be for?")).toBeVisible();
  await expect(page.getByLabel("Were you expecting this contact or request?")).toBeHidden();

  const sourceControl = page.getByLabel("Where did the link come from?");
  await sourceControl.scrollIntoViewIfNeeded();
  await sourceControl.focus();
  const scrollBeforeUpdate = await page.evaluate(() => window.scrollY);
  await sourceControl.selectOption("email");
  await expect(sourceControl).toBeFocused();
  expect(Math.abs((await page.evaluate(() => window.scrollY)) - scrollBeforeUpdate)).toBeLessThan(2);
  await expect(page.getByText("Assessment with optional context")).toBeVisible();
  await expect(page.locator("#context-status")).toContainText(
    "The guidance did not change.",
  );

  await page.getByLabel("What was it supposed to be for?").selectOption("document");
  await expect(sourceControl).toHaveValue("email");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
  await page
    .getByLabel("Were you expecting this contact or request?")
    .selectOption("unexpected");
  await page
    .getByLabel("What did the message or page ask you to do?")
    .selectOption("enter-account-credentials");

  await expect(page.getByRole("heading", { name: "Do not open this link" })).toBeVisible();
  await expect(page.locator("#assessment-heading")).toHaveText("Strong warning signs");
  await expect(page.locator("#assessment-next-steps")).toContainText(
    "Do not enter your password through this link",
  );
  await expect(page.getByText("From your optional answers")).toBeVisible();
  await expect(page.locator("#assessment-change")).toContainText(
    "changed Clue's assessment",
  );
  await expect(page.locator("#observations")).toHaveText(observationsBefore);

  await page
    .getByLabel("Were you expecting this contact or request?")
    .selectOption("expected");
  await expect(page.locator("#assessment-heading")).toHaveText("Proceed with caution");
  await expect(page.locator("#assessment-action")).toContainText(
    "Do not continue until this request is verified.",
  );

  await page.getByRole("button", { name: "Clear answers" }).click();
  await expect(page.locator("#assessment-status-label")).toHaveText(
    "Preliminary assessment",
  );
  await expect(page.locator("#assessment-action")).toHaveText(
    "Verify this link before opening it.",
  );
  await expect(page.getByLabel("Where did the link come from?")).toHaveValue("");
  await expect(page.getByLabel("What was it supposed to be for?")).toHaveValue("");
  await expect(page.getByLabel("Were you expecting this contact or request?")).toHaveValue("");
  await expect(page.locator("#observations")).toHaveText(observationsBefore);
});

test("context reassessment and context editing trigger no network requests", async ({ page }) => {
  const network = await openClue(page);
  await analyze(page, SENSITIVE_URL);
  const requestCount = network.requests.length;

  await page.getByLabel("Where did the link come from?").selectOption("message");
  await page.getByLabel("What was it supposed to be for?").selectOption("payment-or-financial-request");
  await page.waitForTimeout(100);

  expect(network.externalRequests).toEqual([]);
  expect(network.requests.slice(requestCount)).toEqual([]);
});

test("summary excludes optional context reasons until deliberate inclusion", async ({
  context,
  page,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"], {
    origin: "http://127.0.0.1:4173",
  });
  await openClue(page);
  await analyze(page, "https://ordinary.example/account");
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("account-sign-in-or-verification");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
  await page.getByLabel("Were you expecting this contact or request?").selectOption("unexpected");

  await page.getByRole("button", { name: "Review summary" }).click();
  let summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).toContain("Optional user-context reasons were excluded");
  expect(summary).not.toContain("You reported that the contact or request was unexpected");

  await page
    .getByLabel("Include standardized context-derived reasons in this summary")
    .check();
  summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).toContain("You reported that the contact or request was unexpected");
  expect(summary).not.toContain("ordinary.example");
});

test("coverage details expand and collapse with the keyboard", async ({ page }) => {
  await openClue(page);
  await analyze(page, "https://ordinary.example/products?category=books");

  const summary = page.getByText("What Clue examined");
  await summary.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("#coverage-details")).toHaveAttribute("open", "");
  await expect(page.locator("#completeness-stages")).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.locator("#coverage-details")).not.toHaveAttribute("open", "");
});

test("repeated analysis replaces observations and keeps result focus", async ({ page }) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);
  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeFocused();
  await analyze(page, "https://ordinary.example/products?category=books");
  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeFocused();
  await expect(
    page.getByRole("heading", { name: "This link contains another web address" }),
  ).toHaveCount(0);
  await expandSupportingFindings(page);
  await expect(
    page.getByText("No supported structural observations were found."),
  ).toBeVisible();
});
