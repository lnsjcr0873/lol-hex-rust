<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Summoner Career Profile Banner -->
    <div class="p-4 px-6 border-b border-[#162238] bg-[#070b14]/85 backdrop-blur shrink-0 z-10">
      <PlayerTabHeader
        :profile="profile"
        :loading="isLoadingMatches"
        @refresh="refreshData"
      >
        <template #actions>
          <!-- Demo Mode Badge / Toggle if applicable -->
          <div
            v-if="isDemoMode"
            class="px-2.5 py-1.5 rounded-xl border border-amber-600 bg-amber-950/80 text-amber-200 text-xs font-semibold flex items-center space-x-1.5 shadow-sm"
          >
            <Gamepad2 class="w-3.5 h-3.5" />
            <span>模拟演练</span>
          </div>

          <!-- Refresh Button -->
          <button
            @click="refreshData"
            class="p-2 rounded-xl bg-slate-800/80 hover:bg-slate-700 border border-slate-700 text-slate-300 hover:text-white transition active:scale-95 shadow-md flex items-center space-x-1.5 text-xs font-semibold"
            title="重新扫描并同步对局数据"
          >
            <RotateCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isLoadingMatches }" />
            <span>刷新</span>
          </button>
        </template>
      </PlayerTabHeader>
    </div>

    <!-- Main Content Area: Responsive Grid (Left Sidebar + Right Matches List) -->
    <div class="flex-1 overflow-y-auto p-4 lg:p-6" ref="scrollContainer">
      <div class="max-w-[1580px] mx-auto grid grid-cols-1 lg:grid-cols-[340px_minmax(0,1fr)] gap-5 items-start">
        <!-- Left Sidebar Sticky Content -->
        <div class="sticky top-0 space-y-3">
          <NormalTagBlock :privacy="profile?.privacy" :tags="tags" @delete-tag="handleDeleteTag" />
          <SummaryPane
            :analysis="analysis"
            :profile-summary="profile?.recent_summary"
            :queue="activeQueue"
            :sample-count="analysis?.count || matches?.length"
            :current-page="currentPage"
          />
          <PlayerChallenges :challenges-data="challengesData" />
          <ChampionMasteryPane
            :masteries-list="masteriesList"
            :raw-masteries-list="realMasteries"
            @refresh="reloadMasteries"
          />
          <RecentlyPlayers
            side="ally"
            :matches="matches"
            :self-puuid="profile?.puuid"
            @select-summoner="handleSelectSummoner"
          />
          <RecentlyPlayers
            side="enemy"
            :matches="matches"
            :self-puuid="profile?.puuid"
            @select-summoner="handleSelectSummoner"
          />
          <EncounteredGames
            :encountered-list="encounteredGames"
            @select-game="$emit('selectGame', $event)"
          />
        </div>

        <!-- Right Main Column: Pagination & Filters + Match Cards List -->
        <div class="flex flex-col gap-3 min-w-0">
          <!-- Queue and Pagination Filter Bar -->
          <div class="sticky top-0 z-20">
            <MatchHistoryPagination
              :queue="activeQueue"
              :page="currentPage"
              :page-size="pageSize"
              :has-next-page="hasNextPage"
              :disabled="isLoadingMatches || isCollecting"
              :has-active-filter="isFilterActive"
              @open-filters="showFilterModal = true"
              @update:queue="handleQueueChange"
              @update:page-size="handlePageSizeChange"
              @update:page="handleGoToPage"
              @prev-page="handlePrevPage"
              @next-page="handleNextPage"
              @go-to-page="handleGoToPage"
              @first-page="handleFirstPage"
            />
          </div>

          <!-- Matches Loading State: Card-level Skeleton Loaders -->
          <div v-if="isLoadingMatches" class="flex flex-col gap-3">
            <div
              v-for="idx in Math.min(pageSize || 5, 5)"
              :key="idx"
              class="h-[88px] rounded-2xl bg-slate-900/60 border border-slate-800/80 p-3.5 px-4 flex items-center justify-between animate-pulse select-none"
            >
              <!-- Left: Champion Icon & Summoner Spells Skeleton -->
              <div class="flex items-center space-x-3.5">
                <div class="w-12 h-12 rounded-xl bg-slate-800/80 shrink-0"></div>
                <div class="grid grid-cols-2 gap-1 shrink-0">
                  <div class="w-5 h-5 rounded bg-slate-800/80"></div>
                  <div class="w-5 h-5 rounded bg-slate-800/80"></div>
                  <div class="w-5 h-5 rounded bg-slate-800/80"></div>
                  <div class="w-5 h-5 rounded bg-slate-800/80"></div>
                </div>
                <div class="space-y-2 ml-1">
                  <div class="w-24 h-4 rounded bg-slate-800/80"></div>
                  <div class="w-16 h-3 rounded bg-slate-800/60"></div>
                </div>
              </div>

              <!-- Center: Items Slots Skeleton -->
              <div class="flex items-center space-x-1 shrink-0">
                <div v-for="slot in 6" :key="slot" class="w-7 h-7 rounded-lg bg-slate-800/60"></div>
                <div class="w-7 h-7 rounded-lg bg-slate-800/40 ml-1"></div>
              </div>

              <!-- Right: KDA & Score Bars Skeleton -->
              <div class="flex items-center space-x-5 shrink-0">
                <div class="space-y-1.5 text-right">
                  <div class="w-16 h-4 rounded bg-slate-800/80 ml-auto"></div>
                  <div class="w-12 h-3 rounded bg-slate-800/60 ml-auto"></div>
                </div>
                <div class="w-20 h-5 rounded-full bg-slate-800/80"></div>
              </div>
            </div>
          </div>

          <!-- Empty Matches State with Reset Button -->
          <div
            v-else-if="matchSummaries.length === 0"
            class="p-16 text-center text-xs text-slate-400 rounded-2xl bg-[#0a0f1d] border border-[#162238] flex flex-col items-center space-y-3 select-none"
          >
            <Inbox class="w-10 h-10 text-slate-600" />
            <span class="font-bold text-sm text-slate-200">
              {{ activeQueue !== 'all' ? `当前【${getQueueDisplayName(activeQueue)}】暂无对局记录` : '暂无匹配的战绩数据' }}
            </span>
            <span class="text-xs text-slate-500 max-w-xs">
              {{ activeQueue !== 'all' ? '当前模式筛选下暂无对局记录，您可以切换为全部模式或其他队列查看。' : '在客户端进行对局后，历史战绩将在此处同步收录。' }}
            </span>
            <div class="flex items-center space-x-2 mt-2">
              <button
                v-if="isFilterActive"
                @click="handleResetFilters"
                class="px-3.5 py-1.5 rounded-xl bg-cyan-950 text-cyan-300 border border-cyan-500/50 hover:bg-cyan-900 transition text-xs font-semibold active:scale-95 shadow-sm"
              >
                重置高级筛选
              </button>
              <button
                v-if="activeQueue !== 'all'"
                @click="handleQueueChange('all')"
                class="px-3.5 py-1.5 rounded-xl bg-slate-800 text-slate-300 border border-slate-700 hover:bg-slate-700 transition text-xs font-semibold active:scale-95 shadow-sm"
              >
                查看全部模式
              </button>
            </div>
          </div>

          <!-- Match Cards List with Virtual Viewport Optimization -->
          <div v-else class="flex flex-col gap-3">
            <MatchHistoryCard
              v-for="item in matchSummaries"
              :key="item.gameId"
              :summary="item"
              :details="detailsMap[item.gameId]"
              :puuid="profile?.puuid"
              :loading-details="loadingGameIds.has(item.gameId)"
              @load-details="handleLoadDetails"
              @navigate-to-summoner-by-puuid="handleSelectSummoner"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Floating Collect Mode Progress Indicator -->
    <CollectModeProgress
      :active="isCollecting"
      :current-iteration="collectCurrentIteration"
      :max-iteration="collectMaxIteration"
      :matched-count="collectedGames.length"
      :expected-count="collectExpectedCount"
      @stop="handleStopCollect"
    />

    <!-- Advanced Match History Filters Modal -->
    <MatchHistoryFilters
      v-model:show="showFilterModal"
      v-model:simple-state="simpleFilterState"
      v-model:advanced-state="advancedFilterState"
      v-model:active-mode="filterMode"
      :is-collecting="isCollecting"
      @reset="handleResetFilters"
      @start-collect="handleStartCollect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, markRaw, computed, onMounted, onUnmounted, watch } from 'vue'
