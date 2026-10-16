import { Team } from '@shared/types/league-client/match-history'

import { noZero } from '../utils'
import { LcuOrSgpGameSummary } from '../wrapper'
import { toBasicInfo, type MatchBasicInfo } from './match-basic'
import { MatchParticipant } from './participants'
import { WinResult, computeWinResult } from './win-result'

export type TeamIdentifier =
  | 'TEAM-200' // 红色方
  | 'TEAM-100' // 蓝色方
  | `CHERRY-${number}` // 斗魂 N 个队伍
  | 'TEAM-ALL' // 当无法区分队伍的时候使用此占位符
  | 'LOBBY' // 当位于房间内，且尚未进入游戏中时，使用此占位符

export type MatchTeamBan = {
  championId: number
  pickTurn: number
}

export type MatchTeamObjectiveStats = {
  first: boolean | null
  kills: number | null
}

export type MatchTeamObjectives = {
  atakhan: MatchTeamObjectiveStats | null
  baron: MatchTeamObjectiveStats
  champion: MatchTeamObjectiveStats | null
  dragon: MatchTeamObjectiveStats
  horde: MatchTeamObjectiveStats
  inhibitor: MatchTeamObjectiveStats
  riftHerald: MatchTeamObjectiveStats
  tower: MatchTeamObjectiveStats
}

export type MatchTeamInfo = {
  bans: MatchTeamBan[]
  win: boolean | string
  teamId: number
  objectives: MatchTeamObjectives
}

export type MatchTeamStats = {
  teamIdentifier: string
  teamInfo: MatchTeamInfo | null // 如 CHERRY 模式，teamInfo 则没有必要
  winResult: WinResult
  isSurrender: boolean
  win: boolean
  subteamPlacement: number
  maxDamageDealtToChampions: number
  totalDamageDealtToChampions: number
  maxDamageTaken: number
  totalDamageTaken: number
  maxGoldEarned: number
  totalGoldEarned: number
  maxKills: number
  totalKills: number
  totalDeaths: number
  totalAssists: number
  maxCs: number
  totalCs: number
  maxDamageToTowers: number
  totalDamageToTowers: number
  maxHeal: number
  totalHeal: number
  maxKda: number
  totalKda: number
  maxKillParticipation: number
  totalKillParticipation: number
  maxTimeCCingOthers: number
  maxDamageGoldEfficiency: number
  maxDamageShieldedOnTeammates: number | null // sgp only
  totalDamageShieldedOnTeammates: number | null // sgp only
}

export type AggregateTeamStats = {
  teamIdentifier: string
  bans: MatchTeamBan[]
  maxDamageDealtToChampions: number
  totalDamageDealtToChampions: number
  maxDamageTaken: number
  totalDamageTaken: number
  maxGoldEarned: number
  totalGoldEarned: number
  maxKills: number
  totalKills: number
  totalDeaths: number
  totalAssists: number
  maxCs: number
  totalCs: number
  maxDamageToTowers: number
  totalDamageToTowers: number
  maxHeal: number
  totalHeal: number
  maxKda: number
  totalKda: number
  maxKillParticipation: number
  totalKillParticipation: number
  maxTimeCCingOthers: number
  maxDamageGoldEfficiency: number
  maxDamageShieldedOnTeammates: number | null // sgp only
  totalDamageShieldedOnTeammates: number | null // sgp only
}

export type TeamsAdapterResult = {
  teamStatMap: Record<string, MatchTeamStats>
  teamStatsArr: MatchTeamStats[]
  allTeamStats: AggregateTeamStats
}

// 截至至 2025-11-08 部分数据映射关系
function mapLcuToObjectives(team: Team): MatchTeamInfo {
  return {
    bans: team.bans,
    win: team.win,
    teamId: team.teamId,
    objectives: {
      atakhan: null,
      baron: {
        first: team.firstBaron,
        kills: team.baronKills
      },
      champion: null,
      dragon: {
        first: team.firstDargon,
        kills: team.dragonKills
      },
      horde: {
        first: null,
        kills: team.hordeKills
      },
      inhibitor: {
        first: team.firstInhibitor,
        kills: team.inhibitorKills
      },
      riftHerald: {
        first: null,
        kills: team.riftHeraldKills
      },
      tower: {
        first: team.firstTower,
        kills: team.towerKills
      }
    }
  }
}

