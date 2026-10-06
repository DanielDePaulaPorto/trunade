<script setup lang="ts">
import type { CodeCell, Outcome, RunMessage } from '~/composables/types'
import TerminalOutput from './TerminalOutput.vue'

const props = defineProps<{ cell: CodeCell }>()
const { send, onRun, offRun } = useConnection()
const { t } = useI18n()

type Status = 'idle' | 'compiling' | 'running' | 'done'

const source = ref(props.cell.source)
const stdin = ref(props.cell.stdin)
const status = ref<Status>('idle')
const compileMs = ref<number | null>(null)
const runMs = ref<number | null>(null)
const hasOutput = ref(false)
const terminal = ref<InstanceType<typeof TerminalOutput>>()
const runButton = ref<HTMLButtonElement>()
let runId: string | null = null
let endsWithNewline = true

const modified = computed(
  () => source.value !== props.cell.source || stdin.value !== props.cell.stdin,
)
const busy = computed(() => status.value === 'compiling' || status.value === 'running')

/** Run with a spark at the click, or at the ▶ button when started from the keyboard. */
function runWithSpark(event?: MouseEvent) {
  const origin = event ?? runButton.value
  if (origin) spark(origin)
  run()
}

function run() {
  if (busy.value) stop()
  runId = crypto.randomUUID()
  hasOutput.value = true
  compileMs.value = null
  runMs.value = null
  endsWithNewline = true
  status.value = 'compiling'
  terminal.value?.reset()
  onRun(runId, handle)
  send({ type: 'run', runId, cellId: props.cell.id, source: source.value, stdin: stdin.value, ...terminal.value?.size() })
}

function stop() {
  if (runId) send({ type: 'stop', runId })
}

function clearOutput() {
  stop()
  terminal.value?.reset()
  hasOutput.value = false
  status.value = 'idle'
}

function input(data: string) {
  if (runId) send({ type: 'input', runId, data })
}

function handle(message: RunMessage) {
  switch (message.type) {
    case 'compiling':
      status.value = 'compiling'
      break
    case 'running':
      status.value = 'running'
      terminal.value?.setInteractive(message.interactive)
      break
    case 'output':
      write(message.stream === 'stderr' ? `\x1b[31m${message.text}\x1b[0m` : message.text)
      break
    case 'finished':
      status.value = 'done'
      terminal.value?.setInteractive(false)
      writeOutcome(message.outcome)
      compileMs.value = message.compileMs
      runMs.value = message.runMs
      offRun(message.runId)
      runId = null
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

function seconds(ms: number) {
  return `${(ms / 1000).toFixed(2)} s`
}

function reset() {
  source.value = props.cell.source
  stdin.value = props.cell.stdin
}

function openInEditor() {
  send({ type: 'openEditor', line: props.cell.line })
}

onBeforeUnmount(() => {
  stop()
  if (runId) offRun(runId)
})
</script>

<template>
  <section class="code-cell" :class="{ busy }">
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
          <AppIcon name="spinner" class="spin" />
          <span class="shiny">{{ t(status === 'compiling' ? 'compiling' : 'running') }}</span>
        </span>
        <span v-if="compileMs != null" class="chip" :title="t('compileTime')">
          <AppIcon name="hammer" /><CountUp :value="compileMs" :format="seconds" />
        </span>
        <span v-if="runMs != null" class="chip" :title="t('runTime')">
          <AppIcon name="timer" /><CountUp :value="runMs" :format="seconds" />
        </span>
        <button class="tool" :title="t('openInEditor', { line: cell.line })" @click="openInEditor">
          <AppIcon name="pencil" />
        </button>
        <button v-if="modified" class="tool" :title="t('undoEdit')" @click="reset">
          <AppIcon name="undo" />
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
      <CodeEditor v-model="source" :language="cell.fenceLanguage" @run="runWithSpark()" />
      <label v-if="stdin !== null" class="stdin" :title="t('stdin')">
        <AppIcon name="keyboard" class="stdin-icon" />
        <textarea v-model="stdin" :rows="Math.min(8, stdin.split('\n').length)" spellcheck="false" :aria-label="t('stdin')" />
      </label>
      <TerminalOutput v-show="hasOutput" ref="terminal" @input="input" />
    </div>
  </section>
</template>
