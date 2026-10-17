<script setup lang="ts">
import type { CodeThemeMode, LayoutWidth, Locale, ThemeMode } from '~/composables/usePreferences'
import type { IconName } from '~/composables/icons'
import { BODY_FONTS, CODE_FONTS, TITLE_FONTS, type FontChoice } from '~/composables/fonts'
import { staticLesson } from '~/composables/useStatic'

export interface HtmlExportOptions {
  runFirst: boolean
  theme: 'light' | 'dark'
}

const emit = defineEmits<{ close: []; print: []; runAll: []; exportHtml: [options: HtmlExportOptions] }>()
const runBeforeExport = ref(true)
const exportTheme = ref<'light' | 'dark'>('light')
const { t } = useI18n()
const { themeMode, codeThemeMode, layoutWidth, locale, fontScale, titleFont, bodyFont, codeFont, growFont, shrinkFont, resetFont } =
  usePreferences()

interface FontGroup {
  label: 'fontTitles' | 'fontBody' | 'fontCode'
  fonts: Record<string, FontChoice>
  model: Ref<string>
}
const FONT_GROUPS: FontGroup[] = [
  { label: 'fontTitles', fonts: TITLE_FONTS, model: titleFont as Ref<string> },
  { label: 'fontBody', fonts: BODY_FONTS, model: bodyFont as Ref<string> },
  { label: 'fontCode', fonts: CODE_FONTS, model: codeFont as Ref<string> },
]

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
  { keys: [t('keySpace'), '↓', 'PgDn'], label: 'shortcutNext' },
  { keys: [`⇧ ${t('keySpace')}`, '↑', 'PgUp'], label: 'shortcutPrevious' },
  { keys: ['+', '−', '0'], label: 'shortcutFont' },
  { keys: ['D'], label: 'shortcutTheme' },
  { keys: ['W'], label: 'shortcutWidth' },
  { keys: ['F'], label: 'shortcutFullscreen' },
  { keys: ['Z'], label: 'shortcutZoom' },
  { keys: ['R'], label: 'shortcutRunAll' },
  { keys: ['O'], label: 'shortcutSettings' },
  { keys: ['⇧ Enter'], label: 'shortcutRun' },
  { keys: ['Esc'], label: 'shortcutLeave' },
] as const)

/* One group open at a time, so the panel fits without scrolling; the last one stays open next time. */
type Group = 'appearance' | 'fonts' | 'lesson' | 'language' | 'shortcuts'
const GROUP_KEY = 'trunade.settingsGroup'
const openGroup = ref<Group | null>(readOpenGroup())

function readOpenGroup(): Group | null {
  try {
    return (localStorage.getItem(GROUP_KEY) as Group | null) ?? 'appearance'
  } catch {
    return 'appearance'
  }
}

function toggleGroup(group: Group) {
  openGroup.value = openGroup.value === group ? null : group
  try {
    if (openGroup.value) localStorage.setItem(GROUP_KEY, openGroup.value)
    else localStorage.removeItem(GROUP_KEY)
  } catch {
    // only a convenience
  }
}

