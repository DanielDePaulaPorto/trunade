<script setup lang="ts">
import type { QuizCell } from '~/composables/types'

const props = defineProps<{ cell: QuizCell }>()
const { t } = useI18n()

const multiple = computed(() => props.cell.options.filter((o) => o.correct).length > 1)
const chosen = ref<boolean[]>(props.cell.options.map(() => false))
const revealed = ref(false)
const { printing } = usePrint()
/** The PDF is an answer key: printing shows the answers. */
const showAnswers = computed(() => revealed.value || printing.value)
const root = ref<HTMLElement>()

onMounted(() => root.value && renderMath(root.value))

function choose(index: number) {
  if (revealed.value) return
  if (!multiple.value) chosen.value = chosen.value.map(() => false)
  chosen.value[index] = !chosen.value[index]
}

const revealButton = ref<HTMLButtonElement>()

function toggleReveal() {
  if (revealed.value) chosen.value = chosen.value.map(() => false)
  else {
    logAnswer()
    if (answeredRight() && revealButton.value) confetti(revealButton.value)
  }
  revealed.value = !revealed.value
}

/** Exactly the right options chosen. */
function answeredRight() {
  return props.cell.options.every((option, index) => option.correct === chosen.value[index])
}

function logAnswer() {
  const chosenIndexes = chosen.value.flatMap((isChosen, index) => (isChosen ? [index] : []))
  const correct = answeredRight()
  logEvent('quiz', { cell: props.cell.id, question: logText(props.cell.question), chosen: chosenIndexes, correct })
}

/** Which mark the box shows: the student's choice, then the answer key once revealed. */
function mark(index: number): 'check' | 'x' | null {
  const correct = props.cell.options[index]!.correct
  if (showAnswers.value) return correct ? 'check' : chosen.value[index] ? 'x' : null
  return chosen.value[index] && multiple.value ? 'check' : null
}

function optionClass(index: number) {
  if (!showAnswers.value) return { chosen: chosen.value[index] }
  const correct = props.cell.options[index]!.correct
  return { correct, wrong: chosen.value[index] && !correct, chosen: chosen.value[index] }
}
</script>

<template>
  <section ref="root" class="quiz" :class="{ revealed: showAnswers }">
    <div class="markdown quiz-question" v-html="cell.question" />
    <ul class="quiz-options" :role="multiple ? 'group' : 'radiogroup'">
      <li v-for="(option, index) in cell.options" :key="index">
        <button
          class="quiz-option"
          :class="optionClass(index)"
          :role="multiple ? 'checkbox' : 'radio'"
          :aria-checked="showAnswers ? cell.options[index]!.correct : chosen[index]"
          @click="choose(index)"
        >
          <span class="quiz-mark" :class="{ square: multiple }">
            <AppIcon v-if="mark(index)" :name="mark(index)!" />
          </span>
          <span v-html="option.html" />
        </button>
      </li>
    </ul>
    <button ref="revealButton" class="quiz-reveal" @click="toggleReveal">
      {{ revealed ? t('tryAgain') : t('showAnswer') }}
    </button>
  </section>
</template>
