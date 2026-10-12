import type { AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'

export type MapPreferenceKind = 'top' | 'mid' | 'bot' | 'topMid' | 'midBot' | 'balanced'

export interface MapPreference {
  kind: MapPreferenceKind
  pct: number | null
}

export function resolveMapPreference(stats: AggregatedJungleAnalysis): MapPreference {
  const topShare = stats.avgTopZonePercentage
  const midShare = stats.avgMidZonePercentage
  const botShare = stats.avgBotZonePercentage
  const topMidShare = topShare + midShare
  const midBotShare = midShare + botShare

  if (midBotShare >= 0.7 && topShare <= 0.28) {
    return { kind: 'midBot', pct: Math.round(midBotShare * 100) }
  }

  if (topMidShare >= 0.7 && botShare <= 0.28) {
    return { kind: 'topMid', pct: Math.round(topMidShare * 100) }
  }

  const ranked = [
    { kind: 'top' as const, share: topShare },
    { kind: 'mid' as const, share: midShare },
    { kind: 'bot' as const, share: botShare }
  ].sort((a, b) => b.share - a.share)

  if (ranked[0].share >= 0.4 && ranked[0].share - ranked[1].share >= 0.08) {
    return { kind: ranked[0].kind, pct: Math.round(ranked[0].share * 100) }
  }

  return { kind: 'balanced', pct: null }
}

export function mapPreferenceText(pref: MapPreference, short = false): string {
  switch (pref.kind) {
    case 'top':
      return short ? '主抓上' : '倾向抓上'
    case 'mid':
      return short ? '主抓中' : '倾向抓中'
    case 'bot':
      return short ? '主抓下' : '倾向抓下'
    case 'topMid':
      return short ? '偏上中' : '倾向抓上中'
    case 'midBot':
      return short ? '偏中下' : '倾向抓中下'
    default:
      return short ? '均衡' : '均衡打野'
  }
}

export function mapPreferenceColor(pref: MapPreference): string {
  switch (pref.kind) {
    case 'top':
    case 'topMid':
      return 'text-emerald-500 dark:text-emerald-400'
    case 'mid':
      return 'text-amber-500 dark:text-yellow-400'
    case 'bot':
    case 'midBot':
      return 'text-blue-500 dark:text-blue-400'
    default:
      return 'text-slate-400 dark:text-slate-300'
  }
}

export function topsideTextShort(stats: AggregatedJungleAnalysis): string {
  return mapPreferenceText(resolveMapPreference(stats), true)
}

export function topsideTextWithPct(stats: AggregatedJungleAnalysis): string {
  const pref = resolveMapPreference(stats)
  const base = mapPreferenceText(pref)

  if (pref.pct === null) {
    return base
  }

  return `${base} ${pref.pct}%`
}

export function topsideTextTrigger(stats: AggregatedJungleAnalysis): string {
  const pref = resolveMapPreference(stats)
  const base = mapPreferenceText(pref, true)

  if (pref.pct === null) {
    return base
  }

  return `${base} ${pref.pct}%`
}

export function topsideTextColor(stats: AggregatedJungleAnalysis): string {
  return mapPreferenceColor(resolveMapPreference(stats))
}

export function formatWeightSum(value: number): string {
  const rounded = Math.round(value)
  return rounded > 999 ? '999+' : rounded.toString()
}