import { Inbox, RotateCw, Gamepad2 } from 'lucide-vue-next'
import type { SummonerProfile, MatchHistoryItem } from '../../api/types'
import {
  getSummonerProfile,
  getRawMatchHistory,
  getRawMatchTimeline,
  getRawMatchDetail,
  getChampionMastery,
  getPlayerChallenges,
  getMatchDetail,
  getMatchTimeline,
  getMockSummonerProfile
} from '../../api/tauri'
import { getMockRawMatchHistory, getMockRawMatchTimeline } from '../../api/mockData'
import type { LcuOrSgpGameSummary, LcuOrSgpGameDetails } from '@shared/data-adapter/wrapper'
import type { Game, GameTimeline, MatchHistory, Participant, ParticipantIdentity } from '@shared/types/league-client/match-history'
import { analyzeGames } from '@shared/data-adapter/analysis/player'
import { evaluateTacticalTags } from '@shared/data-adapter/tactical-tags'
import { QUEUE_NAME_MAP } from '../match-card/utils/resources'
import PlayerTabHeader from './PlayerTabHeader.vue'
import NormalTagBlock from './NormalTagBlock.vue'
import SummaryPane from './SummaryPane.vue'
import ChampionMasteryPane from './ChampionMasteryPane.vue'
import PlayerChallenges from './widgets/PlayerChallenges.vue'
import RecentlyPlayers from './RecentlyPlayers.vue'
import MatchHistoryPagination from './widgets/MatchHistoryPagination.vue'
import { MatchHistoryCard, provideMatchHistoryCardViewport } from './widgets/match-history-card'
import { usePlayerTabsStore, type PlayerTabItem } from '../../stores/player-tabs'
import MatchHistoryFilters from './widgets/match-history-filters/MatchHistoryFilters.vue'
import CollectModeProgress from './widgets/match-history-filters/CollectModeProgress.vue'
import {
  type SimpleMatchHistoryFilterState,
  type AdvancedFilterState,
  createDefaultSimpleFilter,
  createDefaultAdvancedFilter,
  hasActiveFilter,
  buildPredicateFromSimple,
  buildPredicateFromAst
} from './widgets/match-history-filters/filter-state'
import type { Predicate } from '@shared/data-adapter/predicates/combinators'

