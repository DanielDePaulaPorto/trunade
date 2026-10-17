/*
 * Runs every block, one after the other, to check a lesson before class or
 * before exporting. A block without fixed input runs once with empty input
 * (pipes and time limit), so a program that waits for the keyboard ends
 * instead of hanging the queue.
 */
export interface BlockRunner {
  /** Resolves true when it compiled, exited with 0 and every expected output matched. */
  run: () => Promise<boolean>
  line: number
}

export interface RunAllReport {
  total: number
  done: number
  failed: { id: string; line: number }[]
  finished: boolean
}

const runners = new Map<string, BlockRunner>()
const report = ref<RunAllReport | null>(null)

export function useRunAll() {
  return { report, runAll, closeReport, registerRunner, unregisterRunner }
}

function registerRunner(id: string, runner: BlockRunner) {
  runners.set(id, runner)
}

function unregisterRunner(id: string) {
  runners.delete(id)
}

function closeReport() {
  if (report.value?.finished) report.value = null
}

async function runAll() {
  if (report.value && !report.value.finished) return
  const blocks = blocksInPageOrder()
  const current: RunAllReport = reactive({ total: blocks.length, done: 0, failed: [], finished: false })
  report.value = current
  for (const [id, runner] of blocks) {
    if (!(await runner.run())) current.failed.push({ id, line: runner.line })
    current.done++
  }
  current.finished = true
  logEvent('runAll', { blocks: current.total, failed: current.failed.map((block) => block.line) })
}

function blocksInPageOrder() {
  return [...document.querySelectorAll<HTMLElement>('.code-cell')].flatMap((element) => {
    const runner = runners.get(element.dataset.cellId ?? '')
    return runner ? [[element.dataset.cellId!, runner] as const] : []
  })
}

/** Shows a failed block, opening its step if it is inside one. */
export function revealBlock(id: string) {
  const element = document.querySelector<HTMLElement>(`.code-cell[data-cell-id="${CSS.escape(id)}"]`)
  element?.dispatchEvent(new CustomEvent('trunade:reveal', { bubbles: true }))
  nextTick(() => element?.scrollIntoView({ behavior: 'smooth', block: 'center' }))
}
