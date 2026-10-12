<template>
  <div
    class="relative box-border flex flex-col overflow-hidden rounded-xl border p-2.5 transition-all select-none cursor-pointer group bg-[#0a101d]/90 hover:bg-[#0e1626] backdrop-blur"
    :class="[
      isSelected ? 'ring-2 ring-cyan-400 border-cyan-400' : 'border-slate-800 hover:border-slate-700'
    ]"
    :style="{
      width: '100%',
      minWidth: '220px',
      borderColor: premadeBorderColor
    }"
    @click="handleClick"
  >
    <!-- Premade Corner Cut Decoration -->
    <div
      v-if="premadeColor"
      class="absolute top-0 right-0 z-10 w-4 h-4 translate-x-1/2 -translate-y-1/2 rotate-45 shadow-sm"
      :style="{ backgroundColor: premadeColor }"
      :title="`车队 [${player.premadeTeamId}] 成员`"
    />

    <!-- Header Section -->
    <PlayerInfoCardHeader :player="player" />

    <!-- Core Stats (Winrate, KDA with IQR, Position) -->
    <PlayerInfoCardStats :player="player" />

    <!-- Jungle Pathing Indicator (When Jungle) -->
    <PlayerInfoCardJunglePathing :player="player" />

    <!-- Tactical Tags (21 evaluated tags) -->
    <TagsArea :player="player" />

    <!-- Champion Usage Mini Bar -->
    <PlayerInfoCardChampionUsage :player="player" />

    <!-- Mini Match History Strip -->
    <PlayerInfoCardMatchHistory :player="player" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { EnhancedPlayerCard } from '../../types'
import { PREMADE_TEAM_COLORS } from '../../constants'
import PlayerInfoCardHeader from './PlayerInfoCardHeader.vue'
import PlayerInfoCardStats from './PlayerInfoCardStats.vue'
import PlayerInfoCardJunglePathing from './PlayerInfoCardJunglePathing.vue'
import TagsArea from './player-card-tags/TagsArea.vue'
import PlayerInfoCardChampionUsage from './PlayerInfoCardChampionUsage.vue'
import PlayerInfoCardMatchHistory from './PlayerInfoCardMatchHistory.vue'
import { useOngoingGamePanel } from '../../context'

const props = defineProps<{
  player: EnhancedPlayerCard
  selected?: boolean
}>()

const { selectPlayer } = useOngoingGamePanel()

const isSelected = computed(() => Boolean(props.selected))

const premadeColor = computed(() => {
  if (props.player.premadeTeamId && PREMADE_TEAM_COLORS[props.player.premadeTeamId]) {
    return PREMADE_TEAM_COLORS[props.player.premadeTeamId].foregroundColor
  }
  return undefined
})

const premadeBorderColor = computed(() => {
  if (props.player.premadeTeamId && PREMADE_TEAM_COLORS[props.player.premadeTeamId]) {
    return PREMADE_TEAM_COLORS[props.player.premadeTeamId].borderColor
  }
  return undefined
})

const handleClick = () => {
  selectPlayer(props.player)
}
</script>
