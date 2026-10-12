<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Summoner Career Profile Banner -->
    <div class="p-4 px-6 border-b border-[#162238] bg-[#070b14]/85 backdrop-blur shrink-0 z-10">
      <PlayerTabHeader
        :profile="profile"
        :loading="loading"
        @refresh="$emit('refresh')"
      >
        <template #actions>
          <slot name="header-actions" />
        </template>
      </PlayerTabHeader>
    </div>

    <!-- Main Content Area: Responsive Grid (Left Sidebar + Right Matches List) -->
    <div class="flex-1 overflow-y-auto p-4 lg:p-6">
      <div class="max-w-[1580px] mx-auto grid grid-cols-1 lg:grid-cols-[340px_minmax(0,1fr)] gap-5 items-start">
        <!-- Left Sidebar Sticky Content -->
        <div class="sticky top-0 space-y-3">
          <PlayerTabSidebarContent
            :profile="profile"
            :analysis="analysis"
            :masteries-list="masteriesList"
            :matches="matches"
            :encountered-games="encounteredGames"
            :tags="tags"
            :self-puuid="profile?.puuid"
            @select-summoner="$emit('selectSummoner', $event)"
            @select-game="$emit('selectGame', $event)"
          />
        </div>

        <!-- Right Main Column: Pagination & Filters + Match Cards List -->
        <div class="flex flex-col gap-3 min-w-0">
          <!-- Queue and Pagination Filter Bar -->
          <div class="sticky top-0 z-20">
            <MatchHistoryPagination
              :queue="queue"
              :page="page"
              :page-size="pageSize"
              :has-next-page="hasNextPage"
              :disabled="loading"
              @update:queue="$emit('update:queue', $event)"
              @update:page-size="$emit('update:pageSize', $event)"
              @prev-page="$emit('prevPage')"
              @next-page="$emit('nextPage')"
            />
          </div>

          <!-- Matches Loading State -->
          <div
            v-if="loading"
            class="p-12 text-center text-xs text-slate-400 rounded-2xl bg-[#0a0f1d] border border-[#162238] flex flex-col items-center space-y-3"
          >
            <RotateCw class="w-6 h-6 animate-spin text-[#0ac8b9]" />
            <span class="text-slate-300 font-semibold">正在同步召唤师对局历史...</span>
          </div>

          <!-- Empty Matches State -->
          <div
            v-else-if="matchSummaries.length === 0"
            class="p-16 text-center text-xs text-slate-400 rounded-2xl bg-[#0a0f1d] border border-[#162238] flex flex-col items-center space-y-3"
          >
            <Inbox class="w-10 h-10 text-slate-600" />
            <span class="font-bold text-sm text-slate-200">暂无匹配的战绩数据</span>
            <span class="text-xs text-slate-500 max-w-xs">
              在客户端进行对局后，历史战绩将在此处同步收录。
            </span>
          </div>

          <!-- Match Cards List -->
          <div v-else class="flex flex-col gap-3">
            <MatchCard
              v-for="item in matchSummaries"
              :key="item.gameId"
              :summary="item"
              :details="detailsMap[item.gameId]"
              :puuid="profile?.puuid"
              :loading-details="loadingGameIds.has(item.gameId)"
              @load-details="$emit('loadDetails', $event)"
              @navigate-to-summoner-by-puuid="$emit('selectSummoner', $event)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RotateCw, Inbox } from 'lucide-vue-next'
import type { LcuOrSgpGameSummary, LcuOrSgpGameDetails } from '@shared/data-adapter/wrapper'
import PlayerTabHeader from './PlayerTabHeader.vue'
import PlayerTabSidebarContent from './PlayerTabSidebarContent.vue'
import MatchHistoryPagination from './widgets/MatchHistoryPagination.vue'
import MatchCard from '../match-card/MatchCard.vue'

withDefaults(
  defineProps<{
    profile?: any
    analysis?: any
    masteriesList?: any[]
    matches?: any[]
    matchSummaries?: LcuOrSgpGameSummary[]
    detailsMap?: Record<number, LcuOrSgpGameDetails | null>
    loadingGameIds?: Set<number>
    encounteredGames?: any[]
    tags?: any[]
    loading?: boolean
    queue?: string
    page?: number
    pageSize?: number
    hasNextPage?: boolean
  }>(),
  {
    loading: false,
    queue: 'all',
    page: 1,
    pageSize: 20,
    hasNextPage: true,
    matchSummaries: () => [],
    detailsMap: () => ({}),
    loadingGameIds: () => new Set<number>()
  }
)

defineEmits<{
  refresh: []
  selectSummoner: [puuid: string]
  selectGame: [gameId: number]
  loadDetails: [gameId: number]
  'update:queue': [queue: string]
  'update:pageSize': [size: number]
  prevPage: []
  nextPage: []
}>()
</script>
