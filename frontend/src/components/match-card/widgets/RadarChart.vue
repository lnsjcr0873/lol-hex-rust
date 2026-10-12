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

const noZero = (val: number | undefined) => (val && val > 0 ? val : 1)

const labels = ['输出 (DMG)', '承伤 (TANK)', '经济 (GOLD)', '补兵 (CS)', '战损 (KDA)', '参团 (KP)']

const chartData = computed(() => {
  // If provided standaloneStats directly
  if (props.standaloneStats) {
    const s = props.standaloneStats
    return {
      labels: ['输出', '承伤', '经济', '补刀', 'KDA', '参团'],
      datasets: [
        {
          label: '个人表现',
          data: [
            s.output ?? 60,
            s.survival ?? 60,
            s.gold ?? 60,
            s.vision ?? 60,
            s.kda ?? 60,
            s.join_rate ?? 60
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
          label: '个人',
          data: [50, 50, 50, 50, 50, 50],
          backgroundColor: 'rgba(248,113,113,0.25)',
          borderColor: '#f87171'
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
    Math.min(Math.round(((p.totalDamageDealtToChampions || 0) / maxDmg) * 100), 100),
    Math.min(Math.round(((p.totalDamageTaken || 0) / maxTaken) * 100), 100),
    Math.min(Math.round(((p.goldEarned || 0) / maxGold) * 100), 100),
    Math.min(Math.round(((p.cs || 0) / maxCs) * 100), 100),
    Math.min(Math.round(((p.kda || 0) / maxKda) * 100), 100),
    Math.min(Math.round(((p.killParticipation || 0) / maxKp) * 100), 100)
  ]

  // Team average ratios
  const tDmg = t ? (t.totalDamageDealtToChampions || 0) / teamSize : 0
  const tTaken = t ? (t.totalDamageTaken || 0) / teamSize : 0
  const tGold = t ? (t.totalGoldEarned || 0) / teamSize : 0
  const tCs = t ? (t.totalCs || 0) / teamSize : 0
  const tKda = t ? (t.totalKda || 0) / teamSize : 0
  const tKp = t ? (t.totalKillParticipation || 0) / teamSize : 0

  const tData = [
    Math.min(Math.round((tDmg / maxDmg) * 100), 100),
    Math.min(Math.round((tTaken / maxTaken) * 100), 100),
    Math.min(Math.round((tGold / maxGold) * 100), 100),
    Math.min(Math.round((tCs / maxCs) * 100), 100),
    Math.min(Math.round((tKda / maxKda) * 100), 100),
    Math.min(Math.round((tKp / maxKp) * 100), 100)
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
