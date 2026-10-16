<template>
  <div class="mb-1.5 flex items-center justify-between text-xs select-none bg-slate-950/40 p-1.5 rounded border border-slate-800/80">
    <!-- Winrate with popover -->
    <NPopover :delay="50" :keep-alive-on-hover="true" placement="top" :disabled="!hasGames">
      <template #trigger>
        <div v-if="hasGames" class="flex items-center space-x-1 cursor-pointer">
          <span
            class="font-extrabold tabular-nums font-mono text-[13px]"
            :class="winRateClass"
          >
            {{ player.recent_win_rate }}%
          </span>
          <span class="text-[10px] text-slate-400 font-mono">
            ({{ player.recent_games_count }}场)
          </span>
        </div>
        <div v-else class="flex items-center justify-center">
          <span class="text-[13px] font-medium text-slate-500 font-mono">- %</span>
        </div>
      </template>
      <div v-if="hasGames" class="text-xs text-slate-200 p-1">
        <div class="font-bold text-slate-100 mb-1">近期排位胜率</div>
        <div class="text-[11px] text-slate-300 font-mono">
          胜场: {{ Math.round(((player.recent_win_rate ?? 0) / 100) * (player.recent_games_count ?? 0)) }} ·
          败场: {{ (player.recent_games_count ?? 0) - Math.round(((player.recent_win_rate ?? 0) / 100) * (player.recent_games_count ?? 0)) }}
        </div>
      </div>
    </NPopover>

    <!-- Recent KDA with IQR Outlier Highlight and AkariScorePopover -->
    <AkariScorePopover v-if="hasGames" :score="analysis.akariScore">
      <div class="flex items-center space-x-1 cursor-pointer" :title="'Akari 评分: ' + (Number.isFinite(analysis?.akariScore?.total) ? analysis.akariScore.total.toFixed(2) : '—')">
        <span class="text-[10px] text-slate-400 font-medium">KDA</span>
        <span
          class="font-extrabold tabular-nums font-mono text-[13px] flex items-center"
          :class="kdaHighlightClass"
        >
          <span>{{ formattedKda }}</span>
          <span v-if="iqrStatus === 'over'" class="ml-0.5 text-amber-400 text-[10px]">▲</span>
          <span v-else-if="iqrStatus === 'below'" class="ml-0.5 text-rose-500 text-[10px]">▼</span>
        </span>
      </div>
    </AkariScorePopover>
    <div v-else class="flex items-center justify-center">
      <span class="text-[13px] font-medium text-slate-500 font-mono">N/A</span>
    </div>

    <!-- Position Assignment Indicator -->
    <div v-if="positionRoleLabel" class="shrink-0">
      <span
        class="px-1.5 py-0.2 rounded text-[10px] font-medium"
        :class="positionRoleBadgeClass"
      >
        {{ positionRoleLabel }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../types'
import { AkariScorePopover } from '../../../akari-score'
import { adaptPlayerCardToAggregatedAnalysis } from '../../utils/player-adapter'
import { useOngoingGamePanel } from '../../context'
import { POSITION_DISPLAY_NAMES } from '../../constants'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const { kdaOutliers } = useOngoingGamePanel()

const analysis = computed(() => adaptPlayerCardToAggregatedAnalysis(props.player))

const hasGames = computed(() => {
  return Boolean(props.player.recent_games_count && props.player.recent_games_count > 0)
})

const formattedKda = computed(() => {
  if (analysis.value?.summary?.avgKda !== undefined && !isNaN(analysis.value.summary.avgKda) && analysis.value.count > 0) {
    return analysis.value.summary.avgKda.toFixed(2)
  }
  if (typeof props.player.recent_kda_ratio === 'number' && !isNaN(props.player.recent_kda_ratio) && (props.player.recent_games_count ?? 0) > 0) {
    return props.player.recent_kda_ratio.toFixed(2)
  }
  return '0.00'
})

const winRateClass = computed(() => {
  const rate = props.player.recent_win_rate ?? 0
  if (rate >= 55) return 'text-emerald-400'
  if (rate >= 48) return 'text-slate-200'
  return 'text-rose-400'
})

const iqrStatus = computed(() => {
  return kdaOutliers.value[props.player.puuid]
})

const kdaHighlightClass = computed(() => {
  if (iqrStatus.value === 'over') {
    return 'text-amber-400 font-black'
  }
  if (iqrStatus.value === 'below') {
    return 'text-rose-400'
  }
  return 'text-slate-100'
})

const positionRoleLabel = computed(() => {
  const pos = props.player.assigned_position?.toUpperCase()
  if (!pos || pos === 'NONE' || pos === 'UNKNOWN') return ''
  return POSITION_DISPLAY_NAMES[pos] || pos
})

const positionRoleBadgeClass = computed(() => {
  return 'bg-slate-800 text-slate-300 border border-slate-700/60'
})
</script>
