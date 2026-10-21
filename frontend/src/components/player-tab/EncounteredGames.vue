<template>
  <div
    class="rounded-xl bg-slate-900/80 border border-slate-800 p-3.5 text-xs text-slate-300 select-none shadow-sm"
    v-if="displayedTotal > 0 && !isSelfTab"
  >
    <!-- Header Section -->
    <div class="mb-2.5 flex items-center justify-between">
      <div class="flex items-center space-x-1.5">
        <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
        <span class="text-sm font-extrabold text-slate-100">遭遇历史对局</span>
        <span class="text-[10px] text-slate-400 font-mono">({{ displayedTotal }})</span>
      </div>

      <!-- Pagination Controls -->
      <div class="flex items-center space-x-1" v-if="totalPages > 1">
        <button
          @click="loadPage(currentPage - 1)"
          :disabled="currentPage === 1 || isLoading"
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
        >
          <ChevronLeft class="w-3.5 h-3.5" />
        </button>
        <span class="text-[10px] text-slate-400 font-mono px-1">
          {{ currentPage }} / {{ totalPages }}
        </span>
        <button
          @click="loadPage(currentPage + 1)"
          :disabled="currentPage === totalPages || isLoading"
          class="p-1 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:cursor-not-allowed text-slate-300 transition"
        >
          <ChevronRight class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Games List -->
    <div class="flex flex-col gap-2">
      <div
        v-for="item in games"
        :key="item.recordId"
        class="p-2 rounded-lg bg-slate-950/60 border border-slate-800/80 hover:border-slate-700 hover:bg-slate-800/40 transition cursor-pointer flex items-center justify-between gap-2"
        @click="$emit('selectGame', item.gameId)"
      >
        <div class="flex flex-1 flex-col gap-1.5 min-w-0">
          <!-- Line 1: Type (Ally/Enemy), Queue, Date -->
          <div class="flex items-center justify-between text-[11px]" v-if="item.game">
            <div class="flex items-center space-x-1.5 truncate">
              <span
                class="px-1 py-0.2 rounded font-bold text-[9px] shrink-0"
                :class="item.game.isOpponent ? 'bg-red-950/80 text-red-300 border border-red-800/60' : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'"
              >
                {{ item.game.isOpponent ? '对手' : '队友' }}
              </span>
              <span class="font-bold text-slate-200 truncate">{{ item.game.queueName }}</span>
            </div>
            <span class="text-slate-500 font-mono text-[10px] shrink-0">{{ item.game.date }}</span>
          </div>
          <div v-else class="h-4 w-32 bg-slate-800/60 rounded animate-pulse"></div>

          <!-- Line 2: Player 1 (Self) vs Player 2 (Target) -->
          <div v-if="item.game" class="flex items-center justify-between pt-1 border-t border-slate-800/60">
            <!-- Player 1 (Self) -->
            <div class="flex items-center space-x-1.5">
              <ChampionIcon :champion-id="item.game.selfChampionId" :size="20" round />
              <span
                class="font-bold text-[10px]"
                :class="item.game.selfWin ? 'text-emerald-400' : 'text-rose-400'"
              >
                {{ item.game.selfWin ? '胜' : '负' }}
              </span>
              <span class="text-[10px] text-slate-400 font-mono">{{ item.game.selfKda }}</span>
            </div>

            <span class="text-[9px] text-slate-600 font-mono">VS</span>

            <!-- Player 2 (Encountered) -->
            <div class="flex items-center space-x-1.5">
              <ChampionIcon :champion-id="item.game.targetChampionId" :size="20" round />
              <span
                class="font-bold text-[10px]"
                :class="item.game.targetWin ? 'text-emerald-400' : 'text-rose-400'"
              >
                {{ item.game.targetWin ? '胜' : '负' }}
              </span>
              <span class="text-[10px] text-slate-400 font-mono">{{ item.game.targetKda }}</span>
            </div>
          </div>
          <div v-else class="h-5 w-full bg-slate-800/40 rounded animate-pulse"></div>
        </div>

        <!-- Delete Action -->
        <NPopconfirm
          v-if="item.recordId"
          positive-text="确认"
          negative-text="取消"
          :positive-button-props="{ type: 'warning', size: 'tiny' }"
          :negative-button-props="{ size: 'tiny' }"
          @positive-click="handleDelete(item.recordId, $event)"
        >
          <template #trigger>
            <button
              class="p-1 rounded hover:bg-slate-800 text-slate-500 hover:text-red-400 transition shrink-0"
              @click.stop
              title="删除该条相遇记录"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </template>
          确认从本地数据库中移除该条相遇记录？
        </NPopconfirm>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { ChevronLeft, ChevronRight, Trash2 } from 'lucide-vue-next'
import { NPopconfirm } from 'naive-ui'
import ChampionIcon from '../match-card/widgets/ChampionIcon.vue'
import { queryEncounteredGames, deleteEncounteredGame, getRawMatchDetail, getMatchDetail } from '../../api/tauri'
import { toBasicInfo } from '../../shared/data-adapter/match-history/match-basic'
import { toParticipants } from '../../shared/data-adapter/match-history/participants'
import { QUEUE_NAME_MAP } from '../match-card/utils/resources'
import type { PaginatedEncounteredGames, EncounteredGameRecord } from '../../api/types'

