import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref, nextTick } from 'vue'
import { setActivePinia, createPinia } from 'pinia'
import {
  getNameFromYuriyuri,
  streamerSummonerName,
  masked,
  useStreamerModeMaskedText
} from '../src/components/widgets/useStreamerModeMaskedText'
import { useFreezeValue } from '../src/components/player-tab/utils/freeze'
import {
  insertTextAtSelection,
  normalizeSavedPlayerTagPhrase,
  normalizeSavedPlayerTagPhrases
} from '../src/components/player-tag-edit/cursor'
import {
  PLAYER_TAB_WIDE_MIN_WIDTH,
  ENCOUNTERED_GAMES_PAGE_SIZE,
  SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS,
  SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH
} from '../src/components/player-tab/constants'
import { useOngoingMatchStore } from '../src/stores/ongoing-match'
import type { DraftOptions } from '../src/api/types'
import * as tauriApi from '../src/api/tauri'

describe('Milestone 2 Parity: Streamer Mode Determinism & Masking', () => {
  it('deterministic Horner PUUID hash produces identical pseudonym for the same seed', () => {
    const puuid1 = '01234567-89ab-cdef-0123-456789abcdef'
    const puuid2 = 'fedcba98-7654-3210-fedc-ba9876543210'

    const name1a = getNameFromYuriyuri(puuid1, 'zh-CN')
    const name1b = getNameFromYuriyuri(puuid1, 'zh-CN')
    expect(name1a).toBe(name1b)
    expect(typeof name1a).toBe('string')
    expect(name1a.length).toBeGreaterThan(0)

    const name2a = getNameFromYuriyuri(puuid2, 'zh-CN')
    const name2b = getNameFromYuriyuri(puuid2, 'zh-CN')
    expect(name2a).toBe(name2b)

    const nameEn = getNameFromYuriyuri(puuid1, 'en')
    expect(typeof nameEn).toBe('string')
    expect(nameEn.length).toBeGreaterThan(0)
  })

  it('streamerSummonerName wraps getNameFromYuriyuri safely for empty or special seeds', () => {
    const emptyName = streamerSummonerName('', 0)
    expect(typeof emptyName).toBe('string')
    expect(emptyName.length).toBeGreaterThan(0)

    const specialName = streamerSummonerName('★测试玩家#1234★', 3)
    expect(typeof specialName).toBe('string')
    expect(specialName.length).toBeGreaterThan(0)
  })

  it('masked replaces characters matching mask pattern', () => {
    expect(masked('1234', '****')).toBe('****')
    expect(masked('ABCD', '#####')).toBe('#####')
    expect(masked('', '***')).toBe('***')
  })

  it('useStreamerModeMaskedText switches between raw and masked text', () => {
    const { isStreamerMode, setStreamerMode, summonerName, masked: maskFn } = useStreamerModeMaskedText()

    setStreamerMode(false)
    expect(isStreamerMode.value).toBe(false)
    expect(summonerName('Faker#KR1', 0)).toBe('Faker#KR1')
    expect(maskFn('KR1', '#####')).toBe('KR1')

    setStreamerMode(true)
    expect(isStreamerMode.value).toBe(true)
    expect(summonerName('Faker#KR1', 0)).not.toBe('Faker#KR1')
    expect(maskFn('KR1', '#####')).toBe('#####')

    setStreamerMode(false)
  })
})

describe('Milestone 2 Parity: Freeze Sentinel Composable', () => {
  it('useFreezeValue tracks source updates when unfrozen', () => {
    const source = ref('alpha')
    const { isFrozen, value, freeze, unfreeze } = useFreezeValue(source)

    expect(isFrozen.value).toBe(false)
    expect(value.value).toBe('alpha')

    source.value = 'beta'
    expect(value.value).toBe('beta')
  })

  it('useFreezeValue locks value when frozen, ignoring source changes until unfreezing', () => {
    const source = ref(true)
    const { isFrozen, value, freeze, unfreeze } = useFreezeValue(source)

    expect(value.value).toBe(true)

    freeze()
    expect(isFrozen.value).toBe(true)

    // Source changes while frozen
    source.value = false
    expect(value.value).toBe(true) // Stays frozen at true!

    source.value = false
    expect(value.value).toBe(true)

    // Unfreezing syncs to current source value
    unfreeze()
    expect(isFrozen.value).toBe(false)
    expect(value.value).toBe(false)
  })

  it('useFreezeValue supports initialFrozen option', () => {
    const source = ref('initial')
    const { isFrozen, value, unfreeze } = useFreezeValue(source, { initialFrozen: true })

    expect(isFrozen.value).toBe(true)
    expect(value.value).toBe('initial')

    source.value = 'updated'
    expect(value.value).toBe('initial') // Still initial

    unfreeze()
    expect(value.value).toBe('updated')
  })
})

