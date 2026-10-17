import { readdirSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import { expect, test, type Locator, type Page } from '@playwright/test'

/** Exports are written beside the fixture; do not leave them in the repository. */
const rmOnExit: string[] = []
test.afterAll(() => rmOnExit.forEach((path) => rmSync(path, { force: true })))

test.beforeEach(async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('.code-cell').first()).toBeVisible()
})

function block(page: Page, index: number) {
  return page.locator('.code-cell').nth(index)
}

function terminalText(cell: Locator) {
  return cell.locator('.xterm-rows')
}

/** Page shortcuts only work outside editors and terminals. */
async function pressShortcut(page: Page, key: string) {
  await page.locator('.document h1').first().click()
  await page.keyboard.press(key)
}

test('runs a block with fixed input', async ({ page }) => {
  const cell = block(page, 0)
  await cell.getByTitle('Executar (Shift+Enter)').click()
  await expect(terminalText(cell)).toContainText('42')
})

test('interactive program reads what is typed in the terminal', async ({ page }) => {
  const cell = block(page, 1)
  await cell.getByTitle('Executar (Shift+Enter)').click()
  await expect(terminalText(cell)).toContainText('nome?')
  await page.keyboard.type('Ana\n')
  await expect(terminalText(cell)).toContainText('olá Ana')
})

test('test cases get a verdict each', async ({ page }) => {
  const cell = block(page, 2)
  await cell.getByTitle('Executar (Shift+Enter)').click()
  await expect(cell.locator('.case-mark.accepted')).toHaveCount(1)
  await expect(cell.locator('.case-mark.rejected')).toHaveCount(1)
  await expect(terminalText(cell)).toContainText('esperado «7», saiu «6»')
})

test('template code is hidden, numbered and compiled', async ({ page }) => {
  const cell = block(page, 3)
  const strip = cell.getByRole('button', { name: '1 linha do molde acima' })
  await expect(strip).toBeVisible()
  await expect(cell.locator('.cm-lineNumbers .cm-gutterElement').filter({ hasText: /^2$/ })).toBeVisible()
  await strip.click()
  await expect(cell.locator('.hidden-part')).toContainText('import sys')
  await cell.getByTitle('Executar (Shift+Enter)').click()
  await expect(cell.locator('.case-mark.accepted')).toHaveCount(1)
})

test('quiz marks the right answer once revealed', async ({ page }) => {
  const quiz = page.locator('.quiz')
  await quiz.getByRole('radio', { name: '3' }).click()
  await quiz.getByRole('button', { name: 'Ver resposta' }).click()
  await expect(quiz.locator('.quiz-option.wrong')).toHaveCount(1)
  await expect(quiz.locator('.quiz-option.correct')).toContainText('4')
})

test('steps move forward', async ({ page }) => {
  const steps = page.locator('.steps')
  await expect(steps).toContainText('Leia um número.')
  await steps.getByRole('button', { name: 'Próximo' }).click()
  await expect(steps).toContainText('Resposta final.')
  await expect(steps.getByText('Leia um número.')).toBeHidden()
})

test('printing opens every step and reveals the answers', async ({ page }) => {
  await page.evaluate(() => {
    window.print = () => {
      const visible = (selector: string) =>
        [...document.querySelectorAll<HTMLElement>(selector)].filter((element) => element.offsetParent !== null).length
      ;(window as any).printed = { panels: visible('.steps-panel'), correct: visible('.quiz-option.correct'), title: document.title }
    }
  })
  await pressShortcut(page, 'ControlOrMeta+p')
  const printed = await page.waitForFunction(() => (window as any).printed).then((handle) => handle.jsonValue())
  expect(printed).toEqual({ panels: 2, correct: 1, title: 'lesson' })
  await expect(page).toHaveTitle(/lesson\.md/)
})

test('Z zooms a block and Esc returns', async ({ page }) => {
  await block(page, 0).scrollIntoViewIfNeeded()
  await pressShortcut(page, 'z')
  await expect(page.locator('.code-cell.zoomed')).toHaveCount(1)
  await page.keyboard.press('Escape')
  await expect(page.locator('.code-cell.zoomed')).toHaveCount(0)
})

test('run all reports the failing block', async ({ page }) => {
  await pressShortcut(page, 'r')
  const status = page.locator('.run-all')
  await expect(status.locator('.run-all-ok')).toContainText('2', { timeout: 30_000 })
  await expect(status.locator('.run-all-failed')).toContainText('1')
  await expect(status.getByRole('button', { name: 'linha 23' })).toBeVisible()
})

test('exported HTML works without the server and replays runs', async ({ page }) => {
  await page.locator('.menu-button').click()
  await page.getByRole('button', { name: 'Exportar HTML' }).click()
  const status = page.locator('.export-status')
  await expect(status).toContainText('HTML salvo em', { timeout: 30_000 })
  const path = (await status.innerText()).replace(/^.*HTML salvo em\s*/s, '').trim()
  rmOnExit.push(path)

  await page.goto(`file://${path}`)
  await expect(page).toHaveTitle('lesson.md · Trunade!')
  await expect(page.locator('.offline')).toHaveCount(0)
  const cases = block(page, 2)
  await cases.getByTitle('Executar (Shift+Enter)').click()
  await expect(cases.locator('.case-mark.rejected')).toHaveCount(1)
  await expect(terminalText(cases)).toContainText('esperado «7», saiu «6»')
  const interactive = block(page, 1)
  await interactive.getByTitle('Executar (Shift+Enter)').click()
  await expect(terminalText(interactive)).toContainText('não foi rodado antes da exportação')
})

test('runs reach the usage log', async ({ page }) => {
  await block(page, 0).getByTitle('Executar (Shift+Enter)').click()
  await expect(terminalText(block(page, 0))).toContainText('42')
  const logs = join(process.env.TRUNADE_TEST_HOME!, '.local/share/trunade/logs')
  await expect
    .poll(() => {
      const events = readdirSync(logs).flatMap((file) =>
        readFileSync(join(logs, file), 'utf8').trim().split('\n').map((line) => JSON.parse(line).event),
      )
      return events.includes('start') && events.includes('finished')
    })
    .toBe(true)
})
