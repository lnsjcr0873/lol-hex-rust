<template>
  <n-modal
    :show="show"
    @update:show="$emit('update:show', $event)"
    :mask-closable="true"
    :close-on-esc="true"
    transform-origin="center"
  >
    <div class="w-[720px] max-w-[92vw] h-[82vh] max-h-[760px] flex flex-col rounded-2xl bg-slate-900/95 border border-slate-700/80 backdrop-blur-xl shadow-2xl p-5 text-slate-200 select-none overflow-hidden">
      <!-- Modal Header -->
      <div class="flex items-center justify-between pb-3.5 border-b border-slate-800 shrink-0">
        <div>
          <h2 class="text-base font-bold text-slate-100 flex items-center space-x-2">
            <Trophy class="w-5 h-5 text-amber-400" />
            <span>全量英雄成就与熟练度</span>
          </h2>
          <div class="text-xs text-slate-400 mt-0.5">
            总成就点数: <span class="font-bold text-amber-300 font-mono">{{ totalScore.toLocaleString() }}</span>
            <span class="mx-1.5 text-slate-600">·</span>
            已展示: <span class="text-slate-300 font-semibold">{{ filteredMasteries.length }}</span> / {{ masteries.length }} 位英雄
          </div>
        </div>

        <div class="flex items-center space-x-1.5">
          <button
            @click="handleRefresh"
            :disabled="isRefreshing"
            class="p-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition active:scale-95 border border-slate-700/80"
            title="刷新熟练度数据"
          >
            <RotateCw class="w-4 h-4" :class="{ 'animate-spin': isRefreshing }" />
          </button>
          <button
            @click="$emit('update:show', false)"
            class="p-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white transition active:scale-95 border border-slate-700/80"
            title="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Filter & Sort Bar -->
      <div class="py-3 flex items-center justify-between gap-3 shrink-0">
        <!-- Search Input -->
        <div class="relative flex-1">
          <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            v-model="searchKeyword"
            type="text"
            placeholder="搜索英雄名称或称号 (如: 亚索 / 疾风剑豪 / 盲僧)..."
            class="w-full pl-9 pr-8 py-1.5 rounded-xl bg-slate-950/80 border border-slate-800 focus:border-cyan-500 text-xs text-slate-200 placeholder-slate-500 outline-none transition"
          />
          <button
            v-if="searchKeyword"
            @click="searchKeyword = ''"
            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Sort Selectors -->
        <div class="flex items-center space-x-1 bg-slate-950/80 p-1 rounded-xl border border-slate-800 shrink-0 text-xs">
          <button
            v-for="mode in sortModes"
            :key="mode.key"
            @click="activeSort = mode.key"
            class="px-2.5 py-1 rounded-lg font-medium transition text-[11px]"
            :class="activeSort === mode.key ? 'bg-cyan-950 text-cyan-300 font-semibold' : 'text-slate-400 hover:text-slate-200'"
          >
            {{ mode.label }}
          </button>
        </div>
      </div>

      <!-- Masteries List -->
      <div class="flex-1 overflow-y-auto pr-1">
        <!-- Loading state -->
        <div v-if="loading" class="h-60 flex flex-col items-center justify-center space-y-2 text-slate-400">
          <Loader2 class="w-8 h-8 animate-spin text-cyan-400" />
          <span class="text-xs">加载英雄熟练度数据中...</span>
        </div>

        <!-- Empty result -->
        <div v-else-if="filteredMasteries.length === 0" class="h-60 flex flex-col items-center justify-center space-y-2 text-slate-500">
          <Inbox class="w-8 h-8" />
          <span class="text-xs">未找到符合条件的英雄熟练度记录</span>
        </div>

        <!-- Mastery Items -->
        <div v-else class="space-y-1.5">
          <div
            v-for="item in filteredMasteries"
            :key="item.championId"
            class="p-2.5 px-3 rounded-xl bg-slate-950/60 hover:bg-slate-800/50 border border-slate-800/80 hover:border-slate-700/80 transition flex items-center justify-between"
          >
            <!-- Left: Icon & Names -->
            <div class="flex items-center space-x-3 min-w-0">
              <ChampionIcon :champion-id="item.championId" :size="38" round />

              <div class="min-w-0">
                <div class="flex items-center space-x-2">
                  <span class="font-bold text-slate-100 text-xs">
                    {{ getChampionName(item.championId) }}
                  </span>
                  <span class="text-[10px] text-slate-400 font-medium">
                    {{ getChampionTitle(item.championId) }}
                  </span>
                  <span
                    v-if="item.highestGrade"
                    class="px-1.5 py-0.2 rounded text-[10px] font-bold font-mono"
                    :class="item.highestGrade.startsWith('S') ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40' : 'bg-slate-800 text-slate-300'"
                  >
                    {{ item.highestGrade }}
                  </span>
                  <span
                    v-if="item.chestGranted"
                    class="px-1.5 py-0.2 rounded text-[9px] font-semibold bg-indigo-950 border border-indigo-500/40 text-indigo-300"
                    title="本赛季海克斯宝箱已获取"
                  >
                    宝箱已获取
                  </span>
                </div>

                <div class="text-[10px] text-amber-400 font-mono mt-0.5 flex items-center space-x-2">
                  <span>第 {{ item.championLevel }} 等级</span>
                  <span v-if="item.masteryMilestone != null" class="text-slate-500">
                    里程碑: {{ item.masteryMilestone }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Right: Points -->
            <div class="text-right font-mono shrink-0">
              <div class="text-xs font-bold text-slate-200">
                {{ formatPoints(item.championPoints) }}
              </div>
              <div class="text-[9px] text-slate-500">熟练度点数</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NModal } from 'naive-ui'
