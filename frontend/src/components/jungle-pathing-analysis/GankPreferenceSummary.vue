<template>
  <JunglePathingSection>
    <template #map>
      <GankMap
        :position-points="stats.minutePositions"
        :kill-points="stats.gankPositions"
        :size="140"
        :position-limit="54"
        :kill-limit="100"
        balance-by-lane
        show-side-fill
        position-dot-size-class="h-1 w-1"
        kill-marker-size-class="h-3 w-3"
      />
      <div class="flex flex-col gap-1 pl-0.5 text-[10px] text-slate-400">
        <span class="inline-flex items-center gap-1 whitespace-nowrap">
          <span class="grid w-7.5 grid-cols-3 place-items-center">
            <svg class="h-2 w-2 text-red-500" viewBox="0 0 8 8" aria-hidden="true">
              <circle cx="4" cy="4" r="2" fill="currentColor" />
            </svg>
            <svg class="h-2 w-2 text-amber-400" viewBox="0 0 8 8" aria-hidden="true">
              <circle cx="4" cy="4" r="2" fill="currentColor" />
            </svg>
            <svg class="h-2 w-2 text-blue-500" viewBox="0 0 8 8" aria-hidden="true">
              <circle cx="4" cy="4" r="2" fill="currentColor" />
            </svg>
          </span>
          分钟散点 (1~14m)
        </span>
        <span class="inline-flex items-center gap-1 whitespace-nowrap">
          <span class="grid w-7.5 grid-cols-3 place-items-center">
            <span class="relative h-2.5 w-2.5">
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm bg-red-500" />
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm bg-red-500" />
            </span>
            <span class="relative h-2.5 w-2.5">
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm bg-amber-400" />
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm bg-amber-400" />
            </span>
            <span class="relative h-2.5 w-2.5">
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 rounded-sm bg-blue-500" />
              <span class="absolute top-1/2 left-1/2 h-0.5 w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 rounded-sm bg-blue-500" />
            </span>
          </span>
          Gank 击杀点位 (红上/黄中/蓝下)
        </span>
      </div>
    </template>

    <template #content>
      <MapPreferenceSummary :stats="stats" />
      <ObjectivesSummary :stats="stats" />
    </template>
  </JunglePathingSection>
</template>

<script setup lang="ts">
import type { AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'
import GankMap from './GankMap.vue'
import JunglePathingSection from './JunglePathingSection.vue'
import MapPreferenceSummary from './MapPreferenceSummary.vue'
import ObjectivesSummary from './ObjectivesSummary.vue'

defineProps<{
  stats: AggregatedJungleAnalysis
}>()
</script>
