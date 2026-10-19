import { Participant } from '@shared/types/league-client/match-history'

import { noZero } from '../utils'
import { LcuOrSgpGameSummary } from '../wrapper'
import { toBasicInfo, type MatchBasicInfo } from './match-basic'
import { WinResult, computeWinResult } from './win-result'

export type MatchParticipantPerkSelection = {
  perk: number
  var1: number
  var2: number
  var3: number
}

export type MatchParticipantPerkStyle = {
  description: string
  selections: MatchParticipantPerkSelection[]
  style: number
}

export type MatchParticipantStatPerks = {
  offense: number
  flex: number
  defense: number
}

export type MatchParticipantPerks = {
  statPerks: MatchParticipantStatPerks | null
  styles: MatchParticipantPerkStyle[]
}

export type MatchParticipantPings = {
  allInPings: number
  assistMePings: number
  basicPings: number
  commandPings: number
  dangerPings: number
  enemyMissingPings: number
  enemyVisionPings: number
  getBackPings: number
  holdPings: number
  needVisionPings: number
  onMyWayPings: number
  pushPings: number
  retreatPings: number
  visionClearedPings: number
}

export type MatchParticipant = {
  puuid: string
  participantId: number
  gameName: string
  tagLine: string
  profileIconId: number
  championId: number
  position: string | null
  teamId: number
  playerSubteamId: number
  teamIdentifier: string
  items: number[]
  roleBoundItem: number
  augments: number[]
  spells: number[]
  perks: MatchParticipantPerks
  level: number
  kills: number
  deaths: number
  assists: number
  kda: number
  killParticipation: number
  totalDamageDealtToChampions: number
  damageGoldEfficiency: number
  physicalDamageDealtToChampions: number
  magicDamageDealtToChampions: number
  trueDamageDealtToChampions: number
  totalDamageTaken: number
  physicalDamageTaken: number
  magicDamageTaken: number
  trueDamageTaken: number
  goldEarned: number
  goldSpent: number
  neutralMinionsKilled: number
  totalMinionsKilled: number
  cs: number
  win: boolean
  isSurrender: boolean
  winResult: WinResult
  subteamPlacement: number
  gameEndedInEarlySurrender: boolean
  gameEndedInSurrender: boolean
  teamEarlySurrendered: boolean
  totalDamageToTowers: number
  totalHeal: number
  visionScore: number
  timeCCingOthers: number
  soloKills: number | null
  effectiveHealAndShielding: number | null
  totalDamageShieldedOnTeammates: number | null
  pings: MatchParticipantPings | null
  knockEnemyIntoTeamAndKill: number | null
  killsNearEnemyTurret: number | null
  killsUnderOwnTurret: number | null
  earliestDragonTakedown: number | null
  maxCsAdvantageOnLaneOpponent: number | null
  doubleKills: number
  tripleKills: number
  quadraKills: number
  pentaKills: number
  tier?: string
  championLevel?: number
  isMvp?: boolean
  isSvp?: boolean
  firstBloodKill?: boolean
  wardsPlaced?: number
  wardsKilled?: number
  detectorWardsPlaced?: number
}

// 以 SGP 的格式为参照，将 LCU 数据映射为抽象格式
function mapLcuDataToPerks(participant: Participant): MatchParticipantPerks {
  const stats = participant?.stats || ({} as any)
  return {
    statPerks: null, // lcu has no stat perks record
    styles: [
      {
        description: 'primaryStyle',
        selections: [
          {
            perk: stats.perk0 || 0,
            var1: stats.perk0Var1 || 0,
            var2: stats.perk0Var2 || 0,
            var3: stats.perk0Var3 || 0
          },
          {
            perk: stats.perk1 || 0,
            var1: stats.perk1Var1 || 0,
            var2: stats.perk1Var2 || 0,
            var3: stats.perk1Var3 || 0
          },
          {
            perk: stats.perk2 || 0,
            var1: stats.perk2Var1 || 0,
            var2: stats.perk2Var2 || 0,
            var3: stats.perk2Var3 || 0
          },
          {
            perk: stats.perk3 || 0,
            var1: stats.perk3Var1 || 0,
            var2: stats.perk3Var2 || 0,
            var3: stats.perk3Var3 || 0
          }
        ],
        style: stats.perkPrimaryStyle || 0
      },
      {
        description: 'subStyle',
        selections: [
          {
            perk: stats.perk4 || 0,
            var1: stats.perk4Var1 || 0,
            var2: stats.perk4Var2 || 0,
            var3: stats.perk4Var3 || 0
          },
          {
            perk: stats.perk5 || 0,
            var1: stats.perk5Var1 || 0,
            var2: stats.perk5Var2 || 0,
            var3: stats.perk5Var3 || 0
          }
        ],
        style: stats.perkSubStyle || 0
      }
    ]
  }
}

