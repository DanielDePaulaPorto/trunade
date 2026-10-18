/*
 * `data-reveal` on any element of the lesson (<img data-reveal>, <div data-reveal>…):
 * it keeps its space but stays invisible until the heading of its section
 * reaches the top part of the screen (or shows up when the page cannot scroll
 * further), then appears and stays. Printing shows
 * everything (main.css).
 */
const SECTION_LINE = 0.3 // same line the usage log uses for "current section"
const HEADINGS = ':is(h1, h2, h3)'

export function trackReveals() {
  let frame = 0
  const schedule = () => {
    cancelAnimationFrame(frame)
    frame = requestAnimationFrame(revealReached)
  }
  // New or reloaded cells bring new elements; scrolling brings sections.
  const observer = new MutationObserver(schedule)
  observer.observe(document.body, { childList: true, subtree: true })
  window.addEventListener('scroll', schedule, { passive: true })
  window.addEventListener('resize', schedule)
  schedule()
  return () => {
    cancelAnimationFrame(frame)
    observer.disconnect()
    window.removeEventListener('scroll', schedule)
    window.removeEventListener('resize', schedule)
  }
}

function revealReached() {
  let heading: Element | null = null
  for (const element of document.querySelectorAll(`.document ${HEADINGS}, .document [data-reveal]`)) {
    if (element.matches(HEADINGS)) heading = element
    else if (!element.classList.contains('revealed') && (!heading || reached(heading))) reveal(element)
  }
}

/** At the top part of the screen, or on screen when the page cannot scroll further (last sections). */
function reached(heading: Element) {
  const top = heading.getBoundingClientRect().top
  const atBottom = window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2
  return top <= window.innerHeight * SECTION_LINE || (atBottom && top < window.innerHeight)
}

function reveal(element: Element) {
  element.classList.add('revealed')
  // An animated image starts from its first frame, not mid-loop.
  for (const image of [element, ...element.querySelectorAll('img')]) {
    if (image instanceof HTMLImageElement && /\.gif($|\?)|^data:image\/gif/i.test(image.src)) image.src = image.src
  }
}
