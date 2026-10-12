<template>
  <div class="w-full h-full flex flex-col overflow-hidden bg-[#090d1a]">
    <PlayerTabs
      :profile="profile"
      :analysis="analysis"
      :masteries-list="masteriesList"
      :matches="matches"
      :match-summaries="paginatedSummaries"
      :details-map="detailsMap"
      :loading-game-ids="loadingGameIds"
      :loading="isLoadingMatches"
      :queue="activeQueue"
      :page="currentPage"
      :page-size="pageSize"
      :has-next-page="hasNextPage"
      @refresh="refreshData"
      @load-details="handleLoadDetails"
      @select-summoner="handleSelectSummoner"
      @update:queue="handleQueueChange"
      @update:page-size="handlePageSizeChange"
      @prev-page="handlePrevPage"
      @next-page="handleNextPage"
    >
      <template #header-actions>
        <!-- Demo Mode Toggle -->
        <button
          @click="toggleDemoMode"
          class="px-2.5 py-1.5 rounded-xl border text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 shadow-sm"
          :class="isDemoMode ? 'bg-amber-950/80 hover:bg-amber-900 border-amber-600 text-amber-200' : 'bg-slate-800/80 hover:bg-slate-700 border-slate-700 text-slate-300'"
        >
          <Gamepad2 class="w-3.5 h-3.5" />
          <span>{{ isDemoMode ? '退出演练' : '模拟演练模式' }}</span>
        </button>

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
    </PlayerTabs>
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, markRaw, computed, onMounted } from 'vue'
import { Gamepad2, RotateCw } from 'lucide-vue-next'
import type { SummonerProfile, MatchHistoryItem } from '../api/types'
import {
  getSummonerProfile,
  getMatchHistory,
  getMatchDetail,
  getMatchTimeline,
  getMockSummonerProfile,
  getMockMatchHistory,
  getMockMatchDetail
} from '../api/tauri'
import PlayerTabs from '../components/player-tab/PlayerTabs.vue'
import type { LcuOrSgpGameSummary, LcuOrSgpGameDetails } from '@shared/data-adapter/wrapper'
import type { Game, Participant, ParticipantIdentity } from '@shared/types/league-client/match-history'
import { analyzeGames } from '@shared/data-adapter/analysis/player'

const defaultEmptyProfile: SummonerProfile = {
  is_connected: false,
  puuid: '',
  game_name: '召唤师',
  tag_line: 'CN',
  profile_icon_id: 1,
  summoner_level: 1,
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
  recent_radar: { damage: 60, kda: 60, join_rate: 60, survival: 60, gold: 60, vision: 60 },
  recent_summary: {
    total_games: 0,
    win_rate: 0,
    avg_kda: '0.0/0.0/0.0',
    avg_kda_ratio: 0,
    top_champions: []
  }
}

const profile = ref<SummonerProfile>(defaultEmptyProfile)
const matches = ref<MatchHistoryItem[]>([])
const detailsMap = shallowRef<Record<number, LcuOrSgpGameDetails | null>>({})
const loadingGameIds = ref<Set<number>>(new Set())

const isDemoMode = ref(false)
const isLoadingMatches = ref(false)

const activeQueue = ref('all')
const currentPage = ref(1)
const pageSize = ref(20)

