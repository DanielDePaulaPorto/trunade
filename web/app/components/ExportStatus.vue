<script setup lang="ts">
const { state, closeExport } = useExport()
const { t } = useI18n()
</script>

<template>
  <aside v-if="state" class="run-all export-status" role="status">
    <span v-if="state.phase === 'running'" class="shiny">{{ t('exportRunning') }}</span>
    <span v-else-if="state.phase === 'writing'" class="shiny">{{ t('exportWriting') }}</span>
    <template v-else>
      <span v-if="state.phase === 'done'" class="run-all-ok"><AppIcon name="check" />{{ t('exportDone', { path: state.path }) }}</span>
      <span v-else class="run-all-failed"><AppIcon name="x" />{{ state.error }}</span>
      <button class="tool" :title="t('close')" @click="closeExport"><AppIcon name="close" /></button>
    </template>
  </aside>
</template>
