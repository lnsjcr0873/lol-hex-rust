<template>
  <div class="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-300 select-none space-y-2.5 shadow-sm">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <span class="font-extrabold text-sm text-slate-100">英雄熟练度排行</span>
      <span class="text-[10px] text-slate-400 font-mono">总成就点: {{ totalScore.toLocaleString() }}</span>
    </div>

    <!-- Masteries list -->
    <div class="space-y-2">
      <div
        v-for="item in masteries"
        :key="item.championId"
        class="flex items-center justify-between p-1.5 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 transition"
      >
        <div class="flex items-center space-x-2.5">
          <ChampionIcon :champion-id="item.championId" :size="32" round />
          <div>
            <div class="font-bold text-slate-100">{{ getChampionName(item.championId) }}</div>
            <div class="text-[10px] text-amber-400 font-mono">
              第 {{ item.championLevel }} 等级
              <span v-if="item.highestGrade" class="ml-1 px-1 bg-amber-500/20 rounded font-bold text-amber-300">
                {{ item.highestGrade }}
              </span>
            </div>
          </div>
        </div>

        <div class="text-right font-mono">
          <div class="text-[11px] font-bold text-slate-200">
            {{ formatPoints(item.championPoints) }}
          </div>
          <div class="text-[9px] text-slate-500">熟练度点数</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'
import { getChampionName } from '../match-card/utils/resources'

const props = defineProps<{
  masteriesList?: Array<{
    championId: number
    championLevel: number
    championPoints: number
    highestGrade?: string
  }>
}>()

const masteries = computed(() => {
  if (props.masteriesList && props.masteriesList.length > 0) {
    return props.masteriesList
  }
  return [
    { championId: 266, championLevel: 7, championPoints: 348200, highestGrade: 'S+' },
    { championId: 84, championLevel: 7, championPoints: 219400, highestGrade: 'S' },
    { championId: 39, championLevel: 6, championPoints: 145000, highestGrade: 'S-' },
    { championId: 64, championLevel: 5, championPoints: 89000, highestGrade: 'A+' }
  ]
})

const totalScore = computed(() => {
  return masteries.value.reduce((acc, m) => acc + m.championPoints, 0)
})

function formatPoints(pts: number): string {
  if (pts >= 10000) {
    return `${(pts / 10000).toFixed(1)}万`
  }
  return pts.toLocaleString()
}
</script>
