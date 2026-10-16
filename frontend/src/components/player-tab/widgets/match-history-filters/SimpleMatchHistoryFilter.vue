<template>
  <div class="space-y-4 text-xs text-slate-200 select-none">
    <!-- 1. Win / Loss Filter -->
    <div class="space-y-1.5">
      <div class="font-bold text-slate-300 flex items-center justify-between">
        <span>对局胜负结果</span>
        <span v-if="state.winLoss !== 'all'" class="text-[10px] text-cyan-400 font-normal">已筛选</span>
      </div>
      <div class="grid grid-cols-3 gap-2">
        <button
          v-for="opt in winLossOptions"
          :key="opt.value"
          @click="state.winLoss = opt.value"
          class="py-1.5 px-3 rounded-xl border text-xs font-semibold transition active:scale-95 text-center"
          :class="state.winLoss === opt.value ? 'bg-cyan-950/80 border-cyan-500/60 text-cyan-300 shadow-sm' : 'bg-slate-900/60 hover:bg-slate-800 border-slate-800 text-slate-400 hover:text-slate-200'"
        >
          {{ opt.label }}
        </button>
      </div>
    </div>

    <!-- 2. Time Range Filter -->
    <div class="space-y-1.5">
      <div class="font-bold text-slate-300 flex items-center justify-between">
        <span>对局时间跨度</span>
        <span v-if="state.timeRange !== 'all'" class="text-[10px] text-cyan-400 font-normal">已筛选</span>
      </div>
      <n-select
        v-model:value="state.timeRange"
        :options="timeRangeOptions"
        size="small"
        class="w-full"
      />
    </div>

    <!-- 3. Position / Lane Filter -->
    <div class="space-y-1.5">
      <div class="font-bold text-slate-300 flex items-center justify-between">
        <span>出场位置 / 分路</span>
        <span v-if="state.positions.length > 0" class="text-[10px] text-cyan-400 font-normal">已选 {{ state.positions.length }} 个位置</span>
      </div>
      <div class="grid grid-cols-5 gap-1.5">
        <button
          v-for="pos in positionOptions"
          :key="pos.value"
          @click="togglePosition(pos.value)"
          class="py-1.5 px-1 rounded-xl border text-xs font-semibold transition active:scale-95 text-center flex flex-col items-center justify-center gap-0.5"
          :class="state.positions.includes(pos.value) ? 'bg-cyan-950/80 border-cyan-500/60 text-cyan-300 shadow-sm' : 'bg-slate-900/60 hover:bg-slate-800 border-slate-800 text-slate-400 hover:text-slate-200'"
        >
          <span class="text-xs">{{ pos.icon }}</span>
          <span class="text-[10px]">{{ pos.label }}</span>
        </button>
      </div>
    </div>

    <!-- 4. Champion Filter -->
    <div class="space-y-1.5">
      <div class="font-bold text-slate-300 flex items-center justify-between">
        <span>指定出场英雄</span>
        <span v-if="state.championIds.length > 0" class="text-[10px] text-cyan-400 font-normal">已选 {{ state.championIds.length }} 名英雄</span>
      </div>

      <!-- Selected Champion Chips -->
      <div v-if="state.championIds.length > 0" class="flex flex-wrap gap-1.5 pb-1">
        <div
          v-for="cid in state.championIds"
          :key="cid"
          class="pl-1 pr-2 py-0.5 rounded-lg bg-cyan-950/70 border border-cyan-500/40 text-cyan-200 text-xs flex items-center space-x-1.5 shrink-0"
        >
          <ChampionIcon :champion-id="cid" :size="20" round />
          <span class="text-[11px] font-bold">{{ getChampionName(cid) }}</span>
          <button @click="removeChampion(cid)" class="hover:text-rose-400 text-slate-400 ml-1">
            <X class="w-3 h-3" />
          </button>
        </div>
      </div>

      <!-- Champion Search Input -->
      <div class="relative">
        <Search class="w-3.5 h-3.5 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
        <input
          v-model="championKeyword"
          type="text"
          placeholder="输入英雄名称筛选添加 (如: 亚索 / 盲僧 / 劫)..."
          class="w-full pl-8 pr-8 py-1.5 rounded-xl bg-slate-950/80 border border-slate-800 focus:border-cyan-500 text-xs text-slate-200 placeholder-slate-500 outline-none transition"
        />
        <button
          v-if="championKeyword"
          @click="championKeyword = ''"
          class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Search Match Dropdown List -->
      <div
        v-if="championKeyword.trim() && matchingChampions.length > 0"
        class="max-h-36 overflow-y-auto rounded-xl bg-slate-950 border border-slate-800 p-1 space-y-0.5 shadow-lg"
      >
        <div
          v-for="item in matchingChampions"
          :key="item.id"
          @click="addChampion(item.id)"
          class="p-1 px-2 rounded-lg hover:bg-slate-800/80 cursor-pointer flex items-center justify-between transition"
        >
          <div class="flex items-center space-x-2">
            <ChampionIcon :champion-id="item.id" :size="22" round />
            <span class="text-xs font-semibold text-slate-200">{{ item.name }}</span>
            <span class="text-[10px] text-slate-400">{{ item.title }}</span>
          </div>
          <span
            v-if="state.championIds.includes(item.id)"
            class="text-[10px] text-cyan-400 font-bold"
          >
            已添加
          </span>
          <Plus v-else class="w-3.5 h-3.5 text-slate-400" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NSelect } from 'naive-ui'
