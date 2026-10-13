<script setup lang="ts">
import { Terminal, type ITheme } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'

const MIN_ROWS = 2
const MAX_ROWS = 24
const BASE_FONT_PX = 15
const SHOW_CURSOR = '\x1b[?25h'
const HIDE_CURSOR = '\x1b[?25l'

/** Always a black terminal, whatever the page theme. */
const THEME: ITheme = {
  background: '#000000',
  foreground: '#e8e8e8',
  cursor: '#e8e8e8',
  selectionBackground: '#3a3a3a',
  black: '#000000',
  red: '#ff5f56',
  green: '#a6e22e',
  yellow: '#e6db74',
  blue: '#66d9ef',
  magenta: '#f92672',
  cyan: '#4fd6be',
  white: '#e8e8e8',
  brightBlack: '#75715e',
  brightRed: '#ff8178',
  brightGreen: '#c4f060',
  brightYellow: '#f4ea9a',
  brightBlue: '#9ae7f7',
  brightMagenta: '#fc6a9f',
  brightCyan: '#86ead9',
  brightWhite: '#ffffff',
}

const emit = defineEmits<{ input: [data: string] }>()
const { fontScale, codeFontFamily } = usePreferences()

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
    fontSize: fontSize(),
    fontFamily: codeFontFamily.value,
    theme: THEME,
  })
  fit = new FitAddon()
  terminal.loadAddon(fit)
  terminal.open(container.value!)
  terminal.write(HIDE_CURSOR)
  terminal.onData((data) => {
    if (interactive) emit('input', data)
  })
  terminal.onWriteParsed(growToContent)
  resizeObserver = new ResizeObserver(fitWidth)
  resizeObserver.observe(container.value!)
  fitWidth()
})

watch(fontScale, () => {
  if (!terminal) return
  terminal.options.fontSize = fontSize()
  fitWidth()
})

// xterm measures the glyphs when the family is set: wait for the web font first.
watch(codeFontFamily, async (family) => {
  await document.fonts.load(`${fontSize()}px ${family}`).catch(() => {})
  if (!terminal) return
  terminal.options.fontFamily = family
  fitWidth()
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  terminal?.dispose()
})

function fontSize() {
  return Math.round(BASE_FONT_PX * fontScale.value)
}

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

function setInteractive(value: boolean) {
  interactive = value
  if (!terminal) return
  terminal.write(value ? SHOW_CURSOR : HIDE_CURSOR)
  // A pseudo-terminal already turns \n into \r\n; pipes do not.
  terminal.options.convertEol = !value
  terminal.options.disableStdin = !value
  if (value) terminal.focus()
  else terminal.blur() // give the keys back to page navigation
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
