<script setup lang="ts">
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { TERMINAL_THEMES } from '~/composables/palette'

const MIN_ROWS = 2
const MAX_ROWS = 24

const emit = defineEmits<{ input: [data: string] }>()
const { theme } = useTheme()

const container = ref<HTMLElement>()
let terminal: Terminal | null = null
let fit: FitAddon | null = null
let resizeObserver: ResizeObserver | null = null
let interactive = false

onMounted(() => {
  terminal = new Terminal({
    rows: MIN_ROWS,
    convertEol: true,
    disableStdin: true,
    cursorBlink: true,
    scrollback: 5000,
    fontSize: 15,
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
    theme: TERMINAL_THEMES[theme.value],
  })
  fit = new FitAddon()
  terminal.loadAddon(fit)
  terminal.open(container.value!)
  terminal.onData((data) => {
    if (interactive) emit('input', data)
  })
  terminal.onWriteParsed(growToContent)
  resizeObserver = new ResizeObserver(fitWidth)
  resizeObserver.observe(container.value!)
  fitWidth()
})

watch(theme, (value) => {
  if (terminal) terminal.options.theme = TERMINAL_THEMES[value]
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  terminal?.dispose()
})

function fitWidth() {
  const cols = fit?.proposeDimensions()?.cols
  if (terminal && cols && cols !== terminal.cols) terminal.resize(cols, terminal.rows)
}

/** Grow with the output (up to MAX_ROWS) instead of showing a fixed-height box. */
function growToContent() {
  if (!terminal) return
  const buffer = terminal.buffer.active
  const used = buffer.baseY + buffer.cursorY + 1
  const rows = Math.min(MAX_ROWS, Math.max(MIN_ROWS, used))
  if (rows > terminal.rows) terminal.resize(terminal.cols, rows)
}

function reset() {
  terminal?.reset()
  terminal?.write(HIDE_CURSOR)
  terminal?.resize(terminal.cols, MIN_ROWS)
  setInteractive(false)
}

const SHOW_CURSOR = '\x1b[?25h'
const HIDE_CURSOR = '\x1b[?25l'

function setInteractive(value: boolean) {
  interactive = value
  if (!terminal) return
  terminal.write(value ? SHOW_CURSOR : HIDE_CURSOR)
  // A pseudo-terminal already turns \n into \r\n; pipes do not.
  terminal.options.convertEol = !value
  terminal.options.disableStdin = !value
  if (value) terminal.focus()
}

function write(text: string) {
  terminal?.write(text)
}

function size() {
  return { cols: terminal?.cols ?? 80, rows: MAX_ROWS }
}

defineExpose({ reset, setInteractive, write, size })
</script>

<template>
  <div class="output">
    <div ref="container" />
  </div>
</template>
