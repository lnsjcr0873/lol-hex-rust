import type {
  AggregatedAnalysis,
  AggregatedChampionAnalysis,
  AggregatedJungleAnalysis,
  AkariScore
} from '../../../shared/data-adapter/analysis/player'
import type { EnhancedPlayerCard } from '../types'

export function adaptPlayerCardToAggregatedAnalysis(player: EnhancedPlayerCard): AggregatedAnalysis {
  if (player.analysis) {
    return player.analysis
  }

  const count = player.recent_games_count || 20
  const winRate = (player.recent_win_rate || 50) / 100
  const wins = Math.round(count * winRate)
  const losses = count - wins

  // Count streaks from recent_matches
  let winningStreak = 0
  let losingStreak = 0
  if (player.recent_matches && player.recent_matches.length > 0) {
    let currentStreak = 0
    let streakType: 'win' | 'loss' | null = null
    for (let i = player.recent_matches.length - 1; i >= 0; i--) {
      const match = player.recent_matches[i]
      if (streakType === null) {
        streakType = match
        currentStreak = 1
      } else if (match === streakType) {
        currentStreak++
      } else {
        break
      }
    }
    if (streakType === 'win') winningStreak = currentStreak
    if (streakType === 'loss') losingStreak = currentStreak
  }

  // Spells flash check (D is spell1, F is spell2)
  const isFlashD = player.spell1_id === 4
  const isFlashF = player.spell2_id === 4
  const hasSuspiciousFlashBadge = (player.badges || []).some(
    (b) => b.id.includes('flash') || b.label.includes('闪现')
  )

  let flashOnD = isFlashD ? count : 0
  let flashOnF = isFlashF ? count : 0
  if (hasSuspiciousFlashBadge) {
    flashOnD = Math.max(1, Math.floor(count * 0.35))
    flashOnF = Math.max(1, count - flashOnD)
  }

  // Check easy gank badge
  const hasEasyGankBadge = (player.badges || []).some(
    (b) => b.id.includes('gank') || b.label.includes('容易被抓') || b.label.includes('提款机')
  )
  const hasHardGankBadge = (player.badges || []).some(
    (b) => b.label.includes('难被抓') || b.label.includes('意识极佳')
  )
  let earlyDeaths = 1.1
  if (hasEasyGankBadge) earlyDeaths = 2.4
  if (hasHardGankBadge) earlyDeaths = 0.4

  const kdaRatio = player.recent_kda_ratio || 3.0
  const isExtraordinary = kdaRatio >= 4.8 && winRate >= 0.65
  const isOutstanding = kdaRatio >= 3.8 && winRate >= 0.58

  const akariTotal = Math.min(17.0, Math.max(4.0, (winRate * 4.5) + (kdaRatio * 1.4) + 1.8))
  const akariScore: AkariScore = {
    total: akariTotal,
    maxScore: 17.0,
    kdaScore: Math.min(1.0, kdaRatio / 5.0),
    winRateScore: Math.min(1.0, winRate),
    dmgScore: Math.min(3.0, (player.radar?.output ?? 75) / 33),
    dmgTakenScore: Math.min(2.0, (player.radar?.survival ?? 70) / 45),
    healingScore: 1.2,
    csScore: 1.6,
    goldScore: Math.min(2.0, (player.radar?.gold ?? 75) / 45),
    participationScore: Math.min(2.0, (player.radar?.join_rate ?? 70) / 45),
    visionScore: Math.min(2.0, (player.radar?.vision ?? 60) / 45),
    outstanding: isOutstanding ? { total: akariTotal } as any : null,
    extraordinary: isExtraordinary ? { total: akariTotal } as any : null
  }

  const champAnalysis: AggregatedChampionAnalysis = {
    championId: player.champion_id,
    summary: {
      avgKda: player.champ_recent_win_rate ? (kdaRatio * 1.1) : kdaRatio,
      winRate: (player.champ_recent_win_rate || 55) / 100,
      avgChampionDamagePercentageOfTeam: 0.28,
      avgDamageTakenPercentageOfTeam: 0.22,
      avgCsPercentageOfTeam: 0.28,
      avgCsPerMinute: 8.2,
      avgGoldPercentageOfTeam: 0.26,
      avgKillParticipation: 0.68,
      avgChampionDamagePerMinute: 720,
      kills: 8,
      deaths: 3,
      assists: 7,
      avgChampionDamageRatioToTeamMax: 0.9,
      avgChampionDamageRatioToMax: 0.85,
      avgDamageTakenRatioToTeamMax: 0.85,
      avgDamageTakenRatioToMax: 0.8,
      avgGoldRatioToTeamMax: 0.9,
      avgGoldRatioToMax: 0.85,
      avgCsRatioToTeamMax: 0.9,
      avgCsRatioToMax: 0.85,
      avgTowerDamageRatioToTeamMax: 0.8,
      avgTowerDamageRatioToMax: 0.75,
      avgTowerDamagePercentageOfTeam: 0.25,
      avgVisionScore: 45,
      avgVisionScorePercentageOfTeam: 0.22,
      avgDamageGoldEfficiency: 1.28,
      avgKillDamageEfficiency: 1.2,
      kdaCv: 0.2,
      avgSoloKills: 1.2,
      avgEnemyMissingPings: 1.0,
      avgPings: 10
    },
    winLoss: {
      all: {
        count: player.champ_recent_games || 8,
        wins: Math.round((player.champ_recent_games || 8) * ((player.champ_recent_win_rate || 65) / 100)),
        losses: (player.champ_recent_games || 8) - Math.round((player.champ_recent_games || 8) * ((player.champ_recent_win_rate || 65) / 100)),
        winRate: (player.champ_recent_win_rate || 65) / 100,
        winningStreak: 2,
        losingStreak: 0,
        activeSessionWins: 2,
        activeSessionLosses: 0
      },
      normal: {
        count: player.champ_recent_games || 8,
        wins: Math.round((player.champ_recent_games || 8) * ((player.champ_recent_win_rate || 65) / 100)),
        losses: (player.champ_recent_games || 8) - Math.round((player.champ_recent_games || 8) * ((player.champ_recent_win_rate || 65) / 100)),
        winRate: (player.champ_recent_win_rate || 65) / 100,
        winningStreak: 2,
        losingStreak: 0,
        activeSessionWins: 2,
        activeSessionLosses: 0
      },
      cherry: {
        count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
        activeSessionWins: 0, activeSessionLosses: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
        topHalfRate: 0, avgSubteamPlacement: 0
      }
    },
    positions: {
      TOP: player.assigned_position === 'TOP' ? (player.champ_recent_games || 8) : 0,
      JUNGLE: player.assigned_position === 'JUNGLE' ? (player.champ_recent_games || 8) : 0,
      MIDDLE: player.assigned_position === 'MIDDLE' || player.assigned_position === 'MID' ? (player.champ_recent_games || 8) : 0,
      BOTTOM: player.assigned_position === 'BOTTOM' || player.assigned_position === 'BOT' ? (player.champ_recent_games || 8) : 0,
      UTILITY: player.assigned_position === 'UTILITY' || player.assigned_position === 'SUP' ? (player.champ_recent_games || 8) : 0
    },
    akariScore: {
      total: akariTotal + 0.5,
      maxScore: 17.0,
      kdaScore: 0.9,
      winRateScore: 0.85,
      dmgScore: 2.4,
      dmgTakenScore: 1.6,
      healingScore: 1.0,
      csScore: 1.8,
      goldScore: 1.7,
      participationScore: 1.5,
      visionScore: 1.4,
      outstanding: isOutstanding ? { total: akariTotal } as any : null,
      extraordinary: isExtraordinary ? { total: akariTotal } as any : null
    },
    jungle: null
  }

  return {
    count,
    detailsCount: count,
    summary: {
      avgChampionDamageRatioToTeamMax: 0.9,
      avgChampionDamageRatioToMax: 0.85,
      avgChampionDamagePercentageOfTeam: ((player.radar?.output ?? 75) / 100) * 0.35,
      avgChampionDamagePerMinute: 680,
      avgDamageTakenRatioToTeamMax: 0.85,
      avgDamageTakenRatioToMax: 0.8,
      avgDamageTakenPercentageOfTeam: 0.24,
      avgGoldRatioToTeamMax: 0.9,
      avgGoldRatioToMax: 0.85,
      avgGoldPercentageOfTeam: ((player.radar?.gold ?? 75) / 100) * 0.28,
      avgCsRatioToTeamMax: 0.9,
      avgCsRatioToMax: 0.85,
      avgCsPercentageOfTeam: 0.26,
      avgCsPerMinute: 7.8,
      avgTowerDamageRatioToTeamMax: 0.8,
      avgTowerDamageRatioToMax: 0.75,
      avgTowerDamagePercentageOfTeam: 0.25,
      avgVisionScore: player.radar?.vision ?? 45,
      avgVisionScorePercentageOfTeam: 0.22,
      avgDamageGoldEfficiency: 1.28,
      avgKillParticipation: ((player.radar?.join_rate ?? 70) / 100) * 0.8,
      avgKillDamageEfficiency: kdaRatio > 4.2 ? 1.42 : (kdaRatio < 2.0 ? 0.6 : 1.05),
      kills: Math.round(kdaRatio * 2.5),
      deaths: Math.max(1, Math.round(count * 0.15)),
      assists: Math.round(kdaRatio * 2.8),
      avgKda: kdaRatio,
      kdaCv: 0.25,
      winRate,
      avgSoloKills: kdaRatio > 4.0 ? 1.4 : 0.6,
      avgEnemyMissingPings: winRate < 0.45 ? 2.8 : 0.8,
      avgPings: 12
    },
    winLoss: {
      all: {
        count,
        activeSessionWins: wins,
        activeSessionLosses: losses,
        wins,
        losses,
        winRate,
        winningStreak,
        losingStreak
      },
      normal: {
        count,
        activeSessionWins: wins,
        activeSessionLosses: losses,
        wins,
        losses,
        winRate,
        winningStreak,
        losingStreak
      },
      cherry: {
        count: 0,
        activeSessionWins: 0,
        activeSessionLosses: 0,
        wins: 0,
        losses: 0,
        winRate: 0,
        winningStreak: 0,
        losingStreak: 0,
        top1s: 0,
        topHalfFinishes: 0,
        top1Rate: 0,
        topHalfRate: 0,
        avgSubteamPlacement: 0
      }
    },
    spells: {
      flashOnD,
      flashOnF
    },
    teamSide: {
      redSideCount: Math.floor(count / 2),
      blueSideCount: Math.ceil(count / 2)
    },
    positions: {
      TOP: player.assigned_position === 'TOP' ? count : 0,
      JUNGLE: player.assigned_position === 'JUNGLE' ? count : 0,
      MIDDLE: player.assigned_position === 'MIDDLE' || player.assigned_position === 'MID' ? count : 0,
      BOTTOM: player.assigned_position === 'BOTTOM' || player.assigned_position === 'BOT' ? count : 0,
      UTILITY: player.assigned_position === 'UTILITY' || player.assigned_position === 'SUP' ? count : 0
    },
    champions: {
      [player.champion_id]: champAnalysis
    },
    map: {},
    jungle: (player.assigned_position?.toUpperCase() === 'JUNGLE' || player.assigned_position === '2')
      ? createSynthesizedJungleAnalysis(player)
      : null,
    akariScore,
    details: {
      avgEarlyDeathsWithEnemyJunglerInvolved: earlyDeaths
    }
  }
}

