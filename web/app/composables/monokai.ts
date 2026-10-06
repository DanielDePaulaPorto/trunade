import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'

/** Classic Monokai, used for code in both page themes. */
const COLORS = {
  background: '#272822',
  foreground: '#f8f8f2',
  comment: '#75715e',
  pink: '#f92672',
  yellow: '#e6db74',
  purple: '#ae81ff',
  green: '#a6e22e',
  blue: '#66d9ef',
  orange: '#fd971f',
  selection: '#49483e',
  activeLine: '#3e3d32',
  gutter: '#90908a',
}

const theme = EditorView.theme(
  {
    '&': { backgroundColor: COLORS.background, color: COLORS.foreground },
    '.cm-content': { caretColor: COLORS.foreground },
    '.cm-cursor, .cm-dropCursor': { borderLeftColor: COLORS.foreground },
    '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, ::selection':
      { backgroundColor: COLORS.selection },
    '.cm-gutters': { backgroundColor: COLORS.background, color: COLORS.gutter, border: 'none' },
    '.cm-activeLine': { backgroundColor: COLORS.activeLine },
    '.cm-activeLineGutter': { backgroundColor: COLORS.activeLine, color: COLORS.foreground },
    '.cm-matchingBracket': { backgroundColor: COLORS.selection, outline: `1px solid ${COLORS.gutter}` },
    '.cm-foldPlaceholder': { backgroundColor: 'transparent', border: 'none', color: COLORS.comment },
  },
  { dark: true },
)

const highlight = HighlightStyle.define([
  { tag: t.comment, color: COLORS.comment, fontStyle: 'italic' },
  { tag: [t.keyword, t.operator, t.processingInstruction, t.modifier, t.controlKeyword], color: COLORS.pink },
  { tag: [t.string, t.special(t.string), t.character], color: COLORS.yellow },
  { tag: [t.number, t.bool, t.null, t.atom], color: COLORS.purple },
  { tag: [t.function(t.variableName), t.function(t.propertyName), t.definition(t.function(t.variableName))], color: COLORS.green },
  { tag: [t.typeName, t.className, t.standard(t.typeName), t.namespace], color: COLORS.blue, fontStyle: 'italic' },
  { tag: [t.macroName, t.definition(t.typeName)], color: COLORS.green },
  { tag: [t.propertyName, t.attributeName], color: COLORS.foreground },
  { tag: t.escape, color: COLORS.purple },
  { tag: t.special(t.variableName), color: COLORS.orange },
  { tag: t.invalid, color: COLORS.foreground, backgroundColor: COLORS.pink },
])

export const monokai = [theme, syntaxHighlighting(highlight)]
