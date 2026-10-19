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

  // 1. If matches array exists, aggregate from genuine matches
  if (player.matches && player.matches.length > 0) {
    const matches = player.matches
    const count = matches.length
    const wins = matches.filter((m) => m.win).length
    const losses = count - wins
    const winRate = count > 0 ? wins / count : 0

    let winningStreak = 0
    let losingStreak = 0
    for (const m of matches) {
      if (m.win) {
        if (losingStreak > 0) break
        winningStreak++
      } else {
        if (winningStreak > 0) break
        losingStreak++
      }
    }

    const totalKills = matches.reduce((acc, m) => acc + (m.kills || 0), 0)
    const totalDeaths = matches.reduce((acc, m) => acc + (m.deaths || 0), 0)
    const totalAssists = matches.reduce((acc, m) => acc + (m.assists || 0), 0)
    const avgKda = totalDeaths === 0 ? (totalKills + totalAssists) : (totalKills + totalAssists) / totalDeaths
    const avgCsPerMinute = count > 0 ? matches.reduce((acc, m) => acc + (m.cs_per_min || 0), 0) / count : 0

    // Spells
    const isFlashD = player.spell1_id === 4
    const isFlashF = player.spell2_id === 4
    const flashOnD = isFlashD ? count : 0
    const flashOnF = isFlashF ? count : 0

    // Champions
    const champions: Record<string, AggregatedChampionAnalysis> = {}
    const champGroups = new Map<number, any[]>()
    for (const m of matches) {
      if (m.champion_id > 0) {
        const list = champGroups.get(m.champion_id) || []
        list.push(m)
        champGroups.set(m.champion_id, list)
      }
    }

    for (const [cid, cMatches] of champGroups.entries()) {
      const cCount = cMatches.length
      const cWins = cMatches.filter((m) => m.win).length
      const cLosses = cCount - cWins
      const cWinRate = cCount > 0 ? cWins / cCount : 0
      const cKills = cMatches.reduce((acc, m) => acc + (m.kills || 0), 0) / cCount
      const cDeaths = cMatches.reduce((acc, m) => acc + (m.deaths || 0), 0) / cCount
      const cAssists = cMatches.reduce((acc, m) => acc + (m.assists || 0), 0) / cCount
      const cAvgKda = cDeaths === 0 ? (cKills + cAssists) : (cKills + cAssists) / cDeaths
      const cCsPerMin = cMatches.reduce((acc, m) => acc + (m.cs_per_min || 0), 0) / cCount

      let cWinStreak = 0
      let cLoseStreak = 0
      for (const m of cMatches) {
        if (m.win) {
          if (cLoseStreak > 0) break
          cWinStreak++
        } else {
          if (cWinStreak > 0) break
          cLoseStreak++
        }
      }

      const cDamageShare = cCount > 0 ? cMatches.reduce((acc, m) => acc + (m.damage_share_pct || 0), 0) / cCount : 0
      const cDamage = cCount > 0 ? cMatches.reduce((acc, m) => acc + (m.damage || 0), 0) / cCount : 0
      const cGold = cCount > 0 ? cMatches.reduce((acc, m) => acc + (m.gold || 0), 0) / cCount : 0
      const cVision = cCount > 0 ? cMatches.reduce((acc, m) => acc + (m.vision_score || 0), 0) / cCount : 0
      const cKp = cCount > 0 ? cMatches.reduce((acc, m) => acc + (m.kp_pct || 0), 0) / cCount : 0
      const cDamagePerMin = cCount > 0 ? cMatches.reduce((acc, m) => {
        const dur = (m.game_duration || 1800) / 60
        return acc + (dur > 0 ? (m.damage || 0) / dur : 0)
      }, 0) / cCount : 0

      champions[String(cid)] = {
        championId: cid,
        summary: {
          avgKda: cAvgKda,
          winRate: cWinRate,
          avgChampionDamagePercentageOfTeam: cDamageShare,
          avgDamageTakenPercentageOfTeam: 0,
          avgCsPercentageOfTeam: 0,
          avgCsPerMinute: cCsPerMin,
          avgGoldPercentageOfTeam: 0,
          avgKillParticipation: cKp,
          avgChampionDamagePerMinute: Math.round(cDamagePerMin),
          kills: Math.round(cKills),
          deaths: Math.round(cDeaths),
          assists: Math.round(cAssists),
          avgChampionDamageRatioToTeamMax: 0,
          avgChampionDamageRatioToMax: 0,
          avgDamageTakenRatioToTeamMax: 0,
          avgDamageTakenRatioToMax: 0,
          avgGoldRatioToTeamMax: 0,
          avgGoldRatioToMax: 0,
          avgCsRatioToTeamMax: 0,
          avgCsRatioToMax: 0,
          avgTowerDamageRatioToTeamMax: 0,
          avgTowerDamageRatioToMax: 0,
          avgTowerDamagePercentageOfTeam: 0,
          avgVisionScore: cVision > 0 ? Math.round(cVision) : 0,
          avgVisionScorePercentageOfTeam: 0,
          avgDamageGoldEfficiency: cGold > 0 ? Math.round((cDamage / cGold) * 100) / 100 : 0,
          avgKillDamageEfficiency: 1.0,
          kdaCv: 0,
          avgSoloKills: null,
          avgEnemyMissingPings: null,
          avgPings: null
        },
        winLoss: {
          all: {
            count: cCount,
            wins: cWins,
            losses: cLosses,
            winRate: cWinRate,
            winningStreak: cWinStreak,
            losingStreak: cLoseStreak,
            activeSessionWins: cWins,
            activeSessionLosses: cLosses
          },
          normal: {
            count: cCount,
            wins: cWins,
            losses: cLosses,
            winRate: cWinRate,
            winningStreak: cWinStreak,
            losingStreak: cLoseStreak,
            activeSessionWins: cWins,
            activeSessionLosses: cLosses
          },
          cherry: {
            count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
            activeSessionWins: 0, activeSessionLosses: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
            topHalfRate: 0, avgSubteamPlacement: 0
          }
        },
        positions: {
          TOP: 0, JUNGLE: 0, MIDDLE: 0, BOTTOM: 0, UTILITY: 0
        },
        akariScore: {
          total: 0,
          maxScore: 17.0,
          kdaScore: Math.min(1.0, cAvgKda / 5.0),
          winRateScore: Math.min(1.0, cWinRate),
          dmgScore: 0,
          dmgTakenScore: 0,
          healingScore: 0,
          csScore: 0,
          goldScore: 0,
          participationScore: 0,
          visionScore: 0,
          outstanding: null,
          extraordinary: null
        },
        jungle: null
      }
    }

    const avgDamageShare = count > 0 ? matches.reduce((acc, m) => acc + (m.damage_share_pct || 0), 0) / count : 0
    const avgDamage = count > 0 ? matches.reduce((acc, m) => acc + (m.damage || 0), 0) / count : 0
    const avgGold = count > 0 ? matches.reduce((acc, m) => acc + (m.gold || 0), 0) / count : 0
    const avgVision = count > 0 ? matches.reduce((acc, m) => acc + (m.vision_score || 0), 0) / count : 0
    const avgKp = count > 0 ? matches.reduce((acc, m) => acc + (m.kp_pct || 0), 0) / count : 0
    const avgDamagePerMinute = count > 0 ? matches.reduce((acc, m) => {
      const dur = (m.game_duration || 1800) / 60
      return acc + (dur > 0 ? (m.damage || 0) / dur : 0)
    }, 0) / count : 0

    const kdaScore = Math.min(1.0, avgKda / 5.0)
    const winRateScore = Math.min(1.0, winRate)
    const dmgScore = avgDamageShare > 0 ? Math.min(3.0, (avgDamageShare / 30.0) * 3.0) : Math.min(3.0, (player.radar?.output ?? player.radar?.damage ?? 0) / 33)
    const dmgTakenScore = Math.min(2.0, (player.radar?.survival ?? 0) / 45)
    const csScore = avgCsPerMinute > 0 ? Math.min(2.0, (avgCsPerMinute / 10.0) * 2.0) : Math.min(2.0, (player.radar?.gold ?? 0) / 45)
    const goldScore = avgGold > 0 ? Math.min(2.0, (avgGold / 16000) * 2.0) : Math.min(2.0, (player.radar?.gold ?? 0) / 45)
    const participationScore = avgKp > 0 ? Math.min(2.0, (avgKp / 100.0) * 2.5) : Math.min(2.0, (player.radar?.join_rate ?? 0) / 45)
    const visionScore = avgVision > 0 ? Math.min(2.0, avgVision / 35.0) : Math.min(2.0, (player.radar?.vision ?? 0) / 45)
    const akariTotal = Math.min(17.0, kdaScore + winRateScore + dmgScore + dmgTakenScore + csScore + goldScore + participationScore + visionScore)
    const isExtraordinary = akariTotal >= 13.5 && count >= 5
    const isOutstanding = akariTotal >= 11.5 && count >= 5

    const akariScore: AkariScore = {
      total: akariTotal,
      maxScore: 17.0,
      kdaScore,
      winRateScore,
      dmgScore,
      dmgTakenScore,
      healingScore: 0,
      csScore,
      goldScore,
      participationScore,
      visionScore,
      outstanding: isOutstanding ? ({ total: akariTotal } as any) : null,
      extraordinary: isExtraordinary ? ({ total: akariTotal } as any) : null
    }

    return {
      count,
      detailsCount: count,
      summary: {
        avgChampionDamageRatioToTeamMax: 0,
        avgChampionDamageRatioToMax: 0,
        avgChampionDamagePercentageOfTeam: avgDamageShare,
        avgChampionDamagePerMinute: Math.round(avgDamagePerMinute),
        avgDamageTakenRatioToTeamMax: 0,
        avgDamageTakenRatioToMax: 0,
        avgDamageTakenPercentageOfTeam: 0,
        avgGoldRatioToTeamMax: 0,
        avgGoldRatioToMax: 0,
        avgGoldPercentageOfTeam: 0,
        avgCsRatioToTeamMax: 0,
        avgCsRatioToMax: 0,
        avgCsPercentageOfTeam: 0,
        avgCsPerMinute,
        avgTowerDamageRatioToTeamMax: 0,
        avgTowerDamageRatioToMax: 0,
        avgTowerDamagePercentageOfTeam: 0,
        avgVisionScore: avgVision > 0 ? Math.round(avgVision) : (player.radar?.vision ?? 0),
        avgVisionScorePercentageOfTeam: 0,
        avgDamageGoldEfficiency: avgGold > 0 ? Math.round((avgDamage / avgGold) * 100) / 100 : 0,
        avgKillParticipation: avgKp,
        avgKillDamageEfficiency: 1.0,
        kills: Math.round(totalKills / count),
        deaths: Math.round(totalDeaths / count),
        assists: Math.round(totalAssists / count),
        avgKda,
        kdaCv: 0,
        winRate,
        avgSoloKills: null,
        avgEnemyMissingPings: null,
        avgPings: null
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
          count: 0, activeSessionWins: 0, activeSessionLosses: 0, wins: 0, losses: 0, winRate: 0,
          winningStreak: 0, losingStreak: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
          topHalfRate: 0, avgSubteamPlacement: 0
        }
      },
      spells: {
        flashOnD,
        flashOnF
      },
      teamSide: {
        redSideCount: 0,
        blueSideCount: 0
      },
      positions: {
        TOP: player.assigned_position === 'TOP' ? count : 0,
        JUNGLE: player.assigned_position === 'JUNGLE' ? count : 0,
        MIDDLE: player.assigned_position === 'MIDDLE' || player.assigned_position === 'MID' ? count : 0,
        BOTTOM: player.assigned_position === 'BOTTOM' || player.assigned_position === 'BOT' ? count : 0,
        UTILITY: player.assigned_position === 'UTILITY' || player.assigned_position === 'SUP' ? count : 0
      },
      champions,
      map: {},
      jungle: player.jungleAnalysis ?? null,
      akariScore,
      details: {
        avgEarlyDeathsWithEnemyJunglerInvolved: null
      }
    }
  }

  // 2. If no matches array, check recent_games_count
  const count = player.recent_games_count ?? 0
  if (count === 0) {
    const emptyAkariScore: AkariScore = {
      total: 0,
      maxScore: 17.0,
      kdaScore: 0,
      winRateScore: 0,
      dmgScore: 0,
      dmgTakenScore: 0,
      healingScore: 0,
      csScore: 0,
      goldScore: 0,
      participationScore: 0,
      visionScore: 0,
      outstanding: null,
      extraordinary: null
    }

    return {
      count: 0,
      detailsCount: 0,
      summary: {
        avgChampionDamageRatioToTeamMax: 0,
        avgChampionDamageRatioToMax: 0,
        avgChampionDamagePercentageOfTeam: 0,
        avgChampionDamagePerMinute: 0,
        avgDamageTakenRatioToTeamMax: 0,
        avgDamageTakenRatioToMax: 0,
        avgDamageTakenPercentageOfTeam: 0,
        avgGoldRatioToTeamMax: 0,
        avgGoldRatioToMax: 0,
        avgGoldPercentageOfTeam: 0,
        avgCsRatioToTeamMax: 0,
        avgCsRatioToMax: 0,
        avgCsPercentageOfTeam: 0,
        avgCsPerMinute: 0,
        avgTowerDamageRatioToTeamMax: 0,
        avgTowerDamageRatioToMax: 0,
        avgTowerDamagePercentageOfTeam: 0,
        avgVisionScore: 0,
        avgVisionScorePercentageOfTeam: 0,
        avgDamageGoldEfficiency: 0,
        avgKillParticipation: 0,
        avgKillDamageEfficiency: 1.0,
        kills: 0,
        deaths: 0,
        assists: 0,
        avgKda: 0,
        kdaCv: 0,
        winRate: 0,
        avgSoloKills: null,
        avgEnemyMissingPings: null,
        avgPings: null
      },
      winLoss: {
        all: {
          count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
          activeSessionWins: 0, activeSessionLosses: 0
        },
        normal: {
          count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
          activeSessionWins: 0, activeSessionLosses: 0
        },
        cherry: {
          count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
          activeSessionWins: 0, activeSessionLosses: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
          topHalfRate: 0, avgSubteamPlacement: 0
        }
      },
      spells: {
        flashOnD: 0,
        flashOnF: 0
      },
      teamSide: {
        redSideCount: 0,
        blueSideCount: 0
      },
      positions: {
        TOP: 0, JUNGLE: 0, MIDDLE: 0, BOTTOM: 0, UTILITY: 0
      },
      champions: {},
      map: {},
      jungle: player.jungleAnalysis ?? null,
      akariScore: emptyAkariScore,
      details: {
        avgEarlyDeathsWithEnemyJunglerInvolved: null
      }
    }
  }

  // 3. Fallback when recent_games_count > 0 but matches array is missing:
  // Derive faithfully from scalar fields without inventing fake game metrics
  const winRate = (player.recent_win_rate ?? 0) / 100
  const wins = Math.round(count * winRate)
  const losses = count - wins
  const kdaRatio = player.recent_kda_ratio ?? 0

  let winningStreak = 0
  let losingStreak = 0
  if (player.recent_matches && player.recent_matches.length > 0) {
    for (const match of player.recent_matches) {
      if (match === 'win') {
        if (losingStreak > 0) break
        winningStreak++
      } else {
        if (winningStreak > 0) break
        losingStreak++
      }
    }
  }

  let kills = 0
  let deaths = 0
  let assists = 0
  if (player.recent_kda && typeof player.recent_kda === 'string') {
    const parts = player.recent_kda.split('/').map((s) => parseFloat(s.trim()))
    if (parts.length === 3 && !parts.some(isNaN)) {
      kills = parts[0]
      deaths = parts[1]
      assists = parts[2]
    }
  }

  const isFlashD = player.spell1_id === 4
  const isFlashF = player.spell2_id === 4
  const flashOnD = isFlashD ? count : 0
  const flashOnF = isFlashF ? count : 0

  const kdaScore = Math.min(1.0, kdaRatio / 5.0)
  const winRateScore = Math.min(1.0, winRate)
  const dmgScore = Math.min(3.0, (player.radar?.output ?? player.radar?.damage ?? 0) / 33)
  const dmgTakenScore = Math.min(2.0, (player.radar?.survival ?? 0) / 45)
  const csScore = Math.min(2.0, (player.radar?.gold ?? 0) / 45)
  const goldScore = Math.min(2.0, (player.radar?.gold ?? 0) / 45)
  const participationScore = Math.min(2.0, (player.radar?.join_rate ?? 0) / 45)
  const visionScore = Math.min(2.0, (player.radar?.vision ?? 0) / 45)
  const akariTotal = Math.min(17.0, kdaScore + winRateScore + dmgScore + dmgTakenScore + csScore + goldScore + participationScore + visionScore)
  const isExtraordinary = akariTotal >= 13.5 && count >= 5
  const isOutstanding = akariTotal >= 11.5 && count >= 5

  const akariScore: AkariScore = {
    total: akariTotal,
    maxScore: 17.0,
    kdaScore,
    winRateScore,
    dmgScore,
    dmgTakenScore,
    healingScore: 0,
    csScore,
    goldScore,
    participationScore,
    visionScore,
    outstanding: isOutstanding ? ({ total: akariTotal } as any) : null,
    extraordinary: isExtraordinary ? ({ total: akariTotal } as any) : null
  }

  const champions: Record<string, AggregatedChampionAnalysis> = {}
  if (player.top_champs && player.top_champs.length > 0) {
    for (const tc of player.top_champs) {
      if (tc.champion_id > 0) {
        const tcWinRate = (tc.win_rate || (tc.win_rate_pct / 100))
        const tcWins = Math.round(tcWinRate * tc.games)
        champions[String(tc.champion_id)] = {
          championId: tc.champion_id,
          summary: {
            avgKda: tc.kda,
            winRate: tcWinRate,
            avgChampionDamagePercentageOfTeam: 0,
            avgDamageTakenPercentageOfTeam: 0,
            avgCsPercentageOfTeam: 0,
            avgCsPerMinute: 0,
            avgGoldPercentageOfTeam: 0,
            avgKillParticipation: 0,
            avgChampionDamagePerMinute: 0,
            kills: 0,
            deaths: 0,
            assists: 0,
            avgChampionDamageRatioToTeamMax: 0,
            avgChampionDamageRatioToMax: 0,
            avgDamageTakenRatioToTeamMax: 0,
            avgDamageTakenRatioToMax: 0,
            avgGoldRatioToTeamMax: 0,
            avgGoldRatioToMax: 0,
            avgCsRatioToTeamMax: 0,
            avgCsRatioToMax: 0,
            avgTowerDamageRatioToTeamMax: 0,
            avgTowerDamageRatioToMax: 0,
            avgTowerDamagePercentageOfTeam: 0,
            avgVisionScore: 0,
            avgVisionScorePercentageOfTeam: 0,
            avgDamageGoldEfficiency: 0,
            avgKillDamageEfficiency: 1.0,
            kdaCv: 0,
            avgSoloKills: null,
            avgEnemyMissingPings: null,
            avgPings: null
          },
          winLoss: {
            all: {
              count: tc.games,
              wins: tcWins,
              losses: tc.games - tcWins,
              winRate: tcWinRate,
              winningStreak: 0,
              losingStreak: 0,
              activeSessionWins: tcWins,
              activeSessionLosses: tc.games - tcWins
            },
            normal: {
              count: tc.games,
              wins: tcWins,
              losses: tc.games - tcWins,
              winRate: tcWinRate,
              winningStreak: 0,
              losingStreak: 0,
              activeSessionWins: tcWins,
              activeSessionLosses: tc.games - tcWins
            },
            cherry: {
              count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
              activeSessionWins: 0, activeSessionLosses: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
              topHalfRate: 0, avgSubteamPlacement: 0
            }
          },
          positions: { TOP: 0, JUNGLE: 0, MIDDLE: 0, BOTTOM: 0, UTILITY: 0 },
          akariScore: {
            total: 0, maxScore: 17.0, kdaScore: Math.min(1.0, tc.kda / 5.0), winRateScore: Math.min(1.0, tcWinRate),
            dmgScore: 0, dmgTakenScore: 0, healingScore: 0, csScore: 0, goldScore: 0, participationScore: 0, visionScore: 0,
            outstanding: null, extraordinary: null
          },
          jungle: null
        }
      }
    }
  } else if (player.champion_id > 0 && player.champ_recent_games && player.champ_recent_games > 0) {
    const cCount = player.champ_recent_games
    const cWinRate = (player.champ_recent_win_rate ?? 0) / 100
    const cWins = Math.round(cCount * cWinRate)
    champions[String(player.champion_id)] = {
      championId: player.champion_id,
      summary: {
        avgKda: kdaRatio,
        winRate: cWinRate,
        avgChampionDamagePercentageOfTeam: 0,
        avgDamageTakenPercentageOfTeam: 0,
        avgCsPercentageOfTeam: 0,
        avgCsPerMinute: 0,
        avgGoldPercentageOfTeam: 0,
        avgKillParticipation: 0,
        avgChampionDamagePerMinute: 0,
        kills: Math.round(kills),
        deaths: Math.round(deaths),
        assists: Math.round(assists),
        avgChampionDamageRatioToTeamMax: 0,
        avgChampionDamageRatioToMax: 0,
        avgDamageTakenRatioToTeamMax: 0,
        avgDamageTakenRatioToMax: 0,
        avgGoldRatioToTeamMax: 0,
        avgGoldRatioToMax: 0,
        avgCsRatioToTeamMax: 0,
        avgCsRatioToMax: 0,
        avgTowerDamageRatioToTeamMax: 0,
        avgTowerDamageRatioToMax: 0,
        avgTowerDamagePercentageOfTeam: 0,
        avgVisionScore: 0,
        avgVisionScorePercentageOfTeam: 0,
        avgDamageGoldEfficiency: 0,
        avgKillDamageEfficiency: 1.0,
        kdaCv: 0,
        avgSoloKills: null,
        avgEnemyMissingPings: null,
        avgPings: null
      },
      winLoss: {
        all: {
          count: cCount,
          wins: cWins,
          losses: cCount - cWins,
          winRate: cWinRate,
          winningStreak: 0,
          losingStreak: 0,
          activeSessionWins: cWins,
          activeSessionLosses: cCount - cWins
        },
        normal: {
          count: cCount,
          wins: cWins,
          losses: cCount - cWins,
          winRate: cWinRate,
          winningStreak: 0,
          losingStreak: 0,
          activeSessionWins: cWins,
          activeSessionLosses: cCount - cWins
        },
        cherry: {
          count: 0, wins: 0, losses: 0, winRate: 0, winningStreak: 0, losingStreak: 0,
          activeSessionWins: 0, activeSessionLosses: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
          topHalfRate: 0, avgSubteamPlacement: 0
        }
      },
      positions: { TOP: 0, JUNGLE: 0, MIDDLE: 0, BOTTOM: 0, UTILITY: 0 },
      akariScore: {
        total: 0, maxScore: 17.0, kdaScore: Math.min(1.0, kdaRatio / 5.0), winRateScore: Math.min(1.0, cWinRate),
        dmgScore: 0, dmgTakenScore: 0, healingScore: 0, csScore: 0, goldScore: 0, participationScore: 0, visionScore: 0,
        outstanding: null, extraordinary: null
      },
      jungle: null
    }
  }

  return {
    count,
    detailsCount: count,
    summary: {
      avgChampionDamageRatioToTeamMax: 0,
      avgChampionDamageRatioToMax: 0,
      avgChampionDamagePercentageOfTeam: 0,
      avgChampionDamagePerMinute: 0,
      avgDamageTakenRatioToTeamMax: 0,
      avgDamageTakenRatioToMax: 0,
      avgDamageTakenPercentageOfTeam: 0,
      avgGoldRatioToTeamMax: 0,
      avgGoldRatioToMax: 0,
      avgGoldPercentageOfTeam: 0,
      avgCsRatioToTeamMax: 0,
      avgCsRatioToMax: 0,
      avgCsPercentageOfTeam: 0,
      avgCsPerMinute: 0,
      avgTowerDamageRatioToTeamMax: 0,
      avgTowerDamageRatioToMax: 0,
      avgTowerDamagePercentageOfTeam: 0,
      avgVisionScore: player.radar?.vision ?? 0,
      avgVisionScorePercentageOfTeam: 0,
      avgDamageGoldEfficiency: 0,
      avgKillParticipation: 0,
      avgKillDamageEfficiency: 1.0,
      kills: Math.round(kills),
      deaths: Math.round(deaths),
      assists: Math.round(assists),
      avgKda: kdaRatio,
      kdaCv: 0,
      winRate,
      avgSoloKills: null,
      avgEnemyMissingPings: null,
      avgPings: null
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
        count: 0, activeSessionWins: 0, activeSessionLosses: 0, wins: 0, losses: 0, winRate: 0,
        winningStreak: 0, losingStreak: 0, top1s: 0, topHalfFinishes: 0, top1Rate: 0,
        topHalfRate: 0, avgSubteamPlacement: 0
      }
    },
    spells: {
      flashOnD,
      flashOnF
    },
    teamSide: {
      redSideCount: 0,
      blueSideCount: 0
    },
    positions: {
      TOP: player.assigned_position === 'TOP' ? count : 0,
      JUNGLE: player.assigned_position === 'JUNGLE' ? count : 0,
      MIDDLE: player.assigned_position === 'MIDDLE' || player.assigned_position === 'MID' ? count : 0,
      BOTTOM: player.assigned_position === 'BOTTOM' || player.assigned_position === 'BOT' ? count : 0,
      UTILITY: player.assigned_position === 'UTILITY' || player.assigned_position === 'SUP' ? count : 0
    },
    champions,
    map: {},
    jungle: player.jungleAnalysis ?? null,
    akariScore,
    details: {
      avgEarlyDeathsWithEnemyJunglerInvolved: null
    }
  }
}

export function createSynthesizedJungleAnalysis(player: EnhancedPlayerCard): AggregatedJungleAnalysis | null {
  if (player.jungleAnalysis) {
    return player.jungleAnalysis
  }
  return null
}
