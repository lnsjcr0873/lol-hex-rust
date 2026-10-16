<template>
  <div v-if="analysis" class="w-[520px] max-w-[calc(100vw-2rem)] text-xs text-slate-200 select-none p-3.5 bg-[#0a1120] border border-[#1e2e4a] rounded-xl shadow-2xl relative z-50 backdrop-blur-md">
    <!-- Header -->
    <div class="flex items-center gap-3 pb-2 border-b border-slate-700/60">
      <ChampionIcon :champion-id="analysis.championId" class="w-10 h-10 rounded-lg shadow" />
      <div class="min-w-0 flex-1">
        <div class="truncate text-sm font-bold text-slate-100 flex items-center space-x-2">
          <span>{{ getChampionName(analysis.championId) }}</span>
          <span v-if="mastery" class="px-1.5 py-0.2 rounded text-[10px] bg-amber-950/80 text-amber-300 border border-amber-600/40">
            Lv.{{ mastery.championLevel }}
          </span>
        </div>
        <div class="mt-0.5 flex flex-wrap items-center gap-x-1.5 text-[11px] text-slate-400">
          <span>总场次: {{ analysis.winLoss?.all?.count ?? 0 }}场</span>
          <span>·</span>
          <span>胜场: {{ analysis.winLoss?.all?.wins ?? 0 }}</span>
          <span>·</span>
          <span>败场: {{ analysis.winLoss?.all?.losses ?? 0 }}</span>
        </div>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <div class="flex flex-col items-center">
          <div
            class="text-sm font-extrabold tabular-nums font-mono"
            :class="{
              'text-emerald-400': (analysis.winLoss?.all?.winRate ?? 0) >= 0.53,
              'text-slate-200': (analysis.winLoss?.all?.winRate ?? 0) > 0.47 && (analysis.winLoss?.all?.winRate ?? 0) < 0.53,
              'text-red-400': (analysis.winLoss?.all?.winRate ?? 0) <= 0.47
            }"
          >
            {{ formatPercent(analysis.winLoss?.all?.winRate ?? 0) }}
          </div>
          <div class="text-[10px] text-slate-400">
            综合胜率
          </div>
        </div>
        <NButton
          v-if="onCollectMatches"
          secondary
          size="tiny"
          type="primary"
          :loading="collecting"
          @click="onCollectMatches"
        >
          查看战绩
        </NButton>
      </div>
    </div>

    <!-- Tabs: Overview, Position, Jungle, Mastery -->
    <NTabs class="mt-2" type="line" size="small" :animated="false" default-value="overview">
      <NTabPane name="overview" tab="数据概览">
        <div class="h-56 overflow-y-auto pr-1 space-y-2.5">
          <!-- Standard vs Arena vs ARAM modes breakdown -->
          <div
            v-if="(analysis.winLoss?.normal?.count ?? 0) > 0 && (analysis.winLoss?.cherry?.count ?? 0) > 0"
            class="grid grid-cols-2 gap-2"
          >
            <div class="rounded bg-black/25 p-2 border border-slate-700/40">
              <div class="text-[10px] text-slate-400">标准峡谷模式</div>
              <div class="mt-0.5 font-bold text-slate-100 font-mono">
                {{ formatPercent(analysis.winLoss?.normal?.winRate ?? 0) }} · {{ analysis.winLoss?.normal?.wins ?? 0 }}胜{{ analysis.winLoss?.normal?.losses ?? 0 }}负
              </div>
            </div>
            <div class="rounded bg-black/25 p-2 border border-slate-700/40">
              <div class="text-[10px] text-slate-400">斗魂竞技场</div>
              <div class="mt-0.5 font-bold text-slate-100 font-mono">
                前二率 {{ formatPercent(analysis.winLoss?.cherry?.top1Rate ?? 0) }} · {{ analysis.winLoss?.cherry?.count ?? 0 }}场
              </div>
            </div>
          </div>

          <!-- Key Metrics Grid -->
          <div class="grid grid-cols-3 gap-2">
            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">近期 KDA</span>
              <div class="text-sm font-bold text-slate-100 font-mono mt-0.5">
                {{ analysis.summary?.avgKda?.toFixed(2) ?? 'N/A' }}
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5 font-mono">
                {{ averageKdaLine }}
              </div>
            </div>

            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">分均伤害</span>
              <div class="text-sm font-bold text-slate-100 font-mono mt-0.5">
                {{ Math.round(analysis.summary?.avgChampionDamagePerMinute ?? 0).toLocaleString() }}
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5">
                全队占比 {{ formatPercent(analysis.summary?.avgChampionDamagePercentageOfTeam ?? 0) }}
              </div>
            </div>

            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">承伤占比</span>
              <div class="text-sm font-bold text-slate-100 font-mono mt-0.5">
                {{ formatPercent(analysis.summary?.avgDamageTakenPercentageOfTeam ?? 0) }}
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5">
                相对队内最高 {{ formatPercent(analysis.summary?.avgDamageTakenRatioToTeamMax ?? 0) }}
              </div>
            </div>

            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">分均补兵</span>
              <div class="text-sm font-bold text-slate-100 font-mono mt-0.5">
                {{ analysis.summary?.avgCsPerMinute?.toFixed(1) ?? '0.0' }} 刀
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5">
                补兵占比 {{ formatPercent(analysis.summary?.avgCsPercentageOfTeam ?? 0) }}
              </div>
            </div>

            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">经济占比</span>
              <div class="text-sm font-bold text-slate-100 font-mono mt-0.5">
                {{ formatPercent(analysis.summary?.avgGoldPercentageOfTeam ?? 0) }}
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5">
                伤害/金币转化 {{ formatPercent(analysis.summary?.avgDamageGoldEfficiency ?? 0) }}
              </div>
            </div>

            <div class="p-2 rounded bg-black/25 border border-slate-700/40">
              <span class="text-[10px] text-slate-400">Akari 综合评分</span>
              <div class="text-sm font-bold text-fuchsia-400 font-mono mt-0.5">
                {{ analysis.akariScore?.total?.toFixed(2) ?? 'N/A' }}
              </div>
              <div class="text-[9px] text-slate-500 mt-0.5">
                参团率 {{ formatPercent(analysis.summary?.avgKillParticipation ?? 0) }}
              </div>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane v-if="hasPositionData" name="position" tab="分路占比">
        <div class="h-56 overflow-y-auto pr-1">
          <ChampionPositionChart :positions="analysis.positions!" />
        </div>
      </NTabPane>

      <NTabPane v-if="analysis.jungle" name="jungle" tab="打野分析">
        <div class="h-56 overflow-y-auto pr-1">
          <div class="mb-1 text-[10px] text-slate-400">
            该英雄打野样本: {{ analysis.jungle.gamesAnalyzed }} 场
          </div>
          <JunglePathingContent :stats="analysis.jungle" />
        </div>
      </NTabPane>

      <NTabPane v-if="mastery" name="mastery" tab="英雄熟练度">
        <div class="h-56 overflow-y-auto pr-1">
          <ChampionMasteryContent :mastery="mastery" />
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { JunglePathingContent } from '../jungle-pathing-analysis'
import ChampionIcon from '../widgets/ChampionIcon.vue'
import type { AggregatedChampionAnalysis } from '../../shared/data-adapter/analysis/player'
import { getChampionName } from '../../shared/constants/champion-names'
import { NButton, NTabPane, NTabs } from 'naive-ui'
import { computed } from 'vue'
import ChampionMasteryContent from './ChampionMasteryContent.vue'
import ChampionPositionChart from './ChampionPositionChart.vue'
import type { ChampionAnalysisMastery } from './types'

const props = defineProps<{
  analysis: AggregatedChampionAnalysis
  mastery?: ChampionAnalysisMastery | null
  collecting?: boolean
  onCollectMatches?: () => void
}>()

const formatPercent = (val: number) => `${Math.round((val || 0) * 100)}%`

const averageKdaLine = computed(() => {
  const count = Math.max(props.analysis?.winLoss?.all?.count ?? 1, 1)
  const kills = props.analysis?.summary?.kills ?? 0
  const deaths = props.analysis?.summary?.deaths ?? 0
  const assists = props.analysis?.summary?.assists ?? 0
  return `${(kills / count).toFixed(1)} / ${(deaths / count).toFixed(1)} / ${(assists / count).toFixed(1)}`
})

const hasPositionData = computed(() => {
  return Boolean(props.analysis?.positions)
})
</script>
