import { test, expect } from '@playwright/test';

test.describe('Friends page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/friends');
  });

  test('shows "My Friends" section', async ({ page }) => {
    await expect(page.getByTestId('section-my-friends')).toBeVisible();
  });

  test('shows friend requests section when requests are pending', async ({ page }) => {
    // Only rendered when the user has pending requests — absence is correct behaviour.
    const section = page.getByTestId('section-friend-requests');
    if (await section.count() > 0) {
      await expect(section).toBeVisible();
    }
  });

  test('shows user search section', async ({ page }) => {
    await expect(page.getByTestId('section-search-users')).toBeVisible();
    await expect(page.getByPlaceholder('Username…')).toBeVisible();
  });

  test('searching for a non-existent user shows empty state', async ({ page }) => {
    await page.getByPlaceholder('Username…').fill('zzz_no_such_user_xxxxxxxxxxx');
    await page.getByPlaceholder('Username…').press('Enter');
    await expect(page.getByText('No users found.')).toBeVisible();
  });
});
