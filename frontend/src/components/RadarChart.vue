<template>
  <RadarChartWidget :standalone-stats="stats" :size="size" :puuid="puuid" />
</template>

<script setup lang="ts">
import RadarChartWidget from './match-card/widgets/RadarChart.vue'

withDefaults(
  defineProps<{
    stats?: {
      output?: number
      damage?: number
      kda?: number
      join_rate?: number
      survival?: number
      gold?: number
      vision?: number
    }
    puuid?: string
    size?: number
  }>(),
  {
    size: 180
  }
)
</script>
