import { test, expect } from '@playwright/test';

test.describe('Library page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/library');
  });

  test('shows title', async ({ page }) => {
    await expect(page.getByTestId('section-library')).toBeVisible();
  });

  test('shows "+ New Game" button for admin', async ({ page }) => {
    await expect(page.getByRole('button', { name: '+ New Game' })).toBeVisible();
  });

  test('has a search input', async ({ page }) => {
    await expect(page.getByPlaceholder('Search games…')).toBeVisible();
  });

  test('search filters games by name', async ({ page }) => {
    await page.getByPlaceholder('Search games…').fill('zzz_no_match_xxxxxxxxxxx');
    await expect(page.getByText('No games found.')).toBeVisible();
  });

  test('clicking "+ New Game" reveals the add-game form', async ({ page }) => {
    await page.getByRole('button', { name: '+ New Game' }).click();
    await expect(page.getByPlaceholder('Name *')).toBeVisible();
  });
});
