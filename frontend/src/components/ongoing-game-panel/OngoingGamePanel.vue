<template>
  <div ref="containerRef" class="ongoing-game-panel h-full flex flex-col overflow-hidden relative select-none" :class="{ 'ongoing-game-panel--draft': isDraft }">
    <!-- View Switcher & Panel Bar -->
    <div class="h-10 px-4 bg-[#070b14]/90 border-b border-slate-800/80 flex items-center justify-between shrink-0">
      <div class="flex items-center space-x-3 text-xs">
        <div class="flex items-center space-x-1 bg-slate-900/80 p-0.5 rounded-lg border border-slate-800">
          <button
            @click="layoutMode = 'lane-aligned'"
            class="px-2.5 py-1 rounded-md text-xs font-semibold transition"
            :class="layoutMode === 'lane-aligned' ? 'bg-[#0e273c] text-cyan-300 border border-cyan-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200'"
          >
            5v5 分路对位模式
          </button>
          <button
            @click="layoutMode = 'team-split'"
            class="px-2.5 py-1 rounded-md text-xs font-semibold transition"
            :class="layoutMode === 'team-split' ? 'bg-[#0e273c] text-cyan-300 border border-cyan-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200'"
          >
            蓝红分队总览
          </button>
        </div>

        <div class="flex items-center space-x-2 text-slate-400 text-[11px]">
          <span>排序方式:</span>
          <select
            v-model="sortBy"
            class="bg-slate-900 border border-slate-800 text-slate-200 rounded px-1.5 py-0.5 text-xs outline-none focus:border-cyan-500"
          >
            <option value="position">分路位置 (TOP/JUG/MID/BOT/SUP)</option>
            <option value="premade">开黑车队</option>
            <option value="kda">近期 KDA</option>
            <option value="winrate">综合胜率</option>
          </select>
        </div>

        <div v-if="isDraft" class="flex items-center space-x-2 ml-2">
          <span class="px-2 py-0.5 rounded text-[10px] bg-cyan-950/80 border border-cyan-500/40 text-cyan-300 font-bold">
            草稿演练模式
          </span>
          <button
            @click="emits('exitDraft')"
            class="px-2 py-0.5 rounded text-[10px] bg-amber-950/80 hover:bg-amber-900 border border-amber-600/50 text-amber-200 transition"
          >
            退出草稿
          </button>
        </div>
      </div>

      <div class="flex items-center space-x-2">
        <label class="flex items-center space-x-1.5 text-xs text-slate-400 cursor-pointer">
          <input type="checkbox" v-model="streamerMode" class="rounded bg-slate-900 border-slate-800 text-cyan-500" />
          <span>匿名/主播模式</span>
        </label>
      </div>
    </div>

    <!-- Mode 1: 5v5 Strict Horizontal Lane Alignment View -->
    <div v-if="layoutMode === 'lane-aligned'" class="flex-1 flex flex-col overflow-hidden">
      <!-- Teams Summary Header Bar -->
      <div class="px-6 py-2 bg-[#080d19] border-b border-[#142137] grid grid-cols-[1fr_80px_1fr] gap-4 items-center shrink-0">
        <!-- Blue Header -->
        <div class="flex items-center justify-between px-3 py-1.5 bg-[#0b1626] rounded-lg border border-[#1b3456]">
          <div class="flex items-center space-x-2">
            <span class="w-2.5 h-2.5 rounded-full bg-blue-400"></span>
            <span class="text-xs font-bold text-blue-200 uppercase tracking-wider">我方阵营 (蓝方)</span>
          </div>
          <TeamTagsArea :players="enhancedBlueTeam" :premade-groups="bluePremades" />
        </div>

        <!-- Center Matchup Tag -->
        <div class="text-center">
          <span class="text-[10px] font-black text-slate-400 tracking-widest uppercase bg-[#101b2e] px-2 py-0.5 rounded border border-[#1e2e4a]">
            VS 对位
          </span>
        </div>

        <!-- Red Header -->
        <div class="flex items-center justify-between px-3 py-1.5 bg-[#240e14] rounded-lg border border-[#4a1a24]">
          <TeamTagsArea :players="enhancedRedTeam" :premade-groups="redPremades" />
          <div class="flex items-center space-x-2">
            <span class="text-xs font-bold text-red-200 uppercase tracking-wider">敌方阵营 (红方)</span>
            <span class="w-2.5 h-2.5 rounded-full bg-red-400"></span>
          </div>
        </div>
      </div>

      <!-- 5 Paired Rows: TOP, JUG, MID, BOT, SUP -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        <div
          v-for="pair in pairedLaneRows"
          :key="pair.key"
          class="grid grid-cols-[1fr_80px_1fr] gap-4 items-stretch"
        >
          <!-- Blue Player Card -->
          <div v-if="pair.blue" class="flex flex-col">
            <PlayerInfoCard
              :player="pair.blue"
              :selected="selectedPuuid === pair.blue.puuid"
            />
          </div>
          <div v-else class="rounded-xl border border-dashed border-slate-800 bg-slate-900/20 flex items-center justify-center text-xs text-slate-600">
            无蓝方选手
          </div>

          <!-- Center Position Indicator Capsule -->
          <div class="flex flex-col items-center justify-center">
            <div class="px-2.5 py-1.5 rounded-lg bg-slate-900/80 border border-slate-800 flex flex-col items-center justify-center shadow-inner">
              <span class="text-xs font-black text-slate-200 tracking-wider font-mono">
                {{ pair.positionShort }}
              </span>
              <span class="text-[9px] text-slate-500 font-bold mt-0.5">
                {{ pair.positionName }}
              </span>
            </div>
          </div>

          <!-- Red Player Card -->
          <div v-if="pair.red" class="flex flex-col">
            <PlayerInfoCard
              :player="pair.red"
              :selected="selectedPuuid === pair.red.puuid"
            />
          </div>
          <div v-else class="rounded-xl border border-dashed border-slate-800 bg-slate-900/20 flex items-center justify-center text-xs text-slate-600">
            无红方选手
          </div>
        </div>
      </div>
    </div>

    <!-- Mode 2: Team Split Overview (Two Column Grid) -->
    <div v-else class="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
      <OngoingGameTeam
        team-id="blue"
        :players="enhancedBlueTeam"
        :premade-groups="bluePremades"
        :selected-puuid="selectedPuuid"
      />

      <OngoingGameTeam
        team-id="red"
        :players="enhancedRedTeam"
        :premade-groups="redPremades"
        :selected-puuid="selectedPuuid"
      />
    </div>
  </div>
