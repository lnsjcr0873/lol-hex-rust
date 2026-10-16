<template>
  <MatchPreviewer
    v-model:show="show"
    :summary="summary"
    :details="details"
    :puuid="puuid"
    :hide-privacy="hidePrivacy"
    :can-dry-run-ongoing-game="canDryRunOngoingGame"
    :loading-summary="isLoadingGameSummary"
    :loading-details="isLoadingDetails"
    @load-details="loadDetails"
    @navigate-to-summoner-by-puuid="emits('navigateToSummonerByPuuid', $event)"
    @dry-run-ongoing-game="emits('dryRunOngoingGame', $event)"
  />
</template>

<script lang="ts" setup>
import { computed, ref, shallowRef, watch } from 'vue'
import type { LcuOrSgpGameDetails, LcuOrSgpGameSummary } from '@shared/data-adapter/wrapper'
import { getRawMatchHistory, getRawMatchTimeline, getMatchDetail } from '../../api/tauri'
import MatchPreviewer from './MatchPreviewer.vue'

const props = withDefaults(
  defineProps<{
    gameId?: number
    source?: 'sgp' | 'lcu'
    puuid?: string
    hidePrivacy?: boolean
    canDryRunOngoingGame?: boolean
    summary?: LcuOrSgpGameSummary | null
    details?: LcuOrSgpGameDetails | null
  }>(),
  {
    gameId: 0,
    source: 'lcu',
    hidePrivacy: false,
    canDryRunOngoingGame: false,
    summary: null,
    details: null
  }
)

const emits = defineEmits<{
  navigateToSummonerByPuuid: [puuid: string]
  dryRunOngoingGame: [draft: any]
}>()

const show = defineModel<boolean>('show', { default: false })

const loadedSummary = shallowRef<LcuOrSgpGameSummary | null>(null)
const loadedDetails = shallowRef<LcuOrSgpGameDetails | null>(null)
const isLoadingGameSummary = ref(false)
const isLoadingDetails = ref(false)

const summary = computed<LcuOrSgpGameSummary | null>(() => {
  if (props.summary && (!props.gameId || props.summary.gameId === props.gameId)) {
    return props.summary
  }
  return loadedSummary.value
})

const details = computed<LcuOrSgpGameDetails | null>(() => {
  if (props.details && (!props.gameId || props.details.gameId === props.gameId)) {
    return props.details
  }
  return loadedDetails.value
})

async function fetchSummary() {
  const targetId = props.gameId || props.summary?.gameId || 0
  if (!targetId || targetId <= 0) return
  if (summary.value && summary.value.gameId === targetId) return

  isLoadingGameSummary.value = true
  try {
    let foundGame: any = null
    try {
      const raw = await getRawMatchHistory(props.puuid)
      const gamesList: any[] = Array.isArray(raw)
        ? raw
        : Array.isArray(raw?.games)
        ? raw.games
        : Array.isArray(raw?.games?.games)
        ? raw.games.games
        : []
      foundGame = gamesList.find((g: any) => g && g.gameId === targetId)
    } catch (e) {
      // Fallback to getMatchDetail
    }

    if (!foundGame) {
      try {
        const detail = await getMatchDetail(targetId)
        if (detail?.raw_game) {
          foundGame = detail.raw_game
        }
      } catch (e) {
        // Handled below
      }
    }

    if (foundGame) {
      loadedSummary.value = {
        gameId: targetId,
        source: 'lcu',
        data: foundGame
      }
    }
  } catch (err) {
    console.error(`[ConnectedMatchPreviewer] Failed to load summary for game ${targetId}:`, err)
  } finally {
    isLoadingGameSummary.value = false
  }
}

async function loadDetails() {
  const targetId = props.gameId || summary.value?.gameId || 0
  if (!targetId || targetId <= 0) return
  if (details.value && details.value.gameId === targetId) return

  isLoadingDetails.value = true
  try {
    let timeline: any = null
    try {
      timeline = await getRawMatchTimeline(targetId)
    } catch (e) {
      console.warn(`[ConnectedMatchPreviewer] getRawMatchTimeline failed for ${targetId}:`, e)
    }
    if (!timeline || !timeline.frames) {
      timeline = { frames: [] }
    }
    loadedDetails.value = {
      gameId: targetId,
      source: 'lcu',
      data: timeline
    }
  } catch (err) {
    console.error(`[ConnectedMatchPreviewer] Failed to load timeline for game ${targetId}:`, err)
  } finally {
    isLoadingDetails.value = false
  }
}

watch(
  () => show.value,
  (isOpen) => {
    if (isOpen) {
      if (!summary.value) {
        fetchSummary()
      }
    } else {
      loadedSummary.value = null
      loadedDetails.value = null
    }
  }
)

watch(
  () => props.gameId,
  (newId) => {
    if (show.value && newId > 0) {
      fetchSummary()
    }
  }
)
</script>
