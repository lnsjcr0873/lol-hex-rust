<template>
  <div class="w-full h-full flex flex-col overflow-hidden bg-[#090d1a]">
    <PlayerTabs :is-active="isActive" ref="playerTabsRef">
      <template #header-actions>
        <!-- Demo Mode Toggle -->
        <button
          @click="toggleDemoMode"
          class="px-2.5 py-1.5 rounded-xl border text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 shadow-sm"
          :class="isDemoActive ? 'bg-amber-950/80 hover:bg-amber-900 border-amber-600 text-amber-200' : 'bg-slate-800/80 hover:bg-slate-700 border-slate-700 text-slate-300'"
        >
          <Gamepad2 class="w-3.5 h-3.5" />
          <span>{{ isDemoActive ? '退出演练' : '模拟演练' }}</span>
        </button>

        <!-- Refresh Button -->
        <button
          @click="handleRefreshCurrent"
          class="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 border border-slate-700 text-slate-300 hover:text-white transition active:scale-95 shadow-md flex items-center space-x-1.5 text-xs font-semibold"
          title="重新扫描并同步对局数据"
        >
          <RotateCw class="w-3.5 h-3.5" />
          <span>刷新</span>
        </button>
      </template>
    </PlayerTabs>
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, markRaw, computed } from 'vue'
import { Gamepad2, RotateCw } from 'lucide-vue-next'
import PlayerTabs from '../components/player-tab/PlayerTabs.vue'
import { usePlayerTabsStore } from '../stores/player-tabs'
import { getMockSummonerProfile, getRawMatchTimeline } from '../api/tauri'
import type { LcuOrSgpGameSummary, LcuOrSgpGameDetails } from '@shared/data-adapter/wrapper'
import type { Game, GameTimeline } from '@shared/types/league-client/match-history'

const props = withDefaults(defineProps<{
  isActive?: boolean
}>(), {
  isActive: true
})

const playerTabsRef = ref<InstanceType<typeof PlayerTabs> | null>(null)
const playerTabsStore = usePlayerTabsStore()

// Native Lossless LCU Match History Pipeline for MatchHistory host
const rawGames = ref<Game[]>([])
const detailsMap = shallowRef<Record<number, LcuOrSgpGameDetails | null>>({})

const allMatchSummaries = computed<LcuOrSgpGameSummary[]>(() => {
  return rawGames.value.map((g) => {
    return {
      gameId: g.gameId,
      source: 'lcu',
      data: g
    }
  })
})

async function handleLoadDetails(gameId: number) {
  if (!gameId || gameId <= 0) return
  if (detailsMap.value[gameId]) return

  try {
    let timeline: GameTimeline | null = null
    try {
      timeline = await getRawMatchTimeline(gameId)
    } catch (e) {
      console.warn(`[MatchHistory] getRawMatchTimeline failed for ${gameId}:`, e)
    }
    if (!timeline || !timeline.frames) {
      timeline = { frames: [] }
    }
    detailsMap.value = {
      ...detailsMap.value,
      [gameId]: {
        gameId,
        source: 'lcu',
        data: markRaw(timeline)
      }
    }
  } catch (err) {
    console.error(`[MatchHistory] Failed to load details for ${gameId}:`, err)
  }
}

const isDemoActive = computed(() => {
  return Boolean(playerTabsStore.currentTab?.isMock || playerTabsStore.currentTab?.puuid?.startsWith('mock_'))
})

function toggleDemoMode() {
  if (isDemoActive.value) {
    if (playerTabsStore.currentTabId) {
      playerTabsStore.closeTab(playerTabsStore.currentTabId)
    }
  } else {
    const mock = getMockSummonerProfile()
    playerTabsStore.openTab(mock.puuid, 'lcu', {
      gameName: mock.game_name,
      tagLine: mock.tag_line,
      profileIconId: mock.profile_icon_id,
      summonerLevel: mock.summoner_level,
      isMock: true
    })
  }
}

function handleRefreshCurrent() {
  if (playerTabsStore.currentTabId) {
    playerTabsRef.value?.refreshCurrentTab()
    playerTabsRef.value?.initSelfProfile(false)
  } else {
    playerTabsRef.value?.initSelfProfile(true)
  }
}

function handleLoadSummoner(puuid: string) {
  playerTabsStore.openTab(puuid, 'lcu', {}, true)
}

defineExpose({
  loadSummoner: handleLoadSummoner,
  refresh: handleRefreshCurrent,
  refreshCurrentTab: handleRefreshCurrent,
  rawGames,
  allMatchSummaries,
  detailsMap,
  handleLoadDetails
})
</script>
