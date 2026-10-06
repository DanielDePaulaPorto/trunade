<script setup lang="ts">
import type { Topic } from '~/composables/useSections'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()

const tree = topicTree()
const sectionIndex = ref(currentSectionIndex(tree))
const subIndex = ref<number | null>(null) // null: the section itself is selected
const expanded = ref<number | null>(null)
const list = ref<HTMLElement>()

const section = computed(() => tree[sectionIndex.value])

function go(topic: Topic | undefined) {
  if (topic) scrollToHeading(topic.element)
  emit('close')
}

function selected(): Topic | undefined {
  const current = section.value
  return subIndex.value === null ? current : current?.children[subIndex.value]
}

function move(step: 1 | -1) {
  const children = section.value?.children ?? []
  if (subIndex.value !== null) {
    const next = subIndex.value + step
    if (next >= 0 && next < children.length) subIndex.value = next
    else if (next < 0) subIndex.value = null
    else moveSection(1)
    return
  }
  if (step === 1 && expanded.value === sectionIndex.value && children.length) subIndex.value = 0
  else moveSection(step)
}

function moveSection(step: 1 | -1) {
  const next = sectionIndex.value + step
  if (next < 0 || next >= tree.length) return
  sectionIndex.value = next
  subIndex.value = null
  // Coming up into an open section lands on its last subsection.
  const children = tree[next]!.children
  if (step === -1 && expanded.value === next && children.length) subIndex.value = children.length - 1
}

function onKey(event: KeyboardEvent) {
  const actions: Record<string, () => void> = {
    ArrowDown: () => move(1),
    ArrowUp: () => move(-1),
    ArrowRight: () => {
      if (!section.value?.children.length) return
      expanded.value = sectionIndex.value
      subIndex.value ??= 0
    },
    ArrowLeft: () => {
      subIndex.value = null
      expanded.value = null
    },
    Enter: () => go(selected()),
    Escape: () => emit('close'),
    t: () => emit('close'),
  }
  const action = actions[event.key]
  if (!action) return
  event.preventDefault()
  event.stopPropagation()
  action()
  nextTick(() => list.value?.querySelector('.selected')?.scrollIntoView({ block: 'nearest' }))
}

onMounted(() => window.addEventListener('keydown', onKey, true))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey, true))
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="topics" role="dialog" :aria-label="t('topics')">
      <header class="topics-header">
        <AppIcon name="list" />
        <strong>{{ t('topics') }}</strong>
        <span class="topics-hint">{{ t('topicsHint') }}</span>
      </header>
      <p v-if="tree.length === 0" class="topics-empty">{{ t('noTopics') }}</p>
      <ol ref="list" class="topics-list">
        <li v-for="(topic, i) in tree" :key="i">
          <button
            class="topic"
            :class="{ selected: i === sectionIndex && subIndex === null }"
            @click="go(topic)"
          >
            <span>{{ topic.text }}</span>
            <AppIcon
              v-if="topic.children.length"
              name="chevronRight"
              class="topic-chevron"
              :class="{ open: expanded === i }"
              @click.stop="expanded = expanded === i ? null : i"
            />
          </button>
          <ol v-if="expanded === i" class="topics-sub">
            <li v-for="(child, j) in topic.children" :key="j">
              <button
                class="topic"
                :class="{ selected: i === sectionIndex && subIndex === j }"
                @click="go(child)"
              >
                {{ child.text }}
              </button>
            </li>
          </ol>
        </li>
      </ol>
    </div>
  </div>
</template>
