import type { LcuOrSgpGameDetails, LcuOrSgpGameSummary } from '@shared/data-adapter/wrapper'

export type MatchCardProps = {
  summary: LcuOrSgpGameSummary
  details?: LcuOrSgpGameDetails | null
  puuid?: string
  hidePrivacy?: boolean
  loadingDetails?: boolean
  replayState?: any | null
  canDryRunOngoingGame?: boolean
}

export type MatchCardEmits = {
  loadDetails: [gameId: number]
  downloadReplay: [gameId: number]
  watchReplay: [gameId: number]
  navigateToSummonerByPuuid: [puuid: string, setCurrent?: boolean]
  dryRunOngoingGame: [draft: any]
}

export type MatchCardExpose = {
  setExpanded: (expanded: boolean) => void
}
