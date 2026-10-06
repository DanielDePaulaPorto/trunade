<script setup lang="ts">
import { EditorView, basicSetup } from 'codemirror'
import { keymap } from '@codemirror/view'
import { Compartment, Prec, type Extension } from '@codemirror/state'
import { cpp } from '@codemirror/lang-cpp'
import { python } from '@codemirror/lang-python'
import { oneDark } from '@codemirror/theme-one-dark'

const props = defineProps<{ language: string }>()
const model = defineModel<string>({ required: true })
const emit = defineEmits<{ run: [] }>()

const container = ref<HTMLElement>()
const { theme } = useTheme()
const themeCompartment = new Compartment()
let view: EditorView | null = null

// Light uses CodeMirror's default highlighting; the page CSS supplies the colors around it.
const editorTheme = (value: string): Extension => (value === 'dark' ? oneDark : [])

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
      themeCompartment.of(editorTheme(theme.value)),
      LANGUAGES[props.language]?.() ?? [],
      Prec.highest(
        keymap.of([{ key: 'Shift-Enter', run: () => (emit('run'), true) }]),
      ),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) model.value = update.state.doc.toString()
      }),
    ],
  })
})

// External changes (Reset) replace the document.
watch(model, (value) => {
  if (view && value !== view.state.doc.toString()) {
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } })
  }
})

watch(theme, (value) => {
  view?.dispatch({ effects: themeCompartment.reconfigure(editorTheme(value)) })
})

onBeforeUnmount(() => view?.destroy())
</script>

<template>
  <div ref="container" class="code-editor" />
</template>
