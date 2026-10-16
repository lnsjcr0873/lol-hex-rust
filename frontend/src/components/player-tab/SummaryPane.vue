<template>
  <div class="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-300 select-none space-y-2.5 shadow-sm">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <span class="font-extrabold text-sm text-slate-100">近期战绩综合概览</span>
      <span class="text-[10px] text-slate-400 font-mono">{{ subtitle }}</span>
    </div>

    <!-- Metrics rows -->
    <div class="space-y-1.5 font-sans">
      <!-- Akari score -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">Akari 综合评分</span>
        <AkariScorePopover :score="analysis?.akariScore">
          <span class="font-mono font-bold text-[#0ac8b9] text-xs cursor-pointer hover:underline">
            {{ akariScoreText }}
          </span>
        </AkariScorePopover>
      </div>

      <!-- Win Rate & Streak -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">总胜率 (胜/负)</span>
        <div class="flex items-center space-x-1.5">
          <span class="font-mono text-slate-200">
            <strong :class="winRate >= 50 ? 'text-emerald-400' : 'text-rose-400'">{{ winRate }}%</strong>
            <span class="text-slate-400 text-[11px] ml-1">({{ wins }}胜 {{ losses }}负)</span>
          </span>
          <span
            v-if="currentStreak"
            class="px-1.5 py-0.5 rounded text-[10px] font-bold font-mono border shadow-sm leading-none"
            :class="currentStreak.isWinning
              ? 'border-emerald-500/40 bg-emerald-500/15 text-emerald-300'
              : 'border-rose-500/40 bg-rose-500/15 text-rose-300'"
          >
            {{ currentStreak.count }}{{ currentStreak.isWinning ? '连胜' : '连败' }}
          </span>
        </div>
      </div>

      <!-- Blue / Red Side Distribution -->
      <div v-if="blueSideCount > 0 || redSideCount > 0" class="flex items-center justify-between">
        <span class="text-slate-400">红蓝方阵营</span>
        <div class="flex items-center space-x-2 font-mono text-xs">
          <div class="flex items-center space-x-1 text-sky-400">
            <span class="w-2 h-2 rounded-full bg-sky-400"></span>
            <span>蓝方 {{ blueSideCount }}</span>
          </div>
          <span class="text-slate-500">/</span>
          <div class="flex items-center space-x-1 text-rose-400">
            <span class="w-2 h-2 rounded-full bg-rose-400"></span>
            <span>红方 {{ redSideCount }}</span>
          </div>
        </div>
      </div>

      <!-- KDA -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">平均 KDA 战损比</span>
        <span class="font-mono font-bold text-slate-200">
          {{ avgKda }}
        </span>
      </div>

      <!-- KP% -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">平均参团率 (KP)</span>
        <span class="font-mono text-[#0ac8b9] font-bold">
          {{ avgKp }}{{ avgKp !== '—' ? '%' : '' }}
        </span>
      </div>

      <!-- DMG% -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">队内输出占比</span>
        <span class="font-mono text-slate-200">
          {{ avgDmgShare }}{{ avgDmgShare !== '—' ? '%' : '' }}
        </span>
      </div>

      <!-- TANK% -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">队内承伤占比</span>
        <span class="font-mono text-slate-200">
          {{ avgTankShare }}{{ avgTankShare !== '—' ? '%' : '' }}
        </span>
      </div>

      <!-- CS/min -->
      <div class="flex items-center justify-between">
        <span class="text-slate-400">分均补刀</span>
        <span class="font-mono text-slate-200">
          {{ csPerMinute }}{{ csPerMinute !== '—' ? ' 刀/分' : '' }}
        </span>
      </div>
    </div>

    <!-- Frequently used champions -->
    <div v-if="topChampions.length > 0" class="pt-2 border-t border-slate-800/80">
      <div class="text-[11px] font-bold text-slate-400 mb-2">常用英雄表现</div>
      <div class="flex items-center space-x-2">
        <div
          v-for="ch in topChampions.slice(0, 4)"
          :key="ch.championId"
          class="flex flex-col items-center group cursor-pointer"
          :title="`${getChampionName(ch.championId)}: ${ch.games}场 胜率${ch.winRate}% KDA ${ch.kda}`"
        >
          <div class="relative">
            <ChampionIcon :champion-id="ch.championId" :size="32" round />
            <span class="absolute -bottom-1 -right-1 text-[8px] px-1 bg-black/90 rounded border border-slate-700 text-slate-300 font-mono">
              {{ ch.games }}场
            </span>
          </div>
          <span class="text-[10px] text-[#0ac8b9] font-bold mt-1 font-mono">{{ ch.winRate }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export const QUEUE_NAME_MAP: Record<string, string> = {
  all: '全部',
  ranked_solo: '单双排',
  ranked_flex: '灵活排位',
  aram: '极地大乱斗',
  normal: '匹配模式',
  '420': '单双排',
  '440': '灵活排位',
  '450': '极地大乱斗',
  '430': '匹配模式'
}
</script>

<script setup lang="ts">
import { computed } from 'vue'
import AkariScorePopover from './widgets/AkariScorePopover.vue'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'
import { getChampionName } from '../match-card/utils/resources'

const props = withDefaults(
  defineProps<{
    analysis?: any
    profileSummary?: any
    queue?: string
    sampleCount?: number
    currentPage?: number
  }>(),
  {
    queue: 'all',
    currentPage: 1
  }
)

const computedSampleCount = computed<number>(() => {
  return totalGames.value
})

const subtitle = computed<string>(() => {
  if (computedSampleCount.value === 0) {
    return '暂无对局样本'
  }
  const q = props.queue ? String(props.queue).toLowerCase() : 'all'
  const queueLabel = QUEUE_NAME_MAP[q] ?? QUEUE_NAME_MAP[props.queue || ''] ?? (props.queue ? `${props.queue}` : '对局')
  return `近 ${computedSampleCount.value} 场${queueLabel}样本`
})

const totalGames = computed<number>(() => {
  if (props.analysis?.count !== undefined && props.analysis.count > 0) {
    return props.analysis.count
  }
  if (props.analysis?.winLoss?.all?.count !== undefined && props.analysis.winLoss.all.count > 0) {
    return props.analysis.winLoss.all.count
  }
  if (props.sampleCount !== undefined && props.sampleCount > 0) {
    return props.sampleCount
  }
  return 0
})

const winRate = computed<number>(() => {
  if (props.analysis?.winLoss?.all?.winRate !== undefined) {
    return Math.round(props.analysis.winLoss.all.winRate * 100)
  }
  if (props.analysis?.winLoss?.all?.wins !== undefined && totalGames.value > 0) {
    return Math.round((props.analysis.winLoss.all.wins / totalGames.value) * 100)
  }
  return props.profileSummary?.win_rate ?? 0
})

const wins = computed<number>(() => {
  if (props.analysis?.winLoss?.all?.wins !== undefined) {
    return props.analysis.winLoss.all.wins
  }
  if (totalGames.value > 0) {
    return Math.round((winRate.value / 100) * totalGames.value)
  }
  return 0
})

const losses = computed<number>(() => {
  if (props.analysis?.winLoss?.all?.losses !== undefined) {
    return props.analysis.winLoss.all.losses
  }
  if (totalGames.value > 0) {
    return Math.max(0, totalGames.value - wins.value)
  }
  return 0
})

const currentStreak = computed(() => {
  if (props.currentPage !== undefined && props.currentPage > 1) {
    return null
  }
  if (!props.analysis?.winLoss?.all) return null
  const winStreak = props.analysis.winLoss.all.winningStreak ?? 0
  const loseStreak = props.analysis.winLoss.all.losingStreak ?? 0

  if (winStreak >= 2) {
    return { isWinning: true, count: winStreak }
  }
  if (loseStreak >= 2) {
    return { isWinning: false, count: loseStreak }
  }
  return null
})

const blueSideCount = computed<number>(() => {
  return props.analysis?.teamSide?.blueSideCount ?? 0
})

const redSideCount = computed<number>(() => {
  return props.analysis?.teamSide?.redSideCount ?? 0
})

const akariScoreText = computed(() => {
  const val = props.analysis?.akariScore?.total
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return val.toFixed(2)
  }
  return '—'
})

