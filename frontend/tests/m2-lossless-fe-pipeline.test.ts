import { describe, it, expect } from 'vitest'
import fs from 'fs'
import path from 'path'
import { getRawMatchHistory, getRawMatchTimeline } from '../src/api/tauri'
import {
  mockRawGames,
  mockRawMatchHistory,
  getMockRawMatchHistory,
  mockRawMatchTimeline,
  getMockRawMatchTimeline,
  buildRealisticMockRawGame
} from '../src/api/mockData'
import {
  toBasicInfo,
  toParticipants,
  toTeams,
  toFrames,
  type LcuGameSummary,
  type LcuGameTimeline
} from '../src/shared/data-adapter'
import { resolveParticipantPosition, mapLcuDataToPerks } from '../src/shared/data-adapter/match-history/participants'
import { computeAggregatedPositions } from '../src/shared/data-adapter/analysis/player/aggregate/positions'

describe('Milestone 2 (R2): Lossless Frontend Data Pipeline & Direct Contract Alignment', () => {
  // =========================================================================
  // 1. Elimination of convertMatchItemToLcuGame from Source
  // =========================================================================
  describe('1. Static Code Analysis: Elimination of convertMatchItemToLcuGame', () => {
    it('verifies convertMatchItemToLcuGame is completely absent from MatchHistory.vue', () => {
      const matchHistoryPath = path.resolve(__dirname, '../src/views/MatchHistory.vue')
      const source = fs.readFileSync(matchHistoryPath, 'utf8')

      expect(source).not.toContain('convertMatchItemToLcuGame')
      expect(source).not.toContain('function convertMatchItemToLcuGame')
    })

    it('verifies allMatchSummaries is directly mapped from rawGames without synthetic conversion', () => {
      const matchHistoryPath = path.resolve(__dirname, '../src/views/MatchHistory.vue')
      const source = fs.readFileSync(matchHistoryPath, 'utf8')

      expect(source).toContain('const rawGames = ref<Game[]>([])')
      expect(source).toContain('return rawGames.value.map((g) => {')
      expect(source).toContain("source: 'lcu'")
      expect(source).toContain('data: g')
    })

    it('verifies handleLoadDetails feeds raw timeline into detailsMap', () => {
      const matchHistoryPath = path.resolve(__dirname, '../src/views/MatchHistory.vue')
      const source = fs.readFileSync(matchHistoryPath, 'utf8')

      expect(source).toContain('getRawMatchTimeline(gameId)')
      expect(source).toContain("source: 'lcu'")
      expect(source).toContain('data: markRaw(timeline)')
    })
  })

  // =========================================================================
  // 2. Tauri IPC Raw Endpoints Contracts
  // =========================================================================
  describe('2. Tauri IPC Raw Endpoints & Mock Fallbacks', () => {
    it('getRawMatchHistory returns MatchHistory container with games array', async () => {
      const res = await getRawMatchHistory('test-puuid', 420, 0, 19)
      expect(res).toBeDefined()
      expect(res.accountId).toBe(1001)
      expect(res.games).toBeDefined()
      expect(Array.isArray(res.games.games)).toBe(true)
      expect(res.games.games.length).toBe(20)
    })

    it('getRawMatchTimeline returns GameTimeline container with frames array', async () => {
      const res = await getRawMatchTimeline(7012345671)
      expect(res).toBeDefined()
      expect(Array.isArray(res.frames)).toBe(true)
      expect(res.frames.length).toBeGreaterThanOrEqual(15)
    })
  })

  // =========================================================================
  // 3. Lossless Direct Contract Alignment (toBasicInfo, toParticipants, toTeams)
  // =========================================================================
  describe('3. Unified Data Adapter Contract Compliance', () => {
    const rawGame = mockRawGames[0]
    const summary: LcuGameSummary = {
      gameId: rawGame.gameId,
      source: 'lcu',
      data: rawGame
    }

    it('toBasicInfo parses lossless game metadata', () => {
      const basic = toBasicInfo(summary)
      expect(basic.gameId).toBe(rawGame.gameId)
      expect(basic.gameDuration).toBe(rawGame.gameDuration)
      expect(basic.gameMode).toBe('CLASSIC')
      expect(basic.mapId).toBe(11)
      expect(basic.dataSource).toBe('lcu')
    })

    it('toParticipants parses all 10 participants without dropping fields', () => {
      const basic = toBasicInfo(summary)
      const participants = toParticipants(summary, basic)

      expect(participants).toHaveLength(10)

      // Player 1 (Self: Aatrox)
      const p1 = participants[0]
      expect(p1.participantId).toBe(1)
      expect(p1.championId).toBe(266)
      expect(p1.puuid).toBe('p-blue-1')
      expect(p1.gameName).toBe('飞天小女警')
      expect(p1.kills).toBe(rawGame.participants[0].stats.kills)
      expect(p1.deaths).toBe(rawGame.participants[0].stats.deaths)
      expect(p1.assists).toBe(rawGame.participants[0].stats.assists)
      expect(p1.totalDamageDealtToChampions).toBeGreaterThan(0)
      expect(p1.goldEarned).toBeGreaterThan(0)
    })

    it('toTeams parses Team 100 and Team 200 objectives faithfully', () => {
      const basic = toBasicInfo(summary)
      const participants = toParticipants(summary, basic)
      const teamsResult = toTeams(summary, basic, participants)

      expect(teamsResult.teamStatMap['TEAM-100']).toBeDefined()
      expect(teamsResult.teamStatMap['TEAM-200']).toBeDefined()

      const blueTeam = teamsResult.teamStatMap['TEAM-100']
      expect(blueTeam.win).toBe(true)
      expect(blueTeam.teamInfo?.objectives.tower.kills).toBe(9)
      expect(blueTeam.teamInfo?.objectives.dragon.kills).toBe(3)
      expect(blueTeam.teamInfo?.objectives.baron.kills).toBe(1)
      expect(blueTeam.teamInfo?.objectives.horde.kills).toBe(5)
    })
  })

  // =========================================================================
  // 4. Keystone Runes Genuine Values (var1, var2, var3)
  // =========================================================================
  describe('4. Keystone Runes Genuine Non-Zero Stat Values', () => {
    it('preserves genuine non-zero perk0Var1, perk0Var2, perk0Var3', () => {
      const rawGame = mockRawGames[0]
      const summary: LcuGameSummary = {
        gameId: rawGame.gameId,
        source: 'lcu',
        data: rawGame
      }

      const basic = toBasicInfo(summary)
      const participants = toParticipants(summary, basic)
      const p1 = participants[0]

      expect(p1.perks).toBeDefined()
      expect(p1.perks?.styles).toBeDefined()
      expect(p1.perks?.styles?.length).toBeGreaterThanOrEqual(2)

      // Primary style selection 0 (Keystone: Conqueror 8010)
      const keystone = p1.perks?.styles?.[0].selections[0]
      expect(keystone).toBeDefined()
      expect(keystone?.perk).toBe(8010)
      expect(keystone?.var1).toBe(1845) // Genuine non-zero healing!
      expect(keystone?.var2).toBe(920) // Genuine non-zero damage bonus!
      expect(keystone?.var3).toBe(0)

      // Participant 2 (PTA: 8005)
      const p2 = participants[1]
      const pta = p2.perks?.styles?.[0].selections[0]
      expect(pta?.perk).toBe(8005)
      expect(pta?.var1).toBe(1650)
      expect(pta?.var2).toBe(820)
      expect(pta?.var3).toBe(310)

      // Participant 3 (Electrocute: 8112)
      const p3 = participants[2]
      const electro = p3.perks?.styles?.[0].selections[0]
      expect(electro?.perk).toBe(8112)
      expect(electro?.var1).toBe(2180)
    })
  })

  // =========================================================================
  // 5. 1-18 Skill Level-Up Timestamps
  // =========================================================================
  describe('5. 1-18 Skill Level-Up Timestamps in Timeline', () => {
    const timeline = mockRawMatchTimeline
    const timelineDetails: LcuGameTimeline = {
      gameId: 7012345671,
      source: 'lcu',
      data: timeline
    }

    it('toFrames extracts minute-by-minute frames and event stream', () => {
      const frames = toFrames(timelineDetails)
      expect(frames.length).toBeGreaterThanOrEqual(15)

      // Collect all events across all frames
      const allEvents: any[] = []
      for (const f of frames) {
        if (Array.isArray(f.events)) {
          allEvents.push(...f.events)
        }
      }

      // Filter skill level up events for Participant 1
      const p1Skills = allEvents.filter(
        (e) => e.type === 'SKILL_LEVEL_UP' && e.participantId === 1
      )

      // Must have up to 18 skill leveling events
      expect(p1Skills.length).toBe(18)

      // Verify progression of skill slots and ascending timestamps
      const expectedSlots = [1, 3, 2, 1, 1, 4, 1, 3, 1, 3, 4, 3, 3, 2, 2, 4, 2, 2]
      for (let i = 0; i < 18; i++) {
        expect(p1Skills[i].skillSlot).toBe(expectedSlots[i])
        expect(p1Skills[i].timestamp).toBeGreaterThan(0)
        if (i > 0) {
          expect(p1Skills[i].timestamp).toBeGreaterThan(p1Skills[i - 1].timestamp)
        }
      }
    })
  })

  // =========================================================================
  // 6. Kill Positions ({ x, y }) in Events
  // =========================================================================
  describe('6. Authentic Map Kill Positions ({ x, y })', () => {
    const timeline = mockRawMatchTimeline
    const timelineDetails: LcuGameTimeline = {
      gameId: 7012345671,
      source: 'lcu',
      data: timeline
    }

    it('extracts genuine champion kill coordinates', () => {
      const frames = toFrames(timelineDetails)
      const killEvents = frames.flatMap((f) =>
        (f.events || []).filter((e) => e.type === 'CHAMPION_KILL')
      )

      expect(killEvents.length).toBeGreaterThanOrEqual(4)

      for (const kill of killEvents) {
        expect(kill.position).toBeDefined()
        expect(typeof kill.position.x).toBe('number')
        expect(typeof kill.position.y).toBe('number')
        expect(kill.position.x).toBeGreaterThan(0)
        expect(kill.position.y).toBeGreaterThan(0)
      }

      // Verify specific first blood kill position
      const firstBlood = killEvents[0]
      expect(firstBlood.killerId).toBe(1)
      expect(firstBlood.victimId).toBe(6)
      expect(firstBlood.position.x).toBe(2150)
      expect(firstBlood.position.y).toBe(11400)
    })

    it('extracts building and monster kill positions', () => {
      const frames = toFrames(timelineDetails)
      const buildingKills = frames.flatMap((f) =>
        (f.events || []).filter((e) => e.type === 'BUILDING_KILL')
      )
      const monsterKills = frames.flatMap((f) =>
        (f.events || []).filter((e) => e.type === 'ELITE_MONSTER_KILL')
      )

      expect(buildingKills.length).toBeGreaterThanOrEqual(1)
      expect(buildingKills[0].position.x).toBe(4318)
      expect(buildingKills[0].position.y).toBe(13875)

      expect(monsterKills.length).toBeGreaterThanOrEqual(2)
      expect(monsterKills[0].position.x).toBe(9866)
      expect(monsterKills[0].position.y).toBe(4414)
    })
  })

  // =========================================================================
  // 7. Dynamic Pagination & Queue Filtering with Raw Games
  // =========================================================================
  describe('7. Dynamic Pagination & Queue Filtering with Raw Games', () => {
    it('generates arbitrary large slices up to 200 matches smoothly in demo mode', () => {
      const page1_50 = getMockRawMatchHistory(420, 0, 49)
      expect(page1_50.games.games).toHaveLength(50)
      expect(page1_50.games.games.every((g) => g.queueId === 420)).toBe(true)

      const page1_200 = getMockRawMatchHistory(undefined, 0, 199)
      expect(page1_200.games.games).toHaveLength(200)

      const page2_20 = getMockRawMatchHistory(450, 20, 39)
      expect(page2_20.games.games).toHaveLength(20)
      expect(page2_20.games.games.every((g) => g.queueId === 450)).toBe(true)
    })
  })

  // =========================================================================
  // 8. Position Resolution and NaN-safe Position Aggregation
  // =========================================================================
  describe('8. Position Resolution and NaN-safe Position Aggregation', () => {
    it('correctly maps Support and ADC lane/role to UTILITY and BOTTOM', () => {
      // Support: lane BOTTOM, role DUO_SUPPORT -> UTILITY
      expect(resolveParticipantPosition({}, { lane: 'BOTTOM', role: 'DUO_SUPPORT' })).toBe('UTILITY')
      // Support: lane NONE, role DUO_SUPPORT -> UTILITY
      expect(resolveParticipantPosition({}, { lane: 'NONE', role: 'DUO_SUPPORT' })).toBe('UTILITY')
      // ADC: lane BOTTOM, role DUO_CARRY -> BOTTOM
      expect(resolveParticipantPosition({}, { lane: 'BOTTOM', role: 'DUO_CARRY' })).toBe('BOTTOM')
      // Mid: lane MIDDLE -> MIDDLE
      expect(resolveParticipantPosition({}, { lane: 'MIDDLE', role: 'SOLO' })).toBe('MIDDLE')
      // SGP stats teamPosition SUPPORT -> UTILITY
      expect(resolveParticipantPosition({ teamPosition: 'SUPPORT' }, {})).toBe('UTILITY')
    })

    it('safely aggregates positions without producing NaN for NONE or unassigned lanes', () => {
      const mockGames: any[] = [
        { participant: { position: 'UTILITY' } },
        { participant: { position: 'NONE' } },
        { participant: { position: 'TOP' } },
        { participant: { position: null } },
        { participant: { position: 'UTILITY' } },
      ]
      const aggregated = computeAggregatedPositions(mockGames)
      expect(aggregated).not.toBeNull()
      expect(aggregated!.UTILITY).toBe(2)
      expect(aggregated!.TOP).toBe(1)
      expect(aggregated!.BOTTOM).toBe(0)
      expect(aggregated!.MIDDLE).toBe(0)
      expect(aggregated!.JUNGLE).toBe(0)
      expect(Number.isNaN((aggregated as any).NONE)).toBe(false)
    })

    it('extracts statPerks shards from LCU participant stats', () => {
      const mockParticipantWithStats: any = {
        stats: {
          perk0: 8112,
          perkPrimaryStyle: 8100,
          perkSubStyle: 8000,
          statPerk0: 5008, // Adaptive force
          statPerk1: 5008, // Adaptive force
          statPerk2: 5001  // Health
        }
      }
      const perks = mapLcuDataToPerks(mockParticipantWithStats)
      expect(perks.statPerks).not.toBeNull()
      expect(perks.statPerks!.offense).toBe(5008)
      expect(perks.statPerks!.flex).toBe(5008)
      expect(perks.statPerks!.defense).toBe(5001)

      const mockParticipantEmpty: any = {
        stats: {}
      }
      expect(mapLcuDataToPerks(mockParticipantEmpty).statPerks).toBeNull()
    })
  })
})
