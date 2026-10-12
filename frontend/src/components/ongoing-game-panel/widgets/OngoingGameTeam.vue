<template>
  <div class="flex flex-col gap-2 min-h-0 flex-1">
    <!-- Header with Indicator & TeamTagsArea -->
    <div class="flex items-center justify-between pb-1 border-b border-slate-800/80">
      <div class="flex items-center space-x-2">
        <span
          class="w-2.5 h-2.5 rounded-full"
          :class="teamId === 'blue' ? 'bg-[#40c1ff]' : 'bg-[#ff3333]'"
        />
        <h3 class="text-sm font-bold text-slate-100 tracking-wide">
          {{ teamId === 'blue' ? '我方阵营 (蓝方)' : '敌方阵营 (红方)' }}
        </h3>
        <span class="text-[11px] text-slate-500 font-mono">
          ({{ players.length }} 人)
        </span>
      </div>

      <TeamTagsArea
        v-if="players.length > 0"
        :players="players"
        :premade-groups="premadeGroups"
      />
    </div>

    <!-- Players Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-2.5 min-h-0 flex-1">
      <PlayerInfoCard
        v-for="player of players"
        :key="player.puuid"
        :player="player"
        :selected="selectedPuuid === player.puuid"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { EnhancedPlayerCard, PremadeGroup } from '../types'
import TeamTagsArea from './TeamTagsArea.vue'
import PlayerInfoCard from './player-info-card/PlayerInfoCard.vue'

defineProps<{
  teamId: 'blue' | 'red'
  players: EnhancedPlayerCard[]
  premadeGroups: PremadeGroup[]
  selectedPuuid?: string
}>()
</script>