</template>

<script lang="ts">
export {
  calculateColumnsNeed,
  calculateLinesPerTeam
} from './utils/layout'
export {
  getPlayerWinningStreak,
  getPlayerLosingStreak,
  detectWinRateTeamBadge,
  normalizeWinRatePercent,
  normalizeWinRateRatio
} from './utils/team-tags'
</script>

<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import { useElementSize } from '@vueuse/core'
import type { PlayerCard } from '../../api/types'
import type { EnhancedPlayerCard, PremadeGroup } from './types'
import {
  POSITION_ORDER,
  PREMADE_TEAMS
} from './constants'
import { calculateColumnsNeed, calculateLinesPerTeam } from './utils/layout'
import {
  getPlayerWinningStreak,
  getPlayerLosingStreak,
  detectWinRateTeamBadge
} from './utils/team-tags'
import { provideOngoingGamePanel } from './context'
import { findOutliersByIqr } from '../../shared/data-adapter/utils'
import PlayerInfoCard from './widgets/player-info-card/PlayerInfoCard.vue'
import OngoingGameTeam from './widgets/OngoingGameTeam.vue'
import TeamTagsArea from './widgets/TeamTagsArea.vue'

const props = withDefaults(
  defineProps<{
    blueTeam: PlayerCard[]
    redTeam: PlayerCard[]
    selfPuuid?: string
    selectedPuuid?: string
    isDraft?: boolean
    contentWidth?: number
    contentHeight?: number
  }>(),
  {
    selfPuuid: '',
    selectedPuuid: '',
    isDraft: false
  }
)

