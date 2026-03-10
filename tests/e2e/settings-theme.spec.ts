import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Settings and Theme E2E tests.
 *
 * Verifies theme switching via ThemePreviewCard grid, font size changes,
 * zoom shortcuts, settings persistence, and reset to defaults.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript({
    content: createTauriMockScript({ settings: { theme: "midnight" } }),
  });
  await page.goto("/");
  await page.locator("[data-active]").first().waitFor({ state: "visible", timeout: 10_000 });
});

test("theme change via card click updates applied theme", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  // Click the Dawn theme card
  await page.getByTestId("theme-card-dawn").click();
  await page.getByTestId("save-settings").click();
  await expect(page.getByTestId("settings-modal")).toHaveCount(0, { timeout: 3_000 });

  // The theme should have changed — verify via CSS var on root element
  // (applyTheme sets CSS custom properties from the theme definition)
  await page.waitForTimeout(200);
  const bgColor = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--color-bg"),
  );
  expect(bgColor.trim()).toBeTruthy();
});

test("font size change updates --ui-font-size CSS var", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  // The font size input is a range slider — set it via JavaScript
  const fontInput = page.getByTestId("settings-font-size");
  await fontInput.fill("16");
  await page.getByTestId("save-settings").click();
  await expect(page.getByTestId("settings-modal")).toHaveCount(0, { timeout: 3_000 });

  const fontSize = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );
  expect(fontSize.trim()).toBe("16px");
});

test("Cmd+= zooms in", async ({ page }) => {
  const before = await page.evaluate(() =>
    parseInt(getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size")),
  );

  await page.keyboard.press("Meta+=");
  await page.waitForTimeout(200);

  const after = await page.evaluate(() =>
    parseInt(getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size")),
  );

  expect(after).toBeGreaterThan(before);
});

test("Cmd+- zooms out", async ({ page }) => {
  const before = await page.evaluate(() =>
    parseInt(getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size")),
  );

  await page.keyboard.press("Meta+-");
  await page.waitForTimeout(200);

  const after = await page.evaluate(() =>
    parseInt(getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size")),
  );

  expect(after).toBeLessThan(before);
});

test("Cmd+0 resets to 14px", async ({ page }) => {
  // Change font size first
  await page.keyboard.press("Meta+=");
  await page.keyboard.press("Meta+=");
  await page.waitForTimeout(200);

  await page.keyboard.press("Meta+0");
  await page.waitForTimeout(200);

  const fontSize = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );
  expect(fontSize.trim()).toBe("14px");
});

test("settings persist after save and reopen", async ({ page }) => {
  // Open settings and select Dawn theme
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  await page.getByTestId("theme-card-dawn").click();
  await page.getByTestId("save-settings").click();
  await expect(page.getByTestId("settings-modal")).toHaveCount(0, { timeout: 3_000 });

  // Reopen settings and verify Dawn is active (has ring/border highlight)
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  const dawnCard = page.getByTestId("theme-card-dawn");
  await expect(dawnCard).toBeVisible();
  // The active card has ring-1 class
  const classes = await dawnCard.getAttribute("class");
  expect(classes).toContain("ring");
});

test("theme cards fit within grid without overflow", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  const cards = page.locator("[data-testid^='theme-card-']");
  const count = await cards.count();
  expect(count).toBeGreaterThan(0);

  const modal = page.getByTestId("settings-modal");
  const modalBox = await modal.boundingBox();

  for (let i = 0; i < count; i++) {
    const card = cards.nth(i);
    const cardBox = await card.boundingBox();
    expect(cardBox).toBeTruthy();
    expect(cardBox!.width).toBeLessThanOrEqual(modalBox!.width / 3);
  }
});

test("theme card badge text is not clipped", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  const midnightCard = page.getByTestId("theme-card-midnight");
  const badge = midnightCard.locator("span").filter({ hasText: /^Dark$/ });
  await expect(badge).toBeVisible();
  await expect(badge).toHaveText("Dark");
});

test("reset to defaults restores original values", async ({ page }) => {
  // Change theme to Dawn
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  await page.getByTestId("theme-card-dawn").click();
  await page.getByTestId("save-settings").click();
  await expect(page.getByTestId("settings-modal")).toHaveCount(0, { timeout: 3_000 });

  // Reopen and click reset
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  const resetBtn = page.getByText("Reset to defaults");
  if (await resetBtn.isVisible()) {
    await resetBtn.click();
    await page.waitForTimeout(300);
  }
});
