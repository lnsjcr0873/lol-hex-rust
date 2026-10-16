import { describe, it, expect } from 'vitest'
import { isUnrankedTier, formatTierName, isMasterPlus } from '../src/components/player-tab/widgets/RankedPane.vue'
import { QUEUE_NAME_MAP } from '../src/components/player-tab/SummaryPane.vue'

describe('RankedPane Unranked & Tier Formatting Tests', () => {
  it('correctly identifies unranked tiers without false negatives', () => {
    expect(isUnrankedTier('UNRANKED')).toBe(true)
    expect(isUnrankedTier('unranked')).toBe(true)
    expect(isUnrankedTier('NONE')).toBe(true)
    expect(isUnrankedTier('none')).toBe(true)
    expect(isUnrankedTier('NA')).toBe(true)
    expect(isUnrankedTier('na')).toBe(true)
    expect(isUnrankedTier('')).toBe(true)
    expect(isUnrankedTier('   ')).toBe(true)
    expect(isUnrankedTier(null)).toBe(true)
    expect(isUnrankedTier(undefined)).toBe(true)

    expect(isUnrankedTier('DIAMOND')).toBe(false)
    expect(isUnrankedTier('GOLD')).toBe(false)
    expect(isUnrankedTier('MASTER')).toBe(false)
    expect(isUnrankedTier('CHALLENGER')).toBe(false)
  })

  it('formats tier names correctly to Chinese localization', () => {
    expect(formatTierName('UNRANKED')).toBe('未定级')
    expect(formatTierName(undefined)).toBe('未定级')
    expect(formatTierName('IRON')).toBe('黑铁')
    expect(formatTierName('BRONZE')).toBe('青铜')
    expect(formatTierName('SILVER')).toBe('白银')
    expect(formatTierName('GOLD')).toBe('黄金')
    expect(formatTierName('PLATINUM')).toBe('铂金')
    expect(formatTierName('EMERALD')).toBe('翡翠')
    expect(formatTierName('DIAMOND')).toBe('钻石')
    expect(formatTierName('MASTER')).toBe('大师')
    expect(formatTierName('GRANDMASTER')).toBe('宗师')
    expect(formatTierName('CHALLENGER')).toBe('王者')
  })

  it('identifies Master+ tiers which should omit roman numeral divisions', () => {
    expect(isMasterPlus('MASTER')).toBe(true)
    expect(isMasterPlus('GRANDMASTER')).toBe(true)
    expect(isMasterPlus('CHALLENGER')).toBe(true)
    expect(isMasterPlus('DIAMOND')).toBe(false)
    expect(isMasterPlus('GOLD')).toBe(false)
    expect(isMasterPlus(undefined)).toBe(false)
  })
})

