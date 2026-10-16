<script setup lang="ts">
import { EditorView, basicSetup } from 'codemirror'
import { keymap, lineNumbers } from '@codemirror/view'
import { Compartment, EditorState, Prec, type Extension } from '@codemirror/state'
import { cpp } from '@codemirror/lang-cpp'
import { python } from '@codemirror/lang-python'
import { MONOKAI } from '~/composables/monokai'

/** `firstLine` numbers the gutter as in the whole program when a template hides lines above. */
const props = withDefaults(defineProps<{ language: string; firstLine?: number; readonly?: boolean }>(), {
  firstLine: 1,
  readonly: false,
})
const model = defineModel<string>({ required: true })
const emit = defineEmits<{ run: [] }>()

const container = ref<HTMLElement>()
const { codeTheme } = usePreferences()
const colors = new Compartment()
const numbering = new Compartment()
let view: EditorView | null = null

const LANGUAGES: Record<string, () => Extension> = {
  cpp, 'c++': cpp, cxx: cpp, cc: cpp, c: cpp,
  python, py: python, python3: python,
}

onMounted(() => {
  view = new EditorView({
    parent: container.value!,
    doc: model.value,
    extensions: [
      basicSetup,
      colors.of(MONOKAI[codeTheme.value]),
      numbering.of(lineNumbersFrom(props.firstLine)),
      props.readonly ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : [],
      LANGUAGES[props.language]?.() ?? [],
      Prec.highest(
        keymap.of([
          { key: 'Shift-Enter', run: () => (emit('run'), true) },
          // Escape leaves the editor so page shortcuts (t, space…) work again.
          { key: 'Escape', run: (target) => (target.contentDOM.blur(), true) },
        ]),
      ),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) model.value = update.state.doc.toString()
      }),
    ],
  })
})

// External changes (undo edit) replace the document.
watch(model, (value) => {
  if (view && value !== view.state.doc.toString()) {
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } })
  }
})

watch(codeTheme, (value) => view?.dispatch({ effects: colors.reconfigure(MONOKAI[value]) }))
watch(() => props.firstLine, (value) => view?.dispatch({ effects: numbering.reconfigure(lineNumbersFrom(value)) }))

function lineNumbersFrom(first: number) {
  return lineNumbers({ formatNumber: (line) => String(line + first - 1) })
}

onBeforeUnmount(() => view?.destroy())
</script>

<template>
  <div ref="container" class="code-editor" />
</template>
