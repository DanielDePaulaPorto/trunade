<script setup lang="ts">
const { snapshot, connected, lastError } = useConnection()

watch(
  () => snapshot.value?.fileName,
  (name) => useHead({ title: name ? `${name} · Trunade!` : 'Trunade!' }),
)
</script>

<template>
  <div class="page">
    <header class="topbar">
      <strong class="brand">Trunade!</strong>
      <span class="file">{{ snapshot?.fileName }}</span>
      <span class="spacer" />
      <span class="connection" :class="{ online: connected }">
        {{ connected ? 'conectado' : 'reconectando…' }}
      </span>
    </header>
    <main class="document">
      <p v-if="snapshot?.error" class="banner">{{ snapshot.error }}</p>
      <p v-if="lastError" class="banner" @click="lastError = null">{{ lastError }}</p>
      <template v-for="cell in snapshot?.cells ?? []" :key="cell.id">
        <MarkdownCell v-if="cell.kind === 'markdown'" :html="cell.html" />
        <CodeCell v-else :cell="cell" />
      </template>
    </main>
  </div>
</template>
