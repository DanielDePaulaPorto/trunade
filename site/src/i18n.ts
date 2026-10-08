// Portuguese lives at the root, English under /en/. Pages are paired by path.
export type Lang = 'pt' | 'en'

export const LANGS: Lang[] = ['pt', 'en']

export function langFromPath(path: string): Lang {
  return path === '/en' || path.startsWith('/en/') ? 'en' : 'pt'
}

/** Prefixes a root-relative path with the language. */
export function localize(path: string, lang: Lang): string {
  return lang === 'en' ? `/en${path}` : path
}

/** The same page in the other language. */
export function counterpart(path: string): string {
  return langFromPath(path) === 'en' ? path.replace(/^\/en(?=\/|$)/, '') || '/' : `/en${path}`
}

export const CHROME = {
  pt: {
    docs: 'Documentação',
    install: 'Instalação',
    theme: 'tema',
    themeLight: 'claro',
    themeDark: 'escuro',
    themeLabel: 'Alternar tema claro e escuro',
    otherLang: 'English',
    otherLangLabel: 'Read in English',
    skip: 'Pular para o conteúdo',
    footer: 'Funciona sem internet, como a sua sala.',
    description:
      'Aulas de programação em Markdown, com blocos de código que compilam de verdade e rodam no navegador. Funciona sem internet.',
    homeTitle: 'Trunade! — aulas de programação que rodam',
  },
  en: {
    docs: 'Docs',
    install: 'Install',
    theme: 'theme',
    themeLight: 'light',
    themeDark: 'dark',
    themeLabel: 'Switch between light and dark theme',
    otherLang: 'Português',
    otherLangLabel: 'Ler em português',
    skip: 'Skip to content',
    footer: 'Works offline, like your classroom.',
    description:
      'Programming lessons in Markdown, with code blocks that really compile and run in the browser. Works offline.',
    homeTitle: 'Trunade! — programming lessons that run',
  },
} as const
