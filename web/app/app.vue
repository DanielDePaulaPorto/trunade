<script setup lang="ts">
const { snapshot, connected, lastError, activeRuns } = useConnection()
const { growFont, shrinkFont, resetFont, cycleTheme, toggleLayoutWidth } = usePreferences()
const { t } = useI18n()
const { printLesson } = usePrint()
const { zoomedCell, toggleZoom, closeZoom } = useZoom()

const settingsOpen = ref(false)
const topicsOpen = ref(false)

watch(
  () => snapshot.value?.fileName,
  (name) => useHead({ title: name ? `${name} · Trunade!` : 'Trunade!' }),
)

const SHORTCUTS: Record<string, (event: KeyboardEvent) => void> = {
  t: () => (topicsOpen.value = true),
  ' ': (event) => scrollToAdjacentSection(event.shiftKey ? -1 : 1),
  ArrowDown: () => scrollToAdjacentSection(1),
  ArrowUp: () => scrollToAdjacentSection(-1),
  PageDown: () => scrollToAdjacentSection(1),
  PageUp: () => scrollToAdjacentSection(-1),
  '+': growFont,
  '=': growFont,
  '-': shrinkFont,
  '0': resetFont,
  d: cycleTheme,
  w: toggleLayoutWidth,
  f: toggleFullscreen,
  o: () => (settingsOpen.value = true),
  z: toggleZoom,
}

/** With a block zoomed, moving through the page makes no sense. */
const ZOOM_SHORTCUTS = new Set(['z', 'f', 'd', '+', '=', '-', '0'])

/** Fullscreen for projecting; the browser's own Esc also leaves it. */
function toggleFullscreen() {
  if (document.fullscreenElement) document.exitFullscreen()
  else document.documentElement.requestFullscreen().catch(() => {})
}

/** Keys typed into code, stdin or a running program are not shortcuts. */
function isTyping(target: EventTarget | null) {
  return target instanceof Element && !!target.closest('input, textarea, select, [contenteditable], .cm-editor, .xterm')
}

function onKey(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'p') {
    event.preventDefault()
    printLesson(snapshot.value?.fileName)
    return
  }
  if (event.metaKey || event.ctrlKey || event.altKey || event.defaultPrevented) return
  if (zoomedCell.value && event.key === 'Escape') return closeZoom()
  if (settingsOpen.value || topicsOpen.value || isTyping(event.target)) return
  const action = SHORTCUTS[event.key]
  if (!action || (zoomedCell.value && !ZOOM_SHORTCUTS.has(event.key))) return
  event.preventDefault()
  action(event)
}

/** Close the panel first so it is not in the PDF. */
async function exportPdf() {
  settingsOpen.value = false
  await nextTick()
  printLesson(snapshot.value?.fileName)
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="page">
    <button class="menu-button" :title="t('settings')" @click="settingsOpen = true">
      <AppIcon name="menu" />
    </button>
    <p v-if="!connected" class="offline">{{ t('reconnecting') }}</p>

    <main class="document">
      <header class="masthead">
        <TrunadeSymbol :running="activeRuns > 0" :size="40" />
        <TrunadeName />
        <span class="file">{{ snapshot?.fileName }}</span>
      </header>
      <p v-if="snapshot?.error" class="banner">{{ snapshot.error }}</p>
      <p v-if="lastError" class="banner" @click="lastError = null">{{ lastError }}</p>
      <CellView v-for="cell in snapshot?.cells ?? []" :key="cell.id" :cell="cell" />
      <EndMark v-if="snapshot?.cells.length" />
    </main>

    <Transition name="slide">
      <SettingsPanel v-if="settingsOpen" @close="settingsOpen = false" @print="exportPdf" />
    </Transition>
    <Transition name="fade">
      <TopicsModal v-if="topicsOpen" @close="topicsOpen = false" />
    </Transition>
  </div>
</template>