function getTeamInfo(summary: LcuOrSgpGameSummary): MatchTeamInfo[] {
  if (!summary || !summary.data) return []
  if (summary.source === 'sgp') {
    const teams = summary.data.json?.teams || []

    return teams.map((t) => {
      return {
        bans: t.bans || [],
        win: t.win,
        teamId: t.teamId,
        objectives: t.objectives
      }
    })
  }

  return (summary.data.teams || []).map(mapLcuToObjectives)
}

export function toTeams(
  summary: LcuOrSgpGameSummary,
  basicInfo?: MatchBasicInfo,
  participants?: MatchParticipant[]
): TeamsAdapterResult {
  const teamsInfo = getTeamInfo(summary)
  const safeBasicInfo = basicInfo || toBasicInfo(summary)
  const safeParticipants = Array.isArray(participants) ? participants : []

  // 按 teamIdentifier 分组
  const grouped = safeParticipants.reduce(
    (acc, p) => {
      acc[p.teamIdentifier] = acc[p.teamIdentifier] || []
      acc[p.teamIdentifier].push(p)
      return acc
    },
    {} as Record<string, MatchParticipant[]>
  )

  // 不计算 cherry team
  const primitiveTeamInfoMap = teamsInfo.reduce<Record<string, MatchTeamInfo>>((acc, teamInfo) => {
    acc[`TEAM-${teamInfo.teamId}`] = teamInfo
    return acc
  }, {})

function safeMax(numbers: (number | null | undefined)[]): number {
  const valid = numbers.filter((n): n is number => typeof n === 'number' && Number.isFinite(n))
  return valid.length > 0 ? Math.max(...valid) : 0
}

  // 计算每个团队的统计数据
  const teamStatsArr: MatchTeamStats[] = Object.entries(grouped)
    .filter(([_, teamParticipants]) => teamParticipants && teamParticipants.length > 0)
    .map(([teamIdentifier, teamParticipants]) => {
      const firstParticipant = teamParticipants[0]
      const { isSurrender, result } = computeWinResult(
        safeBasicInfo.endOfGameResult,
        firstParticipant
      )

      const hasShieldStats = teamParticipants.some(
        (p) => typeof p.totalDamageShieldedOnTeammates === 'number' && Number.isFinite(p.totalDamageShieldedOnTeammates)
      )

      return {
        teamIdentifier,
        teamInfo: primitiveTeamInfoMap[teamIdentifier] ?? (null as MatchTeamInfo | null),
        winResult: result,
        isSurrender,
        win: firstParticipant.win,
        subteamPlacement: firstParticipant.subteamPlacement,
        maxDamageDealtToChampions: safeMax(
          teamParticipants.map((p) => p.totalDamageDealtToChampions)
        ),
        totalDamageDealtToChampions: teamParticipants.reduce(
          (acc, p) => acc + (p.totalDamageDealtToChampions || 0),
          0
        ),
        maxDamageTaken: safeMax(teamParticipants.map((p) => p.totalDamageTaken)),
        totalDamageTaken: teamParticipants.reduce((acc, p) => acc + (p.totalDamageTaken || 0), 0),
        maxGoldEarned: safeMax(teamParticipants.map((p) => p.goldEarned)),
        totalGoldEarned: teamParticipants.reduce((acc, p) => acc + (p.goldEarned || 0), 0),
        maxKills: safeMax(teamParticipants.map((p) => p.kills)),
        totalKills: teamParticipants.reduce((acc, p) => acc + (p.kills || 0), 0),
        totalDeaths: teamParticipants.reduce((acc, p) => acc + (p.deaths || 0), 0),
        totalAssists: teamParticipants.reduce((acc, p) => acc + (p.assists || 0), 0),
        maxCs: safeMax(teamParticipants.map((p) => p.cs)),
        totalCs: teamParticipants.reduce((acc, p) => acc + (p.cs || 0), 0),
        maxDamageToTowers: safeMax(teamParticipants.map((p) => p.totalDamageToTowers)),
        totalDamageToTowers: teamParticipants.reduce((acc, p) => acc + (p.totalDamageToTowers || 0), 0),
        maxHeal: safeMax(teamParticipants.map((p) => p.totalHeal)),
        totalHeal: teamParticipants.reduce((acc, p) => acc + (p.totalHeal || 0), 0),
        maxKda: safeMax(teamParticipants.map((p) => p.kda)),
        totalKda: teamParticipants.reduce((acc, p) => acc + (p.kda || 0), 0),
        maxKillParticipation: safeMax(teamParticipants.map((p) => p.killParticipation)),
        totalKillParticipation: teamParticipants.reduce((acc, p) => acc + (p.killParticipation || 0), 0),
        maxTimeCCingOthers: safeMax(teamParticipants.map((p) => p.timeCCingOthers)),
        maxDamageGoldEfficiency: safeMax(
          teamParticipants.map((p) => (p.totalDamageDealtToChampions || 0) / noZero(p.goldEarned || 0))
        ),
        maxDamageShieldedOnTeammates: hasShieldStats
          ? safeMax(teamParticipants.map((p) => p.totalDamageShieldedOnTeammates))
          : null,
        totalDamageShieldedOnTeammates: hasShieldStats
          ? teamParticipants.reduce((acc, p) => acc + (p.totalDamageShieldedOnTeammates ?? 0), 0)
          : null
      }
    })

  const hasAllTeamShieldStats = teamStatsArr.some(
    (t) => typeof t.maxDamageShieldedOnTeammates === 'number' && Number.isFinite(t.maxDamageShieldedOnTeammates)
  )

  const allTeamStats: AggregateTeamStats = {
    teamIdentifier: 'TEAM-ALL',
    bans: Object.values(primitiveTeamInfoMap).flatMap((team) => team.bans),
    maxDamageDealtToChampions: safeMax(teamStatsArr.map((t) => t.maxDamageDealtToChampions)),
    totalDamageDealtToChampions: teamStatsArr.reduce(
      (acc, t) => acc + t.totalDamageDealtToChampions,
      0
    ),
    maxDamageTaken: safeMax(teamStatsArr.map((t) => t.maxDamageTaken)),
    totalDamageTaken: teamStatsArr.reduce((acc, t) => acc + t.totalDamageTaken, 0),
    maxGoldEarned: safeMax(teamStatsArr.map((t) => t.maxGoldEarned)),
    totalGoldEarned: teamStatsArr.reduce((acc, t) => acc + t.totalGoldEarned, 0),
    maxKills: safeMax(teamStatsArr.map((t) => t.maxKills)),
    totalKills: teamStatsArr.reduce((acc, t) => acc + t.totalKills, 0),
    totalDeaths: teamStatsArr.reduce((acc, t) => acc + t.totalDeaths, 0),
    totalAssists: teamStatsArr.reduce((acc, t) => acc + t.totalAssists, 0),
    maxCs: safeMax(teamStatsArr.map((t) => t.maxCs)),
    totalCs: teamStatsArr.reduce((acc, t) => acc + t.totalCs, 0),
    maxDamageToTowers: safeMax(teamStatsArr.map((t) => t.maxDamageToTowers)),
    totalDamageToTowers: teamStatsArr.reduce((acc, t) => acc + t.totalDamageToTowers, 0),
    maxHeal: safeMax(teamStatsArr.map((t) => t.maxHeal)),
    totalHeal: teamStatsArr.reduce((acc, t) => acc + t.totalHeal, 0),
    maxKda: safeMax(teamStatsArr.map((t) => t.maxKda)),
    totalKda: teamStatsArr.reduce((acc, t) => acc + t.totalKda, 0),
    maxKillParticipation: safeMax(teamStatsArr.map((t) => t.maxKillParticipation)),
    totalKillParticipation: teamStatsArr.reduce((acc, t) => acc + t.totalKillParticipation, 0),
    maxTimeCCingOthers: safeMax(teamStatsArr.map((t) => t.maxTimeCCingOthers)),
    maxDamageGoldEfficiency: safeMax(teamStatsArr.map((t) => t.maxDamageGoldEfficiency)),
    maxDamageShieldedOnTeammates: hasAllTeamShieldStats
      ? safeMax(teamStatsArr.map((t) => t.maxDamageShieldedOnTeammates))
      : null,
    totalDamageShieldedOnTeammates: hasAllTeamShieldStats
      ? teamStatsArr.reduce((acc, t) => acc + (t.totalDamageShieldedOnTeammates ?? 0), 0)
      : null
  }

  // 转换为 Map 结构便于查询
  const teamStatMap = teamStatsArr.reduce<Record<string, MatchTeamStats>>((acc, teamStats) => {
    acc[teamStats.teamIdentifier] = teamStats
    return acc
  }, {})

  return {
    teamStatMap,
    teamStatsArr,
    allTeamStats
  }
}
