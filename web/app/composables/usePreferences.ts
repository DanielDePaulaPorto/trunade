import { BODY_FONTS, CODE_FONTS, DEFAULT_FONTS, TITLE_FONTS, type BodyFont, type CodeFont, type TitleFont } from './fonts'

export type ThemeMode = 'auto' | 'light' | 'dark'
export type Theme = 'light' | 'dark'
export type Locale = 'pt' | 'en'
/** Code colors: follow the page theme, or force one Monokai variant. */
export type CodeThemeMode = 'auto' | 'light' | 'dark'
/** Text column: narrow (comfortable reading) or the whole window. */
export type LayoutWidth = 'narrow' | 'wide'

/** The preferences file (~/.config/trunade/preferences.toml), as the server sends it. */
export interface PreferencesFile {
  theme?: string | null
  code_theme?: string | null
  locale?: string | null
  font_scale?: number | null
  text_width?: string | null
  title_font?: string | null
  body_font?: string | null
  code_font?: string | null
}

const STORAGE_KEY = 'trunade-preferences'
const BASE_FONT_PX = 18
const FONT_SCALE = { min: 0.7, max: 2.5, step: 0.1 }

interface Stored {
  themeMode: ThemeMode
  codeThemeMode: CodeThemeMode
  locale: Locale
  fontScale: number
  layoutWidth: LayoutWidth
  titleFont: TitleFont
  bodyFont: BodyFont
  codeFont: CodeFont
}

const stored = { ...defaults(), ...readCache() }
const themeMode = ref<ThemeMode>(stored.themeMode)
const codeThemeMode = ref<CodeThemeMode>(stored.codeThemeMode)
const locale = ref<Locale>(stored.locale)
const fontScale = ref(stored.fontScale)
const layoutWidth = ref<LayoutWidth>(stored.layoutWidth)
const titleFont = ref<TitleFont>(stored.titleFont)
const bodyFont = ref<BodyFont>(stored.bodyFont)
const codeFont = ref<CodeFont>(stored.codeFont)
const systemPrefersDark = ref(false)
const theme = computed<Theme>(() =>
  themeMode.value === 'auto' ? (systemPrefersDark.value ? 'dark' : 'light') : themeMode.value,
)
const codeTheme = computed<Theme>(() =>
  codeThemeMode.value === 'auto' ? theme.value : codeThemeMode.value,
)
const codeFontFamily = computed(() => CODE_FONTS[codeFont.value].family)
let initialized = false
let lastSaved = ''

/**
 * Preferences live in ~/.config/trunade/preferences.toml, so every lesson
 * (on any port) opens the same way. localStorage only caches them to avoid a
 * flash before the WebSocket delivers the file. Font scale sets the root font
 * size, so everything sized in rem grows with it.
 */
export function usePreferences() {
  if (!initialized && import.meta.client) {
    initialized = true
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = query.matches
    query.addEventListener('change', (event) => (systemPrefersDark.value = event.matches))
    watchEffect(applyToPage)
    syncWithServer()
  }
  return {
    themeMode,
    theme,
    codeThemeMode,
    codeTheme,
    locale,
    fontScale,
    layoutWidth,
    titleFont,
    bodyFont,
    codeFont,
    codeFontFamily,
    toggleLayoutWidth,
    setFontScale,
    growFont: () => setFontScale(fontScale.value + FONT_SCALE.step),
    shrinkFont: () => setFontScale(fontScale.value - FONT_SCALE.step),
    resetFont: () => setFontScale(1),
    cycleTheme,
  }
}

function applyToPage() {
  const root = document.documentElement
  const title = TITLE_FONTS[titleFont.value]
  root.dataset.theme = theme.value
  root.dataset.codeTheme = codeTheme.value
  root.dataset.layout = layoutWidth.value
  root.lang = locale.value === 'pt' ? 'pt-BR' : 'en'
  root.style.fontSize = `${BASE_FONT_PX * fontScale.value}px`
  root.style.setProperty('--font-title', title.family)
  root.style.setProperty('--title-weight', String(title.weight))
  root.style.setProperty('--font-text', BODY_FONTS[bodyFont.value].family)
  root.style.setProperty('--font-mono', codeFontFamily.value)
}

