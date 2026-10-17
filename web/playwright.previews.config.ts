import { defineConfig } from '@playwright/test'

/*
 * Not a test suite: renders the site's documentation examples in the real
 * Trunade! page and saves the pictures to site/public/previews (`pnpm previews`).
 */
export default defineConfig({
  testDir: 'previews',
  timeout: 600_000,
  workers: 1,
  use: {
    channel: 'chrome',
    locale: 'pt-BR',
    viewport: { width: 860, height: 900 },
    deviceScaleFactor: 2,
    colorScheme: 'light',
    reducedMotion: 'reduce',
  },
})
