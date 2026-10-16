<template>
  <div class="rounded-xl bg-slate-900/80 border border-slate-800 p-3.5 text-xs text-slate-300 select-none shadow-sm" v-if="games.length > 0">
    <!-- Header Section -->
    <div class="mb-2.5 flex items-center justify-between">
      <div class="flex items-center space-x-1.5">
        <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
        <span class="text-sm font-extrabold text-slate-100">遭遇历史对局</span>
        <span class="text-[10px] text-slate-400 font-mono">({{ games.length }})</span>
      </div>

      <!-- Pagination -->
      <div class="flex items-center space-x-1" v-if="totalPages > 1">
        <button
          @click="currentPage = Math.max(1, currentPage - 1)"
          :disabled="currentPage === 1"
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
        >
          <ChevronLeft class="w-3.5 h-3.5" />
        </button>
        <span class="text-[10px] text-slate-400 font-mono px-1">
          {{ currentPage }} / {{ totalPages }}
        </span>
        <button
          @click="currentPage = Math.min(totalPages, currentPage + 1)"
          :disabled="currentPage === totalPages"
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
        >
          <ChevronRight class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Games list -->
    <div class="flex flex-col gap-2">
      <div
        v-for="item in pagedGames"
        :key="item.gameId"
        class="p-2 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 hover:bg-slate-800/40 transition cursor-pointer flex flex-col gap-1.5"
        @click="$emit('selectGame', item.gameId)"
      >
        <!-- Line 1: Type (Ally/Enemy), Queue, Date -->
        <div class="flex items-center justify-between text-[11px]">
          <div class="flex items-center space-x-1.5">
            <span
              class="px-1 py-0.2 rounded font-bold text-[9px]"
              :class="item.isOpponent ? 'bg-red-950/80 text-red-300 border border-red-800/60' : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'"
            >
              {{ item.isOpponent ? '对手' : '队友' }}
            </span>
            <span class="font-bold text-slate-200">{{ item.queueName }}</span>
          </div>
          <span class="text-slate-500 font-mono text-[10px]">{{ item.date }}</span>
        </div>

        <!-- Line 2: Player 1 (Self) vs Player 2 (Target) -->
        <div class="flex items-center justify-between pt-1 border-t border-slate-800/60">
          <!-- Player 1 (Self) -->
          <div class="flex items-center space-x-1.5">
            <ChampionIcon :champion-id="item.selfChampionId" :size="22" round />
            <span
              class="font-bold text-[10px]"
              :class="item.selfWin ? 'text-emerald-400' : 'text-rose-400'"
            >
              {{ item.selfWin ? '胜' : '负' }}
            </span>
            <span class="text-[10px] text-slate-400 font-mono">{{ item.selfKda }}</span>
          </div>

          <span class="text-[9px] text-slate-600 font-mono">VS</span>

          <!-- Player 2 (Encountered) -->
          <div class="flex items-center space-x-1.5">
            <ChampionIcon :champion-id="item.targetChampionId" :size="22" round />
            <span
              class="font-bold text-[10px]"
              :class="item.targetWin ? 'text-emerald-400' : 'text-rose-400'"
            >
              {{ item.targetWin ? '胜' : '负' }}
            </span>
            <span class="text-[10px] text-slate-400 font-mono">{{ item.targetKda }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ChevronLeft, ChevronRight } from 'lucide-vue-next'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'

interface EncounteredGameRecord {
  gameId: number
  queueName: string
  date: string
  isOpponent: boolean
  selfChampionId: number
  selfWin: boolean
  selfKda: string
  targetChampionId: number
  targetWin: boolean
  targetKda: string
}

const props = defineProps<{
  encounteredList?: EncounteredGameRecord[]
}>()

defineEmits<{
  selectGame: [gameId: number]
}>()

const currentPage = ref(1)
const pageSize = 3

const games = computed<EncounteredGameRecord[]>(() => {
  return props.encounteredList || []
})

const totalPages = computed(() => Math.ceil(games.value.length / pageSize))

const pagedGames = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return games.value.slice(start, start + pageSize)
})
</script>
