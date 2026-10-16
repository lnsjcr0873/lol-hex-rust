<template>
  <div class="mb-1.5 flex items-center space-x-1.5 select-none">
    <NPopover
      v-for="c of championList"
      :key="c.id"
      :delay="60"
      :keep-alive-on-hover="true"
      placement="top"
    >
      <template #trigger>
        <div
          class="relative w-6 h-6 rounded cursor-pointer transition-transform hover:scale-110 shrink-0"
        >
          <ChampionIcon
            :champion-id="c.id"
            ring
            :ring-width="1.5"
            :ring-color="c.winRate >= 50 ? '#3b82f6' : '#ef4444'"
            class="w-full h-full rounded shadow"
          />
          <span
            v-if="c.isStar"
            class="absolute -bottom-1 -right-1 text-[9px] text-amber-300 font-bold leading-none pointer-events-none"
          >
            ★
          </span>
        </div>
      </template>

      <!-- Hover Popover Content: if analysis exists show analysis, else fallback to mastery -->
      <ChampionAnalysisContent
        v-if="c.analysis"
        :analysis="c.analysis"
        :mastery="c.mastery"
      />
      <template v-else>
        <div class="mb-2 flex items-center gap-2 text-xs">
          <ChampionIcon ring :ring-width="1" round class="w-6 h-6" :champion-id="c.id" />
          <div class="text-xs font-bold text-slate-200">
            {{ player.champion_name && c.id === player.champion_id ? player.champion_name : `英雄 #${c.id}` }}
          </div>
        </div>
        <ChampionMasteryContent v-if="c.mastery" :mastery="c.mastery" />
        <div v-else class="text-xs text-slate-400 py-1">
          暂无该英雄近期对局分析数据
        </div>
      </template>
    </NPopover>

    <div v-if="championList.length === 0" class="text-[10px] text-slate-500">
      暂无近期常用英雄数据
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../types'
import ChampionIcon from '../../../widgets/ChampionIcon.vue'
import ChampionAnalysisContent from '../../../champion-analysis/ChampionAnalysisContent.vue'
import ChampionMasteryContent from '../../../champion-analysis/ChampionMasteryContent.vue'
import { adaptPlayerCardToAggregatedAnalysis } from '../../utils/player-adapter'
import type { ChampionAnalysisMastery } from '../../../champion-analysis/types'
import type { AggregatedChampionAnalysis } from '../../../../shared/data-adapter/analysis/player'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const analysis = computed(() => adaptPlayerCardToAggregatedAnalysis(props.player))

interface ChampionItem {
  id: number
  winRate: number
  games: number
  isStar: boolean
  mastery?: ChampionAnalysisMastery | null
  analysis: AggregatedChampionAnalysis | null
}

const championList = computed<ChampionItem[]>(() => {
  const currentChampId = props.player.champion_id
  const currentWinRate = props.player.champ_recent_win_rate ?? 0
  const currentGames = props.player.champ_recent_games ?? 0

  const champStatsMap = new Map<number, { games: number; wins: number; kda?: number }>()

  // 1. If real matches exist, extract the player's actual played champions
  if (props.player.matches && props.player.matches.length > 0) {
    for (const m of props.player.matches) {
      const cid = m.champion_id
      if (cid > 0) {
        const item = champStatsMap.get(cid) || { games: 0, wins: 0, kda: 0 }
        item.games++
        if (m.win) item.wins++
        item.kda = (item.kda || 0) + (m.kda_ratio || 0)
        champStatsMap.set(cid, item)
      }
    }
  }

  // 2. From top_champs from Rust backend if present
  if (props.player.top_champs && props.player.top_champs.length > 0) {
    for (const tc of props.player.top_champs) {
      if (tc.champion_id > 0 && !champStatsMap.has(tc.champion_id)) {
        champStatsMap.set(tc.champion_id, {
          games: tc.games,
          wins: Math.round((tc.win_rate || (tc.win_rate_pct / 100)) * tc.games),
          kda: tc.kda * tc.games
        })
      }
    }
  }

  // 3. Include current locked/selected champion if known and has games
  if (currentChampId > 0 && currentGames > 0 && !champStatsMap.has(currentChampId)) {
    champStatsMap.set(currentChampId, {
      games: currentGames,
      wins: Math.round((currentWinRate / 100) * currentGames)
    })
  }

  // Sort: current picked champion first, then by games descending
  const sortedChamps = Array.from(champStatsMap.entries())
    .sort((a, b) => {
      if (a[0] === currentChampId) return -1
      if (b[0] === currentChampId) return 1
      return b[1].games - a[1].games
    })
    .slice(0, 4)

  if (sortedChamps.length === 0) {
    if (currentChampId > 0) {
      const champAnalysis = analysis.value.champions[currentChampId] || null
      const realMastery = props.player.masteries?.find((m) => m.championId === currentChampId) || null
      return [
        {
          id: currentChampId,
          winRate: currentWinRate,
          games: currentGames,
          isStar: true,
          mastery: realMastery,
          analysis: champAnalysis
        }
      ]
    }
    return []
  }

  return sortedChamps.map(([id, stats]) => {
    const wr = stats.games > 0 ? Math.round((stats.wins / stats.games) * 100) : 0
    const champAnalysis = analysis.value.champions[id] || null
    const realMastery = props.player.masteries?.find((m) => m.championId === id) || null

    return {
      id,
      winRate: wr,
      games: stats.games,
      isStar: id === currentChampId,
      mastery: realMastery,
      analysis: champAnalysis
    }
  })
})
</script>
