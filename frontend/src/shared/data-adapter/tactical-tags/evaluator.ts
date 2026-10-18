import type { AggregatedAnalysis } from '../analysis/player'
import {
  DEFAULT_TAG_SETTINGS,
  EASY_GANK_TAG_CLASSES,
  KILL_DAMAGE_EFFICIENCY_HIGH_THRESHOLD,
  KILL_DAMAGE_EFFICIENCY_LOW_THRESHOLD,
  PREMADE_TEAM_TAG_CLASSES,
  SUMMONER_SPELL_SMITE_ID
} from './constants'
import type {
  EvaluatedTag,
  OngoingGamePanelPlayerCardTagSettings,
  TacticalTagContext,
  TacticalTagDefinition
} from './types'

export function getIsCurrentJungler(
  assignedPosition: string | undefined,
  spells: { spell1Id: number; spell2Id: number } | undefined
): boolean {
  if (assignedPosition?.toUpperCase() === 'JUNGLE') {
    return true
  }
  return (
    spells?.spell1Id === SUMMONER_SPELL_SMITE_ID ||
    spells?.spell2Id === SUMMONER_SPELL_SMITE_ID
  )
}

export function getEasyGankStatus(
  analysis: AggregatedAnalysis,
  isCurrentJungler: boolean
): { kind: 'hard-gank' | 'easy-gank' | 'very-easy-gank'; times: number; count: number } | null {
  if (isCurrentJungler || !analysis.details) {
    return null
  }

  const times = analysis.details.avgEarlyDeathsWithEnemyJunglerInvolved
  if (times === null) {
    return null
  }

  if (times > 2) {
    return { kind: 'very-easy-gank', times, count: analysis.detailsCount }
  }
  if (times >= 1.5) {
    return { kind: 'easy-gank', times, count: analysis.detailsCount }
  }
  if (times < 1) {
    return { kind: 'hard-gank', times, count: analysis.detailsCount }
  }

  return null
}

export function getKillDamageEfficiencyStatus(
  analysis: AggregatedAnalysis
): { kind: 'high' | 'low' | 'normal'; value: number } {
  const kde = analysis.summary?.avgKillDamageEfficiency
  if (!kde || !Number.isFinite(kde) || kde <= 0.05) {
    return { kind: 'normal', value: 1 }
  }
  if (kde > KILL_DAMAGE_EFFICIENCY_HIGH_THRESHOLD) {
    return { kind: 'high', value: kde }
  }
  if (kde < KILL_DAMAGE_EFFICIENCY_LOW_THRESHOLD) {
    return { kind: 'low', value: kde }
  }
  return { kind: 'normal', value: kde }
}

