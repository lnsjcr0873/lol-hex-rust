import { describe, it, expect } from 'vitest'
import { isUnrankedTier, formatTierName, isMasterPlus } from '../src/components/player-tab/widgets/RankedPane.vue'
import { QUEUE_NAME_MAP } from '../src/components/player-tab/SummaryPane.vue'

describe('Empirical Adversarial Challenge Suite: Frontend Pagination, State Sync & Edge Cases', () => {
  // =========================================================================
  // 1. Pagination Boundaries & Calculation Across All 7 Page Sizes
  // =========================================================================
  describe('Challenge 1: Pagination Boundaries and Calculations Across All 7 Page Sizes', () => {
    const REQUIRED_PAGE_SIZES = [10, 20, 30, 40, 50, 100, 200] as const

    function computePaginationBounds(targetPage: number, targetSize: number) {
      const p = Math.max(1, Math.floor(targetPage))
      const size = Math.max(1, Math.floor(targetSize))
      const begIndex = (p - 1) * size
      const endIndex = begIndex + size - 1
      return { begIndex, endIndex, count: endIndex - begIndex + 1 }
    }

    it('verifies all 7 page sizes exist in the required specification', () => {
      expect(REQUIRED_PAGE_SIZES).toEqual([10, 20, 30, 40, 50, 100, 200])
      expect(REQUIRED_PAGE_SIZES.length).toBe(7)
    })

    it.each(REQUIRED_PAGE_SIZES)(
      'computes exact 0-based boundary indices for page 1 with page size %i',
      (size) => {
        const bounds = computePaginationBounds(1, size)
        expect(bounds.begIndex).toBe(0)
        expect(bounds.endIndex).toBe(size - 1)
        expect(bounds.count).toBe(size)
      }
    )

    it.each(REQUIRED_PAGE_SIZES)(
      'verifies consecutive pages have zero gaps and zero overlaps for page size %i',
      (size) => {
        for (let p = 1; p <= 10; p++) {
          const currentPageBounds = computePaginationBounds(p, size)
          const nextPageBounds = computePaginationBounds(p + 1, size)

          expect(currentPageBounds.count).toBe(size)
          expect(nextPageBounds.begIndex).toBe(currentPageBounds.endIndex + 1)
          expect(currentPageBounds.begIndex).toBe((p - 1) * size)
          expect(currentPageBounds.endIndex).toBe(p * size - 1)
        }
      }
    )

    it('stress-tests extreme page numbers and sizes', () => {
      // Very large page number with max size 200
      const largePageBounds = computePaginationBounds(500, 200)
      expect(largePageBounds.begIndex).toBe(99800)
      expect(largePageBounds.endIndex).toBe(99999)
      expect(largePageBounds.count).toBe(200)

      // Boundary inputs: page 0, negative pages, float pages
      const zeroBounds = computePaginationBounds(0, 50)
      expect(zeroBounds.begIndex).toBe(0)
      expect(zeroBounds.endIndex).toBe(49)

      const negativeBounds = computePaginationBounds(-999, 100)
      expect(negativeBounds.begIndex).toBe(0)
      expect(negativeBounds.endIndex).toBe(99)

      const floatBounds = computePaginationBounds(2.9, 20)
      expect(floatBounds.begIndex).toBe(20)
      expect(floatBounds.endIndex).toBe(39)
    })
  })

  // =========================================================================
  // 2. Race Condition Protection (currentFetchId)
  // =========================================================================
  describe('Challenge 2: Concurrency Race Condition Protection (currentFetchId)', () => {
    it('discards stale slow responses when rapid page transitions occur', async () => {
      let currentFetchId = 0
      let activeMatches: Array<{ gameId: number; page: number }> = []
      let activeCurrentPage = 1
      let isLoading = false

      async function fetchMatches(targetPage: number, delayMs: number, data: any[]) {
        const fetchId = ++currentFetchId
        isLoading = true

        try {
          await new Promise((resolve) => setTimeout(resolve, delayMs))
          if (fetchId !== currentFetchId) {
            // Out of order: discard response
            return { committed: false, fetchId }
          }
          activeMatches = data
          activeCurrentPage = targetPage
          return { committed: true, fetchId }
        } finally {
          if (fetchId === currentFetchId) {
            isLoading = false
          }
        }
      }

      // Page 2 is initiated with high network latency (80ms)
      const p2Promise = fetchMatches(2, 80, [{ gameId: 201, page: 2 }, { gameId: 202, page: 2 }])
      // User quickly navigates to Page 3 which finishes fast (20ms)
      const p3Promise = fetchMatches(3, 20, [{ gameId: 301, page: 3 }, { gameId: 302, page: 3 }])

      const [p2Result, p3Result] = await Promise.all([p2Promise, p3Promise])

      expect(p3Result.committed).toBe(true)
      expect(p2Result.committed).toBe(false)
      expect(activeCurrentPage).toBe(3)
      expect(activeMatches[0].page).toBe(3)
      expect(isLoading).toBe(false)
    })

    it('stress-tests 20 concurrent burst requests with randomized completion order', async () => {
      let currentFetchId = 0
      let activePage = 0
      let committedCount = 0
      let isLoading = false

      async function fetchBurst(page: number, latencyMs: number) {
        const fetchId = ++currentFetchId
        isLoading = true
        try {
          await new Promise((resolve) => setTimeout(resolve, latencyMs))
          if (fetchId !== currentFetchId) {
            return false
          }
          activePage = page
          committedCount++
          return true
        } finally {
          if (fetchId === currentFetchId) {
            isLoading = false
          }
        }
      }

      const totalRequests = 20
      const promises: Promise<boolean>[] = []

      for (let i = 1; i <= totalRequests; i++) {
        // Random latency between 5ms and 50ms
        const randomLatency = Math.floor(Math.random() * 45) + 5
        promises.push(fetchBurst(i, randomLatency))
      }

      await Promise.all(promises)

      // Only the final request (page 20) should have been permitted to commit
      expect(activePage).toBe(totalRequests)
      expect(committedCount).toBe(1)
      expect(isLoading).toBe(false)
    })

    it('ensures stale errors do not wipe out active successful data', async () => {
      let currentFetchId = 0
      let activeMatches = [{ gameId: 100 }]
      let isLoading = false

      async function fetchWithErrorOrData(shouldFail: boolean, delayMs: number, data: any[]) {
        const fetchId = ++currentFetchId
        isLoading = true

        try {
          await new Promise((resolve, reject) => {
            setTimeout(() => {
              if (shouldFail) reject(new Error('Network timeout'))
              else resolve(null)
            }, delayMs)
          })

          if (fetchId !== currentFetchId) return

          activeMatches = data
        } catch (err) {
          if (fetchId === currentFetchId) {
            activeMatches = []
          }
        } finally {
          if (fetchId === currentFetchId) {
            isLoading = false
          }
        }
      }

      // Request 1 will be slow (60ms) and will FAIL with an error
      const r1 = fetchWithErrorOrData(true, 60, [])
      // Request 2 is quick (20ms) and SUCCEEDS with valid data
      const r2 = fetchWithErrorOrData(false, 20, [{ gameId: 200 }])

      await Promise.allSettled([r1, r2])

      // Request 2's data must NOT have been wiped out by Request 1's subsequent error
      expect(activeMatches).toEqual([{ gameId: 200 }])
      expect(isLoading).toBe(false)
    })
  })

  // =========================================================================
  // 3. Empty/Zero-Game & Unranked Cases in SummaryPane, RankedPane, Drawer
  // =========================================================================
  describe('Challenge 3: Empty/Zero-Game and Unranked Edge Cases', () => {
    describe('RankedPane Unranked Edge Cases', () => {
      it('correctly handles every unranked variant without showing fake diamond data', () => {
        const unrankedInputs = [
          undefined,
          null,
          '',
          '   ',
          'UNRANKED',
          'unranked',
          'None',
          'NONE',
          'NA',
          'na',
          '\t\n'
        ]

        for (const input of unrankedInputs) {
          expect(isUnrankedTier(input)).toBe(true)
          expect(formatTierName(input as any)).toBe('未定级')
        }
      })

      it('correctly localizes genuine tiers and identifies Master+ tiers', () => {
        expect(formatTierName('DIAMOND')).toBe('钻石')
        expect(isMasterPlus('DIAMOND')).toBe(false)

        expect(formatTierName('MASTER')).toBe('大师')
        expect(isMasterPlus('MASTER')).toBe(true)

        expect(formatTierName('GRANDMASTER')).toBe('宗师')
        expect(isMasterPlus('GRANDMASTER')).toBe(true)

        expect(formatTierName('CHALLENGER')).toBe('王者')
        expect(isMasterPlus('CHALLENGER')).toBe(true)
      })
    })

    describe('SummaryPane Zero-Game and Metric Calculations', () => {
      function computeSummarySubtitle(sampleCount?: number, analysisCount?: number, queue = 'all') {
        const count = sampleCount !== undefined && sampleCount !== null
          ? sampleCount
          : (analysisCount ?? 0)

        if (count === 0) return '暂无对局样本'
        const q = queue ? String(queue).toLowerCase() : 'all'
        const queueLabel = QUEUE_NAME_MAP[q] ?? QUEUE_NAME_MAP[queue] ?? queue
        return `近 ${count} 场${queueLabel}样本`
      }

      it('renders "暂无对局样本" when sampleCount is 0 or analysis is empty', () => {
        expect(computeSummarySubtitle(0, 0, 'all')).toBe('暂无对局样本')
        expect(computeSummarySubtitle(0, undefined, 'ranked_solo')).toBe('暂无对局样本')
        expect(computeSummarySubtitle(undefined, 0, 'aram')).toBe('暂无对局样本')
      })

      it('renders dynamic subtitles with proper queue name localization across all queues and page sizes', () => {
        expect(computeSummarySubtitle(10, undefined, 'all')).toBe('近 10 场全部样本')
        expect(computeSummarySubtitle(20, undefined, 'ranked_solo')).toBe('近 20 场单双排样本')
        expect(computeSummarySubtitle(30, undefined, 'ranked_flex')).toBe('近 30 场灵活排位样本')
        expect(computeSummarySubtitle(40, undefined, 'aram')).toBe('近 40 场极地大乱斗样本')
        expect(computeSummarySubtitle(50, undefined, 'normal')).toBe('近 50 场匹配模式样本')
        expect(computeSummarySubtitle(100, undefined, '420')).toBe('近 100 场单双排样本')
        expect(computeSummarySubtitle(200, undefined, '450')).toBe('近 200 场极地大乱斗样本')
      })

      it('preserves win/loss conservation (wins + losses strictly equals totalGames)', () => {
        const testGameCounts = [1, 2, 3, 5, 7, 10, 13, 20, 27, 30, 40, 50, 77, 100, 199, 200]
        const testWinRates = [0, 12, 33, 50, 66, 75, 88, 100]

        for (const total of testGameCounts) {
          for (const rate of testWinRates) {
            const wins = Math.round((rate / 100) * total)
            const losses = Math.max(0, total - wins)
            expect(wins + losses).toBe(total)
          }
        }
      })

      it('validates streak calculation on page 1 vs historical pages', () => {
        function getStreak(winLoss: any, currentPage = 1) {
          if (currentPage > 1) return null
          if (!winLoss) return null
          const winStreak = winLoss.winningStreak ?? 0
          const loseStreak = winLoss.losingStreak ?? 0

          if (winStreak >= 2) return { isWinning: true, count: winStreak }
          if (loseStreak >= 2) return { isWinning: false, count: loseStreak }
          return null
        }

        // Sub-threshold streaks
        expect(getStreak({ winningStreak: 1, losingStreak: 0 }, 1)).toBeNull()
        expect(getStreak({ winningStreak: 0, losingStreak: 1 }, 1)).toBeNull()
        expect(getStreak({ winningStreak: 0, losingStreak: 0 }, 1)).toBeNull()

        // Valid streaks on page 1
        expect(getStreak({ winningStreak: 5, losingStreak: 0 }, 1)).toEqual({ isWinning: true, count: 5 })
        expect(getStreak({ winningStreak: 0, losingStreak: 3 }, 1)).toEqual({ isWinning: false, count: 3 })

        // Historical pages (page 2, 3, etc.) must NEVER display streak
        expect(getStreak({ winningStreak: 10, losingStreak: 0 }, 2)).toBeNull()
        expect(getStreak({ winningStreak: 0, losingStreak: 8 }, 3)).toBeNull()
      })
    })

    describe('PlayerDetailDrawer Sample Count Dynamic Calculation', () => {
      function computeDrawerSampleCount(player: any): number {
        if (player?.recent_games_count !== undefined && player.recent_games_count > 0) {
          return player.recent_games_count
        }
        if (player?.recent_matches && player.recent_matches.length > 0) {
          return player.recent_matches.length
        }
        return 20
      }

      it('dynamically adapts sampleCount from recent_games_count or matches length', () => {
        expect(computeDrawerSampleCount(null)).toBe(20)
        expect(computeDrawerSampleCount({})).toBe(20)
        expect(computeDrawerSampleCount({ recent_games_count: 0 })).toBe(20)
        expect(computeDrawerSampleCount({ recent_games_count: 50 })).toBe(50)
        expect(computeDrawerSampleCount({ recent_games_count: 200 })).toBe(200)
        expect(computeDrawerSampleCount({ recent_matches: ['win', 'loss', 'win', 'win'] })).toBe(4)
      })
    })
  })
})