provideMatchHistoryCardViewport({ active: true })

const props = defineProps<{
  tab: PlayerTabItem
  isActive?: boolean
  selfPuuid?: string
}>()

const playerTabsStore = usePlayerTabsStore()
const effectiveSelfPuuid = computed(() => props.selfPuuid || playerTabsStore.tabs.find(t => t.isSelf)?.puuid || '')

const emit = defineEmits<{
  selectSummoner: [puuid: string]
  selectGame: [gameId: number]
  profileUpdated: [profile: SummonerProfile]
}>()

const scrollContainer = ref<HTMLElement | null>(null)

const isDemoMode = computed(() => Boolean(props.tab.isMock || props.tab.puuid.startsWith('mock_')))
const profile = ref<SummonerProfile>({
  is_connected: !isDemoMode.value,
  puuid: props.tab.puuid,
  game_name: props.tab.gameName,
  tag_line: props.tab.tagLine,
  profile_icon_id: props.tab.profileIconId,
  summoner_level: props.tab.summonerLevel,
  tier_solo: 'UNRANKED',
  division_solo: '',
  lp_solo: 0,
  wins_solo: 0,
  losses_solo: 0,
  win_rate_solo: 0,
  tier_flex: 'UNRANKED',
  division_flex: '',
  lp_flex: 0,
  wins_flex: 0,
  losses_flex: 0,
  win_rate_flex: 0,
  recent_radar: { output: 0, damage: 0, kda: 0, join_rate: 0, survival: 0, gold: 0, vision: 0 },
  recent_summary: {
    total_games: 0,
    win_rate: 0,
    avg_kda: '0.0/0.0/0.0',
    avg_kda_ratio: 0,
    top_champions: []
  }
})

const matches = ref<MatchHistoryItem[]>([])
const rawGames = ref<Game[]>([])
const detailsMap = shallowRef<Record<number, LcuOrSgpGameDetails | null>>({})
const loadingGameIds = ref<Set<number>>(new Set())
const isLoadingMatches = ref(false)

const activeQueue = ref('all')
const currentPage = ref(1)

function getInitialPageSize(): number {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem('hexbrain_preferences') : null
    if (raw) {
      const parsed = JSON.parse(raw)
      if (typeof parsed?.defaultPageSize === 'number' && [10, 20, 50, 100].includes(parsed.defaultPageSize)) {
        return parsed.defaultPageSize
      }
    }
  } catch {
    // ignore
  }
  return 20
}

const pageSize = ref(getInitialPageSize())
const hasNextPage = ref(true)
const currentFetchId = ref(0)
const realMasteries = ref<any[]>([])
const challengesData = ref<any>(null)
const tags = ref<any[]>([])
const encounteredGames = ref<any[]>([])

const QUEUE_MAP: Record<string, number | undefined> = {
  all: undefined,
  ranked_solo: 420,
  ranked_flex: 440,
  aram: 450,
  normal: 430
}

const QUEUE_LABELS: Record<string, string> = {
  all: '全部模式',
  ranked_solo: '单双排位',
  ranked_flex: '灵活排位',
  aram: '极地大乱斗',
  normal: '匹配模式'
}

function getQueueDisplayName(q?: string) {
  if (!q) return '全部模式'
  return QUEUE_LABELS[q] || q
}

