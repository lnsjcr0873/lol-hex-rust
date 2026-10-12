<template>
  <div class="relative w-full">
    <MatchCardOverview @toggle-expand="toggleExpand" />

    <KeepAlive>
      <MatchCardDetails v-if="isExpanded" />
    </KeepAlive>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { provideMatchCard } from './context'
import type { MatchCardProps, MatchCardEmits, MatchCardExpose } from './types'
import MatchCardOverview from './MatchCardOverview.vue'
import MatchCardDetails from './MatchCardDetails.vue'

const props = withDefaults(defineProps<MatchCardProps>(), {
  details: null,
  hidePrivacy: false,
  loadingDetails: false,
  replayState: null,
  canDryRunOngoingGame: false
})

const emits = defineEmits<MatchCardEmits>()

const isExpanded = defineModel<boolean>('isExpanded', {
  required: false,
  default: false
})

function toggleExpand() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value && !props.details && !props.loadingDetails) {
    emits('loadDetails', props.summary.gameId)
  }
}

watch(isExpanded, (val) => {
  if (val && !props.details && !props.loadingDetails) {
    emits('loadDetails', props.summary.gameId)
  }
})

provideMatchCard({
  isExpanded: () => isExpanded.value,
  summary: () => props.summary,
  puuid: () => props.puuid,
  details: () => props.details,
  hidePrivacy: () => props.hidePrivacy,
  loadingDetails: () => props.loadingDetails,
  replayState: () => props.replayState,
  canDryRunOngoingGame: () => props.canDryRunOngoingGame,

  navigateToSummonerByPuuid: (puuid: string, setCurrent?: boolean) => {
    emits('navigateToSummonerByPuuid', puuid, setCurrent)
  },
  loadReplay: (gameId: number) => {
    emits('downloadReplay', gameId)
  },
  watchReplay: (gameId: number) => {
    emits('watchReplay', gameId)
  },
  loadDetails: (gameId: number) => {
    emits('loadDetails', gameId)
  },
  dryRunOngoingGame: (draft: any) => {
    emits('dryRunOngoingGame', draft)
  }
})

defineExpose<MatchCardExpose>({
  setExpanded: (expanded: boolean) => {
    isExpanded.value = expanded
  }
})
</script>
