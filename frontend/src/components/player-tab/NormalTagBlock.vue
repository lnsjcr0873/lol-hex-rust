<template>
  <div class="space-y-2 select-none">
    <!-- Privacy warning block -->
    <div
      v-if="privacy === 'PRIVATE'"
      class="rounded-xl bg-red-950/80 border border-red-800/80 p-3 text-red-200 text-xs shadow-sm"
    >
      <div class="mb-1 font-bold flex items-center space-x-1.5 text-red-300">
        <AlertTriangle class="w-4 h-4" />
        <span>召唤师生涯私密保护</span>
      </div>
      <div class="text-[11px] leading-relaxed text-red-300/80">
        该玩家在国服开启了生涯私密保护，部分历史战绩或成就数据可能受限。
      </div>
    </div>

    <!-- Tagged player notes -->
    <div
      v-for="(tagItem, idx) in tagsList"
      :key="idx"
      class="rounded-xl bg-blue-950/80 border border-blue-800/80 p-3 text-blue-200 text-xs shadow-sm"
    >
      <div class="mb-1.5 flex items-center justify-between">
        <div class="flex items-center space-x-1.5 font-bold text-blue-300">
          <Tag class="w-3.5 h-3.5 text-cyan-400" />
          <span>召唤师标记备注</span>
        </div>

        <NPopconfirm
          v-if="allowDelete"
          positive-text="确认删除"
          negative-text="取消"
          :positive-button-props="{ type: 'warning', size: 'tiny' }"
          :negative-button-props="{ size: 'tiny' }"
          @positive-click="removeTag(tagItem, idx)"
        >
          <template #trigger>
            <button
              class="text-blue-400 hover:text-rose-400 transition"
              title="移除标记"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </template>
          确定删除该玩家的标记备注吗？
        </NPopconfirm>
      </div>

      <div class="text-[11px] text-blue-100 bg-blue-900/40 p-2 rounded-lg border border-blue-800/50 break-words whitespace-pre-wrap">
        {{ tagItem.tag }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { AlertTriangle, Tag, X } from 'lucide-vue-next'
import { NPopconfirm, useMessage } from 'naive-ui'
import { deletePlayerTag } from '../../api/tauri'
import type { SavedPlayerTagItem } from '../../api/types'

const props = withDefaults(
  defineProps<{
    privacy?: string
    tags?: (SavedPlayerTagItem | { tag: string; puuid?: string; selfPuuid?: string; markedBySelf?: boolean })[]
    selfPuuid?: string
    allowDelete?: boolean
  }>(),
  {
    privacy: 'PUBLIC',
    allowDelete: true
  }
)

const emit = defineEmits<{
  deleteTag: [index: number]
  tagDeleted: []
}>()

const message = useMessage()
const tagsList = ref(props.tags || [])

watch(
  () => props.tags,
  (newTags) => {
    tagsList.value = newTags ? [...newTags] : []
  },
  { deep: true }
)

async function removeTag(tagItem: any, idx: number) {
  if (tagItem.puuid) {
    try {
      await deletePlayerTag(tagItem.puuid, tagItem.selfPuuid || props.selfPuuid)
      message.success('已移除玩家标记')
    } catch (err) {
      console.warn('Failed to delete player tag via SQLite IPC:', err)
    }
  }
  tagsList.value.splice(idx, 1)
  emit('deleteTag', idx)
  emit('tagDeleted')
}
</script>
