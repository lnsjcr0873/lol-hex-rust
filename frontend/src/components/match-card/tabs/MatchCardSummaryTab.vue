<template>
  <div class="space-y-3">
    <!-- Ban List Header if Cherry mode or Global bans -->
    <div
      v-if="basicInfo.isCherrySubteam && teams.allTeamStats.bans.length > 0"
      class="flex items-center justify-end p-2 bg-slate-900/60 rounded-lg border border-slate-800 text-xs"
    >
      <span class="text-slate-400 mr-2">本局全局禁用:</span>
      <div class="flex items-center space-x-1">
        <ChampionIcon
          v-for="ban in teams.allTeamStats.bans"
          :key="ban.championId"
          :champion-id="ban.championId"
          :size="20"
          class="rounded"
        />
      </div>
    </div>

    <!-- Team Tables -->
    <TeamTable
      v-for="team in sortedTeams"
      :key="team.teamIdentifier"
      :team-identifier="team.teamIdentifier"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMatchCard } from '../context'
import TeamTable from '../widgets/TeamTable.vue'
import ChampionIcon from '../widgets/ChampionIcon.vue'

const { teams, basicInfo } = useMatchCard()

const sortedTeams = computed(() => {
  if (basicInfo.value.isCherrySubteam) {
    return teams.value.teamStatsArr.slice().sort((a, b) => a.subteamPlacement - b.subteamPlacement)
  }
  return teams.value.teamStatsArr.slice().sort((a, b) => a.teamIdentifier.localeCompare(b.teamIdentifier))
})
</script>
