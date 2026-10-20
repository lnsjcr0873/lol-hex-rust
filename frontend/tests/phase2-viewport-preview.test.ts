import { describe, it, expect } from 'vitest'
import { toMatchPreviewState } from '../src/components/match-preview/types'
import { MATCH_CARD_COLLAPSED_HEIGHT_PX, MATCH_HISTORY_CARD_OVERSCAN_PX } from '../src/components/player-tab/widgets/match-history-card/constants'

describe('Phase 2: Virtual Viewport & Match Previewer Tests', () => {
  it('correctly exports viewport constants', () => {
    expect(MATCH_CARD_COLLAPSED_HEIGHT_PX).toBe(116)
    expect(MATCH_HISTORY_CARD_OVERSCAN_PX).toBe(640)
  })

  it('transforms number payload to MatchPreviewState', () => {
    const state = toMatchPreviewState(
      { summary: 12345678, puuid: 'test-puuid' },
      'lcu'
    )

    expect(state.gameId).toBe(12345678)
    expect(state.source).toBe('lcu')
    expect(state.puuid).toBe('test-puuid')
    expect(state.summary).toBeUndefined()
  })

  it('transforms LcuOrSgpGameSummary payload to MatchPreviewState', () => {
    const mockSummary = {
      gameId: 99887766,
      source: 'lcu' as const,
      data: {
        gameId: 99887766,
        gameCreation: 1700000000000,
        gameDuration: 1800,
        gameMode: 'CLASSIC',
        mapId: 11,
        queueId: 420,
        participants: [],
        participantIdentities: []
      } as any
    }

    const state = toMatchPreviewState(
      { summary: mockSummary, puuid: 'my-puuid' },
      'lcu'
    )

    expect(state.gameId).toBe(99887766)
    expect(state.source).toBe('lcu')
    expect(state.puuid).toBe('my-puuid')
    expect(state.summary?.gameId).toBe(99887766)
  })

  it('matches matching details if gameId matches', () => {
    const mockSummary = {
      gameId: 554433,
      source: 'lcu' as const,
      data: { gameId: 554433 } as any
    }
    const mockDetails = {
      gameId: 554433,
      source: 'lcu' as const,
      data: { frames: [] } as any
    }

    const state = toMatchPreviewState(
      { summary: mockSummary, details: mockDetails },
      'lcu'
    )

    expect(state.details).toBeDefined()
    expect(state.details?.gameId).toBe(554433)
  })
})
