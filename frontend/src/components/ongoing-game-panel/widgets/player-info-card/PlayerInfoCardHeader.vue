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
          v-if="currentChampAnalysis"
          :analysis="currentChampAnalysis"
          :mastery="playerMastery"
        />
        <template v-else>
          <div class="mb-2 flex items-center gap-2 text-xs">
            <ChampionIcon ring :ring-width="1" round class="w-6 h-6" :champion-id="player.champion_id" />
            <div class="text-xs font-bold text-slate-200">
              {{ player.champion_name || `英雄 #${player.champion_id}` }}
            </div>
          </div>
          <ChampionMasteryContent v-if="playerMastery" :mastery="playerMastery" />
          <div v-else class="text-xs text-slate-400 py-1">
            暂无该英雄近期对局分析数据
          </div>
        </template>
      </NPopover>

      <!-- Spells (Spell 1 & Spell 2) -->
      <div class="flex flex-col space-y-1 shrink-0">
        <img
          :src="s1Icon"
          :alt="player.spell1_name || '技能1'"
          :title="player.spell1_name || '技能1'"
          class="w-4 h-4 rounded border border-slate-700 object-cover bg-slate-800"
          @error="s1Fallback"
        />
        <img
          :src="s2Icon"
          :alt="player.spell2_name || '技能2'"
          :title="player.spell2_name || '技能2'"
          class="w-4 h-4 rounded border border-slate-700 object-cover bg-slate-800"
          @error="s2Fallback"
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
          <span v-if="!streamerMode && player.tag_line" class="text-[10px] text-slate-400 shrink-0">
            #{{ player.tag_line }}
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
import { computed, ref, watch } from 'vue'
import { NDropdown, NPopover, useMessage } from 'naive-ui'
import type { EnhancedPlayerCard } from '../../types'
import { RANKED_MEDAL_MAP, PREMADE_TEAM_COLORS } from '../../constants'
import ChampionIcon from '../../../widgets/ChampionIcon.vue'
import ChampionAnalysisContent from '../../../champion-analysis/ChampionAnalysisContent.vue'
import ChampionMasteryContent from '../../../champion-analysis/ChampionMasteryContent.vue'
import { adaptPlayerCardToAggregatedAnalysis } from '../../utils/player-adapter'
import { useOngoingGamePanel } from '../../context'
import type { ChampionAnalysisMastery } from '../../../champion-analysis/types'

const props = defineProps<{
  player: EnhancedPlayerCard
}>()

const s1Default = computed(() => props.player.spell1_icon || 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png')
const s2Default = computed(() => {
  if (props.player.spell2_icon) return props.player.spell2_icon
  const pos = props.player.assigned_position?.toUpperCase()
  if (pos === 'JUNGLE' || pos === 'JUG') {
    return 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png'
  }
  return 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png'
})

const s1Icon = ref(s1Default.value)
const s2Icon = ref(s2Default.value)

watch(() => props.player.spell1_icon, () => {
  s1Icon.value = s1Default.value
})
watch(() => props.player.spell2_icon, () => {
  s2Icon.value = s2Default.value
})

function s1Fallback() {
  s1Icon.value = 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png'
}

function s2Fallback() {
  const pos = props.player.assigned_position?.toUpperCase()
  s2Icon.value = (pos === 'JUNGLE' || pos === 'JUG')
    ? 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png'
    : 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png'
}

const message = useMessage()
const { streamerMode, inspectPlayer, selectPlayer } = useOngoingGamePanel()

const analysis = computed(() => adaptPlayerCardToAggregatedAnalysis(props.player))
const currentChampAnalysis = computed(() => analysis.value?.champions?.[props.player.champion_id] ?? null)

const displayName = computed(() => {
  if (streamerMode.value) {
    return props.player.champion_name || '玩家'
  }
  return props.player.game_name || (props.player as any).summoner_name || props.player.champion_name || '玩家'
})

const premadeColor = computed(() => {
  if (props.player.premadeTeamId && PREMADE_TEAM_COLORS[props.player.premadeTeamId]) {
    return PREMADE_TEAM_COLORS[props.player.premadeTeamId].foregroundColor
  }
  return undefined
})

const rankMedal = computed(() => {
  const tierKey = (props.player.tier || 'UNRANKED').toUpperCase()
  if (tierKey === 'UNRANKED' || tierKey === 'NONE' || tierKey === '') {
    return undefined
  }
  return RANKED_MEDAL_MAP[tierKey]
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
  if (!div || div === 'NA' || div === 'NONE' || div.trim() === '') {
    return translated
  }
  // Sanitize div: if div already contains tier name (e.g. "白银 I" or "SILVER I"), remove the tier prefix
  const cleanDiv = div.replace(translated, '').replace(new RegExp(`^${tier}\\s*`, 'i'), '').trim()
  return cleanDiv ? `${translated} ${cleanDiv}` : translated
}

const playerMastery = computed<ChampionAnalysisMastery | null>(() => {
  if (props.player.masteries && props.player.masteries.length > 0) {
    const found = props.player.masteries.find((m) => m.championId === props.player.champion_id)
    if (found) return found
  }
  return null
})

const actionOptions = [
  { label: '查看战绩画像', key: 'view_profile' },
  { label: '展开深度战术抽屉', key: 'open_drawer' },
  { label: '复制召唤师名', key: 'copy_name' },
  { label: '复制 PUUID', key: 'copy_puuid' }
]

const handleActionSelect = async (key: string) => {
  if (key === 'view_profile') {
    inspectPlayer(props.player.puuid)
  } else if (key === 'open_drawer') {
    selectPlayer(props.player)
  } else if (key === 'copy_name') {
    const textToCopy = props.player.tag_line
      ? `${props.player.game_name}#${props.player.tag_line}`
      : (props.player.game_name || '')
    try {
      await navigator.clipboard?.writeText(textToCopy)
      message.success(`已复制召唤师名: ${textToCopy}`)
    } catch {
      message.warning('复制失败，请检查剪贴板权限')
    }
  } else if (key === 'copy_puuid') {
    try {
      await navigator.clipboard?.writeText(props.player.puuid || '')
      message.success('已复制 PUUID')
    } catch {
      message.warning('复制失败，请检查剪贴板权限')
    }
  }
}
</script>
