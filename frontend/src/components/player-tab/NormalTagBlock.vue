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
        <button
          v-if="allowDelete"
          @click="removeTag(idx)"
          class="text-blue-400 hover:text-rose-400 transition"
          title="移除标记"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="text-[11px] text-blue-100 bg-blue-900/40 p-2 rounded-lg border border-blue-800/50">
        {{ tagItem.tag }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { AlertTriangle, Tag, X } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    privacy?: string
    tags?: Array<{ tag: string; selfPuuid?: string }>
    allowDelete?: boolean
  }>(),
  {
    privacy: 'PUBLIC',
    allowDelete: true
  }
)

const emit = defineEmits<{
  deleteTag: [index: number]
}>()

const tagsList = ref(props.tags || [])

function removeTag(idx: number) {
  tagsList.value.splice(idx, 1)
  emit('deleteTag', idx)
}
</script>