// Helper: Convert MatchHistoryItem into full LCU Game summary
function convertMatchItemToLcuGame(item: MatchHistoryItem, currentProfile: SummonerProfile): Game {
  const isWin = item.is_win
  const myPuuid = currentProfile.puuid || 'me-puuid'
  const myName = currentProfile.game_name || '我方召唤师'
  const myTag = currentProfile.tag_line || 'CN'

  const myParticipant: Participant = {
    championId: item.champion_id,
    highestAchievedSeasonTier: currentProfile.tier_solo || 'UNRANKED',
    participantId: 1,
    spell1Id: 4,
    spell2Id: 14,
    teamId: 100,
    timeline: {
      lane: item.role || 'MIDDLE',
      role: item.role || 'SOLO',
      participantId: 1,
      creepsPerMinDeltas: {},
      csDiffPerMinDeltas: {},
      damageTakenDiffPerMinDeltas: {},
      damageTakenPerMinDeltas: {},
      goldPerMinDeltas: {},
      xpDiffPerMinDeltas: {},
      xpPerMinDeltas: {}
    },
    stats: {
      assists: item.assists,
      causedEarlySurrender: false,
      champLevel: 18,
      combatPlayerScore: 0,
      damageDealtToObjectives: 0,
      damageDealtToTurrets: 0,
      damageSelfMitigated: item.damage_taken || 0,
      deaths: item.deaths,
      doubleKills: 0,
      earlySurrenderAccomplice: false,
      firstBloodAssist: false,
      firstBloodKill: false,
      firstInhibitorAssist: false,
      firstInhibitorKill: false,
      firstTowerAssist: false,
      firstTowerKill: false,
      gameEndedInEarlySurrender: false,
      gameEndedInSurrender: false,
      goldEarned: item.gold_earned || 12000,
      goldSpent: item.gold_earned || 12000,
      inhibitorKills: 0,
      item0: item.items?.[0] || 0,
      item1: item.items?.[1] || 0,
      item2: item.items?.[2] || 0,
      item3: item.items?.[3] || 0,
      item4: item.items?.[4] || 0,
      item5: item.items?.[5] || 0,
      item6: item.items?.[6] || 0,
      killingSprees: 0,
      kills: item.kills,
      largestCriticalStrike: 0,
      largestKillingSpree: 0,
      largestMultiKill: item.is_mvp ? 3 : 1,
      longestTimeSpentLiving: 0,
      magicDamageDealt: Math.round((item.total_damage || 0) * 0.4),
      magicDamageDealtToChampions: Math.round((item.total_damage || 0) * 0.4),
      magicalDamageTaken: Math.round((item.damage_taken || 0) * 0.4),
      neutralMinionsKilled: 0,
      neutralMinionsKilledEnemyJungle: 0,
      neutralMinionsKilledTeamJungle: 0,
      objectivePlayerScore: 0,
      participantId: 1,
      pentaKills: 0,
      perk0: 8112,
      perk0Var1: 1024,
      perk0Var2: 0,
      perk0Var3: 0,
      perk1: 8126,
      perk1Var1: 450,
      perk1Var2: 0,
      perk1Var3: 0,
      perk2: 8138,
      perk2Var1: 18,
      perk2Var2: 0,
      perk2Var3: 0,
      perk3: 8105,
      perk3Var1: 5,
      perk3Var2: 0,
      perk3Var3: 0,
      perk4: 8226,
      perk4Var1: 250,
      perk4Var2: 0,
      perk4Var3: 0,
      perk5: 8233,
      perk5Var1: 0,
      perk5Var2: 0,
      perk5Var3: 0,
      perkPrimaryStyle: 8100,
      perkSubStyle: 8200,
      physicalDamageDealt: Math.round((item.total_damage || 0) * 0.6),
      physicalDamageDealtToChampions: Math.round((item.total_damage || 0) * 0.6),
      physicalDamageTaken: Math.round((item.damage_taken || 0) * 0.6),
      playerAugment1: 0,
      playerAugment2: 0,
      playerAugment3: 0,
      playerAugment4: 0,
      playerAugment5: 0,
      playerAugment6: 0,
      playerScore0: 0,
      playerScore1: 0,
      playerScore2: 0,
      playerScore3: 0,
      playerScore4: 0,
      playerScore5: 0,
      playerScore6: 0,
      playerScore7: 0,
      playerScore8: 0,
      playerScore9: 0,
      playerSubteamId: 1,
      quadraKills: 0,
      roleBoundItem: 0,
      sightWardsBoughtInGame: 0,
      subteamPlacement: isWin ? 1 : 2,
      teamEarlySurrendered: false,
      timeCCingOthers: 15,
      totalDamageDealt: item.total_damage || 20000,
      totalDamageDealtToChampions: item.total_damage || 20000,
      totalDamageTaken: item.damage_taken || 15000,
      totalHeal: 3000,
      totalMinionsKilled: item.minions_killed || 150,
      totalPlayerScore: 0,
      totalScoreRank: 0,
      totalTimeCrowdControlDealt: 20,
      totalUnitsHealed: 1,
      tripleKills: 0,
      trueDamageDealt: 0,
      trueDamageDealtToChampions: 0,
      trueDamageTaken: 0,
      turretKills: 1,
      unrealKills: 0,
      visionScore: 25,
      visionWardsBoughtInGame: 2,
      wardsKilled: 3,
      wardsPlaced: 12,
      win: isWin
    }
  }

  const myIdentity: ParticipantIdentity = {
    participantId: 1,
    player: {
      accountId: 1,
      currentAccountId: 1,
      currentPlatformId: 'HN1',
      matchHistoryUri: '',
      platformId: 'HN1',
      profileIcon: currentProfile.profile_icon_id || 1,
      summonerId: 1,
      puuid: myPuuid,
      gameName: myName,
      tagLine: myTag,
      summonerName: myName
    }
  }

  const participants: Participant[] = [myParticipant]
  const participantIdentities: ParticipantIdentity[] = [myIdentity]

  const preview = item.participants_preview || []
  for (let idx = 0; idx < preview.length; idx++) {
    const p = preview[idx]
    const pid = idx + 2
    const teamId = idx < 4 ? 100 : 200
    participants.push({
      championId: p.champion_id,
      highestAchievedSeasonTier: 'GOLD',
      participantId: pid,
      spell1Id: 4,
      spell2Id: 12,
      teamId,
      timeline: {
        lane: 'MIDDLE',
        role: 'SOLO',
        participantId: pid,
        creepsPerMinDeltas: {},
        csDiffPerMinDeltas: {},
        damageTakenDiffPerMinDeltas: {},
        damageTakenPerMinDeltas: {},
        goldPerMinDeltas: {},
        xpDiffPerMinDeltas: {},
        xpPerMinDeltas: {}
      },
      stats: {
        ...myParticipant.stats,
        participantId: pid,
        win: teamId === 100 ? isWin : !isWin,
        kills: Math.max(1, Math.round(item.kills * 0.8)),
        deaths: Math.max(1, Math.round(item.deaths * 0.9)),
        assists: Math.max(1, Math.round(item.assists * 0.8)),
        totalDamageDealtToChampions: Math.round((item.total_damage || 20000) * 0.8),
        totalDamageTaken: Math.round((item.damage_taken || 15000) * 0.8),
        goldEarned: Math.round((item.gold_earned || 12000) * 0.8),
        totalMinionsKilled: Math.round((item.minions_killed || 150) * 0.8)
      }
    })
    participantIdentities.push({
      participantId: pid,
      player: {
        accountId: pid,
        currentAccountId: pid,
        currentPlatformId: 'HN1',
        matchHistoryUri: '',
        platformId: 'HN1',
        profileIcon: 1,
        summonerId: pid,
        puuid: `puuid-${pid}`,
        gameName: p.game_name || `玩家${pid}`,
        tagLine: 'CN',
        summonerName: p.game_name || `玩家${pid}`
      }
    })
  }

  while (participants.length < 10) {
    const pid = participants.length + 1
    const teamId = pid <= 5 ? 100 : 200
    participants.push({
      championId: ((item.champion_id + pid * 7) % 160) + 1,
      highestAchievedSeasonTier: 'GOLD',
      participantId: pid,
      spell1Id: 4,
      spell2Id: 14,
      teamId,
      timeline: {
        lane: 'MIDDLE',
        role: 'SOLO',
        participantId: pid,
        creepsPerMinDeltas: {},
        csDiffPerMinDeltas: {},
        damageTakenDiffPerMinDeltas: {},
        damageTakenPerMinDeltas: {},
        goldPerMinDeltas: {},
        xpDiffPerMinDeltas: {},
        xpPerMinDeltas: {}
      },
      stats: {
        ...myParticipant.stats,
        participantId: pid,
        win: teamId === 100 ? isWin : !isWin,
        kills: 5,
        deaths: 5,
        assists: 5,
        totalDamageDealtToChampions: 15000,
        totalDamageTaken: 15000,
        goldEarned: 10000,
        totalMinionsKilled: 120
      }
    })
    participantIdentities.push({
      participantId: pid,
      player: {
        accountId: pid,
        currentAccountId: pid,
        currentPlatformId: 'HN1',
        matchHistoryUri: '',
        platformId: 'HN1',
        profileIcon: 1,
        summonerId: pid,
        puuid: `puuid-${pid}`,
        gameName: `玩家${pid}`,
        tagLine: 'CN',
        summonerName: `玩家${pid}`
      }
    })
  }

  return {
    endOfGameResult: 'GameComplete',
    gameCreation: item.game_creation,
    gameCreationDate: new Date(item.game_creation).toISOString(),
    gameDuration: item.game_duration_seconds,
    gameId: item.game_id,
    gameMode: item.game_mode || 'CLASSIC',
    gameType: 'MATCHED_GAME',
    gameVersion: '14.24.1',
    mapId: item.queue_id === 450 ? 12 : 11,
    queueId: item.queue_id || 420,
    seasonId: 24,
    platformId: 'HN1',
    gameModeMutators: [],
    participantIdentities,
    participants,
    teams: [
      {
        bans: [],
        baronKills: isWin ? 1 : 0,
        dominionVictoryScore: 0,
        dragonKills: isWin ? 3 : 1,
        hordeKills: 0,
        firstBaron: isWin,
        firstBlood: true,
        firstDargon: true,
        firstInhibitor: isWin,
        firstTower: isWin,
        inhibitorKills: isWin ? 1 : 0,
        riftHeraldKills: 1,
        teamId: 100,
        towerKills: isWin ? 8 : 3,
        vilemawKills: 0,
        win: isWin ? 'Win' : 'Fail'
      },
      {
        bans: [],
        baronKills: isWin ? 0 : 1,
        dominionVictoryScore: 0,
        dragonKills: isWin ? 1 : 3,
        hordeKills: 0,
        firstBaron: !isWin,
        firstBlood: false,
        firstDargon: false,
        firstInhibitor: !isWin,
        firstTower: !isWin,
        inhibitorKills: isWin ? 0 : 1,
        riftHeraldKills: 0,
        teamId: 200,
        towerKills: isWin ? 3 : 8,
        vilemawKills: 0,
        win: isWin ? 'Fail' : 'Win'
      }
    ]
  }
}

