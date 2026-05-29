import { test as setup, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const authFile = path.join(__dirname, '.auth/user.json');

setup('authenticate via Keycloak', async ({ page }) => {
  const baseUrl = process.env.E2E_BASE_URL ?? 'http://localhost:5173';
  const username = process.env.E2E_USERNAME ?? 'tester';
  const password = process.env.E2E_PASSWORD ?? 'geheim';

  await page.goto(baseUrl);

  // Keycloak redirects the browser to the login page.
  await page.waitForURL(/\/realms\/Clync\//, { timeout: 10_000 });

  await page.fill('#username', username);
  await page.fill('#password', password);
  await page.click('[name="login"]');

  // Keycloak may prompt for profile completion on first login.
  // Submit the form with placeholder values if it appears.
  const profileForm = page.getByRole('heading', { name: 'Update Account Information' });
  if (await profileForm.isVisible({ timeout: 3_000 }).catch(() => false)) {
    await page.fill('#email', `${username}@example.com`);
    await page.fill('#firstName', username);
    await page.fill('#lastName', 'Test');
    await page.getByRole('button', { name: 'Submit' }).click();
  }

  // Wait for the redirect back to the app — the /me page is the default route.
  await page.waitForURL(`${baseUrl}/**`, { timeout: 15_000 });
  await expect(page).toHaveURL(/localhost:5173/);

  fs.mkdirSync(path.dirname(authFile), { recursive: true });
  await page.context().storageState({ path: authFile });
});
