import { FIXED_CARD_WIDTH_PX_NUMBER } from '../constants'

/**
 * Calculates adaptive column count (bounded between 3 and 8) based on container width and team size.
 * Breakpoints: 240 * (col + 0.25)
 *  - col 8: 1980px
 *  - col 7: 1740px
 *  - col 6: 1500px
 *  - col 5: 1260px
 *  - col 4: 1020px
 *  - col 3: 780px
 */
export function calculateColumnsNeed(contentWidth: number, teamSize: number = 5): number {
  const maxTeamMembers = Math.max(teamSize, 5)
  const maxAllowed = [8, 7, 6, 5, 4, 3].find(
    (col) => contentWidth > FIXED_CARD_WIDTH_PX_NUMBER * (col + 0.25)
  )
  return Math.min(maxAllowed || 3, maxTeamMembers)
}

/**
 * Calculates display lines/rows per team based on columns and team member count.
 */
export function calculateLinesPerTeam(columns: number, teamSize: number = 5): number {
  const maxMembers = Math.max(teamSize, 5)
  if (!columns || columns <= 0) return 1
  return Math.ceil(maxMembers / columns)
}
