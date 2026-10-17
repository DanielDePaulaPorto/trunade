<script setup lang="ts">
import type { CodeCell, Outcome, RunMessage, TestCase, Verdict } from '~/composables/types'
import { staticLesson, type RecordedMessage, type RunRecord } from '~/composables/useStatic'
import TerminalOutput from './TerminalOutput.vue'
import { zoomScale } from '~/composables/useZoom'

const props = defineProps<{ cell: CodeCell }>()
const { send, onRun, offRun } = useConnection()
const { t } = useI18n()
const { zoomedCell, zoomCell, closeZoom } = useZoom()
const zoomed = computed(() => zoomedCell.value === props.cell.id)
const scale = computed(() => (zoomed.value ? zoomScale() : 1))

type Status = 'idle' | 'compiling' | 'running' | 'done'
/** ✓ only for an expected output matched by a program that exited with 0. */
type CaseResult = 'accepted' | 'rejected' | 'ran'

const source = ref(props.cell.source)
const cases = ref<TestCase[]>(copyCases())
const caseResults = ref<CaseResult[]>([])
const judged = computed(() => cases.value.length > 1 || cases.value.some((c) => c.expected !== null))
const status = ref<Status>('idle')
const compileMs = ref<number | null>(null)
const runMs = ref<number | null>(null)
const hasOutput = ref(false)
const terminal = ref<InstanceType<typeof TerminalOutput>>()
const runButton = ref<HTMLButtonElement>()
let runId: string | null = null
let endsWithNewline = true
let casesStarted = 0
let settle: ((passed: boolean) => void) | null = null
// What the server sent for the run on screen, for the HTML export.
let recording: RunRecord | null = null
let recordingStart = 0
let lastRun: RunRecord | null = null

const modified = computed(
  () => source.value !== props.cell.source || JSON.stringify(cases.value) !== JSON.stringify(props.cell.cases),
)

function copyCases() {
  return props.cell.cases.map((testCase) => ({ ...testCase }))
}
const busy = computed(() => status.value === 'compiling' || status.value === 'running')

// Template code hidden around the block; the gutter keeps the whole program's numbering.
const showBefore = ref(false)
const showAfter = ref(false)
const beforeLines = computed(() => countLines(props.cell.before))
const afterLines = computed(() => countLines(props.cell.after))
const sourceLines = computed(() => source.value.split('\n').length)

function countLines(text: string) {
  return text ? text.replace(/\n$/, '').split('\n').length : 0
}

function hiddenLabel(count: number, where: 'Above' | 'Below') {
  return count === 1 ? t(`hiddenLine${where}` as const) : t(`hiddenLines${where}` as const, { count })
}

/** Run with a spark at the click, or at the ▶ button when started from the keyboard. */
function runWithSpark(event?: MouseEvent) {
  const origin = event ?? runButton.value
  if (origin) spark(origin)
  run()
}

/** For "run all": resolves once this run finishes, true if everything passed. */
function runAndWait() {
  run()
  return new Promise<boolean>((resolve) => (settle = resolve))
}

function run() {
  if (busy.value) stop()
  settle?.(false)
  settle = null
  runId = crypto.randomUUID()
  hasOutput.value = true
  compileMs.value = null
  runMs.value = null
  endsWithNewline = true
  casesStarted = 0
  caseResults.value = []
  recording = { source: source.value, cases: cases.value.map((c) => ({ ...c })), events: [] }
  recordingStart = performance.now()
  lastRun = null
  status.value = 'compiling'
  terminal.value?.reset()
  onRun(runId, handle)
  send({ type: 'run', runId, cellId: props.cell.id, source: source.value, cases: cases.value, ...terminal.value?.size() })
}

function stop() {
  if (runId) send({ type: 'stop', runId })
}

function clearOutput() {
  stop()
  terminal.value?.reset()
  hasOutput.value = false
  caseResults.value = []
  lastRun = null
  status.value = 'idle'
}

function input(data: string) {
  if (runId) send({ type: 'input', runId, data })
}

