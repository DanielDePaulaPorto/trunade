/*
 * The fonts a lesson can be shown with. All of them are embedded (offline);
 * the CSS for each family is listed in nuxt.config.ts.
 */
export interface FontChoice {
  label: string
  family: string
  /** Display faces with a single weight must not be synthesized bold. */
  weight?: number
  /** Letter spacing for titles (Gloock: 7%, like the logo). */
  tracking?: string
}

const MONO_FALLBACK = 'ui-monospace, SFMono-Regular, Menlo, monospace'
const SANS_FALLBACK = 'ui-sans-serif, system-ui, sans-serif'
const SERIF_FALLBACK = 'ui-serif, Georgia, serif'

export const TITLE_FONTS = {
  gloock: { label: 'Gloock', family: `"Gloock", ${SERIF_FALLBACK}`, weight: 400, tracking: '0.07em' },
  'ibm-plex-mono': { label: 'IBM Plex Mono', family: `"IBM Plex Mono", ${MONO_FALLBACK}`, weight: 700 },
  fraunces: { label: 'Fraunces', family: `"Fraunces Variable", ${SERIF_FALLBACK}`, weight: 700 },
  'cormorant-garamond': { label: 'Cormorant Garamond', family: `"Cormorant Garamond Variable", ${SERIF_FALLBACK}`, weight: 700 },
  'young-serif': { label: 'Young Serif', family: `"Young Serif", ${SERIF_FALLBACK}`, weight: 400 },
} satisfies Record<string, FontChoice>

export const BODY_FONTS = {
  'jetbrains-mono': { label: 'JetBrains Mono', family: `"JetBrains Mono Variable", ${MONO_FALLBACK}` },
  'fira-code': { label: 'Fira Code', family: `"Fira Code Variable", ${MONO_FALLBACK}` },
  inter: { label: 'Inter', family: `"Inter Variable", ${SANS_FALLBACK}` },
  'open-sans': { label: 'Open Sans', family: `"Open Sans Variable", ${SANS_FALLBACK}` },
} satisfies Record<string, FontChoice>

export const CODE_FONTS = {
  'jetbrains-mono': BODY_FONTS['jetbrains-mono'],
  'fira-code': BODY_FONTS['fira-code'],
} satisfies Record<string, FontChoice>

export type TitleFont = keyof typeof TITLE_FONTS
export type BodyFont = keyof typeof BODY_FONTS
export type CodeFont = keyof typeof CODE_FONTS

export const DEFAULT_FONTS = {
  title: 'gloock' as TitleFont,
  body: 'jetbrains-mono' as BodyFont,
  code: 'jetbrains-mono' as CodeFont,
}