const emits = defineEmits<{
  selectPlayer: [player: EnhancedPlayerCard]
  inspectPlayer: [puuid: string]
  'preview-game': [payload: { gameId: number; puuid?: string }]
  exitDraft: []
}>()

const containerRef = ref<HTMLElement | null>(null)
const { width: measuredWidth, height: measuredHeight } = useElementSize(containerRef)

const contentWidth = computed(() => props.contentWidth || measuredWidth.value || 1200)
const contentHeight = computed(() => props.contentHeight || measuredHeight.value || 800)

const columnsNeed = computed(() => {
  const maxTeamMembers = Math.max(props.blueTeam.length, props.redTeam.length, 5)
  return calculateColumnsNeed(contentWidth.value, maxTeamMembers)
})

const linesPerTeam = computed(() => {
  const maxMembers = Math.max(props.blueTeam.length, props.redTeam.length, 5)
  return calculateLinesPerTeam(columnsNeed.value, maxMembers)
})

const layoutMode = ref<'lane-aligned' | 'team-split'>('lane-aligned')
const sortBy = ref<'position' | 'premade' | 'kda' | 'winrate'>('position')
const streamerMode = ref(false)

// Calculate IQR outliers across all 10 players
const kdaOutliers = computed<Record<string, 'over' | 'below'>>(() => {
  const all = [...props.blueTeam, ...props.redTeam]
  if (all.length < 4) return {}

  const list = all.map((p) => ({
    puuid: p.puuid,
    kda: p.recent_kda_ratio ?? 0
  }))

  const res = findOutliersByIqr(list, (i) => i.kda, 1.2)
  const map: Record<string, 'over' | 'below'> = {}
  res.over.forEach((item) => {
    map[item.puuid] = 'over'
  })
  res.below.forEach((item) => {
    map[item.puuid] = 'below'
  })
  return map
})

// Build premade parties (A, B, C...)
const premadeTeamIdMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  const all = [...props.blueTeam, ...props.redTeam]

  all.forEach((player) => {
    if (player.premade_party && player.premade_party.party_id > 0) {
      const partyIdx = (player.premade_party.party_id - 1) % PREMADE_TEAMS.length
      map[player.puuid] = PREMADE_TEAMS[partyIdx]
    }
  })

  return map
})

const getPremadeGroupsForTeam = (team: PlayerCard[], prefix = ''): PremadeGroup[] => {
  const partyBuckets: Record<number, string[]> = {}
  team.forEach((p) => {
    if (p.premade_party && p.premade_party.party_id > 0) {
      if (!partyBuckets[p.premade_party.party_id]) {
        partyBuckets[p.premade_party.party_id] = []
      }
      partyBuckets[p.premade_party.party_id].push(p.puuid)
    }
  })

  return Object.entries(partyBuckets).map(([partyIdStr, puuids]) => {
    const partyId = Math.max(1, Number(partyIdStr) || 1)
    const partyIdx = (partyId - 1) % PREMADE_TEAMS.length
    const letter = PREMADE_TEAMS[partyIdx] || 'A'
    const members = team.filter((p) => puuids.includes(p.puuid))
    const winRateTeamInfo = detectWinRateTeamBadge(members)
    const teamLabel = prefix ? `${prefix}-车队 ${letter}` : `车队 ${letter}`

    return {
      id: letter,
      label: teamLabel,
      puuids,
      winRateTeamInfo
    }
  })
}

