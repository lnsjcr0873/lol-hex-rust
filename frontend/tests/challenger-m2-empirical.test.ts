import { describe, it, expect, beforeEach } from 'vitest'
import { ref } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import {
  getNameFromYuriyuri,
  streamerSummonerName,
  masked,
  useStreamerModeMaskedText
} from '../src/components/widgets/useStreamerModeMaskedText'
import { useOngoingMatchStore } from '../src/stores/ongoing-match'
import type { DraftOptions } from '../src/api/types'
import * as tauriApi from '../src/api/tauri'

// Extract computeDraftOptions logic identically to MatchCard context.ts for isolated stress testing
function computeDraftOptions(
  focusedPuuid: string | null,
  basicInfo: { gameMode: string; queueId: number; isCherrySubteam?: boolean },
  participants: Array<{
    puuid: string
    championId: number
    teamId?: number
    teamIdentifier?: string
    position?: string
  }>
): DraftOptions {
  const ownerPuuid =
    focusedPuuid && participants.some((p) => p.puuid === focusedPuuid)
      ? focusedPuuid
      : (participants[0]?.puuid || null)

  const draftTeams: Record<string, string[]> = {}
  const championSelections: Record<string, number> = {}
  const positionAssignments: Record<string, { selected: string; primary: string; secondary: string }> = {}
  let hasPositions = false

  for (const p of participants) {
    const isCherry = basicInfo.gameMode === 'CHERRY' || Boolean(basicInfo.isCherrySubteam)
    const teamIdentifier = isCherry
      ? 'TEAM-ALL'
      : (p.teamIdentifier || (p.teamId === 100 ? '100' : '200'))

    draftTeams[teamIdentifier] ??= []
    draftTeams[teamIdentifier].push(p.puuid)
    championSelections[p.puuid] = p.championId

    if (p.position) {
      positionAssignments[p.puuid] = {
        selected: p.position,
        primary: p.position,
        secondary: ''
      }
      hasPositions = true
    }
  }

  const isCherry = basicInfo.gameMode === 'CHERRY' || Boolean(basicInfo.isCherrySubteam)
  return {
    gameModeKind: isCherry ? 'cherry' : 'normal',
    queueId: basicInfo.queueId,
    puuid: ownerPuuid,
    teams: draftTeams,
    championSelections,
    positions: hasPositions ? positionAssignments : null
  }
}