describe('Milestone 2 Parity: Player Tag Edit & Phrase Normalization', () => {
  it('insertTextAtSelection appends at end when inputEl is not provided', () => {
    const res = insertTextAtSelection(null, 'hello', ' world')
    expect(res.value).toBe('hello world')
    expect(res.cursorPosition).toBe(11)
  })

  it('insertTextAtSelection replaces selected range when selection boundaries exist', () => {
    const mockInput = {
      focus: vi.fn(),
      textareaElRef: {
        selectionStart: 6,
        selectionEnd: 11
      } as any
    }

    const res = insertTextAtSelection(mockInput, 'hello world today', 'there')
    expect(res.value).toBe('hello there today')
    expect(res.cursorPosition).toBe(11)
  })

  it('normalizeSavedPlayerTagPhrase enforces 100 character cap and trims whitespace', () => {
    expect(normalizeSavedPlayerTagPhrase('  绝活哥  ')).toBe('绝活哥')
    expect(normalizeSavedPlayerTagPhrase(null)).toBe('')
    expect(normalizeSavedPlayerTagPhrase(undefined)).toBe('')

    const longString = 'a'.repeat(150)
    const trimmed = normalizeSavedPlayerTagPhrase(longString, SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH)
    expect(trimmed.length).toBe(100)
    expect(trimmed).toBe('a'.repeat(100))
  })

  it('normalizeSavedPlayerTagPhrases deduplicates, strips empty items, and caps at 20 items', () => {
    const rawList = [
      ' 绝活哥 ',
      '通天代',
      '绝活哥', // duplicate
      '', // empty
      '   ', // whitespace
      ...Array.from({ length: 30 }, (_, i) => `短语${i}`)
    ]

    const result = normalizeSavedPlayerTagPhrases(
      rawList,
      SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS,
      SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH
    )

    expect(result.length).toBe(20) // Capped at 20
    expect(result[0]).toBe('绝活哥')
    expect(result[1]).toBe('通天代')
    expect(result.filter(p => p === '绝活哥').length).toBe(1) // No duplicates
    expect(result.every(p => p.length > 0 && p.length <= 100)).toBe(true)
  })
})

describe('Milestone 2 Parity: Responsive Breakpoint & Storage Constants', () => {
  it('constants match LeagueAkari specifications', () => {
    expect(PLAYER_TAB_WIDE_MIN_WIDTH).toBe(1064)
    expect(ENCOUNTERED_GAMES_PAGE_SIZE).toBe(10)
    expect(SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS).toBe(20)
    expect(SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH).toBe(100)
  })
})

