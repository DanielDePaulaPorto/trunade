<script setup lang="ts">
import { revealBlock } from '~/composables/useRunAll'

const { report, closeReport } = useRunAll()
const { t } = useI18n()
</script>

<template>
  <aside v-if="report" class="run-all" role="status">
    <template v-if="!report.finished">
      <span class="shiny">{{ t('runAllProgress', { done: report.done, total: report.total }) }}</span>
    </template>
    <template v-else>
      <span v-if="report.total === 0">{{ t('runAllNone') }}</span>
      <span v-else class="run-all-summary">
        <span class="run-all-ok"><AppIcon name="check" />{{ report.total - report.failed.length }}</span>
        <span v-if="report.failed.length" class="run-all-failed"><AppIcon name="x" />{{ report.failed.length }}</span>
      </span>
      <button
        v-for="block in report.failed"
        :key="block.id"
        class="run-all-link"
        :title="t('runAllShow')"
        @click="revealBlock(block.id)"
      >
        {{ t('lineNumber', { line: block.line }) }}
      </button>
      <button class="tool" :title="t('close')" @click="closeReport"><AppIcon name="close" /></button>
    </template>
  </aside>
</template>