const GROUPS = computed(() => {
  const theme = THEMES.find((option) => option.mode === themeMode.value)!
  const width = WIDTHS.find((option) => option.value === layoutWidth.value)!
  return [
    { id: 'appearance', icon: 'sun', title: t('appearance'), summary: `${t(theme.label)} · ${t(width.label)} · ${Math.round(fontScale.value * 100)}%` },
    { id: 'fonts', icon: 'font', title: t('fonts'), summary: [TITLE_FONTS[titleFont.value], BODY_FONTS[bodyFont.value], CODE_FONTS[codeFont.value]].map((font) => font.label).join(' · ') },
    { id: 'lesson', icon: 'printer', title: t('runAndExport'), summary: t('runAndExportSummary') },
    { id: 'language', icon: 'globe', title: t('language'), summary: LOCALES.find((option) => option.value === locale.value)!.label },
    { id: 'shortcuts', icon: 'keyboard', title: t('shortcuts'), summary: t('shortcutsSummary') },
  ] as { id: Group; icon: IconName; title: string; summary: string }[]
})

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

      <section v-for="(group, index) in GROUPS" :key="group.id" class="settings-group" :style="{ '--i': index + 2 }">
        <button class="group-header" :aria-expanded="openGroup === group.id" @click="toggleGroup(group.id)">
          <span class="group-icon"><AppIcon :name="group.icon" /></span>
          <span class="group-text">
            <strong>{{ group.title }}</strong>
            <small>{{ group.summary }}</small>
          </span>
          <AppIcon name="chevronRight" class="group-chevron" />
        </button>

        <div v-if="openGroup === group.id" class="group-body">
          <template v-if="group.id === 'appearance'">
            <h3>{{ t('theme') }}</h3>
            <div class="segmented">
              <button v-for="option in THEMES" :key="option.mode" :class="{ active: themeMode === option.mode }" @click="themeMode = option.mode">
                <AppIcon :name="option.icon" />{{ t(option.label) }}
              </button>
            </div>
            <h3>{{ t('codeColors') }}</h3>
            <div class="segmented">
              <button v-for="option in CODE_THEMES" :key="option.mode" :class="{ active: codeThemeMode === option.mode }" @click="codeThemeMode = option.mode">
                {{ t(option.label) }}
              </button>
            </div>
            <h3>{{ t('textWidth') }}</h3>
            <div class="segmented">
              <button v-for="option in WIDTHS" :key="option.value" :class="{ active: layoutWidth === option.value }" @click="layoutWidth = option.value">
                <AppIcon :name="option.icon" />{{ t(option.label) }}
              </button>
            </div>
            <h3>{{ t('fontSize') }}</h3>
            <div class="stepper">
              <button class="tool" @click="shrinkFont"><AppIcon name="minus" /></button>
              <span class="stepper-value">{{ Math.round(fontScale * 100) }}%</span>
              <button class="tool" @click="growFont"><AppIcon name="plus" /></button>
              <button class="tool" :disabled="fontScale === 1" @click="resetFont"><AppIcon name="reset" /></button>
            </div>
          </template>

          <template v-else-if="group.id === 'fonts'">
            <div v-for="fontGroup in FONT_GROUPS" :key="fontGroup.label" class="font-group">
              <h3>{{ t(fontGroup.label) }}</h3>
              <div class="font-options">
                <button
                  v-for="(font, id) in fontGroup.fonts"
                  :key="id"
                  :class="{ active: fontGroup.model.value === id }"
                  :style="{ fontFamily: font.family, fontWeight: font.weight }"
                  @click="fontGroup.model.value = id"
                >
                  {{ font.label }}
                </button>
              </div>
            </div>
            <p class="hint">{{ t('preferencesSaved') }}</p>
          </template>

          <template v-else-if="group.id === 'lesson'">
            <button class="export-button" @click="emit('runAll')"><AppIcon name="play" />{{ t('runAll') }}</button>
            <p class="hint">{{ t('runAllHint') }}</p>
            <button class="export-button" @click="emit('print')"><AppIcon name="printer" />{{ t('exportPdf') }}</button>
            <p class="hint">{{ t('exportPdfHint') }}</p>
            <div class="export-html">
              <div class="segmented" role="group" :aria-label="t('exportHtmlTheme')">
                <button :class="{ active: exportTheme === 'light' }" @click="exportTheme = 'light'"><AppIcon name="sun" />{{ t('themeLight') }}</button>
                <button :class="{ active: exportTheme === 'dark' }" @click="exportTheme = 'dark'"><AppIcon name="moon" />{{ t('themeDark') }}</button>
              </div>
              <label class="check-option"><input v-model="runBeforeExport" type="checkbox">{{ t('exportRunFirst') }}</label>
              <button class="export-button" @click="emit('exportHtml', { runFirst: runBeforeExport, theme: exportTheme })">
                <AppIcon name="code" />{{ t('exportHtml') }}
              </button>
            </div>
            <p class="hint">{{ t('exportHtmlHint') }}</p>
          </template>

          <div v-else-if="group.id === 'language'" class="segmented">
            <button v-for="option in LOCALES" :key="option.value" :class="{ active: locale === option.value }" @click="locale = option.value">
              {{ option.label }}
            </button>
          </div>

          <dl v-else class="shortcuts">
            <template v-for="shortcut in SHORTCUTS" :key="shortcut.label">
              <dt><kbd v-for="key in shortcut.keys" :key="key">{{ key }}</kbd></dt>
              <dd>{{ t(shortcut.label) }}</dd>
            </template>
          </dl>
        </div>
      </section>
    </aside>
  </div>
</template>