const avgKda = computed(() => {
  const val = props.analysis?.summary?.avgKda
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return val.toFixed(2)
  }
  if (props.profileSummary?.avg_kda !== undefined) {
    return String(props.profileSummary.avg_kda)
  }
  return '—'
})

const avgKp = computed(() => {
  const val = props.analysis?.summary?.avgKillParticipation
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return (val * 100).toFixed(0)
  }
  return props.profileSummary?.avg_kp ? String(props.profileSummary.avg_kp) : '—'
})

const avgDmgShare = computed(() => {
  const val = props.analysis?.summary?.avgChampionDamagePercentageOfTeam
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return (val * 100).toFixed(1)
  }
  return props.profileSummary?.avg_damage_share ? String(props.profileSummary.avg_damage_share) : '—'
})

const avgTankShare = computed(() => {
  const val = props.analysis?.summary?.avgDamageTakenPercentageOfTeam
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return (val * 100).toFixed(1)
  }
  return props.profileSummary?.avg_damage_taken_share ? String(props.profileSummary.avg_damage_taken_share) : '—'
})

const csPerMinute = computed(() => {
  const val = props.analysis?.summary?.avgCsPerMinute
  if (val !== undefined && val !== null && Number.isFinite(val)) {
    return val.toFixed(1)
  }
  return props.profileSummary?.avg_cs_per_min ? String(props.profileSummary.avg_cs_per_min) : '—'
})

const topChampions = computed(() => {
  if (props.analysis?.champions) {
    const list = (Array.isArray(props.analysis.champions)
      ? props.analysis.champions
      : Object.values(props.analysis.champions)) as any[]
    if (list.length > 0) {
      return list
        .filter((c) => (c.winLoss?.all?.count ?? 0) >= 1)
        .sort((a, b) => (b.winLoss?.all?.count || 0) - (a.winLoss?.all?.count || 0))
        .map((c: any) => ({
          championId: c.championId,
          games: c.winLoss?.all?.count ?? 0,
          winRate: Math.round((c.winLoss?.all?.winRate || 0) * 100),
          kda: Number.isFinite(c.summary?.avgKda) ? (c.summary.avgKda).toFixed(1) : '0.0'
        }))
    }
  }
  if (props.profileSummary?.top_champions?.length) {
    return props.profileSummary.top_champions.map((c: any) => ({
      championId: c.champion_id,
      games: c.games,
      winRate: c.win_rate,
      kda: c.kda
    }))
  }
  return []
})
</script>
