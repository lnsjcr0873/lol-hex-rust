export function getTeamIndicatorColorClass(team: string) {
  switch (team) {
    case 'TEAM-100':
    case 'blue':
    case '100':
      return 'bg-[#40c1ff]'

    case 'TEAM-200':
    case 'red':
    case '200':
      return 'bg-[#ff3333]'

    case 'LOBBY':
      return 'bg-neutral-500 dark:bg-neutral-200'

    default:
      return null
  }
}

export const WIN_RATE_TEAM_TAG_BG_CLASSES = 'bg-[#7e2c85]'
export const LOSS_RATE_TEAM_TAG_BG_CLASSES = 'bg-[#893b3b]'
export const WIN_RATE_GTE_50_TEXT_CLASSES = 'text-emerald-400 font-bold'
export const WIN_RATE_LT_50_TEXT_CLASSES = 'text-red-400 font-bold'
