import { mkdtempSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { join } from 'node:path'
import { defineConfig } from '@playwright/test'

/*
 * End-to-end tests against the real `trunade` binary (with the generated web
 * embedded), driven by the Chrome already installed: `pnpm test:e2e`.
 * HOME points to a temporary directory so preferences and usage logs never
 * touch the user's own ~/.config and ~/.local.
 */
const PORT = 4799
const home = (process.env.TRUNADE_TEST_HOME ??= mkdtempSync(join(tmpdir(), 'trunade-e2e-')))

export default defineConfig({
  testDir: 'tests/e2e',
  timeout: 60_000,
  expect: { timeout: 15_000 },
  workers: 1, // one server, one page state
  use: {
    channel: 'chrome',
    baseURL: `http://localhost:${PORT}`,
    locale: 'pt-BR',
    viewport: { width: 1280, height: 900 },
  },
  webServer: {
    command: `cargo run --quiet --manifest-path ../server/Cargo.toml -- tests/e2e/fixtures/lesson.md --no-open --port ${PORT}`,
    url: `http://localhost:${PORT}/api/document`,
    env: {
      HOME: home,
      XDG_DATA_HOME: '',
      // cargo and rustup still live in the real home
      RUSTUP_HOME: process.env.RUSTUP_HOME ?? join(homedir(), '.rustup'),
      CARGO_HOME: process.env.CARGO_HOME ?? join(homedir(), '.cargo'),
    },
    timeout: 300_000,
    reuseExistingServer: false,
  },
})
