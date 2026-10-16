<template>
  <div class="flex items-center space-x-3 text-xs select-none">
    <!-- Team Win Rate & KDA (Shown when valid stats exist) -->
    <template v-if="hasStats">
      <NPopover :delay="50" :keep-alive-on-hover="true" placement="top">
        <template #trigger>
          <div class="flex items-center space-x-1 cursor-pointer">
            <span class="text-slate-400 text-[11px]">胜率:</span>
            <span
              class="font-extrabold tabular-nums font-mono text-sm"
              :class="avgWinRate >= 50 ? WIN_RATE_GTE_50_TEXT_CLASSES : WIN_RATE_LT_50_TEXT_CLASSES"
            >
              {{ avgWinRate.toFixed(0) }}%
            </span>
          </div>
        </template>
        <div class="text-xs text-slate-200 p-1">
          <div class="font-bold text-slate-100 mb-0.5">队伍平均胜率</div>
          <div class="text-[11px] text-slate-300 font-mono">
            {{ avgWinRate.toFixed(1) }}% ({{ Math.round((avgWinRate / 100) * totalGames) }} 胜 / {{ totalGames }} 场)
          </div>
        </div>
      </NPopover>

      <!-- Divider -->
      <div class="h-3 w-px bg-slate-700/80"></div>

      <!-- Team Average KDA -->
      <NPopover :delay="50" :keep-alive-on-hover="true" placement="top">
        <template #trigger>
          <div class="flex items-center space-x-1 cursor-pointer">
            <span class="text-slate-400 text-[11px]">KDA:</span>
            <span class="font-extrabold text-slate-100 tabular-nums font-mono text-sm">
              {{ avgKda.toFixed(2) }}
            </span>
          </div>
        </template>
        <div class="text-xs text-slate-200 p-1">
          <div class="font-bold text-slate-100 mb-0.5">队伍场均 KDA 比值</div>
          <div v-if="teamCombatStats" class="text-[11px] text-slate-300 font-mono">
            场均击杀 {{ teamCombatStats.kills }} / 场均阵亡 {{ teamCombatStats.deaths }} / 场均助攻 {{ teamCombatStats.assists }}
          </div>
          <div v-else class="text-[11px] text-slate-400">
            暂无场均战斗明细数据
          </div>
        </div>
      </NPopover>
    </template>

    <!-- Divider (if both stats and premade groups exist) -->
    <div v-if="hasStats && premadeGroups.length > 0" class="h-3 w-px bg-slate-700/80"></div>

    <!-- Premade Groups Badges -->
    <div v-if="premadeGroups.length > 0" class="flex items-center space-x-1.5">
      <NPopover
        v-for="group of premadeGroups"
        :key="group.id"
        :delay="50"
        :keep-alive-on-hover="true"
        placement="top"
      >
        <template #trigger>
          <div class="flex items-center overflow-hidden rounded cursor-pointer transition-transform hover:scale-105 shadow-sm">
            <!-- Group Letter Tag -->
            <div
              class="px-1.5 py-0.5 text-[10px] font-bold"
              :style="{
                backgroundColor: getPremadeTheme(group.id).foregroundColor,
                color: getPremadeTheme(group.id).color
              }"
            >
              车队 {{ group.id }} ({{ group.puuids.length }}排)
            </div>

            <!-- Win Rate Team / Loss Rate Team Badges -->
            <div
              v-if="group.winRateTeamInfo?.type === 'win-rate-team'"
              class="px-1.5 py-0.5 text-[10px] font-bold text-amber-200 bg-[#7e2c85]"
            >
              👑 胜率队
            </div>
            <div
              v-else-if="group.winRateTeamInfo?.type === 'loss-rate-team'"
              class="px-1.5 py-0.5 text-[10px] font-bold text-slate-200 bg-[#893b3b]"
            >
              ❄️ 败率队
            </div>
          </div>
        </template>

        <!-- Popover showing member list -->
        <TinyPlayerChampionList :players="getGroupPlayers(group.puuids)" />
      </NPopover>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import type { EnhancedPlayerCard, PremadeGroup } from '../types'
import { PREMADE_TEAM_COLORS } from '../constants'
import {
  WIN_RATE_GTE_50_TEXT_CLASSES,
  WIN_RATE_LT_50_TEXT_CLASSES
} from '../utils/theme'
import TinyPlayerChampionList from './TinyPlayerChampionList.vue'

const props = defineProps<{
  players: EnhancedPlayerCard[]
  premadeGroups: PremadeGroup[]
}>()

const hasStats = computed(() => {
  return props.players.some((p) => (p.recent_games_count ?? 0) > 0)
})

const avgWinRate = computed(() => {
  const valid = props.players.filter((p) => (p.recent_games_count ?? 0) > 0)
  if (valid.length === 0) return 0
  const sum = valid.reduce((acc, p) => acc + (p.recent_win_rate ?? 0), 0)
  return sum / valid.length
})

const avgKda = computed(() => {
  const valid = props.players.filter((p) => (p.recent_games_count ?? 0) > 0)
  if (valid.length === 0) return 0
  const sum = valid.reduce((acc, p) => acc + (p.recent_kda_ratio ?? 0), 0)
  return sum / valid.length
})

const totalGames = computed(() => {
  return props.players.reduce((acc, p) => acc + (p.recent_games_count ?? 0), 0)
})

const teamCombatStats = computed(() => {
  let killsSum = 0
  let deathsSum = 0
  let assistsSum = 0
  let validCount = 0

  for (const p of props.players) {
    if (p.matches && p.matches.length > 0) {
      let pK = 0, pD = 0, pA = 0
      for (const m of p.matches) {
        pK += m.kills || 0
        pD += m.deaths || 0
        pA += m.assists || 0
      }
      killsSum += pK / p.matches.length
      deathsSum += pD / p.matches.length
      assistsSum += pA / p.matches.length
      validCount++
    } else if (p.recent_kda && typeof p.recent_kda === 'string') {
      const parts = p.recent_kda.split('/').map((s) => parseFloat(s.trim()))
      if (parts.length === 3 && !parts.some(isNaN)) {
        killsSum += parts[0]
        deathsSum += parts[1]
        assistsSum += parts[2]
        validCount++
      }
    }
  }

  if (validCount > 0) {
    return {
      kills: killsSum.toFixed(1),
      deaths: deathsSum.toFixed(1),
      assists: assistsSum.toFixed(1)
    }
  }

  return null
})

const getPremadeTheme = (groupId: string) => {
  return PREMADE_TEAM_COLORS[groupId] || {
    foregroundColor: '#48e5db',
    color: '#000',
    borderColor: '#48e5dbd0'
  }
}

const getGroupPlayers = (puuids: string[]): EnhancedPlayerCard[] => {
  return props.players.filter((p) => puuids.includes(p.puuid))
}
</script>