function handle(message: RunMessage) {
  record(message)
  switch (message.type) {
    case 'compiling':
      status.value = 'compiling'
      break
    case 'running':
      status.value = 'running'
      casesStarted++
      if (cases.value.length > 1) writeCaseHeader(message.case)
      terminal.value?.setInteractive(message.interactive)
      break
    case 'caseFinished':
      writeOutcome(message.outcome)
      if (message.verdict) writeVerdict(message.verdict)
      caseResults.value[message.case] = caseResult(message.case, message.outcome, message.verdict)
      break
    case 'output':
      write(message.stream === 'stderr' ? `\x1b[31m${message.text}\x1b[0m` : message.text)
      break
    case 'finished':
      status.value = 'done'
      terminal.value?.setInteractive(false)
      if (casesStarted === 0) writeOutcome(message.outcome) // compile error: no case ran
      compileMs.value = message.compileMs
      runMs.value = message.runMs
      offRun(message.runId)
      runId = null
      settle?.(passed(message.outcome))
      settle = null
  }
}

function write(text: string) {
  if (!text) return
  terminal.value?.write(text)
  endsWithNewline = text.endsWith('\n')
}

function writeOutcome(outcome: Outcome) {
  const note = describe(outcome)
  if (!note) return
  const color = outcome.kind === 'exited' || outcome.kind === 'stopped' ? 33 : 31
  write(`${endsWithNewline ? '' : '\r\n'}\x1b[${color}m[${note}]\x1b[0m`)
}

function record(message: RunMessage) {
  if (!recording) return
  const { runId: _, ...recorded } = message
  recording.events.push({ at: Math.round(performance.now() - recordingStart), message: recorded as RecordedMessage })
  if (message.type === 'finished') {
    lastRun = recording
    recording = null
  }
}

/** The run on screen still matches the code and input shown. */
function currentRun() {
  const unchanged = lastRun && lastRun.source === source.value && JSON.stringify(lastRun.cases) === JSON.stringify(cases.value)
  return unchanged ? lastRun : null
}

function writeCaseHeader(index: number) {
  const separator = index === 0 ? '' : endsWithNewline ? '\r\n' : '\r\n\r\n'
  write(`${separator}\x1b[2m── ${t('caseNumber', { n: index + 1 })} ──\x1b[0m\r\n`)
}

function writeVerdict(verdict: Verdict) {
  const line = verdict.kind === 'accepted'
    ? `\x1b[32m✓ ${t('accepted')}\x1b[0m`
    : `\x1b[31m✗ ${t('wrongAnswer', {
        line: verdict.line,
        expected: quoted(verdict.expected),
        actual: quoted(verdict.actual),
      })}\x1b[0m`
  write(`${endsWithNewline ? '' : '\r\n'}${line}\r\n`)
}

function quoted(line: string | null) {
  return line === null ? t('endOfOutput') : `«${line}»`
}

function passed(outcome: Outcome) {
  const allCasesRan = casesStarted === cases.value.length
  return outcome.kind === 'exited' && allCasesRan && caseResults.value.every((result) => result !== 'rejected')
}

function caseResult(index: number, outcome: Outcome, verdict: Verdict | null): CaseResult {
  if (verdict) return verdict.kind === 'accepted' ? 'accepted' : 'rejected'
  const clean = outcome.kind === 'exited' && outcome.code === 0
  return clean && cases.value[index]?.expected === null ? 'ran' : 'rejected'
}

function describe(outcome: Outcome): string {
  switch (outcome.kind) {
    case 'exited': return outcome.code === 0 ? '' : t('exitCode', { code: outcome.code })
    case 'signaled': return outcome.signal
    case 'compileFailed': return t('compileFailed')
    case 'timedOut': return t(outcome.phase === 'compile' ? 'compileTimedOut' : 'runTimedOut')
    case 'outputLimitExceeded': return t('outputLimit', { limit: outcome.limit })
    case 'stopped': return t('stopped')
    case 'error': return outcome.message
  }
}

function rows(text: string) {
  return Math.max(1, Math.min(8, text.replace(/\n$/, '').split('\n').length))
}

function seconds(ms: number) {
  return `${(ms / 1000).toFixed(2)} s`
}

function reset() {
  source.value = props.cell.source
  cases.value = copyCases()
}

function openInEditor() {
  send({ type: 'openEditor', line: props.cell.line })
}

const { registerExportable, unregisterExportable } = useExport()
registerExportable(props.cell.id, {
  edits: () => ({ source: source.value, cases: cases.value.map((c) => ({ ...c })) }),
  lastRun: currentRun,
})

const { registerRunner, unregisterRunner } = useRunAll()
registerRunner(props.cell.id, { runsAlone: () => cases.value.length > 0, run: runAndWait, line: props.cell.line })

onBeforeUnmount(() => {
  unregisterRunner(props.cell.id)
  unregisterExportable(props.cell.id)
  settle?.(false)
  stop()
  if (runId) offRun(runId)
})
</script>

