import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface PlayerTabItem {
  id: string
  puuid: string
  sgpServerId: string
  gameName: string
  tagLine: string
  profileIconId: number
  summonerLevel: number
  isLoading: boolean
  isSelf: boolean
  isMock?: boolean
  title?: string
}

export interface SearchHistoryItem {
  id: string
  gameName: string
  tagLine: string
  puuid: string
  sgpServerId: string
  profileIconId: number
  timestamp: number
}

const STORAGE_KEY = 'hexbrain:player-search-history'

function loadSavedSearchHistory(): SearchHistoryItem[] {
  try {
    if (typeof localStorage !== 'undefined') {
      const raw = localStorage.getItem(STORAGE_KEY)
      if (raw) {
        const parsed = JSON.parse(raw)
        if (Array.isArray(parsed)) return parsed
      }
    }
  } catch (e) {
    console.warn('Failed to parse search history:', e)
  }
  return []
}

function persistSearchHistory(items: SearchHistoryItem[]) {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
    }
  } catch (e) {
    console.warn('Failed to save search history:', e)
  }
}

export const usePlayerTabsStore = defineStore('player-tabs', () => {
  const tabs = ref<PlayerTabItem[]>([])
  const currentTabId = ref<string | null>(null)
  const searchHistory = ref<SearchHistoryItem[]>(loadSavedSearchHistory())

  const currentTab = computed(() => {
    if (!currentTabId.value) return null
    return tabs.value.find((t) => t.id === currentTabId.value) || null
  })

  const hasTabs = computed(() => tabs.value.length > 0)

  function formatTabId(puuid: string, sgpServerId = 'lcu'): string {
    return `${sgpServerId}:${puuid}`
  }

  function openTab(
    puuid: string,
    sgpServerId = 'lcu',
    info?: Partial<PlayerTabItem>,
    setCurrent = true
  ): PlayerTabItem {
    const tabId = formatTabId(puuid, sgpServerId)
    const existing = tabs.value.find((t) => t.id === tabId)

    if (existing) {
      if (info) {
        Object.assign(existing, info)
      }
      if (setCurrent) {
        currentTabId.value = tabId
      }
      return existing
    }

    const newTab: PlayerTabItem = {
      id: tabId,
      puuid,
      sgpServerId,
      gameName: info?.gameName || '召唤师',
      tagLine: info?.tagLine || '',
      profileIconId: info?.profileIconId ?? 1,
      summonerLevel: info?.summonerLevel ?? 0,
      isLoading: info?.isLoading ?? false,
      isSelf: info?.isSelf ?? false,
      isMock: info?.isMock ?? false,
      title: info?.title || (info?.gameName ? `${info.gameName}#${info.tagLine || ''}` : '召唤师')
    }

    const MAX_TABS = 16
    if (tabs.value.length >= MAX_TABS) {
      const evictIndex = tabs.value.findIndex((t) => t.id !== currentTabId.value && !t.isSelf)
      if (evictIndex !== -1) {
        tabs.value.splice(evictIndex, 1)
      } else {
        tabs.value.shift()
      }
      if (!tabs.value.some((t) => t.id === currentTabId.value)) {
        currentTabId.value = tabs.value[0]?.id || null
      }
    }

    tabs.value.push(newTab)

    if (setCurrent || tabs.value.length === 1 || !currentTabId.value) {
      currentTabId.value = tabId
    }

    return newTab
  }

  function closeTab(id: string) {
    const index = tabs.value.findIndex((t) => t.id === id)
    if (index === -1) return

    tabs.value.splice(index, 1)

    if (currentTabId.value === id) {
      if (tabs.value.length > 0) {
        const nextIndex = Math.min(index, tabs.value.length - 1)
        currentTabId.value = tabs.value[nextIndex].id
      } else {
        currentTabId.value = null
      }
    }
  }

  function closeOtherTabs(targetId: string) {
    const target = tabs.value.find((t) => t.id === targetId)
    if (!target) return
    tabs.value = [target]
    currentTabId.value = targetId
  }

  function closeTabsToTheRight(targetId: string) {
    const index = tabs.value.findIndex((t) => t.id === targetId)
    if (index === -1) return

    tabs.value = tabs.value.slice(0, index + 1)
    if (!tabs.value.some((t) => t.id === currentTabId.value)) {
      currentTabId.value = targetId
    }
  }

  function closeAllTabs() {
    tabs.value = []
    currentTabId.value = null
  }

  function setCurrentTab(id: string | null) {
    if (id === null || tabs.value.some((t) => t.id === id)) {
      currentTabId.value = id
    }
  }

  function updateTab(id: string, updates: Partial<PlayerTabItem>) {
    const tab = tabs.value.find((t) => t.id === id)
    if (tab) {
      Object.assign(tab, updates)
      if (updates.gameName || updates.tagLine) {
        tab.title = `${tab.gameName}#${tab.tagLine}`
      }
    }
  }

  function moveTab(fromIndex: number, toIndex: number) {
    if (
      fromIndex < 0 ||
      fromIndex >= tabs.value.length ||
      toIndex < 0 ||
      toIndex >= tabs.value.length
    ) {
      return
    }
    const [item] = tabs.value.splice(fromIndex, 1)
    tabs.value.splice(toIndex, 0, item)
  }

  function addSearchHistory(item: {
    gameName: string
    tagLine?: string
    puuid?: string
    sgpServerId?: string
    profileIconId?: number
  }) {
    const trimmedName = (item.gameName || '').trim()
    if (!trimmedName || trimmedName === '召唤师') {
      return
    }
    const fullTag = (item.tagLine || '').trim()
    const entryId = item.puuid || `${trimmedName}#${fullTag}`

    const filtered = searchHistory.value.filter(
      (h) => h.id !== entryId && !(h.gameName === trimmedName && h.tagLine === fullTag)
    )

    const newEntry: SearchHistoryItem = {
      id: entryId,
      gameName: trimmedName,
      tagLine: fullTag,
      puuid: item.puuid || '',
      sgpServerId: item.sgpServerId || 'lcu',
      profileIconId: item.profileIconId ?? 1,
      timestamp: Date.now()
    }

    filtered.unshift(newEntry)
    searchHistory.value = filtered.slice(0, 20)
    persistSearchHistory(searchHistory.value)
  }

  function removeSearchHistory(id: string) {
    searchHistory.value = searchHistory.value.filter((h) => h.id !== id)
    persistSearchHistory(searchHistory.value)
  }

  function clearSearchHistory() {
    searchHistory.value = []
    persistSearchHistory([])
  }

  return {
    tabs,
    currentTabId,
    currentTab,
    hasTabs,
    formatTabId,
    openTab,
    closeTab,
    closeOtherTabs,
    closeTabsToTheRight,
    closeAllTabs,
    setCurrentTab,
    updateTab,
    moveTab,
    searchHistory,
    addSearchHistory,
    removeSearchHistory,
    clearSearchHistory
  }
})
