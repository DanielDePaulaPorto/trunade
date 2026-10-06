<script setup lang="ts">
import type { Cell } from '~/composables/types'

defineProps<{ cell: Cell }>()
</script>

<template>
  <MarkdownCell v-if="cell.kind === 'markdown'" :html="cell.html" />
  <CodeCell v-else-if="cell.kind === 'code'" :cell="cell" />
  <QuizCell v-else-if="cell.kind === 'quiz'" :cell="cell" />
  <StepsCell v-else :cell="cell" />
</template>
