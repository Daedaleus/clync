import { test, expect } from '@playwright/test';

test.describe('Groups page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/groups');
  });

  test('shows "My Groups" section', async ({ page }) => {
    await expect(page.getByTestId('section-my-groups')).toBeVisible();
  });

  test('shows "+ Create" button', async ({ page }) => {
    await expect(page.getByRole('button', { name: '+ Create' })).toBeVisible();
  });

  test('clicking "+ Create" reveals the create form', async ({ page }) => {
    await page.getByRole('button', { name: '+ Create' }).click();
    await expect(page.getByPlaceholder('Group name', { exact: true })).toBeVisible();
  });

  test('cancelling hides the create form', async ({ page }) => {
    await page.getByRole('button', { name: '+ Create' }).click();
    await page.getByRole('button', { name: 'Cancel' }).click();
    await expect(page.getByRole('button', { name: '+ Create' })).toBeVisible();
  });

  test('shows group search section', async ({ page }) => {
    await expect(page.getByText(/All Groups|Search Public Groups/)).toBeVisible();
  });
});