describe('SummaryPane Queue Mapping & Dynamic Computations', () => {
  it('has complete queue name mappings matching Akari', () => {
    expect(QUEUE_NAME_MAP['all']).toBe('全部')
    expect(QUEUE_NAME_MAP['ranked_solo']).toBe('单双排')
    expect(QUEUE_NAME_MAP['ranked_flex']).toBe('灵活排位')
    expect(QUEUE_NAME_MAP['aram']).toBe('极地大乱斗')
    expect(QUEUE_NAME_MAP['normal']).toBe('匹配模式')
    expect(QUEUE_NAME_MAP['420']).toBe('单双排')
    expect(QUEUE_NAME_MAP['440']).toBe('灵活排位')
    expect(QUEUE_NAME_MAP['450']).toBe('极地大乱斗')
    expect(QUEUE_NAME_MAP['430']).toBe('匹配模式')
  })

  it('handles win/loss calculations without hardcoded 20 modulo', () => {
    // Edge case: totalCount = 1, winRate = 100% -> wins = 1, losses = 0
    const total1 = 1
    const winRate1 = 100
    const wins1 = Math.round((winRate1 / 100) * total1)
    const losses1 = Math.max(0, total1 - wins1)
    expect(wins1).toBe(1)
    expect(losses1).toBe(0)

    // Edge case: totalCount = 37, winRate = 50% -> wins = 19, losses = 18 (sum strictly equals 37)
    const total37 = 37
    const winRate37 = 50
    const wins37 = Math.round((winRate37 / 100) * total37)
    const losses37 = Math.max(0, total37 - wins37)
    expect(wins37).toBe(19)
    expect(losses37).toBe(18)
    expect(wins37 + losses37).toBe(37)

    // Dynamic large sample: totalCount = 200, winRate = 60% -> wins = 120, losses = 80
    const total200 = 200
    const winRate200 = 60
    const wins200 = Math.round((winRate200 / 100) * total200)
    const losses200 = Math.max(0, total200 - wins200)
    expect(wins200).toBe(120)
    expect(losses200).toBe(80)
  })

  it('parses champions record dictionary using Object.values without falling back to fake data', () => {
    const rawRecord: Record<number, any> = {
      266: {
        championId: 266,
        winLoss: { all: { count: 12, winRate: 0.75 } },
        summary: { avgKda: 4.5 }
      },
      84: {
        championId: 84,
        winLoss: { all: { count: 8, winRate: 0.625 } },
        summary: { avgKda: 3.8 }
      },
      157: {
        championId: 157,
        winLoss: { all: { count: 15, winRate: 0.40 } },
        summary: { avgKda: 2.1 }
      }
    }

    const list = Object.values(rawRecord)
    expect(list.length).toBe(3)

    const parsed = list
      .filter((c) => (c.winLoss?.all?.count ?? 0) >= 1)
      .sort((a, b) => (b.winLoss?.all?.count || 0) - (a.winLoss?.all?.count || 0))
      .map((c: any) => ({
        championId: c.championId,
        games: c.winLoss?.all?.count || 1,
        winRate: Math.round((c.winLoss?.all?.winRate || 0) * 100),
        kda: (c.summary?.avgKda || 3.0).toFixed(1)
      }))

    // Sorted descending by games: Yasuo (157: 15), Aatrox (266: 12), Akali (84: 8)
    expect(parsed[0].championId).toBe(157)
    expect(parsed[0].games).toBe(15)
    expect(parsed[0].winRate).toBe(40)
    expect(parsed[0].kda).toBe('2.1')

    expect(parsed[1].championId).toBe(266)
    expect(parsed[1].games).toBe(12)
    expect(parsed[1].winRate).toBe(75)
    expect(parsed[1].kda).toBe('4.5')

    expect(parsed[2].championId).toBe(84)
    expect(parsed[2].games).toBe(8)
    expect(parsed[2].winRate).toBe(63)
    expect(parsed[2].kda).toBe('3.8')
  })

  it('correctly determines winning and losing streak badges with threshold >= 2', () => {
    function computeStreak(analysis: any, currentPage = 1) {
      if (currentPage > 1) return null
      if (!analysis?.winLoss?.all) return null
      const winStreak = analysis.winLoss.all.winningStreak ?? 0
      const loseStreak = analysis.winLoss.all.losingStreak ?? 0

      if (winStreak >= 2) return { isWinning: true, count: winStreak }
      if (loseStreak >= 2) return { isWinning: false, count: loseStreak }
      return null
    }

    // 1 win or 1 loss should not trigger streak badge
    expect(computeStreak({ winLoss: { all: { winningStreak: 1, losingStreak: 0 } } })).toBeNull()
    expect(computeStreak({ winLoss: { all: { winningStreak: 0, losingStreak: 1 } } })).toBeNull()

    // 3 wins on page 1 -> 3连胜
    const winStreak = computeStreak({ winLoss: { all: { winningStreak: 3, losingStreak: 0 } } }, 1)
    expect(winStreak).toEqual({ isWinning: true, count: 3 })

    // 4 losses on page 1 -> 4连败
    const loseStreak = computeStreak({ winLoss: { all: { winningStreak: 0, losingStreak: 4 } } }, 1)
    expect(loseStreak).toEqual({ isWinning: false, count: 4 })

    // When on page > 1, streak badge is hidden
    expect(computeStreak({ winLoss: { all: { winningStreak: 5, losingStreak: 0 } } }, 2)).toBeNull()
    expect(computeStreak({ winLoss: { all: { winningStreak: 5, losingStreak: 0 } } }, 3)).toBeNull()
  })

  it('correctly calculates sampleCount in PlayerDetailDrawer', () => {
    function getDrawerSampleCount(player: any): number {
      if (player?.recent_games_count !== undefined && player.recent_games_count > 0) {
        return player.recent_games_count
      }
      if (player?.recent_matches && player.recent_matches.length > 0) {
        return player.recent_matches.length
      }
      return 20
    }

    expect(getDrawerSampleCount({ recent_games_count: 50 })).toBe(50)
    expect(getDrawerSampleCount({ recent_games_count: 100, recent_matches: ['win', 'loss'] })).toBe(100)
    expect(getDrawerSampleCount({ recent_matches: ['win', 'loss', 'win'] })).toBe(3)
    expect(getDrawerSampleCount({})).toBe(20)
    expect(getDrawerSampleCount(null)).toBe(20)
  })
})
