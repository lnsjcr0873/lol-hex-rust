<template>
  <div
    ref="wrapperEl"
    class="relative w-full min-w-[280px]"
    :aria-hidden="hidden || undefined"
    :data-match-card-render-state="renderState"
    :style="wrapperStyle"
  >
    <Transition name="match-history-card-materialize" :css="animateMaterialization">
      <MatchCard
        v-if="renderState === 'materialized'"
        v-model:is-expanded="isExpanded"
        :summary="summary"
        :puuid="puuid"
        :details="details"
        :loading-details="loadingDetails"
        :hide-privacy="hidePrivacy"
        :replay-state="replayState"
        :can-dry-run-ongoing-game="canDryRunOngoingGame"
        @navigate-to-summoner-by-puuid="handleNavigateToSummonerByPuuid"
        @load-details="(gameId) => emit('loadDetails', gameId)"
        @download-replay="(gameId) => emit('downloadReplay', gameId)"
        @watch-replay="(gameId) => emit('watchReplay', gameId)"
        @dry-run-ongoing-game="(draft) => emit('dryRunOngoingGame', draft)"
      />
      <!-- Placeholder Skeleton when out of viewport -->
      <div
        v-else
        class="box-border flex h-[96px] w-full items-center justify-between rounded-xl border border-slate-800/60 bg-slate-900/40 p-3.5 px-4 animate-pulse select-none"
      >
        <div class="flex items-center space-x-3.5">
          <div class="h-12 w-12 rounded-xl bg-slate-800/60 shrink-0"></div>
          <div class="grid grid-cols-2 gap-1 shrink-0">
            <div class="h-5 w-5 rounded bg-slate-800/60"></div>
            <div class="h-5 w-5 rounded bg-slate-800/60"></div>
          </div>
          <div class="space-y-2 ml-1">
            <div class="h-4 w-24 rounded bg-slate-800/60"></div>
            <div class="h-3 w-16 rounded bg-slate-800/40"></div>
          </div>
        </div>
        <div class="flex items-center space-x-1 shrink-0">
          <div v-for="s in 6" :key="s" class="h-7 w-7 rounded-lg bg-slate-800/40"></div>
        </div>
        <div class="flex items-center space-x-4 shrink-0">
          <div class="h-4 w-16 rounded bg-slate-800/60"></div>
          <div class="h-5 w-20 rounded-full bg-slate-800/60"></div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useTimeoutFn } from '@vueuse/core'
import MatchCard from '../../../match-card/MatchCard.vue'
import type {
  MatchCardEmits,
  MatchCardExpose,
  MatchCardProps
} from '../../../match-card/types'
import {
  MATCH_CARD_COLLAPSED_HEIGHT_PX,
  MATCH_HISTORY_CARD_RECYCLE_DELAY_MS
} from './constants'
import type { MatchHistoryCardOptimizationMode } from './types'
import { useMatchHistoryCardViewport } from './viewport'

type MatchHistoryCardWrapperProps = MatchCardProps & {
  hidden?: boolean
  optimizationMode?: MatchHistoryCardOptimizationMode
}

const props = withDefaults(defineProps<MatchHistoryCardWrapperProps>(), {
  details: null,
  hidePrivacy: false,
  loadingDetails: false,
  replayState: null,
  canDryRunOngoingGame: false,
  hidden: false,
  optimizationMode: 'recycle'
})

const emit = defineEmits<MatchCardEmits>()
const isExpanded = defineModel<boolean>('isExpanded', {
  required: false,
  default: false
})

const wrapperEl = ref<HTMLElement | null>(null)
const viewport = useMatchHistoryCardViewport()

const renderState = ref<'placeholder' | 'materialized'>('placeholder')
const isNearViewport = ref(false)
const animateMaterialization = ref(false)

let viewportRegistration: ReturnType<typeof viewport.register> | null = null
let hasReceivedViewportResult = false

const wrapperStyle = computed(() => ({
  display: props.hidden ? 'none' : undefined,
  minHeight: `${MATCH_CARD_COLLAPSED_HEIGHT_PX}px`
}))

const { start: startRecycleTimer, stop: stopRecycleTimer } = useTimeoutFn(
  () => {
    if (
      renderState.value === 'materialized' &&
      props.optimizationMode === 'recycle' &&
      !props.hidden &&
      !isNearViewport.value &&
      !isExpanded.value
    ) {
      renderState.value = 'placeholder'
    }
  },
  MATCH_HISTORY_CARD_RECYCLE_DELAY_MS,
  { immediate: false }
)

const materialize = (animate = true) => {
  stopRecycleTimer()

  if (renderState.value === 'materialized') {
    return
  }

  animateMaterialization.value = animate
  renderState.value = 'materialized'
}

const scheduleRecycle = () => {
  stopRecycleTimer()

  if (
    renderState.value !== 'materialized' ||
    props.optimizationMode !== 'recycle' ||
    props.hidden ||
    isNearViewport.value ||
    isExpanded.value
  ) {
    return
  }

  startRecycleTimer()
}

const handleNearViewportChange = (isNear: boolean) => {
  const animate = hasReceivedViewportResult
  hasReceivedViewportResult = true
  isNearViewport.value = isNear

  if (props.hidden) {
    return
  }

  if (isNear) {
    materialize(animate)
  } else {
    scheduleRecycle()
  }
}

const handleNavigateToSummonerByPuuid = (puuid: string, setCurrent?: boolean) => {
  emit('navigateToSummonerByPuuid', puuid, setCurrent)
}

const setExpanded = (expanded: boolean) => {
  isExpanded.value = expanded

  if (expanded && !props.hidden) {
    materialize()
  } else if (!expanded) {
    scheduleRecycle()
  }
}

watch(
  () => props.hidden,
  async (isHidden) => {
    stopRecycleTimer()
    isNearViewport.value = false

    if (isHidden) {
      return
    }

    await nextTick()

    if (isExpanded.value) {
      materialize()
    } else {
      viewportRegistration?.refresh()
    }
  }
)

watch(
  () => props.optimizationMode,
  (mode) => {
    if (mode === 'once') {
      stopRecycleTimer()
    } else {
      scheduleRecycle()
    }
  }
)

watch(isExpanded, (expanded) => {
  if (expanded && !props.hidden) {
    materialize()
  } else if (!expanded) {
    scheduleRecycle()
  }
})

onMounted(() => {
  const element = wrapperEl.value

  if (!element) {
    return
  }

  viewportRegistration = viewport.register(element, handleNearViewportChange)

  if (isExpanded.value && !props.hidden) {
    materialize(false)
  }
})

onBeforeUnmount(() => {
  viewportRegistration?.unregister()
})

defineExpose<MatchCardExpose>({
  setExpanded
})
</script>

<style scoped>
.match-history-card-materialize-enter-active {
  transition: opacity 180ms ease-out;
}

.match-history-card-materialize-enter-from {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .match-history-card-materialize-enter-active {
    transition-duration: 0.01ms;
  }
}
</style>
