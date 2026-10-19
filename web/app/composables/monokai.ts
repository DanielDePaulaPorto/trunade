import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t, type Tag } from '@lezer/highlight'
import type { Extension } from '@codemirror/state'
import type { Theme } from './usePreferences'

export interface Palette {
  background: string
  foreground: string
  comment: string
  keyword: string
  string: string
  number: string
  function: string
  type: string
  variable: string
  selection: string
  activeLine: string
  gutter: string
}

/** Classic Monokai. */
export const DARK: Palette = {
  background: '#272822',
  foreground: '#f8f8f2',
  comment: '#75715e',
  keyword: '#f92672',
  string: '#e6db74',
  number: '#ae81ff',
  function: '#a6e22e',
  type: '#66d9ef',
  variable: '#fd971f',
  selection: '#49483e',
  activeLine: '#3e3d32',
  gutter: '#90908a',
}

/** Same hue families, darkened to read on a light background. */
const LIGHT: Palette = {
  background: '#fdfaf4',
  foreground: '#2c292d',
  comment: '#8c8577',
  keyword: '#d3245c',
  string: '#9c6d00',
  number: '#6d4fc4',
  function: '#3a8a12',
  type: '#1b7f9c',
  variable: '#c25b00',
  selection: '#eadfc8',
  activeLine: '#f4eee2',
  gutter: '#a59f85',
}

function monokai(palette: Palette, dark: boolean): Extension {
  const theme = EditorView.theme(
    {
      '&': { backgroundColor: palette.background, color: palette.foreground },
      '.cm-content': { caretColor: palette.foreground },
      '.cm-cursor, .cm-dropCursor': { borderLeftColor: palette.foreground },
      '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, ::selection':
        { backgroundColor: palette.selection },
      '.cm-gutters': { backgroundColor: palette.background, color: palette.gutter, border: 'none' },
      '.cm-activeLine': { backgroundColor: palette.activeLine },
      '.cm-activeLineGutter': { backgroundColor: palette.activeLine, color: palette.foreground },
      '.cm-matchingBracket': { backgroundColor: palette.selection, outline: `1px solid ${palette.gutter}` },
      '.cm-foldPlaceholder': { backgroundColor: 'transparent', border: 'none', color: palette.comment },
    },
    { dark },
  )
  return [theme, syntaxHighlighting(HighlightStyle.define(highlightRules(palette)))]
}

export interface HighlightRule {
  tag: Tag | Tag[]
  color: string
  fontStyle?: string
  backgroundColor?: string
}

/** Token colors, shared by the editor and the Moodle copy (inline styles). */
export function highlightRules(palette: Palette): HighlightRule[] {
  return [
    { tag: t.comment, color: palette.comment, fontStyle: 'italic' },
    { tag: [t.keyword, t.operator, t.processingInstruction, t.modifier, t.controlKeyword], color: palette.keyword },
    { tag: [t.string, t.special(t.string), t.character], color: palette.string },
    { tag: [t.number, t.bool, t.null, t.atom, t.escape], color: palette.number },
    {
      tag: [t.function(t.variableName), t.function(t.propertyName), t.definition(t.function(t.variableName)), t.macroName],
      color: palette.function,
    },
    { tag: [t.typeName, t.className, t.standard(t.typeName), t.namespace], color: palette.type, fontStyle: 'italic' },
    { tag: [t.propertyName, t.attributeName], color: palette.foreground },
    { tag: t.special(t.variableName), color: palette.variable },
    { tag: t.invalid, color: palette.background, backgroundColor: palette.keyword },
  ]
}

export const MONOKAI: Record<Theme, Extension> = {
  dark: monokai(DARK, true),
  light: monokai(LIGHT, false),
}
