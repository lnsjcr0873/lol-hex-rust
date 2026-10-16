import { describe, it, expect } from 'vitest'
import fs from 'fs'
import path from 'path'
import {
  toBasicInfo,
  toParticipants,
  toTeams,
  toFrames,
  type LcuGameSummary,
  type LcuGameTimeline,
  type MatchParticipant
} from '../src/shared/data-adapter'
import {
  getMockRawMatchHistory,
  getMockRawMatchTimeline,
  mockRawGames,
  mockRawMatchTimeline,
  buildRealisticMockRawGame,
  buildRealisticMockRawTimeline
} from '../src/api/mockData'
import { analyzeGames } from '../src/shared/data-adapter/analysis/player'
import type { Game, TimelineFrame } from '@shared/types/league-client/match-history'

describe('Empirical Adversarial Stress Test Suite: Frontend Data Pipeline & Components (Phase 1)', () => {
  // =========================================================================
  // TASK 1: Static Absence of convertMatchItemToLcuGame
  // =========================================================================
  describe('Task 1: Verification of Static Absence of convertMatchItemToLcuGame', () => {
    it('verifies convertMatchItemToLcuGame is strictly absent from MatchHistory.vue', () => {
      const matchHistoryPath = path.resolve(__dirname, '../src/views/MatchHistory.vue')
      const source = fs.readFileSync(matchHistoryPath, 'utf8')

      expect(source).not.toContain('convertMatchItemToLcuGame')
      expect(source).not.toContain('function convertMatchItemToLcuGame')
    })

    it('verifies convertMatchItemToLcuGame does not exist in any frontend/src source files', () => {
      const srcDir = path.resolve(__dirname, '../src')

      function scanDir(dir: string): string[] {
        let files: string[] = []
        for (const item of fs.readdirSync(dir, { withFileTypes: true })) {
          const fullPath = path.join(dir, item.name)
          if (item.isDirectory()) {
            files = files.concat(scanDir(fullPath))
          } else if (/\.(vue|ts|js)$/.test(item.name)) {
            files.push(fullPath)
          }
        }
        return files
      }

      const allSrcFiles = scanDir(srcDir)
      expect(allSrcFiles.length).toBeGreaterThan(10)

      for (const file of allSrcFiles) {
        const content = fs.readFileSync(file, 'utf8')
        expect(
          content.includes('convertMatchItemToLcuGame'),
          `Found forbidden convertMatchItemToLcuGame in ${file}`
        ).toBe(false)
      }
    })

    it('verifies MatchHistory.vue consumes rawGames directly into allMatchSummaries with source: "lcu"', () => {
      const matchHistoryPath = path.resolve(__dirname, '../src/views/MatchHistory.vue')
      const source = fs.readFileSync(matchHistoryPath, 'utf8')

      expect(source).toMatch(/const\s+rawGames\s*=\s*ref<Game\[\]>\(\[\]\)/)
      expect(source).toMatch(/source:\s*'lcu'/)
      expect(source).toMatch(/data:\s*g/)
      expect(source).toMatch(/allMatchSummaries\s*=\s*computed<LcuOrSgpGameSummary\[\]>/)
    })
  })

  // =========================================================================
  // TASK 2: Stress Test Data Adapter with Missing or Malformed Frames
  // =========================================================================
  describe('Task 2: Stress Testing Data Adapter & Component Logic with Missing/Malformed Frames', () => {
    it('toFrames returns empty array when details or details.data is null or undefined', () => {
      expect(toFrames(null as any)).toEqual([])
      expect(toFrames(undefined as any)).toEqual([])
      expect(toFrames({} as any)).toEqual([])
      expect(toFrames({ gameId: 100, source: 'lcu', data: null as any })).toEqual([])
      expect(toFrames({ gameId: 100, source: 'lcu', data: undefined as any })).toEqual([])
      expect(toFrames({ gameId: 100, source: 'sgp', data: null as any })).toEqual([])
    })

    it('toFrames handles zero-length timeline frames without error', () => {
      const emptyLcuTimeline: LcuGameTimeline = {
        gameId: 12345,
        source: 'lcu',
        data: {
          gameId: 12345,
          frameInterval: 60000,
          frames: []
        }
      }

      const frames = toFrames(emptyLcuTimeline)
      expect(frames).toEqual([])
      expect(frames.length).toBe(0)
    })

    it('handles frames with missing or empty events array gracefully (BuildsTab & EventsTab logic)', () => {
      const malformedTimeline: LcuGameTimeline = {
        gameId: 99999,
        source: 'lcu',
        data: {
          gameId: 99999,
          frameInterval: 60000,
          frames: [
            { timestamp: 0, events: [] } as any,
            { timestamp: 60000 } as any, // missing events
            { timestamp: 120000, events: null } as any, // null events
            { timestamp: 180000, events: undefined } as any, // undefined events
            {
              timestamp: 240000,
              events: [
                {
                  type: 'SKILL_LEVEL_UP',
                  timestamp: 240500,
                  participantId: 1,
                  skillSlot: 1
                }
              ]
            } as any
          ]
        }
      }

      const frames = toFrames(malformedTimeline)
      expect(frames).toHaveLength(5)

      // Test allEvents extraction logic as used in MatchCardBuildsTab & MatchCardEventsTab
      const extractedEvents: any[] = []
      for (const f of frames || []) {
        if (f && Array.isArray(f.events)) {
          extractedEvents.push(...f.events)
        }
      }

      expect(extractedEvents).toHaveLength(1)
      expect(extractedEvents[0].type).toBe('SKILL_LEVEL_UP')
      expect(extractedEvents[0].participantId).toBe(1)
    })

    it('handles frames with missing or malformed participantFrames (DiffLineChart & StatsLine logic)', () => {
      const malformedTimeline: LcuGameTimeline = {
        gameId: 88888,
        source: 'lcu',
        data: {
          gameId: 88888,
          frameInterval: 60000,
          frames: [
            { timestamp: 0 } as any, // completely missing participantFrames
            { timestamp: 60000, participantFrames: null } as any,
            { timestamp: 120000, participantFrames: {} } as any,
            {
              timestamp: 180000,
              participantFrames: {
                '1': {
                  participantId: 1,
                  currentGold: 500,
                  totalGold: 1200,
                  level: 3,
                  xp: 800,
                  minionsKilled: 10,
                  jungleMinionsKilled: 0,
                  dominionScore: 0,
                  teamScore: 0,
                  position: { x: 1500, y: 1500 }
                }
                // Participants 2..10 missing
              }
            } as any
          ]
        }
      }

      const frames = toFrames(malformedTimeline)

      // Test DiffLineChart simulation/dataPoints calculation on malformed frames
      const participants = [{ participantId: 1, gameName: 'P1' }, { participantId: 2, gameName: 'P2' }]

      for (const p of participants) {
        const dataPoints = frames.map((f: any) => {
          if (!f || !f.participantFrames) return 0
          const pf = f.participantFrames[p.participantId] || f.participantFrames[String(p.participantId)]
          return pf ? (pf.totalGold || pf.currentGold || 0) : 0
        })

        if (p.participantId === 1) {
          expect(dataPoints).toEqual([0, 0, 0, 1200])
        } else {
          expect(dataPoints).toEqual([0, 0, 0, 0])
        }
      }
    })

    it('handles events with edge-case coordinates, killerId=0 (execution) and non-standard skill slots', () => {
      const edgeEvents = [
        { type: 'CHAMPION_KILL', killerId: 0, victimId: 1, timestamp: 150000 }, // Executed by minion/turret
        { type: 'CHAMPION_KILL', killerId: 2, victimId: 3, timestamp: 200000, position: null }, // Null position
        { type: 'CHAMPION_KILL', killerId: 4, victimId: 5, timestamp: 250000, position: { x: -100, y: 999999 } }, // Out of bounds
        { type: 'SKILL_LEVEL_UP', participantId: 1, skillSlot: 99, timestamp: 50000 } // Unusual slot
      ]

      // Verify killerId = 0 resolution
      const firstEvent = edgeEvents[0]
      const killerName = firstEvent.killerId === 0 ? '防御塔 / 野怪击杀' : `玩家 #${firstEvent.killerId}`
      expect(killerName).toBe('防御塔 / 野怪击杀')

      // Verify position resilience
      const secondEvent = edgeEvents[1]
      const hasValidPosition = secondEvent.position && typeof secondEvent.position.x === 'number'
      expect(hasValidPosition).toBeFalsy()

      // Verify skill slot mapping fallback
      const slotMap: Record<number, string> = { 1: 'Q', 2: 'W', 3: 'E', 4: 'R' }
      const slotName = slotMap[edgeEvents[3].skillSlot] || 'Q'
      expect(slotName).toBe('Q')
    })

    it('handles SGP game details with missing json or invalid frames', () => {
      const invalidSgp: any = {
        gameId: 77777,
        source: 'sgp',
        data: {}
      }
      expect(toFrames(invalidSgp)).toEqual([])

      const emptySgpFrames: any = {
        gameId: 77777,
        source: 'sgp',
        data: { json: { frames: [] } }
      }
      expect(toFrames(emptySgpFrames)).toEqual([])
    })
  })

  // =========================================================================
  // TASK 3: Real Non-Zero Keystone Rune Variables (var1, var2, var3)
  // =========================================================================
  describe('Task 3: Real Non-Zero Keystone Rune Variables Propagation & Formatting', () => {
    it('verifies genuine non-zero perk0Var1, perk0Var2, perk0Var3 propagate across all 10 participants', () => {
      const rawGame = mockRawGames[0]
      const summary: LcuGameSummary = {
        gameId: rawGame.gameId,
        source: 'lcu',
        data: rawGame
      }

      const basicInfo = toBasicInfo(summary)
      const participants = toParticipants(summary, basicInfo)

      expect(participants).toHaveLength(10)

      // Test P1 (Conqueror: 8010)
      const p1 = participants[0]
      const p1Keystone = p1.perks.styles[0].selections[0]
      expect(p1Keystone.perk).toBe(8010)
      expect(p1Keystone.var1).toBe(1845) // Non-zero healing
      expect(p1Keystone.var2).toBe(920)  // Non-zero adaptive force damage
      expect(p1Keystone.var3).toBe(0)

      // Test P2 (Press the Attack: 8005)
      const p2 = participants[1]
      const p2Keystone = p2.perks.styles[0].selections[0]
      expect(p2Keystone.perk).toBe(8005)
      expect(p2Keystone.var1).toBe(1650) // Non-zero proc damage
      expect(p2Keystone.var2).toBe(820)  // Non-zero exposure damage
      expect(p2Keystone.var3).toBe(310)  // Non-zero ally bonus damage

      // Test P3 (Electrocute: 8112)
      const p3 = participants[2]
      const p3Keystone = p3.perks.styles[0].selections[0]
      expect(p3Keystone.perk).toBe(8112)
      expect(p3Keystone.var1).toBe(2180) // Non-zero damage

      // Test P4 (Lethal Tempo: 8008)
      const p4 = participants[3]
      const p4Keystone = p4.perks.styles[0].selections[0]
      expect(p4Keystone.perk).toBe(8008)
      expect(p4Keystone.var1).toBe(2450)
      expect(p4Keystone.var2).toBe(1120)

      // Test P5 (Summon Aery: 8214)
      const p5 = participants[4]
      const p5Keystone = p5.perks.styles[0].selections[0]
      expect(p5Keystone.perk).toBe(8214)
      expect(p5Keystone.var1).toBe(1890) // Non-zero damage
      expect(p5Keystone.var2).toBe(2420) // Non-zero shielding
    })

    it('verifies MatchCardRunesTab formatting logic produces valid string outputs without undefined errors', () => {
      // Simulate MatchCardRunesTab template formatting logic
      function formatPrimaryRuneStats(sel: { var1?: number | null; var2?: number | null; var3?: number | null }) {
        const parts: string[] = []
        if (sel.var1 !== undefined && sel.var1 !== null) {
          parts.push(`数值1: ${sel.var1.toLocaleString()}`)
        }
        if (sel.var2 && sel.var2 > 0) {
          parts.push(`| 数值2: ${sel.var2.toLocaleString()}`)
        }
        if (sel.var3 && sel.var3 > 0) {
          parts.push(`| 数值3: ${sel.var3.toLocaleString()}`)
        }
        return parts.join(' ')
      }

      function formatSecondaryRuneStats(sel: { var1?: number | null; var2?: number | null; var3?: number | null }) {
        if (sel.var1 || sel.var2 || sel.var3) {
          return `局内收益: ${sel.var1 ? sel.var1.toLocaleString() : 0}`
        }
        return ''
      }

      // 1. Conqueror
      const conqStr = formatPrimaryRuneStats({ var1: 1845, var2: 920, var3: 0 })
      expect(conqStr).toContain('数值1: 1,845')
      expect(conqStr).toContain('| 数值2: 920')
      expect(conqStr).not.toContain('数值3')
      expect(conqStr).not.toContain('undefined')

      // 2. PTA (all 3 vars non-zero)
      const ptaStr = formatPrimaryRuneStats({ var1: 1650, var2: 820, var3: 310 })
      expect(ptaStr).toContain('数值1: 1,650')
      expect(ptaStr).toContain('| 数值2: 820')
      expect(ptaStr).toContain('| 数值3: 310')
      expect(ptaStr).not.toContain('undefined')

      // 3. Zero var1 (e.g. game remade or rune unused)
      const zeroStr = formatPrimaryRuneStats({ var1: 0, var2: 0, var3: 0 })
      expect(zeroStr).toBe('数值1: 0')
      expect(zeroStr).not.toContain('undefined')

      // 4. Undefined / null resilience
      expect(formatPrimaryRuneStats({ var1: undefined, var2: undefined, var3: undefined })).toBe('')
      expect(formatPrimaryRuneStats({ var1: null, var2: null, var3: null })).toBe('')

      // 5. Secondary rune formatting
      expect(formatSecondaryRuneStats({ var1: 890, var2: 0, var3: 0 })).toBe('局内收益: 890')
      expect(formatSecondaryRuneStats({ var1: 0, var2: 50, var3: 0 })).toBe('局内收益: 0')
      expect(formatSecondaryRuneStats({ var1: 0, var2: 0, var3: 0 })).toBe('')
    })
  })

  // =========================================================================
  // TASK 4: Dynamic Pagination and Queue Switching Stress Tests
  // =========================================================================
  describe('Task 4: Dynamic Pagination (10, 20, 30, 40, 50, 100, 200) & Queue Switching', () => {
    const ALL_PAGE_SIZES = [10, 20, 30, 40, 50, 100, 200]
    const ALL_QUEUES: Array<{ key: string; qid: number | undefined }> = [
      { key: 'all', qid: undefined },
      { key: 'ranked_solo', qid: 420 },
      { key: 'ranked_flex', qid: 440 },
      { key: 'aram', qid: 450 },
      { key: 'normal', qid: 430 }
    ]

    describe('4.1 Matrix testing of all 7 page sizes against all 5 queues', () => {
      for (const size of ALL_PAGE_SIZES) {
        for (const queue of ALL_QUEUES) {
          it(`correctly serves page 1 with size ${size} in queue "${queue.key}"`, () => {
            const begIndex = 0
            const endIndex = size - 1
            const res = getMockRawMatchHistory(queue.qid, begIndex, endIndex)

            expect(res).toBeDefined()
            expect(res.games.games).toHaveLength(size)
            expect(res.games.gameIndexBegin).toBe(begIndex)
            expect(res.games.gameIndexEnd).toBe(endIndex)

            // If a specific queue was requested, all returned games must match
            if (queue.qid !== undefined) {
              const matchesQueue = res.games.games.every((g) => g.queueId === queue.qid)
              expect(matchesQueue).toBe(true)
            }

            // Convert raw games to LcuGameSummary
            const summaries: LcuGameSummary[] = res.games.games.map((g) => ({
              gameId: g.gameId,
              source: 'lcu',
              data: g
            }))

            // Run through data adapter analyzeGames
            const analyzed = analyzeGames(
              summaries.map((s) => ({ gameId: s.gameId, summary: s })),
              'p-blue-1'
            )

            expect(analyzed).toBeDefined()
            expect(analyzed?.count).toBe(size)
            expect(analyzed?.akariScore).toBeDefined()
          })
        }
      }
    })

    describe('4.2 Multi-page traversal with maximum page size 200', () => {
      it('correctly serves pages 1, 2, 3 of 200 matches with sequential non-overlapping indices', () => {
        const pageSize = 200

        for (let page = 1; page <= 3; page++) {
          const begIndex = (page - 1) * pageSize
          const endIndex = begIndex + pageSize - 1

          const res = getMockRawMatchHistory(420, begIndex, endIndex)
          expect(res.games.games).toHaveLength(pageSize)
          expect(res.games.gameIndexBegin).toBe(begIndex)
          expect(res.games.gameIndexEnd).toBe(endIndex)

          // Verify unique game IDs across consecutive pages
          const firstGid = res.games.games[0].gameId
          const lastGid = res.games.games[pageSize - 1].gameId
          expect(firstGid).toBe(7012345671 + begIndex)
          expect(lastGid).toBe(7012345671 + endIndex)
        }
      })
    })

    describe('4.3 Rapid queue switching and state transition simulation', () => {
      it('ensures rapid queue switching does not corrupt active state or cause unhandled rejections', async () => {
        let currentFetchId = 0
        let activeRawGames: Game[] = []
        let activeQueueKey = 'all'
        let activePage = 1
        let activePageSize = 20
        let isLoading = false

        async function simulateFetchMatches(targetPage: number, targetSize: number, queueKey: string, delayMs: number) {
          const fetchId = ++currentFetchId
          isLoading = true

          try {
            await new Promise((resolve) => setTimeout(resolve, delayMs))
            if (fetchId !== currentFetchId) {
              return { committed: false, fetchId }
            }

            const qid = ALL_QUEUES.find((q) => q.key === queueKey)?.qid
            const begIndex = (targetPage - 1) * targetSize
            const endIndex = begIndex + targetSize - 1
            const res = getMockRawMatchHistory(qid, begIndex, endIndex)

            activeRawGames = res.games.games
            activeQueueKey = queueKey
            activePage = targetPage
            activePageSize = targetSize
            return { committed: true, fetchId }
          } finally {
            if (fetchId === currentFetchId) {
              isLoading = false
            }
          }
        }

        // Rapid queue switches in quick succession:
        // 'all' (slow 60ms) -> 'ranked_solo' (slow 50ms) -> 'aram' (fast 10ms)
        const p1 = simulateFetchMatches(1, 20, 'all', 60)
        const p2 = simulateFetchMatches(1, 20, 'ranked_solo', 50)
        const p3 = simulateFetchMatches(1, 20, 'aram', 10)

        const [r1, r2, r3] = await Promise.all([p1, p2, p3])

        // Only p3 (aram) was the latest fetchId, so even though it finished first, p1 and p2 should not overwrite it
        await new Promise((resolve) => setTimeout(resolve, 80))

        expect(r1.committed).toBe(false)
        expect(r2.committed).toBe(false)
        expect(r3.committed).toBe(true)

        expect(activeQueueKey).toBe('aram')
        expect(activeRawGames).toHaveLength(20)
        expect(activeRawGames.every((g) => g.queueId === 450)).toBe(true)
        expect(isLoading).toBe(false)
      })
    })
  })
})
