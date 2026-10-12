import { computed } from 'vue'

export function useWinResultStyleType(winResult?: string) {
  if (winResult === 'win') return 'win'
  if (winResult === 'loss') return 'loss'
  return 'neutral'
}

export function useCardBorderClass(winResult?: string) {
  if (winResult === 'win') {
    return 'border-blue-600/60 dark:border-blue-500/60 shadow-[0_0_12px_rgba(59,130,246,0.12)]'
  }
  if (winResult === 'loss') {
    return 'border-red-600/60 dark:border-red-500/60 shadow-[0_0_12px_rgba(239,68,68,0.12)]'
  }
  return 'border-neutral-600/40 dark:border-neutral-500/40'
}

export function getTeamColor(teamIdentifier?: string) {
  if (teamIdentifier === '100' || teamIdentifier === 'blue') {
    return '#3b82f6'
  }
  if (teamIdentifier === '200' || teamIdentifier === 'red') {
    return '#ef4444'
  }
  return '#10b981'
}

export const playerColors = [
  '#3b82f6', // 蓝方 1 (上)
  '#06b6d4', // 蓝方 2 (野)
  '#6366f1', // 蓝方 3 (中)
  '#38bdf8', // 蓝方 4 (下)
  '#818cf8', // 蓝方 5 (辅)
  '#ef4444', // 红方 1 (上)
  '#f97316', // 红方 2 (野)
  '#ec4899', // 红方 3 (中)
  '#f43f5e', // 红方 4 (下)
  '#fb7185'  // 红方 5 (辅)
]
