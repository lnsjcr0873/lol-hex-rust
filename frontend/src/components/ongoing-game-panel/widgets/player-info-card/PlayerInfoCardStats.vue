<template>
  <div class="mb-1.5 flex items-center justify-between text-xs select-none bg-slate-950/40 p-1.5 rounded border border-slate-800/80">
    <!-- Winrate with popover -->
    <NPopover :delay="50" :keep-alive-on-hover="true" placement="top">
      <template #trigger>
        <div class="flex items-center space-x-1 cursor-pointer">
          <span
            class="font-extrabold tabular-nums font-mono text-[13px]"
            :class="winRateClass"
          >
            {{ player.recent_win_rate }}%
          </span>
          <span class="text-[10px] text-slate-400 font-mono">
            ({{ player.recent_games_count || 20 }}场)
          </span>
        </div>
      </template>
      <div class="text-xs text-slate-200 p-1">
        <div class="font-bold text-slate-100 mb-1">近期排位胜率</div>
        <div class="text-[11px] text-slate-300 font-mono">
          胜场: {{ Math.round((player.recent_win_rate / 100) * (player.recent_games_count || 20)) }} ·
          败场: {{ (player.recent_games_count || 20) - Math.round((player.recent_win_rate / 100) * (player.recent_games_count || 20)) }}
        </div>
      </div>
    </NPopover>

    <!-- Recent KDA with IQR Outlier Highlight and AkariScorePopover -->
    <AkariScorePopover :score="analysis.akariScore">
      <div class="flex items-center space-x-1 cursor-pointer" :title="'Akari 评分: ' + analysis.akariScore.total.toFixed(2)">
        <span class="text-[10px] text-slate-400 font-medium">KDA</span>
        <span
          class="font-extrabold tabular-nums font-mono text-[13px] flex items-center"
          :class="kdaHighlightClass"
        >
          <span>{{ player.recent_kda_ratio ? player.recent_kda_ratio.toFixed(2) : player.recent_kda }}</span>
          <span v-if="iqrStatus === 'over'" class="ml-0.5 text-amber-400 text-[10px]">▲</span>
          <span v-else-if="iqrStatus === 'below'" class="ml-0.5 text-rose-500 text-[10px]">▼</span>
        </span>
      </div>
    </AkariScorePopover>

    <!-- Position Assignment Indicator (Primary / Fill) -->
    <div class="shrink-0">
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

const winRateClass = computed(() => {
  const rate = props.player.recent_win_rate || 50
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
  return POSITION_DISPLAY_NAMES[pos] || pos || '首选'
})

const positionRoleBadgeClass = computed(() => {
  return 'bg-slate-800 text-slate-300 border border-slate-700/60'
})
</script>
