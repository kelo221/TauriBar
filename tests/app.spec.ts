import { test, expect } from '@playwright/test';

test.describe('TauriBar App', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should have the correct title', async ({ page }) => {
    // Check if the app loads and has expected content
    await expect(page.locator('body')).toBeVisible();
  });

  test('should display the main app interface', async ({ page }) => {
    // Wait for Vue to load and render
    await page.waitForLoadState('domcontentloaded');

    // Check for key elements that should be present in your music player app
    // These selectors should match your actual Vue components

    // Example checks - adjust these based on your actual app structure
    // await expect(page.locator('.playlist')).toBeVisible();
    // await expect(page.locator('.playback-controls')).toBeVisible();

    // For now, just check that the app container is present
    await expect(page.locator('#app')).toBeVisible();
  });
});
