<template>
  <JunglePathingSection>
    <template #map>
      <div class="relative overflow-hidden rounded shadow-sm" :style="{ width: `${MAP_SIZE}px`, height: `${MAP_SIZE}px` }">
        <img class="absolute h-full w-full object-cover" :src="map11" alt="Summoners Rift" />
        <svg class="absolute h-full w-full pointer-events-none" viewBox="0 0 100 100">
          <polygon points="0,0 100,100 0,100" fill="rgba(60,140,255,0.08)" />
          <polygon points="0,0 100,100 100,0" fill="rgba(255,60,60,0.08)" />
          <line
            x1="0"
            y1="0"
            x2="100"
            y2="100"
            stroke="rgba(255,255,255,0.4)"
            stroke-width="0.8"
            stroke-dasharray="4,3"
          />
        </svg>

        <!-- Camp Start Markers -->
        <NPopover
          v-for="(pt, i) of firstClearCampMapPoints"
          :key="`camp-${i}`"
          :delay="50"
          :keep-alive-on-hover="false"
          :show-arrow="false"
        >
          <template #trigger>
            <div
              class="absolute z-10 -translate-x-1/2 -translate-y-1/2 rounded-full cursor-pointer transition-transform hover:scale-125"
              :style="{
                left: `${pt.left}px`,
                top: `${pt.top}px`,
                width: `${pt.size}px`,
                height: `${pt.size}px`,
                backgroundColor: campMarkerColors[pt.camp],
                border: pt.hasInvade ? '1.5px solid rgba(251,191,36,0.9)' : '1px solid rgba(255,255,255,0.7)'
              }"
            />
          </template>

          <div class="min-w-36 text-[11px] text-slate-200 p-1">
            <div class="mb-1 flex items-center gap-1.5 font-bold text-slate-100">
              <span
                class="w-2.5 h-2.5 rounded-full"
                :style="{ backgroundColor: campMarkerColors[pt.camp] }"
              />
              <span>{{ pt.campLabel }}</span>
            </div>
            <div class="my-1 border-b border-slate-700/60"></div>
            <div class="flex flex-col gap-1.5">
              <div v-for="entry of pt.entries" :key="`${entry.side}-${entry.kind}`">
                <div class="flex items-center gap-1.5 text-xs">
                  <span class="w-2 h-2 rounded-full" :class="entry.indicatorClass" />
                  <span class="font-semibold text-slate-200">{{ entry.sideLabel }}</span>
                  <span class="text-slate-400 text-[10px]">({{ entry.kindLabel }})</span>
                </div>
                <div class="pl-3.5 text-[10px] text-slate-300">
                  出场 {{ entry.count }} 场 / 占比 {{ Math.round((entry.count / (entry.sideTotal || 1)) * 100) }}%
                </div>
              </div>
            </div>
          </div>
        </NPopover>

        <!-- Early Gank (Lv3 / Lv4) Positions -->
        <div
          v-for="(pt, i) of earlyGankMapPoints"
          :key="`eg-${i}`"
          class="pointer-events-none absolute z-20 h-3 w-3 -translate-x-1/2 -translate-y-1/2"
          :style="{ left: `${pt.left}px`, top: `${pt.top}px` }"
        >
          <span
            class="absolute top-1/2 left-1/2 h-0.5 w-full -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm"
            :class="pt.level === 3 ? 'bg-orange-500 shadow-sm' : 'bg-purple-500 shadow-sm'"
          />
          <span
            class="absolute top-1/2 left-1/2 h-0.5 w-full -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm"
            :class="pt.level === 3 ? 'bg-orange-500 shadow-sm' : 'bg-purple-500 shadow-sm'"
          />
        </div>
      </div>

      <!-- Legend -->
      <div class="flex flex-col gap-1 pl-0.5 text-[10px] text-slate-400">
        <span class="inline-flex items-center gap-1 whitespace-nowrap">
          <span class="inline-block h-2 w-2 rounded-full border border-slate-400 bg-slate-300" />
          <span>己方野区起手</span>
        </span>
        <span class="inline-flex items-center gap-1 whitespace-nowrap text-amber-400">
          <span class="inline-block h-2 w-2 rounded-full border border-amber-400 bg-amber-500/30" />
          <span>入侵敌方开野</span>
        </span>
        <span class="inline-flex items-center gap-1 whitespace-nowrap text-orange-400">
          <span class="relative h-2 w-2 inline-block">
            <span class="absolute top-1/2 left-1/2 h-0.5 w-2.5 -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm bg-orange-500" />
            <span class="absolute top-1/2 left-1/2 h-0.5 w-2.5 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm bg-orange-500" />
          </span>
          <span>3级首抓落点</span>
        </span>
        <span class="inline-flex items-center gap-1 whitespace-nowrap text-purple-400">
          <span class="relative h-2 w-2 inline-block">
            <span class="absolute top-1/2 left-1/2 h-0.5 w-2.5 -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm bg-purple-500" />
            <span class="absolute top-1/2 left-1/2 h-0.5 w-2.5 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm bg-purple-500" />
          </span>
          <span>4级首抓落点</span>
        </span>
      </div>
    </template>

    <template #content>
      <!-- First Clear Section -->
      <div>
        <div class="mb-1 font-bold text-slate-100">首轮刷野路线</div>
        <div class="flex flex-col gap-1.5 text-slate-300 text-[11px]">
          <div v-for="row of firstClearRows" :key="`first-clear-${row.side}`" class="grid grid-cols-[52px_1fr] items-start gap-x-2">
            <span class="inline-flex items-center gap-1 self-start font-bold text-slate-200">
              <span :class="['w-2 h-2 rounded-full', row.indicatorClass]" />
              <span>{{ row.label }}</span>
            </span>
            <div class="flex flex-col gap-0.5">
              <div class="leading-4 flex items-center gap-1">
                <span class="w-1.5 h-1.5 rotate-45 bg-slate-400 inline-block" />
                <span :class="row.own.textClass">{{ row.own.text }}</span>
              </div>
              <div class="leading-4 flex items-center gap-1">
                <span class="w-1.5 h-1.5 rotate-45 bg-amber-400 inline-block" />
                <span :class="row.invade.textClass">{{ row.invade.text }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Early Gank Section -->
      <div>
        <div class="mb-1 font-bold text-slate-100">早期抓人倾向 (前 4 分钟)</div>
        <div class="flex flex-col gap-1.5 text-slate-300 text-[11px]">
          <div v-for="row of earlyGankRows" :key="`early-gank-${row.side}`" class="grid grid-cols-[52px_1fr] items-start gap-x-2">
            <span class="inline-flex items-center gap-1 self-start font-bold text-slate-200">
              <span :class="['w-2 h-2 rounded-full', row.indicatorClass]" />
              <span>{{ row.label }}</span>
            </span>
            <div class="flex flex-col gap-0.5">
              <span :class="row.level3.textClass">3级抓人: {{ row.level3.text }}</span>
              <span :class="row.level4.textClass">4级抓人: {{ row.level4.text }}</span>
            </div>
          </div>
        </div>
      </div>
    </template>
  </JunglePathingSection>
</template>

<script setup lang="ts">
import map11 from '../../assets/maps/11.png'
import { mapToImagePosition } from '../../shared/utils/game-map'
import type { JungleCamp } from '../../shared/data-adapter/analysis/player'
import { BLUE_SIDE_CAMPS, RED_SIDE_CAMPS } from '../../shared/data-adapter/analysis/player/constants'
import { NPopover } from 'naive-ui'
import { computed } from 'vue'
import JunglePathingSection from './JunglePathingSection.vue'
import type {
  FirstClearAndGankSummaryProps,
  FirstClearCampMapPoint,
  FirstClearCampMapPointEntry,
  FirstClearRow,
  EarlyGankRow,
  TeamSide
} from './types'

const MAP_SIZE = 140
const CAMP_MARKER_SIZE = 9

const { stats } = defineProps<FirstClearAndGankSummaryProps>()

const TEAM_SIDES: readonly TeamSide[] = ['blue', 'red']
const CAMP_KEYS: readonly JungleCamp[] = ['red', 'blue', 'wolves', 'raptors']

const sideIndicatorColorClass = (side: TeamSide) =>
  side === 'blue' ? 'bg-blue-500' : 'bg-red-500'

const sideLabel = (side: TeamSide) => (side === 'blue' ? '蓝方' : '红方')

const campNames: Record<JungleCamp, string> = {
  red: '红Buff',
  blue: '蓝Buff',
  wolves: '三狼',
  raptors: '六鸟'
}

const campMarkerColors: Record<JungleCamp, string> = {
  red: 'rgba(239, 68, 68, 0.85)',
  blue: 'rgba(59, 130, 246, 0.85)',
  wolves: 'rgba(156, 163, 175, 0.85)',
  raptors: 'rgba(168, 85, 247, 0.85)'
}

const sumCampCount = (camps: Record<JungleCamp, number>) =>
  CAMP_KEYS.reduce((total, camp) => total + (camps[camp] ?? 0), 0)

const firstClearGames = (side: TeamSide) =>
  side === 'blue' ? (stats.firstClearCamp?.blueGames ?? 0) : (stats.firstClearCamp?.redGames ?? 0)

const firstClearInvadeCamps = (side: TeamSide) =>
  side === 'blue' ? stats.firstClearCamp?.blueInvade : stats.firstClearCamp?.redInvade

const campStartEntries = (camps: Record<JungleCamp, number>, total: number) => {
  return CAMP_KEYS.filter((camp) => (camps[camp] ?? 0) > 0)
    .sort((a, b) => (camps[b] ?? 0) - (camps[a] ?? 0))
    .map((camp) => `${campNames[camp]} ${Math.round(((camps[camp] ?? 0) / (total || 1)) * 100)}%`)
    .join(', ')
}

const createFirstClearRow = (side: TeamSide): FirstClearRow => {
  const games = firstClearGames(side)
  const ownCamps = stats.firstClearCamp?.[side] ?? { red: 0, blue: 0, wolves: 0, raptors: 0 }
  const invadeCamps = firstClearInvadeCamps(side) ?? { red: 0, blue: 0, wolves: 0, raptors: 0 }
  const ownStarts = sumCampCount(ownCamps)
  const invadeStarts = sumCampCount(invadeCamps)

  const ownText =
    games <= 0 || ownStarts <= 0
      ? '暂无起手数据'
      : `起手 ${Math.round((ownStarts / (games || 1)) * 100)}% (${campStartEntries(ownCamps, ownStarts)})`

  const invadeText =
    games <= 0 || invadeStarts <= 0
      ? '暂无入侵起手'
      : `入侵 ${Math.round((invadeStarts / (games || 1)) * 100)}% (${campStartEntries(invadeCamps, invadeStarts)})`

  return {
    side,
    label: sideLabel(side),
    indicatorClass: sideIndicatorColorClass(side),
    own: {
      text: ownText,
      textClass: ownStarts > 0 ? 'text-slate-200' : 'text-slate-500'
    },
    invade: {
      text: invadeText,
      textClass: invadeStarts > 0 ? 'text-amber-300 font-semibold' : 'text-slate-500'
    }
  }
}

const createEarlyGankRow = (side: TeamSide): EarlyGankRow => {
  const byTeam = stats.earlyGank?.byTeam ?? {
    blueGames: 0,
    redGames: 0,
    blueLevel3GankCount: 0,
    blueLevel4GankCount: 0,
    redLevel3GankCount: 0,
    redLevel4GankCount: 0
  }
  const games = side === 'blue' ? byTeam.blueGames : byTeam.redGames
  const lv3 = side === 'blue' ? byTeam.blueLevel3GankCount : byTeam.redLevel3GankCount
  const lv4 = side === 'blue' ? byTeam.blueLevel4GankCount : byTeam.redLevel4GankCount

  return {
    side,
    label: sideLabel(side),
    indicatorClass: sideIndicatorColorClass(side),
    level3: {
      text: lv3 > 0 ? `${lv3} 场触发 (${Math.round((lv3 / (games || 1)) * 100)}%)` : '未发生',
      textClass: lv3 > 0 ? 'text-orange-300 font-medium' : 'text-slate-500'
    },
    level4: {
      text: lv4 > 0 ? `${lv4} 场触发 (${Math.round((lv4 / (games || 1)) * 100)}%)` : '未发生',
      textClass: lv4 > 0 ? 'text-purple-300 font-medium' : 'text-slate-500'
    }
  }
}

const firstClearRows = computed<FirstClearRow[]>(() => TEAM_SIDES.map(createFirstClearRow))
const earlyGankRows = computed<EarlyGankRow[]>(() => TEAM_SIDES.map(createEarlyGankRow))

const firstClearCampMapPoints = computed<FirstClearCampMapPoint[]>(() => {
  if (!stats.firstClearCamp) return []

  const blueOwnStarts = sumCampCount(stats.firstClearCamp.blue)
  const redOwnStarts = sumCampCount(stats.firstClearCamp.red)
  const blueInvadeStarts = sumCampCount(stats.firstClearCamp.blueInvade)
  const redInvadeStarts = sumCampCount(stats.firstClearCamp.redInvade)
  const blueGames = firstClearGames('blue')
  const redGames = firstClearGames('red')
  const points: FirstClearCampMapPoint[] = []

  for (const camp of BLUE_SIDE_CAMPS) {
    const entries: FirstClearCampMapPointEntry[] = []
    const blueOwnCount = stats.firstClearCamp.blue[camp.camp] ?? 0
    const redInvadeCount = stats.firstClearCamp.redInvade[camp.camp] ?? 0

    if (blueOwnCount > 0) {
      entries.push({
        side: 'blue',
        sideLabel: '蓝方',
        indicatorClass: 'bg-blue-500',
        kind: 'own',
        kindLabel: '己方',
        count: blueOwnCount,
        kindTotal: blueOwnStarts,
        sideTotal: blueGames,
        lines: []
      })
    }

    if (redInvadeCount > 0) {
      entries.push({
        side: 'red',
        sideLabel: '红方',
        indicatorClass: 'bg-red-500',
        kind: 'invade',
        kindLabel: '入侵',
        count: redInvadeCount,
        kindTotal: redInvadeStarts,
        sideTotal: redGames,
        lines: []
      })
    }

    if (entries.length > 0) {
      points.push({
        ...mapToImagePosition(camp.x, camp.y, MAP_SIZE, MAP_SIZE, 11),
        camp: camp.camp,
        campLabel: campNames[camp.camp],
        size: CAMP_MARKER_SIZE,
        hasInvade: entries.some((e) => e.kind === 'invade'),
        entries
      })
    }
  }

  for (const camp of RED_SIDE_CAMPS) {
    const entries: FirstClearCampMapPointEntry[] = []
    const redOwnCount = stats.firstClearCamp.red[camp.camp] ?? 0
    const blueInvadeCount = stats.firstClearCamp.blueInvade[camp.camp] ?? 0

    if (redOwnCount > 0) {
      entries.push({
        side: 'red',
        sideLabel: '红方',
        indicatorClass: 'bg-red-500',
        kind: 'own',
        kindLabel: '己方',
        count: redOwnCount,
        kindTotal: redOwnStarts,
        sideTotal: redGames,
        lines: []
      })
    }

    if (blueInvadeCount > 0) {
      entries.push({
        side: 'blue',
        sideLabel: '蓝方',
        indicatorClass: 'bg-blue-500',
        kind: 'invade',
        kindLabel: '入侵',
        count: blueInvadeCount,
        kindTotal: blueInvadeStarts,
        sideTotal: blueGames,
        lines: []
      })
    }

    if (entries.length > 0) {
      points.push({
        ...mapToImagePosition(camp.x, camp.y, MAP_SIZE, MAP_SIZE, 11),
        camp: camp.camp,
        campLabel: campNames[camp.camp],
        size: CAMP_MARKER_SIZE,
        hasInvade: entries.some((e) => e.kind === 'invade'),
        entries
      })
    }
  }

  return points
})

const earlyGankMapPoints = computed(() => {
  if (!stats.earlyGank) return []
  return [
    ...(stats.earlyGank.level3KillPositions || []).map((pt) => ({
      ...mapToImagePosition(pt.x, pt.y, MAP_SIZE, MAP_SIZE, 11),
      lane: pt.lane,
      level: 3 as const
    })),
    ...(stats.earlyGank.level4KillPositions || []).map((pt) => ({
      ...mapToImagePosition(pt.x, pt.y, MAP_SIZE, MAP_SIZE, 11),
      lane: pt.lane,
      level: 4 as const
    }))
  ]
})
</script>
