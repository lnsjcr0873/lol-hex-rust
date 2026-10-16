import { LcuOrSgpGameSummary } from '../wrapper'

export type MatchBasicInfo = {
  dataSource: LcuOrSgpGameSummary['source']
  gameVersion: string
  gameId: number
  isTwoTeam: boolean
  isCherrySubteam: boolean
  endOfGameResult?: string
  gameCreation: number
  gameDuration: number
  gameType: string
  queueId: number
  gameMode: string
  mapId: number
  gameModeMutators: string[] | null
}

export function toBasicInfo(summary: LcuOrSgpGameSummary): MatchBasicInfo {
  if (!summary) {
    return {
      dataSource: 'lcu',
      gameVersion: '',
      gameId: 0,
      isTwoTeam: true,
      isCherrySubteam: false,
      endOfGameResult: undefined,
      gameCreation: 0,
      gameDuration: 0,
      gameType: 'MATCHED_GAME',
      queueId: 0,
      gameMode: 'CLASSIC',
      mapId: 11,
      gameModeMutators: null
    }
  }

  const { source, data } = summary

  if (source === 'sgp') {
    const json = data?.json || ({} as any)
    return {
      dataSource: source,
      gameVersion: json.gameVersion || '',
      gameId: json.gameId || summary.gameId || 0,
      isTwoTeam: json.gameMode !== 'CHERRY',
      isCherrySubteam: json.gameMode === 'CHERRY',
      endOfGameResult: json.endOfGameResult,
      gameCreation: json.gameCreation || 0,
      gameDuration: json.gameDuration || 0,
      gameType: json.gameType || 'MATCHED_GAME',
      queueId: json.queueId || 0,
      gameMode: json.gameMode || 'CLASSIC',
      mapId: json.mapId || 11,
      gameModeMutators: json.gameModeMutators ?? null
    }
  }

  const rawData = data || ({} as any)
  return {
    dataSource: source,
    gameVersion: rawData.gameVersion || '',
    gameId: rawData.gameId || summary.gameId || 0,
    isTwoTeam: rawData.gameMode !== 'CHERRY',
    isCherrySubteam: rawData.gameMode === 'CHERRY',
    endOfGameResult: rawData.endOfGameResult,
    gameCreation: rawData.gameCreation || 0,
    gameDuration: rawData.gameDuration || 0,
    gameType: rawData.gameType || 'MATCHED_GAME',
    queueId: rawData.queueId || 0,
    gameMode: rawData.gameMode || 'CLASSIC',
    mapId: rawData.mapId || 11,
    gameModeMutators: rawData.gameModeMutators ?? null
  }
}
