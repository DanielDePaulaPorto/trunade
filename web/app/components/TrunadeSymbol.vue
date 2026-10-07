<script setup lang="ts">
/*
 * The Trunade! symbol (design/identidade/simbolo-animado): a terminal skull
 * whose eyes are the `>_` prompt, inside a rotor of three arcs. Idle, the `_`
 * blinks; running, the rotor turns and the eyes glow; when the run ends the
 * rotor takes one turn backwards (the reset). Below 24px use the favicon.
 */
const props = defineProps<{ running?: boolean; size?: number | string }>()

const gradientId = `trn-g-${useId()}`
const resetting = ref(false)

watch(
  () => props.running,
  (running, wasRunning) => {
    resetting.value = !running && !!wasRunning && !prefersLessMotion()
  },
)
</script>

<template>
  <svg
    class="trn"
    :class="{ 'is-running': running, 'is-reset': resetting && !running }"
    :width="size ?? 32"
    :height="size ?? 32"
    viewBox="0 0 64 64"
    aria-hidden="true"
  >
    <linearGradient :id="gradientId" gradientUnits="userSpaceOnUse" x1="9.05" y1="18.75" x2="44.03" y2="8.39">
      <stop offset="0" stop-opacity=".16" />
      <stop offset=".55" stop-opacity=".6" />
      <stop offset="1" />
    </linearGradient>
    <g class="trn-rotor" :stroke="`url(#${gradientId})`" @animationend="resetting = false">
      <path d="M9.05 18.75A26.5 26.5 0 0 1 44.03 8.39" />
      <path d="M9.05 18.75A26.5 26.5 0 0 1 44.03 8.39" transform="rotate(120 32 32)" />
      <path d="M9.05 18.75A26.5 26.5 0 0 1 44.03 8.39" transform="rotate(240 32 32)" />
    </g>
    <path
      class="trn-skull"
      d="M23.5 36.95A8.5 8.5 0 0 1 18 29V25a8.5 8.5 0 0 1 8.5 -8.5H37.5a8.5 8.5 0 0 1 8.5 8.5V29A8.5 8.5 0 0 1 40.5 36.95V46.5H35.78V41.9H33.88V46.5H30.12V41.9H28.22V46.5H23.5Z"
    />
    <path class="trn-eye" d="M22.2 22.5h3.4L30.2 27.6L25.6 32.7h-3.4L26.8 27.6Z" />
    <path class="trn-eye trn-cursor" d="M33.6 30.1h8.4v2.6h-8.4Z" />
  </svg>
</template>
