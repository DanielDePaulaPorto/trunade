/*
 * Page events for the server's usage log (see server/src/usage_log.rs): which
 * section is on screen, quiz answers, steps, zoom, printing. Nothing about
 * who is in the room. Events while disconnected are simply lost.
 */
const SECTION_SETTLE_MS = 1500
const SECTION_LINE = 0.3 // a heading above 30% of the screen is the current one

let lastSection: HTMLElement | null = null
let settleTimer: ReturnType<typeof setTimeout> | undefined

export function logEvent(event: string, data: Record<string, unknown> = {}) {
  useConnection().send({ type: 'log', event, data })
}

/** Records the section once the page stops on it, not every heading scrolled past. */
export function trackUsage() {
  const onScroll = () => {
    clearTimeout(settleTimer)
    settleTimer = setTimeout(logCurrentSection, SECTION_SETTLE_MS)
  }
  const onVisibility = () => logEvent('visibility', { visible: document.visibilityState === 'visible' })
  window.addEventListener('scroll', onScroll, { passive: true })
  document.addEventListener('visibilitychange', onVisibility)
  onScroll()
  return () => {
    clearTimeout(settleTimer)
    window.removeEventListener('scroll', onScroll)
    document.removeEventListener('visibilitychange', onVisibility)
  }
}

function logCurrentSection() {
  const line = window.innerHeight * SECTION_LINE
  const current = [...document.querySelectorAll<HTMLElement>('.document .markdown :is(h1, h2, h3)')]
    .filter((heading) => heading.offsetParent !== null)
    .findLast((heading) => heading.getBoundingClientRect().top <= line)
  if (!current || current === lastSection) return
  lastSection = current
  const title = (current.dataset.text ?? current.textContent ?? '').trim()
  logEvent('section', { title, level: Number(current.tagName[1]) })
}

/** Plain text of rendered HTML, short enough for a log line. */
export function logText(html: string, limit = 120) {
  const element = document.createElement('div')
  element.innerHTML = html
  return (element.textContent ?? '').replace(/\s+/g, ' ').trim().slice(0, limit)
}