import { Trophy, RotateCw, X, Search, Loader2, Inbox } from 'lucide-vue-next'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'
import { getChampionName, getChampionTitle } from '../match-card/utils/resources'

interface MasteryItem {
  championId: number
  championLevel: number
  championPoints: number
  highestGrade?: string
  chestGranted?: boolean
  masteryMilestone?: number
  lastPlayTime?: number
}

const props = withDefaults(
  defineProps<{
    show: boolean
    masteries?: MasteryItem[]
    loading?: boolean
  }>(),
  {
    show: false,
    masteries: () => [],
    loading: false
  }
)

const emit = defineEmits<{
  'update:show': [show: boolean]
  refresh: []
}>()

const searchKeyword = ref('')
const activeSort = ref<'points' | 'level' | 'grade'>('points')
const isRefreshing = ref(false)

const sortModes = [
  { key: 'points', label: '按点数排序' },
  { key: 'level', label: '按等级排序' },
  { key: 'grade', label: '按评价排序' }
] as const

const GRADE_RANKS: Record<string, number> = {
  'S+': 15, 'S': 14, 'S-': 13,
  'A+': 12, 'A': 11, 'A-': 10,
  'B+': 9,  'B': 8,  'B-': 7,
  'C+': 6,  'C': 5,  'C-': 4,
  'D+': 3,  'D': 2,  'D-': 1,
}

const totalScore = computed(() => {
  return props.masteries.reduce((acc, m) => acc + (m.championPoints || 0), 0)
})

const filteredMasteries = computed(() => {
  let list = [...props.masteries]
  const q = searchKeyword.value.trim().toLowerCase()

  if (q) {
    list = list.filter((item) => {
      const name = getChampionName(item.championId).toLowerCase()
      const title = getChampionTitle(item.championId).toLowerCase()
      return name.includes(q) || title.includes(q) || String(item.championId).includes(q)
    })
  }

  list.sort((a, b) => {
    if (activeSort.value === 'points') {
      return (b.championPoints || 0) - (a.championPoints || 0)
    }
    if (activeSort.value === 'level') {
      if ((b.championLevel || 0) !== (a.championLevel || 0)) {
        return (b.championLevel || 0) - (a.championLevel || 0)
      }
      return (b.championPoints || 0) - (a.championPoints || 0)
    }
    if (activeSort.value === 'grade') {
      const rankA = GRADE_RANKS[a.highestGrade || ''] || 0
      const rankB = GRADE_RANKS[b.highestGrade || ''] || 0
      if (rankA !== rankB) return rankB - rankA
      return (b.championPoints || 0) - (a.championPoints || 0)
    }
    return 0
  })

  return list
})

function formatPoints(pts?: number | null): string {
  const n = typeof pts === 'number' && Number.isFinite(pts) ? pts : 0
  if (n >= 10000) {
    return `${(n / 10000).toFixed(1)}万`
  }
  return n.toLocaleString()
}

async function handleRefresh() {
  isRefreshing.value = true
  emit('refresh')
  setTimeout(() => {
    isRefreshing.value = false
  }, 500)
}
</script>
