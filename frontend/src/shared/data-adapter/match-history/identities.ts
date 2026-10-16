import { LcuOrSgpGameSummary } from '../wrapper'

export type MatchParticipantIdentity = {
  puuid: string
  gameName: string
  tagLine: string
  profileIconId: number
  participantId: number
  teamId: number // raw
}

export function toIdentities(summary: LcuOrSgpGameSummary): MatchParticipantIdentity[] {
  if (summary.source === 'sgp') {
    return (summary.data?.json?.participants || []).map((participant) => ({
      puuid: participant.puuid || '',
      gameName: participant.riotIdGameName || participant.summonerName || '',
      tagLine: participant.riotIdTagline || '',
      profileIconId: participant.profileIcon || 0,
      participantId: participant.participantId || 0,
      teamId: participant.teamId || 0
    }))
  }

  const teamIdMap = new Map<number, number>()

  ;(summary.data?.participants || []).forEach((participant) => {
    teamIdMap.set(participant.participantId, participant.teamId)
  })

  return (summary.data?.participantIdentities || []).map((identity) => ({
    puuid: identity.player?.puuid || '',
    gameName: identity.player?.gameName || identity.player?.summonerName || '',
    tagLine: identity.player?.tagLine || '',
    profileIconId: identity.player?.profileIcon || 0,
    participantId: identity.participantId || 0,
    teamId: teamIdMap.get(identity.participantId) ?? 0
  }))
}
