<script setup lang="ts">
import type { CodeThemeMode, LayoutWidth, Locale, ThemeMode } from '~/composables/usePreferences'
import type { IconName } from '~/composables/icons'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const { themeMode, codeThemeMode, layoutWidth, locale, fontScale, growFont, shrinkFont, resetFont } = usePreferences()

const THEMES: { mode: ThemeMode; icon: IconName; label: 'themeAuto' | 'themeLight' | 'themeDark' }[] = [
  { mode: 'auto', icon: 'monitor', label: 'themeAuto' },
  { mode: 'light', icon: 'sun', label: 'themeLight' },
  { mode: 'dark', icon: 'moon', label: 'themeDark' },
]
const WIDTHS: { value: LayoutWidth; icon: IconName; label: 'widthNarrow' | 'widthWide' }[] = [
  { value: 'narrow', icon: 'narrow', label: 'widthNarrow' },
  { value: 'wide', icon: 'wide', label: 'widthWide' },
]
const CODE_THEMES: { mode: CodeThemeMode; label: 'codeAuto' | 'codeLight' | 'codeDark' }[] = [
  { mode: 'auto', label: 'codeAuto' },
  { mode: 'light', label: 'codeLight' },
  { mode: 'dark', label: 'codeDark' },
]
const LOCALES: { value: Locale; label: string }[] = [
  { value: 'pt', label: 'Português' },
  { value: 'en', label: 'English' },
]
const SHORTCUTS = computed(() => [
  { keys: ['T'], label: 'shortcutTopics' },
  { keys: [t('keySpace'), '↓'], label: 'shortcutNext' },
  { keys: [`⇧ ${t('keySpace')}`, '↑'], label: 'shortcutPrevious' },
  { keys: ['+', '−', '0'], label: 'shortcutFont' },
  { keys: ['D'], label: 'shortcutTheme' },
  { keys: ['W'], label: 'shortcutWidth' },
  { keys: ['F'], label: 'shortcutFullscreen' },
  { keys: ['O'], label: 'shortcutSettings' },
  { keys: ['⇧ Enter'], label: 'shortcutRun' },
  { keys: ['Esc'], label: 'shortcutLeave' },
] as const)

function onKey(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey || event.altKey) return
  if (event.key === 'Escape' || event.key === 'o') emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <aside class="settings" :aria-label="t('settings')">
      <header class="settings-header" style="--i: 1">
        <strong>{{ t('settings') }}</strong>
        <button class="tool" :title="t('close')" @click="emit('close')"><AppIcon name="close" /></button>
      </header>

      <section style="--i: 2">
        <h3><AppIcon name="type" />{{ t('fontSize') }}</h3>
        <div class="stepper">
          <button class="tool" @click="shrinkFont"><AppIcon name="minus" /></button>
          <span class="stepper-value">{{ Math.round(fontScale * 100) }}%</span>
          <button class="tool" @click="growFont"><AppIcon name="plus" /></button>
          <button class="tool" :disabled="fontScale === 1" @click="resetFont"><AppIcon name="reset" /></button>
        </div>
        <p class="hint">{{ t('fontSizeHint') }}</p>
      </section>

      <section style="--i: 3">
        <h3><AppIcon name="sun" />{{ t('theme') }}</h3>
        <div class="segmented">
          <button v-for="option in THEMES" :key="option.mode" :class="{ active: themeMode === option.mode }" @click="themeMode = option.mode">
            <AppIcon :name="option.icon" />{{ t(option.label) }}
          </button>
        </div>
      </section>

      <section style="--i: 4">
        <h3><AppIcon name="wide" />{{ t('textWidth') }}</h3>
        <div class="segmented">
          <button v-for="option in WIDTHS" :key="option.value" :class="{ active: layoutWidth === option.value }" @click="layoutWidth = option.value">
            <AppIcon :name="option.icon" />{{ t(option.label) }}
          </button>
        </div>
      </section>

      <section style="--i: 5">
        <h3><AppIcon name="code" />{{ t('codeColors') }}</h3>
        <div class="segmented">
          <button v-for="option in CODE_THEMES" :key="option.mode" :class="{ active: codeThemeMode === option.mode }" @click="codeThemeMode = option.mode">
            {{ t(option.label) }}
          </button>
        </div>
      </section>

      <section style="--i: 6">
        <h3><AppIcon name="globe" />{{ t('language') }}</h3>
        <div class="segmented">
          <button v-for="option in LOCALES" :key="option.value" :class="{ active: locale === option.value }" @click="locale = option.value">
            {{ option.label }}
          </button>
        </div>
      </section>

      <section style="--i: 7">
        <h3><AppIcon name="keyboard" />{{ t('shortcuts') }}</h3>
        <dl class="shortcuts">
          <template v-for="shortcut in SHORTCUTS" :key="shortcut.label">
            <dt><kbd v-for="key in shortcut.keys" :key="key">{{ key }}</kbd></dt>
            <dd>{{ t(shortcut.label) }}</dd>
          </template>
        </dl>
      </section>
    </aside>
  </div>
</template>
