<script setup lang="ts">
defineProps<{ html: string }>()
</script>

<template>
  <!-- Raw HTML is intended: the author's own .md may embed divs, images and iframes. -->
  <div class="markdown" v-html="html" />
</template>
