<template>
  <div class="flex min-h-44 items-center justify-center gap-6 select-none p-2">
    <div class="w-36 h-36 shrink-0 relative flex items-center justify-center">
      <Doughnut v-if="entries.length" :data="chartData" :options="chartOptions" />
      <span v-else class="text-xs text-slate-500">无位置数据</span>
    </div>
    <div class="grid min-w-44 gap-1.5 text-xs">
      <div
        v-for="entry of entries"
        :key="entry.position"
        class="grid grid-cols-[10px_1fr_auto] items-center gap-2 rounded bg-slate-800/60 border border-slate-700/50 px-2 py-1.5"
      >
        <span class="w-2.5 h-2.5 rounded-full" :style="{ backgroundColor: entry.color }" />
        <span class="text-slate-200 font-medium">
          {{ entry.label }}
        </span>
        <span class="font-semibold text-slate-100 tabular-nums font-mono">
          {{ entry.count }}场 · {{ formatPercent(entry.count / (total || 1)) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AggregatedPositionAnalysis } from '../../shared/data-adapter/analysis/player/aggregate/positions'
import { ArcElement, Chart as ChartJS, type ChartData, type ChartOptions, Tooltip } from 'chart.js'
import { computed } from 'vue'
import { Doughnut } from 'vue-chartjs'

ChartJS.register(ArcElement, Tooltip)

const props = defineProps<{
  positions: AggregatedPositionAnalysis
}>()

const POSITION_COLORS = {
  TOP: '#ef4444',
  JUNGLE: '#10b981',
  MIDDLE: '#f59e0b',
  BOTTOM: '#3b82f6',
  UTILITY: '#a855f7'
} as const

const POSITION_LABELS: Record<string, string> = {
  TOP: '上路',
  JUNGLE: '打野',
  MIDDLE: '中路',
  BOTTOM: '下路',
  UTILITY: '辅助'
}

const entries = computed(() => {
  if (!props.positions) return []
  return Object.entries(props.positions)
    .filter(([, count]) => count > 0)
    .sort((a, b) => b[1] - a[1])
    .map(([position, count]) => ({
      position: position as keyof typeof POSITION_COLORS,
      count,
      label: POSITION_LABELS[position] ?? position,
      color: POSITION_COLORS[position as keyof typeof POSITION_COLORS] ?? '#94a3b8'
    }))
})

const total = computed(() => entries.value.reduce((sum, entry) => sum + entry.count, 0))
const formatPercent = (value: number) => `${(value * 100).toFixed(0)}%`

const chartData = computed<ChartData<'doughnut'>>(() => ({
  labels: entries.value.map((entry) => entry.label),
  datasets: [
    {
      data: entries.value.map((entry) => entry.count),
      backgroundColor: entries.value.map((entry) => entry.color),
      borderWidth: 0,
      hoverOffset: 3
    }
  ]
}))

const chartOptions: ChartOptions<'doughnut'> = {
  responsive: true,
  maintainAspectRatio: false,
  animation: false,
  cutout: '62%',
  plugins: {
    legend: { display: false },
    tooltip: {
      callbacks: {
        label: (context) =>
          `${context.label}: ${context.parsed}场 (${formatPercent(context.parsed / (total.value || 1))})`
      }
    }
  }
}
</script>