const props = withDefaults(
  defineProps<{
    puuid?: string
    selfPuuid?: string
    isSelfTab?: boolean
    pageSize?: number
    encounteredList?: any[]
  }>(),
  {
    pageSize: 3,
    isSelfTab: false
  }
)

const emit = defineEmits<{
  selectGame: [gameId: number]
}>()

const currentPage = ref(1)
const isLoading = ref(false)
const pagedData = ref<PaginatedEncounteredGames | null>(null)
const gameMap = reactive<Record<number, any>>({})

const displayedTotal = computed(() => {
  if (pagedData.value && pagedData.value.total > 0) {
    return pagedData.value.total
  }
  if (props.encounteredList) {
    return props.encounteredList.length
  }
  return 0
})

const totalPages = computed(() => {
  if (pagedData.value && pagedData.value.pageSize > 0) {
    return Math.max(1, Math.ceil(pagedData.value.total / pagedData.value.pageSize))
  }
  if (props.encounteredList && props.encounteredList.length > 0) {
    return Math.max(1, Math.ceil(props.encounteredList.length / props.pageSize))
  }
  return 1
})

async function fetchGameSummary(gameId: number) {
  if (gameMap[gameId]) return gameMap[gameId]
  try {
    const raw = await getRawMatchDetail(gameId)
    if (raw) {
      gameMap[gameId] = raw.raw_game || raw
      return gameMap[gameId]
    }
    const parsed = await getMatchDetail(gameId)
    if (parsed) {
      gameMap[gameId] = parsed.raw_game || parsed
      return gameMap[gameId]
    }
  } catch (err) {
    console.warn(`[EncounteredGames] fetchGameSummary failed for ${gameId}:`, err)
  }
  return null
}

async function loadPage(page = 1) {
  if (!props.puuid || props.isSelfTab) {
    pagedData.value = null
    return
  }

  isLoading.value = true
  currentPage.value = page

  try {
    const res = await queryEncounteredGames({
      puuid: props.puuid,
      selfPuuid: props.selfPuuid,
      page,
      pageSize: props.pageSize,
      timeOrder: 'desc'
    })

    pagedData.value = res

    if (res?.data?.length) {
      await Promise.all((res.data || []).map((r) => fetchGameSummary(r.gameId)))
    }
  } catch (err) {
    console.error('[EncounteredGames] Failed to query encountered games:', err)
  } finally {
    isLoading.value = false
  }
}

async function handleDelete(recordId: number, e?: Event) {
  if (e) e.stopPropagation()
  try {
    await deleteEncounteredGame(recordId)
    await loadPage(currentPage.value)
  } catch (err) {
    console.error('[EncounteredGames] Failed to delete encounter record:', err)
  }
}

const games = computed(() => {
  if (!pagedData.value || pagedData.value.total === 0) {
    if (props.encounteredList && props.encounteredList.length > 0) {
      const start = (currentPage.value - 1) * props.pageSize
      const slice = props.encounteredList.slice(start, start + props.pageSize)
      return slice.map((g) => ({
        recordId: g.gameId,
        gameId: g.gameId,
        game: g
      }))
    }
    return []
  }

  return (pagedData.value.data || []).map((r: EncounteredGameRecord) => {
    const g = gameMap[r.gameId]
    if (!g) {
      return { recordId: r.id, gameId: r.gameId, game: null }
    }

    try {
      const basic = toBasicInfo({ gameId: r.gameId, source: 'lcu', data: g })
      const participants = toParticipants({ gameId: r.gameId, source: 'lcu', data: g }, basic)
      const p1 = participants.find((p) => p.puuid === r.selfPuuid)
      const p2 = participants.find((p) => p.puuid === r.puuid)

      if (!p1 || !p2) {
        return { recordId: r.id, gameId: r.gameId, game: null }
      }

      const isOpponent = p1.teamIdentifier !== p2.teamIdentifier
      const dateStr = basic.gameCreation
        ? new Date(basic.gameCreation).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
        : ''

      return {
        recordId: r.id,
        gameId: r.gameId,
        game: {
          queueName: QUEUE_NAME_MAP[basic.queueId] || '对局',
          date: dateStr,
          isOpponent,
          selfChampionId: p1.championId,
          selfWin: p1.win,
          selfKda: `${p1.kills}/${p1.deaths}/${p1.assists}`,
          targetChampionId: p2.championId,
          targetWin: p2.win,
          targetKda: `${p2.kills}/${p2.deaths}/${p2.assists}`
        }
      }
    } catch {
      return { recordId: r.id, gameId: r.gameId, game: null }
    }
  })
})

watch(
  [() => props.puuid, () => props.selfPuuid, () => props.isSelfTab],
  ([puuid, _selfPuuid, isSelfTab]) => {
    if (!isSelfTab && puuid) {
      loadPage(1)
    } else {
      pagedData.value = null
    }
  },
  { immediate: true }
)
</script>
