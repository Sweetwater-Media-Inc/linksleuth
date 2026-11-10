import { expect, test } from "@playwright/test";

test("remains usable at a mobile viewport without horizontal page overflow", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Understand a link before you click it." })).toBeVisible();

  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);

  await page.getByLabel("URL to analyze").fill("https://mobile.example/products");
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await expect(page.getByRole("heading", { name: "Clue's assessment" })).toBeVisible();
  await expect(page.getByText("No supported structural observations were found.")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "No obvious structural warning signs found." }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Answer optional questions" }).click();
  await page.getByRole("button", { name: "Next question" }).first().click();

  const resultDimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(resultDimensions.scrollWidth).toBeLessThanOrEqual(resultDimensions.clientWidth);
});

test("stacks URL and context reason groups at a narrow viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await page
    .getByLabel("URL to analyze")
    .fill(
      "https://top.example/?target=tracking.example%2F%2Fpath%3Fadurl%3D%2F%2Fstorage.example%2Flanding",
    );
  await page.getByRole("button", { name: "Analyze URL" }).click();
  await page.getByRole("button", { name: "Answer optional questions" }).click();
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByRole("button", { name: "Next question" }).first().click();
  await page.getByLabel("What was the link supposed to be for?").selectOption("document");
  await page.getByRole("button", { name: "Next question" }).nth(1).click();
  await page
    .getByLabel("Were you expecting this contact or request?")
    .selectOption("unexpected");

  await expect(page.getByRole("heading", { name: "Do not open this link" })).toBeVisible();
  const reasonBoxes = await page.locator(".assessment-reason-group").evaluateAll((elements) =>
    elements.map((element) => {
      const box = element.getBoundingClientRect();
      return { top: box.top, width: box.width };
    }),
  );
  expect(reasonBoxes).toHaveLength(2);
  expect(reasonBoxes[1].top).toBeGreaterThan(reasonBoxes[0].top);
  expect(Math.abs(reasonBoxes[0].width - reasonBoxes[1].width)).toBeLessThan(2);

  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
});
