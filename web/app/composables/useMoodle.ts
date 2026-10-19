/*
 * "Copy for Moodle": the lesson as plain HTML to paste into a Moodle Page.
 * Moodle keeps only what is written on each element, so there is no script,
 * no stylesheet and no web font: every style is inline, code is colored with
 * inline Monokai spans, formulas go back to \( … \) for Moodle's MathJax
 * filter, diagrams become images. Outputs are what the page shows now (run
 * everything with R first). The clipboard gets HTML (rich paste) and the same
 * HTML as text (for the editor's source view).
 */
import { highlightCode, tagHighlighter } from '@lezer/highlight'
import { cppLanguage } from '@codemirror/lang-cpp'
import { pythonLanguage } from '@codemirror/lang-python'
import type { Cell, CodeCell, QuizCell, StepsCell } from './types'
import { DARK, highlightRules } from './monokai'
import { drawnDiagrams, exportableBlock, inlineImages } from './useExport'

const MONO = "'JetBrains Mono', Menlo, Consolas, monospace"
const STYLE = {
  code: `background:${DARK.background};color:${DARK.foreground};padding:12px 16px;border-radius:8px;overflow:auto;font:14px/1.5 ${MONO};white-space:pre;margin:0`,
  terminal: `background:#000;padding:10px 16px;border-radius:8px;overflow:auto;font:13px/1.5 ${MONO};white-space:pre;margin:4px 0 0`,
  label: 'margin:10px 0 0;font-size:12px;font-weight:700;text-transform:uppercase;letter-spacing:.04em;color:#4d5f59',
  block: 'margin:20px 0',
  inlineCode: `background:#eef2f0;padding:0 4px;border-radius:4px;font-family:${MONO};font-size:.92em`,
  pre: `background:#f4f7f6;padding:12px 16px;border-radius:8px;overflow:auto;font:14px/1.5 ${MONO}`,
  table: 'border-collapse:collapse;margin:12px 0',
  cell: 'border:1px solid #d3dcd8;padding:4px 10px',
  quote: 'border-left:4px solid #d3dcd8;margin:12px 0;padding:2px 14px;color:#4d5f59',
  quiz: 'background:#f4f7f6;border:1px solid #d3dcd8;border-radius:8px;padding:12px 16px;margin:20px 0',
}
const ALERT_COLORS: Record<string, string> = {
  note: '#0b8f6a',
  tip: '#0b8f6a',
  important: '#0e6e53',
  warning: '#a85d18',
  caution: '#d3245c',
}
const CODE_LANGUAGES = { cpp: cppLanguage, 'c++': cppLanguage, cxx: cppLanguage, cc: cppLanguage, c: cppLanguage }
const PYTHON = ['python', 'py', 'python3']

/** Copies the lesson; call it straight from the click (the clipboard needs the gesture). */
export function copyForMoodle() {
  const html = lessonHtml()
  const blob = (type: string) => html.then((text) => new Blob([text], { type }))
  return navigator.clipboard.write([new ClipboardItem({ 'text/html': blob('text/html'), 'text/plain': blob('text/plain') })])
}

async function lessonHtml() {
  const { snapshot } = useConnection()
  const drawn = drawnInLightTheme()
  const parts = await Promise.all((snapshot.value?.cells ?? []).map((cell) => cellHtml(cell, drawn)))
  return `<div>${parts.join('\n')}</div>`
}

async function cellHtml(cell: Cell, drawn: Map<string, string>): Promise<string> {
  switch (cell.kind) {
    case 'markdown':
      return markdownHtml(drawn.get(cell.id) ?? cell.html)
    case 'code':
      return codeHtml(cell)
    case 'quiz':
      return quizHtml(cell)
    case 'steps':
      return stepsHtml(cell, drawn)
  }
}

// ----------------------------------------------------------------- code

function codeHtml(cell: CodeCell) {
  const block = exportableBlock(cell.id)
  const edits = block?.edits()
  const program = block?.program() ?? cell.source
  const { t } = useI18n()
  const parts = [`<pre style="${STYLE.code}"><code>${unindented(highlight(program, cell.fenceLanguage))}</code></pre>`]
  for (const testCase of edits?.cases ?? cell.cases) {
    if (testCase.stdin.trim()) parts.push(labelled(t('stdin'), testCase.stdin, '#e6db74'))
    if (testCase.expected !== null) parts.push(labelled(t('expectedOutput'), testCase.expected, '#a6e22e'))
  }
  const output = block?.terminalText()
  if (output) parts.push(labelled(t('output'), output, '#e8e8e8'))
  return `<div style="${STYLE.block}">${parts.join('')}</div>`
}

function labelled(label: string, text: string, color: string) {
  return `<p style="${STYLE.label}">${escape(label)}</p><pre style="${STYLE.terminal};color:${color}">${unindented(escape(text.replace(/\n$/, '')))}</pre>`
}

/**
 * In Moodle, a code line starting with spaces showed its HTML as text (the
 * content is read like Markdown somewhere along the way). So no line starts
 * with a space or is empty: indentation goes inside a span. The span carries a
 * style because Moodle's editor (TinyMCE) unwraps spans with no attributes,
 * which put the spaces back. The copied text is the same.
 */
