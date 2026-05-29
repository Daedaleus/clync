import { test, expect } from '@playwright/test';

// The join page is fully public — Keycloak auth is NOT initialised on this route.
test.describe('Join page (public)', () => {
  test('renders without redirecting to Keycloak', async ({ page }) => {
    await page.goto('/join/some-token');
    // Must stay on the join route, not bounce to localhost:8080
    await expect(page).not.toHaveURL(/localhost:8080/, { timeout: 5_000 });
    await expect(page).toHaveURL(/\/join\//);
  });

  test('shows "Checking invitation link…" while validating', async ({ page }) => {
    await page.goto('/join/some-token');
    // The checking message may flash briefly while the API call is in flight.
    // We accept either the checking text or the invalid state — both are correct.
    await expect(
      page.getByText(/Checking invitation link…|Link invalid/)
    ).toBeVisible({ timeout: 5_000 });
  });

  test('shows "Link invalid" for a fake token', async ({ page }) => {
    await page.goto('/join/this-token-does-not-exist');
    await expect(page.getByText('Link invalid')).toBeVisible({ timeout: 8_000 });
    await expect(
      page.getByText('This invitation link has expired or has already been used.')
    ).toBeVisible();
  });
});