export function toParticipants(
  summary: LcuOrSgpGameSummary,
  basicInfo?: MatchBasicInfo
): MatchParticipant[] {
  if (!summary) return []
  const safeBasicInfo = basicInfo || toBasicInfo(summary)
  const { source, data } = summary

  if (source === 'sgp') {
    const participantsList = data?.json?.participants || []
    const hasFullTeam = participantsList.length > 1
    const totalKills = hasFullTeam
      ? participantsList.reduce(
          (acc, p) => {
            const teamIdentifier = safeBasicInfo.isCherrySubteam
              ? `CHERRY-${p.playerSubteamId || 0}`
              : `TEAM-${p.teamId || 0}`

            acc[teamIdentifier] = (acc[teamIdentifier] || 0) + (p.kills || 0)
            return acc
          },
          {} as Record<string, number>
        )
      : {}

    const participants = participantsList.map<MatchParticipant>((p) => {
      const teamIdentifier = safeBasicInfo.isCherrySubteam
        ? `CHERRY-${p.playerSubteamId || 0}`
        : `TEAM-${p.teamId || 0}`

      const { isSurrender, result } = computeWinResult(safeBasicInfo.endOfGameResult, p)

      return {
        puuid: p.puuid || '',
        participantId: p.participantId || 0,
        gameName: p.riotIdGameName || p.summonerName || '',
        tagLine: p.riotIdTagline || '',
        profileIconId: p.profileIcon || 0,
        championId: p.championId || 0,
        position: p.teamPosition || null,
        teamId: p.teamId || 0,
        playerSubteamId: p.playerSubteamId || 0,
        teamIdentifier,
        items: [p.item0, p.item1, p.item2, p.item3, p.item4, p.item5, p.item6],
        roleBoundItem: p.roleBoundItem,
        augments: [
          p.playerAugment1,
          p.playerAugment2,
          p.playerAugment3,
          p.playerAugment4,
          p.playerAugment5,
          p.playerAugment6
        ],
        spells: [p.spell1Id, p.spell2Id],
        perks: p.perks,
        level: p.champLevel || 1,
        kills: p.kills || 0,
        deaths: p.deaths || 0,
        assists: p.assists || 0,
        kda: ((p.kills || 0) + (p.assists || 0)) / noZero(p.deaths || 0),
        killParticipation:
          hasFullTeam && totalKills[teamIdentifier] && totalKills[teamIdentifier] > 0
            ? Math.min(1, ((p.kills || 0) + (p.assists || 0)) / totalKills[teamIdentifier])
            : null,
        totalDamageDealtToChampions: p.totalDamageDealtToChampions || 0,
        damageGoldEfficiency:
          (p.totalDamageDealtToChampions || 0) / noZero(p.goldEarned || 0),
        physicalDamageDealtToChampions: p.physicalDamageDealtToChampions || 0,
        magicDamageDealtToChampions: p.magicDamageDealtToChampions || 0,
        trueDamageDealtToChampions: p.trueDamageDealtToChampions || 0,
        totalDamageTaken: p.totalDamageTaken || 0,
        physicalDamageTaken: p.physicalDamageTaken || 0,
        magicDamageTaken: p.magicDamageTaken || 0,
        trueDamageTaken: p.trueDamageTaken || 0,
        goldEarned: p.goldEarned || 0,
        goldSpent: p.goldSpent || 0,
        neutralMinionsKilled: p.neutralMinionsKilled || 0,
        totalMinionsKilled: p.totalMinionsKilled || 0,
        cs: (p.neutralMinionsKilled || 0) + (p.totalMinionsKilled || 0),
        win: p.win ?? false,
        isSurrender,
        winResult: result,
        subteamPlacement: p.subteamPlacement || 0,
        gameEndedInEarlySurrender: p.gameEndedInEarlySurrender ?? false,
        gameEndedInSurrender: p.gameEndedInSurrender ?? false,
        teamEarlySurrendered: p.teamEarlySurrendered ?? false,
        totalDamageToTowers: p.damageDealtToTurrets || 0,
        totalHeal: p.totalHeal || 0,
        visionScore: p.visionScore ?? 0,
        timeCCingOthers: p.timeCCingOthers,
        soloKills: p.challenges?.soloKills ?? null,
        effectiveHealAndShielding: p.challenges?.effectiveHealAndShielding ?? null,
        totalDamageShieldedOnTeammates: p.totalDamageShieldedOnTeammates,
        knockEnemyIntoTeamAndKill: p.challenges?.knockEnemyIntoTeamAndKill ?? null,
        killsNearEnemyTurret: p.challenges?.killsNearEnemyTurret ?? null,
        killsUnderOwnTurret: p.challenges?.killsUnderOwnTurret ?? null,
        earliestDragonTakedown: p.challenges?.earliestDragonTakedown ?? null,
        maxCsAdvantageOnLaneOpponent: p.challenges?.maxCsAdvantageOnLaneOpponent ?? null,

        pings: {
          allInPings: p.allInPings,
          assistMePings: p.assistMePings,
          basicPings: p.basicPings,
          commandPings: p.commandPings,
          dangerPings: p.dangerPings,
          enemyMissingPings: p.enemyMissingPings,
          enemyVisionPings: p.enemyVisionPings,
          getBackPings: p.getBackPings,
          holdPings: p.holdPings,
          needVisionPings: p.needVisionPings,
          onMyWayPings: p.onMyWayPings,
          pushPings: p.pushPings,
          retreatPings: p.retreatPings,
          visionClearedPings: p.visionClearedPings
        },

        doubleKills: p.doubleKills,
        tripleKills: p.tripleKills,
        quadraKills: p.quadraKills,
        pentaKills: p.pentaKills
      }
    })

    return assignMvpSvp(participants)
  }

  const rawParticipants = data.participants || []
  const hasFullTeam = rawParticipants.length > 1

  const totalKills = hasFullTeam
    ? rawParticipants.reduce(
        (acc, p) => {
          const stats = p.stats || ({} as any)
          const teamIdentifier = safeBasicInfo.isCherrySubteam
            ? `CHERRY-${stats.playerSubteamId || 0}`
            : `TEAM-${p.teamId || 0}`
          acc[teamIdentifier] = (acc[teamIdentifier] || 0) + (stats.kills || 0)
          return acc
        },
        {} as Record<string, number>
      )
    : {}

  const participants = (data.participants || [])
    .map<MatchParticipant | null>((participant) => {
      const identity = (data.participantIdentities || []).find(
        (i) => i.participantId === participant.participantId
      )
      const player = identity?.player || (participant as any).player || (participant as any)
      const puuid = player?.puuid || (participant as any).puuid || ''
      if (!identity && !puuid && !player?.gameName && !player?.summonerName) {
        return null
      }

      const stats = participant.stats || ({} as any)
      const teamIdentifier = safeBasicInfo.isCherrySubteam
        ? `CHERRY-${stats.playerSubteamId || 0}`
        : `TEAM-${participant.teamId || 0}`

      const { isSurrender, result } = computeWinResult(safeBasicInfo.endOfGameResult, stats)

      return {
        puuid,
        participantId: participant.participantId,
        gameName: player?.gameName || player?.summonerName || (participant as any).gameName || (participant as any).summonerName || '',
        tagLine: player?.tagLine || (participant as any).tagLine || '',
        profileIconId: player?.profileIcon || player?.profileIconId || (participant as any).profileIcon || 0,
        championId: participant.championId || 0,
        position: (participant as any).stats?.teamPosition || (participant as any).stats?.individualPosition || participant.timeline?.lane || null,
        teamId: participant.teamId || 0,
        playerSubteamId: stats.playerSubteamId || 0,
        teamIdentifier,
        items: [
          stats.item0 || 0,
          stats.item1 || 0,
          stats.item2 || 0,
          stats.item3 || 0,
          stats.item4 || 0,
          stats.item5 || 0,
          stats.item6 || 0
        ],
        roleBoundItem: stats.roleBoundItem || 0,
        augments: [
          stats.playerAugment1 || 0,
          stats.playerAugment2 || 0,
          stats.playerAugment3 || 0,
          stats.playerAugment4 || 0,
          stats.playerAugment5 || 0,
          stats.playerAugment6 || 0
        ],
        spells: [participant.spell1Id || 0, participant.spell2Id || 0],
        perks: mapLcuDataToPerks(participant),
        level: stats.champLevel || 1,
        kills: stats.kills || 0,
        deaths: stats.deaths || 0,
        assists: stats.assists || 0,
        kda:
          ((stats.kills || 0) + (stats.assists || 0)) / noZero(stats.deaths || 0),
        killParticipation:
          hasFullTeam && totalKills[teamIdentifier] && totalKills[teamIdentifier] > 0
            ? Math.min(1, ((stats.kills || 0) + (stats.assists || 0)) / totalKills[teamIdentifier])
            : null,
        totalDamageDealtToChampions: stats.totalDamageDealtToChampions || 0,
        damageGoldEfficiency:
          (stats.totalDamageDealtToChampions || 0) / noZero(stats.goldEarned || 0),
        physicalDamageDealtToChampions: stats.physicalDamageDealtToChampions || 0,
        magicDamageDealtToChampions: stats.magicDamageDealtToChampions || 0,
        trueDamageDealtToChampions: stats.trueDamageDealtToChampions || 0,
        totalDamageTaken: stats.totalDamageTaken || 0,
        physicalDamageTaken: stats.physicalDamageTaken || 0,
        magicDamageTaken: stats.magicalDamageTaken || 0,
        trueDamageTaken: stats.trueDamageTaken || 0,
        goldEarned: stats.goldEarned || 0,
        goldSpent: stats.goldSpent || 0,
        neutralMinionsKilled: stats.neutralMinionsKilled || 0,
        totalMinionsKilled: stats.totalMinionsKilled || 0,
        cs: (stats.neutralMinionsKilled || 0) + (stats.totalMinionsKilled || 0),
        win: stats.win ?? false,
        isSurrender,
        winResult: result,
        subteamPlacement: stats.subteamPlacement || 0,
        gameEndedInEarlySurrender: stats.gameEndedInEarlySurrender ?? false,
        gameEndedInSurrender: stats.gameEndedInSurrender ?? false,
        teamEarlySurrendered: stats.teamEarlySurrendered ?? false,
        totalDamageToTowers: stats.damageDealtToTurrets || 0,
        totalHeal: stats.totalHeal || 0,
        visionScore: stats.visionScore ?? 0,
        timeCCingOthers: stats.timeCCingOthers || 0,

        effectiveHealAndShielding: null, // lcu has no effective heal and shielding record
        totalDamageShieldedOnTeammates: null, // lcu has no total damage shielded on teammates record
        soloKills: null, // lcu has no solo kills record
        pings: null, // lcu has no pings record
        knockEnemyIntoTeamAndKill: null, // lcu has no knock enemy into team and kill record
        killsNearEnemyTurret: null, // lcu has no kills near enemy turret record
        killsUnderOwnTurret: null, // lcu has no kills under own turret record
        earliestDragonTakedown: null, // lcu has no earliest dragon takedown record
        maxCsAdvantageOnLaneOpponent: null, // lcu has no max cs advantage on lane opponent record

        doubleKills: stats.doubleKills || 0,
        tripleKills: stats.tripleKills || 0,
        quadraKills: stats.quadraKills || 0,
        pentaKills: stats.pentaKills || 0
      }
    })
    .filter(isNotNull)

  return assignMvpSvp(participants)
}

