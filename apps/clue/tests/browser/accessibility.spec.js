import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const DESTINATION_URL =
  "https://outer.example/?target=https%3A%2F%2Fdestination.example%2Fprivate%3Ftoken%3Dfabricated-token";

async function openClue(page) {
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
}

async function analyze(page, url) {
  await page.getByLabel("URL to analyze").fill(url);
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeVisible();
}

async function expectNoAxeViolations(page) {
  const result = await new AxeBuilder({ page }).analyze();
  expect(result.violations).toEqual([]);
}

test("initial analyzer has no automated accessibility violations", async ({ page }) => {
  await openClue(page);
  await expectNoAxeViolations(page);
});

test("preliminary result has no automated accessibility violations", async ({ page }) => {
  await openClue(page);
  await analyze(page, "https://ordinary.example/products?category=books");
  await expectNoAxeViolations(page);
});

test("strong-warning context has no automated accessibility violations", async ({ page }) => {
  await openClue(page);
  await analyze(page, DESTINATION_URL);
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("account-sign-in-or-verification");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
  await page.getByLabel("Were you expecting this contact or request?").selectOption("unexpected");
  await page
    .getByLabel("What did the message or page ask you to do?")
    .selectOption("enter-account-credentials");
  await expect(page.locator("#assessment-heading")).toHaveText("Strong warning signs");
  await expectNoAxeViolations(page);
});

test("hostname disclosure has no automated accessibility violations", async ({ page }) => {
  await openClue(page);
  await analyze(page, DESTINATION_URL);
  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await expect(page.locator(".destination-card")).toBeVisible();
  await expectNoAxeViolations(page);
});

test("expanded technical and detailed disclosure has no automated accessibility violations", async ({
  page,
}) => {
  await openClue(page);
  await analyze(page, DESTINATION_URL);
  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await page.locator(".destination-technical > summary").click();
  await page.getByRole("button", { name: "Reveal detailed evidence" }).click();
  await expect(page.locator("#destination-details")).toBeVisible();
  await expectNoAxeViolations(page);
});

test("mobile stacked results have no automated accessibility violations", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openClue(page);
  await analyze(page, DESTINATION_URL);
  await expectNoAxeViolations(page);
});
