<template>
  <div class="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-300 select-none space-y-2.5 shadow-sm">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <div class="flex items-center space-x-1.5">
        <Trophy class="w-4 h-4 text-amber-400" />
        <span class="font-extrabold text-sm text-slate-100">英雄熟练度排行</span>
      </div>
      <button
        v-if="masteries.length > 0"
        @click="showModal = true"
        class="text-[11px] text-cyan-400 hover:text-cyan-300 transition flex items-center space-x-0.5"
      >
        <span>查看全部</span>
        <ChevronRight class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Masteries list (Top 5-6) -->
    <div v-if="masteries.length > 0" class="space-y-2">
      <div
        v-for="item in masteries.slice(0, 6)"
        :key="item.championId"
        @click="showModal = true"
        class="flex items-center justify-between p-1.5 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 transition cursor-pointer group"
      >
        <div class="flex items-center space-x-2.5">
          <ChampionIcon :champion-id="item.championId" :size="32" round />
          <div>
            <div class="font-bold text-slate-100 group-hover:text-cyan-300 transition">
              {{ getChampionName(item.championId) }}
            </div>
            <div class="text-[10px] text-amber-400 font-mono">
              第 {{ item.championLevel }} 等级
              <span v-if="item.highestGrade" class="ml-1 px-1 bg-amber-500/20 rounded font-bold text-amber-300">
                {{ item.highestGrade }}
              </span>
            </div>
          </div>
        </div>

        <div class="text-right font-mono">
          <div class="text-[11px] font-bold text-slate-200">
            {{ formatPoints(item.championPoints) }}
          </div>
          <div class="text-[9px] text-slate-500">熟练度点数</div>
        </div>
      </div>

      <!-- View all footer button -->
      <button
        @click="showModal = true"
        class="w-full py-1.5 text-center text-[11px] font-medium text-slate-400 hover:text-cyan-300 rounded-lg hover:bg-slate-800/50 transition border border-dashed border-slate-800 hover:border-cyan-500/40"
      >
        查看全部 {{ allMasteries.length || masteries.length }} 位英雄熟练度
      </button>
    </div>

    <div v-else class="text-center py-4 text-slate-500 font-mono text-[11px]">
      暂无英雄熟练度数据
    </div>

    <!-- Full Champion Mastery Modal -->
    <ChampionMasteryModal
      v-model:show="showModal"
      :masteries="allMasteries.length > 0 ? allMasteries : masteries"
      @refresh="$emit('refresh')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Trophy, ChevronRight } from 'lucide-vue-next'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'
import { getChampionName } from '../match-card/utils/resources'
import ChampionMasteryModal from './ChampionMasteryModal.vue'

const props = defineProps<{
  masteriesList?: Array<{
    championId: number
    championLevel: number
    championPoints: number
    highestGrade?: string
    chestGranted?: boolean
    masteryMilestone?: number
  }>
  rawMasteriesList?: Array<{
    championId: number
    championLevel: number
    championPoints: number
    highestGrade?: string
    chestGranted?: boolean
    masteryMilestone?: number
  }>
}>()

defineEmits<{
  refresh: []
}>()

const showModal = ref(false)

const masteries = computed(() => {
  return props.masteriesList || []
})

const allMasteries = computed(() => {
  return props.rawMasteriesList || props.masteriesList || []
})

function formatPoints(pts?: number | null): string {
  const n = typeof pts === 'number' && Number.isFinite(pts) ? pts : 0
  if (n >= 10000) {
    return `${(n / 10000).toFixed(1)}万`
  }
  return n.toLocaleString()
}
</script>
