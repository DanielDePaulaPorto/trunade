/*
 * PDF export through the browser's print dialog. While `printing` is on, the
 * page shows everything (all steps, quiz answers, whole terminal output) and
 * the print stylesheet hides the controls. The title becomes the lesson name,
 * which the dialog suggests as the file name.
 */
const printing = ref(false)

export function usePrint() {
  return { printing, printLesson }
}

async function printLesson(fileName?: string) {
  if (printing.value) return
  printing.value = true
  logEvent('print')
  const title = document.title
  if (fileName) document.title = fileName.replace(/\.md$/i, '')
  await nextTick()
  await renderedFrames(2) // xterm repaints on animation frames
  try {
    window.print() // blocks until the dialog closes
  } finally {
    document.title = title
    printing.value = false
  }
}

const FRAME_WAIT_LIMIT_MS = 300

/** Hidden tabs get no animation frames: never wait for them forever. */
function renderedFrames(count: number) {
  return new Promise<void>((resolve) => {
    const step = (left: number) => (left === 0 ? resolve() : requestAnimationFrame(() => step(left - 1)))
    step(count)
    setTimeout(resolve, FRAME_WAIT_LIMIT_MS)
  })
}
