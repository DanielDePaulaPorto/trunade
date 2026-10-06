<script setup lang="ts">
import { ICONS, type IconName } from '~/composables/icons'

defineProps<{ name: IconName; size?: number | string }>()
</script>

<template>
  <svg
    class="icon"
    :class="`icon-${name}`"
    :width="size ?? '1em'"
    :height="size ?? '1em'"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    v-html="ICONS[name]"
  />
</template>
