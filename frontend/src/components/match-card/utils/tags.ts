import { computed, type Ref } from 'vue'
import { MatchParticipant } from '@shared/data-adapter/match-history/participants'
import { TeamsAdapterResult } from '@shared/data-adapter/match-history/teams'
import { MatchBasicInfo } from '@shared/data-adapter/match-history/match-basic'

export interface PlayerTag {
  label: string
  color: string
  textColor: string
  content?: string
  priority?: number
}

export function computePlayerTags(
  participant: MatchParticipant | null,
  team: any | null,
  teams: TeamsAdapterResult,
  basicInfo: MatchBasicInfo
): PlayerTag[] {
  if (!participant) return []

  const tags: PlayerTag[] = []

  // MVP / SVP
  if (participant.isMvp) {
    tags.push({
      label: '👑 MVP',
      color: 'bg-amber-500/20 border border-amber-500/50',
      textColor: 'text-amber-400 font-black',
      content: '本局表现最佳选手 (MVP)',
      priority: 1000
    })
  } else if (participant.isSvp) {
    tags.push({
      label: '👑 SVP',
      color: 'bg-purple-500/20 border border-purple-500/50',
      textColor: 'text-purple-300 font-black',
      content: '败方表现最佳选手 (SVP)',
      priority: 950
    })
  }

  // Multi-kills
  if (participant.pentaKills > 0) {
    tags.push({
      label: participant.pentaKills > 1 ? `五杀 x${participant.pentaKills}` : '五杀',
      color: 'bg-rose-600/25 border border-rose-500/60',
      textColor: 'text-rose-300 font-bold',
      content: '单场完成五杀 (Penta Kill)',
      priority: 900
    })
  } else if (participant.quadraKills > 0) {
    tags.push({
      label: participant.quadraKills > 1 ? `四杀 x${participant.quadraKills}` : '四杀',
      color: 'bg-rose-600/20 border border-rose-500/40',
      textColor: 'text-rose-300 font-bold',
      content: '单场完成四杀 (Quadra Kill)',
      priority: 800
    })
  } else if (participant.tripleKills > 0) {
    tags.push({
      label: participant.tripleKills > 1 ? `三杀 x${participant.tripleKills}` : '三杀',
      color: 'bg-orange-500/20 border border-orange-500/40',
      textColor: 'text-orange-300 font-semibold',
      content: '单场完成三杀 (Triple Kill)',
      priority: 700
    })
  }

  // Damage leader
  if (
    participant.totalDamageDealtToChampions > 0 &&
    participant.totalDamageDealtToChampions === teams.allTeamStats.maxDamageDealtToChampions
  ) {
    tags.push({
      label: '全场输出最高',
      color: 'bg-red-950/80 border border-red-500/40',
      textColor: 'text-red-400 font-bold',
      content: `对英雄输出 ${participant.totalDamageDealtToChampions.toLocaleString()} (全场第 1)`,
      priority: 600
    })
  } else if (team && participant.totalDamageDealtToChampions === team.maxDamageDealtToChampions) {
    tags.push({
      label: '队内输出第一',
      color: 'bg-red-950/60 border border-red-500/30',
      textColor: 'text-red-300',
      content: `队内伤害占比最高`,
      priority: 500
    })
  }

  // Damage taken leader
  if (
    participant.totalDamageTaken > 0 &&
    participant.totalDamageTaken === teams.allTeamStats.maxDamageTaken
  ) {
    tags.push({
      label: '全场承伤最高',
      color: 'bg-slate-800/80 border border-slate-600/50',
      textColor: 'text-slate-300',
      content: `承受伤害 ${participant.totalDamageTaken.toLocaleString()}`,
      priority: 400
    })
  }

  // Gold leader
  if (participant.goldEarned > 0 && participant.goldEarned === teams.allTeamStats.maxGoldEarned) {
    tags.push({
      label: '经济第一',
      color: 'bg-amber-950/60 border border-amber-600/30',
      textColor: 'text-amber-300',
      content: `获得金币 ${(participant.goldEarned / 1000).toFixed(1)}k`,
      priority: 300
    })
  }

  // First blood
  if (participant.firstBloodKill) {
    tags.push({
      label: '第一滴血',
      color: 'bg-rose-950/60 border border-rose-600/30',
      textColor: 'text-rose-300',
      content: '斩获对局一血',
      priority: 200
    })
  }

  // Perfect KDA
  if (participant.deaths === 0 && (participant.kills > 0 || participant.assists > 0)) {
    tags.push({
      label: '完美KDA',
      color: 'bg-amber-500/20 border border-amber-400/40',
      textColor: 'text-amber-300 font-bold',
      content: '未阵亡 (Perfect)',
      priority: 850
    })
  }

  return tags.sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0))
}