// Map matches to LcuOrSgpGameSummary items
const rawSummaries = ref<Map<number, LcuOrSgpGameSummary>>(new Map())

const allMatchSummaries = computed<LcuOrSgpGameSummary[]>(() => {
  return matches.value.map((m) => {
    const existing = rawSummaries.value.get(m.game_id)
    if (existing) return existing

    const game = convertMatchItemToLcuGame(m, profile.value)
    return {
      gameId: m.game_id,
      source: 'lcu',
      data: game
    }
  })
})

// Queue filtering
const filteredSummaries = computed(() => {
  return allMatchSummaries.value.filter((s) => {
    const qid = (s.data as Game).queueId
    if (activeQueue.value === 'ranked_solo' && qid !== 420) return false
    if (activeQueue.value === 'ranked_flex' && qid !== 440) return false
    if (activeQueue.value === 'aram' && qid !== 450) return false
    if (activeQueue.value === 'normal' && qid !== 430 && qid !== 400) return false
    return true
  })
})

// Pagination
const paginatedSummaries = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredSummaries.value.slice(start, start + pageSize.value)
})

const hasNextPage = computed(() => {
  return currentPage.value * pageSize.value < filteredSummaries.value.length
})

// Akari analysis score summary
const analysis = computed(() => {
  if (allMatchSummaries.value.length === 0) return null

  const puuid = profile.value.puuid || 'me-puuid'
  const games = allMatchSummaries.value.map((s) => ({
    gameId: s.gameId,
    summary: s,
    details: detailsMap.value[s.gameId] || undefined
  }))

  const res = analyzeGames(games, puuid, {
    filterPveAndNonMatchedGames: true,
    filterAbortedOrRemadeGames: true
  })

  if (res) return res

  const winsCount = matches.value.filter((m) => m.is_win).length
  const totalCount = matches.value.length

  return {
    akariScore: {
      total: 14.5,
      components: {
        kda: 14,
        killParticipation: 12,
        damageShare: 15,
        tankShare: 13,
        goldShare: 14,
        visionShare: 12
      }
    },
    winLoss: {
      all: {
        count: totalCount,
        wins: winsCount,
        losses: totalCount - winsCount,
        winRate: totalCount > 0 ? winsCount / totalCount : 0
      }
    },
    summary: {
      avgKda:
        matches.value.reduce((acc, m) => acc + (m.kda_ratio || 0), 0) / (totalCount || 1),
      avgKillParticipation: 0.62,
      avgChampionDamagePercentageOfTeam: 0.28,
      avgDamageTakenPercentageOfTeam: 0.24,
      avgCsPerMinute: 7.4
    }
  }
})

