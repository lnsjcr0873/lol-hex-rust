<template>
  <div class="flex flex-col gap-3 w-full">
    <!-- Player Custom Tags & Privacy Warnings -->
    <NormalTagBlock :privacy="profile?.privacy" :tags="tags" />

    <!-- 20-Game Summary & Akari Score Pane -->
    <SummaryPane :analysis="analysis" :profile-summary="profile?.recent_summary" />

    <!-- Summoner Challenges & Assets -->
    <PlayerChallenges :challenges-data="challengesData" />

    <!-- Champion Masteries Pane -->
    <ChampionMasteryPane :masteries-list="masteriesList" />

    <!-- Teammates Encountered -->
    <RecentlyPlayers
      side="ally"
      :matches="matches"
      :self-puuid="selfPuuid"
      @select-summoner="$emit('selectSummoner', $event)"
    />

    <!-- Opponents Encountered -->
    <RecentlyPlayers
      side="enemy"
      :matches="matches"
      :self-puuid="selfPuuid"
      @select-summoner="$emit('selectSummoner', $event)"
    />

    <!-- Past Encountered Games -->
    <EncounteredGames
      :encountered-list="encounteredGames"
      @select-game="$emit('selectGame', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import NormalTagBlock from './NormalTagBlock.vue'
import SummaryPane from './SummaryPane.vue'
import ChampionMasteryPane from './ChampionMasteryPane.vue'
import RecentlyPlayers from './RecentlyPlayers.vue'
import EncounteredGames from './EncounteredGames.vue'
import PlayerChallenges from './widgets/PlayerChallenges.vue'

defineProps<{
  profile?: any
  analysis?: any
  challengesData?: any
  masteriesList?: any[]
  matches?: any[]
  encounteredGames?: any[]
  tags?: any[]
  selfPuuid?: string
}>()

defineEmits<{
  selectSummoner: [puuid: string]
  selectGame: [gameId: number]
}>()
</script>
