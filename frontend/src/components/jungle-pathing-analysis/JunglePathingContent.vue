<template>
  <div class="flex flex-col gap-1.5 p-1 select-none">
    <GankPreferenceSummary :stats="stats" />
    <FirstClearAndGankSummary :stats="stats" />
  </div>
</template>

<script setup lang="ts">
import type { AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'
import FirstClearAndGankSummary from './FirstClearAndGankSummary.vue'
import GankPreferenceSummary from './GankPreferenceSummary.vue'

defineProps<{
  stats: AggregatedJungleAnalysis
}>()
</script>