function convertRawGameToMatchItem(game: Game, selfPuuid?: string): MatchHistoryItem {
  const myIdent = game.participantIdentities?.find(i => i.player?.puuid === selfPuuid) || game.participantIdentities?.[0]
  const myPid = myIdent?.participantId ?? 1
  const myPart = (selfPuuid && (!game.participantIdentities || game.participantIdentities.length === 0))
    ? (game.participants?.find((p: any) => p.puuid === selfPuuid) || game.participants?.[0])
    : (game.participants?.find(p => p.participantId === myPid) || game.participants?.[0])
  const stats = myPart?.stats

  // Evaluate MVP and SVP across all participants in the match
  const rawParts = game.participants || []
  let bestWinPart: any = null
  let bestWinScore = -Infinity
  let bestLossPart: any = null
  let bestLossScore = -Infinity

  for (const p of rawParts) {
    const s = p.stats
    if (!s) continue
    const kda = ((s.kills || 0) + (s.assists || 0) * 0.7) / Math.max(1, s.deaths || 0)
    const dmg = s.totalDamageDealtToChampions || 0
    const cs = (s.totalMinionsKilled || 0) + (s.neutralMinionsKilled || 0)
    const score = kda * 3.0 + dmg / 5000.0 + cs / 50.0
    if (s.win) {
      if (score > bestWinScore) {
        bestWinScore = score
        bestWinPart = p
      }
    } else {
      if (score > bestLossScore) {
        bestLossScore = score
        bestLossPart = p
      }
    }
  }

  const isMvp = myPart != null && myPart === bestWinPart
  const isSvp = myPart != null && myPart === bestLossPart

  const participantsPreview = rawParts.map(p => {
    const ident = game.participantIdentities?.find(i => i.participantId === p.participantId)
    const pAny = p as any
    return {
      participant_id: p.participantId,
      puuid: pAny.puuid || ident?.player?.puuid || '',
      summoner_name: pAny.summonerName || pAny.gameName || ident?.player?.summonerName || ident?.player?.gameName || '',
      game_name: pAny.gameName || pAny.summonerName || ident?.player?.gameName || ident?.player?.summonerName || '',
      tag_line: pAny.tagLine || ident?.player?.tagLine || '',
      team_id: p.teamId,
      is_blue: p.teamId === 100,
      champion_id: p.championId,
      champion_name: '',
      champion_icon: p.championId ? `https://game.gtimg.cn/images/lol/act/img/skinloading/${p.championId}000.jpg` : 'https://game.gtimg.cn/images/lol/act/img/skinloading/142000.jpg',
      is_win: p.stats?.win ?? false,
      kills: p.stats?.kills,
      deaths: p.stats?.deaths,
      assists: p.stats?.assists
    }
  })

  return {
    game_id: game.gameId,
    game_creation: game.gameCreation,
    game_duration_seconds: game.gameDuration,
    game_mode: game.gameMode,
    queue_id: game.queueId,
    queue_name: '',
    is_win: stats?.win ?? false,
    team_id: myPart?.teamId,
    is_blue: myPart?.teamId === 100,
    champion_id: myPart?.championId ?? 0,
    champion_name: '',
    champion_icon: (myPart?.championId ?? 0) > 0 ? `https://game.gtimg.cn/images/lol/act/img/skinloading/${myPart?.championId}000.jpg` : 'https://game.gtimg.cn/images/lol/act/img/skinloading/142000.jpg',
    role: myPart?.timeline?.lane || '',
    kills: stats?.kills ?? 0,
    deaths: stats?.deaths ?? 0,
    assists: stats?.assists ?? 0,
    kda_ratio: stats ? Math.round((((stats.kills ?? 0) + (stats.assists ?? 0)) / Math.max(1, stats.deaths ?? 0)) * 10) / 10 : 0,
    is_mvp: isMvp,
    is_svp: isSvp,
    items: [stats?.item0 ?? 0, stats?.item1 ?? 0, stats?.item2 ?? 0, stats?.item3 ?? 0, stats?.item4 ?? 0, stats?.item5 ?? 0, stats?.item6 ?? 0],
    total_damage: stats?.totalDamageDealtToChampions ?? 0,
    damage_share_pct: 0,
    damage_taken: stats?.totalDamageTaken ?? 0,
    gold_earned: stats?.goldEarned ?? 0,
    minions_killed: (stats?.totalMinionsKilled ?? 0) + (stats?.neutralMinionsKilled ?? 0),
    participants_preview: participantsPreview
  }
}

const showFilterModal = ref(false)
const filterMode = ref<'simple' | 'advanced'>('simple')
const simpleFilterState = ref<SimpleMatchHistoryFilterState>(createDefaultSimpleFilter())
const advancedFilterState = ref<AdvancedFilterState>(createDefaultAdvancedFilter())

const isFilterActive = computed(() => {
  return hasActiveFilter(simpleFilterState.value, advancedFilterState.value, filterMode.value)
})

const activePredicate = computed<Predicate<LcuOrSgpGameSummary> | null>(() => {
  const selfPuuid = profile.value.puuid || props.tab.puuid
  if (filterMode.value === 'simple') {
    if (!hasActiveFilter(simpleFilterState.value, advancedFilterState.value, 'simple')) return null
    return buildPredicateFromSimple(simpleFilterState.value, selfPuuid)
  } else {
    if (!hasActiveFilter(simpleFilterState.value, advancedFilterState.value, 'advanced')) return null
    return buildPredicateFromAst(advancedFilterState.value, selfPuuid)
  }
})

