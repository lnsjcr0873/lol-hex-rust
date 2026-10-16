import {
  AKARI_DAMAGE_TAKEN_WEIGHT,
  AKARI_DAMAGE_WEIGHT,
  AKARI_GOLD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_GOLD_WEIGHT,
  AKARI_MAX_SCORE,
  AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_VISION_MAX_SCORE,
  AGGREGATE_AKARI_OUTSTANDING_THRESHOLD,
  AGGREGATE_AKARI_EXTRAORDINARY_THRESHOLD
} from '../constants'
import {
  scoreCsPerMinute,
  scoreExpectedContribution,
  scoreHealing,
  scoreKda,
  scoreParticipation,
  scoreWinRate
} from '../scoring'
import type { AkariScore, SingleAnalysis } from '../types/single'

export function computeSingleAkariScore(summary?: SingleAnalysis['summary'] | null): AkariScore {
  if (!summary) {
    return {
      kdaScore: 0,
      winRateScore: 0,
      dmgScore: 0,
      dmgTakenScore: 0,
      healingScore: 0,
      csScore: 0,
      goldScore: 0,
      participationScore: 0,
      visionScore: 0,
      total: 0,
      maxScore: AKARI_MAX_SCORE,
      outstanding: false,
      extraordinary: false
    }
  }

  const kdaScore = scoreKda(summary.kda)
  const winRateScore = scoreWinRate(summary.win ? 1 : 0)
  const dmgScore = scoreExpectedContribution(
    summary.championDamageRatioToExpectedContribution,
    AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
    AKARI_DAMAGE_WEIGHT
  )
  const dmgTakenScore = scoreExpectedContribution(
    summary.damageTakenRatioToExpectedContribution,
    AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
    AKARI_DAMAGE_TAKEN_WEIGHT
  )
  const healingScore = scoreHealing(
    summary.healingRatioToTeamAverageDamageTaken,
    summary.teamParticipantCount
  )
  const csScore = scoreCsPerMinute(summary.csPerMinute)
  const goldScore = scoreExpectedContribution(
    summary.goldRatioToExpectedContribution,
    AKARI_GOLD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
    AKARI_GOLD_WEIGHT
  )
  const participationScore = scoreParticipation(summary.killParticipation)
  const visionScore = scoreExpectedContribution(
    summary.visionScoreRatioToExpectedContribution,
    AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
    AKARI_VISION_MAX_SCORE
  )

  const total =
    kdaScore +
    winRateScore +
    dmgScore +
    dmgTakenScore +
    healingScore +
    csScore +
    goldScore +
    participationScore +
    visionScore

  const safeNum = (n: number) => (Number.isFinite(n) ? n : 0)

  return {
    kdaScore: safeNum(kdaScore),
    winRateScore: safeNum(winRateScore),
    dmgScore: safeNum(dmgScore),
    dmgTakenScore: safeNum(dmgTakenScore),
    healingScore: safeNum(healingScore),
    csScore: safeNum(csScore),
    goldScore: safeNum(goldScore),
    participationScore: safeNum(participationScore),
    visionScore: safeNum(visionScore),
    total: safeNum(total),
    maxScore: AKARI_MAX_SCORE,
    outstanding: safeNum(total) >= AGGREGATE_AKARI_OUTSTANDING_THRESHOLD,
    extraordinary: safeNum(total) >= AGGREGATE_AKARI_EXTRAORDINARY_THRESHOLD
  }
}
