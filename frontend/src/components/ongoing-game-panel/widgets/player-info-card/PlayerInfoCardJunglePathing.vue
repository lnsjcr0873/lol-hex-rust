<template>
  <div v-if="isJungler" class="mb-1.5 w-full select-none">
    <NPopover :delay="60" :keep-alive-on-hover="true" placement="right-start">
      <template #trigger>
        <div
          class="flex w-full cursor-pointer items-center gap-2 rounded border border-indigo-500/50 bg-indigo-950/30 px-2 py-1.5 transition-all hover:bg-indigo-950/50 hover:border-indigo-400"
        >
          <!-- Mini Heatmap Preview -->
          <div class="shrink-0">
            <GankMap
              :position-points="jungleStats.minutePositions"
              :kill-points="jungleStats.gankPositions"
              :size="40"
              heatmap
              :heatmap-grid-size="7"
              :heatmap-cell-limit="12"
            />
          </div>

          <!-- Gank Bias Info -->
          <div class="flex min-w-0 flex-1 flex-col justify-center gap-0.5 text-[11px]">
            <div class="flex items-center gap-1.5">
              <span class="font-bold" :class="topsideTextColor(jungleStats)">
                {{ topsideTextTrigger(jungleStats) }}
              </span>
              <span class="text-slate-500 text-[10px]">·</span>
              <span class="text-slate-400 text-[10px]">前14m偏好</span>
            </div>

            <!-- Lane weights -->
            <div class="flex min-w-0 text-[10px] text-slate-300 font-mono">
              <span class="text-red-400 mr-1">上 {{ formatWeightSum(jungleStats.topZoneWeightSum) }}</span>
              <span class="text-slate-600 mr-1">|</span>
              <span class="text-amber-400 mr-1">中 {{ formatWeightSum(jungleStats.midZoneWeightSum) }}</span>
              <span class="text-slate-600 mr-1">|</span>
              <span class="text-blue-400">下 {{ formatWeightSum(jungleStats.botZoneWeightSum) }}</span>
            </div>
          </div>
        </div>
      </template>

      <!-- Full Popover Content -->
      <div class="max-w-[500px]">
        <JunglePathingContent :stats="jungleStats" />
      </div>
    </NPopover>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../types'
import { GankMap, JunglePathingContent } from '../../../jungle-pathing-analysis'
import {
  formatWeightSum,
  topsideTextColor,
  topsideTextTrigger
} from '../../../jungle-pathing-analysis/preference'
import { createSynthesizedJungleAnalysis } from '../../utils/player-adapter'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const isJungler = computed(() => {
  const pos = props.player.assigned_position?.toUpperCase()
  if (pos === 'JUNGLE' || pos === 'JUG') return true
  return props.player.spell1_id === 11 || props.player.spell2_id === 11
})

const jungleStats = computed(() => {
  return createSynthesizedJungleAnalysis(props.player)
})
</script>
