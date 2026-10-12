import type { AggregatedAnalysis } from '../analysis/player'
import type { LcuOrSgpGameSummary } from '../wrapper'

export interface OngoingGamePanelPlayerCardTagSettings {
  showPremadeTeamTag: boolean
  showSuspiciousFlashPositionTag: boolean
  showWinningStreakTag: boolean
  showLosingStreakTag: boolean
  showSoloKillsTag: boolean
  showEasyGankTag: boolean
  showGreatPerformanceTag: boolean
  showAverageTeamDamageTag: boolean
  showAverageTeamDamageTakenTag: boolean
  showAverageTeamGoldTag: boolean
  showAverageCsPerMinuteTag: boolean
  showAverageDamageGoldEfficiencyTag: boolean
  showAverageEnemyMissingPingsTag: boolean
  showAverageVisionScoreTag: boolean
  showAverageKillDamageEfficiencyTag: boolean
  showSelfTag: boolean
  showMetTag: boolean
  showTaggedTag: boolean
  showWinRateTeamTag: boolean
  showPrivacyTag: boolean
  showAkariScoreTag: boolean
}

export interface PlayerTagNote {
  tag: string
  markedBySelf: boolean
  selfPuuid?: string
}

export interface PlayerSavedInfo {
  tags?: PlayerTagNote[]
  encounteredGames?: {
    total: number
    data?: any[]
  }
}

export interface TacticalTagContext {
  puuid: string
  selfPuuid: string | null
  settings?: Partial<OngoingGamePanelPlayerCardTagSettings>
  analysis: AggregatedAnalysis | null
  summoner?: {
    privacy?: string
    gameName?: string
    tagLine?: string
    displayName?: string
  }
  savedInfo?: PlayerSavedInfo | null
  premadeTeamId?: string
  assignedPosition?: string
  spells?: {
    spell1Id: number
    spell2Id: number
  }
  encounteredCount?: number
  cachedGames?: Record<number, LcuOrSgpGameSummary>
}

export interface EvaluatedTag {
  id: string
  label: string
  tagClass: string
  description?: string
  popoverData?: Record<string, any>
  priority: number
}

export interface TacticalTagDefinition {
  id: string
  evaluate: (ctx: TacticalTagContext) => EvaluatedTag | null
}
