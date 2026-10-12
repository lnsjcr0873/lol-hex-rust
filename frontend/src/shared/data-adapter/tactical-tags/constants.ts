import type { OngoingGamePanelPlayerCardTagSettings } from './types'

export const DEFAULT_TAG_SETTINGS: OngoingGamePanelPlayerCardTagSettings = {
  showPremadeTeamTag: true,
  showSuspiciousFlashPositionTag: true,
  showWinningStreakTag: true,
  showLosingStreakTag: true,
  showSoloKillsTag: true,
  showEasyGankTag: true,
  showGreatPerformanceTag: true,
  showAverageTeamDamageTag: false,
  showAverageTeamDamageTakenTag: false,
  showAverageTeamGoldTag: false,
  showAverageCsPerMinuteTag: false,
  showAverageDamageGoldEfficiencyTag: false,
  showAverageEnemyMissingPingsTag: false,
  showAverageVisionScoreTag: false,
  showAverageKillDamageEfficiencyTag: true,
  showSelfTag: true,
  showMetTag: true,
  showTaggedTag: true,
  showWinRateTeamTag: true,
  showPrivacyTag: true,
  showAkariScoreTag: false
}

export const SUMMONER_SPELL_SMITE_ID = 11

export const PREMADE_TEAM_TAG_CLASSES: Record<string, string> = {
  A: 'bg-[#0f6f68] text-white dark:bg-[#48e5db] dark:text-black',
  B: 'bg-[#1f3fa6] text-white dark:bg-[#628aff] dark:text-black',
  C: 'bg-[#5c6000] text-white dark:bg-[#d4de17] dark:text-black',
  D: 'bg-[#1a7a2a] text-white dark:bg-[#2eda3e] dark:text-black',
  E: 'bg-[#8a4400] text-white dark:bg-[#ff9f1c] dark:text-black',
  F: 'bg-[#8a2a00] text-white dark:bg-[#da4e2e] dark:text-white',
  G: 'bg-[#6a0d6a] text-white dark:bg-[#bc2ebc] dark:text-white',
  H: 'bg-[#a2133f] text-white dark:bg-[#fa4e80] dark:text-black',
  I: 'bg-[#0b3d91] text-white dark:bg-[#0b3d91] dark:text-white',
  J: 'bg-[#7f0000] text-white dark:bg-[#7f0000] dark:text-white',
  K: 'bg-[#5a2a0b] text-white dark:bg-[#8b4513] dark:text-white',
  L: 'bg-[#333333] text-white dark:bg-[#555555] dark:text-white'
}

export const EASY_GANK_TAG_CLASSES = {
  'hard-gank': 'bg-[#24606d] text-white',
  'easy-gank': 'bg-[#8f541e] text-white',
  'very-easy-gank': 'bg-[#a81919] text-white'
}

export const KILL_DAMAGE_EFFICIENCY_LOW_THRESHOLD = 0.65
export const KILL_DAMAGE_EFFICIENCY_HIGH_THRESHOLD = 1.35