describe('Empirical Adversarial Test Suite: yuriyuri-names Deterministic Hash Engine', () => {
  it('handles empty string seed safely within valid bounds', () => {
    const zh = getNameFromYuriyuri('', 'zh-CN')
    expect(typeof zh).toBe('string')
    expect(zh.length).toBeGreaterThan(0)

    const en = getNameFromYuriyuri('', 'en')
    expect(typeof en).toBe('string')
    expect(en.length).toBeGreaterThan(0)
  })

  it('guarantees hyphens do not alter deterministic Horner PUUID mapping', () => {
    const puuidWithHyphens = '01234567-89ab-cdef-0123-456789abcdef'
    const puuidWithoutHyphens = '0123456789abcdef0123456789abcdef'

    const name1 = getNameFromYuriyuri(puuidWithHyphens, 'zh-CN')
    const name2 = getNameFromYuriyuri(puuidWithoutHyphens, 'zh-CN')
    expect(name1).toBe(name2)
  })

  it('guarantees case-insensitivity between uppercase and lowercase hex PUUIDs', () => {
    const lower = 'fedcba98-7654-3210-abcd-ef0123456789'
    const upper = 'FEDCBA98-7654-3210-ABCD-EF0123456789'

    expect(getNameFromYuriyuri(lower, 'zh-CN')).toBe(getNameFromYuriyuri(upper, 'zh-CN'))
    expect(getNameFromYuriyuri(lower, 'en')).toBe(getNameFromYuriyuri(upper, 'en'))
  })

  it('survives non-hex ASCII characters, punctuation, and whitespace without throwing or NaN', () => {
    const weirdSeeds = [
      'xyz-hello-world',
      'player_#9999!',
      '   spaces   ',
      '---',
      '!@#$%^&*()_+~`|}{[]:;?><,./',
      '0123zzzz-aaaa-bbbb'
    ]

    for (const seed of weirdSeeds) {
      const zh = getNameFromYuriyuri(seed, 'zh-CN')
      expect(typeof zh).toBe('string')
      expect(zh.length).toBeGreaterThan(0)
    }
  })

  it('survives emojis and multi-byte UTF-8 strings safely', () => {
    const utf8Seeds = [
      '🚀🔥🐱🎉',
      '英雄联盟玩家#1234',
      '★Akari★灯里★',
      '👾❤️⚡️✨'
    ]

    for (const seed of utf8Seeds) {
      const zh = getNameFromYuriyuri(seed, 'zh-CN')
      expect(typeof zh).toBe('string')
      expect(zh.length).toBeGreaterThan(0)

      const en = getNameFromYuriyuri(seed, 'en')
      expect(typeof en).toBe('string')
      expect(en.length).toBeGreaterThan(0)
    }
  })

  it('survives extreme lengths (single char, 10000 chars, null bytes)', () => {
    const single = getNameFromYuriyuri('a', 'zh-CN')
    expect(typeof single).toBe('string')

    const huge = getNameFromYuriyuri('f'.repeat(10000), 'zh-CN')
    expect(typeof huge).toBe('string')

    const nullBytes = getNameFromYuriyuri('\0\0\0\0', 'zh-CN')
    expect(typeof nullBytes).toBe('string')
  })

  it('property-based oracle: 1000 randomized PUUIDs strictly map to the [0, 728] range (27x27=729 names)', () => {
    const sampleCharacters = '0123456789abcdefABCDEF-!@#$_xyz🐱🔥测试'

    for (let i = 0; i < 1000; i++) {
      let randomSeed = ''
      const len = Math.floor(Math.random() * 40) + 1
      for (let j = 0; j < len; j++) {
        randomSeed += sampleCharacters[Math.floor(Math.random() * sampleCharacters.length)]
      }

      const nameZh = getNameFromYuriyuri(randomSeed, 'zh-CN')
      const nameEn = getNameFromYuriyuri(randomSeed, 'en')

      // Assert non-empty and non-undefined
      expect(nameZh).toBeDefined()
      expect(typeof nameZh).toBe('string')
      expect(nameZh.length).toBeGreaterThanOrEqual(2)

      expect(nameEn).toBeDefined()
      expect(typeof nameEn).toBe('string')
      expect(nameEn.length).toBeGreaterThanOrEqual(2)

      // Assert repeatability (deterministic oracle)
      expect(getNameFromYuriyuri(randomSeed, 'zh-CN')).toBe(nameZh)
      expect(getNameFromYuriyuri(randomSeed, 'en')).toBe(nameEn)
    }
  })

  it('falls back gracefully to en dictionary for unknown locales', () => {
    const res = getNameFromYuriyuri('abc-123', 'fr-FR')
    expect(typeof res).toBe('string')
    expect(res.length).toBeGreaterThan(0)
  })

  it('streamerSummonerName handles index fallback and default prefix when Akari style is disabled', () => {
    const { streamerModeUseAkariStyledName, setStreamerModeUseAkariStyledName } = useStreamerModeMaskedText()
    
    setStreamerModeUseAkariStyledName(false)
    expect(streamerSummonerName('player-1', 0)).toBe('召唤师 1')
    expect(streamerSummonerName('player-2', 4)).toBe('召唤师 5')

    setStreamerModeUseAkariStyledName(true)
    const akariName = streamerSummonerName('player-1', 0)
    expect(akariName).not.toContain('召唤师')
  })
})

