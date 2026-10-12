<template>
  <div class="h-full flex flex-col space-y-3">
    <!-- Sub-tab switcher -->
    <div class="flex items-center space-x-2">
      <button
        @click="currentSection = 'diff-line-chart'"
        class="px-3 py-1 rounded-md text-xs font-semibold border transition"
        :class="currentSection === 'diff-line-chart' ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 border-transparent'"
      >
        经济与输出走势图
      </button>
      <button
        @click="currentSection = 'stats-line'"
        class="px-3 py-1 rounded-md text-xs font-semibold border transition"
        :class="currentSection === 'stats-line' ? 'bg-[#0f233a] text-[#0ac8b9] border-[#0ac8b9]' : 'text-slate-400 hover:text-slate-200 border-transparent'"
      >
        逐帧属性与站位回放
      </button>
    </div>

    <!-- Active view -->
    <div class="flex-1 min-h-0">
      <MatchCardDiffLineChart v-if="currentSection === 'diff-line-chart'" />
      <MatchCardStatsLine v-else-if="currentSection === 'stats-line'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import MatchCardDiffLineChart from './timeline/MatchCardDiffLineChart.vue'
import MatchCardStatsLine from './timeline/MatchCardStatsLine.vue'

const currentSection = ref<'diff-line-chart' | 'stats-line'>('diff-line-chart')
</script>
