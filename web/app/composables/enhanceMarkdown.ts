import katex from 'katex'
import type { Theme } from './usePreferences'

/** Turns comrak's math markers ($…$, $$…$$, ```math) into KaTeX output. */
export function renderMath(root: HTMLElement) {
  for (const element of root.querySelectorAll<HTMLElement>('[data-math-style]')) {
    const displayMode = element.dataset.mathStyle === 'display'
    const target = element.closest('pre') ?? element
    const host = document.createElement(displayMode ? 'div' : 'span')
    host.className = displayMode ? 'math-display' : 'math-inline'
    katex.render(element.textContent ?? '', host, { displayMode, throwOnError: false })
    target.replaceWith(host)
  }
}

let mermaidCounter = 0

/** ```mermaid blocks become diagrams; the source is kept to redraw on theme change. */
export async function renderMermaid(root: HTMLElement, theme: Theme) {
  for (const code of root.querySelectorAll<HTMLElement>('pre > code.language-mermaid')) {
    const host = document.createElement('div')
    host.className = 'mermaid-diagram'
    host.dataset.source = code.textContent ?? ''
    code.parentElement!.replaceWith(host)
  }
  const diagrams = root.querySelectorAll<HTMLElement>('.mermaid-diagram')
  if (diagrams.length === 0) return
  // Mermaid is large: load it only for documents that have diagrams.
  const { default: mermaid } = await import('mermaid')
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: 'strict',
    theme: theme === 'dark' ? 'dark' : 'default',
    fontFamily: 'inherit',
  })
  for (const host of diagrams) {
    try {
      const { svg } = await mermaid.render(`mermaid-${++mermaidCounter}`, host.dataset.source ?? '')
      host.innerHTML = svg
    } catch (error) {
      host.textContent = String(error)
      host.classList.add('mermaid-error')
    }
  }
}

/**
 * Iframes default to the text width (CSS). Pages served next to the lesson
 * (same origin) are measured: content wider than the frame is scaled down to
 * fit, and the frame takes the content height, so neither scrollbar appears.
 * Other sites cannot be measured by the browser and keep their attributes.
 */
export function fitIframes(root: HTMLElement) {
  for (const iframe of root.querySelectorAll('iframe')) {
    if (iframe.dataset.width) {
      scaleDeclaredWidth(iframe, Number(iframe.dataset.width))
      continue
    }
    const fit = () => {
      const page = sameOriginPage(iframe)
      if (!page) return
      scaleToWidth(iframe, page)
      fitHeight(iframe, page)
      // Later size changes (menus, late content) only adjust the height:
      // rescaling would resize the page again and loop.
      watchContent(page, () => fitHeight(iframe, page))
    }
    iframe.addEventListener('load', fit)
    if (iframe.contentDocument?.readyState === 'complete') fit()
  }
}

const DEFAULT_IFRAME_HEIGHT_PX = 500

/**
 * `<iframe data-width="1200">`: the page is laid out at the width it needs and
 * scaled down to the column, which works for any site, measurable or not.
 */
function scaleDeclaredWidth(iframe: HTMLIFrameElement, width: number) {
  const height = Number(iframe.getAttribute('height')) || DEFAULT_IFRAME_HEIGHT_PX
  const frame = document.createElement('div')
  frame.className = 'iframe-scaler'
  iframe.replaceWith(frame)
  frame.append(iframe)
  Object.assign(iframe.style, { width: `${width}px`, height: `${height}px`, transformOrigin: '0 0' })
  const fit = () => {
    const scale = Math.min(1, frame.clientWidth / width)
    iframe.style.transform = `scale(${scale})`
    frame.style.height = `${height * scale}px`
  }
  new ResizeObserver(fit).observe(frame)
  fit()
}

function sameOriginPage(iframe: HTMLIFrameElement): HTMLElement | null {
  try {
    const page = iframe.contentDocument?.documentElement
    return page && iframe.contentDocument?.body ? page : null
  } catch {
    return null // cross-origin
  }
}

function scaleToWidth(iframe: HTMLIFrameElement, page: HTMLElement) {
  page.style.zoom = '1'
  page.style.overflow = 'hidden'
  const scale = Math.min(1, iframe.clientWidth / page.scrollWidth)
  page.style.zoom = String(scale)
}

const HEIGHT_TOLERANCE_PX = 2

function fitHeight(iframe: HTMLIFrameElement, page: HTMLElement) {
  const height = Math.ceil(page.getBoundingClientRect().height)
  if (Math.abs(iframe.clientHeight - height) > HEIGHT_TOLERANCE_PX) {
    iframe.style.height = `${height}px`
  }
}

function watchContent(page: HTMLElement, onChange: () => void) {
  let pending = false
  new ResizeObserver(() => {
    if (pending) return
    pending = true
    requestAnimationFrame(() => {
      pending = false
      onChange()
    })
  }).observe(page.ownerDocument.body)
}
