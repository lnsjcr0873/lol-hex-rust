import type { PreparedGame } from '../types/helpers'

export interface AggregatedPositionAnalysis {
  TOP: number
  JUNGLE: number
  MIDDLE: number
  BOTTOM: number
  UTILITY: number
}

export function computeAggregatedPositions(
  games: PreparedGame[]
): AggregatedPositionAnalysis | null {
  if (!games.length) {
    return null
  }

  const positions: AggregatedPositionAnalysis = {
    TOP: 0,
    JUNGLE: 0,
    MIDDLE: 0,
    BOTTOM: 0,
    UTILITY: 0
  }

  let validCount = 0

  for (const { participant } of games) {
    if (participant.position && participant.position in positions) {
      positions[participant.position as keyof AggregatedPositionAnalysis] += 1
      validCount += 1
    }
  }

  return validCount > 0 ? positions : null
}
