/*
 * Small visual effects, written for Trunade! (ideas from vue-bits: ClickSpark,
 * DecryptedText). All of them are skipped when the user asks for less motion.
 */

export function prefersLessMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

const SPARKS = 8
const SPARK_MS = 450

/** A burst of short lines around a point (or the center of an element). */
export function spark(at: MouseEvent | HTMLElement) {
  if (prefersLessMotion()) return
  const { x, y } = at instanceof MouseEvent ? { x: at.clientX, y: at.clientY } : center(at)
  const burst = document.createElement('div')
  burst.className = 'spark-burst'
  burst.style.left = `${x}px`
  burst.style.top = `${y}px`
  for (let i = 0; i < SPARKS; i++) {
    const line = document.createElement('span')
    line.style.setProperty('--angle', `${(360 / SPARKS) * i}deg`)
    burst.append(line)
  }
  document.body.append(burst)
  setTimeout(() => burst.remove(), SPARK_MS)
}

function center(element: HTMLElement) {
  const rect = element.getBoundingClientRect()
  return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
}

const SCRAMBLE_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#$%&*<>'
const DECRYPT_STEP_MS = 30
const DECRYPT_STEPS = 20

/**
 * Section titles (h1, h2) show scrambled characters that resolve left to right
 * the first time they scroll into view. Titles with markup inside are left alone.
 */
export function decryptHeadings(root: HTMLElement) {
  if (prefersLessMotion()) return
  const headings = [...root.querySelectorAll<HTMLElement>('h1, h2')].filter(
    (h) => h.childElementCount === 0 && h.textContent?.trim(),
  )
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue
      observer.unobserve(entry.target)
      decrypt(entry.target as HTMLElement)
    }
  })
  for (const heading of headings) {
    heading.dataset.text = heading.textContent ?? ''
    observer.observe(heading)
  }
}

function decrypt(heading: HTMLElement) {
  const text = heading.dataset.text ?? ''
  let step = 0
  const timer = setInterval(() => {
    step++
    const revealed = Math.floor((text.length * step) / DECRYPT_STEPS)
    heading.textContent = [...text]
      .map((char, i) => (i < revealed || char === ' ' ? char : randomChar()))
      .join('')
    if (step >= DECRYPT_STEPS) {
      clearInterval(timer)
      heading.textContent = text
    }
  }, DECRYPT_STEP_MS)
}

function randomChar() {
  return SCRAMBLE_CHARS[Math.floor(Math.random() * SCRAMBLE_CHARS.length)]
}
