<script setup lang="ts">
const { snapshot, connected, lastError, activeRuns } = useConnection()
const { growFont, shrinkFont, resetFont, cycleTheme } = usePreferences()
const { t } = useI18n()

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
  '+': growFont,
  '=': growFont,
  '-': shrinkFont,
  '0': resetFont,
  d: cycleTheme,
  s: () => (settingsOpen.value = true),
}

/** Keys typed into code, stdin or a running program are not shortcuts. */
function isTyping(target: EventTarget | null) {
  return target instanceof Element && !!target.closest('input, textarea, select, [contenteditable], .cm-editor, .xterm')
}

function onKey(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey || event.altKey || event.defaultPrevented) return
  if (settingsOpen.value || topicsOpen.value || isTyping(event.target)) return
  const action = SHORTCUTS[event.key]
  if (!action) return
  event.preventDefault()
  action(event)
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
        <SkullMark :running="activeRuns > 0" :size="34" />
        <span class="wordmark">Trunade!</span>
        <span class="file">{{ snapshot?.fileName }}</span>
      </header>
      <p v-if="snapshot?.error" class="banner">{{ snapshot.error }}</p>
      <p v-if="lastError" class="banner" @click="lastError = null">{{ lastError }}</p>
      <template v-for="cell in snapshot?.cells ?? []" :key="cell.id">
        <MarkdownCell v-if="cell.kind === 'markdown'" :html="cell.html" />
        <CodeCell v-else-if="cell.kind === 'code'" :cell="cell" />
        <QuizCell v-else :cell="cell" />
      </template>
    </main>

    <Transition name="slide">
      <SettingsPanel v-if="settingsOpen" @close="settingsOpen = false" />
    </Transition>
    <Transition name="fade">
      <TopicsModal v-if="topicsOpen" @close="topicsOpen = false" />
    </Transition>
  </div>
</template>
