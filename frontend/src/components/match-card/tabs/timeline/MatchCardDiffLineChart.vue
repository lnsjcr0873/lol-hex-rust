<template>
  <div class="flex h-[520px] w-full gap-4">
    <!-- Left: Line Chart Canvas -->
    <div class="flex-1 min-h-0 bg-slate-950/80 rounded-xl border border-slate-800 p-3">
      <Line :data="chartData" :options="chartOptions" />
    </div>

    <!-- Right: Metric & Player Controls -->
    <NScrollbar class="w-56 shrink-0 bg-slate-900/90 rounded-xl border border-slate-800 p-3">
      <div class="space-y-4 text-xs">
        <!-- Metric Radio Group -->
        <div>
          <div class="font-bold text-slate-200 mb-2">数据对比维度</div>
          <NRadioGroup v-model:value="selectedMetric" name="metricGroup">
            <div class="flex flex-col space-y-1.5">
              <NRadio value="gold">经济曲线 (Gold)</NRadio>
              <NRadio value="cs">补刀走势 (CS)</NRadio>
              <NRadio value="exp">经验增长 (EXP)</NRadio>
              <NRadio value="damageDealt">输出走势 (DMG)</NRadio>
              <NRadio value="damageTaken">承伤走势 (TANK)</NRadio>
            </div>
          </NRadioGroup>
        </div>

        <div class="h-px bg-slate-800"></div>

        <!-- Team Average Toggle -->
        <div>
          <div class="font-bold text-slate-200 mb-2">团队平均基准线</div>
          <div class="space-y-1.5">
            <label class="flex items-center space-x-2 cursor-pointer text-slate-300">
              <input type="checkbox" v-model="showBlueTeamAvg" class="rounded border-slate-700 text-blue-500" />
              <span class="text-blue-400 font-medium">蓝方平均走势</span>
            </label>
            <label class="flex items-center space-x-2 cursor-pointer text-slate-300">
              <input type="checkbox" v-model="showRedTeamAvg" class="rounded border-slate-700 text-red-500" />
              <span class="text-red-400 font-medium">红方平均走势</span>
            </label>
          </div>
        </div>

        <div class="h-px bg-slate-800"></div>

        <!-- Player Selection -->
        <div>
          <div class="flex items-center justify-between font-bold text-slate-200 mb-2">
            <span>选手多选</span>
            <button @click="toggleAllPlayers" class="text-[11px] text-[#0ac8b9] hover:underline">
              {{ allSelected ? '全不选' : '全选' }}
            </button>
          </div>
          <div class="space-y-1.5">
            <label
              v-for="(p, idx) in participants"
              :key="p.participantId"
              class="flex items-center space-x-2 cursor-pointer text-slate-300 truncate"
            >
              <input
                type="checkbox"
                :value="p.participantId"
                v-model="selectedPlayers"
                class="rounded border-slate-700"
              />
              <span class="w-2.5 h-2.5 rounded-sm shrink-0" :style="{ backgroundColor: playerColors[idx % playerColors.length] }"></span>
              <span class="truncate text-[11px]">{{ p.gameName || `P${p.participantId}` }}</span>
            </label>
          </div>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js'
import { Line } from 'vue-chartjs'
import { NRadioGroup, NRadio, NScrollbar } from 'naive-ui'
import { useMatchCard } from '../../context'
import { playerColors } from '../../utils/theme'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend)

const { participants, frames, basicInfo } = useMatchCard()

type MetricType = 'gold' | 'cs' | 'exp' | 'damageDealt' | 'damageTaken'
const selectedMetric = ref<MetricType>('gold')
const showBlueTeamAvg = ref(true)
const showRedTeamAvg = ref(true)

const selectedPlayers = ref<number[]>(participants.value.map((p) => p.participantId))

const allSelected = computed(() => selectedPlayers.value.length === participants.value.length)

function toggleAllPlayers() {
  if (allSelected.value) {
    selectedPlayers.value = []
  } else {
    selectedPlayers.value = participants.value.map((p) => p.participantId)
  }
}

// Generate labels (minutes 0, 1, 2, ... N)
const totalMinutes = computed(() => Math.max(Math.floor(basicInfo.value.gameDuration / 60), 15))

const timeLabels = computed(() => {
  const count = frames.value.length > 0 ? frames.value.length : totalMinutes.value + 1
  return Array.from({ length: count }, (_, i) => `${i}分`)
})