/** The server sends the file on connect and after every change; the page saves on every change. */
function syncWithServer() {
  const { serverPreferences, send } = useConnection()
  const save = (value: Stored) => {
    const file = toFile(value)
    const json = JSON.stringify(file)
    if (json === lastSaved || !serverPreferences.value) return
    lastSaved = json
    send({ type: 'savePreferences', preferences: file })
  }
  watch(serverPreferences, (file) => {
    if (!file) return
    const known = fromFile(file)
    // No file yet: create it from what this browser had.
    if (Object.keys(known).length === 0) return save(current())
    const merged = { ...current(), ...known }
    lastSaved = JSON.stringify(toFile(merged))
    apply(merged)
  })
  watch(current, (value) => {
    writeCache(value)
    save(value)
  })
}

function current(): Stored {
  return {
    themeMode: themeMode.value,
    codeThemeMode: codeThemeMode.value,
    locale: locale.value,
    fontScale: fontScale.value,
    layoutWidth: layoutWidth.value,
    titleFont: titleFont.value,
    bodyFont: bodyFont.value,
    codeFont: codeFont.value,
  }
}

function apply(value: Stored) {
  themeMode.value = value.themeMode
  codeThemeMode.value = value.codeThemeMode
  locale.value = value.locale
  setFontScale(value.fontScale)
  layoutWidth.value = value.layoutWidth
  titleFont.value = value.titleFont
  bodyFont.value = value.bodyFont
  codeFont.value = value.codeFont
}

function toFile(value: Stored): PreferencesFile {
  return {
    theme: value.themeMode,
    code_theme: value.codeThemeMode,
    locale: value.locale,
    font_scale: value.fontScale,
    text_width: value.layoutWidth,
    title_font: value.titleFont,
    body_font: value.bodyFont,
    code_font: value.codeFont,
  }
}

/** Only known values: the file is hand-editable and may hold typos. */
function fromFile(file: PreferencesFile): Partial<Stored> {
  const pick = <T extends string>(value: unknown, allowed: readonly T[]) =>
    allowed.includes(value as T) ? (value as T) : undefined
  const result: Partial<Stored> = {
    themeMode: pick(file.theme, ['auto', 'light', 'dark']),
    codeThemeMode: pick(file.code_theme, ['auto', 'light', 'dark']),
    locale: pick(file.locale, ['pt', 'en']),
    fontScale: typeof file.font_scale === 'number' ? file.font_scale : undefined,
    layoutWidth: pick(file.text_width, ['narrow', 'wide']),
    titleFont: pick(file.title_font, Object.keys(TITLE_FONTS) as TitleFont[]),
    bodyFont: pick(file.body_font, Object.keys(BODY_FONTS) as BodyFont[]),
    codeFont: pick(file.code_font, Object.keys(CODE_FONTS) as CodeFont[]),
  }
  return Object.fromEntries(Object.entries(result).filter(([, value]) => value !== undefined))
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
  return {
    themeMode: 'auto',
    codeThemeMode: 'dark',
    locale: browserLocale.startsWith('pt') ? 'pt' : 'en',
    fontScale: 1,
    layoutWidth: 'narrow',
    titleFont: DEFAULT_FONTS.title,
    bodyFont: DEFAULT_FONTS.body,
    codeFont: DEFAULT_FONTS.code,
  }
}

// Storage can be unavailable (private windows, blocked site data): it is only a cache.
function readCache(): Partial<Stored> {
  try {
    const cached = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as Partial<Stored>
    return fromFile(toFile({ ...defaults(), ...cached }))
  } catch {
    return {}
  }
}

function writeCache(value: Stored) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value))
  } catch {
    // keep working without the cache
  }
}
