<script setup lang="ts">
import type { Locale, ThemeMode } from '~/composables/usePreferences'
import type { IconName } from '~/composables/icons'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const { themeMode, locale, fontScale, growFont, shrinkFont, resetFont } = usePreferences()

const THEMES: { mode: ThemeMode; icon: IconName; label: 'themeAuto' | 'themeLight' | 'themeDark' }[] = [
  { mode: 'auto', icon: 'monitor', label: 'themeAuto' },
  { mode: 'light', icon: 'sun', label: 'themeLight' },
  { mode: 'dark', icon: 'moon', label: 'themeDark' },
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
  { keys: ['S'], label: 'shortcutSettings' },
  { keys: ['⇧ Enter'], label: 'shortcutRun' },
  { keys: ['Esc'], label: 'shortcutLeave' },
] as const)

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <aside class="settings" :aria-label="t('settings')">
      <header class="settings-header">
        <strong>{{ t('settings') }}</strong>
        <button class="tool" :title="t('close')" @click="emit('close')"><AppIcon name="close" /></button>
      </header>

      <section>
        <h3><AppIcon name="type" />{{ t('fontSize') }}</h3>
        <div class="stepper">
          <button class="tool" @click="shrinkFont"><AppIcon name="minus" /></button>
          <span class="stepper-value">{{ Math.round(fontScale * 100) }}%</span>
          <button class="tool" @click="growFont"><AppIcon name="plus" /></button>
          <button class="tool" :disabled="fontScale === 1" @click="resetFont"><AppIcon name="reset" /></button>
        </div>
        <p class="hint">{{ t('fontSizeHint') }}</p>
      </section>

      <section>
        <h3><AppIcon name="sun" />{{ t('theme') }}</h3>
        <div class="segmented">
          <button v-for="option in THEMES" :key="option.mode" :class="{ active: themeMode === option.mode }" @click="themeMode = option.mode">
            <AppIcon :name="option.icon" />{{ t(option.label) }}
          </button>
        </div>
      </section>

      <section>
        <h3><AppIcon name="globe" />{{ t('language') }}</h3>
        <div class="segmented">
          <button v-for="option in LOCALES" :key="option.value" :class="{ active: locale === option.value }" @click="locale = option.value">
            {{ option.label }}
          </button>
        </div>
      </section>

      <section>
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