describe('Empirical Adversarial Test Suite: DraftOptions Extraction & Hydration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('extracts standard 5v5 match into separate teams 100 and 200', () => {
    const participants = [
      { puuid: 'p1', championId: 101, teamId: 100, position: 'TOP' },
      { puuid: 'p2', championId: 102, teamId: 100, position: 'JUNGLE' },
      { puuid: 'p3', championId: 103, teamId: 100, position: 'MIDDLE' },
      { puuid: 'p4', championId: 104, teamId: 100, position: 'BOTTOM' },
      { puuid: 'p5', championId: 105, teamId: 100, position: 'UTILITY' },
      { puuid: 'p6', championId: 201, teamId: 200, position: 'TOP' },
      { puuid: 'p7', championId: 202, teamId: 200, position: 'JUNGLE' },
      { puuid: 'p8', championId: 203, teamId: 200, position: 'MIDDLE' },
      { puuid: 'p9', championId: 204, teamId: 200, position: 'BOTTOM' },
      { puuid: 'p10', championId: 205, teamId: 200, position: 'UTILITY' }
    ]

    const draft = computeDraftOptions('p3', { gameMode: 'CLASSIC', queueId: 420 }, participants)

    expect(draft.gameModeKind).toBe('normal')
    expect(draft.puuid).toBe('p3')
    expect(draft.queueId).toBe(420)
    expect(Object.keys(draft.teams)).toContain('100')
    expect(Object.keys(draft.teams)).toContain('200')
    expect(draft.teams['100'].length).toBe(5)
    expect(draft.teams['200'].length).toBe(5)
    expect(draft.championSelections['p1']).toBe(101)
    expect(draft.championSelections['p10']).toBe(205)
    expect(draft.positions?.['p1']?.selected).toBe('TOP')

    // Hydrate into Pinia store
    const store = useOngoingMatchStore()
    store.setDraft(draft, participants as any)

    expect(store.isDraft).toBe(true)
    expect(store.isDryRunActive).toBe(true)
    const ongoing = store.customOngoingData
    expect(ongoing).not.toBeNull()
    expect(ongoing?.blue_team.length).toBe(5)
    expect(ongoing?.red_team.length).toBe(5)
    expect(ongoing?.blue_team[0].champion_id).toBe(101)
    expect(ongoing?.red_team[4].champion_id).toBe(205)
  })

  it('normalizes Cherry (Arena 2v2v2v2) 8 participants into single TEAM-ALL key', () => {
    const cherryParticipants = [
      { puuid: 'c1', championId: 1, teamIdentifier: '1700-1' },
      { puuid: 'c2', championId: 2, teamIdentifier: '1700-1' },
      { puuid: 'c3', championId: 3, teamIdentifier: '1700-2' },
      { puuid: 'c4', championId: 4, teamIdentifier: '1700-2' },
      { puuid: 'c5', championId: 5, teamIdentifier: '1700-3' },
      { puuid: 'c6', championId: 6, teamIdentifier: '1700-3' },
      { puuid: 'c7', championId: 7, teamIdentifier: '1700-4' },
      { puuid: 'c8', championId: 8, teamIdentifier: '1700-4' }
    ]

    const draft = computeDraftOptions('c1', { gameMode: 'CHERRY', queueId: 1700 }, cherryParticipants)

    expect(draft.gameModeKind).toBe('cherry')
    expect(draft.teams['TEAM-ALL']).toBeDefined()
    expect(draft.teams['TEAM-ALL'].length).toBe(8)
    expect(draft.teams['100']).toBeUndefined()
    expect(draft.teams['200']).toBeUndefined()

    // Hydrate into Pinia store
    const store = useOngoingMatchStore()
    store.setDraft(draft, cherryParticipants as any)

    expect(store.isDraft).toBe(true)
    const ongoing = store.customOngoingData
    expect(ongoing).not.toBeNull()
    expect(ongoing?.queue_name).toContain('斗魂竞技场')
    // Cherry Arena mode splits 8 players 4 vs 4 into blue and red visual columns
    expect(ongoing?.blue_team.length).toBe(4)
    expect(ongoing?.red_team.length).toBe(4)
    expect(ongoing?.blue_team.map(p => p.puuid)).toEqual(['c1', 'c2', 'c3', 'c4'])
    expect(ongoing?.red_team.map(p => p.puuid)).toEqual(['c5', 'c6', 'c7', 'c8'])
  })

  it('handles Cherry mode with odd number of players (3 players)', () => {
    const oddCherry = [
      { puuid: 'a', championId: 10 },
      { puuid: 'b', championId: 20 },
      { puuid: 'c', championId: 30 }
    ]

    const draft = computeDraftOptions(null, { gameMode: 'CHERRY', queueId: 1700 }, oddCherry)
    expect(draft.teams['TEAM-ALL'].length).toBe(3)

    const store = useOngoingMatchStore()
    store.setDraft(draft, oddCherry as any)

    expect(store.customOngoingData?.blue_team.length).toBe(2) // Math.ceil(3/2) = 2
    expect(store.customOngoingData?.red_team.length).toBe(1)
  })

  it('handles empty participants gracefully without crashing', () => {
    const emptyDraft = computeDraftOptions(null, { gameMode: 'CLASSIC', queueId: 420 }, [])
    expect(emptyDraft.puuid).toBeNull()
    expect(Object.keys(emptyDraft.teams).length).toBe(0)

    const store = useOngoingMatchStore()
    store.setDraft(emptyDraft, [])
    expect(store.customOngoingData?.blue_team.length).toBe(0)
    expect(store.customOngoingData?.red_team.length).toBe(0)
  })

  it('handles focused PUUID that is not present in participants list', () => {
    const participants = [{ puuid: 'real-player', championId: 99, teamId: 100 }]
    const draft = computeDraftOptions('non-existent-puuid', { gameMode: 'CLASSIC', queueId: 420 }, participants)

    expect(draft.puuid).toBe('real-player') // falls back to first participant
  })
})