const chartData = computed(() => {
  const datasets: any[] = []

  // Extract or simulate frames for each participant
  const hasRawFrames = frames.value.length > 0 && frames.value[0]?.participantFrames

  // 1. Players lines
  participants.value.forEach((p, idx) => {
    if (!selectedPlayers.value.includes(p.participantId)) return

    let dataPoints: number[] = []

    if (hasRawFrames) {
      dataPoints = frames.value.map((f: any) => {
        const pf = f.participantFrames?.[p.participantId] || f.participantFrames?.[String(p.participantId)]
        if (!pf) return 0
        switch (selectedMetric.value) {
          case 'gold':
            return pf.totalGold || pf.currentGold || 0
          case 'cs':
            return (pf.minionsKilled || 0) + (pf.jungleMinionsKilled || 0)
          case 'exp':
            return pf.xp || 0
          case 'damageDealt':
            return pf.damageStats?.totalDamageDoneToChampions || pf.totalGold || 0
          case 'damageTaken':
            return pf.damageStats?.totalDamageTaken || pf.totalGold || 0
          default:
            return 0
        }
      })
    } else {
      // Smooth curve based on final match stats
      const steps = totalMinutes.value + 1
      let finalVal = 0
      switch (selectedMetric.value) {
        case 'gold':
          finalVal = p.goldEarned || 12000
          break
        case 'cs':
          finalVal = p.cs || 180
          break
        case 'exp':
          finalVal = p.championLevel * 1200
          break
        case 'damageDealt':
          finalVal = p.totalDamageDealtToChampions || 20000
          break
        case 'damageTaken':
          finalVal = p.totalDamageTaken || 18000
          break
      }

      dataPoints = Array.from({ length: steps }, (_, m) => {
        const progress = m / (steps - 1)
        // Natural LoL progression curve
        return Math.round(finalVal * Math.pow(progress, 1.3))
      })
    }

    datasets.push({
      label: p.gameName || `P${p.participantId}`,
      data: dataPoints,
      borderColor: playerColors[idx % playerColors.length],
      backgroundColor: playerColors[idx % playerColors.length],
      tension: 0.3,
      borderWidth: 2,
      pointRadius: 1.5,
      pointHoverRadius: 4
    })
  })

  // 2. Team averages
  if (showBlueTeamAvg.value) {
    const isBlueTeam = (id?: string) => id === '100' || id === 'blue' || id === 'TEAM-100'
    const bluePlayers = datasets.filter((_, i) => isBlueTeam(participants.value[i]?.teamIdentifier))
    if (bluePlayers.length > 0) {
      const avgData = timeLabels.value.map((_, minuteIdx) => {
        const sum = bluePlayers.reduce((acc, d) => acc + (d.data[minuteIdx] || 0), 0)
        return Math.round(sum / bluePlayers.length)
      })
      datasets.push({
        label: '蓝方队伍均值',
        data: avgData,
        borderColor: '#38bdf8',
        borderWidth: 2,
        borderDash: [5, 5],
        pointRadius: 0,
        tension: 0.3
      })
    }
  }

  if (showRedTeamAvg.value) {
    const isRedTeam = (id?: string) => id === '200' || id === 'red' || id === 'TEAM-200'
    const redPlayers = datasets.filter((_, i) => isRedTeam(participants.value[i]?.teamIdentifier))
    if (redPlayers.length > 0) {
      const avgData = timeLabels.value.map((_, minuteIdx) => {
        const sum = redPlayers.reduce((acc, d) => acc + (d.data[minuteIdx] || 0), 0)
        return Math.round(sum / redPlayers.length)
      })
      datasets.push({
        label: '红方队伍均值',
        data: avgData,
        borderColor: '#f87171',
        borderWidth: 2,
        borderDash: [5, 5],
        pointRadius: 0,
        tension: 0.3
      })
    }
  }

  return {
    labels: timeLabels.value,
    datasets
  }
})

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  interaction: {
    mode: 'index' as const,
    intersect: false
  },
  plugins: {
    legend: {
      display: false
    },
    tooltip: {
      backgroundColor: 'rgba(15, 23, 42, 0.95)',
      titleColor: '#f8fafc',
      bodyColor: '#e2e8f0',
      borderColor: 'rgba(148, 163, 184, 0.3)',
      borderWidth: 1,
      padding: 10
    }
  },
  scales: {
    x: {
      grid: {
        color: 'rgba(255, 255, 255, 0.06)'
      },
      ticks: {
        color: '#94a3b8',
        font: { size: 10 }
      }
    },
    y: {
      grid: {
        color: 'rgba(255, 255, 255, 0.06)'
      },
      ticks: {
        color: '#cbd5e1',
        font: { size: 10 }
      }
    }
  }
}))
</script>
