import {
  scoreExpectedContribution,
  scoreKda,
  scoreWinRate,
  scoreCsPerMinute,
  scoreParticipation,
  scoreHealing
} from './analysis/player/scoring'
import {
  AKARI_DAMAGE_WEIGHT,
  AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_WIN_RATE_WEIGHT,
  AKARI_KDA_MAX_SCORE,
  AKARI_MAX_SCORE
} from './analysis/player/constants'
import {
  TeamUpGraph,
  calculateTogetherTimes,
  UnionFind,
  combinations,
  removeSubsets,
  mergeOverlappingSets
} from '../utils/team-up-calc'
import {
  evaluateTacticalTags,
  TACTICAL_TAG_DEFINITIONS
} from './tactical-tags'
import type { TacticalTagContext } from './tactical-tags/types'

console.log('--- TEST 1: Akari 9-Dimension Scoring Logic ---')
console.log('AKARI_MAX_SCORE:', AKARI_MAX_SCORE, '(expected: 17)')
if (AKARI_MAX_SCORE !== 17) throw new Error('AKARI_MAX_SCORE is not 17')

const kdaScore1 = scoreKda(2.0)
const kdaScore2 = scoreKda(7.444)
const kdaScore3 = scoreKda(4.0)
console.log('KDA score at 2.0:', kdaScore1, '(expected: 0)')
console.log('KDA score at 7.444:', kdaScore2, '(expected: ~1.0)')
console.log('KDA score at 4.0:', kdaScore3)

if (Math.abs(kdaScore1 - 0) > 0.001) throw new Error('kdaScore1 mismatch')
if (Math.abs(kdaScore2 - 1.0) > 0.05) throw new Error('kdaScore2 mismatch')

// Damage score: ratio = 1.5 out of 2.0 full score ratio, weight = 3
const dmgScore = scoreExpectedContribution(
  1.5,
  AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_DAMAGE_WEIGHT
)
console.log('Damage score (1.5 ratio):', dmgScore, '(expected: 1.5 out of 3.0)')
if (Math.abs(dmgScore - 1.5) > 0.001) throw new Error('dmgScore mismatch')

const winScore = scoreWinRate(0.75)
console.log('Win rate score (75%):', winScore, '(expected: 0.5)')
if (Math.abs(winScore - 0.5) > 0.001) throw new Error('winScore mismatch')

console.log('--- TEST 2: TeamUpCalc & UnionFind Premade Detection ---')
const matches = [
  { id: '1', players: ['P1', 'P2', 'P3', 'P4', 'P5'] },
  { id: '2', players: ['P1', 'P2', 'P3', 'P6', 'P7'] },
  { id: '3', players: ['P1', 'P2', 'P3', 'P8', 'P9'] },
  { id: '4', players: ['P1', 'P2', 'P10', 'P11', 'P12'] }
]
const players = ['P1', 'P2', 'P3', 'P4']
const togetherTimes = calculateTogetherTimes(matches, players, 3)
console.log('Together times with threshold 3:', togetherTimes)
// P1, P2, P3 should have 3 games together (1, 2, 3)
const trio = togetherTimes.find(t => t.players.length === 3 && t.players.includes('P1') && t.players.includes('P2') && t.players.includes('P3'))
if (!trio || trio.times !== 3) {
  throw new Error(`Expected trio P1, P2, P3 to have 3 games together, found: ${JSON.stringify(trio)}`)
}
console.log('TeamUpGraph detected trio successfully!')

const uf = new UnionFind(['A', 'B', 'C', 'D'])
uf.union('A', 'B')
uf.union('C', 'D')
console.log('UF A and B connected:', uf.find('A') === uf.find('B'))
console.log('UF A and C connected:', uf.find('A') === uf.find('C'))
if (uf.find('A') !== uf.find('B')) throw new Error('UF union A, B failed')
if (uf.find('A') === uf.find('C')) throw new Error('UF disjoint check failed')

console.log('--- TEST 3: 21 Tactical Tags Evaluation ---')
if (TACTICAL_TAG_DEFINITIONS.length !== 21) {
  throw new Error(`Expected exactly 21 tactical tag definitions, got ${TACTICAL_TAG_DEFINITIONS.length}`)
}
console.log(`Verified 21 tactical tag definitions registered: ${TACTICAL_TAG_DEFINITIONS.map(t => t.id).join(', ')}`)

const testContext: TacticalTagContext = {
  puuid: 'puuid-1',
  selfPuuid: 'puuid-1',
  premadeTeamId: 'A',
  assignedPosition: 'MIDDLE',
  spells: { spell1Id: 4, spell2Id: 14 },
  summoner: { privacy: 'PUBLIC' },
  savedInfo: {
    tags: [{ tag: '大神', markedBySelf: true }],
    encounteredGames: { total: 5 }
  },
  analysis: {
    count: 20,
    detailsCount: 20,
    winLoss: {
      all: { count: 20, wins: 18, losses: 2, winRate: 0.9, winningStreak: 5, losingStreak: 0 }
    } as any,
    summary: {
      avgKills: 10,
      avgDeaths: 2,
      avgAssists: 8,
      avgKda: 9.0,
      avgCsPerMinute: 8.5,
      avgDamageGoldEfficiency: 1.35,
      avgChampionDamagePercentageOfTeam: 0.32,
      avgDamageTakenPercentageOfTeam: 0.20,
      avgGoldPercentageOfTeam: 0.26,
      avgVisionScore: 28,
      avgSoloKills: 1.8,
      avgEnemyMissingPings: 3.5,
      avgKillDamageEfficiency: 1.45
    } as any,
    details: {
      avgEarlyDeathsWithEnemyJunglerInvolved: 2.5
    } as any,
    spells: {
      flashOnD: 5,
      flashOnF: 15
    } as any,
    akariScore: {
      total: 8.8,
      outstanding: true,
      extraordinary: true
    } as any
  } as any
}

const evaluated = evaluateTacticalTags(testContext)
console.log('Evaluated tags count:', evaluated.length)
console.log('Active tags:\n' + evaluated.map(t => `  - ${t.id}: ${t.label} [${t.tagClass}]`).join('\n'))

const activeTagIds = new Set(evaluated.map(t => t.id))
const expectedTags = [
  'self',
  'tagged',
  'premade-team',
  'high-win-rate',
  'met',
  'winning-streak',
  'great-performance',
  'suspicious-flash-position',
  'easy-gank',
  'solo-kills',
  'average-kill-damage-efficiency'
]

for (const expected of expectedTags) {
  if (!activeTagIds.has(expected)) {
    throw new Error(`Expected tag '${expected}' to be active!`)
  }
}

console.log('--- ALL TESTS PASSED SUCCESSFULLY! ---')