const INDENT = '<span style="white-space:pre">'

function unindented(html: string) {
  return html
    .split('\n')
    .map((line) => (line.trim() ? line.replace(/^[ \t]+/, (space) => `${INDENT}${space}</span>`) : `${INDENT}${line}</span>`))
    .join('\n')
}

/** Monokai colors as inline styles, from the same parser the editor uses. */
function highlight(code: string, fenceLanguage: string) {
  const language = PYTHON.includes(fenceLanguage) ? pythonLanguage : CODE_LANGUAGES[fenceLanguage as keyof typeof CODE_LANGUAGES]
  if (!language) return escape(code)
  const highlighter = tagHighlighter(
    highlightRules(DARK).map(({ tag, color, fontStyle, backgroundColor }) => ({
      tag,
      // A "class" here is an inline style without spaces; several are joined with spaces.
      class: [`color:${color}`, fontStyle && `font-style:${fontStyle}`, backgroundColor && `background:${backgroundColor}`]
        .filter(Boolean)
        .join(';'),
    })),
  )
  let html = ''
  highlightCode(
    code,
    language.parser.parse(code),
    highlighter,
    (text, styles) => (html += styles ? `<span style="${styles.split(' ').join(';')}">${escape(text)}</span>` : escape(text)),
    () => (html += '\n'),
  )
  return html
}

// ------------------------------------------------------------- markdown

/**
 * Markdown cells as drawn on the page, in the light theme: colors that come
 * from the page (var(--brand), color-mix, currentColor) are resolved, SVG
 * figures and diagrams become images, KaTeX goes back to TeX. Synchronous on
 * purpose: the theme switch is never painted.
 */
function drawnInLightTheme() {
  const root = document.documentElement
  const theme = root.dataset.theme
  root.dataset.theme = 'light'
  try {
    const drawn = new Map<string, string>()
    for (const cell of document.querySelectorAll<HTMLElement>('.markdown[data-cell-id]')) {
      drawn.set(cell.dataset.cellId!, standalone(cell))
    }
    return drawn
  } finally {
    if (theme) root.dataset.theme = theme
  }
}

const RESOLVED = ['fill', 'stroke', 'color', 'background-color', 'border-color', 'opacity', 'font-weight']

function standalone(original: HTMLElement) {
  const copy = original.cloneNode(true) as HTMLElement
  const originals = [original, ...original.querySelectorAll<Element>('*')]
  const copies = [copy, ...copy.querySelectorAll<Element>('*')]
  originals.forEach((element, index) => resolveColors(element, copies[index]!))
  for (const math of copy.querySelectorAll('.math-inline, .math-display')) mathToTex(math)
  for (const svg of copy.querySelectorAll('svg')) svgToImage(svg, originals[copies.indexOf(svg)] as SVGSVGElement)
  for (const heading of copy.querySelectorAll<HTMLElement>('[data-text]')) heading.textContent = heading.dataset.text ?? ''
  for (const element of copy.querySelectorAll('[data-reveal]')) element.removeAttribute('data-reveal')
  removeComments(copy)
  return copy.innerHTML
}