const bluePremades = computed(() => getPremadeGroupsForTeam(props.blueTeam, '蓝方'))
const redPremades = computed(() => getPremadeGroupsForTeam(props.redTeam, '红方'))

const enhancePlayer = (player: PlayerCard): EnhancedPlayerCard => {
  const premadeId = premadeTeamIdMap.value[player.puuid]
  return {
    ...player,
    premadeTeamId: premadeId
  }
}

const sortPlayers = (players: EnhancedPlayerCard[]): EnhancedPlayerCard[] => {
  const sorted = [...players]
  if (sortBy.value === 'position') {
    return sorted.sort((a, b) => {
      const orderA = POSITION_ORDER[a.assigned_position?.toUpperCase()] ?? 99
      const orderB = POSITION_ORDER[b.assigned_position?.toUpperCase()] ?? 99
      return orderA - orderB
    })
  }
  if (sortBy.value === 'kda') {
    return sorted.sort((a, b) => (b.recent_kda_ratio || 0) - (a.recent_kda_ratio || 0))
  }
  if (sortBy.value === 'winrate') {
    return sorted.sort((a, b) => (b.recent_win_rate || 0) - (a.recent_win_rate || 0))
  }
  if (sortBy.value === 'premade') {
    return sorted.sort((a, b) => (a.premadeTeamId || 'Z').localeCompare(b.premadeTeamId || 'Z'))
  }
  return sorted
}

const enhancedBlueTeam = computed(() => sortPlayers(props.blueTeam.map(enhancePlayer)))
const enhancedRedTeam = computed(() => sortPlayers(props.redTeam.map(enhancePlayer)))

// 5v5 paired lane alignment (TOP, JUG, MID, BOT, SUP)
interface PairedRow {
  key: string
  positionShort: string
  positionName: string
  blue?: EnhancedPlayerCard
  red?: EnhancedPlayerCard
}

