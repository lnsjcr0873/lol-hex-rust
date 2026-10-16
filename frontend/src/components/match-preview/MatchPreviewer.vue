<template>
  <NModal
    v-model:show="show"
    preset="card"
    :mask-closable="true"
    :closable="true"
    style="width: min(1040px, 95vw); max-height: 90vh; background-color: #070d18; border: 1px solid #1e293b; border-radius: 1rem; overflow-y: auto;"
    :segmented="{ content: 'soft' }"
  >
    <div class="p-1">
      <MatchCard
        v-if="summary"
        :summary="summary"
        :details="details"
        :puuid="puuid"
        :loading-details="loadingDetails"
        :hide-privacy="hidePrivacy"
        :can-dry-run-ongoing-game="canDryRunOngoingGame"
        is-expanded
        @navigate-to-summoner-by-puuid="emits('navigateToSummonerByPuuid', $event)"
        @dry-run-ongoing-game="emits('dryRunOngoingGame', $event)"
        @load-details="handleLoadDetails"
      />
      <div
        v-else
        class="flex h-48 w-full items-center justify-center rounded-xl border border-slate-800 bg-slate-900/60 text-slate-400 select-none"
      >
        <template v-if="loadingSummary">
          <div class="flex items-center gap-2 text-cyan-400">
            <NSpin :size="16" />
            <span class="text-xs font-semibold">正在加载对局详情...</span>
          </div>
        </template>
        <template v-else>
          <div class="flex flex-col items-center gap-1.5 text-xs">
            <span class="text-slate-300 font-bold">暂无该对局数据</span>
            <span class="text-[11px] text-slate-500">无法在本地 LCU 或 SGP 网关中定位该对局记录</span>
          </div>
        </template>
      </div>
    </div>
  </NModal>
</template>

<script lang="ts" setup>
import { NModal, NSpin } from 'naive-ui'
import type { LcuOrSgpGameDetails, LcuOrSgpGameSummary } from '@shared/data-adapter/wrapper'
import MatchCard from '../match-card/MatchCard.vue'

const {
  summary = null,
  details = null,
  puuid,
  hidePrivacy = false,
  canDryRunOngoingGame = false,
  loadingSummary = false,
  loadingDetails = false
} = defineProps<{
  summary?: LcuOrSgpGameSummary | null
  details?: LcuOrSgpGameDetails | null
  puuid?: string
  hidePrivacy?: boolean
  canDryRunOngoingGame?: boolean
  loadingSummary?: boolean
  loadingDetails?: boolean
}>()

const emits = defineEmits<{
  loadDetails: [summary: LcuOrSgpGameSummary]
  navigateToSummonerByPuuid: [puuid: string]
  dryRunOngoingGame: [draft: any]
}>()

const show = defineModel<boolean>('show', { default: false })

const handleLoadDetails = () => {
  if (summary) {
    emits('loadDetails', summary)
  }
}
</script>
