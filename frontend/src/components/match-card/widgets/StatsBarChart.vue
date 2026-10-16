<template>
  <div class="w-80 p-3 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl text-slate-100">
    <div v-if="title" class="text-xs font-bold text-center mb-2.5 text-slate-200">
      {{ title }} (10人横向对比)
    </div>
    <div class="w-full" :style="{ height: `${Math.max(safeChartData.length * 24, 180)}px` }">
      <Bar :data="barData" :options="chartOptions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  BarElement,
  CategoryScale,
  LinearScale
} from 'chart.js'
import { Bar } from 'vue-chartjs'
import { useMatchCard } from '../context'
import { getChampionName } from '../utils/resources'
import { getTeamColor } from '../utils/theme'

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale)

const props = withDefaults(
  defineProps<{
    title?: string
    chartData?: Array<{
      participantId: number
      value: number
    }>
  }>(),
  {
    title: '',
    chartData: () => []
  }
)

const safeChartData = computed(() => (Array.isArray(props.chartData) ? props.chartData : []))

const { participants } = useMatchCard()

const participantMap = computed(() => {
  const map: Record<number, any> = {}
  for (const p of participants.value) {
    map[p.participantId] = p
  }
  return map
})

const barData = computed(() => {
  const dataList = safeChartData.value
  const labels = dataList.map((d) => {
    const p = participantMap.value[d.participantId]
    return p ? getChampionName(p.championId) : `P${d.participantId}`
  })

  const values = dataList.map((d) => (Number.isFinite(d.value) ? d.value : 0))
  const colors = dataList.map((d) => {
    const p = participantMap.value[d.participantId]
    return p ? getTeamColor(p.teamIdentifier) : '#3b82f6'
  })

  return {
    labels,
    datasets: [
      {
        data: values,
        backgroundColor: colors,
        borderRadius: 4
      }
    ]
  }
})

const chartOptions = computed(() => ({
  indexAxis: 'y' as const,
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: false
    },
    tooltip: {
      callbacks: {
        label: (ctx: any) => {
          const item = safeChartData.value[ctx.dataIndex]
          const p = item ? participantMap.value[item.participantId] : null
          const name = p ? (p.gameName || p.summonerName || `P${p.participantId}`) : ''
          return `${name}: ${ctx.formattedValue}`
        }
      }
    }
  },
  scales: {
    x: {
      grid: {
        color: 'rgba(255, 255, 255, 0.08)'
      },
      ticks: {
        color: '#94a3b8',
        font: { size: 10 }
      }
    },
    y: {
      grid: {
        display: false
      },
      ticks: {
        color: '#e2e8f0',
        font: { size: 11, weight: 'bold' as const }
      }
    }
  }
}))
</script>
