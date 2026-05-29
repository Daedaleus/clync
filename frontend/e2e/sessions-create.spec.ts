import { test, expect } from '@playwright/test';

test('create a session and delete it afterwards', async ({ page }) => {
  await page.goto('/sessions');

  // Snapshot the session IDs that exist before we create anything.
  const before = new Set(
    await page.locator('[data-testid^="session-"]').evaluateAll(
      (els) => els.map((el) => (el as HTMLElement).dataset['testid'] ?? ''),
    ),
  );

  const tomorrow = new Date();
  tomorrow.setDate(tomorrow.getDate() + 1);
  const tomorrowStr = tomorrow.toISOString().split('T')[0]; // YYYY-MM-DD

  // Open the create form
  await page.getByRole('button', { name: '+ New Time' }).click();
  const form = page.getByTestId('session-create-form');

  // Pick a game from the GamePicker dropdown
  await form.getByPlaceholder('Choose game from library…').fill('Factorio');
  await page.getByRole('button', { name: 'Factorio' }).click();

  // Set date to tomorrow — the first available time slot is pre-selected
  await form.locator('input[type="date"]').fill(tomorrowStr);

  // Submit
  await form.getByRole('button', { name: 'Schedule' }).click();
  await expect(form).not.toBeVisible({ timeout: 8_000 });

  // Identify the newly created row by diffing before/after testid sets
  const after = await page.locator('[data-testid^="session-"]').evaluateAll(
    (els) => els.map((el) => (el as HTMLElement).dataset['testid'] ?? ''),
  );
  const newTestId = after.find((id) => !before.has(id));
  expect(newTestId, 'New session row should appear after creation').toBeDefined();

  // Verify it's visible then delete it
  const newRow = page.getByTestId(newTestId!);
  await expect(newRow).toBeVisible();
  await newRow.getByRole('button', { name: 'Delete' }).click();
  await expect(newRow).not.toBeVisible({ timeout: 5_000 });
});