describe('Milestone 2 Parity: Dry-Run Ongoing Game Simulation & Pinia Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  const sampleDraft: DraftOptions = {
    gameModeKind: 'normal',
    queueId: 420,
    puuid: 'p1',
    teams: {
      '100': ['p1', 'p2', 'p3', 'p4', 'p5'],
      '200': ['p6', 'p7', 'p8', 'p9', 'p10']
    },
    championSelections: {
      p1: 103,
      p2: 64,
      p3: 157,
      p4: 222,
      p5: 412,
      p6: 266,
      p7: 76,
      p8: 238,
      p9: 81,
      p10: 89
    },
    positions: {
      p1: { selected: 'TOP' },
      p2: { selected: 'JUNGLE' },
      p3: { selected: 'MIDDLE' },
      p4: { selected: 'BOTTOM' },
      p5: { selected: 'UTILITY' }
    }
  }

  const sampleParticipants: any[] = [
    { puuid: 'p1', championId: 103, gameName: 'P1', tagLine: 'KR1', spells: [4, 12] },
    { puuid: 'p2', championId: 64, gameName: 'P2', tagLine: 'KR1', spells: [4, 11] }
  ]

  it('ongoingMatchStore initializes with empty draft state', () => {
    const store = useOngoingMatchStore()
    expect(store.draft).toBeNull()
    expect(store.isDraft).toBe(false)
    expect(store.isDryRunActive).toBe(false)
    expect(store.customOngoingData).toBeNull()
  })

  it('ongoingMatchStore setDraft correctly builds 5v5 team data for GameInfo hydration', () => {
    const store = useOngoingMatchStore()
    store.setDraft(sampleDraft, sampleParticipants)

    expect(store.isDraft).toBe(true)
    expect(store.isDryRunActive).toBe(true)
    expect(store.draft).not.toBeNull()
    expect(store.customOngoingData).not.toBeNull()

    const custom = store.customOngoingData
    expect(custom?.queue_id).toBe(420)
    expect(custom?.blue_team.length).toBe(5)
    expect(custom?.red_team.length).toBe(5)

    // Verify champion and spell mappings
    const p1 = custom?.blue_team.find(p => p.puuid === 'p1')
    expect(p1).toBeDefined()
    expect(p1?.champion_id).toBe(103)
    expect(p1?.spell1_id).toBe(4)
    expect(p1?.spell2_id).toBe(12)
    expect(p1?.game_name).toBe('P1')
  })

  it('ongoingMatchStore handles Cherry (Arena) draft options correctly', () => {
    const store = useOngoingMatchStore()
    const cherryDraft: DraftOptions = {
      gameModeKind: 'cherry',
      queueId: 1700,
      puuid: 'p1',
      teams: {
        'TEAM-ALL': ['p1', 'p2', 'p3', 'p4']
      },
      championSelections: {
        p1: 103,
        p2: 64,
        p3: 157,
        p4: 222
      }
    }

    store.setDraft(cherryDraft)

    expect(store.isDraft).toBe(true)
    expect(store.draft?.gameModeKind).toBe('cherry')
    expect(store.customOngoingData?.queue_name).toContain('斗魂竞技场')
  })

  it('ongoingMatchStore clearDraft restores store to clean state', () => {
    const store = useOngoingMatchStore()
    store.setDraft(sampleDraft)
    expect(store.isDraft).toBe(true)

    store.clearDraft()
    expect(store.isDraft).toBe(false)
    expect(store.isDryRunActive).toBe(false)
    expect(store.draft).toBeNull()
    expect(store.customOngoingData).toBeNull()
  })
})

describe('Milestone 2 Parity: SQLite IPC Methods Contract Verification', () => {
  it('exposes all required SQLite IPC methods with proper fallback signatures', () => {
    expect(typeof tauriApi.listSavedPlayerTags).toBe('function')
    expect(typeof tauriApi.getPlayerTags).toBe('function')
    expect(typeof tauriApi.savePlayerTag).toBe('function')
    expect(typeof tauriApi.deletePlayerTag).toBe('function')
    expect(typeof tauriApi.getPlayerTagPhrases).toBe('function')
    expect(typeof tauriApi.updatePlayerTagPhrases).toBe('function')
    expect(typeof tauriApi.setPlayerTagPhrasePanelExpanded).toBe('function')
    expect(typeof tauriApi.queryEncounteredGames).toBe('function')
    expect(typeof tauriApi.deleteEncounteredGame).toBe('function')
  })

  it('getPlayerTags gracefully falls back to empty array when puuid is empty or invoke fails', async () => {
    const emptyResult = await tauriApi.getPlayerTags('')
    expect(emptyResult).toEqual([])
  })

  it('getPlayerTagPhrases returns default preset phrases in mock/fallback mode', async () => {
    const phrases = await tauriApi.getPlayerTagPhrases()
    expect(Array.isArray(phrases)).toBe(true)
    expect(phrases.length).toBeGreaterThanOrEqual(8)
    expect(phrases).toContain('绝活哥')
    expect(phrases).toContain('通天代')
  })

  it('queryEncounteredGames returns paginated structure with fallback defaults', async () => {
    const result = await tauriApi.queryEncounteredGames('dummy-puuid', 1, 10)
    expect(result).toBeDefined()
    expect(Array.isArray(result.data)).toBe(true)
    expect(result.page).toBe(1)
    expect(result.pageSize).toBe(10)
    expect(typeof result.total).toBe('number')
  })
})
