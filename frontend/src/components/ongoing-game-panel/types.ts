import type { PlayerCard } from '../../api/types'
import type { AggregatedAnalysis, AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'
import type { ChampionAnalysisMastery } from '../champion-analysis/types'
import type { PlayerSavedInfo } from '../../shared/data-adapter/tactical-tags'

export interface EnhancedPlayerCard extends PlayerCard {
  analysis?: AggregatedAnalysis | null
  jungleAnalysis?: AggregatedJungleAnalysis | null
  masteries?: ChampionAnalysisMastery[]
  savedInfo?: PlayerSavedInfo | null
  premadeTeamId?: string // 'A', 'B', 'C', ...
}

export interface PremadeGroup {
  id: string
  label: string
  puuids: string[]
  times?: number
  winRateTeamInfo?: {
    type: 'win-rate-team' | 'loss-rate-team'
  }
}

export interface OngoingTeamData {
  teamId: 'blue' | 'red'
  label: string
  players: EnhancedPlayerCard[]
  avgWinRate: number
  avgKda: number
  totalKills: number
  totalDeaths: number
  totalAssists: number
  totalGames: number
  premadeGroups: PremadeGroup[]
}
