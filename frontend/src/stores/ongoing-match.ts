import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { OngoingMatchData, PlayerCard, RadarStat, DraftOptions } from '../api/types'
import type { MatchParticipant } from '@shared/data-adapter/match-history/participants'
import {
  QUEUE_NAME_MAP,
  getChampionName,
  getChampionIconUrl,
  getSpellName,
  getSpellIconUrl
} from '../components/match-card/utils/resources'

export { type DraftOptions }

export const useOngoingMatchStore = defineStore('ongoing-match', () => {
  const draft = ref<DraftOptions | null>(null)
  const customOngoingData = ref<OngoingMatchData | null>(null)

  const isDryRunActive = computed(() => customOngoingData.value !== null)
  const isDraft = computed(() => draft.value !== null)

  function buildPlayerCard(
    puuid: string,
    teamColor: 'blue' | 'red',
    cellId: number,
    championId: number,
    position: string,
    participant?: MatchParticipant
  ): PlayerCard {
    const defaultRadar: RadarStat = {
      output: 65,
      damage: 65,
      kda: 60,
      join_rate: 55,
      survival: 70,
      gold: 60,
      vision: 50
    }

    const spell1 = participant?.spells?.[0] || 4
    const spell2 = participant?.spells?.[1] || 14

    return {
      puuid,
      summoner_id: puuid,
      game_name: participant?.gameName || `模拟召唤师 ${cellId + 1}`,
      tag_line: participant?.tagLine || 'CN',
      profile_icon_id: participant?.profileIconId || 1,
      summoner_level: 100,
      team: teamColor,
      cell_id: cellId,
      assigned_position: position || (participant?.position ?? 'MIDDLE'),
      champion_id: championId,
      champion_name: getChampionName(championId),
      champion_title: '',
      champion_icon: getChampionIconUrl(championId),
      is_locked: true,
      spell1_id: spell1,
      spell1_name: getSpellName(spell1),
      spell1_icon: getSpellIconUrl(spell1),
      spell2_id: spell2,
      spell2_name: getSpellName(spell2),
      spell2_icon: getSpellIconUrl(spell2),
      tier: 'DIAMOND',
      division: 'I',
      lp: 75,
      recent_kda: participant ? `${participant.kills}/${participant.deaths}/${participant.assists}` : '3.50:1',
      recent_kda_ratio: 3.5,
      recent_win_rate: 0.60,
      recent_games_count: 20,
      champ_recent_games: 5,
      champ_recent_win_rate: 0.60,
      recent_matches: ['win', 'win', 'loss', 'win', 'win'],
      badges: [],
      radar: defaultRadar
    }
  }

  function setDraft(newDraft: DraftOptions, participants: MatchParticipant[] = []) {
    if (!newDraft) return
    draft.value = newDraft

    const parts = Array.isArray(participants) ? participants : []
    const partMap = new Map<string, MatchParticipant>()
    for (const p of parts) {
      if (p.puuid) partMap.set(p.puuid, p)
    }

    const blueTeam: PlayerCard[] = []
    const redTeam: PlayerCard[] = []

    const teams = newDraft.teams || {}
    const teamKeys = Object.keys(teams)
    const isCherry = newDraft.gameModeKind === 'cherry' || teamKeys.includes('TEAM-ALL')

    if (isCherry) {
      const allPuuids = teams['TEAM-ALL'] || Object.values(teams).flat()
      allPuuids.forEach((puuid, idx) => {
        const champId = newDraft.championSelections?.[puuid] || 0
        const pos = newDraft.positions?.[puuid]?.selected || 'MIDDLE'
        const teamColor = idx < Math.ceil(allPuuids.length / 2) ? 'blue' : 'red'
        const card = buildPlayerCard(puuid, teamColor, idx, champId, pos, partMap.get(puuid))
        if (teamColor === 'blue') {
          blueTeam.push(card)
        } else {
          redTeam.push(card)
        }
      })
    } else {
      const blueKey = teamKeys.find((k) => k === '100' || k.toLowerCase().includes('blue')) || teamKeys[0] || '100'
      const redKey = teamKeys.find((k) => k === '200' || k.toLowerCase().includes('red')) || teamKeys[1] || '200'

      const bluePuuids = teams[blueKey] || []
      const redPuuids = teams[redKey] || []

      bluePuuids.forEach((puuid, idx) => {
        const champId = newDraft.championSelections?.[puuid] || 0
        const pos = newDraft.positions?.[puuid]?.selected || 'MIDDLE'
        blueTeam.push(buildPlayerCard(puuid, 'blue', idx, champId, pos, partMap.get(puuid)))
      })

      redPuuids.forEach((puuid, idx) => {
        const champId = newDraft.championSelections?.[puuid] || 0
        const pos = newDraft.positions?.[puuid]?.selected || 'MIDDLE'
        redTeam.push(buildPlayerCard(puuid, 'red', idx + 5, champId, pos, partMap.get(puuid)))
      })
    }

    const queueName = QUEUE_NAME_MAP[newDraft.queueId] || (isCherry ? '斗魂竞技场(模拟)' : '峡谷演练(5v5)')

    customOngoingData.value = {
      gameflow_phase: 'InProgress',
      queue_id: newDraft.queueId,
      queue_name: `${queueName} · 战术演练`,
      blue_team: blueTeam,
      red_team: redTeam,
      is_my_team_blue: true,
      my_puuid: newDraft.puuid || blueTeam[0]?.puuid || '',
      lcu_connected: true,
      is_active: true,
      is_mock: true
    }
  }

  function clearDraft() {
    draft.value = null
    customOngoingData.value = null
  }

  return {
    draft,
    customOngoingData,
    isDraft,
    isDryRunActive,
    setDraft,
    clearDraft
  }
})
