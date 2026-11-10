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
    page.getByRole("heading", { name: "No clear structural warning signs" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Answer optional questions" }).click();
  await page.getByRole("button", { name: "Next question" }).first().click();

  const resultDimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(resultDimensions.scrollWidth).toBeLessThanOrEqual(resultDimensions.clientWidth);
});
