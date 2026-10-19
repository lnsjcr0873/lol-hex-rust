export interface ParsedRole {
  current: string
  assignmentReason: string
  primary: string
  secondary: string
  fill: string
}

// assignmentReason: PRIMARY, SECONDARY, FILL_SECONDARY, FILL_PRIMARY, AUTOFILL
// NONE, UNSELECTED
// TOP, MIDDLE, JUNGLE, BOTTOM, UTILITY
export function parseSelectedRole(role: string | null): ParsedRole {
  if (!role) {
    return {
      current: 'NONE',
      assignmentReason: 'NONE',
      primary: 'NONE',
      secondary: 'NONE',
      fill: 'NONE'
    }
  }

  const segments = role.split('.')
  if (segments.length !== 4 && segments.length !== 5) {
    return {
      current: 'NONE',
      assignmentReason: 'NONE',
      primary: 'NONE',
      secondary: 'NONE',
      fill: 'NONE'
    }
  }

  const [p1, p2, p3, p4, p5] = segments

  return {
    current: p1,
    assignmentReason: p2,
    primary: p3,
    secondary: p4,
    fill: p5 || 'NONE'
  }
}

export const TIER_NAMES: Record<string, string> = {
  IRON: '黑铁',
  BRONZE: '青铜',
  SILVER: '白银',
  GOLD: '黄金',
  PLATINUM: '铂金',
  EMERALD: '翡翠',
  DIAMOND: '钻石',
  MASTER: '大师',
  GRANDMASTER: '宗师',
  CHALLENGER: '王者'
}

export function isUnrankedTier(tier?: string | null): boolean {
  if (!tier) return true
  const upper = tier.trim().toUpperCase()
  return upper === 'UNRANKED' || upper === 'NONE' || upper === 'NA' || upper === ''
}

export function formatTierName(tier?: string | null): string {
  if (!tier || isUnrankedTier(tier)) return '未定级'
  const trimmed = tier.trim().toUpperCase()
  return TIER_NAMES[trimmed] || tier.trim()
}

export function isMasterPlus(tier?: string | null): boolean {
  if (!tier) return false
  const upper = tier.trim().toUpperCase()
  return upper === 'MASTER' || upper === 'GRANDMASTER' || upper === 'CHALLENGER'
}

export function formatTier(tier?: string | null, div?: string | null): string {
  if (!tier || isUnrankedTier(tier)) return '未定级'
  const translated = formatTierName(tier)
  if (!div || div === 'NA' || div === 'NONE' || div.trim() === '' || isMasterPlus(tier)) {
    return translated
  }
  // Sanitize div: if div already contains tier name (e.g. "白银 I" or "SILVER I"), remove the tier prefix
  const cleanDiv = div.replace(translated, '').replace(new RegExp(`^${tier}\\s*`, 'i'), '').trim()
  return cleanDiv ? `${translated} ${cleanDiv}` : translated
}

