<script setup lang="ts">
import type { StepsCell } from '~/composables/types'

/* Exercise in steps (idea from vue-bits Stepper). Every step stays mounted so
   code edits and outputs survive moving back and forth. */
const props = defineProps<{ cell: StepsCell }>()
const { t } = useI18n()

const current = ref(0)
const { printing } = usePrint()
const direction = ref<1 | -1>(1)
const last = computed(() => props.cell.steps.length - 1)

function go(index: number) {
  if (index < 0 || index > last.value || index === current.value) return
  direction.value = index > current.value ? 1 : -1
  current.value = index
}
</script>

<template>
  <section class="steps">
    <ol class="steps-track">
      <li v-for="(step, index) in cell.steps" :key="index" :class="{ done: index < current, active: index === current }">
        <button class="steps-dot" :aria-current="index === current ? 'step' : undefined" @click="go(index)">
          <AppIcon v-if="index < current" name="check" />
          <span v-else>{{ index + 1 }}</span>
        </button>
        <span class="steps-title">{{ step.title }}</span>
      </li>
    </ol>
    <div class="steps-body" :class="direction === 1 ? 'forward' : 'backward'">
      <div v-for="(step, index) in cell.steps" v-show="printing || index === current" :key="index" class="steps-panel">
        <p v-if="printing" class="steps-print-title">{{ index + 1 }}. {{ step.title }}</p>
        <CellView v-for="child in step.cells" :key="child.id" :cell="child" />
      </div>
    </div>
    <footer class="steps-footer">
      <button class="steps-button secondary" :disabled="current === 0" @click="go(current - 1)">
        {{ t('previousStep') }}
      </button>
      <span class="steps-progress">{{ t('stepProgress', { current: current + 1, total: cell.steps.length }) }}</span>
      <button class="steps-button" :disabled="current === last" @click="go(current + 1)">
        {{ t('nextStep') }}
      </button>
    </footer>
  </section>
</template>
