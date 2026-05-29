import { defineConfig, devices } from '@playwright/test';

/**
 * E2E tests run against the local dev stack (must be running before you invoke Playwright):
 *   npm run dev   — frontend on http://localhost:5173
 *   backend       — on http://localhost:3000
 *   Keycloak      — on http://localhost:8080
 *   SurrealDB     — on http://localhost:8000
 *
 * Override via environment variables:
 *   E2E_BASE_URL   — frontend URL   (default: http://localhost:5173)
 *   E2E_USERNAME   — Keycloak user  (default: tester)
 *   E2E_PASSWORD   — Keycloak pass  (default: geheim)
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 1,
  reporter: [['html', { open: 'never' }]],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  expect: { timeout: 8_000 },
  projects: [
    // Logs in via Keycloak and saves auth state — runs first, once.
    {
      name: 'setup',
      testMatch: /auth\.setup\.ts/,
    },
    // Authenticated tests — depend on the saved auth state.
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        storageState: 'e2e/.auth/user.json',
      },
      dependencies: ['setup'],
      testIgnore: /auth\.setup\.ts/,
    },
    // Public tests (no auth) — run independently.
    {
      name: 'public',
      use: { ...devices['Desktop Chrome'] },
      testMatch: /public\/.+\.spec\.ts/,
    },
  ],
});
