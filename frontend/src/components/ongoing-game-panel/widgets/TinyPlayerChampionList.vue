<template>
  <div class="flex flex-col gap-2 p-1 min-w-44 select-none">
    <div class="text-[11px] font-bold text-slate-400 mb-0.5">
      车队成员列表
    </div>
    <div
      v-for="p of players"
      :key="p.puuid"
      class="flex items-center gap-2 cursor-pointer transition-colors hover:bg-slate-800/50 p-1 rounded"
      @click="inspectPlayer(p.puuid)"
    >
      <ChampionIcon class="w-6 h-6 rounded-full shrink-0" round :champion-id="p.champion_id" />
      <div class="min-w-0 flex-1 flex items-baseline gap-1">
        <span class="text-xs font-bold text-slate-100 truncate hover:text-cyan-400">
          {{ p.game_name }}
        </span>
        <span v-if="p.tag_line" class="text-[10px] text-slate-400">
          #{{ p.tag_line }}
        </span>
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
