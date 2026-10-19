<script setup lang="ts">
import { Terminal, type ITheme } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'

const MIN_ROWS = 2
/** The terminal grows with the output, with no scrollbar; this only stops a runaway program. */
const MAX_ROWS = 1000
/** What the program sees as the terminal height (PTY). */
const PROGRAM_ROWS = 24
const BASE_FONT_PX = 15
const PRINT_FONT_PX = 12
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

/** Font multiplier while the block is zoomed. */
const props = withDefaults(defineProps<{ scale?: number }>(), { scale: 1 })
const emit = defineEmits<{ input: [data: string] }>()
const { fontScale, codeFontFamily } = usePreferences()
const { printing } = usePrint()

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

watch([fontScale, () => props.scale], () => {
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
  return printing.value ? PRINT_FONT_PX : Math.round(BASE_FONT_PX * fontScale.value * props.scale)
}

/** Printing changes the font size: refit, keeping every line visible. */
watch(printing, () => {
  if (!terminal) return
  terminal.options.fontSize = fontSize()
  terminal.resize(terminal.cols, Math.max(MIN_ROWS, Math.min(MAX_ROWS, linesUsed())))
  terminal.scrollToTop()
  fitWidth()
})

function linesUsed() {
  const buffer = terminal!.buffer.active
  return buffer.baseY + buffer.cursorY + 1
}

function fitWidth() {
  const cols = fit?.proposeDimensions()?.cols
  if (terminal && cols && cols !== terminal.cols) terminal.resize(cols, terminal.rows)
}

/** Grow with the output instead of showing a fixed-height box that scrolls. */
function growToContent() {
  if (!terminal) return
  const rows = Math.min(MAX_ROWS, Math.max(MIN_ROWS, linesUsed()))
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

/** What the terminal shows, as plain text (colors dropped). */
function text() {
  if (!terminal) return ''
  const buffer = terminal.buffer.active
  const lines: string[] = []
  for (let i = 0; i < buffer.length; i++) lines.push(buffer.getLine(i)?.translateToString(true) ?? '')
  while (lines.length && !lines.at(-1)!.trim()) lines.pop()
  return lines.join('\n')
}

function size() {
  return { cols: terminal?.cols ?? 80, rows: PROGRAM_ROWS }
}

defineExpose({ reset, setInteractive, write, size, text })
</script>

<template>
  <div class="output">
    <div ref="container" />
  </div>
</template>
