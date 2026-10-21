import type { PlayerCard } from '../../../api/types'
import {
  WIN_RATE_TEAM_MIN_MATCHES,
  WIN_RATE_TEAM_MIN_SIZE,
  WIN_RATE_TEAM_MIN_WIN_RATE,
  WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK,
  LOSS_RATE_TEAM_MIN_SIZE,
  LOSS_RATE_TEAM_MAX_WIN_RATE,
  LOSS_RATE_TEAM_MIN_LOSS_STREAK
} from '../constants'

/**
 * Normalizes win rate to 0..100 percentage scale.
 * e.g. 0.60 -> 60, 60 -> 60.
 */
export function normalizeWinRatePercent(wr: number | undefined | null): number {
  if (wr === undefined || wr === null || !Number.isFinite(wr)) return 0
  return wr > 1 ? wr : wr * 100
}

/**
 * Normalizes win rate to 0.0..1.0 ratio scale.
 * e.g. 60 -> 0.60, 0.60 -> 0.60.
 */
export function normalizeWinRateRatio(wr: number | undefined | null): number {
  if (wr === undefined || wr === null || !Number.isFinite(wr)) return 0
  return wr > 1 ? wr / 100 : Math.max(0, wr)
}

/**
 * Calculates consecutive winning streak for a player from recent_matches.
 */
export function getPlayerWinningStreak(player: Partial<PlayerCard>): number {
  if (Array.isArray(player.recent_matches)) {
    let streak = 0
    for (const m of player.recent_matches) {
      if (m === 'win') streak++
      else break
    }
    return streak
  }
  return 0
}

/**
 * Calculates consecutive losing streak for a player from recent_matches.
 */
export function getPlayerLosingStreak(player: Partial<PlayerCard>): number {
  if (Array.isArray(player.recent_matches)) {
    let streak = 0
    for (const m of player.recent_matches) {
      if (m === 'loss') streak++
      else break
    }
    return streak
  }
  return 0
}

export type WinRateTeamBadgeType = 'win-rate-team' | 'loss-rate-team'

export interface WinRateTeamInfo {
  type: WinRateTeamBadgeType
}

/**
 * Detects whether a premade squad qualifies as "👑 胜率队" or "❄️ 败率队".
 */
export function detectWinRateTeamBadge(members: Partial<PlayerCard>[]): WinRateTeamInfo | undefined {
  if (!members || members.length === 0) return undefined

  // 1. Check "胜率队"
  if (members.length >= WIN_RATE_TEAM_MIN_SIZE) {
    let hasOneHighWinRateMember = false
    let otherMembersWinTotalStreak = 0

    for (const m of members) {
      const wrRatio = normalizeWinRateRatio(m.recent_win_rate)
      const count = m.recent_games_count ?? 0
      const streak = getPlayerWinningStreak(m)

      if (!hasOneHighWinRateMember && wrRatio >= WIN_RATE_TEAM_MIN_WIN_RATE && count >= WIN_RATE_TEAM_MIN_MATCHES) {
        hasOneHighWinRateMember = true
      } else {
        otherMembersWinTotalStreak += streak
      }
    }

    const avgStreak = members.length > 1 ? otherMembersWinTotalStreak / (members.length - 1) : 0
    const squadAvgWr = members.reduce((sum, m) => sum + normalizeWinRatePercent(m.recent_win_rate), 0) / members.length

    if ((hasOneHighWinRateMember && avgStreak >= WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK) || squadAvgWr >= 80) {
      return { type: 'win-rate-team' }
    }
  }

  // 2. Check "败率队"
  if (members.length >= LOSS_RATE_TEAM_MIN_SIZE) {
    let allLossQualified = true
    for (const m of members) {
      const wrRatio = normalizeWinRateRatio(m.recent_win_rate)
      if (wrRatio > LOSS_RATE_TEAM_MAX_WIN_RATE) {
        allLossQualified = false
        break
      }
    }

    const squadAvgWr = members.reduce((sum, m) => sum + normalizeWinRatePercent(m.recent_win_rate), 0) / members.length
    const hasSevereLossStreak = members.some((m) => getPlayerLosingStreak(m) >= LOSS_RATE_TEAM_MIN_LOSS_STREAK)

    if (allLossQualified || squadAvgWr <= 25 || hasSevereLossStreak) {
      return { type: 'loss-rate-team' }
    }
  }

  return undefined
}
