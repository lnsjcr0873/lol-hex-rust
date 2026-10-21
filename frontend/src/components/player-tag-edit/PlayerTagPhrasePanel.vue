<template>
  <div class="flex flex-col gap-1.5">
    <div class="flex min-w-0 items-center justify-between gap-2">
      <div class="flex min-w-0 items-center gap-1.5">
        <NButton
          text
          size="tiny"
          class="cursor-pointer py-1!"
          :focusable="false"
          @click="handleToggleExpanded"
        >
          <span class="flex items-center gap-1.5">
            <NIcon size="14">
              <ChevronDown20Regular v-if="expanded" />
              <ChevronRight20Regular v-else />
            </NIcon>
            <span>预设短语</span>
          </span>
        </NButton>

        <span class="text-[11px] text-slate-400">
          {{ phrases.length }} / {{ maxItems }}
        </span>
      </div>

      <NPopconfirm
        v-if="expanded"
        type="warning"
        :show-icon="false"
        positive-text="确认清除"
        negative-text="取消"
        :positive-button-props="{ type: 'warning', size: 'tiny' }"
        :negative-button-props="{ size: 'tiny' }"
        @positive-click="emit('clear')"
      >
        <template #trigger>
          <NButton
            text
            size="tiny"
            class="cursor-pointer py-1!"
            :disabled="disabled || !canClear"
            :focusable="false"
          >
            清除标记
          </NButton>
        </template>
        确认清除该玩家的全部标记内容？
      </NPopconfirm>
    </div>

    <NCollapseTransition :show="expanded">
      <div class="flex flex-col gap-2 py-1">
        <div v-if="phrases.length || !isAtLimit" class="flex flex-wrap gap-1">
          <NButtonGroup v-for="phrase in phrases" :key="phrase" size="tiny">
            <NButton size="tiny" secondary :disabled="disabled" @click="emit('append', phrase)">
              <span class="max-w-[200px] truncate" :title="phrase">{{ phrase }}</span>
            </NButton>
            <NTooltip>
              <template #trigger>
                <NButton
                  size="tiny"
                  secondary
                  :disabled="disabled"
                  :aria-label="`删除短语 ${phrase}`"
                  @click="handleDelete(phrase)"
                >
                  <template #icon>
                    <NIcon><Dismiss20Regular /></NIcon>
                  </template>
                </NButton>
              </template>
              删除短语 {{ phrase }}
            </NTooltip>
          </NButtonGroup>

          <NButton
            v-if="!isAddingPhrase && !isAtLimit"
            size="tiny"
            quaternary
            :disabled="disabled"
            @click="handleStartAdding"
          >
            <template #icon>
              <NIcon><Add20Regular /></NIcon>
            </template>
            添加短语
          </NButton>
        </div>

        <div v-if="isAddingPhrase" class="flex min-w-0 items-start gap-1.5">
          <NInput
            ref="phraseInput"
            v-model:value="draftPhrase"
            size="small"
            clearable
            show-count
            :maxlength="maxPhraseLength"
            :disabled="disabled || isAtLimit"
            placeholder="输入预设短语（最多100字）"
            @keydown="handlePhraseInputKeydown"
          />

          <NTooltip :disabled="!addDisabledReason">
            <template #trigger>
              <span class="inline-flex">
                <NButton
                  size="small"
                  type="primary"
                  secondary
                  :disabled="!canAdd"
                  aria-label="保存短语"
                  @click="handleAdd"
                >
                  <template #icon>
                    <NIcon><Add20Regular /></NIcon>
                  </template>
                </NButton>
              </span>
            </template>
            {{ addDisabledReason }}
          </NTooltip>
          <NButton
            size="small"
            quaternary
            :disabled="disabled"
            aria-label="取消添加"
            @click="handleCancelAdding"
          >
            <template #icon>
              <NIcon><Dismiss20Regular /></NIcon>
            </template>
          </NButton>
        </div>

        <div
          v-if="!phrases.length && !isAddingPhrase"
          class="text-xs leading-5 text-slate-400"
        >
          暂无预设短语，点击上方按钮添加常用快捷短语。
        </div>
      </div>
    </NCollapseTransition>
  </div>
</template>

<script setup lang="ts">
import {
  Add20Regular,
  ChevronDown20Regular,
  ChevronRight20Regular,
  Dismiss20Regular
} from '@vicons/fluent'
import {
  NButton,
  NButtonGroup,
  NCollapseTransition,
  NIcon,
  NInput,
  NPopconfirm,
  NTooltip,
  type InputInst
} from 'naive-ui'
import { computed, nextTick, ref, watch } from 'vue'
import {
  SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS,
  SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH
} from '../player-tab/constants'
import {
  normalizeSavedPlayerTagPhrase,
  normalizeSavedPlayerTagPhrases
} from './cursor'

const props = withDefaults(
  defineProps<{
    phrases: string[]
    expanded: boolean
    disabled?: boolean
    canClear?: boolean
  }>(),
  {
    disabled: false,
    canClear: false
  }
)

const emit = defineEmits<{
  'update:phrases': [phrases: string[]]
  'update:expanded': [expanded: boolean]
  append: [phrase: string]
  clear: []
}>()

const maxItems = SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS
const maxPhraseLength = SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH
const draftPhrase = ref('')
const isAddingPhrase = ref(false)
const phraseInput = ref<InputInst | null>(null)

const normalizedDraftPhrase = computed(() => normalizeSavedPlayerTagPhrase(draftPhrase.value))
const isAtLimit = computed(() => props.phrases.length >= maxItems)
const isDuplicate = computed(() => props.phrases.includes(normalizedDraftPhrase.value))
const canAdd = computed(
  () =>
    !props.disabled &&
    !isAtLimit.value &&
    Boolean(normalizedDraftPhrase.value) &&
    !isDuplicate.value
)

const addDisabledReason = computed(() => {
  if (normalizedDraftPhrase.value && isDuplicate.value) {
    return '该短语已存在'
  }
  return ''
})

const handleStartAdding = () => {
  if (props.disabled || isAtLimit.value) {
    return
  }

  isAddingPhrase.value = true
  void nextTick(() => phraseInput.value?.focus())
}

const handleCancelAdding = () => {
  isAddingPhrase.value = false
  draftPhrase.value = ''
}

const handleToggleExpanded = () => {
  if (props.expanded) {
    handleCancelAdding()
  }

  emit('update:expanded', !props.expanded)
}

const handlePhraseInputKeydown = (event: KeyboardEvent) => {
  if (event.isComposing) {
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    handleAdd()
    return
  }

  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    handleCancelAdding()
  }
}

const handleAdd = () => {
  if (!canAdd.value) {
    return
  }

  emit(
    'update:phrases',
    normalizeSavedPlayerTagPhrases([...props.phrases, normalizedDraftPhrase.value])
  )
  handleCancelAdding()
}

const handleDelete = (phrase: string) => {
  if (props.disabled) {
    return
  }

  emit(
    'update:phrases',
    props.phrases.filter((item) => item !== phrase)
  )
}

watch(
  () => props.expanded,
  (expanded) => {
    if (!expanded) {
      handleCancelAdding()
    }
  }
)

watch(isAtLimit, (atLimit) => {
  if (atLimit) {
    handleCancelAdding()
  }
})
</script>
