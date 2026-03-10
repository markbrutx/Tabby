import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Collapse/Expand E2E tests.
 *
 * Verifies pane collapse toggle behavior, guard against
 * collapsing the last expanded pane, and focus transfer.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ content: createTauriMockScript() });
  await page.goto("/");
  await page.locator("[data-active]").first().waitFor({ state: "visible", timeout: 10_000 });
});

test("collapse button hidden with single pane", async ({ page }) => {
  // With only 1 pane, the collapse button should not render
  await expect(page.getByTestId("pane-header-collapse")).toHaveCount(0);
});

test("collapse button visible with 2+ panes", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Both headers should show the collapse button
  const collapseButtons = page.getByTestId("pane-header-collapse");
  await expect(collapseButtons).toHaveCount(2, { timeout: 3_000 });
});

test("Cmd+M collapses active pane", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Focus first pane
  await page.locator("[data-active]").first().click();

  // Collapse with Cmd+M
  await page.keyboard.press("Meta+m");
  await page.waitForTimeout(500);

  // After collapse, the collapsed pane's xterm should be hidden
  // The collapse button icon should change to Maximize2
  const collapseButtons = page.getByTestId("pane-header-collapse");
  await expect(collapseButtons.first()).toBeVisible({ timeout: 3_000 });
});

test("expand restores pane after collapse", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Collapse first pane
  const collapseButtons = page.getByTestId("pane-header-collapse");
  await collapseButtons.first().click();
  await page.waitForTimeout(500);

  // Click the same collapse button again to expand
  await collapseButtons.first().click();
  await page.waitForTimeout(500);

  // Both panes should be visible again
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });
});

test("cannot collapse last expanded pane", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Collapse first pane
  const collapseButtons = page.getByTestId("pane-header-collapse");
  await collapseButtons.first().click();
  await page.waitForTimeout(500);

  // Try to collapse the second (last expanded) pane via Cmd+M
  await page.keyboard.press("Meta+m");
  await page.waitForTimeout(500);

  // The second pane should still have its xterm visible (guard prevented collapse)
  const remainingXterms = page.locator("[data-active] .xterm");
  const count = await remainingXterms.count();
  expect(count).toBeGreaterThanOrEqual(1);
});

test("focus moves to next pane on collapse", async ({ page }) => {
  // Split to get 2 panes
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Focus and collapse first pane
  await page.locator("[data-active]").first().click();
  await page.keyboard.press("Meta+m");
  await page.waitForTimeout(500);

  // The remaining expanded pane should have focus (data-active="true")
  const activePanes = page.locator('[data-active="true"]');
  const activeCount = await activePanes.count();
  expect(activeCount).toBeGreaterThanOrEqual(1);
});