// Collect mode state
const isCollecting = ref(false)
const collectCurrentIteration = ref(0)
const collectMaxIteration = ref(10)
const collectExpectedCount = ref(20)
const collectedGames = ref<Game[]>([])
const isLoadedByCollectMode = ref(false)
const collectAbortSignal = ref<{ aborted: boolean }>({ aborted: false })

function handleResetFilters() {
  simpleFilterState.value = createDefaultSimpleFilter()
  advancedFilterState.value = createDefaultAdvancedFilter()
  if (isCollecting.value) {
    handleStopCollect()
  }
  isLoadedByCollectMode.value = false
  collectedGames.value = []
  if (rawGames.value.length > 0) {
    matches.value = rawGames.value.map((g) => convertRawGameToMatchItem(g, profile.value.puuid))
  }
}

function handleStopCollect() {
  collectAbortSignal.value.aborted = true
  isCollecting.value = false
}

async function handleStartCollect(options: {
  countPerIteration: number
  expectedCount: number
  maxIteration: number
}) {
  if (isCollecting.value) return

  const countPerIteration = Math.max(1, Math.min(200, Number.isFinite(options.countPerIteration) ? Math.floor(options.countPerIteration) : 20))
  const expectedCount = Math.max(1, Math.min(500, Number.isFinite(options.expectedCount) ? Math.floor(options.expectedCount) : 50))
  const maxIteration = Math.max(1, Math.min(100, Number.isFinite(options.maxIteration) ? Math.floor(options.maxIteration) : 10))

  const pred = activePredicate.value
  isCollecting.value = true
  isLoadedByCollectMode.value = true
  collectAbortSignal.value = { aborted: false }
  collectCurrentIteration.value = 0
  collectMaxIteration.value = maxIteration
  collectExpectedCount.value = expectedCount
  collectedGames.value = []

  const selfPuuid = profile.value.puuid || props.tab.puuid
  const queueId = QUEUE_MAP[activeQueue.value]
  const seenGameIds = new Set<number>()

  try {
    while (
      collectedGames.value.length < expectedCount &&
      collectCurrentIteration.value < maxIteration &&
      !collectAbortSignal.value.aborted
    ) {
      const begIndex = collectCurrentIteration.value * countPerIteration
      const endIndex = begIndex + countPerIteration - 1

      let batchGames: Game[] = []

      if (isDemoMode.value) {
        const mockHistory = getMockRawMatchHistory(queueId, begIndex, endIndex)
        batchGames = mockHistory?.games?.games || []
      } else {
        try {
          const rawHistory = await getRawMatchHistory(selfPuuid, queueId, begIndex, endIndex)
          if (rawHistory?.games?.games) {
            batchGames = rawHistory.games.games
          }
        } catch (e) {
          console.warn('Collect mode getRawMatchHistory error:', e)
          break
        }
      }

      if (collectAbortSignal.value.aborted) break
      if (!batchGames || batchGames.length === 0) break

      const matched: Game[] = []
      for (const game of batchGames) {
        if (!game || typeof game.gameId !== 'number' || seenGameIds.has(game.gameId)) continue

        const summary: LcuOrSgpGameSummary = {
          gameId: game.gameId,
          source: 'lcu',
          data: game
        }

        if (!pred || pred(summary)) {
          seenGameIds.add(game.gameId)
          matched.push(game)
          if (collectedGames.value.length + matched.length >= expectedCount) {
            break
          }
        }
      }

      if (matched.length > 0) {
        collectedGames.value = [...collectedGames.value, ...matched]
        matches.value = collectedGames.value.map((g) => convertRawGameToMatchItem(g, selfPuuid))
      }

      collectCurrentIteration.value++

      if (batchGames.length < countPerIteration) {
        break
      }
    }
  } finally {
    isCollecting.value = false
  }
}

const rawSummaries = shallowRef<Map<number, LcuOrSgpGameSummary>>(new Map())

async function enrichCurrentGames(games: Game[], targetFetchId: number) {
  if (isDemoMode.value || !profile.value.is_connected) return

  const toEnrich = games.filter((g) => {
    const existing = rawSummaries.value.get(g.gameId)
    const existingCount = (existing?.data as Game)?.participants?.length ?? 0
    const gCount = g.participants?.length ?? 0
    return existingCount <= 1 && gCount <= 1
  })

  if (toEnrich.length === 0) return

  const CONCURRENCY = 3
  let cursor = 0

  async function worker() {
    while (cursor < toEnrich.length) {
      if (targetFetchId !== currentFetchId.value) return
      const g = toEnrich[cursor++]
      if (!g) break

      try {
        const fullDetail = await getRawMatchDetail(g.gameId)
        if (targetFetchId !== currentFetchId.value) return
        if (fullDetail && Array.isArray(fullDetail.participants) && fullDetail.participants.length > 1) {
          const next = new Map(rawSummaries.value)
          next.set(g.gameId, {
            gameId: g.gameId,
            source: 'lcu',
            data: markRaw(fullDetail)
          })
          rawSummaries.value = next

          const selfPuuid = profile.value.puuid || props.tab.puuid
          const mIdx = matches.value.findIndex((m) => m.game_id === g.gameId)
          if (mIdx !== -1) {
            matches.value[mIdx] = convertRawGameToMatchItem(fullDetail, selfPuuid)
          }
        }
      } catch (err) {
        // Non-blocking
      }
    }
  }

  await Promise.all(
    Array.from({ length: Math.min(CONCURRENCY, toEnrich.length) }, () => worker())
  )
}