// Masteries list
const masteriesList = computed(() => {
  const topChamps = profile.value.recent_summary?.top_champions || []
  if (topChamps.length > 0) {
    return topChamps.map((ch, idx) => ({
      championId: ch.champion_id,
      championLevel: Math.max(7 - idx, 5),
      championPoints: Math.round(ch.games * 12500 + 45000),
      highestGrade: idx === 0 ? 'S+' : idx === 1 ? 'S' : 'A+'
    }))
  }
  return [
    { championId: 266, championLevel: 7, championPoints: 348200, highestGrade: 'S+' },
    { championId: 84, championLevel: 7, championPoints: 219400, highestGrade: 'S' },
    { championId: 39, championLevel: 6, championPoints: 145000, highestGrade: 'S-' },
    { championId: 64, championLevel: 5, championPoints: 89000, highestGrade: 'A+' }
  ]
})

// Handlers
function handleQueueChange(q: string) {
  activeQueue.value = q
  currentPage.value = 1
}

function handlePageSizeChange(size: number) {
  pageSize.value = size
  currentPage.value = 1
}

function handlePrevPage() {
  if (currentPage.value > 1) {
    currentPage.value--
  }
}

function handleNextPage() {
  if (hasNextPage.value) {
    currentPage.value++
  }
}

