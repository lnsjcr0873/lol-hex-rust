export function getCherryWinningTeamCount(teamCount: number) {
  if (teamCount <= 0) return 0
  return Math.max(1, Math.floor(teamCount / 2))
}

export function getCherryTeamCount(
  participants: {
    playerSubteamId?: number
    subteamPlacement?: number
    teamIdentifier?: string
  }[]
) {
  if (!participants || !Array.isArray(participants)) {
    return 0
  }

  const teamIds = new Set<number | string>()

  for (const participant of participants) {
    if (!participant) continue

    if (participant.subteamPlacement !== undefined && participant.subteamPlacement <= 0) {
      continue
    }

    if (participant.playerSubteamId && participant.playerSubteamId > 0) {
      teamIds.add(participant.playerSubteamId)
      continue
    }

    if (typeof participant.teamIdentifier === 'string' && participant.teamIdentifier.startsWith('CHERRY-')) {
      teamIds.add(participant.teamIdentifier)
    }
  }

  return teamIds.size
}

export function isCherryPlacementWin(placement: number, teamCount: number) {
  const winningCount = getCherryWinningTeamCount(teamCount)
  return placement > 0 && winningCount > 0 && placement <= winningCount
}
