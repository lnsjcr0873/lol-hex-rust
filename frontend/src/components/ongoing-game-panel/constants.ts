import BronzeMedal from '../../assets/ranked-icons/bronze.png'
import ChallengerMedal from '../../assets/ranked-icons/challenger.png'
import DiamondMedal from '../../assets/ranked-icons/diamond.png'
import EmeraldMedal from '../../assets/ranked-icons/emerald.png'
import GoldMedal from '../../assets/ranked-icons/gold.png'
import GrandmasterMedal from '../../assets/ranked-icons/grandmaster.png'
import IronMedal from '../../assets/ranked-icons/iron.png'
import MasterMedal from '../../assets/ranked-icons/master.png'
import PlatinumMedal from '../../assets/ranked-icons/platinum.png'
import SilverMedal from '../../assets/ranked-icons/silver.png'

export const IQR_THRESHOLD = 0.65

export const RANKED_MEDAL_MAP: Record<string, string> = {
  IRON: IronMedal,
  BRONZE: BronzeMedal,
  SILVER: SilverMedal,
  GOLD: GoldMedal,
  PLATINUM: PlatinumMedal,
  EMERALD: EmeraldMedal,
  DIAMOND: DiamondMedal,
  MASTER: MasterMedal,
  GRANDMASTER: GrandmasterMedal,
  CHALLENGER: ChallengerMedal
}

export const PREMADE_TEAMS = [
  'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L'
]

export const PREMADE_TEAM_COLORS: Record<string, { foregroundColor: string; color: string; borderColor: string }> = {
  A: { foregroundColor: '#48e5db', color: '#000', borderColor: '#48e5dbd0' },
  B: { foregroundColor: '#628aff', color: '#000', borderColor: '#628affd0' },
  C: { foregroundColor: '#d4de17', color: '#000', borderColor: '#d4de17d0' },
  D: { foregroundColor: '#2eda3e', color: '#000', borderColor: '#2eda3ed0' },
  E: { foregroundColor: '#ff9f1c', color: '#000', borderColor: '#ff9f1cd0' },
  F: { foregroundColor: '#da4e2e', color: '#fff', borderColor: '#da4e2ed0' },
  G: { foregroundColor: '#bc2ebc', color: '#fff', borderColor: '#bc2ebcd0' },
  H: { foregroundColor: '#fa4e80', color: '#000', borderColor: '#fa4e80d0' },
  I: { foregroundColor: '#0b3d91', color: '#fff', borderColor: '#0b3d91d0' },
  J: { foregroundColor: '#7f0000', color: '#fff', borderColor: '#7f0000d0' },
  K: { foregroundColor: '#8b4513', color: '#fff', borderColor: '#8b4513d0' },
  L: { foregroundColor: '#555555', color: '#fff', borderColor: '#555555d0' }
}

export const FIXED_CARD_WIDTH_PX_LITERAL = '240px'
export const FIXED_CARD_WIDTH_PX_NUMBER = 240

export const POSITION_ORDER: Record<string, number> = {
  NONE: 0,
  TOP: 1,
  JUNGLE: 2,
  JUG: 2,
  MIDDLE: 3,
  MID: 3,
  BOTTOM: 4,
  BOT: 4,
  ADC: 4,
  UTILITY: 5,
  SUP: 5,
  SUPPORT: 5
}

export const POSITION_DISPLAY_NAMES: Record<string, string> = {
  TOP: '上单',
  JUNGLE: '打野',
  JUG: '打野',
  MIDDLE: '中单',
  MID: '中单',
  BOTTOM: '下路',
  BOT: '下路',
  ADC: '射手',
  UTILITY: '辅助',
  SUP: '辅助',
  SUPPORT: '辅助',
  NONE: '未知'
}

export const WIN_RATE_TEAM_MIN_MATCHES = 13
export const WIN_RATE_TEAM_MIN_SIZE = 3
export const LOSS_RATE_TEAM_MIN_SIZE = 2
export const WIN_RATE_TEAM_MIN_WIN_RATE = 0.9
export const LOSS_RATE_TEAM_MAX_WIN_RATE = 0.25

export const STARRED_CHAMPION_LEVEL = 60
