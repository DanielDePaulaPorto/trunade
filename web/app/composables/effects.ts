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

/** Puts every title back to its real text (used before printing). */
export function settleHeadings(root: HTMLElement) {
  for (const heading of root.querySelectorAll<HTMLElement>('[data-text]')) {
    heading.dataset.settled = ''
    heading.textContent = heading.dataset.text ?? ''
  }
}

function decrypt(heading: HTMLElement) {
  const text = heading.dataset.text ?? ''
  let step = 0
  const timer = setInterval(() => {
    if (heading.dataset.settled !== undefined) return clearInterval(timer)
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

/* Confetti for a right answer: paper bits thrown up from an element, falling with gravity. */
const CONFETTI_PIECES = 140
const CONFETTI_MS = 2600
const GRAVITY = 0.32
// Brand inks: malachite in two tones, glow, and bronze only as a pinch.
const CONFETTI_COLORS = ['#0b8f6a', '#0b8f6a', '#0e6e53', '#5ef2d0', '#5ef2d0', '#a85d18']

interface Piece {
  x: number
  y: number
  vx: number
  vy: number
  size: number
  angle: number
  spin: number
  color: string
}

export function confetti(from: HTMLElement) {
  if (prefersLessMotion()) return
  const canvas = document.createElement('canvas')
  canvas.className = 'confetti'
  const ratio = window.devicePixelRatio || 1
  canvas.width = window.innerWidth * ratio
  canvas.height = window.innerHeight * ratio
  document.body.append(canvas)
  const context = canvas.getContext('2d')!
  context.scale(ratio, ratio)

  const origin = center(from)
  const pieces: Piece[] = Array.from({ length: CONFETTI_PIECES }, () => {
    const direction = -Math.PI / 2 + (Math.random() - 0.5) * 1.6 // mostly upwards, fanned out
    const speed = 9 + Math.random() * 11
    return {
      x: origin.x,
      y: origin.y,
      vx: Math.cos(direction) * speed,
      vy: Math.sin(direction) * speed,
      size: 6 + Math.random() * 6,
      angle: Math.random() * Math.PI,
      spin: (Math.random() - 0.5) * 0.4,
      color: CONFETTI_COLORS[Math.floor(Math.random() * CONFETTI_COLORS.length)]!,
    }
  })

  const start = performance.now()
  const frame = (now: number) => {
    const elapsed = now - start
    context.clearRect(0, 0, window.innerWidth, window.innerHeight)
    context.globalAlpha = Math.min(1, 2 * (1 - elapsed / CONFETTI_MS)) // fade over the last half
    for (const piece of pieces) {
      piece.vx *= 0.985 // air
      piece.vy = piece.vy * 0.985 + GRAVITY
      piece.x += piece.vx
      piece.y += piece.vy
      piece.angle += piece.spin
      context.save()
      context.translate(piece.x, piece.y)
      context.rotate(piece.angle)
      context.scale(1, Math.cos(piece.angle * 2)) // paper turning over
      context.fillStyle = piece.color
      context.fillRect(-piece.size / 2, -piece.size / 4, piece.size, piece.size / 2)
      context.restore()
    }
    if (elapsed < CONFETTI_MS) requestAnimationFrame(frame)
    else canvas.remove()
  }
  requestAnimationFrame(frame)
}
