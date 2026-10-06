export interface Topic {
  element: HTMLElement
  text: string
  children: Topic[]
}

const SCROLL_MARGIN_PX = 24
const SAME_SPOT_PX = 8
const SCROLL_SETTLE_MS = 700

let lastTarget: { element: HTMLElement; at: number } | null = null

function headings(): HTMLElement[] {
  return [...document.querySelectorAll<HTMLElement>('.document > .markdown :is(h1, h2, h3)')]
}

/** Smoothly brings the next (or previous) heading to the top of the screen. */
export function scrollToAdjacentSection(direction: 1 | -1) {
  const all = headings()
  if (all.length === 0) return
  const recent = lastTarget && performance.now() - lastTarget.at < SCROLL_SETTLE_MS
  // While a smooth scroll is still running, continue from where it is heading.
  const reference = recent ? top(lastTarget!.element) : SCROLL_MARGIN_PX
  const target =
    direction === 1
      ? all.find((h) => top(h) > reference + SAME_SPOT_PX)
      : all.findLast((h) => top(h) < reference - SAME_SPOT_PX)
  if (target) {
    scrollToHeading(target)
  } else if (direction === 1) {
    window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' })
  } else {
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

export function scrollToHeading(element: HTMLElement) {
  lastTarget = { element, at: performance.now() }
  window.scrollTo({ top: window.scrollY + top(element) - SCROLL_MARGIN_PX, behavior: 'smooth' })
}

/**
 * Sections and their subsections. With a single h1 (the lesson title), h2 are
 * the sections and h3 the subsections; otherwise h1 and h2.
 */
export function topicTree(): Topic[] {
  const all = headings()
  const single = all.filter((h) => h.tagName === 'H1').length === 1
  const [sectionTag, subTag] = single ? ['H2', 'H3'] : ['H1', 'H2']
  const tree: Topic[] = []
  for (const element of all) {
    // A title may be mid-animation (decrypt effect): prefer its stored text.
    const text = (element.dataset.text ?? element.textContent ?? '').trim()
    const topic = { element, text, children: [] }
    if (element.tagName === sectionTag) tree.push(topic)
    else if (element.tagName === subTag) tree.at(-1)?.children.push(topic)
  }
  return tree
}

/** Index of the section the reader is currently in. */
export function currentSectionIndex(tree: Topic[]): number {
  const index = tree.findLastIndex((t) => top(t.element) <= SCROLL_MARGIN_PX + SAME_SPOT_PX)
  return Math.max(0, index)
}

function top(element: HTMLElement) {
  return element.getBoundingClientRect().top
}
