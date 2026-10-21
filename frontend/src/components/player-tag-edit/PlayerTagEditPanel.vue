<template>
  <div class="w-[380px] p-2 text-slate-200 select-none">
    <!-- Header: Summoner Identity -->
    <div class="mb-3 flex min-w-0 items-center gap-2.5">
      <div class="mr-auto text-sm font-bold text-slate-100">玩家标记备注</div>

      <template v-if="summoner">
        <img
          :src="getProfileIconUrl(summoner.profileIconId || 1)"
          class="w-6 h-6 shrink-0 rounded border border-slate-700 object-cover"
          @error="(e: any) => { e.target.src = 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/1.png' }"
        />
        <StreamerModeMaskedText :puuid="props.puuid">
          <template #masked>
            <div class="flex min-w-0 items-baseline gap-1">
              <span class="max-w-[120px] truncate text-xs font-bold text-slate-200">{{ maskedSummonerName }}</span>
              <span class="text-[11px] text-slate-400">#{{ maskedTagLine }}</span>
            </div>
          </template>
          <div class="flex min-w-0 items-baseline gap-1">
            <span class="max-w-[120px] truncate text-xs font-bold text-slate-200">{{ summoner.gameName }}</span>
            <span class="text-[11px] text-slate-400">#{{ summoner.tagLine }}</span>
          </div>
        </StreamerModeMaskedText>
      </template>

      <template v-else>
        <span class="text-xs text-slate-400">
          {{ isLoadingTag ? '正在读取标记...' : '未知召唤师' }}
        </span>
      </template>
    </div>

    <!-- Textarea -->
    <NInput
      ref="inputEl"
      v-model:value="text"
      :placeholder="placeholderText"
      type="textarea"
      :autosize="{ minRows: 3, maxRows: 4 }"
      :disabled="isLoadingTag || isSaving"
      @keydown="handleInputKeydown"
    />

    <!-- Preset Phrases Panel -->
    <PlayerTagPhrasePanel
      class="mt-2.5"
      :phrases="presetPhrases"
      :expanded="isPhrasePanelExpanded"
      :disabled="!isReady || isUpdatingPhrases"
      :can-clear="Boolean(text)"
      @append="appendPhrase"
      @clear="clearText"
      @update:expanded="handleUpdatePhrasePanelExpanded"
      @update:phrases="handleUpdatePhrases"
    />

    <!-- Footer Actions -->
    <div class="mt-3.5 flex justify-end gap-2 items-center">
      <NButton size="small" :disabled="isSaving" @click="emit('cancel')">
        取消
      </NButton>
      <NButton
        size="small"
        type="primary"
        :disabled="!isReady"
        :loading="isSaving"
        @click="handleSaveTag"
      >
        <span class="flex items-center gap-1.5 font-medium">
          <span class="text-xs px-1 py-0.5 rounded bg-black/20 text-slate-300 font-mono">Shift+Enter</span>
          <span>保存</span>
        </span>
      </NButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { NButton, NInput, useMessage, type InputInst } from 'naive-ui'
import StreamerModeMaskedText from '../widgets/StreamerModeMaskedText.vue'
import { useStreamerModeMaskedText } from '../widgets/useStreamerModeMaskedText'
import { getProfileIconUrl } from '../match-card/utils/resources'
import {
  getPlayerTags,
  savePlayerTag,
  deletePlayerTag,
  getPlayerTagPhrases,
  updatePlayerTagPhrases,
  setPlayerTagPhrasePanelExpanded
} from '../../api/tauri'
import { focusTextInput, insertTextAtSelection } from './cursor'
import PlayerTagPhrasePanel from './PlayerTagPhrasePanel.vue'
import type { PlayerTagEditPanelSummoner } from './types'

const props = defineProps<{
  puuid: string
  selfPuuid?: string
  summoner?: PlayerTagEditPanelSummoner | null
}>()

const emit = defineEmits<{
  saved: []
  cancel: []
}>()

const message = useMessage()
const { masked, summonerName: streamerSummonerName } = useStreamerModeMaskedText()