const allMatchSummaries = computed<LcuOrSgpGameSummary[]>(() => {
  const currentList = isLoadedByCollectMode.value ? collectedGames.value : rawGames.value
  return currentList.map((g) => {
    const existing = rawSummaries.value.get(g.gameId)
    if (existing) return existing

    return {
      gameId: g.gameId,
      source: 'lcu',
      data: g
    }
  })
})

const filteredSummaries = computed(() => {
  let list = allMatchSummaries.value

  if (!isLoadedByCollectMode.value && activePredicate.value) {
    list = list.filter((s) => activePredicate.value!(s))
  }

  return list.filter((s) => {
    const qid = (s.data as Game).queueId
    if (activeQueue.value === 'ranked_solo' && qid !== 420) return false
    if (activeQueue.value === 'ranked_flex' && qid !== 440) return false
    if (activeQueue.value === 'aram' && qid !== 450) return false
    if (activeQueue.value === 'normal' && qid !== 430 && qid !== 400) return false
    return true
  })
})

const matchSummaries = computed(() => filteredSummaries.value)

const analysis = computed(() => {
  if (filteredSummaries.value.length === 0) return null

  const puuid = profile.value.puuid || props.tab.puuid
  const games = filteredSummaries.value.map((s) => ({
    gameId: s.gameId,
    summary: s,
    details: detailsMap.value[s.gameId] || undefined
  }))

  const res = analyzeGames(games, puuid, {
    filterPveAndNonMatchedGames: true,
    filterAbortedOrRemadeGames: true
  })

  const winsCount = filteredSummaries.value.filter((s) => {
    const game = s.data as Game
    const myIdent = game?.participantIdentities?.find((i) => i.player?.puuid === puuid)
    const myPart = myIdent
      ? game?.participants?.find((p) => p.participantId === myIdent.participantId)
      : (game?.participants?.find((p: any) => p.puuid === puuid) || game?.participants?.[0])
    return myPart?.stats?.win ?? false
  }).length
  const totalCount = filteredSummaries.value.length

  if (res) {
    return {
      ...res,
      count: totalCount,
      queue: activeQueue.value,
      sampleCount: totalCount
    }
  }

  return null
})

const masteriesList = computed(() => {
  if (realMasteries.value && realMasteries.value.length > 0) {
    return [...realMasteries.value]
      .sort((a, b) => (b.championPoints || 0) - (a.championPoints || 0))
      .slice(0, 6)
      .map((m) => ({
        championId: m.championId,
        championLevel: m.championLevel ?? 1,
        championPoints: m.championPoints ?? 0,
        highestGrade: m.highestGrade || '',
        chestGranted: m.chestGranted ?? false,
        masteryMilestone: m.masteryMilestone ?? 0
      }))
  }
  return []
})

async function fetchPageMatches(
  targetPage = currentPage.value,
  targetSize = pageSize.value,
  queue = activeQueue.value
) {
  isLoadedByCollectMode.value = false
  const fetchId = ++currentFetchId.value
  isLoadingMatches.value = true

  try {
    const queueId = QUEUE_MAP[queue]
    const begIndex = (targetPage - 1) * targetSize
    const endIndex = begIndex + targetSize - 1

    if (isDemoMode.value) {
      const mockHistory = getMockRawMatchHistory(queueId, begIndex, endIndex)
      const games = mockHistory?.games?.games || []

      if (fetchId !== currentFetchId.value) return

      rawGames.value = games
      matches.value = games.map((g) => convertRawGameToMatchItem(g, profile.value.puuid))
      hasNextPage.value = games.length >= targetSize
      currentPage.value = targetPage
      pageSize.value = targetSize
      activeQueue.value = queue
      return
    }

    const puuid = profile.value.puuid || props.tab.puuid
    let rawHistory: MatchHistory | null = null
    try {
      rawHistory = await getRawMatchHistory(puuid, queueId, begIndex, endIndex)
    } catch (e) {
      console.warn('getRawMatchHistory failed, attempting fallback:', e)
    }

    if (fetchId !== currentFetchId.value) return

    if (rawHistory && rawHistory.games && Array.isArray(rawHistory.games.games) && rawHistory.games.games.length > 0) {
      rawGames.value = rawHistory.games.games
      matches.value = rawHistory.games.games.map((g) => convertRawGameToMatchItem(g, profile.value.puuid))
      hasNextPage.value = rawHistory.games.games.length >= targetSize
      enrichCurrentGames(rawHistory.games.games, fetchId)
    } else {
      if (!profile.value.is_connected) {
        const mockHistory = getMockRawMatchHistory(queueId, begIndex, endIndex)
        const games = mockHistory?.games?.games || []
        rawGames.value = games
        matches.value = games.map((g) => convertRawGameToMatchItem(g, profile.value.puuid))
        hasNextPage.value = games.length >= targetSize
      } else {
        rawGames.value = []
        matches.value = []
        hasNextPage.value = false
      }
    }

    currentPage.value = targetPage
    pageSize.value = targetSize
    activeQueue.value = queue
  } catch (err) {
    if (fetchId === currentFetchId.value) {
      console.error('Failed to fetch match history page:', err)
      rawGames.value = []
      matches.value = []
      hasNextPage.value = false
    }
  } finally {
    if (fetchId === currentFetchId.value) {
      isLoadingMatches.value = false
    }
  }
}

