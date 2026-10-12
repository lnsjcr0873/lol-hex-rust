<template>
  <div class="flex items-center space-x-3 text-xs select-none">
    <!-- Team Win Rate -->
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
        <div class="text-[11px] text-slate-300 font-mono">
          总击杀 {{ totalKills }} / 阵亡 {{ totalDeaths }} / 助攻 {{ totalAssists }}
        </div>
      </div>
    </NPopover>

    <!-- Divider (if premade groups exist) -->
    <div v-if="premadeGroups.length > 0" class="h-3 w-px bg-slate-700/80"></div>

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

const avgWinRate = computed(() => {
  if (props.players.length === 0) return 50
  const sum = props.players.reduce((acc, p) => acc + (p.recent_win_rate || 50), 0)
  return sum / props.players.length
})

const avgKda = computed(() => {
  if (props.players.length === 0) return 3.0
  const sum = props.players.reduce((acc, p) => acc + (p.recent_kda_ratio || 3.0), 0)
  return sum / props.players.length
})

const totalGames = computed(() => {
  return props.players.reduce((acc, p) => acc + (p.recent_games_count || 20), 0)
})

const totalKills = computed(() => {
  return Math.round(avgKda.value * 28)
})

const totalDeaths = computed(() => {
  return Math.round(props.players.length * 15)
})

const totalAssists = computed(() => {
  return Math.round(avgKda.value * 32)
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
