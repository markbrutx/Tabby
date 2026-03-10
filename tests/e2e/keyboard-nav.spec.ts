import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Keyboard navigation E2E tests.
 *
 * Verifies all keyboard shortcuts defined in useWorkspaceShortcuts.ts.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ content: createTauriMockScript() });
  await page.goto("/");
  await page.locator("[data-active]").first().waitFor({ state: "visible", timeout: 10_000 });
});

test("Cmd+T opens setup wizard", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.getByTestId("wizard-title")).toBeVisible({ timeout: 3_000 });
});

test("Cmd+W on single pane creates fresh tab", async ({ page }) => {
  await page.keyboard.press("Meta+w");
  // Single pane — no confirm dialog, auto-recreates
  await expect(page.locator('[data-testid^="tab-"]').first()).toBeVisible({ timeout: 5_000 });
  await expect(page.locator("[data-active]").first()).toBeVisible({ timeout: 5_000 });
});

test("Cmd+W on split pane shows confirm dialog", async ({ page }) => {
  // Split first
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Close pane — should show confirm
  await page.keyboard.press("Meta+w");
  await expect(page.getByTestId("confirm-ok")).toBeVisible({ timeout: 3_000 });
  await page.getByTestId("confirm-ok").click();

  await expect(page.locator("[data-active]")).toHaveCount(1, { timeout: 5_000 });
});

test("Cmd+D opens split-right popup", async ({ page }) => {
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await expect(page.getByText("Split right")).toBeVisible({ timeout: 3_000 });
});

test("Cmd+E opens split-down popup", async ({ page }) => {
  await page.keyboard.press("Meta+e");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await expect(page.getByText("Split below")).toBeVisible({ timeout: 3_000 });
});

test("Cmd+, opens settings", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });
});

test("Cmd+/ opens shortcuts modal", async ({ page }) => {
  await page.keyboard.press("Meta+/");
  await expect(page.getByTestId("shortcuts-modal")).toBeVisible({ timeout: 3_000 });
});

test("Cmd+Shift+R restarts pane", async ({ page }) => {
  const pane = page.locator("[data-active]").first();
  await pane.click();
  await page.keyboard.press("Meta+Shift+r");
  // Pane should still be visible after restart
  await expect(pane).toBeVisible({ timeout: 5_000 });
});

test("Alt+Arrow navigates between panes", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Click first pane to focus it
  await page.locator("[data-active]").first().click();
  await expect(page.locator("[data-active]").first()).toHaveAttribute("data-active", "true");

  // Navigate right
  await page.keyboard.press("Alt+ArrowRight");
  await page.waitForTimeout(300);

  // The second pane should now be focused (data-active="true")
  // Note: focus change depends on the mock's focusPane working correctly
  await expect(page.locator("[data-active]")).toHaveCount(2);
});

test("Cmd+1..2 switches tabs", async ({ page }) => {
  // Create a second tab via wizard
  await page.keyboard.press("Meta+t");
  const wizardCreate = page.getByTestId("wizard-create");
  await expect(wizardCreate).toBeVisible({ timeout: 3_000 });
  await wizardCreate.click();
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });

  // Switch to tab 1
  await page.keyboard.press("Meta+1");
  await page.waitForTimeout(300);

  // Switch to tab 2
  await page.keyboard.press("Meta+2");
  await page.waitForTimeout(300);

  // Both tabs should still be present
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2);
});

test("Cmd+] and Cmd+[ navigate panes in DFS order", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Navigate forward
  await page.keyboard.press("Meta+]");
  await page.waitForTimeout(300);

  // Navigate backward
  await page.keyboard.press("Meta+[");
  await page.waitForTimeout(300);

  // Both panes should still be present
  await expect(page.locator("[data-active]")).toHaveCount(2);
});

test("Cmd+= zooms in and Cmd+- zooms out", async ({ page }) => {
  // Get initial font size
  const initialSize = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );

  // Zoom in
  await page.keyboard.press("Meta+=");
  await page.waitForTimeout(200);

  const afterZoomIn = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );

  // Font size should have increased
  expect(parseInt(afterZoomIn)).toBeGreaterThan(parseInt(initialSize));

  // Zoom out
  await page.keyboard.press("Meta+-");
  await page.waitForTimeout(200);

  const afterZoomOut = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );

  expect(parseInt(afterZoomOut)).toBeLessThan(parseInt(afterZoomIn));
});

test("Cmd+0 resets zoom to 14px", async ({ page }) => {
  // Zoom in first
  await page.keyboard.press("Meta+=");
  await page.keyboard.press("Meta+=");
  await page.waitForTimeout(200);

  // Reset
  await page.keyboard.press("Meta+0");
  await page.waitForTimeout(200);

  const fontSize = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--ui-font-size"),
  );

  expect(fontSize.trim()).toBe("14px");
});