function handleQueueChange(q: string) {
  fetchPageMatches(1, pageSize.value, q)
}

function handlePageSizeChange(size: number) {
  fetchPageMatches(1, size, activeQueue.value)
}

function handlePrevPage() {
  if (currentPage.value > 1) {
    fetchPageMatches(currentPage.value - 1, pageSize.value, activeQueue.value)
  }
}

function handleNextPage() {
  if (hasNextPage.value) {
    fetchPageMatches(currentPage.value + 1, pageSize.value, activeQueue.value)
  }
}

function handleGoToPage(targetPage: number) {
  if (targetPage >= 1) {
    fetchPageMatches(targetPage, pageSize.value, activeQueue.value)
  }
}

function handleFirstPage() {
  fetchPageMatches(1, pageSize.value, activeQueue.value)
}

function setGameDetails(gameId: number, details: LcuOrSgpGameDetails) {
  const current = { ...detailsMap.value }
  current[gameId] = details
  const keys = Object.keys(current)
  if (keys.length > 80) {
    for (let i = 0; i < keys.length - 80; i++) {
      delete current[Number(keys[i])]
    }
  }
  detailsMap.value = current
}

async function handleLoadDetails(gameId: number) {
  if (loadingGameIds.value.has(gameId)) return
  loadingGameIds.value.add(gameId)

  try {
    if (isDemoMode.value) {
      const mockTimeline = getMockRawMatchTimeline(gameId)
      setGameDetails(gameId, {
        gameId,
        source: 'lcu',
        data: markRaw(mockTimeline)
      })
    } else {
      try {
        const detail = await getMatchDetail(gameId)
        if (detail?.raw_game) {
          const next = new Map(rawSummaries.value)
          next.set(gameId, {
            gameId,
            source: 'lcu',
            data: markRaw(detail.raw_game)
          })
          rawSummaries.value = next
        }
      } catch (e) {
        // Non-blocking
      }

      let timeline: GameTimeline | null = null
      try {
        timeline = await getRawMatchTimeline(gameId)
      } catch (e) {
        console.warn(`getRawMatchTimeline failed for ${gameId}, attempting getMatchTimeline fallback:`, e)
      }

      if (!timeline || !timeline.frames || timeline.frames.length === 0) {
        const legacyTimeline = await getMatchTimeline(gameId)
        if (legacyTimeline && legacyTimeline.frames) {
          timeline = legacyTimeline as GameTimeline
        }
      }

      if (!timeline || !timeline.frames) {
        timeline = { frames: [] }
      }

      setGameDetails(gameId, {
        gameId,
        source: 'lcu',
        data: markRaw(timeline)
      })
    }
  } catch (err) {
    console.error(`Failed to load details for match ${gameId}:`, err)
    setGameDetails(gameId, {
      gameId,
      source: 'lcu',
      data: markRaw({ frames: [] })
    })
  } finally {
    loadingGameIds.value.delete(gameId)
  }
}

function handleSelectSummoner(puuid: string) {
  emit('selectSummoner', puuid)
}

async function reloadMasteries() {
  try {
    const puuid = profile.value.puuid || props.tab.puuid
    realMasteries.value = (await getChampionMastery(puuid)) || []
  } catch (e) {
    console.warn('Failed to reload masteries:', e)
  }
}

async function loadData() {
  isLoadingMatches.value = true
  try {
    if (isDemoMode.value) {
      profile.value = getMockSummonerProfile()
      realMasteries.value = (await getChampionMastery(profile.value.puuid)) || []
      challengesData.value = {
        playerChallenges: [
          { id: 505001, currentValue: 168 },
          { id: 510001, currentValue: 145 },
          { id: 510011, currentValue: 38 },
          { id: 504003, currentValue: 12 },
          { id: 504002, currentValue: 88 },
          { id: 504004, currentValue: 45 }
        ]
      }
    } else {
      const p = await getSummonerProfile(props.tab.puuid)
      if (p) {
        profile.value = p
        emit('profileUpdated', p)
        realMasteries.value = (await getChampionMastery(p.puuid)) || []
      }
      try {
        const cData = await getPlayerChallenges(props.tab.puuid)
        challengesData.value = cData?.playerChallenges ? cData : { playerChallenges: [] }
      } catch (e) {
        console.warn('Failed to load player challenges:', e)
        challengesData.value = { playerChallenges: [] }
      }
    }
    await fetchPageMatches(1, pageSize.value, activeQueue.value)
    updateEncounteredGames()
    loadPlayerTags()
  } catch (err) {
    console.error('loadData error for tab:', props.tab.id, err)
  } finally {
    isLoadingMatches.value = false
  }
}

