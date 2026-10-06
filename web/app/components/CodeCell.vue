<script setup lang="ts">
import type { CodeCell, Outcome, RunMessage } from '~/composables/types'
import TerminalOutput from './TerminalOutput.vue'

const props = defineProps<{ cell: CodeCell }>()
const { send, onRun, offRun } = useConnection()

type Status = 'idle' | 'compiling' | 'running' | 'done'

const source = ref(props.cell.source)
const stdin = ref(props.cell.stdin)
const status = ref<Status>('idle')
const summary = ref('')
const hasOutput = ref(false)
const terminal = ref<InstanceType<typeof TerminalOutput>>()
let runId: string | null = null
let endsWithNewline = true

const modified = computed(
  () => source.value !== props.cell.source || stdin.value !== props.cell.stdin,
)
const busy = computed(() => status.value === 'compiling' || status.value === 'running')

function run() {
  if (busy.value) stop()
  runId = crypto.randomUUID()
  hasOutput.value = true
  summary.value = ''
  endsWithNewline = true
  status.value = 'compiling'
  terminal.value?.reset()
  onRun(runId, handle)
  send({ type: 'run', runId, cellId: props.cell.id, source: source.value, stdin: stdin.value, ...terminal.value?.size() })
}

function stop() {
  if (runId) send({ type: 'stop', runId })
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
      summary.value = timings(message.compileMs, message.runMs)
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
    case 'exited': return outcome.code === 0 ? '' : `código de saída ${outcome.code}`
    case 'signaled': return outcome.signal
    case 'compileFailed': return 'falha na compilação'
    case 'timedOut': return outcome.phase === 'compile' ? 'compilação excedeu o tempo' : 'tempo limite excedido'
    case 'outputLimitExceeded': return `saída excedeu ${outcome.limit} bytes`
    case 'stopped': return 'interrompido'
    case 'error': return outcome.message
  }
}

function timings(compileMs: number | null, runMs: number | null): string {
  const seconds = (ms: number) => `${(ms / 1000).toFixed(2)} s`
  return [
    compileMs != null && `compilou em ${seconds(compileMs)}`,
    runMs != null && `rodou em ${seconds(runMs)}`,
  ].filter(Boolean).join(' · ')
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

const statusLabel = computed(() => ({
  idle: '',
  compiling: 'compilando…',
  running: 'executando…',
  done: summary.value,
})[status.value])
</script>

<template>
  <section class="code-cell" :class="{ busy }">
    <!-- Two glows travel along the 2px gap around the body while the cell runs. -->
    <div class="star star-bottom" aria-hidden="true" />
    <div class="star star-top" aria-hidden="true" />
    <div class="cell-body">
      <header class="cell-toolbar">
        <span class="language">{{ cell.fenceLanguage }}</span>
        <button class="link" title="Abrir no editor" @click="openInEditor">linha {{ cell.line }}</button>
        <span v-if="cell.flags.length" class="flags">{{ cell.flags.join(' ') }}</span>
        <span class="spacer" />
        <span class="status">{{ statusLabel }}</span>
        <button v-if="modified" class="secondary" title="Voltar ao texto do arquivo" @click="reset">desfazer edição</button>
        <button v-if="busy" class="stop" title="Interromper" @click="stop">■</button>
        <button v-else class="run" title="Executar (Shift+Enter)" @click="run">▶</button>
      </header>
      <CodeEditor v-model="source" :language="cell.fenceLanguage" @run="run" />
      <label v-if="stdin !== null" class="stdin">
        <span>entrada (stdin)</span>
        <textarea v-model="stdin" :rows="Math.min(8, stdin.split('\n').length)" spellcheck="false" />
      </label>
      <TerminalOutput v-show="hasOutput" ref="terminal" @input="input" />
    </div>
  </section>
</template>