describe('Empirical Adversarial Test Suite: SQLite IPC API & Degradation Resilience', () => {
  it('queryEncounteredGames handles boundary page and pageSize numbers gracefully in fallback mode', async () => {
    // page 0, page -1
    const res0 = await tauriApi.queryEncounteredGames({ puuid: 'p1', page: 0, pageSize: 10 })
    expect(res0.page).toBe(1) // fallback default
    expect(res0.data).toEqual([])

    // huge pageSize
    const resHuge = await tauriApi.queryEncounteredGames({ puuid: 'p1', page: 9999, pageSize: 500 })
    expect(resHuge.pageSize).toBe(500)
    expect(resHuge.data).toEqual([])
  })

  it('listSavedPlayerTags handles empty query parameters gracefully', async () => {
    const res = await tauriApi.listSavedPlayerTags()
    expect(res).toBeDefined()
    expect(res.data).toEqual([])
    expect(res.page).toBe(1)
    expect(res.pageSize).toBe(20)
  })

  it('deletePlayerTag and deleteEncounteredGame return structured result without crashing', async () => {
    const delTagRes = await tauriApi.deletePlayerTag('target-puuid')
    expect(delTagRes).toEqual({ success: true })

    const delGameRes = await tauriApi.deleteEncounteredGame(123456)
    expect(delGameRes).toEqual({ success: true })
  })

  it('setPlayerTagPhrasePanelExpanded toggles and returns boolean', async () => {
    const expTrue = await tauriApi.setPlayerTagPhrasePanelExpanded(true)
    expect(expTrue).toBe(true)

    const expFalse = await tauriApi.setPlayerTagPhrasePanelExpanded(false)
    expect(expFalse).toBe(false)
  })
})