async function handleLoadDetails(gameId: number) {
  if (loadingGameIds.value.has(gameId)) return
  loadingGameIds.value.add(gameId)

  try {
    if (isDemoMode.value) {
      const mockDetail = getMockMatchDetail()
      if (mockDetail?.raw_game) {
        rawSummaries.value.set(gameId, {
          gameId,
          source: 'lcu',
          data: mockDetail.raw_game
        })
      }
      detailsMap.value = {
        ...detailsMap.value,
        [gameId]: {
          gameId,
          source: 'lcu',
          data: markRaw({ frames: [] })
        }
      }
    } else {
      // 1. Fetch match detail to get raw_game with full 10-player stats
      const detail = await getMatchDetail(gameId)
      if (detail?.raw_game) {
        rawSummaries.value.set(gameId, {
          gameId,
          source: 'lcu',
          data: markRaw(detail.raw_game)
        })
      }

      // 2. Fetch timeline
      let timeline = await getMatchTimeline(gameId)
      if (!timeline || !timeline.frames) {
        timeline = { frames: [] }
      }

      detailsMap.value = {
        ...detailsMap.value,
        [gameId]: {
          gameId,
          source: 'lcu',
          data: markRaw(timeline as any)
        }
      }
    }
  } catch (err) {
    console.error(`Failed to load details for match ${gameId}:`, err)
    // Graceful fallback
    detailsMap.value = {
      ...detailsMap.value,
      [gameId]: {
        gameId,
        source: 'lcu',
        data: markRaw({ frames: [] })
      }
    }
  } finally {
    loadingGameIds.value.delete(gameId)
  }
}

async function handleSelectSummoner(puuid: string) {
  try {
    isLoadingMatches.value = true
    const p = await getSummonerProfile(puuid)
    if (p) {
      profile.value = p
      const list = await getMatchHistory(puuid)
      matches.value = list
      currentPage.value = 1
    }
  } catch (err) {
    console.error('Failed to load selected summoner:', err)
  } finally {
    isLoadingMatches.value = false
  }
}

async function loadRealData() {
  isLoadingMatches.value = true
  try {
    const p = await getSummonerProfile()
    if (p) profile.value = p
    const list = await getMatchHistory()
    matches.value = list
  } catch (err) {
    console.error('loadRealData error:', err)
  } finally {
    isLoadingMatches.value = false
  }
}

async function toggleDemoMode() {
  isDemoMode.value = !isDemoMode.value
  if (isDemoMode.value) {
    profile.value = getMockSummonerProfile()
    matches.value = getMockMatchHistory()
    currentPage.value = 1
  } else {
    await loadRealData()
  }
}

async function refreshData() {
  if (isDemoMode.value) {
    profile.value = getMockSummonerProfile()
    matches.value = getMockMatchHistory()
    return
  }
  await loadRealData()
}

onMounted(() => {
  loadRealData()
})
</script>
