import { test, expect } from '@playwright/test';

test.describe('Me page', () => {
  test('redirects to /me after login and shows greeting', async ({ page }) => {
    await page.goto('/');
    await page.waitForURL(/\/me/);
    await expect(page.getByTestId('greeting')).toContainText('tester');
  });

  test.describe('content', () => {
    test.beforeEach(async ({ page }) => {
      await page.goto('/me');
    });

    test('shows upcoming sessions section', async ({ page }) => {
      await expect(page.getByTestId('section-upcoming-sessions')).toBeVisible();
    });

    test('shows your groups section', async ({ page }) => {
      await expect(page.getByTestId('section-your-groups')).toBeVisible();
    });

    test('shows your games section', async ({ page }) => {
      await expect(page.getByTestId('section-your-games')).toBeVisible();
    });
  });
});