function computeParticipantScore(p: MatchParticipant): number {
  const kda = ((p.kills || 0) + (p.assists || 0) * 0.7) / Math.max(1, p.deaths || 0)
  const dmg = p.totalDamageDealtToChampions || 0
  const kp = p.killParticipation ?? 0.2
  const cs = p.cs || 0
  return kda * 3.0 + dmg / 5000.0 + kp * 5.0 + cs / 50.0
}

function assignMvpSvp(participants: MatchParticipant[]): MatchParticipant[] {
  if (participants.length <= 1) return participants

  const winGroup = participants.filter((p) => p.win)
  const lossGroup = participants.filter((p) => !p.win)

  let bestWin: MatchParticipant | null = null
  let bestWinScore = -Infinity
  for (const p of winGroup) {
    const s = computeParticipantScore(p)
    if (s > bestWinScore) {
      bestWinScore = s
      bestWin = p
    }
  }

  let bestLoss: MatchParticipant | null = null
  let bestLossScore = -Infinity
  for (const p of lossGroup) {
    const s = computeParticipantScore(p)
    if (s > bestLossScore) {
      bestLossScore = s
      bestLoss = p
    }
  }

  for (const p of participants) {
    p.isMvp = bestWin !== null && p === bestWin
    p.isSvp = bestLoss !== null && p === bestLoss
  }

  return participants
}

function isNotNull<T>(value: T | null): value is T {
  return value !== null
}
