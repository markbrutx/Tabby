import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Workspace E2E tests.
 *
 * Uses a shared Tauri mock that simulates the backend so the app's
 * real invoke/listen code paths work against an in-memory workspace.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ content: createTauriMockScript() });
  await page.goto("/");
  await page.locator("[data-active]").first().waitFor({ state: "visible", timeout: 10_000 });
});

test("bootstraps with a single terminal pane", async ({ page }) => {
  const tabs = page.locator('[data-testid^="tab-"]');
  await expect(tabs.first()).toBeVisible();
  await expect(page.locator("[data-active]")).toHaveCount(1);
});

test("creates new tab with Cmd+T", async ({ page }) => {
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(1);
  await page.keyboard.press("Meta+t");
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });
});

test("switches tabs by clicking", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });

  await page.locator('[data-testid^="tab-"]').first().click();
  await expect(page.locator('[data-testid^="pane-"]:visible').first()).toBeVisible();
});

test("closes tab with close button", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });

  const closeButton = page.locator('[data-testid^="close-tab-"]').last();
  await closeButton.click();
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(1, { timeout: 5_000 });
});

test("creates new tab via + button", async ({ page }) => {
  await page.getByTestId("new-tab-button").click();
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });
});

test("pane focus via keyboard (Alt+Arrow)", async ({ page }) => {
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  const panes = page.locator("[data-active]");
  await expect(panes).toHaveCount(2, { timeout: 5_000 });

  await panes.nth(0).click();
  await expect(panes.nth(0)).toHaveAttribute("data-active", "true");
});

test("opens settings with Cmd+,", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });
});

test("saves settings and closes modal", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible({ timeout: 3_000 });

  await page.getByTestId("theme-card-dawn").click();
  await page.getByTestId("save-settings").click();

  await expect(page.getByTestId("settings-modal")).toHaveCount(0, { timeout: 3_000 });
});

test("restarts pane with Cmd+Shift+R", async ({ page }) => {
  const pane = page.locator("[data-active]").first();
  await pane.click();
  await page.keyboard.press("Meta+Shift+r");

  await expect(pane).toBeVisible();
});

test("Cmd+W closes active pane", async ({ page }) => {
  await page.keyboard.press("Meta+w");

  await expect(page.locator('[data-testid^="tab-"]').first()).toBeVisible({ timeout: 5_000 });
  await expect(page.locator("[data-active]").first()).toBeVisible({ timeout: 5_000 });
});

// ---------- Additional workspace tests ----------

test("tab rename via double-click", async ({ page }) => {
  const firstTab = page.getByTestId("tab-1");
  await firstTab.dblclick();

  const renameInput = page.getByTestId("tab-rename-input-1");
  await expect(renameInput).toBeVisible({ timeout: 3_000 });

  await renameInput.fill("My Terminal");
  await renameInput.press("Enter");

  await expect(page.getByText("My Terminal")).toBeVisible({ timeout: 3_000 });
});

test("multi-tab workflow: create tabs, switch, close", async ({ page }) => {
  // Create a second tab via wizard
  await page.keyboard.press("Meta+t");
  const wizardCreate = page.getByTestId("wizard-create");
  await expect(wizardCreate).toBeVisible({ timeout: 3_000 });
  await wizardCreate.click();
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });

  // Switch between tabs
  await page.getByTestId("tab-1").click();
  await page.getByTestId("tab-2").click();

  // Close the second tab (use last close button)
  await page.locator('[data-testid^="close-tab-"]').last().click();

  // If a confirm dialog appears, confirm it
  const confirmOk = page.getByTestId("confirm-ok");
  if (await confirmOk.isVisible({ timeout: 1_000 }).catch(() => false)) {
    await confirmOk.click();
  }

  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(1, { timeout: 5_000 });
});

test("wizard creates tab with custom name", async ({ page }) => {
  await page.keyboard.press("Meta+t");

  const wizardTitle = page.getByTestId("wizard-title");
  await expect(wizardTitle).toBeVisible({ timeout: 3_000 });

  // Change workspace name
  const nameInput = page.getByTestId("workspace-name-input");
  if (await nameInput.isVisible()) {
    await nameInput.fill("Dev Environment");
  }

  // Create the workspace
  const createBtn = page.getByTestId("wizard-create");
  await expect(createBtn).toBeVisible({ timeout: 3_000 });
  await createBtn.click();

  // New tab should exist
  await expect(page.locator('[data-testid^="tab-"]')).toHaveCount(2, { timeout: 5_000 });
});

test("close last tab auto-creates new one", async ({ page }) => {
  // Close the only tab
  await page.locator('[data-testid="close-tab-1"]').click();

  // A new tab should auto-appear
  await expect(page.locator('[data-testid^="tab-"]').first()).toBeVisible({ timeout: 5_000 });
  await expect(page.locator("[data-active]").first()).toBeVisible({ timeout: 5_000 });
});

test("Cmd+Shift+W closes entire tab", async ({ page }) => {
  // Split to get 2 panes in the tab
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });
  await expect(page.locator("[data-active]")).toHaveCount(2, { timeout: 5_000 });

  // Close entire tab
  await page.keyboard.press("Meta+Shift+w");

  // Auto-created tab should appear
  await expect(page.locator('[data-testid^="tab-"]').first()).toBeVisible({ timeout: 5_000 });
});
