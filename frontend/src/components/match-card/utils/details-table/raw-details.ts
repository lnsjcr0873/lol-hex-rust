import { computed, toValue } from 'vue'
import { computeSingleAkariScore, computeSingleSummary } from '@shared/data-adapter/analysis/player'
import type { AkariScore } from '@shared/data-adapter/analysis/player'
import { useMatchCard } from '../../context'

export function useRawDetails() {
  const { summary, basicInfo, participants } = useMatchCard()

  const akariScoresByPuuid = computed(() => {
    const scores: Record<string, AkariScore> = {}

    for (const participant of participants.value) {
      const teamParticipants = participants.value.filter(
        (p) => p.teamIdentifier === participant.teamIdentifier
      )
      const summaryAnalysis = computeSingleSummary(
        basicInfo.value,
        participant,
        teamParticipants,
        participants.value
      )

      scores[participant.puuid] = computeSingleAkariScore(summaryAnalysis)
    }

    return scores
  })

  return computed(() => {
    const currentSummary = toValue(summary)
    if (!currentSummary) return []

    const { source, data } = currentSummary

    if (source === 'sgp') {
      const isCherryMode = data.json.gameMode === 'CHERRY'
      return (data.json.participants || [])
        .map((p: any) => {
          const { challenges, missions, PlayerBehavior, ...rest } = p
          const dmg = p.totalDamageDealtToChampions || 0
          const gold = p.goldEarned || 1
          return {
            ...rest,
            ...PlayerBehavior,
            ...challenges,
            damageGoldEfficiency: (dmg / gold) * 100,
            akariScore: akariScoresByPuuid.value[p.puuid],
            championId: p.championId,
            participantId: p.participantId,
            identity: {
              puuid: p.puuid,
              gameName: p.riotIdGameName || p.summonerName || '',
              tagLine: p.riotIdTagline || '',
              teamIdentifier: isCherryMode ? `CHERRY-${p.playerSubteamId}` : `TEAM-${p.teamId}`
            }
          }
        })
    }

    // LCU source
    const rawData = data as any
    const isCherryMode = rawData.gameMode === 'CHERRY'
    const lcuParticipants = rawData.participants || []
    const identities = rawData.participantIdentities || []

    return lcuParticipants
      .map((p: any) => {
        const idObj = identities.find((i: any) => i.participantId === p.participantId)
        const puuid = idObj?.player?.puuid || ''
        const sname = idObj?.player?.gameName || idObj?.player?.summonerName || ''
        const tag = idObj?.player?.tagLine || ''
        const stats = p.stats || {}
        const dmg = stats.totalDamageDealtToChampions || 0
        const gold = stats.goldEarned || 1

        return {
          ...stats,
          damageGoldEfficiency: (dmg / gold) * 100,
          akariScore: akariScoresByPuuid.value[puuid],
          championId: p.championId,
          participantId: p.participantId,
          identity: {
            puuid,
            gameName: sname,
            tagLine: tag,
            teamIdentifier: isCherryMode ? `CHERRY-${p.teamId}` : `TEAM-${p.teamId}`
          }
        }
      })
      .filter(Boolean)
  })
}