/** Inline styles and currentColor attributes written with the page's own colors. */
function resolveColors(original: Element, copy: Element) {
  const style = original.getAttribute('style') ?? ''
  const usesPage = /var\(|color-mix\(|currentcolor/i
  const attributes = ['fill', 'stroke'].filter((name) => usesPage.test(original.getAttribute(name) ?? ''))
  if (!usesPage.test(style) && !attributes.length && !(original instanceof SVGSVGElement)) return
  const computed = getComputedStyle(original)
  for (const property of RESOLVED) {
    if (style.includes(property) || attributes.includes(property) || original instanceof SVGSVGElement) {
      ;(copy as HTMLElement).style.setProperty(property, computed.getPropertyValue(property))
    }
  }
  for (const name of attributes) copy.setAttribute(name, computed.getPropertyValue(name))
}

/** Notes left in HTML comments stay out of the copy. */
function removeComments(root: Node) {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_COMMENT)
  const comments: Node[] = []
  while (walker.nextNode()) comments.push(walker.currentNode)
  for (const comment of comments) comment.parentNode?.removeChild(comment)
}

function mathToTex(host: Element) {
  const tex = host.querySelector('annotation[encoding="application/x-tex"]')?.textContent ?? host.textContent ?? ''
  if (host.classList.contains('math-display')) {
    const paragraph = document.createElement('p')
    paragraph.style.textAlign = 'center'
    paragraph.textContent = `\\[${tex}\\]`
    host.replaceWith(paragraph)
  } else {
    host.replaceWith(`\\(${tex}\\)`)
  }
}

/** Moodle drops inline SVG; an SVG image keeps the figure exactly. */
function svgToImage(svg: SVGSVGElement, original: SVGSVGElement | undefined) {
  const box = original?.getBoundingClientRect()
  const viewBox = svg.viewBox.baseVal
  const width = svg.getAttribute('width') ?? String(Math.round(box?.width || viewBox?.width || 300))
  svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
  // An image cannot load the page's web fonts: use a monospace the reader has.
  if (!svg.getAttribute('font-family')) svg.style.setProperty('font-family', 'ui-monospace, Menlo, Consolas, monospace')
  if (!svg.getAttribute('width')) svg.setAttribute('width', width)
  const image = document.createElement('img')
  image.src = `data:image/svg+xml;base64,${utf8Base64(svg.outerHTML)}`
  image.alt = svg.getAttribute('aria-label') ?? ''
  image.setAttribute('width', width)
  image.style.maxWidth = '100%'
  image.style.height = 'auto'
  ;(svg.closest('.mermaid-diagram') ?? svg).replaceWith(image)
}

async function markdownHtml(html: string) {
  const template = document.createElement('template')
  template.innerHTML = await inlineImages(html)
  const root = template.content
  formulasToTex(root)
  diagramsToImages(root)
  for (const element of root.querySelectorAll('[data-reveal]')) element.removeAttribute('data-reveal')
  styleAlerts(root)
  styleElements(root)
  return template.innerHTML
}

/** Moodle's MathJax filter reads \( … \) and \[ … \]. */
function formulasToTex(root: DocumentFragment) {
  for (const element of root.querySelectorAll<HTMLElement>('[data-math-style]')) {
    const tex = element.textContent ?? ''
    const display = element.dataset.mathStyle === 'display'
    const pre = element.closest('pre')
    if (pre) {
      const paragraph = document.createElement('p')
      paragraph.style.textAlign = 'center'
      paragraph.textContent = `\\[${tex}\\]`
      pre.replaceWith(paragraph)
    } else {
      element.replaceWith(display ? `\\[${tex}\\]` : `\\(${tex}\\)`)
    }
  }
}

/** The diagram drawn on the page, as an image (Moodle drops inline SVG). */
function diagramsToImages(root: DocumentFragment) {
  const drawn = drawnDiagrams()
  for (const code of root.querySelectorAll('pre > code.language-mermaid')) {
    const svg = drawn[code.textContent ?? '']
    if (!svg) continue
    const image = document.createElement('img')
    image.src = `data:image/svg+xml;base64,${utf8Base64(svg)}`
    image.alt = ''
    image.style.maxWidth = '100%'
    code.parentElement!.replaceWith(image)
  }
}

function styleAlerts(root: DocumentFragment) {
  for (const alert of root.querySelectorAll<HTMLElement>('.markdown-alert')) {
    const kind = [...alert.classList].find((name) => name.startsWith('markdown-alert-'))?.slice(15) ?? 'note'
    const color = ALERT_COLORS[kind] ?? ALERT_COLORS.note
    alert.setAttribute('style', `border-left:4px solid ${color};background:#f4f7f6;border-radius:0 8px 8px 0;padding:4px 16px;margin:16px 0`)
    alert.querySelector<HTMLElement>('.markdown-alert-title')?.setAttribute('style', `font-weight:700;color:${color};margin:8px 0 4px`)
  }
}

function styleElements(root: DocumentFragment) {
  const rules: [string, string][] = [
    [':not(pre) > code', STYLE.inlineCode],
    ['pre', STYLE.pre],
    ['table', STYLE.table],
    ['th, td', STYLE.cell],
    ['blockquote:not(.markdown-alert)', STYLE.quote],
    ['img', 'max-width:100%;height:auto'],
  ]
  for (const [selector, style] of rules) {
    for (const element of root.querySelectorAll(selector)) {
      element.setAttribute('style', `${style};${element.getAttribute('style') ?? ''}`)
    }
  }
  for (const pre of root.querySelectorAll('pre')) pre.innerHTML = unindented(pre.innerHTML)
}

// ------------------------------------------------------- quiz and steps

async function quizHtml(cell: QuizCell) {
  const { t } = useI18n()
  const letter = (index: number) => String.fromCharCode(97 + index)
  const options = await Promise.all(cell.options.map((option) => markdownHtml(option.html)))
  const answers = cell.options.flatMap((option, index) => (option.correct ? [`${letter(index)}) ${options[index]}`] : []))
  return `<div style="${STYLE.quiz}">${await markdownHtml(cell.question)}<ol type="a">${options
    .map((option) => `<li>${option}</li>`)
    .join('')}</ol><details><summary style="cursor:pointer;font-weight:700;color:#0e6e53">${escape(t('showAnswer'))}</summary><p>${answers.join('<br>')}</p></details></div>`
}

async function stepsHtml(cell: StepsCell, drawn: Map<string, string>) {
  const parts = await Promise.all(
    cell.steps.map(async (step, index) => {
      const cells = await Promise.all(step.cells.map((child) => cellHtml(child, drawn)))
      return `<h4>${index + 1}. ${escape(step.title)}</h4>${cells.join('\n')}`
    }),
  )
  return parts.join('\n')
}

function utf8Base64(text: string) {
  return btoa(Array.from(new TextEncoder().encode(text), (byte) => String.fromCharCode(byte)).join(''))
}

function escape(text: string) {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}
