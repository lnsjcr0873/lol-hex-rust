<template>
  <div class="relative flex items-center justify-center select-none" :style="{ width: `${size}px`, height: `${size}px` }">
    <Radar :data="chartData" :options="chartOptions" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Chart as ChartJS,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
  Legend
} from 'chart.js'
import { Radar } from 'vue-chartjs'
import { useMatchCard } from '../context'

ChartJS.register(RadialLinearScale, PointElement, LineElement, Filler, Tooltip, Legend)

const props = withDefaults(
  defineProps<{
    puuid?: string
    size?: number
    standaloneStats?: {
      output?: number
      damage?: number
      kda?: number
      join_rate?: number
      survival?: number
      gold?: number
      vision?: number
    }
  }>(),
  {
    size: 260
  }
)

let matchCardCtx: any = null
try {
  matchCardCtx = useMatchCard()
} catch {
  // Standalone mode outside of MatchCardContext
}

const noZero = (val: number | undefined) => (val && val > 0 && Number.isFinite(val) ? val : 1)
const safePercent = (val: number, max: number) => {
  if (!Number.isFinite(val) || !Number.isFinite(max) || max <= 0) return 0
  return Math.min(Math.max(Math.round((val / max) * 100), 0), 100)
}

const labels = ['输出 (DMG)', '承伤 (TANK)', '经济 (GOLD)', '补兵 (CS)', '战损 (KDA)', '参团 (KP)']

const chartData = computed(() => {
  // If provided standaloneStats directly
  if (props.standaloneStats) {
    const s = props.standaloneStats
    return {
      labels: ['输出', '承伤', '经济', '视野', 'KDA', '参团'],
      datasets: [
        {
          label: '个人表现',
          data: [
            safePercent(s.output ?? s.damage ?? 0, 100),
            safePercent(s.survival ?? 0, 100),
            safePercent(s.gold ?? 0, 100),
            safePercent(s.vision ?? 0, 100),
            safePercent(s.kda ?? 0, 100),
            safePercent(s.join_rate ?? 0, 100)
          ],
          backgroundColor: 'rgba(10, 200, 185, 0.25)',
          borderColor: '#0ac8b9',
          pointBackgroundColor: '#00efd6',
          pointBorderColor: '#070b14',
          pointRadius: 3
        }
      ]
    }
  }

  if (!matchCardCtx) {
    return {
      labels,
      datasets: [
        {
          label: '暂无数据',
          data: [0, 0, 0, 0, 0, 0],
          backgroundColor: 'rgba(148,163,184,0.1)',
          borderColor: '#64748b'
        }
      ]
    }
  }

  const { participants, teams, puuid: ctxPuuid } = matchCardCtx
  const targetPuuid = props.puuid || ctxPuuid.value
  const p = participants.value.find((x: any) => x.puuid === targetPuuid) || participants.value[0]
  const allStats = teams.value.allTeamStats || {}
  const t = teams.value.teamStatMap?.[p?.teamIdentifier] || null
  const teamMembers = participants.value.filter((x: any) => x.teamIdentifier === p?.teamIdentifier)
  const teamSize = Math.max(teamMembers.length, 1)

  if (!p) {
    return {
      labels,
      datasets: [
        {
          label: '暂无数据',
          data: [0, 0, 0, 0, 0, 0],
          backgroundColor: 'rgba(148,163,184,0.1)'
        }
      ]
    }
  }

  // Personal ratios to match max
  const maxDmg = noZero(allStats.maxDamageDealtToChampions)
  const maxTaken = noZero(allStats.maxDamageTaken)
  const maxGold = noZero(allStats.maxGoldEarned)
  const maxCs = noZero(allStats.maxCs)
  const maxKda = noZero(allStats.maxKda)
  const maxKp = noZero(allStats.maxKillParticipation)

  const pData = [
    safePercent(p.totalDamageDealtToChampions || 0, maxDmg),
    safePercent(p.totalDamageTaken || 0, maxTaken),
    safePercent(p.goldEarned || 0, maxGold),
    safePercent(p.cs || 0, maxCs),
    safePercent(p.kda || 0, maxKda),
    safePercent(p.killParticipation || 0, maxKp)
  ]

  // Team average ratios
  const tDmg = t ? (t.totalDamageDealtToChampions || 0) / teamSize : 0
  const tTaken = t ? (t.totalDamageTaken || 0) / teamSize : 0
  const tGold = t ? (t.totalGoldEarned || 0) / teamSize : 0
  const tCs = t ? (t.totalCs || 0) / teamSize : 0
  const tKda = t ? (t.totalKda || 0) / teamSize : 0
  const tKp = t ? (t.totalKillParticipation || 0) / teamSize : 0

  const tData = [
    safePercent(tDmg, maxDmg),
    safePercent(tTaken, maxTaken),
    safePercent(tGold, maxGold),
    safePercent(tCs, maxCs),
    safePercent(tKda, maxKda),
    safePercent(tKp, maxKp)
  ]

  return {
    labels,
    datasets: [
      {
        label: `${p.gameName || '个人'}`,
        data: pData,
        backgroundColor: 'rgba(248, 113, 113, 0.28)',
        borderColor: '#f87171',
        borderWidth: 2,
        pointBackgroundColor: '#f87171',
        pointBorderColor: '#fff',
        pointHoverRadius: 5
      },
      {
        label: '队伍均值',
        data: tData,
        backgroundColor: 'rgba(148, 163, 184, 0.2)',
        borderColor: '#94a3b8',
        borderWidth: 1.5,
        borderDash: [3, 3],
        pointBackgroundColor: '#cbd5e1',
        pointBorderColor: '#64748b',
        pointRadius: 2.5
      }
    ]
  }
})

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: true,
      labels: {
        color: '#cbd5e1',
        font: { size: 10, weight: 'bold' as const },
        boxWidth: 12
      }
    },
    tooltip: {
      backgroundColor: 'rgba(15, 23, 42, 0.95)',
      titleColor: '#f8fafc',
      bodyColor: '#e2e8f0',
      borderColor: 'rgba(148, 163, 184, 0.3)',
      borderWidth: 1,
      callbacks: {
        label: (ctx: any) => `${ctx.dataset.label}: ${ctx.formattedValue}%`
      }
    }
  },
  scales: {
    r: {
      min: 0,
      max: 100,
      ticks: {
        stepSize: 25,
        display: false
      },
      grid: {
        color: 'rgba(255, 255, 255, 0.1)'
      },
      angleLines: {
        color: 'rgba(255, 255, 255, 0.12)'
      },
      pointLabels: {
        color: '#94a3b8',
        font: { size: 10, weight: 'bold' as const }
      }
    }
  }
}))
</script>
