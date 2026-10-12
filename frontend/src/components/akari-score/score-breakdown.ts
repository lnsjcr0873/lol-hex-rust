import type { AkariScore } from '../../shared/data-adapter/analysis/player'
import {
  AKARI_CS_MAX_SCORE,
  AKARI_DAMAGE_TAKEN_WEIGHT,
  AKARI_DAMAGE_WEIGHT,
  AKARI_GOLD_WEIGHT,
  AKARI_HEALING_MAX_SCORE,
  AKARI_KDA_MAX_SCORE,
  AKARI_PARTICIPATION_WEIGHT,
  AKARI_VISION_MAX_SCORE,
  AKARI_WIN_RATE_WEIGHT
} from '../../shared/data-adapter/analysis/player/constants'

export type AkariScorePartKey =
  | 'kdaScore'
  | 'winRateScore'
  | 'dmgScore'
  | 'dmgTakenScore'
  | 'healingScore'
  | 'csScore'
  | 'goldScore'
  | 'participationScore'
  | 'visionScore'

export type AkariScoreProgressStatus = 'default' | 'info' | 'success' | 'warning' | 'error'

export type AkariScoreBreakdownItem = {
  key: AkariScorePartKey
  label: string
  value: number
  max: number | null
  progressPercentage: number
  progressStatus: AkariScoreProgressStatus
}

export const AKARI_SCORE_PARTS: Array<{
  key: AkariScorePartKey
  label: string
  max: number | null
}> = [
  {
    key: 'kdaScore',
    label: 'KDA',
    max: AKARI_KDA_MAX_SCORE
  },
  {
    key: 'winRateScore',
    label: '胜率',
    max: AKARI_WIN_RATE_WEIGHT
  },
  {
    key: 'dmgScore',
    label: '输出伤害',
    max: AKARI_DAMAGE_WEIGHT
  },
  {
    key: 'dmgTakenScore',
    label: '承受伤害',
    max: AKARI_DAMAGE_TAKEN_WEIGHT
  },
  {
    key: 'healingScore',
    label: '团队治疗',
    max: AKARI_HEALING_MAX_SCORE
  },
  {
    key: 'csScore',
    label: '补兵发育',
    max: AKARI_CS_MAX_SCORE
  },
  {
    key: 'goldScore',
    label: '经济积累',
    max: AKARI_GOLD_WEIGHT
  },
  {
    key: 'participationScore',
    label: '参团率',
    max: AKARI_PARTICIPATION_WEIGHT
  },
  {
    key: 'visionScore',
    label: '视野得分',
    max: AKARI_VISION_MAX_SCORE
  }
]

function clampProgressPercentage(value: number) {
  if (!Number.isFinite(value)) {
    return 0
  }

  return Math.min(Math.max(value, 0), 100)
}

export function getAkariScoreBreakdownItems(score: AkariScore): AkariScoreBreakdownItem[] {
  return AKARI_SCORE_PARTS.map((part) => {
    const value = score[part.key] ?? 0
    const max = part.max
    const hasFixedMax = max !== null

    return {
      ...part,
      value,
      progressPercentage: hasFixedMax && max > 0 ? clampProgressPercentage((value / max) * 100) : 100,
      progressStatus: (hasFixedMax ? 'default' : 'info') as AkariScoreProgressStatus
    }
  })
}
