import { TimelineFrame } from '@shared/types/league-client/match-history'
import {
  DetailedChampionKillEvent,
  DetailedParticipantFrame,
  DetailedTimelineFrame
} from '@shared/types/sgp/match-history'

import { LcuOrSgpGameDetails } from '../wrapper'

export function isSgpChampionKillEvent(event: any): event is DetailedChampionKillEvent {
  if (!event || typeof event !== 'object') return false

  return (
    event.type === 'CHAMPION_KILL' &&
    (Array.isArray(event.victimDamageDealt) || Array.isArray(event.victimDamageReceived))
  )
}

export function isSgpDetailedTimelineFrame(frame: any): frame is DetailedTimelineFrame {
  if (!frame || typeof frame !== 'object') return false

  const anyFrame = frame?.participantFrames?.[1]

  return Boolean(anyFrame && 'damageStats' in anyFrame && 'championStats' in anyFrame)
}

export function isSgpDetailedParticipantFrame(frame: any): frame is DetailedParticipantFrame {
  if (!frame || typeof frame !== 'object') return false

  return 'damageStats' in frame && 'championStats' in frame
}

export function toFrames(details: LcuOrSgpGameDetails): (DetailedTimelineFrame | TimelineFrame)[] {
  if (!details || !details.data) return []
  const { source, data } = details

  if (source === 'sgp') {
    const sgpFrames = data?.json?.frames
    return Array.isArray(sgpFrames) ? sgpFrames : []
  }

  const rawData = data as any
  if (Array.isArray(rawData?.frames)) {
    return rawData.frames
  }
  if (Array.isArray(rawData?.timeline?.frames)) {
    return rawData.timeline.frames
  }
  if (Array.isArray(rawData)) {
    return rawData
  }

  return []
}
