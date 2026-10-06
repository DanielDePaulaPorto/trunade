<script setup lang="ts">
/* Counts from zero to `value` (idea from vue-bits CountUp). */
const props = defineProps<{ value: number; format: (value: number) => string }>()

const DURATION_MS = 600
const shown = ref(0)

function animate(target: number) {
  if (prefersLessMotion()) {
    shown.value = target
    return
  }
  const start = performance.now()
  const tick = (now: number) => {
    const progress = Math.min(1, (now - start) / DURATION_MS)
    shown.value = target * (1 - (1 - progress) ** 3) // ease-out cubic
    if (progress < 1) requestAnimationFrame(tick)
  }
  requestAnimationFrame(tick)
}

onMounted(() => animate(props.value))
watch(() => props.value, animate)
</script>

<template>
  <span>{{ format(shown) }}</span>
</template>
