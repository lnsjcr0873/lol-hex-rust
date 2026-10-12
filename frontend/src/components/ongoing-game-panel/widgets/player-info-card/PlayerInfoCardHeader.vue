<template>
  <div class="mb-2 flex items-start justify-between">
    <div class="flex items-center space-x-2.5 min-w-0 flex-1">
      <!-- Champion Avatar with Hover Popover for ChampionAnalysisContent -->
      <NPopover :delay="60" :keep-alive-on-hover="true" placement="right-start">
        <template #trigger>
          <div class="relative cursor-pointer transition-transform hover:scale-105 shrink-0">
            <ChampionIcon
              :champion-id="player.champion_id"
              class="w-11 h-11 rounded-lg border border-slate-700 shadow"
            />
            <span
              v-if="player.summoner_level"
              class="absolute -bottom-1 -right-1 px-1 py-0.2 bg-slate-900/90 border border-slate-700 text-[9px] text-slate-300 font-mono rounded"
            >
              {{ player.summoner_level }}
            </span>
          </div>
        </template>
        <ChampionAnalysisContent
          :analysis="analysis.champions[player.champion_id]"
          :mastery="playerMastery"
        />
      </NPopover>

      <!-- Spells (Spell 1 & Spell 2) -->
      <div class="flex flex-col space-y-1 shrink-0">
        <img
          :src="player.spell1_icon"
          :alt="player.spell1_name"
          :title="player.spell1_name"
          class="w-4 h-4 rounded border border-slate-700 object-cover"
        />
        <img
          :src="player.spell2_icon"
          :alt="player.spell2_name"
          :title="player.spell2_name"
          class="w-4 h-4 rounded border border-slate-700 object-cover"
        />
      </div>

      <!-- Summoner Name & Rank Tier Badge -->
      <div class="flex flex-col min-w-0 flex-1">
        <div class="flex items-center space-x-1.5 truncate">
          <span
            class="text-xs font-bold text-slate-100 truncate cursor-pointer hover:text-cyan-400 transition-colors"
            :style="{ color: premadeColor }"
            @click="inspectPlayer(player.puuid)"
          >
            {{ displayName }}
          </span>
          <span v-if="!streamerMode" class="text-[10px] text-slate-400 shrink-0">
            #{{ player.tag_line || 'CN' }}
          </span>
        </div>

        <!-- Rank Tier & LP -->
        <div class="flex items-center space-x-1.5 mt-0.5">
          <img
            v-if="rankMedal"
            :src="rankMedal"
            :alt="player.tier"
            class="w-4 h-4 object-contain shrink-0"
          />
          <span class="text-[11px] font-semibold text-amber-400/90">
            {{ formatTier(player.tier, player.division) }}
          </span>
          <span class="text-[10px] text-slate-400 font-mono">
            {{ player.lp ?? 0 }} LP
          </span>
        </div>
      </div>
    </div>

    <!-- Action Menu -->
    <div class="shrink-0 ml-1">
      <NDropdown
        trigger="click"
        size="small"
        placement="bottom-end"
        :options="actionOptions"
        @select="handleActionSelect"
      >
        <button
          class="p-1 rounded text-slate-400 hover:text-slate-100 hover:bg-slate-800 transition"
          title="玩家操作"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="1"/><circle cx="12" cy="5" r="1"/><circle cx="12" cy="19" r="1"/>
          </svg>
        </button>
      </NDropdown>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NDropdown, NPopover } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../types'
import { RANKED_MEDAL_MAP, PREMADE_TEAM_COLORS } from '../../constants'
import ChampionIcon from '../../../widgets/ChampionIcon.vue'
import ChampionAnalysisContent from '../../../champion-analysis/ChampionAnalysisContent.vue'
import { adaptPlayerCardToAggregatedAnalysis } from '../../utils/player-adapter'
import { useOngoingGamePanel } from '../../context'
import type { ChampionAnalysisMastery } from '../../../champion-analysis/types'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const { streamerMode, inspectPlayer, selectPlayer } = useOngoingGamePanel()

const analysis = computed(() => adaptPlayerCardToAggregatedAnalysis(props.player))

const displayName = computed(() => {
  if (streamerMode.value) {
    return props.player.champion_name || '玩家'
  }
  return props.player.game_name
})

const premadeColor = computed(() => {
  if (props.player.premadeTeamId && PREMADE_TEAM_COLORS[props.player.premadeTeamId]) {
    return PREMADE_TEAM_COLORS[props.player.premadeTeamId].foregroundColor
  }
  return undefined
})

const rankMedal = computed(() => {
  const tierKey = (props.player.tier || 'UNRANKED').toUpperCase()
  return RANKED_MEDAL_MAP[tierKey] || RANKED_MEDAL_MAP.GOLD
})

const formatTier = (tier?: string, div?: string) => {
  if (!tier || tier === 'UNRANKED') return '未定级'
  const tierMap: Record<string, string> = {
    IRON: '黑铁',
    BRONZE: '青铜',
    SILVER: '白银',
    GOLD: '黄金',
    PLATINUM: '铂金',
    EMERALD: '翡翠',
    DIAMOND: '钻石',
    MASTER: '大师',
    GRANDMASTER: '宗师',
    CHALLENGER: '王者'
  }
  const translated = tierMap[tier.toUpperCase()] || tier
  return div ? `${translated} ${div}` : translated
}

const playerMastery = computed<ChampionAnalysisMastery>(() => {
  return {
    championId: props.player.champion_id,
    championLevel: 7,
    championPoints: 125600,
    championSeasonMilestone: 3,
    highestGrade: 'S+',
    lastPlayTime: Date.now() - 86400000
  }
})

const actionOptions = [
  { label: '查看战绩画像', key: 'view_profile' },
  { label: '展开深度战术抽屉', key: 'open_drawer' },
  { label: '复制召唤师名', key: 'copy_name' },
  { label: '复制 PUUID', key: 'copy_puuid' }
]

const handleActionSelect = (key: string) => {
  if (key === 'view_profile') {
    inspectPlayer(props.player.puuid)
  } else if (key === 'open_drawer') {
    selectPlayer(props.player)
  } else if (key === 'copy_name') {
    navigator.clipboard?.writeText(props.player.game_name)
  } else if (key === 'copy_puuid') {
    navigator.clipboard?.writeText(props.player.puuid)
  }
}
</script>