export function createSynthesizedJungleAnalysis(player: EnhancedPlayerCard): AggregatedJungleAnalysis {
  if (player.jungleAnalysis) {
    return player.jungleAnalysis
  }

  const isBlue = player.team === 'blue'
  const count = player.champ_recent_games || 12

  const level3Kills = [
    { x: 2800, y: 11200, lane: 'top' as const },
    { x: 12200, y: 3200, lane: 'bot' as const }
  ]
  const level4Kills = [
    { x: 7400, y: 7200, lane: 'mid' as const },
    { x: 13000, y: 2800, lane: 'bot' as const }
  ]

  return {
    gamesAnalyzed: count,
    avgTopZonePercentage: 0.28,
    avgMidZonePercentage: 0.32,
    avgBotZonePercentage: 0.40,
    topZoneWeightSum: 24,
    midZoneWeightSum: 30,
    botZoneWeightSum: 42,
    totalZoneWeightSum: 96,
    totalTopGanks: 5,
    totalMidGanks: 6,
    totalBotGanks: 8,
    avgTopGanks: 0.4,
    avgMidGanks: 0.5,
    avgBotGanks: 0.7,
    firstClearCamp: {
      blueGames: isBlue ? count : 0,
      redGames: isBlue ? 0 : count,
      blue: { red: 5, blue: 6, wolves: 1, raptors: 0 },
      red: { red: 6, blue: 5, wolves: 0, raptors: 1 },
      blueInvade: { red: 1, blue: 0, wolves: 0, raptors: 0 },
      redInvade: { red: 0, blue: 1, wolves: 0, raptors: 0 }
    },
    earlyGank: {
      level3GankRate: 0.45,
      level3GankCount: 5,
      level3KillPositions: level3Kills,
      level4GankRate: 0.55,
      level4GankCount: 6,
      level4KillPositions: level4Kills,
      byTeam: {
        blueGames: isBlue ? count : 0,
        redGames: isBlue ? 0 : count,
        blueLevel3GankRate: 0.45,
        blueLevel3GankCount: 4,
        blueLevel3KillPositions: level3Kills,
        blueLevel4GankRate: 0.55,
        blueLevel4GankCount: 7,
        blueLevel4KillPositions: level4Kills,
        redLevel3GankRate: 0.42,
        redLevel3GankCount: 5,
        redLevel3KillPositions: level3Kills,
        redLevel4GankRate: 0.52,
        redLevel4GankCount: 6,
        redLevel4KillPositions: level4Kills
      }
    },
    minutePositions: [
      { x: 3830, y: 7880, lane: 'top', minute: 1 },
      { x: 5200, y: 6400, lane: 'top', minute: 3 },
      { x: 7000, y: 7000, lane: 'mid', minute: 6 },
      { x: 8200, y: 5400, lane: 'bot', minute: 8 },
      { x: 11200, y: 4000, lane: 'bot', minute: 11 },
      { x: 12800, y: 3100, lane: 'bot', minute: 14 }
    ],
    gankPositions: [
      { x: 12600, y: 3000, lane: 'bot' },
      { x: 7500, y: 7300, lane: 'mid' },
      { x: 2600, y: 11800, lane: 'top' }
    ],
    objectives: {
      firstDragonRate: 0.65,
      soloDragonRate: 0.42,
      avgDragons: 2.4,
      avgFirstDragonTime: 420,
      avgVoidgrubs: 3.2,
      avgFirstVoidgrubTime: 380,
      avgHeralds: 0.8,
      avgFirstHeraldTime: 860,
      avgBarons: 1.1,
      avgFirstBaronTime: 1380
    }
  }
}
