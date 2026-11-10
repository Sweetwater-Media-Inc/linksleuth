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

test("loads the actual WASM adapter and retains accepted schema versions outside the primary header", async ({
  page,
}) => {
  await openClue(page);
  await analyze(page, SENSITIVE_URL);

  await expect(page.getByRole("heading", { name: "What Clue observed" })).toBeVisible();
  await expect(page.locator(".results-header")).not.toContainText("Wire 1");
  await expect(page.locator(".results-header")).not.toContainText("Report 1");
  await page.getByText("About this result").click();
  await expect(page.locator(".result-about")).toContainText("browser contract 1");
  await expect(page.locator(".result-about")).toContainText("consumer report 1");
  await expect(page.locator("#results")).toHaveAttribute("data-wire-schema-version", "1");
  await expect(page.locator("#results")).toHaveAttribute(
    "data-consumer-schema-version",
    "1",
  );
});

test("renders encoded destinations and performs no analysis-triggered requests", async ({
  page,
}) => {
  const network = await openClue(page);
  const assetRequestCount = network.requests.length;

  await analyze(page, SENSITIVE_URL);
  await expect(
    page.getByRole("heading", { name: "This link contains another web address" }),
  ).toBeVisible();
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

  await expect(
    page.getByText("No supported structural observations were found."),
  ).toBeVisible();
  await expect(
    page.getByText(
      "This does not mean the link is safe. LinkSleuth only examines supported URL structures.",
    ),
  ).toBeVisible();
  await expect(page.locator("#results")).not.toContainText("Safe");
});

test("keeps unsupported original-path analysis separate from completed stages", async ({ page }) => {
  await openClue(page);
  await analyze(page, "https:////private.example/path?category=books");

  await expect(
    page.getByText("Clue could not fully examine every supported structure in this link."),
  ).toBeVisible();
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
    "token",
    "/private/path",
  ]) {
    expect(resultsText).not.toContain(sensitive);
  }

  await page.getByRole("button", { name: "Review summary" }).click();
  const summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).toContain("Clue privacy-reduced summary");
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
  await expect(page.getByRole("status")).toHaveText("Summary copied to the clipboard.");
  const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
  expect(clipboardText).toBe(summary);
});

test("supports keyboard submission and accessible summary controls", async ({ page }) => {
  await openClue(page);
  const input = page.getByLabel("URL to analyze");
  await input.fill(
    "https://keyboard.example/?target=https%3A%2F%2Fcandidate.example%2F",
  );
  await input.press("Enter");

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

  for (const title of [
    "This link contains a web address without a stated scheme",
    "Encoded query separators were found",
    "A parameter contains its own query structure",
    "A hostname-like reference could not be classified confidently",
  ]) {
    await expect(page.getByRole("heading", { name: title })).toBeVisible();
  }
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
  await page.waitForTimeout(100);
  expect(network.externalRequests).toEqual([]);
  expect(network.requests.slice(assetRequestCount)).toEqual([]);
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
  await expect(page.getByRole("heading", { name: "What Clue observed" })).toBeFocused();
  await analyze(page, "https://ordinary.example/products?category=books");
  await expect(page.getByRole("heading", { name: "What Clue observed" })).toBeFocused();
  await expect(
    page.getByRole("heading", { name: "This link contains another web address" }),
  ).toHaveCount(0);
  await expect(
    page.getByText("No supported structural observations were found."),
  ).toBeVisible();
});
