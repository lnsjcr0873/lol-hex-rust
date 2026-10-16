<template>
  <div class="w-full h-full flex flex-col overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Player Tabs Navigation Strip -->
    <PlayerTabStrip
      :tabs="playerTabsStore.tabs"
      :active-tab-id="playerTabsStore.currentTabId"
      @activate="handleActivateTab"
      @close="handleCloseTab"
      @refresh="handleRefreshTab"
      @close-others="handleCloseOthers"
      @close-to-right="handleCloseToRight"
      @open-search="handleOpenSearch"
    >
      <template #actions>
        <slot name="header-actions" />
      </template>
    </PlayerTabStrip>

    <!-- Main Viewport: Active Player Tab (v-show preserves state) or Startup Pane -->
    <div class="flex-1 overflow-hidden relative">
      <template v-if="playerTabsStore.currentTabId">
        <PlayerTab
          v-for="tab in playerTabsStore.tabs"
          :key="tab.id"
          :tab="tab"
          :is-active="tab.id === playerTabsStore.currentTabId"
          v-show="tab.id === playerTabsStore.currentTabId"
          :ref="(el) => setTabRef(tab.id, el)"
          @select-summoner="handleSelectSummoner"
          @select-game="handleSelectGame"
          @profile-updated="handleProfileUpdated(tab.id, $event)"
        />
      </template>

      <!-- Startup Pane when no tabs or searching -->
      <StartupPane
        v-else
        :current-profile="selfProfile"
        @open-summoner="handleOpenSummonerFromStartup"
      />
    </div>

    <!-- Cross-Page Connected Match Previewer Modal -->
    <ConnectedMatchPreviewer
      v-model:show="showPreviewModal"
      :game-id="previewGameId"
      :puuid="playerTabsStore.currentTab?.puuid"
      @navigate-to-summoner-by-puuid="handleSelectSummoner"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePlayerTabsStore, type PlayerTabItem } from '../../stores/player-tabs'
import PlayerTabStrip from './widgets/PlayerTabStrip.vue'
import PlayerTab from './PlayerTab.vue'
import StartupPane from './StartupPane.vue'
import ConnectedMatchPreviewer from '../match-preview/ConnectedMatchPreviewer.vue'
import { getSummonerProfile } from '../../api/tauri'
import type { SummonerProfile } from '../../api/types'

const playerTabsStore = usePlayerTabsStore()

const selfProfile = ref<SummonerProfile | null>(null)
const tabComponentsMap = new Map<string, any>()

function setTabRef(tabId: string, el: any) {
  if (el) {
    tabComponentsMap.set(tabId, el)
  } else {
    tabComponentsMap.delete(tabId)
  }
}

const showPreviewModal = ref(false)
const previewGameId = ref(0)

function handleSelectGame(gameId: number) {
  previewGameId.value = gameId
  showPreviewModal.value = true
}

function handleActivateTab(id: string) {
  playerTabsStore.setCurrentTab(id)
}

function handleCloseTab(id: string) {
  playerTabsStore.closeTab(id)
}

function handleCloseOthers(id: string) {
  playerTabsStore.closeOtherTabs(id)
}

function handleCloseToRight(id: string) {
  playerTabsStore.closeTabsToTheRight(id)
}

function handleOpenSearch() {
  playerTabsStore.setCurrentTab(null)
}

function handleSelectSummoner(puuid: string) {
  openSummonerTab(puuid)
}

function handleOpenSummonerFromStartup(payload: {
  puuid: string
  gameName?: string
  tagLine?: string
  profileIconId?: number
  isMock?: boolean
}) {
  playerTabsStore.openTab(payload.puuid, 'lcu', {
    gameName: payload.gameName || '召唤师',
    tagLine: payload.tagLine || '',
    profileIconId: payload.profileIconId ?? 1,
    isMock: payload.isMock ?? false
  })
}

function openSummonerTab(puuid: string, meta?: Partial<PlayerTabItem>) {
  playerTabsStore.openTab(puuid, 'lcu', meta, true)
}

function handleProfileUpdated(tabId: string, updatedProfile: SummonerProfile) {
  playerTabsStore.updateTab(tabId, {
    gameName: updatedProfile.game_name,
    tagLine: updatedProfile.tag_line,
    profileIconId: updatedProfile.profile_icon_id,
    summonerLevel: updatedProfile.summoner_level,
    title: `${updatedProfile.game_name}#${updatedProfile.tag_line}`
  })
}

function handleRefreshTab(tabId: string) {
  tabComponentsMap.get(tabId)?.refresh?.()
}

async function initSelfProfile() {
  try {
    const p = await getSummonerProfile()
    if (p && p.puuid && p.is_connected) {
      selfProfile.value = p
      // If no tabs currently opened, auto-open self tab
      if (playerTabsStore.tabs.length === 0) {
        playerTabsStore.openTab(p.puuid, 'lcu', {
          gameName: p.game_name,
          tagLine: p.tag_line,
          profileIconId: p.profile_icon_id,
          summonerLevel: p.summoner_level,
          isSelf: true
        })
      }
    }
  } catch (err) {
    console.warn('initSelfProfile failed:', err)
  }
}

onMounted(() => {
  initSelfProfile()
})

defineExpose({
  openSummoner: openSummonerTab,
  refreshCurrentTab: () => {
    if (playerTabsStore.currentTabId) {
      handleRefreshTab(playerTabsStore.currentTabId)
    }
  }
})
</script>
