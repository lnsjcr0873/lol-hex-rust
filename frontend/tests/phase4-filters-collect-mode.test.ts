import { describe, it, expect, vi } from 'vitest'
import {
  createDefaultSimpleFilter,
  createDefaultAdvancedFilter,
  hasActiveFilter,
  buildPredicateFromSimple,
  buildPredicateFromAst,
  type SimpleMatchHistoryFilterState,
  type AdvancedFilterState
} from '../src/components/player-tab/widgets/match-history-filters/filter-state'
import type { LcuOrSgpGameSummary } from '../src/shared/data-adapter/wrapper'
import type { Game } from '../src/shared/types/league-client/match-history'
import { mockRawGames } from '../src/api/mockData'

function createMockSummary(gameId: number, overrides: Partial<Game> = {}): LcuOrSgpGameSummary {
  const baseGame = mockRawGames.find(g => g.gameId === gameId) || mockRawGames[0]
  const game: Game = {
    ...baseGame,
    gameId,
    ...overrides
  }
  return {
    gameId,
    source: 'lcu',
    data: game
  }
}

describe('Phase 4: Advanced Match History Filter & AST Compiler Tests', () => {
  const selfPuuid = 'mock_puuid_self'

  describe('1. Simple Filter State & Predicate Compilation', () => {
    it('detects default simple filter as inactive', () => {
      const simple = createDefaultSimpleFilter()
      const advanced = createDefaultAdvancedFilter()
      expect(hasActiveFilter(simple, advanced, 'simple')).toBe(false)
    })

    it('detects simple filter active when any criterion is set', () => {
      const advanced = createDefaultAdvancedFilter()
      
      const s1 = createDefaultSimpleFilter()
      s1.winLoss = 'win'
      expect(hasActiveFilter(s1, advanced, 'simple')).toBe(true)

      const s2 = createDefaultSimpleFilter()
      s2.positions = ['TOP']
      expect(hasActiveFilter(s2, advanced, 'simple')).toBe(true)

      const s3 = createDefaultSimpleFilter()
      s3.championIds = [266]
      expect(hasActiveFilter(s3, advanced, 'simple')).toBe(true)
    })

    it('filters games by win/loss correctly', () => {
      const simpleWin: SimpleMatchHistoryFilterState = {
        ...createDefaultSimpleFilter(),
        winLoss: 'win'
      }
      const predWin = buildPredicateFromSimple(simpleWin, selfPuuid)

      // Game 1 where self won
      const winningGame = createMockSummary(1001, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, teamId: 100, stats: { win: true } } as any]
      })
      // Game 2 where self lost
      const losingGame = createMockSummary(1002, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, teamId: 100, stats: { win: false } } as any]
      })

      expect(predWin(winningGame)).toBe(true)
      expect(predWin(losingGame)).toBe(false)
    })

    it('filters games by champion ID', () => {
      const simpleChamp: SimpleMatchHistoryFilterState = {
        ...createDefaultSimpleFilter(),
        championIds: [266, 103] // Aatrox or Ahri
      }
      const pred = buildPredicateFromSimple(simpleChamp, selfPuuid)

      const gameAatrox = createMockSummary(2001, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, championId: 266, stats: {} } as any]
      })
      const gameYasuo = createMockSummary(2002, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, championId: 157, stats: {} } as any]
      })

      expect(pred(gameAatrox)).toBe(true)
      expect(pred(gameYasuo)).toBe(false)
    })

    it('filters games by position / lane', () => {
      const simplePos: SimpleMatchHistoryFilterState = {
        ...createDefaultSimpleFilter(),
        positions: ['TOP', 'MIDDLE']
      }
      const pred = buildPredicateFromSimple(simplePos, selfPuuid)

      const gameTop = createMockSummary(3001, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, timeline: { lane: 'TOP' }, stats: {} } as any]
      })
      const gameJungle = createMockSummary(3002, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, timeline: { lane: 'JUNGLE' }, stats: {} } as any]
      })

      expect(pred(gameTop)).toBe(true)
      expect(pred(gameJungle)).toBe(false)
    })
  })

  describe('2. Advanced Filter AST Tree & Predicate Compilation', () => {
    it('detects empty AST root as inactive', () => {
      const simple = createDefaultSimpleFilter()
      const advanced = createDefaultAdvancedFilter()
      expect(hasActiveFilter(simple, advanced, 'advanced')).toBe(false)
    })

    it('compiles AST node for KDA and kill thresholds', () => {
      const advanced: AdvancedFilterState = {
        rootId: 'root',
        nodeMap: {
          root: {
            id: 'root',
            type: 'and',
            children: ['node_kills', 'node_win']
          },
          node_kills: {
            id: 'node_kills',
            type: 'numberBetween',
            params: { metric: 'kills', min: 10, max: 99 }
          },
          node_win: {
            id: 'node_win',
            type: 'isWin'
          }
        }
      }

      const simple = createDefaultSimpleFilter()
      expect(hasActiveFilter(simple, advanced, 'advanced')).toBe(true)

      const pred = buildPredicateFromAst(advanced, selfPuuid)

      // Matching: 12 kills and won
      const matchGame = createMockSummary(4001, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, stats: { kills: 12, win: true } } as any]
      })

      // Non-matching: 5 kills and won
      const lowKillGame = createMockSummary(4002, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, stats: { kills: 5, win: true } } as any]
      })

      // Non-matching: 15 kills and lost
      const lostGame = createMockSummary(4003, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, stats: { kills: 15, win: false } } as any]
      })

      expect(pred(matchGame)).toBe(true)
      expect(pred(lowKillGame)).toBe(false)
      expect(pred(lostGame)).toBe(false)
    })

    it('compiles nested OR groups in AST', () => {
      const advanced: AdvancedFilterState = {
        rootId: 'root',
        nodeMap: {
          root: {
            id: 'root',
            type: 'or',
            children: ['champ_aatrox', 'champ_darius']
          },
          champ_aatrox: {
            id: 'champ_aatrox',
            type: 'isChampion',
            params: { championId: 266 }
          },
          champ_darius: {
            id: 'champ_darius',
            type: 'isChampion',
            params: { championId: 122 }
          }
        }
      }

      const pred = buildPredicateFromAst(advanced, selfPuuid)

      const g1 = createMockSummary(5001, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, championId: 266, stats: {} } as any]
      })
      const g2 = createMockSummary(5002, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, championId: 122, stats: {} } as any]
      })
      const g3 = createMockSummary(5003, {
        participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
        participants: [{ participantId: 1, championId: 99, stats: {} } as any]
      })

      expect(pred(g1)).toBe(true)
      expect(pred(g2)).toBe(true)
      expect(pred(g3)).toBe(false)
    })
  })

  describe('3. Collect Mode Simulation & Pagination Engine', () => {
    it('simulates collecting games over multiple iterations until expected count is reached', async () => {
      const filter: SimpleMatchHistoryFilterState = {
        ...createDefaultSimpleFilter(),
        winLoss: 'win'
      }
      const pred = buildPredicateFromSimple(filter, selfPuuid)

      // Mock match history pages: 10 games per batch, alternating win/loss
      const createPage = (batchIndex: number) => {
        return Array.from({ length: 10 }).map((_, i) => {
          const gameId = batchIndex * 10 + i + 1
          const isWin = i % 2 === 0 // 5 wins, 5 losses per page
          return {
            gameId,
            teams: [{ teamId: 100, win: isWin ? 'Win' : 'Fail' }],
            participantIdentities: [{ participantId: 1, player: { puuid: selfPuuid } } as any],
            participants: [{ participantId: 1, stats: { win: isWin } } as any]
          } as Game
        })
      }

      const expectedCount = 12
      const maxIteration = 5
      const collected: Game[] = []
      let iteration = 0

      while (collected.length < expectedCount && iteration < maxIteration) {
        const batch = createPage(iteration)
        for (const g of batch) {
          const summary: LcuOrSgpGameSummary = { gameId: g.gameId, source: 'lcu', data: g }
          if (pred(summary)) {
            collected.push(g)
            if (collected.length >= expectedCount) break
          }
        }
        iteration++
      }

      // Each iteration gives 5 wins. Iteration 0: 5, Iteration 1: 5 (total 10), Iteration 2: 2 (total 12).
      expect(collected.length).toBe(12)
      expect(iteration).toBe(3)
      expect(collected.every(g => g.participants[0].stats.win)).toBe(true)
    })

    it('terminates early when abort signal is triggered', async () => {
      let aborted = false
      const collected: Game[] = []
      const maxIteration = 10

      for (let iteration = 0; iteration < maxIteration; iteration++) {
        if (iteration === 2) {
          aborted = true
        }
        if (aborted) break
        collected.push({ gameId: iteration } as Game)
      }

      expect(collected.length).toBe(2)
      expect(aborted).toBe(true)
    })
  })
})