export const TACTICAL_TAG_DEFINITIONS: TacticalTagDefinition[] = [
  // 1. self
  {
    id: 'self',
    evaluate: (ctx) => {
      const show = ctx.settings?.showSelfTag ?? DEFAULT_TAG_SETTINGS.showSelfTag
      if (!show || !ctx.selfPuuid || ctx.puuid !== ctx.selfPuuid) return null
      return {
        id: 'self',
        label: '本人',
        tagClass: 'bg-[#37246c] text-white',
        description: '当前客户端登录玩家',
        priority: 100
      }
    }
  },

  // 2. tagged
  {
    id: 'tagged',
    evaluate: (ctx) => {
      const show = ctx.settings?.showTaggedTag ?? DEFAULT_TAG_SETTINGS.showTaggedTag
      const tags = ctx.savedInfo?.tags
      if (!show || !tags || tags.length === 0) return null
      const firstTag = tags[0]?.tag ?? '已备注'
      return {
        id: 'tagged',
        label: `备注: ${firstTag}`,
        tagClass: 'bg-[#49914d] text-white',
        description: '本地备注玩家',
        popoverData: { tags },
        priority: 95
      }
    }
  },

  // 3. premade-team
  {
    id: 'premade-team',
    evaluate: (ctx) => {
      const show = ctx.settings?.showPremadeTeamTag ?? DEFAULT_TAG_SETTINGS.showPremadeTeamTag
      const teamId = ctx.premadeTeamId
      if (!show || !teamId) return null
      const cls = PREMADE_TEAM_TAG_CLASSES[teamId] ?? 'bg-teal-600 text-white'
      return {
        id: 'premade-team',
        label: `车队 ${teamId}`,
        tagClass: cls,
        description: `预组队车队成员 [${teamId}]`,
        popoverData: { premadeTeamId: teamId },
        priority: 90
      }
    }
  },

  // 4. high-win-rate
  {
    id: 'high-win-rate',
    evaluate: (ctx) => {
      const show = ctx.settings?.showWinRateTeamTag ?? DEFAULT_TAG_SETTINGS.showWinRateTeamTag
      const analysis = ctx.analysis
      if (!show || !analysis) return null
      if (analysis.winLoss.all.count < 16 || analysis.winLoss.all.winRate < 0.85) return null
      return {
        id: 'high-win-rate',
        label: '绝活/胜率队',
        tagClass: 'bg-[#7e2c85] text-white',
        description: `近期 ${analysis.winLoss.all.count} 场胜率 ${(analysis.winLoss.all.winRate * 100).toFixed(1)}%`,
        popoverData: {
          count: analysis.winLoss.all.count,
          winRate: analysis.winLoss.all.winRate,
          wins: analysis.winLoss.all.wins
        },
        priority: 85
      }
    }
  },

  // 5. met
  {
    id: 'met',
    evaluate: (ctx) => {
      const show = ctx.settings?.showMetTag ?? DEFAULT_TAG_SETTINGS.showMetTag
      const count = ctx.encounteredCount ?? ctx.savedInfo?.encounteredGames?.total ?? 0
      if (!show || count <= 0) return null
      return {
        id: 'met',
        label: `曾遇 ${count}次`,
        tagClass: 'bg-[#5cacea] text-black font-semibold',
        description: `历史共同交手过 ${count} 场`,
        popoverData: { count },
        priority: 80
      }
    }
  },

  // 6. privacy
  {
    id: 'privacy',
    evaluate: (ctx) => {
      const show = ctx.settings?.showPrivacyTag ?? DEFAULT_TAG_SETTINGS.showPrivacyTag
      if (!show || ctx.summoner?.privacy !== 'PRIVATE') return null
      return {
        id: 'privacy',
        label: '隐藏战绩',
        tagClass: 'bg-[#870808] text-white',
        description: '该玩家设置为生涯隐私保密',
        priority: 75
      }
    }
  },

  // 7. winning-streak
  {
    id: 'winning-streak',
    evaluate: (ctx) => {
      const show = ctx.settings?.showWinningStreakTag ?? DEFAULT_TAG_SETTINGS.showWinningStreakTag
      const streak = ctx.analysis?.winLoss.all.winningStreak ?? 0
      if (!show || streak < 3) return null
      return {
        id: 'winning-streak',
        label: `${streak}连胜`,
        tagClass: 'bg-[#18571c] text-white',
        description: `当前处于 ${streak} 连胜势头中`,
        popoverData: { streak },
        priority: 70
      }
    }
  },

  // 8. losing-streak
  {
    id: 'losing-streak',
    evaluate: (ctx) => {
      const show = ctx.settings?.showLosingStreakTag ?? DEFAULT_TAG_SETTINGS.showLosingStreakTag
      const streak = ctx.analysis?.winLoss.all.losingStreak ?? 0
      if (!show || streak < 3) return null
      return {
        id: 'losing-streak',
        label: `${streak}连败`,
        tagClass: 'bg-[#893b3b] text-white',
        description: `当前处于 ${streak} 连败低谷中`,
        popoverData: { streak },
        priority: 68
      }
    }
  },

  // 9. great-performance
  {
    id: 'great-performance',
    evaluate: (ctx) => {
      const show = ctx.settings?.showGreatPerformanceTag ?? DEFAULT_TAG_SETTINGS.showGreatPerformanceTag
      const score = ctx.analysis?.akariScore
      if (!show || !score) return null
      if (!score.outstanding && !score.extraordinary) return null
      const isExtra = Boolean(score.extraordinary)
      return {
        id: 'great-performance',
        label: isExtra ? '通天代' : '表现优异',
        tagClass: 'bg-[#b81b86] text-white font-bold',
        description: `Akari 评分表现极佳: ${score.total.toFixed(2)}`,
        popoverData: { score: score.total, extraordinary: isExtra },
        priority: 65
      }
    }
  },

  // 10. suspicious-flash-position
  {
    id: 'suspicious-flash-position',
    evaluate: (ctx) => {
      const show = ctx.settings?.showSuspiciousFlashPositionTag ?? DEFAULT_TAG_SETTINGS.showSuspiciousFlashPositionTag
      const spells = ctx.analysis?.spells
      if (!show || !spells) return null
      if (spells.flashOnD <= 0 || spells.flashOnF <= 0) return null
      return {
        id: 'suspicious-flash-position',
        label: '闪现异位',
        tagClass: 'bg-[#c2410c] text-white',
        description: `近期对局中同时存在 D 闪 (${spells.flashOnD}场) 与 F 闪 (${spells.flashOnF}场)，可能换人或代练`,
        popoverData: { flashOnD: spells.flashOnD, flashOnF: spells.flashOnF },
        priority: 60
      }
    }
  },

  // 11. easy-gank
  {
    id: 'easy-gank',
    evaluate: (ctx) => {
      const show = ctx.settings?.showEasyGankTag ?? DEFAULT_TAG_SETTINGS.showEasyGankTag
      if (!show || !ctx.analysis) return null
      const isJungler = getIsCurrentJungler(ctx.assignedPosition, ctx.spells)
      const status = getEasyGankStatus(ctx.analysis, isJungler)
      if (!status) return null

      const labels = {
        'very-easy-gank': '极易被抓',
        'easy-gank': '容易被抓',
        'hard-gank': '难被抓'
      }
      return {
        id: 'easy-gank',
        label: labels[status.kind],
        tagClass: EASY_GANK_TAG_CLASSES[status.kind],
        description: `前15分钟平均被敌方打野参与击杀: ${status.times.toFixed(2)} 次/场`,
        popoverData: status,
        priority: 55
      }
    }
  },

  // 12. solo-kills
  {
    id: 'solo-kills',
    evaluate: (ctx) => {
      const show = ctx.settings?.showSoloKillsTag ?? DEFAULT_TAG_SETTINGS.showSoloKillsTag
      const solo = ctx.analysis?.summary.avgSoloKills
      if (!show || solo === null || solo === undefined || solo < 1.0) return null
      return {
        id: 'solo-kills',
        label: `单杀怪 ${solo.toFixed(1)}`,
        tagClass: 'bg-[#9019a8] text-white',
        description: `场均单杀次数达 ${solo.toFixed(2)} 次`,
        popoverData: { avgSoloKills: solo },
        priority: 50
      }
    }
  },

  // 13. average-team-damage
  {
    id: 'average-team-damage',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageTeamDamageTag ?? DEFAULT_TAG_SETTINGS.showAverageTeamDamageTag
      const rate = ctx.analysis?.summary.avgChampionDamagePercentageOfTeam
      if (!show || rate === undefined || rate < 0.25) return null
      return {
        id: 'average-team-damage',
        label: `输出 ${(rate * 100).toFixed(0)}%`,
        tagClass: 'bg-[#692723] text-white',
        description: `输出占全队比例: ${(rate * 100).toFixed(1)}%`,
        popoverData: { rate },
        priority: 45
      }
    }
  },

  // 14. average-team-damage-taken
  {
    id: 'average-team-damage-taken',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageTeamDamageTakenTag ?? DEFAULT_TAG_SETTINGS.showAverageTeamDamageTakenTag
      const rate = ctx.analysis?.summary.avgDamageTakenPercentageOfTeam
      if (!show || rate === undefined || rate < 0.25) return null
      return {
        id: 'average-team-damage-taken',
        label: `承伤 ${(rate * 100).toFixed(0)}%`,
        tagClass: 'bg-[#135225] text-white',
        description: `承伤占全队比例: ${(rate * 100).toFixed(1)}%`,
        popoverData: { rate },
        priority: 40
      }
    }
  },

  // 15. average-team-gold
  {
    id: 'average-team-gold',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageTeamGoldTag ?? DEFAULT_TAG_SETTINGS.showAverageTeamGoldTag
      const rate = ctx.analysis?.summary.avgGoldPercentageOfTeam
      if (!show || rate === undefined || rate < 0.23) return null
      return {
        id: 'average-team-gold',
        label: `经济 ${(rate * 100).toFixed(0)}%`,
        tagClass: 'bg-[#a73d2a] text-white',
        description: `经济占全队比例: ${(rate * 100).toFixed(1)}%`,
        popoverData: { rate },
        priority: 35
      }
    }
  },

  // 16. average-cs-per-minute
  {
    id: 'average-cs-per-minute',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageCsPerMinuteTag ?? DEFAULT_TAG_SETTINGS.showAverageCsPerMinuteTag
      const cs = ctx.analysis?.summary.avgCsPerMinute
      if (!show || cs === undefined || cs < 7.0) return null
      return {
        id: 'average-cs-per-minute',
        label: `补刀 ${cs.toFixed(1)}/分`,
        tagClass: 'bg-[#5a4a1f] text-white',
        description: `分均补兵达 ${cs.toFixed(2)} 刀`,
        popoverData: { avgCsPerMinute: cs },
        priority: 30
      }
    }
  },

  // 17. average-damage-gold-efficiency
  {
    id: 'average-damage-gold-efficiency',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageDamageGoldEfficiencyTag ?? DEFAULT_TAG_SETTINGS.showAverageDamageGoldEfficiencyTag
      const eff = ctx.analysis?.summary.avgDamageGoldEfficiency
      if (!show || eff === undefined || eff < 1.2) return null
      return {
        id: 'average-damage-gold-efficiency',
        label: `伤金转化 ${(eff * 100).toFixed(0)}%`,
        tagClass: 'bg-[#8f411e] text-white',
        description: `伤金转化率达 ${(eff * 100).toFixed(1)}% (吃草挤奶)`,
        popoverData: { avgDamageGoldEfficiency: eff },
        priority: 25
      }
    }
  },

  // 18. average-enemy-missing-pings
  {
    id: 'average-enemy-missing-pings',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageEnemyMissingPingsTag ?? DEFAULT_TAG_SETTINGS.showAverageEnemyMissingPingsTag
      const pings = ctx.analysis?.summary.avgEnemyMissingPings
      if (!show || pings === null || pings === undefined || pings < 2.0) return null
      return {
        id: 'average-enemy-missing-pings',
        label: `问号标记 ${pings.toFixed(1)}`,
        tagClass: 'bg-[#e7da30] text-black font-semibold',
        description: `场均发送敌人缺失问号信号: ${pings.toFixed(2)} 次`,
        popoverData: { avgEnemyMissingPings: pings },
        priority: 20
      }
    }
  },

  // 19. average-vision-score
  {
    id: 'average-vision-score',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageVisionScoreTag ?? DEFAULT_TAG_SETTINGS.showAverageVisionScoreTag
      const vis = ctx.analysis?.summary.avgVisionScore
      if (!show || vis === undefined || vis < 25) return null
      return {
        id: 'average-vision-score',
        label: `视野控 ${vis.toFixed(0)}`,
        tagClass: 'bg-[#2451a6] text-white',
        description: `场均视野得分达 ${vis.toFixed(1)} 分`,
        popoverData: { avgVisionScore: vis },
        priority: 15
      }
    }
  },

  // 20. average-kill-damage-efficiency
  {
    id: 'average-kill-damage-efficiency',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAverageKillDamageEfficiencyTag ?? DEFAULT_TAG_SETTINGS.showAverageKillDamageEfficiencyTag
      if (!show || !ctx.analysis) return null
      const tag = getKillDamageEfficiencyStatus(ctx.analysis)
      if (tag.kind === 'normal') return null
      const isHigh = tag.kind === 'high'
      return {
        id: 'average-kill-damage-efficiency',
        label: isHigh ? '人头收割' : '虚空伤害',
        tagClass: 'bg-[#04614b] text-white',
        description: isHigh
          ? `收割怪: 人头击杀转化比高 (${(tag.value * 100).toFixed(1)}%)`
          : `虚空伤害: 伤害充足但击杀收割偏低 (${(tag.value * 100).toFixed(1)}%)`,
        popoverData: { kind: tag.kind, value: tag.value },
        priority: 10
      }
    }
  },

  // 21. akari-score
  {
    id: 'akari-score',
    evaluate: (ctx) => {
      const show = ctx.settings?.showAkariScoreTag ?? DEFAULT_TAG_SETTINGS.showAkariScoreTag
      const score = ctx.analysis?.akariScore?.total
      if (!show || score === undefined || score <= 0) return null
      return {
        id: 'akari-score',
        label: `Akari ${score.toFixed(2)}`,
        tagClass: 'bg-[#b81b86] text-white',
        description: `Akari 综合评分: ${score.toFixed(2)} / 17.0`,
        popoverData: { akariScore: score },
        priority: 5
      }
    }
  }
]

export function evaluateTacticalTags(
  ctx: TacticalTagContext,
  customSettings?: Partial<OngoingGamePanelPlayerCardTagSettings>
): EvaluatedTag[] {
  const mergedCtx: TacticalTagContext = {
    ...ctx,
    settings: {
      ...DEFAULT_TAG_SETTINGS,
      ...ctx.settings,
      ...customSettings
    }
  }

  const result: EvaluatedTag[] = []
  for (const def of TACTICAL_TAG_DEFINITIONS) {
    const evaluated = def.evaluate(mergedCtx)
    if (evaluated) {
      result.push(evaluated)
    }
  }

  // Sort tags by priority descending
  return result.sort((a, b) => b.priority - a.priority)
}
