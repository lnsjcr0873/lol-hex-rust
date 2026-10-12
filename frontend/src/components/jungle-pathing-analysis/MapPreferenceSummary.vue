<template>
  <div>
    <div class="mb-1 font-bold text-slate-100 flex items-center gap-1.5">
      <span>分路倾向分析</span>
    </div>
    <div class="flex flex-col gap-1 text-[11px]">
      <div class="flex items-center gap-1.5 text-slate-300">
        <span class="text-slate-200">样本分析: {{ stats.gamesAnalyzed }} 场</span>
        <span class="text-slate-600">·</span>
        <span class="whitespace-nowrap font-semibold" :class="topsideTextColor(stats)">
          {{ topsideTextWithPct(stats) }}
        </span>
      </div>

      <div class="flex items-baseline gap-2 text-slate-300">
        <span class="shrink-0 text-slate-400">区域权重:</span>
        <span class="inline-flex gap-2 whitespace-nowrap font-mono text-[11px]">
          <span class="inline-flex items-baseline gap-1">
            <span class="text-red-400 font-semibold">上</span>
            <span class="text-slate-200">{{ formatWeightSum(stats.topZoneWeightSum) }}</span>
          </span>
          <span class="text-slate-600">|</span>
          <span class="inline-flex items-baseline gap-1">
            <span class="text-amber-400 font-semibold">中</span>
            <span class="text-slate-200">{{ formatWeightSum(stats.midZoneWeightSum) }}</span>
          </span>
          <span class="text-slate-600">|</span>
          <span class="inline-flex items-baseline gap-1">
            <span class="text-blue-400 font-semibold">下</span>
            <span class="text-slate-200">{{ formatWeightSum(stats.botZoneWeightSum) }}</span>
          </span>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'
import { formatWeightSum, topsideTextColor, topsideTextWithPct } from './preference'

defineProps<{
  stats: AggregatedJungleAnalysis
}>()
</script>
