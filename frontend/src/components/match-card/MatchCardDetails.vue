<template>
  <div
    class="relative mt-1.5 p-3 rounded-xl border border-solid select-none bg-slate-950/90 shadow-2xl transition-all"
    :class="cardBorderClass"
  >
    <!-- Top Details Header with Tabs -->
    <div class="flex items-center justify-between pb-3 mb-3 border-b border-slate-800/80 gap-3">
      <!-- 7 Sub-tabs switcher -->
      <TabSwitch
        v-model:selected-tab="selectedTab"
        :tabs="tabs"
        :win-result="team?.winResult"
      />

      <!-- Action buttons -->
      <div class="flex items-center space-x-2 shrink-0">
        <button
          v-if="details"
          @click="loadDetails(basicInfo.gameId)"
          class="px-2.5 py-1 rounded-md text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-slate-300 border border-slate-700 transition"
          title="刷新时间线数据"
        >
          刷新
        </button>
      </div>
    </div>

    <!-- Active Tab Content with KeepAlive -->
    <div class="min-h-[300px]">
      <KeepAlive>
        <MatchCardSummaryTab v-if="selectedTab === 'summary'" />
        <MatchCardDetailsTab v-else-if="selectedTab === 'details'" />
        <MatchCardRunesTab v-else-if="selectedTab === 'runes'" />
        <MatchCardBuildsTab v-else-if="selectedTab === 'builds'" />
        <MatchCardEventsTab v-else-if="selectedTab === 'events'" />
        <MatchCardTimelineTab v-else-if="selectedTab === 'timeline'" />
        <div v-else-if="selectedTab === 'radar'" class="flex flex-col items-center justify-center p-6 bg-slate-900/50 rounded-xl border border-slate-800">
          <div class="text-xs font-bold text-slate-200 mb-4">
            全景 6 维综合能力雷达图 (个人 vs 队伍平均)
          </div>
          <RadarChart :puuid="participant?.puuid" :size="320" />
        </div>
      </KeepAlive>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMatchCard } from './context'
import { useCardBorderClass } from './utils/theme'
import TabSwitch from './widgets/TabSwitch.vue'
import RadarChart from './widgets/RadarChart.vue'
import MatchCardSummaryTab from './tabs/MatchCardSummaryTab.vue'
import MatchCardDetailsTab from './tabs/MatchCardDetailsTab.vue'
import MatchCardRunesTab from './tabs/MatchCardRunesTab.vue'
import MatchCardBuildsTab from './tabs/MatchCardBuildsTab.vue'
import MatchCardEventsTab from './tabs/MatchCardEventsTab.vue'
import MatchCardTimelineTab from './tabs/MatchCardTimelineTab.vue'

const { team, basicInfo, participant, details, loadDetails } = useMatchCard()

const selectedTab = ref('summary')

const cardBorderClass = computed(() => useCardBorderClass(team.value?.winResult))

const tabs = [
  { label: '数据总览', value: 'summary' },
  { label: '详细数据表', value: 'details' },
  { label: '符文面板', value: 'runes' },
  { label: '技能与出装', value: 'builds' },
  { label: '事件时间轴', value: 'events' },
  { label: '经济走势图', value: 'timeline' },
  { label: '六维雷达图', value: 'radar' }
]
</script>
