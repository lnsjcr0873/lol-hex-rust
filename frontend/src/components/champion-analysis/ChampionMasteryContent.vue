<template>
  <div class="grid grid-cols-2 gap-1.5 text-xs select-none">
    <div class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">熟练度等级</div>
      <div class="mt-0.5 font-bold text-amber-400 tabular-nums">
        Lv. {{ mastery.championLevel }}
      </div>
    </div>

    <div class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">熟练度总点数</div>
      <div class="mt-0.5 font-bold text-slate-100 tabular-nums font-mono">
        {{ (mastery.championPoints || 0).toLocaleString() }}
      </div>
    </div>

    <div v-if="mastery.highestGrade" class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">历史最高评级</div>
      <div class="mt-0.5 font-bold text-emerald-400">
        {{ mastery.highestGrade }}
      </div>
    </div>

    <div v-if="mastery.championSeasonMilestone" class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">赛季里程碑点数</div>
      <div class="mt-0.5 font-bold text-slate-200">
        {{ mastery.championSeasonMilestone }}
      </div>
    </div>

    <div v-if="mastery.tokensEarned" class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">已获代币</div>
      <div class="mt-0.5 font-bold text-slate-200">
        {{ mastery.tokensEarned }}
      </div>
    </div>

    <div v-if="mastery.lastPlayTime" class="rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">最后使用日期</div>
      <div class="mt-0.5 font-semibold text-slate-300 font-mono">
        {{ formatMasteryTime(mastery.lastPlayTime) }}
      </div>
    </div>

    <div v-if="hasLevelProgress" class="col-span-2 rounded bg-black/20 p-2 border border-slate-700/50">
      <div class="text-[10px] text-slate-400">升级进度</div>
      <div class="mt-0.5 text-xs text-slate-200 font-mono">
        {{ (mastery.championPointsSinceLastLevel || 0).toLocaleString() }} /
        {{ (mastery.championPointsUntilNextLevel || 0).toLocaleString() }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs'
import { computed } from 'vue'
import type { ChampionAnalysisMastery } from './types'

const { mastery } = defineProps<{
  mastery: ChampionAnalysisMastery
}>()

const formatMasteryTime = (value: number) => dayjs(value).format('YYYY-MM-DD')

const hasLevelProgress = computed(
  () =>
    mastery.championPointsSinceLastLevel !== undefined &&
    mastery.championPointsUntilNextLevel !== undefined
)
</script>
