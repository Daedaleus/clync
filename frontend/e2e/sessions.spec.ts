import { test, expect } from '@playwright/test';

test.describe('Sessions page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/sessions');
  });

  test('shows section headings', async ({ page }) => {
    await expect(page.getByTestId('section-sessions')).toBeVisible();
    await expect(page.getByTestId('section-group-sessions')).toBeVisible();
    await expect(page.getByTestId('section-global-sessions')).toBeVisible();
  });

  test('shows "+ New Time" button', async ({ page }) => {
    await expect(page.getByRole('button', { name: '+ New Time' })).toBeVisible();
  });

  test('clicking "+ New Time" reveals the create form', async ({ page }) => {
    await page.getByRole('button', { name: '+ New Time' }).click();
    await expect(page.getByTestId('session-create-form')).toBeVisible();
  });

  test('cancelling hides the create form', async ({ page }) => {
    await page.getByRole('button', { name: '+ New Time' }).click();
    await page.getByTestId('session-create-form').getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByTestId('session-create-form')).not.toBeVisible();
  });
});