const text = ref('')
const hasLoadedTag = ref(false)
const isLoadingTag = ref(false)
const isSaving = ref(false)
const isUpdatingPhrases = ref(false)
const isPhrasePanelExpanded = ref(false)
const presetPhrases = ref<string[]>([
  '绝活哥', '通天代', '连胜中', '容易被抓', '单杀怪', '心态好', '车队打手', '带崩全队'
])
const inputEl = ref<InputInst | null>(null)

const isReady = computed(() => hasLoadedTag.value && !isLoadingTag.value && !isSaving.value)

const placeholderText = computed(() => {
  const name = props.summoner ? `${props.summoner.gameName} #${props.summoner.tagLine}` : '该玩家'
  return `输入针对 ${name} 的战术标记或评价备注...`
})

const maskedSummonerName = computed(() => {
  const seed = props.summoner?.gameName || props.summoner?.puuid || props.puuid
  return streamerSummonerName(seed, 0)
})

const maskedTagLine = computed(() => masked(props.summoner?.tagLine || '', '#####'))

const focus = (cursorPosition?: number) => {
  requestAnimationFrame(() => {
    focusTextInput(inputEl.value, cursorPosition)
  })
}

const appendPhrase = (phrase: string) => {
  const result = insertTextAtSelection(inputEl.value, text.value, phrase)
  text.value = result.value
  focus(result.cursorPosition)
}

const clearText = () => {
  text.value = ''
  focus(0)
}

const handleUpdatePhrases = async (phrases: string[]) => {
  if (isUpdatingPhrases.value) return
  isUpdatingPhrases.value = true
  try {
    const updated = await updatePlayerTagPhrases(phrases)
    presetPhrases.value = updated
  } catch (err) {
    console.warn('Failed to update player tag phrases:', err)
    message.warning('更新预设短语失败')
  } finally {
    isUpdatingPhrases.value = false
  }
}

const handleUpdatePhrasePanelExpanded = async (expanded: boolean) => {
  isPhrasePanelExpanded.value = expanded
  try {
    await setPlayerTagPhrasePanelExpanded(expanded)
  } catch (err) {
    console.warn('Failed to persist phrase panel expanded state:', err)
  }
}

const loadSelfTag = async () => {
  hasLoadedTag.value = false
  if (!props.puuid) {
    text.value = ''
    hasLoadedTag.value = true
    return
  }

  isLoadingTag.value = true
  try {
    const [tags, phrases] = await Promise.all([
      getPlayerTags(props.puuid, props.selfPuuid),
      getPlayerTagPhrases().catch(() => presetPhrases.value)
    ])

    if (Array.isArray(phrases) && phrases.length > 0) {
      presetPhrases.value = phrases
    }

    const selfTag = tags.find((t) => t.markedBySelf) || tags[0]
    text.value = selfTag ? selfTag.tag : ''
  } catch (err) {
    console.warn('Failed to load player tag from SQLite:', err)
  } finally {
    isLoadingTag.value = false
    hasLoadedTag.value = true
  }
}

const handleSaveTag = async () => {
  if (!isReady.value || !props.puuid) return

  isSaving.value = true
  const trimmed = text.value.trim()

  try {
    if (trimmed) {
      await savePlayerTag({
        puuid: props.puuid,
        selfPuuid: props.selfPuuid,
        tag: trimmed
      })
    } else {
      await deletePlayerTag(props.puuid, props.selfPuuid)
    }

    message.success('玩家标记已保存')
    emit('saved')
  } catch (err) {
    console.warn('Failed to save player tag to SQLite:', err)
    message.error('保存玩家标记失败')
  } finally {
    isSaving.value = false
  }
}

const isSaveTagShortcut = (event: KeyboardEvent) => {
  return (
    (event.key === 'Enter' || event.code === 'NumpadEnter') &&
    event.shiftKey &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.metaKey &&
    !event.isComposing
  )
}

const handleInputKeydown = (event: KeyboardEvent) => {
  if (!isSaveTagShortcut(event) || !isReady.value) return

  event.preventDefault()
  event.stopPropagation()
  handleSaveTag()
}

onMounted(() => {
  loadSelfTag()
})

watch(
  () => props.puuid,
  () => {
    loadSelfTag()
  }
)

watch(
  isReady,
  (ready) => {
    if (ready) {
      focus()
    }
  },
  { flush: 'post' }
)
</script>
