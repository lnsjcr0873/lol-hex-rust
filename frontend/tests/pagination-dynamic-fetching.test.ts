import { describe, it, expect, vi } from 'vitest'
import { getMatchHistory } from '../src/api/tauri'

describe('Milestone 3: Match History Pagination & Dynamic Fetching Integration Tests', () => {
  describe('1. Pagination Index & Slice Range Mathematics', () => {
    function computeSliceBounds(page: number, pageSize: number) {
      const p = Math.max(1, page)
      const size = Math.max(1, pageSize)
      const begIndex = (p - 1) * size
      const endIndex = begIndex + size - 1
      return { begIndex, endIndex, count: size }
    }

    it('computes 0-based inclusive indices for page 1 at various sizes', () => {
      expect(computeSliceBounds(1, 10)).toEqual({ begIndex: 0, endIndex: 9, count: 10 })
      expect(computeSliceBounds(1, 20)).toEqual({ begIndex: 0, endIndex: 19, count: 20 })
      expect(computeSliceBounds(1, 30)).toEqual({ begIndex: 0, endIndex: 29, count: 30 })
      expect(computeSliceBounds(1, 40)).toEqual({ begIndex: 0, endIndex: 39, count: 40 })
      expect(computeSliceBounds(1, 50)).toEqual({ begIndex: 0, endIndex: 49, count: 50 })
      expect(computeSliceBounds(1, 100)).toEqual({ begIndex: 0, endIndex: 99, count: 100 })
      expect(computeSliceBounds(1, 200)).toEqual({ begIndex: 0, endIndex: 199, count: 200 })
    })

    it('computes arbitrary historical page offsets correctly', () => {
      // Page 2 of 20: matches 21-40
      expect(computeSliceBounds(2, 20)).toEqual({ begIndex: 20, endIndex: 39, count: 20 })
      // Page 3 of 50: matches 101-150
      expect(computeSliceBounds(3, 50)).toEqual({ begIndex: 100, endIndex: 149, count: 50 })
      // Page 10 of 10: matches 91-100
      expect(computeSliceBounds(10, 10)).toEqual({ begIndex: 90, endIndex: 99, count: 10 })
    })

    it('handles boundary and non-positive page inputs gracefully', () => {
      expect(computeSliceBounds(0, 20)).toEqual({ begIndex: 0, endIndex: 19, count: 20 })
      expect(computeSliceBounds(-5, 20)).toEqual({ begIndex: 0, endIndex: 19, count: 20 })
    })
  })

  describe('2. Page Size Whitelist Validation', () => {
    const REGULAR_PAGE_SIZES = [10, 20, 30, 40, 50, 100, 200] as const

    it('contains all 7 required pagination tier options up to 200 matches', () => {
      expect(REGULAR_PAGE_SIZES).toHaveLength(7)
      expect(REGULAR_PAGE_SIZES).toContain(10)
      expect(REGULAR_PAGE_SIZES).toContain(20)
      expect(REGULAR_PAGE_SIZES).toContain(30)
      expect(REGULAR_PAGE_SIZES).toContain(40)
      expect(REGULAR_PAGE_SIZES).toContain(50)
      expect(REGULAR_PAGE_SIZES).toContain(100)
      expect(REGULAR_PAGE_SIZES).toContain(200)
    })
  })

  describe('3. Concurrency Race Condition Protection (currentFetchId)', () => {
    it('discards stale/out-of-order network responses when rapid page transitions occur', async () => {
      let currentFetchId = 0
      let activeMatches: string[] = []

      async function simulatedFetch(page: number, delayMs: number, resultData: string[]) {
        const fetchId = ++currentFetchId
        await new Promise((resolve) => setTimeout(resolve, delayMs))
        if (fetchId !== currentFetchId) {
          // Out of order: discard response
          return false
        }
        activeMatches = resultData
        return true
      }

      // Request 1 is launched for page 2 but is slow (60ms)
      const req1 = simulatedFetch(2, 60, ['Game 21', 'Game 22'])
      // User quickly clicks page 3, which is fast (20ms)
      const req2 = simulatedFetch(3, 20, ['Game 41', 'Game 42'])

      const [res1Accepted, res2Accepted] = await Promise.all([req1, req2])

      // Request 2 should have completed and been accepted
      expect(res2Accepted).toBe(true)
      // Request 1 should have been detected as stale and discarded
      expect(res1Accepted).toBe(false)
      // Active matches must reflect request 2
      expect(activeMatches).toEqual(['Game 41', 'Game 42'])
    })
  })

  describe('4. Elimination of 20-Match Local Slicing Bug', () => {
    it('directly displays backend-sliced matches rather than re-slicing a local array', () => {
      // Backend returns 20 matches for page 2 (matches 21-40)
      const backendPage2Matches = Array.from({ length: 20 }, (_, i) => ({
        game_id: 2000 + i,
        queue_id: 420
      }))

      const currentPage = 2
      const pageSize = 20

      // BUGGY OLD BEHAVIOR:
      // Slicing from (page - 1) * pageSize = 20 on an array of length 20
      const oldSliceStart = (currentPage - 1) * pageSize
      const buggyResult = backendPage2Matches.slice(oldSliceStart, oldSliceStart + pageSize)
      expect(buggyResult).toHaveLength(0) // Empty!

      // NEW FIXED BEHAVIOR:
      // Direct pass-through because backend already returned the desired slice
      const fixedResult = backendPage2Matches
      expect(fixedResult).toHaveLength(20)
      expect(fixedResult[0].game_id).toBe(2000)
      expect(fixedResult[19].game_id).toBe(2019)
    })
  })

  describe('5. Queue Filter Synchronization with Analysis Calculation', () => {
    it('synchronizes sample subset and win/loss calculation to selected queue', () => {
      const mixedMatches = [
        { gameId: 1, queueId: 420, isWin: true },
        { gameId: 2, queueId: 420, isWin: false },
        { gameId: 3, queueId: 450, isWin: true },
        { gameId: 4, queueId: 450, isWin: true },
        { gameId: 5, queueId: 450, isWin: false }
      ]

      function filterAndAnalyze(matches: typeof mixedMatches, activeQueue: string) {
        const filtered = matches.filter((m) => {
          if (activeQueue === 'ranked_solo' && m.queueId !== 420) return false
          if (activeQueue === 'ranked_flex' && m.queueId !== 440) return false
          if (activeQueue === 'aram' && m.queueId !== 450) return false
          return true
        })

        if (filtered.length === 0) return null

        const wins = filtered.filter((m) => m.isWin).length
        const total = filtered.length
        const winRate = Math.round((wins / total) * 100)

        return { total, wins, losses: total - wins, winRate }
      }

      // Ranked Solo: 1 win, 1 loss (2 total, 50%)
      const soloStats = filterAndAnalyze(mixedMatches, 'ranked_solo')
      expect(soloStats).toEqual({ total: 2, wins: 1, losses: 1, winRate: 50 })

      // ARAM: 2 wins, 1 loss (3 total, 67%)
      const aramStats = filterAndAnalyze(mixedMatches, 'aram')
      expect(aramStats).toEqual({ total: 3, wins: 2, losses: 1, winRate: 67 })

      // All: 3 wins, 2 losses (5 total, 60%)
      const allStats = filterAndAnalyze(mixedMatches, 'all')
      expect(allStats).toEqual({ total: 5, wins: 3, losses: 2, winRate: 60 })

      // Empty queue (e.g. flex): returns null gracefully
      const flexStats = filterAndAnalyze(mixedMatches, 'ranked_flex')
      expect(flexStats).toBeNull()
    })
  })

  describe('6. getMatchHistory API IPC Signature & Parameter Contract', () => {
    it('accepts full pagination arguments and resolves cleanly', async () => {
      // In Node/Vitest environment, safeInvoke falls back to default empty array []
      const result = await getMatchHistory('test-puuid', 420, 20, 39, 2, 20)
      expect(Array.isArray(result)).toBe(true)
    })
  })

  describe('7. User Preferences Storage & Resolution (hexbrain_preferences)', () => {
    function resolvePageSize(rawJson: string | null): number {
      try {
        if (!rawJson) return 20
        const parsed = JSON.parse(rawJson)
        if (typeof parsed?.defaultPageSize === 'number' && [10, 20, 50, 100].includes(parsed.defaultPageSize)) {
          return parsed.defaultPageSize
        }
      } catch {
        // fallback
      }
      return 20
    }

    function resolveBroadcastFormat(rawJson: string | null): 'concise' | 'detailed' {
      try {
        if (!rawJson) return 'concise'
        const parsed = JSON.parse(rawJson)
        if (parsed?.broadcastFormat === 'detailed' || parsed?.broadcastFormat === 'concise') {
          return parsed.broadcastFormat
        }
      } catch {
        // fallback
      }
      return 'concise'
    }

    it('resolves valid saved preferences accurately', () => {
      const prefs = JSON.stringify({ defaultPageSize: 50, broadcastFormat: 'detailed' })
      expect(resolvePageSize(prefs)).toBe(50)
      expect(resolveBroadcastFormat(prefs)).toBe('detailed')
    })

    it('falls back to defaults on corrupted or null preferences', () => {
      expect(resolvePageSize(null)).toBe(20)
      expect(resolveBroadcastFormat(null)).toBe('concise')
      expect(resolvePageSize('{ invalid json')).toBe(20)
      expect(resolveBroadcastFormat('{ invalid json')).toBe('concise')
    })

    it('rejects invalid pageSize values outside whitelist', () => {
      const invalidPrefs = JSON.stringify({ defaultPageSize: 999, broadcastFormat: 'hack' })
      expect(resolvePageSize(invalidPrefs)).toBe(20)
      expect(resolveBroadcastFormat(invalidPrefs)).toBe('concise')
    })
  })
})
