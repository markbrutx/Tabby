import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Browser Pane E2E tests.
 *
 * Verifies browser pane creation, toolbar, and navigation.
 * In non-Tauri mode (E2E via Vite dev server), BrowserPane renders an iframe.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ content: createTauriMockScript() });
  await page.goto("/");
  await page.locator("[data-active]").first().waitFor({ state: "visible", timeout: 10_000 });
});

test("creates browser pane via split popup", async ({ page }) => {
  // Open split popup
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });

  // Select Browser mode
  await page.getByText("Browser", { exact: true }).click();

  // Confirm
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  // Browser pane should appear
  await expect(page.locator('[data-testid^="browser-pane-"]')).toBeVisible({ timeout: 5_000 });
});

test("browser toolbar shows default URL", async ({ page }) => {
  // Split with browser pane
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.getByText("Browser", { exact: true }).click();
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  // URL input should have a value
  const urlInput = page.getByTestId("browser-url-input");
  await expect(urlInput).toBeVisible({ timeout: 5_000 });
  const value = await urlInput.inputValue();
  expect(value.length).toBeGreaterThan(0);
});

test("navigates to URL via toolbar Enter key", async ({ page }) => {
  // Create browser pane
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.getByText("Browser", { exact: true }).click();
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  const urlInput = page.getByTestId("browser-url-input");
  await expect(urlInput).toBeVisible({ timeout: 5_000 });

  // Clear and type new URL
  await urlInput.fill("https://example.com");
  await urlInput.press("Enter");

  // The browser pane should still be visible after navigation
  await expect(page.locator('[data-testid^="browser-pane-"]')).toBeVisible({ timeout: 5_000 });
});

test("Go button triggers navigation", async ({ page }) => {
  // Create browser pane
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.getByText("Browser", { exact: true }).click();
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  const urlInput = page.getByTestId("browser-url-input");
  await expect(urlInput).toBeVisible({ timeout: 5_000 });

  await urlInput.fill("https://example.org");

  // Click Go button
  await page.getByTestId("browser-go-btn").click();

  // Browser pane should still be present
  await expect(page.locator('[data-testid^="browser-pane-"]')).toBeVisible({ timeout: 5_000 });
});

test("reload button works without crash", async ({ page }) => {
  // Create browser pane
  await page.keyboard.press("Meta+d");
  const dialog = page.locator("[role=dialog]");
  await expect(dialog).toBeVisible({ timeout: 3_000 });
  await page.getByText("Browser", { exact: true }).click();
  await page.keyboard.press("Enter");
  await expect(dialog).toHaveCount(0, { timeout: 3_000 });

  await expect(page.getByTestId("browser-reload-btn")).toBeVisible({ timeout: 5_000 });

  // Click reload
  await page.getByTestId("browser-reload-btn").click();

  // Pane should still be intact
  await expect(page.locator('[data-testid^="browser-pane-"]')).toBeVisible({ timeout: 5_000 });
});
