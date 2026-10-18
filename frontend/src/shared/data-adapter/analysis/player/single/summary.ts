import { MatchBasicInfo } from '../../../match-history/match-basic'
import { MatchParticipant } from '../../../match-history/participants'
import { noZero } from '../../../utils'
import type { SingleSummaryAnalysis } from '../types/single'

export function computeSingleSummary(
  basic: MatchBasicInfo,
  participant: MatchParticipant,
  teamParticipants: MatchParticipant[],
  allParticipants: MatchParticipant[]
): SingleSummaryAnalysis {
  const getVisionScore = (participant: MatchParticipant) => participant.visionScore ?? 0
  const getExpectedContributionRatio = (value: number, total: number) => {
    if (teamParticipants.length <= 1) {
      return 0
    }

    return value / noZero(total) / (1 / teamParticipants.length)
  }

  const durationMinutes = Math.max((basic.gameDuration || 0) / 60, 1)

  const teamMaxChampionDmg = Math.max(...teamParticipants.map((p) => p.totalDamageDealtToChampions), 0)
  const teamMaxDmgTaken = Math.max(...teamParticipants.map((p) => p.totalDamageTaken), 0)
  const teamMaxGold = Math.max(...teamParticipants.map((p) => p.goldEarned), 0)
  const teamMaxCs = Math.max(...teamParticipants.map((p) => p.cs), 0)
  const teamMaxTowerDmg = Math.max(...teamParticipants.map((p) => p.totalDamageToTowers), 0)
  const teamMaxShieldedDmg = Math.max(
    ...teamParticipants.map((p) => p.totalDamageShieldedOnTeammates ?? 0),
    0
  )
  const teamTotalChampionDmg = teamParticipants.reduce(
    (acc, p) => acc + p.totalDamageDealtToChampions,
    0
  )
  const teamTotalDmgTaken = teamParticipants.reduce((acc, p) => acc + p.totalDamageTaken, 0)
  const teamAverageDmgTaken = teamTotalDmgTaken / noZero(teamParticipants.length)
  const teamTotalGold = teamParticipants.reduce((acc, p) => acc + p.goldEarned, 0)
  const teamTotalCs = teamParticipants.reduce((acc, p) => acc + p.cs, 0)
  const teamTotalTowerDmg = teamParticipants.reduce((acc, p) => acc + p.totalDamageToTowers, 0)
  const teamTotalVisionScore = teamParticipants.reduce((acc, p) => acc + getVisionScore(p), 0)
  const teamTotalShieldedDmg = teamParticipants.reduce(
    (acc, p) => acc + (p.totalDamageShieldedOnTeammates ?? 0),
    0
  )
  const teamTotalKills = teamParticipants.reduce((acc, p) => acc + p.kills, 0)

  const maxChampionDmg = Math.max(...allParticipants.map((p) => p.totalDamageDealtToChampions), 0)
  const maxDmgTaken = Math.max(...allParticipants.map((p) => p.totalDamageTaken), 0)
  const maxGold = Math.max(...allParticipants.map((p) => p.goldEarned), 0)
  const maxCs = Math.max(...allParticipants.map((p) => p.cs), 0)
  const maxTowerDmg = Math.max(...allParticipants.map((p) => p.totalDamageToTowers), 0)
  const maxShieldedDmg = Math.max(
    ...allParticipants.map((p) => p.totalDamageShieldedOnTeammates ?? 0),
    0
  )

  return {
    championDamageRatioToTeamMax:
      participant.totalDamageDealtToChampions / noZero(teamMaxChampionDmg),
    championDamageRatioToExpectedContribution: getExpectedContributionRatio(
      participant.totalDamageDealtToChampions,
      teamTotalChampionDmg
    ),
    championDamageRatioToMax: participant.totalDamageDealtToChampions / noZero(maxChampionDmg),
    championDamagePercentageOfTeam:
      teamParticipants.length > 1
        ? participant.totalDamageDealtToChampions / noZero(teamTotalChampionDmg)
        : null,
    championDamagePerMinute: participant.totalDamageDealtToChampions / durationMinutes,
    damageTakenRatioToTeamMax: participant.totalDamageTaken / noZero(teamMaxDmgTaken),
    damageTakenRatioToExpectedContribution: getExpectedContributionRatio(
      participant.totalDamageTaken,
      teamTotalDmgTaken
    ),
    damageTakenRatioToMax: participant.totalDamageTaken / noZero(maxDmgTaken),
    damageTakenPercentageOfTeam:
      teamParticipants.length > 1
        ? participant.totalDamageTaken / noZero(teamTotalDmgTaken)
        : null,
    healingRatioToTeamAverageDamageTaken: participant.totalHeal / noZero(teamAverageDmgTaken),
    teamParticipantCount: teamParticipants.length,
    goldRatioToTeamMax: participant.goldEarned / noZero(teamMaxGold),
    goldRatioToExpectedContribution: getExpectedContributionRatio(
      participant.goldEarned,
      teamTotalGold
    ),
    goldRatioToMax: participant.goldEarned / noZero(maxGold),
    goldPercentageOfTeam:
      teamParticipants.length > 1
        ? participant.goldEarned / noZero(teamTotalGold)
        : null,
    csRatioToTeamMax: participant.cs / noZero(teamMaxCs),
    csRatioToMax: participant.cs / noZero(maxCs),
    csPercentageOfTeam:
      teamParticipants.length > 1
        ? participant.cs / noZero(teamTotalCs)
        : null,
    csPerMinute: participant.cs / durationMinutes,
    towerDamageRatioToTeamMax: participant.totalDamageToTowers / noZero(teamMaxTowerDmg),
    towerDamageRatioToMax: participant.totalDamageToTowers / noZero(maxTowerDmg),
    towerDamagePercentageOfTeam:
      teamParticipants.length > 1
        ? participant.totalDamageToTowers / noZero(teamTotalTowerDmg)
        : null,
    visionScorePercentageOfTeam:
      teamParticipants.length > 1
        ? getVisionScore(participant) / noZero(teamTotalVisionScore)
        : null,
    visionScoreRatioToExpectedContribution: getExpectedContributionRatio(
      getVisionScore(participant),
      teamTotalVisionScore
    ),
    totalDamageShieldedOnTeammatesRatioToTeamMax:
      participant.totalDamageShieldedOnTeammates !== null
        ? participant.totalDamageShieldedOnTeammates / noZero(teamMaxShieldedDmg)
        : null,
    totalDamageShieldedOnTeammatesRatioToMax:
      participant.totalDamageShieldedOnTeammates !== null
        ? participant.totalDamageShieldedOnTeammates / noZero(maxShieldedDmg)
        : null,
    totalDamageShieldedOnTeammatesPercentageOfTeam:
      participant.totalDamageShieldedOnTeammates !== null && teamParticipants.length > 1
        ? participant.totalDamageShieldedOnTeammates / noZero(teamTotalShieldedDmg)
        : null,
    kda: participant.kda,
    win: participant.winResult === 'win',
    killParticipation:
      teamParticipants.length > 1 && teamTotalKills > 0
        ? Math.min(1, (participant.kills + participant.assists) / teamTotalKills)
        : null,
    damageGoldEfficiency: participant.totalDamageDealtToChampions / noZero(participant.goldEarned),
    killDamageEfficiency: (() => {
      if (
        teamParticipants.length <= 1 ||
        teamTotalKills === 0 ||
        teamTotalChampionDmg === 0 ||
        participant.totalDamageDealtToChampions === 0
      ) {
        return 1
      }
      const val =
        (participant.kills / teamTotalKills) /
        (participant.totalDamageDealtToChampions / teamTotalChampionDmg)
      return Number.isFinite(val) ? val : 1
    })()
  }
}
