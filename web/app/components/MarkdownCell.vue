<script setup lang="ts">
const props = defineProps<{ html: string }>()
const { theme } = usePreferences()
const root = ref<HTMLElement>()

async function enhance() {
  await nextTick()
  if (!root.value) return
  renderMath(root.value)
  fitIframes(root.value)
  await renderMermaid(root.value, theme.value)
}

onMounted(enhance)
watch(() => props.html, enhance)
watch(theme, () => root.value && renderMermaid(root.value, theme.value))
</script>

<template>
  <!-- Raw HTML is intended: the author's own .md may embed divs, images and iframes. -->
  <div ref="root" class="markdown" v-html="html" />
</template>
