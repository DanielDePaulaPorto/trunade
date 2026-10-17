/*
 * Every ````markdown preview=ID block in the site's docs is opened in the real
 * Trunade! page, run, and photographed. The site's remark plugin
 * (site/plugins/previews.mjs) puts the picture right after the block.
 *
 * Block options, after preview=ID:
 *   input="Ana"   run the first block by hand and type these lines (| separates)
 *   reveal        click "show answer" on the quiz
 */
import { spawn, type ChildProcess } from 'node:child_process'
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { expect, test, type Page } from '@playwright/test'

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const SITE = join(ROOT, 'site')
const BINARY = join(ROOT, 'server/target/debug/trunade')
const OUTPUT = join(SITE, 'public/previews')
const DOCS = { pt: join(SITE, 'src/pages/docs'), en: join(SITE, 'src/pages/en/docs') }
const PORTS = { pt: 4830, en: 4831 }

/** Page chrome that is not part of a lesson. */
const HIDE = `.masthead, .menu-button, .end-mark, .run-all, .offline { display: none !important; }
  .document { padding-top: 1.25rem !important; padding-bottom: 0.5rem !important; }`

interface Example {
  id: string
  markdown: string
  input: string[] | null
  reveal: boolean
}

function examples(dir: string): Example[] {
  const found: Example[] = []
  for (const file of readdirSync(dir).filter((name) => name.endsWith('.md'))) {
    const text = readFileSync(join(dir, file), 'utf8')
    const block = /^(`{4,}|~{3,})markdown[ \t]+([^\n]*preview=[^\n]*)\n([\s\S]*?)\n\1[ \t]*$/gm
    for (const [, , meta, markdown] of text.matchAll(block)) {
      const id = /preview=([\w-]+)/.exec(meta!)![1]!
      const input = /input="([^"]*)"/.exec(meta!)?.[1]?.split('|') ?? null
      found.push({ id, markdown: markdown!, input, reveal: /\breveal\b/.test(meta!) })
    }
  }
  return found
}

function startServer(lesson: string, home: string, port: number): ChildProcess {
  return spawn(BINARY, [lesson, '--no-open', '--port', String(port)], {
    env: { ...process.env, HOME: home, XDG_DATA_HOME: join(home, 'data') },
    stdio: 'ignore',
  })
}

function homeWithPreferences(lang: 'pt' | 'en') {
  const home = mkdtempSync(join(tmpdir(), 'trunade-previews-'))
  mkdirSync(join(home, '.config/trunade'), { recursive: true })
  writeFileSync(
    join(home, '.config/trunade/preferences.toml'),
    `theme = "light"\ncode_theme = "dark"\nlocale = "${lang}"\ntext_width = "narrow"\n`,
  )
  return home
}

async function render(page: Page, url: string, example: Example) {
  await page.goto(url)
  await expect(page.locator('.document')).toBeVisible()
  await page.addStyleTag({ content: HIDE })
  await page.evaluate(() => document.fonts.ready)
  const blocks = page.locator('.code-cell')
  if (example.input) {
    await blocks.first().locator('.run').click()
    await expect(blocks.first().locator('.xterm-rows')).not.toBeEmpty()
    for (const line of example.input) {
      await page.keyboard.type(line)
      await page.keyboard.press('Enter')
    }
    await expect(blocks.first().locator('.run')).toBeVisible() // finished: ▶ is back
  } else if (await blocks.count()) {
    await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur())
    await page.keyboard.press('r')
    await expect(page.locator('.run-all .run-all-summary')).toBeAttached({ timeout: 120_000 }) // hidden by HIDE
  }
  if (example.reveal) await page.locator('.quiz-reveal').first().click()
  if (await page.locator('.mermaid-diagram').count()) await expect(page.locator('.mermaid-diagram svg').first()).toBeVisible()
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur()) // no focus ring in the picture
  await page.mouse.move(0, 0) // no hover state either
  await page.waitForTimeout(400) // terminals paint on the next frames
}

for (const lang of ['pt', 'en'] as const) {
  test(`previews ${lang}`, async ({ page }) => {
    const work = mkdtempSync(join(tmpdir(), 'trunade-previews-lesson-'))
    const lesson = join(work, 'lesson.md')
    writeFileSync(lesson, '# Trunade!\n')
    const server = startServer(lesson, homeWithPreferences(lang), PORTS[lang])
    const url = `http://localhost:${PORTS[lang]}/`
    const sizes: Record<string, { width: number; height: number }> = {}
    rmSync(join(OUTPUT, lang), { recursive: true, force: true }) // no pictures of removed examples
    mkdirSync(join(OUTPUT, lang), { recursive: true })
    try {
      await expect.poll(() => fetch(url).then((r) => r.ok).catch(() => false)).toBe(true)
      for (const example of examples(DOCS[lang])) {
        // An invisible marker tells when the server has the new document.
        const marker = `<!-- preview ${lang}/${example.id} -->`
        writeFileSync(lesson, `${example.markdown}\n\n${marker}\n`)
        await expect.poll(() => fetch(`${url}api/document`).then((r) => r.text()).then((text) => text.includes(marker))).toBe(true)
        await render(page, url, example)
        const document = page.locator('.document')
        const box = (await document.boundingBox())!
        await document.screenshot({ path: join(OUTPUT, lang, `${example.id}.png`) })
        sizes[example.id] = { width: Math.round(box.width), height: Math.round(box.height) }
      }
    } finally {
      server.kill()
    }
    writeFileSync(join(OUTPUT, lang, 'sizes.json'), `${JSON.stringify(sizes, null, 2)}\n`)
  })
}
