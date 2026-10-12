import {
  type InjectionKey,
  type MaybeRefOrGetter,
  type Ref,
  computed,
  inject,
  provide,
  toRef,
  toValue
} from 'vue'
import { toBasicInfo, type MatchBasicInfo } from '@shared/data-adapter/match-history/match-basic'
import { toParticipants, type MatchParticipant } from '@shared/data-adapter/match-history/participants'
import { toTeams, type TeamsAdapterResult } from '@shared/data-adapter/match-history/teams'
import { toFrames } from '@shared/data-adapter/match-history/frames'
import type { LcuOrSgpGameDetails, LcuOrSgpGameSummary } from '@shared/data-adapter/wrapper'

export type MatchCardContext = {
  isExpanded: Ref<boolean>
  puuid: Ref<string | undefined>
  details: Ref<LcuOrSgpGameDetails | null | undefined>
  summary: Ref<LcuOrSgpGameSummary>
  hidePrivacy: Ref<boolean>
  loadingDetails: Ref<boolean>
  replayState: Ref<any>
  canDryRunOngoingGame: Ref<boolean>

  basicInfo: Ref<MatchBasicInfo>
  participants: Ref<MatchParticipant[]>
  teams: Ref<TeamsAdapterResult>
  frames: Ref<any[]>

  participant: Ref<MatchParticipant | null>
  team: Ref<any>

  navigateToSummonerByPuuid: (puuid: string, setCurrent?: boolean) => void
  loadReplay: (gameId: number) => void
  watchReplay: (gameId: number) => void
  loadDetails: (gameId: number) => void
  dryRunOngoingGame: () => void
}

export const MatchCardContextKey: InjectionKey<MatchCardContext> = Symbol('MatchCardContext')

export function useMatchCard(): MatchCardContext {
  const context = inject(MatchCardContextKey)
  if (!context) {
    throw new Error('useMatchCard must be used within a match card component')
  }
  return context
}

export function provideMatchCard(props: {
  isExpanded: MaybeRefOrGetter<boolean>
  summary: MaybeRefOrGetter<LcuOrSgpGameSummary>
  puuid: MaybeRefOrGetter<string | undefined>
  details: MaybeRefOrGetter<LcuOrSgpGameDetails | null | undefined>
  hidePrivacy: MaybeRefOrGetter<boolean>
  loadingDetails: MaybeRefOrGetter<boolean>
  replayState: MaybeRefOrGetter<any>
  canDryRunOngoingGame: MaybeRefOrGetter<boolean>

  navigateToSummonerByPuuid: (puuid: string, setCurrent?: boolean) => void
  loadReplay: (gameId: number) => void
  watchReplay: (gameId: number) => void
  loadDetails: (gameId: number) => void
  dryRunOngoingGame: (draft: any) => void
}) {
  const basicInfo = computed(() => toBasicInfo(toValue(props.summary)))
  const participants = computed(() => toParticipants(toValue(props.summary), basicInfo.value))
  const teams = computed(() => toTeams(toValue(props.summary), basicInfo.value, participants.value))
  const frames = computed(() => {
    const d = toValue(props.details)
    if (!d) return []
    return toFrames(d)
  })

  const participant = computed(() => {
    const focused = toValue(props.puuid)
    return participants.value.find((p) => p.puuid === focused) ?? participants.value[0] ?? null
  })

  const team = computed(() => {
    if (!participant.value) return null
    return teams.value.teamStatMap[participant.value.teamIdentifier] ?? null
  })

  provide(MatchCardContextKey, {
    isExpanded: toRef(props.isExpanded),
    summary: toRef(props.summary),
    details: toRef(props.details),
    puuid: toRef(props.puuid),
    loadingDetails: toRef(props.loadingDetails),
    replayState: toRef(props.replayState),
    hidePrivacy: toRef(props.hidePrivacy),
    canDryRunOngoingGame: toRef(props.canDryRunOngoingGame),

    basicInfo,
    participants,
    teams,
    frames,

    participant,
    team,

    navigateToSummonerByPuuid: props.navigateToSummonerByPuuid,
    loadReplay: props.loadReplay,
    watchReplay: props.watchReplay,
    loadDetails: props.loadDetails,
    dryRunOngoingGame: () => {
      props.dryRunOngoingGame({})
    }
  })
}
