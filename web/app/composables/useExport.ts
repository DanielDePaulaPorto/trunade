/*
 * Saves the lesson as one HTML file beside the .md (the server writes it; see
 * server/src/export.rs and useStatic.ts). What goes in is what the page shows:
 * edited code and input, and the last run of each block, which ▶ replays.
 */
import type { Cell, TestCase } from './types'
import { BODY_FONTS, CODE_FONTS, TITLE_FONTS } from './fonts'
import { withPageEdits, type RunRecord, type StaticLesson } from './useStatic'

export interface ExportableBlock {
  edits: () => { source: string; cases: TestCase[] }
  /** The run on screen, if it was made with the code and input now shown. */
  lastRun: () => RunRecord | null
}

export type ExportState =
  | { phase: 'running' | 'writing' }
  | { phase: 'done'; path: string }
  | { phase: 'failed'; error: string }

const blocks = new Map<string, ExportableBlock>()
const state = ref<ExportState | null>(null)

export function useExport() {
  return { state, exportHtml, closeExport, registerExportable, unregisterExportable }
}

function registerExportable(id: string, block: ExportableBlock) {
  blocks.set(id, block)
}

function unregisterExportable(id: string) {
  blocks.delete(id)
}

function closeExport() {
  state.value = null
}

/** Called by useConnection when the server answers. */
export function exportFinished(path: string | null, error: string | null) {
  state.value = path ? { phase: 'done', path } : { phase: 'failed', error: error ?? '?' }
}

/** Light by default: the file is read later, often printed or projected. */
async function exportHtml({ runFirst, theme }: { runFirst: boolean; theme: 'light' | 'dark' }) {
  const { snapshot, send } = useConnection()
  if (!snapshot.value) return
  if (runFirst) {
    state.value = { phase: 'running' }
    await useRunAll().runAll()
  }
  state.value = { phase: 'writing' }
  const edits = new Map([...blocks].map(([id, block]) => [id, block.edits()]))
  const runs = Object.fromEntries(
    [...blocks].flatMap(([id, block]) => {
      const run = block.lastRun()
      return run ? [[id, run]] : []
    }),
  )
  const lesson: StaticLesson = {
    snapshot: { ...snapshot.value, cells: await withInlineImages(withPageEdits(snapshot.value.cells, edits)) },
    preferences: fixedPreferences(theme),
    runs,
    diagrams: drawnDiagrams(),
    exportedAt: new Date().toISOString(),
  }
  send({ type: 'exportHtml', lesson, fonts: fontsInUse() })
}

/** The chosen theme, never "automatic" (it would follow the reader's system). */
function fixedPreferences(theme: 'light' | 'dark') {
  const { preferencesFile, codeThemeMode } = usePreferences()
  const codeTheme = codeThemeMode.value === 'auto' ? theme : codeThemeMode.value
  return { ...preferencesFile(), theme, code_theme: codeTheme }
}

/** Family names the exported CSS must keep (first name of each stack). */
function fontsInUse() {
  const { titleFont, bodyFont, codeFont } = usePreferences()
  const stacks = [TITLE_FONTS[titleFont.value].family, BODY_FONTS[bodyFont.value].family, CODE_FONTS[codeFont.value].family]
  const names = stacks.map((stack) => stack.split(',')[0]!.trim().replace(/^["']|["']$/g, ''))
  if (document.querySelector('.katex')) names.push('KaTeX')
  return [...new Set(names)]
}

function drawnDiagrams() {
  const diagrams: Record<string, string> = {}
  for (const host of document.querySelectorAll<HTMLElement>('.mermaid-diagram')) {
    if (host.dataset.source && host.querySelector('svg')) diagrams[host.dataset.source] = host.innerHTML
  }
  return diagrams
}

/** Images next to the .md travel inside the file as data URLs. */
async function withInlineImages(cells: Cell[]): Promise<Cell[]> {
  return Promise.all(
    cells.map(async (cell) => {
      if (cell.kind === 'markdown') return { ...cell, html: await inlineImages(cell.html) }
      if (cell.kind === 'steps') {
        const steps = await Promise.all(cell.steps.map(async (step) => ({ ...step, cells: await withInlineImages(step.cells) })))
        return { ...cell, steps }
      }
      return cell
    }),
  )
}

async function inlineImages(html: string) {
  if (!html.includes('<img')) return html
  const template = document.createElement('template')
  template.innerHTML = html
  for (const image of template.content.querySelectorAll('img[src]')) {
    const src = image.getAttribute('src')!
    if (/^(data:|https?:)/.test(src)) continue
    const dataUrl = await toDataUrl(src)
    if (dataUrl) image.setAttribute('src', dataUrl)
  }
  return template.innerHTML
}

async function toDataUrl(src: string) {
  try {
    const blob = await fetch(src).then((response) => (response.ok ? response.blob() : null))
    if (!blob) return null
    return await new Promise<string>((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => resolve(reader.result as string)
      reader.onerror = reject
      reader.readAsDataURL(blob)
    })
  } catch {
    return null
  }
}
