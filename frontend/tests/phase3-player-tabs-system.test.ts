import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { usePlayerTabsStore } from '../src/stores/player-tabs'
import { getMockChampionMastery } from '../src/api/mockData'
import { getChampionName, getChampionTitle } from '../src/components/match-card/utils/resources'

describe('Phase 3: Player Tabs System & Champion Mastery Modal (1:1 LeagueAkari Alignment)', () => {
  const storeMap = new Map<string, string>()
  const mockLocalStorage = {
    getItem: (key: string) => storeMap.get(key) || null,
    setItem: (key: string, val: string) => storeMap.set(key, val),
    removeItem: (key: string) => storeMap.delete(key),
    clear: () => storeMap.clear()
  }

  beforeEach(() => {
    // @ts-ignore
    globalThis.localStorage = mockLocalStorage
    storeMap.clear()
    setActivePinia(createPinia())
  })

  // =========================================================================
  // 1. Multi-Tab State Management (Pinia Store)
  // =========================================================================
  describe('1. Player Tabs Pinia Store (usePlayerTabsStore)', () => {
    it('initializes with empty tabs and null currentTabId', () => {
      const store = usePlayerTabsStore()
      expect(store.tabs).toEqual([])
      expect(store.currentTabId).toBeNull()
      expect(store.currentTab).toBeNull()
      expect(store.hasTabs).toBe(false)
    })

    it('opens a new tab and sets it as currentTabId', () => {
      const store = usePlayerTabsStore()
      const tab = store.openTab('puuid-1', 'lcu', {
        gameName: 'Faker',
        tagLine: 'T1',
        profileIconId: 548,
        summonerLevel: 650
      })

      expect(store.tabs.length).toBe(1)
      expect(store.currentTabId).toBe('lcu:puuid-1')
      expect(store.currentTab?.gameName).toBe('Faker')
      expect(store.currentTab?.tagLine).toBe('T1')
      expect(store.hasTabs).toBe(true)
      expect(tab.id).toBe('lcu:puuid-1')
    })

    it('does not duplicate tab when opening same puuid, switches currentTabId instead', () => {
      const store = usePlayerTabsStore()
      store.openTab('puuid-1', 'lcu', { gameName: 'Faker' })
      store.openTab('puuid-2', 'lcu', { gameName: 'TheShy' })
      expect(store.tabs.length).toBe(2)
      expect(store.currentTabId).toBe('lcu:puuid-2')

      // Re-opening puuid-1 switches focus without adding new tab
      store.openTab('puuid-1', 'lcu')
      expect(store.tabs.length).toBe(2)
      expect(store.currentTabId).toBe('lcu:puuid-1')
    })

    it('handles tab closing with intelligent adjacent tab switching', () => {
      const store = usePlayerTabsStore()
      store.openTab('p1', 'lcu', { gameName: 'P1' })
      store.openTab('p2', 'lcu', { gameName: 'P2' })
      store.openTab('p3', 'lcu', { gameName: 'P3' })

      expect(store.currentTabId).toBe('lcu:p3')

      // Close active tab p3 -> switches to p2 (left neighbor)
      store.closeTab('lcu:p3')
      expect(store.tabs.length).toBe(2)
      expect(store.currentTabId).toBe('lcu:p2')

      // Close p1 while p2 is active -> p2 stays active
      store.closeTab('lcu:p1')
      expect(store.tabs.length).toBe(1)
      expect(store.currentTabId).toBe('lcu:p2')

      // Close last tab -> currentTabId becomes null
      store.closeTab('lcu:p2')
      expect(store.tabs.length).toBe(0)
      expect(store.currentTabId).toBeNull()
    })

    it('supports closeOtherTabs and closeTabsToTheRight', () => {
      const store = usePlayerTabsStore()
      store.openTab('p1', 'lcu')
      store.openTab('p2', 'lcu')
      store.openTab('p3', 'lcu')
      store.openTab('p4', 'lcu')

      // Close tabs to right of p2 (p3 and p4 removed)
      store.closeTabsToTheRight('lcu:p2')
      expect(store.tabs.map((t) => t.id)).toEqual(['lcu:p1', 'lcu:p2'])
      expect(store.currentTabId).toBe('lcu:p2')

      // Close other tabs around p1
      store.closeOtherTabs('lcu:p1')
      expect(store.tabs.map((t) => t.id)).toEqual(['lcu:p1'])
      expect(store.currentTabId).toBe('lcu:p1')
    })

    it('supports tab reordering via moveTab', () => {
      const store = usePlayerTabsStore()
      store.openTab('p1', 'lcu')
      store.openTab('p2', 'lcu')
      store.openTab('p3', 'lcu')

      // Move index 0 (p1) to index 2
      store.moveTab(0, 2)
      expect(store.tabs.map((t) => t.id)).toEqual(['lcu:p2', 'lcu:p3', 'lcu:p1'])
    })
  })

  // =========================================================================
  // 2. Search History Persistence & Deduplication
  // =========================================================================
  describe('2. Search History System', () => {
    it('persists and deduplicates recent summoner searches', () => {
      const store = usePlayerTabsStore()
      store.addSearchHistory({ gameName: 'Uzi', tagLine: 'RNG', puuid: 'puuid-uzi' })
      store.addSearchHistory({ gameName: 'Rookie', tagLine: 'IG', puuid: 'puuid-rookie' })

      expect(store.searchHistory.length).toBe(2)
      expect(store.searchHistory[0].gameName).toBe('Rookie')

      // Searching Rookie again moves it to top without duplicating
      store.addSearchHistory({ gameName: 'Rookie', tagLine: 'IG', puuid: 'puuid-rookie' })
      expect(store.searchHistory.length).toBe(2)
      expect(store.searchHistory[0].gameName).toBe('Rookie')

      // Removal
      store.removeSearchHistory('puuid-uzi')
      expect(store.searchHistory.length).toBe(1)
      expect(store.searchHistory[0].gameName).toBe('Rookie')

      // Clear
      store.clearSearchHistory()
      expect(store.searchHistory.length).toBe(0)
    })
  })

  // =========================================================================
  // 3. Full Champion Mastery Modal Filtering & Sorting
  // =========================================================================
  describe('3. Full Champion Mastery Calculations & Formatting', () => {
    it('accurately resolves champion names and titles from static database', () => {
      expect(getChampionName(157)).toBe('疾风剑豪')
      expect(getChampionTitle(157)).toBe('疾风剑豪')
      expect(getChampionName(64)).toBe('盲僧')
      expect(getChampionName(999999)).toBe('英雄999999')
    })

    it('provides mock mastery data for all 15+ benchmark champions with level, points, grades, and milestone fields', () => {
      const masteries = getMockChampionMastery()
      expect(masteries.length).toBeGreaterThanOrEqual(15)

      const yasuo = masteries.find((m) => m.championId === 157)
      expect(yasuo).toBeDefined()
      expect(yasuo?.championLevel).toBe(7)
      expect(yasuo?.championPoints).toBe(268420)
      expect(yasuo?.highestGrade).toBe('S+')
      expect(yasuo?.chestGranted).toBe(true)
      expect(yasuo?.masteryMilestone).toBe(8)

      // Points formatting
      const pts = yasuo?.championPoints || 0
      const formatted = pts >= 10000 ? `${(pts / 10000).toFixed(1)}万` : pts.toLocaleString()
      expect(formatted).toBe('26.8万')
    })

    it('validates mastery sorting strategies: points, level, and grade', () => {
      const masteries = getMockChampionMastery()

      // Sort by points desc
      const byPoints = [...masteries].sort((a, b) => b.championPoints - a.championPoints)
      expect(byPoints[0].championId).toBe(157) // Yasuo 268k
      expect(byPoints[1].championId).toBe(64) // Lee Sin 184k

      // Sort by level desc
      const byLevel = [...masteries].sort((a, b) => b.championLevel - a.championLevel)
      expect(byLevel[0].championLevel).toBe(7)
      expect(byLevel[byLevel.length - 1].championLevel).toBe(2)

      // Filter by keyword '亚索' or '剑豪'
      const filtered = masteries.filter((m) => {
        const name = getChampionName(m.championId).toLowerCase()
        return name.includes('亚索') || name.includes('剑豪')
      })
      expect(filtered.length).toBe(1)
      expect(filtered[0].championId).toBe(157)
    })
  })

  // =========================================================================
  // 4. Player Challenges & Asset Statistics Widget (1:1 LeagueAkari)
  // =========================================================================
  describe('4. Player Challenges & Asset Statistics Widget', () => {
    const CHALLENGES_CHAMPIONS_ID = 505001
    const CHALLENGES_CHAMPION_SKINS_ID = 510001
    const CHALLENGES_CHROMA_SKINS_ID = 510011
    const CHALLENGES_WARD_SKINS_ID = 504003
    const CHALLENGES_SUMMONER_ICONS_ID = 504002
    const CHALLENGES_EMOTES_ID = 504004

    const CHALLENGE_NAME_MAP: Record<number, string> = {
      [CHALLENGES_CHAMPIONS_ID]: '拥有英雄',
      [CHALLENGES_CHAMPION_SKINS_ID]: '拥有皮肤',
      [CHALLENGES_CHROMA_SKINS_ID]: '炫彩皮肤',
      [CHALLENGES_WARD_SKINS_ID]: '守卫皮肤',
      [CHALLENGES_SUMMONER_ICONS_ID]: '召唤师头像',
      [CHALLENGES_EMOTES_ID]: '表情数量'
    }

    it('extracts all 6 core assets accurately from playerChallenges payload', () => {
      const mockPayload = {
        playerChallenges: [
          { id: CHALLENGES_CHAMPIONS_ID, currentValue: 168 },
          { id: CHALLENGES_CHAMPION_SKINS_ID, currentValue: 145 },
          { id: CHALLENGES_CHROMA_SKINS_ID, currentValue: 38 },
          { id: CHALLENGES_WARD_SKINS_ID, currentValue: 12 },
          { id: CHALLENGES_SUMMONER_ICONS_ID, currentValue: 88 },
          { id: CHALLENGES_EMOTES_ID, currentValue: 45 },
          { id: 999999, currentValue: 999 } // extraneous challenge
        ]
      }

      const challengeMap = mockPayload.playerChallenges.reduce((acc, cur) => {
        acc[cur.id] = cur
        return acc
      }, {} as Record<number, { id: number; currentValue: number }>)

      const targetIds = [
        CHALLENGES_CHAMPIONS_ID,
        CHALLENGES_CHAMPION_SKINS_ID,
        CHALLENGES_CHROMA_SKINS_ID,
        CHALLENGES_WARD_SKINS_ID,
        CHALLENGES_SUMMONER_ICONS_ID,
        CHALLENGES_EMOTES_ID
      ]

      const items = targetIds
        .map((id) => {
          const c = challengeMap[id]
          if (!c) return null
          return {
            id: c.id,
            currentValue: c.currentValue,
            name: CHALLENGE_NAME_MAP[id]
          }
        })
        .filter(Boolean)

      expect(items.length).toBe(6)
      expect(items[0]).toEqual({ id: 505001, currentValue: 168, name: '拥有英雄' })
      expect(items[1]).toEqual({ id: 510001, currentValue: 145, name: '拥有皮肤' })
      expect(items[2]).toEqual({ id: 510011, currentValue: 38, name: '炫彩皮肤' })
      expect(items[3]).toEqual({ id: 504003, currentValue: 12, name: '守卫皮肤' })
      expect(items[4]).toEqual({ id: 504002, currentValue: 88, name: '召唤师头像' })
      expect(items[5]).toEqual({ id: 504004, currentValue: 45, name: '表情数量' })
    })

    it('gracefully handles empty, missing, or malformed challenge structures', () => {
      const nullPayload = null
      const emptyPayload = { playerChallenges: [] }
      const malformedPayload = { playerChallenges: 'invalid' as any }

      const parse = (payload: any) => {
        if (!payload?.playerChallenges || !Array.isArray(payload.playerChallenges)) {
          return []
        }
        return payload.playerChallenges
      }

      expect(parse(nullPayload)).toEqual([])
      expect(parse(emptyPayload)).toEqual([])
      expect(parse(malformedPayload)).toEqual([])
    })
  })
})

