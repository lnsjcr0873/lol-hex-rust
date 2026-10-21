<template>
  <div class="flex flex-col gap-2 p-1 min-w-56 select-none">
    <div class="text-[11px] font-bold text-slate-400 mb-0.5 flex items-center justify-between">
      <span>车队成员画像</span>
      <span class="text-[10px] text-slate-500 font-mono">{{ players.length }} 人</span>
    </div>
    <div
      v-for="p of players"
      :key="p.puuid"
      class="flex items-center justify-between gap-2 cursor-pointer transition-colors hover:bg-slate-800/50 p-1 rounded"
      @click="inspectPlayer(p.puuid)"
    >
      <div class="flex items-center gap-2 min-w-0">
        <ChampionIcon class="w-6 h-6 rounded-full shrink-0" round :champion-id="p.champion_id" />
        <div class="min-w-0 flex items-baseline gap-1">
          <span class="text-xs font-bold text-slate-100 truncate hover:text-cyan-400 max-w-[110px]">
            {{ p.game_name }}
          </span>
          <span v-if="p.tag_line" class="text-[10px] text-slate-400">
            #{{ p.tag_line }}
          </span>
        </div>
      </div>
      <div class="text-[11px] font-mono text-cyan-300 shrink-0">
        {{ p.recent_win_rate }}% 胜率 ({{ p.recent_games_count }}场)
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ChampionIcon from '../../widgets/ChampionIcon.vue'
import type { EnhancedPlayerCard } from '../types'
import { useOngoingGamePanel } from '../context'

defineProps<{
  players: EnhancedPlayerCard[]
}>()

const { inspectPlayer } = useOngoingGamePanel()
</script>
