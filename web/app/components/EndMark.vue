<script setup lang="ts">
/*
 * End of the lesson: the vortex alone (brand book: favicon), static.
 * The brand never animates this drawing.
 */
const { t } = useI18n()
const gradientId = `trn-end-${useId()}`
</script>

<template>
  <footer class="end-mark" role="separator" :aria-label="t('endOfLesson')">
    <svg viewBox="0 0 64 64" width="40" height="40" aria-hidden="true">
      <linearGradient :id="gradientId" gradientUnits="userSpaceOnUse" x1="9.93" y1="20.26" x2="36.34" y2="7.38">
        <stop offset="0" stop-opacity="0.42" />
        <stop offset=".55" stop-opacity=".6" />
        <stop offset="1" />
      </linearGradient>
      <g fill="none" :stroke="`url(#${gradientId})`" stroke-width="12" stroke-linecap="round">
        <path d="M9.93 20.26A25 25 0 0 1 36.34 7.38" />
        <path d="M9.93 20.26A25 25 0 0 1 36.34 7.38" transform="rotate(120 32 32)" />
        <path d="M9.93 20.26A25 25 0 0 1 36.34 7.38" transform="rotate(240 32 32)" />
      </g>
    </svg>
  </footer>
</template>