const pairedLaneRows = computed<PairedRow[]>(() => {
  const allPlayers = [...enhancedBlueTeam.value, ...enhancedRedTeam.value]
  const canyonLanes = new Set(['TOP', 'JUG', 'JUNGLE', 'MID', 'MIDDLE', 'BOT', 'BOTTOM', 'ADC', 'SUP', 'UTILITY', 'SUPPORT', '1', '2', '3', '4', '5'])
  const isNonCanyonOrNoLanes = allPlayers.length > 0 && !allPlayers.some(p => p.assigned_position && canyonLanes.has(p.assigned_position.toUpperCase()))

  if (sortBy.value !== 'position' || isNonCanyonOrNoLanes) {
    const maxLen = Math.max(enhancedBlueTeam.value.length, enhancedRedTeam.value.length, 5)
    const metricLabels: Record<string, string> = {
      kda: 'KDA 对比',
      winrate: '胜率 对比',
      premade: '车队 梯队'
    }
    const label = isNonCanyonOrNoLanes ? '对位 席位' : (metricLabels[sortBy.value] || '战力 顺位')
    const rows: PairedRow[] = []
    for (let i = 0; i < maxLen; i++) {
      rows.push({
        key: `rank-${i}`,
        positionShort: `#${i + 1}`,
        positionName: label,
        blue: enhancedBlueTeam.value[i],
        red: enhancedRedTeam.value[i]
      })
    }
    return rows
  }

  const laneDefs = [
    { key: 'TOP', short: 'TOP', name: '上路对位', matchKeys: ['TOP', '1'] },
    { key: 'JUG', short: 'JUG', name: '打野对位', matchKeys: ['JUG', 'JUNGLE', '2'] },
    { key: 'MID', short: 'MID', name: '中路对位', matchKeys: ['MID', 'MIDDLE', '3'] },
    { key: 'BOT', short: 'BOT', name: '下路对位', matchKeys: ['BOT', 'BOTTOM', 'ADC', '4'] },
    { key: 'SUP', short: 'SUP', name: '辅助对位', matchKeys: ['SUP', 'UTILITY', 'SUPPORT', '5'] }
  ]

  const usedBlue = new Set<string>()
  const usedRed = new Set<string>()

  const getPlayerKey = (p: EnhancedPlayerCard, idx: number) =>
    p.puuid || p.summoner_id || `${p.game_name || 'summoner'}-${idx}`

  const rows: PairedRow[] = []

  for (const lane of laneDefs) {
    const bluePlayer = enhancedBlueTeam.value.find(
      (p, idx) =>
        !usedBlue.has(getPlayerKey(p, idx)) &&
        Boolean(p.assigned_position) &&
        lane.matchKeys.includes(p.assigned_position.toUpperCase())
    )
    const redPlayer = enhancedRedTeam.value.find(
      (p, idx) =>
        !usedRed.has(getPlayerKey(p, idx)) &&
        Boolean(p.assigned_position) &&
        lane.matchKeys.includes(p.assigned_position.toUpperCase())
    )

    if (bluePlayer) usedBlue.add(getPlayerKey(bluePlayer, enhancedBlueTeam.value.indexOf(bluePlayer)))
    if (redPlayer) usedRed.add(getPlayerKey(redPlayer, enhancedRedTeam.value.indexOf(redPlayer)))

    rows.push({
      key: lane.key,
      positionShort: lane.short,
      positionName: lane.name,
      blue: bluePlayer,
      red: redPlayer
    })
  }

  // Any remaining unassigned players (e.g. ARAM or Arena)
  const remainingBlue = enhancedBlueTeam.value.filter(
    (p, idx) => !usedBlue.has(getPlayerKey(p, idx))
  )
  const remainingRed = enhancedRedTeam.value.filter(
    (p, idx) => !usedRed.has(getPlayerKey(p, idx))
  )

  // Place remaining blue players into rows without blue
  for (const bPlayer of remainingBlue) {
    const targetRow = rows.find((r) => !r.blue)
    if (targetRow) {
      targetRow.blue = bPlayer
    } else {
      rows.push({
        key: `extra-blue-${rows.length}`,
        positionShort: 'FLEX',
        positionName: '自选对位',
        blue: bPlayer
      })
    }
  }

  // Place remaining red players into rows without red
  for (const rPlayer of remainingRed) {
    const targetRow = rows.find((r) => !r.red)
    if (targetRow) {
      targetRow.red = rPlayer
    } else {
      rows.push({
        key: `extra-red-${rows.length}`,
        positionShort: 'FLEX',
        positionName: '自选对位',
        red: rPlayer
      })
    }
  }

  return rows
})

// Provide ongoing game panel context
provideOngoingGamePanel({
  kdaOutliers,
  selfPuuid: toRef(props, 'selfPuuid'),
  streamerMode,
  premadeTeamIdMap,
  columnsNeed,
  linesPerTeam,
  contentWidth,
  contentHeight,
  selectPlayer: (player) => emits('selectPlayer', player),
  inspectPlayer: (puuid) => emits('inspectPlayer', puuid),
  previewGame: (gameId, puuid) => emits('preview-game', { gameId, puuid })
})
</script>

<style scoped>
.ongoing-game-panel {
  position: relative;
  isolation: isolate;
  --draft-grid-bg-color: rgba(6, 182, 212, 0.04);
  --draft-grid-line-color: rgba(6, 182, 212, 0.15);
}

.ongoing-game-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: -1;
  pointer-events: none;
  opacity: 0;
  background-color: var(--draft-grid-bg-color);
  background-image:
    linear-gradient(var(--draft-grid-line-color) 1px, transparent 1px),
    linear-gradient(90deg, var(--draft-grid-line-color) 1px, transparent 1px);
  background-position: center top;
  background-size: 28px 28px;
  transition: opacity 0.2s ease;
}

.ongoing-game-panel--draft::before {
  opacity: 1;
}
</style>
