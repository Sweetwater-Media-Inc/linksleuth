import { expect, test } from "@playwright/test";

async function analyze(page, url) {
  await page.getByLabel("URL to analyze").fill(url);
  await page.getByRole("button", { name: "Analyze URL" }).click();
}

async function expectNoHorizontalOverflow(page) {
  const dimensions = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth);
}

test("stacks the complete workflow at 390px without horizontal overflow", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Understand a link before you click it." }),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);

  await analyze(page, "https://mobile.example/products");
  await expect(
    page.getByRole("heading", { name: "No obvious structural warning signs found." }),
  ).toBeVisible();
  await expect(page.getByLabel("Where did the link come from?")).toBeVisible();
  await expect(page.getByLabel("What was it supposed to be for?")).toBeVisible();

  const { assessmentBox, contextBox } = await page.evaluate(() => ({
    assessmentBox: document.querySelector("#assessment").getBoundingClientRect().toJSON(),
    contextBox: document.querySelector(".context-panel").getBoundingClientRect().toJSON(),
  }));
  expect(contextBox.y).toBeGreaterThan(assessmentBox.y + assessmentBox.height);

  await expect(page.locator("#supporting-findings")).not.toHaveAttribute("open", "");
  await page.locator("#supporting-findings > summary").click();
  await expect(page.getByText("No supported structural observations were found.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Review summary" })).toBeVisible();
  await expectNoHorizontalOverflow(page);
});

test("uses a single readable column at the 768px tablet width", async ({ page }) => {
  await page.setViewportSize({ width: 768, height: 1024 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await analyze(
    page,
    "https://tablet.example/?target=https%3A%2F%2Fdestination.example%2Fcontent",
  );

  const { assessmentBox, contextBox } = await page.evaluate(() => ({
    assessmentBox: document.querySelector("#assessment").getBoundingClientRect().toJSON(),
    contextBox: document.querySelector(".context-panel").getBoundingClientRect().toJSON(),
  }));
  expect(contextBox.y).toBeGreaterThan(assessmentBox.y + assessmentBox.height);
  expect(Math.abs(contextBox.width - assessmentBox.width)).toBeLessThan(2);
  await expectNoHorizontalOverflow(page);
});

test("keeps every major section aligned and bounded at 1920px", async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await analyze(
    page,
    "https://desktop.example/?target=https%3A%2F%2Fdestination.example%2Fcontent",
  );

  const resultsBox = await page.locator("#results").boundingBox();
  const analyzerBox = await page.locator(".analyzer-panel").boundingBox();
  const { assessmentBox, contextBox } = await page.evaluate(() => ({
    assessmentBox: document.querySelector("#assessment").getBoundingClientRect().toJSON(),
    contextBox: document.querySelector(".context-panel").getBoundingClientRect().toJSON(),
  }));
  expect(resultsBox.width).toBeLessThanOrEqual(1408);
  expect(Math.abs(resultsBox.width - analyzerBox.width)).toBeLessThan(2);
  expect(Math.abs(assessmentBox.y - contextBox.y)).toBeLessThan(2);
  expect(contextBox.x).toBeGreaterThan(assessmentBox.x + assessmentBox.width);
  const boundaries = await page.evaluate(() =>
    [".header-inner", ".intro", ".analyzer-panel", "#results", "footer"].map(
      (selector) => {
        const box = document.querySelector(selector).getBoundingClientRect();
        return { selector, left: box.left, right: box.right };
      },
    ),
  );
  for (const boundary of boundaries.slice(1)) {
    expect(Math.abs(boundary.left - boundaries[0].left), boundary.selector).toBeLessThan(2);
    expect(Math.abs(boundary.right - boundaries[0].right), boundary.selector).toBeLessThan(2);
  }
  await expectNoHorizontalOverflow(page);
});

test("wraps long disclosed evidence safely at mobile width", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  const longToken = "fabricated-token-".repeat(120);
  const candidate = encodeURIComponent(
    `https://destination.example/private?token=${longToken}#private-fragment`,
  );
  await analyze(page, `https://mobile.example/?target=${candidate}`);
  await page.getByRole("button", { name: "Reveal destination hostnames" }).click();
  await page.getByRole("button", { name: "Reveal detailed evidence" }).click();

  await expect(page.locator("#destination-details")).toContainText("fabricated-token");
  await expect(page.locator("#destination-details")).not.toContainText("private-fragment");
  await expectNoHorizontalOverflow(page);
});

test("stacks URL and context reason groups at a narrow viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expect(page.getByText("Local analyzer ready")).toBeVisible();
  await analyze(
    page,
    "https://top.example/?target=tracking.example%2F%2Fpath%3Fadurl%3D%2F%2Fstorage.example%2Flanding",
  );
  await page.getByLabel("Where did the link come from?").selectOption("email");
  await page.getByLabel("What was it supposed to be for?").selectOption("document");
  await page.getByRole("button", { name: "Show two more optional questions" }).click();
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
  await expectNoHorizontalOverflow(page);
});