import { Search, X, Plus } from 'lucide-vue-next'
import type { SimpleMatchHistoryFilterState, SimpleWinLoss, SimpleTimeRange } from './filter-state'
import ChampionIcon from '../../../match-card/widgets/ChampionIcon.vue'
import { getChampionName, getChampionTitle, CHAMPION_NAMES_MAP } from '../../../match-card/utils/resources'

const props = defineProps<{
  modelValue: SimpleMatchHistoryFilterState
}>()

const emit = defineEmits<{
  'update:modelValue': [val: SimpleMatchHistoryFilterState]
}>()

const state = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

const winLossOptions: Array<{ label: string; value: SimpleWinLoss }> = [
  { label: '全部结果', value: 'all' },
  { label: '仅限胜利', value: 'win' },
  { label: '仅限失败', value: 'loss' }
]

const timeRangeOptions: Array<{ label: string; value: SimpleTimeRange }> = [
  { label: '全部历史对局', value: 'all' },
  { label: '最近 3 小时内', value: 'last3Hours' },
  { label: '最近 12 小时内', value: 'last12Hours' },
  { label: '最近 24 小时内 (1天)', value: 'last24Hours' },
  { label: '最近 3 天内', value: 'last3Days' },
  { label: '最近 7 天内 (1周)', value: 'last7Days' },
  { label: '最近 30 天内 (1月)', value: 'last30Days' }
]

const positionOptions = [
  { label: '上路', value: 'TOP', icon: '🛡️' },
  { label: '打野', value: 'JUNGLE', icon: '🌲' },
  { label: '中路', value: 'MIDDLE', icon: '⚡' },
  { label: '下路', value: 'BOTTOM', icon: '🏹' },
  { label: '辅助', value: 'UTILITY', icon: '✨' }
]

function togglePosition(pos: string) {
  const list = [...state.value.positions]
  const idx = list.indexOf(pos)
  if (idx !== -1) {
    list.splice(idx, 1)
  } else {
    list.push(pos)
  }
  state.value.positions = list
}

const championKeyword = ref('')

const allChampionEntries = computed(() => {
  return Object.entries(CHAMPION_NAMES_MAP).map(([idStr, name]) => {
    const id = Number(idStr)
    return {
      id,
      name: String(name),
      title: String(getChampionTitle(id) || '')
    }
  })
})

const matchingChampions = computed(() => {
  const q = championKeyword.value.trim().toLowerCase()
  if (!q) return []
  return allChampionEntries.value
    .filter((c) => c.name.toLowerCase().includes(q) || c.title.toLowerCase().includes(q) || String(c.id).includes(q))
    .slice(0, 10)
})

function addChampion(cid: number) {
  if (!state.value.championIds.includes(cid)) {
    state.value.championIds = [...state.value.championIds, cid]
  }
  championKeyword.value = ''
}

function removeChampion(cid: number) {
  state.value.championIds = state.value.championIds.filter((id) => id !== cid)
}
</script>