<template>
  <section
    class="code-cell"
    :class="{ busy, zoomed }"
    :data-cell-id="cell.id"
    :style="zoomed ? { '--cell-scale': scale } : undefined"
  >
    <!-- Two comets travel along the ring around the body while the cell runs. -->
    <div class="star star-bottom" aria-hidden="true" />
    <div class="star star-top" aria-hidden="true" />
    <div class="cell-body">
      <header class="cell-toolbar">
        <span class="language">{{ cell.fenceLanguage }}</span>
        <span v-if="cell.flags.length" class="chip" :title="t('flags')">
          <AppIcon name="flag" />{{ cell.flags.join(' ') }}
        </span>
        <span class="spacer" />
        <span v-if="busy" class="chip">
          <span class="shiny">{{ t(status === 'compiling' ? 'compiling' : 'running') }}</span>
        </span>
        <span v-if="compileMs != null" class="chip" :title="t('compileTime')">
          <AppIcon name="hammer" /><CountUp :value="compileMs" :format="seconds" />
        </span>
        <span v-if="runMs != null" class="chip" :title="t('runTime')">
          <AppIcon name="timer" /><CountUp :value="runMs" :format="seconds" />
        </span>
        <button v-if="!staticLesson" class="tool" :title="t('openInEditor', { line: cell.line })" @click="openInEditor">
          <AppIcon name="pencil" />
        </button>
        <button v-if="modified" class="tool" :title="t('undoEdit')" @click="reset">
          <AppIcon name="undo" />
        </button>
        <button v-if="zoomed" class="tool" :title="t('zoomOut')" @click="closeZoom">
          <AppIcon name="minimize" />
        </button>
        <button v-else class="tool" :title="t('zoomIn')" @click="zoomCell(cell.id)">
          <AppIcon name="maximize" />
        </button>
        <button v-if="hasOutput && !busy" class="tool" :title="t('clearOutput')" @click="clearOutput">
          <AppIcon name="eraser" />
        </button>
        <button v-if="busy" class="tool stop" :title="t('stop')" @click="stop">
          <AppIcon name="stop" />
        </button>
        <button v-else ref="runButton" class="tool run" :title="t('run')" @click="runWithSpark">
          <AppIcon name="play" />
        </button>
      </header>
      <button v-if="beforeLines" class="hidden-code" :aria-expanded="showBefore" @click="showBefore = !showBefore">
        <AppIcon name="chevronRight" />{{ hiddenLabel(beforeLines, 'Above') }}
      </button>
      <CodeEditor
        v-if="showBefore"
        class="hidden-part"
        :model-value="cell.before.replace(/\n$/, '')"
        :language="cell.fenceLanguage"
        readonly
      />
      <CodeEditor v-model="source" :language="cell.fenceLanguage" :first-line="beforeLines + 1" @run="runWithSpark()" />
      <CodeEditor
        v-if="showAfter"
        class="hidden-part"
        :model-value="cell.after.replace(/\n$/, '')"
        :language="cell.fenceLanguage"
        :first-line="beforeLines + sourceLines + 1"
        readonly
      />
      <button v-if="afterLines" class="hidden-code below" :aria-expanded="showAfter" @click="showAfter = !showAfter">
        <AppIcon name="chevronRight" />{{ hiddenLabel(afterLines, 'Below') }}
      </button>
      <div v-for="(testCase, index) in cases" :key="index" class="case">
        <span v-if="judged" class="case-mark" :class="caseResults[index]" :title="t('caseNumber', { n: index + 1 })">
          <AppIcon v-if="caseResults[index] === 'accepted'" name="check" />
          <AppIcon v-else-if="caseResults[index] === 'rejected'" name="x" />
          <template v-else>{{ index + 1 }}</template>
        </span>
        <label class="stdin" :title="t('stdin')">
          <AppIcon name="keyboard" class="stdin-icon" />
          <textarea v-model="testCase.stdin" :rows="rows(testCase.stdin)" spellcheck="false" :aria-label="t('stdin')" />
        </label>
        <label v-if="testCase.expected !== null" class="stdin expected" :title="t('expectedOutput')">
          <AppIcon name="equal" class="stdin-icon" />
          <textarea v-model="testCase.expected" :rows="rows(testCase.expected)" spellcheck="false" :aria-label="t('expectedOutput')" />
        </label>
      </div>
      <TerminalOutput v-show="hasOutput" ref="terminal" :scale="scale" @input="input" />
    </div>
  </section>
</template>
