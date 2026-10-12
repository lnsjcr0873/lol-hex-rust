<template>
  <div class="w-64 text-xs text-slate-200 select-none p-1">
    <div class="flex items-center justify-between gap-4 pb-2 border-b border-slate-700/60">
      <div class="flex items-center space-x-1.5">
        <span class="w-2 h-2 rounded-full bg-fuchsia-500"></span>
        <span class="font-bold text-slate-100">
          Akari 综合评分
        </span>
      </div>
      <span class="font-bold text-slate-100 tabular-nums">
        <span class="text-fuchsia-400 font-extrabold">{{ formatValue(score.total, totalPrecision) }}</span>
        <span class="px-1 text-slate-500">/</span>
        <span class="text-slate-400">{{ formatMax(score.maxScore) }}</span>
      </span>
    </div>

    <div class="mt-2.5 flex flex-col gap-2">
      <div v-for="item in items" :key="item.key" class="flex flex-col gap-1">
        <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-x-3 text-[11px]">
          <span class="truncate text-slate-400">
            {{ item.label }}
          </span>
          <span class="text-right text-slate-200 tabular-nums font-mono">
            <span :class="item.value > 0 ? 'text-slate-100 font-semibold' : 'text-slate-500'">{{ formatValue(item.value) }}</span>
            <span class="px-1 text-slate-500">/</span>
            <span class="text-slate-400">{{ formatMax(item.max) }}</span>
          </span>
        </div>
        <NProgress
          :height="5"
          :percentage="item.progressPercentage"
          :show-indicator="false"
          :status="item.progressStatus"
          color="#d946ef"
          rail-color="rgba(255, 255, 255, 0.08)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AkariScore } from '../../shared/data-adapter/analysis/player'
import { NProgress } from 'naive-ui'
import { computed } from 'vue'
import { getAkariScoreBreakdownItems } from './score-breakdown'

const props = withDefaults(
  defineProps<{
    score: AkariScore
    totalPrecision?: number
  }>(),
  {
    totalPrecision: 2
  }
)

const items = computed(() => getAkariScoreBreakdownItems(props.score))

const formatValue = (value: number, precision = 2) => {
  if (!Number.isFinite(value)) {
    return 'N/A'
  }
  return value.toFixed(precision)
}

const formatMax = (value: number | null) => {
  if (value === null) {
    return '无上限'
  }
  return Number.isInteger(value) ? value.toString() : value.toFixed(2)
}
</script>
