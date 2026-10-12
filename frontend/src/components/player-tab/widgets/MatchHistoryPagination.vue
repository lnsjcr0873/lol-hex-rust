<template>
  <div class="rounded-xl bg-slate-900/80 border border-slate-800 p-2.5 px-3 flex items-center justify-between flex-wrap gap-2 text-xs text-slate-300 shadow-sm select-none">
    <!-- Queue Selection Mode -->
    <div class="flex items-center space-x-1">
      <button
        v-for="q in queueOptions"
        :key="q.value"
        @click="$emit('update:queue', q.value)"
        class="px-2.5 py-1 rounded-lg text-xs font-semibold transition active:scale-95"
        :class="queue === q.value ? 'bg-cyan-950 text-cyan-300 border border-cyan-500/50 shadow-sm' : 'hover:bg-slate-800 text-slate-400 hover:text-slate-200'"
      >
        {{ q.label }}
      </button>
    </div>

    <!-- Page Controls: Prev / Page Num / Next -->
    <div class="flex items-center space-x-2">
      <!-- Page Size Selector -->
      <select
        :value="pageSize"
        @change="$emit('update:pageSize', Number(($event.target as HTMLSelectElement).value))"
        class="bg-slate-950 border border-slate-800 rounded-lg px-2 py-0.8 text-xs text-slate-300 focus:outline-none focus:border-cyan-500 cursor-pointer"
      >
        <option :value="10">10 场/页</option>
        <option :value="20">20 场/页</option>
        <option :value="30">30 场/页</option>
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

        <span class="text-xs font-mono font-bold text-slate-200 px-1.5">
          第 {{ page }} 页
        </span>

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
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'

const queueOptions = [
  { label: '全部模式', value: 'all' },
  { label: '单双排位', value: 'ranked_solo' },
  { label: '灵活排位', value: 'ranked_flex' },
  { label: '极地大乱斗', value: 'aram' },
  { label: '匹配模式', value: 'normal' }
]

withDefaults(
  defineProps<{
    queue?: string
    page?: number
    pageSize?: number
    hasNextPage?: boolean
    disabled?: boolean
  }>(),
  {
    queue: 'all',
    page: 1,
    pageSize: 20,
    hasNextPage: true,
    disabled: false
  }
)

defineEmits<{
  'update:queue': [queue: string]
  'update:pageSize': [size: number]
  prevPage: []
  nextPage: []
}>()
</script>
