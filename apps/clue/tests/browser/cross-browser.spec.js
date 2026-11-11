import { expect, test } from "@playwright/test";

const MOTIVATING_URL =
  "https://social.example/link/v2?aid=1988&scene=bio_url&target=tracking.example./%2Fpcs/click?fake-id%26amp%3B%26amp%3Badurl=%2F%2F%73%74%6F%72%61%67%65%2E%65%78%61%6D%70%6C%65%2Flanding.html#synthetic-fragment";

test("actual WASM supports the complete controlled-disclosure workflow", async ({ page }) => {
  const requests = [];
  const externalRequests = [];
  page.on("request", (request) => requests.push(request.url()));
  await page.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) {
      externalRequests.push(url.href);
      await route.abort();
      return;
    }
    await route.continue();
  });

  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  const requestsAfterLoad = requests.length;

  await page.getByLabel("URL to analyze").fill("https://ordinary.example/content");
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await expect(page.locator("#assessment-heading")).toHaveText(
    "No clear structural warning signs",
  );

  await page.getByLabel("URL to analyze").fill(MOTIVATING_URL);
  await page.getByRole("button", { name: "Analyze URL" }).click();

  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeFocused();
  await expect(page.locator("#assessment-heading")).toHaveText("Proceed with caution");
  await expect(page.locator("#supporting-findings")).not.toHaveAttribute("open", "");

  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("document");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
  await page
    .getByLabel("Were you expecting this contact or request?")
    .selectOption("unexpected");
  await expect(page.locator("#assessment-heading")).toHaveText("Strong warning signs");
  await expect(page.getByRole("heading", { name: "Do not open this link" })).toBeVisible();

  await page.getByRole("button", { name: "Clear answers" }).click();
  await expect(page.locator("#assessment-heading")).toHaveText("Proceed with caution");
  await expect(page.locator("#assessment-status-label")).toHaveText("Preliminary assessment");

  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  const card = page.locator(".destination-card").first();
  await expect(card).toContainText("Possible destination found");
  await expect(card).toContainText("storage.example");
  await expect(card).toContainText("Clue identified this address inside the original link");
  await expect(card.getByText("candidate-1", { exact: true })).toBeHidden();
  await card.locator(".destination-technical > summary").click();
  await expect(card).toContainText("Candidate identifier");

  await page.getByRole("button", { name: "Reveal detailed evidence" }).click();
  await expect(page.locator("#destination-details")).toContainText("Source parameter");
  await page.getByRole("button", { name: "Hide detailed evidence" }).click();
  await expect(page.locator("#destination-details")).toBeEmpty();

  await page.getByRole("button", { name: "Review summary" }).click();
  await expect(page.getByRole("heading", { name: "Review before copying" })).toBeFocused();
  const summary = await page.getByLabel("Selectable summary text").inputValue();
  expect(summary).toContain("Clue privacy-reduced summary");
  expect(summary).not.toContain("storage.example");
  expect(summary).not.toContain("fake-id");

  await page.waitForTimeout(50);
  expect(externalRequests).toEqual([]);
  expect(requests.slice(requestsAfterLoad)).toEqual([]);
});
