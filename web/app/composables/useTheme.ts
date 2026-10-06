export type ThemeMode = 'auto' | 'light' | 'dark'
export type Theme = 'light' | 'dark'

const STORAGE_KEY = 'trunade-theme'
const MODES: ThemeMode[] = ['auto', 'light', 'dark']

const mode = ref<ThemeMode>(readStoredMode())
const systemPrefersDark = ref(false)
const theme = computed<Theme>(() =>
  mode.value === 'auto' ? (systemPrefersDark.value ? 'dark' : 'light') : mode.value,
)
let initialized = false

/** Light/dark choice shared by the page, the code editors and the terminals. */
export function useTheme() {
  if (!initialized && import.meta.client) {
    initialized = true
    const query = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = query.matches
    query.addEventListener('change', (event) => (systemPrefersDark.value = event.matches))
    watchEffect(() => (document.documentElement.dataset.theme = theme.value))
    watch(mode, storeMode)
  }
  return { mode, theme, cycleMode }
}

function cycleMode() {
  mode.value = MODES[(MODES.indexOf(mode.value) + 1) % MODES.length]!
}

// Storage can be unavailable (private windows, blocked site data): fall back to auto.
function readStoredMode(): ThemeMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY) as ThemeMode | null
    return stored && MODES.includes(stored) ? stored : 'auto'
  } catch {
    return 'auto'
  }
}

function storeMode(value: ThemeMode) {
  try {
    localStorage.setItem(STORAGE_KEY, value)
  } catch {
    // keep working without persistence
  }
}
