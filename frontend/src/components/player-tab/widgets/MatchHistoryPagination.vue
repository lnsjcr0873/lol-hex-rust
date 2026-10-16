<template>
  <div class="rounded-xl bg-slate-900/80 border border-slate-800 p-2.5 px-3 flex items-center justify-between flex-wrap gap-2 text-xs text-slate-300 shadow-sm select-none">
    <!-- Queue Selection Mode -->
    <div class="flex items-center space-x-1">
      <button
        v-for="q in queueOptions"
        :key="q.value"
        :disabled="disabled"
        @click="$emit('update:queue', q.value)"
        class="px-2.5 py-1 rounded-lg text-xs font-semibold transition active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed"
        :class="queue === q.value ? 'bg-cyan-950 text-cyan-300 border border-cyan-500/50 shadow-sm' : 'hover:bg-slate-800 text-slate-400 hover:text-slate-200'"
      >
        {{ q.label }}
      </button>
    </div>

    <!-- Right Controls: Filter Button + Page Controls -->
    <div class="flex items-center space-x-2">
      <!-- Advanced Filter Trigger Button -->
      <button
        @click="$emit('openFilters')"
        :disabled="disabled"
        class="px-2.5 py-1 rounded-lg text-xs font-semibold transition active:scale-95 flex items-center space-x-1.5 border"
        :class="hasActiveFilter ? 'bg-cyan-950 text-cyan-300 border-cyan-500/60 shadow-sm' : 'bg-slate-950/60 hover:bg-slate-800 border-slate-800 text-slate-400 hover:text-slate-200'"
        title="高级战绩筛选器与深度收集"
      >
        <Filter class="w-3.5 h-3.5" :class="{ 'text-cyan-400': hasActiveFilter }" />
        <span>筛选</span>
        <span
          v-if="hasActiveFilter"
          class="w-2 h-2 rounded-full bg-cyan-400 shadow-sm shadow-cyan-400/50"
        ></span>
      </button>

      <div class="h-4 w-px bg-slate-800 shrink-0 mx-0.5"></div>

      <!-- Page Size Selector: Full 7 Options [10, 20, 30, 40, 50, 100, 200] -->
      <select
        :value="pageSize"
        :disabled="disabled"
        @change="$emit('update:pageSize', Number(($event.target as HTMLSelectElement).value))"
        class="bg-slate-950 border border-slate-800 rounded-lg px-2 py-0.8 text-xs text-slate-300 focus:outline-none focus:border-cyan-500 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
      >
        <option v-for="size in PAGE_SIZE_OPTIONS" :key="size" :value="size">
          {{ size }} 场/页
        </option>
      </select>

      <!-- Pagination Buttons -->
      <div class="flex items-center space-x-1">
        <button
          @click="$emit('prevPage')"
          :disabled="page <= 1 || disabled"
          class="p-1 rounded-lg bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
          title="上一页"
        >
          <ChevronLeft class="w-3.5 h-3.5" />
        </button>

        <!-- Clickable Page Number Opening Popover for Direct Jump & Back to First Page -->
        <NPopover
          v-model:show="isPopoverOpen"
          trigger="click"
          :disabled="disabled"
          placement="bottom"
        >
          <template #trigger>
            <button
              :disabled="disabled"
              class="text-xs font-mono font-bold text-slate-200 px-2 py-0.5 rounded-md hover:bg-slate-800 hover:text-cyan-400 transition cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
              title="点击跳转指定页"
            >
              第 {{ page }} 页
            </button>
          </template>
          <div class="flex flex-col gap-2 p-1.5 text-xs select-none min-w-[190px]">
            <span class="text-[11px] text-slate-400 font-medium">跳转到指定页</span>
            <div class="flex items-center gap-1.5">
              <NInputNumber
                v-model:value="jumpTargetPage"
                :min="1"
                :disabled="disabled"
                size="small"
                placeholder="页码"
                @keydown.enter="handleJump"
                class="w-24!"
              />
              <button
                @click="handleJump"
                :disabled="disabled || !jumpTargetPage || jumpTargetPage < 1"
                class="px-2 py-1 rounded-md bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 disabled:cursor-not-allowed text-white text-xs font-medium transition flex items-center gap-1 shrink-0"
                title="跳转至该页"
              >
                <span>跳转</span>
                <ArrowRight class="w-3 h-3" />
              </button>
              <button
                @click="handleGoFirst"
                :disabled="disabled || page <= 1"
                class="p-1 rounded-md bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 hover:text-white transition flex items-center justify-center shrink-0"
                title="回到第一页"
              >
                <ChevronsLeft class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </NPopover>

        <button
          @click="$emit('nextPage')"
          :disabled="!hasNextPage || disabled"
          class="p-1 rounded-lg bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
          title="下一页"
        >
          <ChevronRight class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ChevronLeft, ChevronRight, ChevronsLeft, ArrowRight, Filter } from 'lucide-vue-next'
import { NPopover, NInputNumber } from 'naive-ui'

const PAGE_SIZE_OPTIONS = [10, 20, 30, 40, 50, 100, 200] as const

const queueOptions = [
  { label: '全部模式', value: 'all' },
  { label: '单双排位', value: 'ranked_solo' },
  { label: '灵活排位', value: 'ranked_flex' },
  { label: '极地大乱斗', value: 'aram' },
  { label: '匹配模式', value: 'normal' }
]

const props = withDefaults(
  defineProps<{
    queue?: string
    page?: number
    pageSize?: number
    hasNextPage?: boolean
    disabled?: boolean
    hasActiveFilter?: boolean
  }>(),
  {
    queue: 'all',
    page: 1,
    pageSize: 20,
    hasNextPage: true,
    disabled: false,
    hasActiveFilter: false
  }
)

const emit = defineEmits<{
  'update:queue': [queue: string]
  'update:pageSize': [size: number]
  'update:page': [page: number]
  prevPage: []
  nextPage: []
  goToPage: [page: number]
  firstPage: []
  openFilters: []
}>()

const isPopoverOpen = ref(false)
const jumpTargetPage = ref<number | null>(props.page)

watch(
  () => props.page,
  (newPage) => {
    jumpTargetPage.value = newPage
  }
)

watch(isPopoverOpen, (open) => {
  if (open) {
    jumpTargetPage.value = props.page
  }
})

function handleJump() {
  if (props.disabled || !jumpTargetPage.value || !Number.isFinite(jumpTargetPage.value) || jumpTargetPage.value < 1) return
  const target = Math.max(1, Math.min(10000, Math.floor(jumpTargetPage.value)))
  isPopoverOpen.value = false
  emit('update:page', target)
  emit('goToPage', target)
}

function handleGoFirst() {
  if (props.disabled || props.page <= 1) return
  isPopoverOpen.value = false
  jumpTargetPage.value = 1
  emit('update:page', 1)
  emit('firstPage')
}
</script>