function loadPlayerTags() {
  const targetPuuid = profile.value.puuid || props.tab.puuid
  if (!targetPuuid) {
    tags.value = []
    return
  }

  const customTags: Array<{ tag: string; isCustom?: boolean }> = []
  try {
    const key = `hexbrain_player_tags_${targetPuuid}`
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(key) : null
    if (raw) {
      const parsed = JSON.parse(raw)
      if (Array.isArray(parsed)) {
        for (const item of parsed) {
          if (item?.tag) customTags.push({ tag: item.tag, isCustom: true })
        }
      }
    }
  } catch {
    // ignore
  }

  const tacticalList: Array<{ tag: string }> = []
  if (analysis.value) {
    try {
      const evaluated = evaluateTacticalTags({
        puuid: targetPuuid,
        selfPuuid: effectiveSelfPuuid.value,
        analysis: analysis.value,
        encounteredCount: encounteredGames.value.length
      })
      for (const t of evaluated.slice(0, 3)) {
        tacticalList.push({ tag: `${t.label}: ${t.description || ''}` })
      }
    } catch (e) {
      console.warn('Failed to evaluate tactical tags:', e)
    }
  }

  tags.value = [...customTags, ...tacticalList]
}

function handleDeleteTag(idx: number) {
  const targetPuuid = profile.value.puuid || props.tab.puuid
  tags.value.splice(idx, 1)
  if (targetPuuid) {
    try {
      const remainingCustom = tags.value.filter((t: any) => t.isCustom)
      localStorage.setItem(`hexbrain_player_tags_${targetPuuid}`, JSON.stringify(remainingCustom))
    } catch {
      // ignore
    }
  }
}

function updateEncounteredGames() {
  if (isDemoMode.value) {
    encounteredGames.value = [
      {
        gameId: 999901,
        queueName: '排位赛(单双)',
        date: '04-15 19:30',
        isOpponent: true,
        selfChampionId: 103,
        selfWin: true,
        selfKda: '12/2/8',
        targetChampionId: 157,
        targetWin: false,
        targetKda: '3/8/4'
      },
      {
        gameId: 999902,
        queueName: '极地大乱斗',
        date: '04-14 21:10',
        isOpponent: false,
        selfChampionId: 222,
        selfWin: true,
        selfKda: '18/4/15',
        targetChampionId: 81,
        targetWin: true,
        targetKda: '14/5/12'
      }
    ]
    return
  }

  const selfPuuid = effectiveSelfPuuid.value
  const targetPuuid = profile.value.puuid || props.tab.puuid
  if (!selfPuuid || !targetPuuid || selfPuuid === targetPuuid || !rawGames.value || rawGames.value.length === 0) {
    encounteredGames.value = []
    return
  }

  const result: any[] = []
  for (const g of rawGames.value) {
    const sIdent = g.participantIdentities?.find(i => i.player?.puuid === selfPuuid)
    const tIdent = g.participantIdentities?.find(i => i.player?.puuid === targetPuuid)
    if (sIdent && tIdent) {
      const sPart = g.participants?.find(p => p.participantId === sIdent.participantId)
      const tPart = g.participants?.find(p => p.participantId === tIdent.participantId)
      if (sPart && tPart) {
        const isOpponent = (sPart as any).teamId !== (tPart as any).teamId
        const sStats = sPart.stats || ({} as any)
        const tStats = tPart.stats || ({} as any)
        const dateStr = g.gameCreation
          ? new Date(g.gameCreation).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
          : ''
        result.push({
          gameId: g.gameId,
          queueName: QUEUE_NAME_MAP[g.queueId] || '匹配对局',
          date: dateStr,
          isOpponent,
          selfChampionId: sPart.championId,
          selfWin: Boolean(sStats.win),
          selfKda: `${sStats.kills ?? 0}/${sStats.deaths ?? 0}/${sStats.assists ?? 0}`,
          targetChampionId: tPart.championId,
          targetWin: Boolean(tStats.win),
          targetKda: `${tStats.kills ?? 0}/${tStats.deaths ?? 0}/${tStats.assists ?? 0}`
        })
      }
    }
  }
  encounteredGames.value = result
}

watch([() => rawGames.value, () => analysis.value], () => {
  updateEncounteredGames()
  loadPlayerTags()
})

function refreshData() {
  loadData()
}

onMounted(() => {
  loadData()
})

onUnmounted(() => {
  handleStopCollect()
})

defineExpose({
  refresh: refreshData
})
</script>
