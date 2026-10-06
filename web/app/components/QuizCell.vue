<script setup lang="ts">
import type { QuizCell } from '~/composables/types'

const props = defineProps<{ cell: QuizCell }>()
const { t } = useI18n()

const multiple = computed(() => props.cell.options.filter((o) => o.correct).length > 1)
const chosen = ref<boolean[]>(props.cell.options.map(() => false))
const revealed = ref(false)
const root = ref<HTMLElement>()

onMounted(() => root.value && renderMath(root.value))

function choose(index: number) {
  if (revealed.value) return
  if (!multiple.value) chosen.value = chosen.value.map(() => false)
  chosen.value[index] = !chosen.value[index]
}

function toggleReveal() {
  if (revealed.value) chosen.value = chosen.value.map(() => false)
  revealed.value = !revealed.value
}

function optionClass(index: number) {
  if (!revealed.value) return { chosen: chosen.value[index] }
  const correct = props.cell.options[index]!.correct
  return { correct, wrong: chosen.value[index] && !correct, chosen: chosen.value[index] }
}
</script>

<template>
  <section ref="root" class="quiz" :class="{ revealed }">
    <div class="markdown quiz-question" v-html="cell.question" />
    <ul class="quiz-options" :role="multiple ? 'group' : 'radiogroup'">
      <li v-for="(option, index) in cell.options" :key="index">
        <button
          class="quiz-option"
          :class="optionClass(index)"
          :role="multiple ? 'checkbox' : 'radio'"
          :aria-checked="chosen[index]"
          @click="choose(index)"
        >
          <span class="quiz-mark" :class="{ square: multiple }">
            <AppIcon v-if="revealed && cell.options[index]!.correct" name="check" />
          </span>
          <span v-html="option.html" />
        </button>
      </li>
    </ul>
    <button class="quiz-reveal" @click="toggleReveal">
      {{ revealed ? t('tryAgain') : t('showAnswer') }}
    </button>
  </section>
</template>
