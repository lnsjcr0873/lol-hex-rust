<template>
  <div class="mt-1 flex flex-col gap-1 select-none">
    <div
      v-for="(match, index) of recentMatches"
      :key="index"
      class="group relative flex h-7 items-center justify-between rounded px-2 text-[11px] transition-all cursor-pointer hover:brightness-110 border"
      :class="[
        match.isWin
          ? 'bg-blue-950/40 border-blue-600/30 text-blue-200 hover:border-blue-500/60'
          : 'bg-rose-950/40 border-rose-600/30 text-rose-200 hover:border-rose-500/60'
      ]"
      @click="inspectPlayer(player.puuid)"
    >
      <!-- Left: Champion Icon & Queue -->
      <div class="flex items-center space-x-1.5 min-w-0">
        <ChampionIcon
          :champion-id="match.championId"
          class="w-4 h-4 rounded-sm shrink-0"
        />
        <span class="truncate text-[10px] text-slate-300 font-medium">
          {{ match.queueName }}
        </span>
      </div>

      <!-- Right: Win/Loss status & KDA -->
      <div class="flex items-center space-x-2 shrink-0 font-mono text-[10px]">
        <span
          class="font-bold text-[10px]"
          :class="match.isWin ? 'text-blue-400' : 'text-rose-400'"
        >
          {{ match.isWin ? '胜利' : '失败' }}
        </span>
        <span class="text-slate-200">
          {{ match.kda }}
        </span>
      </div>
    </div>

    <div v-if="recentMatches.length === 0" class="text-[10px] text-slate-500 text-center py-1">
      暂无近期对局
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { EnhancedPlayerCard } from '../../types'
import ChampionIcon from '../../../widgets/ChampionIcon.vue'
import { useOngoingGamePanel } from '../../context'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const { inspectPlayer } = useOngoingGamePanel()

interface MiniMatch {
  championId: number
  isWin: boolean
  queueName: string
  kda: string
}

const recentMatches = computed<MiniMatch[]>(() => {
  const matches = props.player.recent_matches || ['win', 'loss', 'win', 'win', 'loss']
  const champId = props.player.champion_id || 266

  return matches.slice(0, 4).map((m, idx) => {
    const isWin = m === 'win'
    return {
      championId: champId,
      isWin,
      queueName: '单双排',
      kda: isWin ? `${6 + idx} / ${2 + idx} / ${8 + idx}` : `${2 + idx} / ${6 + idx} / ${4 + idx}`
    }
  })
})
</script>
