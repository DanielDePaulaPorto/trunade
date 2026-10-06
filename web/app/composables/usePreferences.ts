export type ThemeMode = 'auto' | 'light' | 'dark'
export type Theme = 'light' | 'dark'
export type Locale = 'pt' | 'en'
/** Code colors: follow the page theme, or force one Monokai variant. */
export type CodeThemeMode = 'auto' | 'light' | 'dark'
/** Text column: narrow (comfortable reading) or the whole window. */
export type LayoutWidth = 'narrow' | 'wide'

const STORAGE_KEY = 'trunade-preferences'
const BASE_FONT_PX = 18
const FONT_SCALE = { min: 0.7, max: 2.5, step: 0.1 }

interface Stored {
  themeMode: ThemeMode
  codeThemeMode: CodeThemeMode
  locale: Locale
  fontScale: number
  layoutWidth: LayoutWidth
}

const stored = readStored()
const themeMode = ref<ThemeMode>(stored.themeMode)
const codeThemeMode = ref<CodeThemeMode>(stored.codeThemeMode)
const locale = ref<Locale>(stored.locale)
const fontScale = ref(stored.fontScale)
const layoutWidth = ref<LayoutWidth>(stored.layoutWidth)
const systemPrefersDark = ref(false)
const theme = computed<Theme>(() =>
  themeMode.value === 'auto' ? (systemPrefersDark.value ? 'dark' : 'light') : themeMode.value,
)
const codeTheme = computed<Theme>(() =>
  codeThemeMode.value === 'auto' ? theme.value : codeThemeMode.value,
)
let initialized = false

/**
 * Per-browser preferences. Font scale sets the root font size, so everything
 * sized in rem grows with it; browser zoom still multiplies on top.
 */
export function usePreferences() {
  if (!initialized && import.meta.client) {
    initialized = true
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = query.matches
    query.addEventListener('change', (event) => (systemPrefersDark.value = event.matches))
    watchEffect(() => {
      const root = document.documentElement
      root.dataset.theme = theme.value
      root.dataset.codeTheme = codeTheme.value
      root.dataset.layout = layoutWidth.value
      root.lang = locale.value === 'pt' ? 'pt-BR' : 'en'
      root.style.fontSize = `${BASE_FONT_PX * fontScale.value}px`
    })
    watch([themeMode, codeThemeMode, locale, fontScale, layoutWidth], store)
  }
  return {
    themeMode,
    theme,
    codeThemeMode,
    codeTheme,
    locale,
    fontScale,
    layoutWidth,
    toggleLayoutWidth,
    setFontScale,
    growFont: () => setFontScale(fontScale.value + FONT_SCALE.step),
    shrinkFont: () => setFontScale(fontScale.value - FONT_SCALE.step),
    resetFont: () => setFontScale(1),
    cycleTheme,
  }
}

function setFontScale(value: number) {
  const clamped = Math.min(FONT_SCALE.max, Math.max(FONT_SCALE.min, value))
  fontScale.value = Math.round(clamped * 10) / 10
}

function toggleLayoutWidth() {
  layoutWidth.value = layoutWidth.value === 'narrow' ? 'wide' : 'narrow'
}

function cycleTheme() {
  const order: ThemeMode[] = ['auto', 'light', 'dark']
  themeMode.value = order[(order.indexOf(themeMode.value) + 1) % order.length]!
}

function defaults(): Stored {
  const browserLocale = import.meta.client ? navigator.language : 'pt'
  return { themeMode: 'auto', codeThemeMode: 'auto', locale: browserLocale.startsWith('pt') ? 'pt' : 'en', fontScale: 1, layoutWidth: 'narrow' }
}

// Storage can be unavailable (private windows, blocked site data): use defaults.
function readStored(): Stored {
  try {
    return { ...defaults(), ...JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') }
  } catch {
    return defaults()
  }
}

function store() {
  try {
    const value: Stored = {
      themeMode: themeMode.value,
      codeThemeMode: codeThemeMode.value,
      locale: locale.value,
      fontScale: fontScale.value,
      layoutWidth: layoutWidth.value,
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
  } catch {
    // keep working without persistence
  }
}
