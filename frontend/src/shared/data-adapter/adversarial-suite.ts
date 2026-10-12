import {
  scoreExpectedContribution,
  scoreKda,
  scoreWinRate,
  scoreCsPerMinute,
  scoreParticipation,
  scoreHealing
} from './analysis/player/scoring'
import {
  AKARI_CS_FULL_SCORE_PER_MINUTE,
  AKARI_CS_MAX_SCORE,
  AKARI_CS_MIN_SCORE_PER_MINUTE,
  AKARI_DAMAGE_TAKEN_WEIGHT,
  AKARI_DAMAGE_WEIGHT,
  AKARI_EXPECTED_CONTRIBUTION_BASELINE_RATIO,
  AKARI_GOLD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_GOLD_WEIGHT,
  AKARI_HEALING_FULL_SCORE_TEAM_AVERAGE_DAMAGE_TAKEN_RATIO,
  AKARI_HEALING_MAX_SCORE,
  AKARI_HEALING_MIN_TEAM_AVERAGE_DAMAGE_TAKEN_RATIO,
  AKARI_HEALING_SOLO_FULL_SCORE_TEAM_AVERAGE_DAMAGE_TAKEN_RATIO,
  AKARI_KDA_BASELINE,
  AKARI_KDA_MAX_SCORE,
  AKARI_KDA_WEIGHT,
  AKARI_MAX_SCORE,
  AKARI_PARTICIPATION_MIN_SHARE,
  AKARI_PARTICIPATION_WEIGHT,
  AKARI_STANDARD_EXPECTED_CONTRIBUTION_FULL_SCORE_RATIO,
  AKARI_VISION_MAX_SCORE,
  AKARI_WIN_RATE_BASELINE,
  AKARI_WIN_RATE_WEIGHT
} from './analysis/player/constants'
import { computeSingleAkariScore } from './analysis/player/single/akari'
import { computeAggregatedAkariScore } from './analysis/player/aggregate/akari'
import type { SingleAnalysis } from './analysis/player/types/single'
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
  TACTICAL_TAG_DEFINITIONS,
  getEasyGankStatus,
  getIsCurrentJungler,
  getKillDamageEfficiencyStatus
} from './tactical-tags'
import {
  DEFAULT_TAG_SETTINGS,
  KILL_DAMAGE_EFFICIENCY_HIGH_THRESHOLD,
  KILL_DAMAGE_EFFICIENCY_LOW_THRESHOLD,
  SUMMONER_SPELL_SMITE_ID
} from './tactical-tags/constants'
import type { TacticalTagContext } from './tactical-tags/types'
import { computePlayerTags } from '../../components/match-card/utils/tags'
import type { MatchParticipant } from './match-history/participants'
import type { TeamsAdapterResult } from './match-history/teams'
import type { MatchBasicInfo } from './match-history/match-basic'

let passCount = 0
let failCount = 0

function assert(condition: boolean, msg: string) {
  if (!condition) {
    failCount++
    console.error(`❌ ASSERTION FAILED: ${msg}`)
    throw new Error(`Assertion failed: ${msg}`)
  } else {
    passCount++
  }
}

function assertClose(actual: number, expected: number, eps = 0.0001, msg = '') {
  const diff = Math.abs(actual - expected)
  if (diff > eps) {
    failCount++
    console.error(`❌ NUMERICAL ASSERTION FAILED: ${msg} (actual: ${actual}, expected: ${expected}, diff: ${diff})`)
    throw new Error(`Numerical assertion failed: ${msg} (actual: ${actual}, expected: ${expected})`)
  } else {
    passCount++
  }
}

console.log('======================================================================')
console.log('🚀 HEXBRAIN ADVERSARIAL ALGORITHM & SCORING VERIFICATION HARNESS 🚀')
console.log('======================================================================\n')

// ======================================================================
// SUITE 1: AKARI 9-DIMENSION SCORING FORMULAS & BOUNDARY VALUES
// ======================================================================
console.log('>>> [SUITE 1] Akari 9-Dimension Scoring Mathematical Verification')

// 1.1 AKARI_MAX_SCORE Constant Invariant
assert(AKARI_MAX_SCORE === 17, `AKARI_MAX_SCORE must equal 17 (got ${AKARI_MAX_SCORE})`)
assert(
  AKARI_KDA_MAX_SCORE +
    AKARI_WIN_RATE_WEIGHT +
    AKARI_DAMAGE_WEIGHT +
    AKARI_DAMAGE_TAKEN_WEIGHT +
    AKARI_HEALING_MAX_SCORE +
    AKARI_CS_MAX_SCORE +
    AKARI_GOLD_WEIGHT +
    AKARI_PARTICIPATION_WEIGHT +
    AKARI_VISION_MAX_SCORE === 17,
  'Sum of dimension weights must strictly equal 17'
)

// 1.2 scoreKda
console.log('  Testing scoreKda boundary values...')
// At or below baseline (2.0): score must be 0
assertClose(scoreKda(2.0), 0, 0.00001, 'KDA at baseline 2.0')
assertClose(scoreKda(1.5), 0, 0.00001, 'KDA below baseline 1.5')
assertClose(scoreKda(0.0), 0, 0.00001, 'KDA at 0.0')
assertClose(scoreKda(-10.0), 0, 0.00001, 'KDA negative -10')
assertClose(scoreKda(-Infinity), 0, 0.00001, 'KDA negative infinity')

// Max score at 7.4444... (effectiveKda = 5.4444444 = 49/9, sqrt = 7/3, (7/3)*(3/7) = 1.0)
const exactMaxKda = 2 + (7 / 3) ** 2 // 2 + 49/9 = 67/9 ~ 7.444444444444445
assertClose(scoreKda(exactMaxKda), 1.0, 0.00001, 'Exact max score KDA')
// Above max score: capped at 1.0
assertClose(scoreKda(8.0), 1.0, 0.00001, 'KDA above max 8.0')
assertClose(scoreKda(50.0), 1.0, 0.00001, 'Extreme KDA 50.0')
assertClose(scoreKda(Infinity), 1.0, 0.00001, 'KDA +Infinity capped at 1.0')

// Monotonicity check
let prevKdaScore = -1
for (let k = 0; k <= 10; k += 0.25) {
  const s = scoreKda(k)
  assert(!isNaN(s) && isFinite(s), `KDA score at ${k} is NaN or not finite`)
  assert(s >= prevKdaScore, `KDA score at ${k} (${s}) is less than score at ${k - 0.25} (${prevKdaScore})`)
  assert(s >= 0 && s <= 1.0, `KDA score at ${k} out of bounds [0, 1]`)
  prevKdaScore = s
}

// 1.3 scoreWinRate
console.log('  Testing scoreWinRate boundary values...')
assertClose(scoreWinRate(0.5), 0, 0.00001, 'WinRate at baseline 0.5')
assertClose(scoreWinRate(0.4), 0, 0.00001, 'WinRate below baseline 0.4')
assertClose(scoreWinRate(0.0), 0, 0.00001, 'WinRate at 0.0')
assertClose(scoreWinRate(-1.0), 0, 0.00001, 'WinRate negative')
assertClose(scoreWinRate(0.75), 0.5, 0.00001, 'WinRate at 0.75')
assertClose(scoreWinRate(1.0), 1.0, 0.00001, 'WinRate at 1.0')
assertClose(scoreWinRate(1.5), 1.0, 0.00001, 'WinRate > 1.0 capped at 1.0')
assertClose(scoreWinRate(Infinity), 1.0, 0.00001, 'WinRate Infinity capped at 1.0')

// 1.4 scoreExpectedContribution (Damage, Taken, Gold, Vision)
console.log('  Testing scoreExpectedContribution...')
// Damage: baseline=1.0, full=2.0, weight=3.0
assertClose(scoreExpectedContribution(1.0, 2.0, 3.0), 0.0, 0.00001, 'Dmg ratio at 1.0')
assertClose(scoreExpectedContribution(0.5, 2.0, 3.0), 0.0, 0.00001, 'Dmg ratio below 1.0')
assertClose(scoreExpectedContribution(0.0, 2.0, 3.0), 0.0, 0.00001, 'Dmg ratio at 0.0')
assertClose(scoreExpectedContribution(-5.0, 2.0, 3.0), 0.0, 0.00001, 'Dmg ratio negative')
assertClose(scoreExpectedContribution(1.5, 2.0, 3.0), 1.5, 0.00001, 'Dmg ratio at 1.5')
assertClose(scoreExpectedContribution(2.0, 2.0, 3.0), 3.0, 0.00001, 'Dmg ratio at full 2.0')
assertClose(scoreExpectedContribution(3.5, 2.0, 3.0), 3.0, 0.00001, 'Dmg ratio > 2.0 capped at 3.0')
assertClose(scoreExpectedContribution(Infinity, 2.0, 3.0), 3.0, 0.00001, 'Dmg ratio Infinity capped at 3.0')

// Gold: baseline=1.0, full=1.5, weight=2.0
assertClose(scoreExpectedContribution(1.0, 1.5, 2.0), 0.0, 0.00001, 'Gold ratio at 1.0')
assertClose(scoreExpectedContribution(1.25, 1.5, 2.0), 1.0, 0.00001, 'Gold ratio at 1.25')
assertClose(scoreExpectedContribution(1.5, 1.5, 2.0), 2.0, 0.00001, 'Gold ratio at full 1.5')
assertClose(scoreExpectedContribution(2.0, 1.5, 2.0), 2.0, 0.00001, 'Gold ratio > 1.5 capped at 2.0')

// 1.5 scoreHealing (Solo vs Team)
console.log('  Testing scoreHealing boundary values...')
// Team mode (5 participants): min=0.2, full=1.4, maxScore=2.0
assertClose(scoreHealing(0.2, 5), 0.0, 0.00001, 'Healing team at baseline 0.2')
assertClose(scoreHealing(0.1, 5), 0.0, 0.00001, 'Healing team below baseline')
assertClose(scoreHealing(0.0, 5), 0.0, 0.00001, 'Healing team at 0.0')
assertClose(scoreHealing(-2.0, 5), 0.0, 0.00001, 'Healing team negative')
assertClose(scoreHealing(0.8, 5), 1.0, 0.00001, 'Healing team midpoint (0.8 - 0.2) / 1.2 * 2 = 1.0')
assertClose(scoreHealing(1.4, 5), 2.0, 0.00001, 'Healing team full score at 1.4')
assertClose(scoreHealing(2.5, 5), 2.0, 0.00001, 'Healing team > 1.4 capped at 2.0')

// Solo mode (1 participant): min=0.2, full=1.0, maxScore=2.0
assertClose(scoreHealing(0.2, 1), 0.0, 0.00001, 'Healing solo at baseline 0.2')
assertClose(scoreHealing(0.6, 1), 1.0, 0.00001, 'Healing solo midpoint (0.6 - 0.2) / 0.8 * 2 = 1.0')
assertClose(scoreHealing(1.0, 1), 2.0, 0.00001, 'Healing solo full score at 1.0')
assertClose(scoreHealing(1.5, 1), 2.0, 0.00001, 'Healing solo > 1.0 capped at 2.0')

// 1.6 scoreCsPerMinute
console.log('  Testing scoreCsPerMinute boundary values...')
// min=5.0, full=10.0, maxScore=2.0
assertClose(scoreCsPerMinute(5.0), 0.0, 0.00001, 'CS/min at baseline 5.0')
assertClose(scoreCsPerMinute(3.0), 0.0, 0.00001, 'CS/min below baseline 3.0')
assertClose(scoreCsPerMinute(0.0), 0.0, 0.00001, 'CS/min at 0.0')
assertClose(scoreCsPerMinute(-10.0), 0.0, 0.00001, 'CS/min negative')
assertClose(scoreCsPerMinute(7.5), 1.0, 0.00001, 'CS/min at 7.5')
assertClose(scoreCsPerMinute(10.0), 2.0, 0.00001, 'CS/min at full 10.0')
assertClose(scoreCsPerMinute(14.0), 2.0, 0.00001, 'CS/min > 10.0 capped at 2.0')

// 1.7 scoreParticipation
console.log('  Testing scoreParticipation boundary values...')
// min=0.3, full=1.0, maxScore=2.0
assertClose(scoreParticipation(0.3), 0.0, 0.00001, 'KP at baseline 0.3')
assertClose(scoreParticipation(0.15), 0.0, 0.00001, 'KP below baseline')
assertClose(scoreParticipation(0.0), 0.0, 0.00001, 'KP at 0.0')
assertClose(scoreParticipation(-0.5), 0.0, 0.00001, 'KP negative')
assertClose(scoreParticipation(0.65), 1.0, 0.00001, 'KP midpoint (0.65 - 0.3)/0.7 * 2 = 1.0')
assertClose(scoreParticipation(1.0), 2.0, 0.00001, 'KP at 1.0')
assertClose(scoreParticipation(1.2), 2.0, 0.00001, 'KP > 1.0 capped at 2.0')

// 1.8 Full Single Akari Score Integration & Extreme Limits
console.log('  Testing computeSingleAkariScore at absolute limits...')
const zeroSummary: SingleAnalysis['summary'] = {
  championDamageRatioToTeamMax: 0,
  championDamageRatioToExpectedContribution: 0,
  championDamageRatioToMax: 0,
  championDamagePercentageOfTeam: 0,
  championDamagePerMinute: 0,
  damageTakenRatioToTeamMax: 0,
  damageTakenRatioToExpectedContribution: 0,
  damageTakenRatioToMax: 0,
  damageTakenPercentageOfTeam: 0,
  healingRatioToTeamAverageDamageTaken: 0,
  teamParticipantCount: 5,
  goldRatioToTeamMax: 0,
  goldRatioToExpectedContribution: 0,
  goldRatioToMax: 0,
  goldPercentageOfTeam: 0,
  csRatioToTeamMax: 0,
  csRatioToMax: 0,
  csPercentageOfTeam: 0,
  csPerMinute: 0,
  towerDamageRatioToTeamMax: 0,
  towerDamageRatioToMax: 0,
  towerDamagePercentageOfTeam: 0,
  visionScorePercentageOfTeam: 0,
  visionScoreRatioToExpectedContribution: 0,
  totalDamageShieldedOnTeammatesRatioToTeamMax: null,
  totalDamageShieldedOnTeammatesRatioToMax: null,
  totalDamageShieldedOnTeammatesPercentageOfTeam: null,
  kda: 0,
  win: false,
  killParticipation: 0,
  damageGoldEfficiency: 0,
  killDamageEfficiency: 0
}

const zeroScore = computeSingleAkariScore(zeroSummary)
assertClose(zeroScore.total, 0.0, 0.00001, 'Worst case total Akari score must be exactly 0.0')
assert(zeroScore.maxScore === 17.0, 'maxScore must be 17.0')
assert(!isNaN(zeroScore.total), 'zeroScore total is not NaN')

const maxCarrySummary: SingleAnalysis['summary'] = {
  ...zeroSummary,
  kda: 50.0, // max 1.0
  win: true, // max 1.0
  championDamageRatioToExpectedContribution: 3.0, // max 3.0
  damageTakenRatioToExpectedContribution: 3.0, // max 2.0
  healingRatioToTeamAverageDamageTaken: 2.0, // max 2.0
  csPerMinute: 12.0, // max 2.0
  goldRatioToExpectedContribution: 2.0, // max 2.0
  killParticipation: 1.0, // max 2.0
  visionScoreRatioToExpectedContribution: 3.0 // max 2.0
}

const maxCarryScore = computeSingleAkariScore(maxCarrySummary)
assertClose(maxCarryScore.kdaScore, 1.0, 0.00001, 'Max KDA score = 1.0')
assertClose(maxCarryScore.winRateScore, 1.0, 0.00001, 'Max WinRate score = 1.0')
assertClose(maxCarryScore.dmgScore, 3.0, 0.00001, 'Max Damage score = 3.0')
assertClose(maxCarryScore.dmgTakenScore, 2.0, 0.00001, 'Max Damage Taken score = 2.0')
assertClose(maxCarryScore.healingScore, 2.0, 0.00001, 'Max Healing score = 2.0')
assertClose(maxCarryScore.csScore, 2.0, 0.00001, 'Max CS score = 2.0')
assertClose(maxCarryScore.goldScore, 2.0, 0.00001, 'Max Gold score = 2.0')
assertClose(maxCarryScore.participationScore, 2.0, 0.00001, 'Max KP score = 2.0')
assertClose(maxCarryScore.visionScore, 2.0, 0.00001, 'Max Vision score = 2.0')
assertClose(maxCarryScore.total, 17.0, 0.00001, 'Extreme carry total score must be exactly 17.0')

// 1.9 Aggregate Akari Score Thresholds
console.log('  Testing computeAggregatedAkariScore outstanding & extraordinary thresholds...')
const dummyPreparedGame = {
  single: { summary: maxCarrySummary }
} as any

// Outstanding requires: total >= 6.5 && count >= 5
const aggOutstandingFailCount = computeAggregatedAkariScore({
  count: 4,
  summary: { avgKda: 10, winRate: 1.0 } as any,
  games: [dummyPreparedGame, dummyPreparedGame, dummyPreparedGame, dummyPreparedGame]
})
assert(aggOutstandingFailCount.total === 17.0, 'Total is 17.0')
assert(aggOutstandingFailCount.outstanding === false, 'Outstanding must be false when count < 5')

const aggOutstandingPass = computeAggregatedAkariScore({
  count: 5,
  summary: { avgKda: 10, winRate: 1.0 } as any,
  games: [dummyPreparedGame, dummyPreparedGame, dummyPreparedGame, dummyPreparedGame, dummyPreparedGame]
})
assert(aggOutstandingPass.outstanding === true, 'Outstanding must be true when total >= 6.5 and count >= 5')
assert(aggOutstandingPass.extraordinary === false, 'Extraordinary must be false when count < 8')

const aggExtraordinaryPass = computeAggregatedAkariScore({
  count: 8,
  summary: { avgKda: 10, winRate: 1.0 } as any,
  games: Array(8).fill(dummyPreparedGame)
})
assert(aggExtraordinaryPass.extraordinary === true, 'Extraordinary must be true when total >= 8.0 and count >= 8')

// 1.10 Fuzz testing 10,000 random inputs for NaN or unhandled exceptions
console.log('  Fuzz testing scoring functions with 10,000 randomized float values...')
for (let i = 0; i < 10000; i++) {
  const val = (Math.random() - 0.5) * 2000 // range [-1000, 1000]
  const kdaS = scoreKda(val)
  assert(!isNaN(kdaS) && isFinite(kdaS) && kdaS >= 0 && kdaS <= 1, `Fuzz scoreKda failed on ${val}`)
  const winS = scoreWinRate(val)
  assert(!isNaN(winS) && isFinite(winS) && winS >= 0 && winS <= 1, `Fuzz scoreWinRate failed on ${val}`)
  const dmgS = scoreExpectedContribution(val, 2.0, 3.0)
  assert(!isNaN(dmgS) && isFinite(dmgS) && dmgS >= 0 && dmgS <= 3, `Fuzz scoreExpectedContribution failed on ${val}`)
  const healS = scoreHealing(val, Math.random() > 0.5 ? 5 : 1)
  assert(!isNaN(healS) && isFinite(healS) && healS >= 0 && healS <= 2, `Fuzz scoreHealing failed on ${val}`)
  const csS = scoreCsPerMinute(val)
  assert(!isNaN(csS) && isFinite(csS) && csS >= 0 && csS <= 2, `Fuzz scoreCsPerMinute failed on ${val}`)
  const kpS = scoreParticipation(val)
  assert(!isNaN(kpS) && isFinite(kpS) && kpS >= 0 && kpS <= 2, `Fuzz scoreParticipation failed on ${val}`)
}
console.log('  ✅ Suite 1: All 10,000 scoring fuzz trials and boundary tests passed!')

// ======================================================================
// SUITE 2: TEAM PREMADE DETECTION (TeamUpGraph & UnionFind)
// ======================================================================
console.log('\n>>> [SUITE 2] Team Premade Detection & Graph Algorithms')

// 2.1 UnionFind Disjoint & Cyclic Set Stress Test
console.log('  Testing UnionFind disjoint sets and cyclic merges...')
const uf = new UnionFind(['P1', 'P2', 'P3', 'P4', 'P5', 'P6', 'P7', 'P8', 'P9', 'P10'])
// Initially all are disjoint
assert(uf.find('P1') === 'P1', 'P1 initially self root')
assert(uf.find('P1') !== uf.find('P2'), 'P1 and P2 initially disjoint')

// Cyclic union: P1-P2, P2-P3, P3-P1
uf.union('P1', 'P2')
uf.union('P2', 'P3')
uf.union('P3', 'P1') // cycle
assert(uf.find('P1') === uf.find('P2'), 'P1 connected to P2')
assert(uf.find('P2') === uf.find('P3'), 'P2 connected to P3')
assert(uf.find('P1') === uf.find('P3'), 'P1 connected to P3 in cyclic group')

// Disjoint second component: P4-P5
uf.union('P4', 'P5')
assert(uf.find('P4') === uf.find('P5'), 'P4 connected to P5')
assert(uf.find('P1') !== uf.find('P4'), 'Component {P1,P2,P3} disjoint from {P4,P5}')

// Transitive chain: P6-P7, P7-P8, P8-P9, P9-P10
uf.union('P6', 'P7')
uf.union('P7', 'P8')
uf.union('P8', 'P9')
uf.union('P9', 'P10')
assert(uf.find('P6') === uf.find('P10'), 'Chain P6..P10 connected')
assert(uf.find('P1') !== uf.find('P6'), 'P1 disjoint from chain P6..P10')

// 2.2 combinations generator
console.log('  Testing combinations powerset generator...')
const cEmpty = combinations([])
assert(cEmpty.length === 1 && cEmpty[0].length === 0, 'combinations([]) gives [[]]')

const c3 = combinations(['A', 'B', 'C'])
assert(c3.length === 8, `combinations 3 elements should be 2^3 = 8 (got ${c3.length})`)

const c5 = combinations(['1', '2', '3', '4', '5'])
assert(c5.length === 32, `combinations 5 elements should be 2^5 = 32 (got ${c5.length})`)

// 2.3 removeSubsets & mergeOverlappingSets
console.log('  Testing removeSubsets & mergeOverlappingSets...')
const nestedObjects = [
  { p: ['A'] },
  { p: ['A', 'B'] },
  { p: ['A', 'B', 'C'] },
  { p: ['D', 'E'] }
]
const filtered = removeSubsets(nestedObjects, (o) => o.p)
assert(filtered.length === 2, `removeSubsets should keep only ['A','B','C'] and ['D','E'] (got ${filtered.length})`)
assert(filtered.some((f) => f.p.length === 3 && f.p.includes('C')), 'Largest superset retained')
assert(filtered.some((f) => f.p.length === 2 && f.p.includes('D')), 'Disjoint set retained')

const overlappingSets = [
  ['A', 'B'],
  ['B', 'C'],
  ['D', 'E'],
  ['E', 'F']
]
const merged = mergeOverlappingSets(overlappingSets)
assert(merged.length === 2, `mergeOverlappingSets should merge into 2 components (got ${merged.length})`)
const comp1 = merged.find((m) => m.includes('A'))
const comp2 = merged.find((m) => m.includes('D'))
assert(comp1 !== undefined && comp1.length === 3, 'Component 1 is {A, B, C}')
assert(comp2 !== undefined && comp2.length === 3, 'Component 2 is {D, E, F}')

// 2.4 calculateTogetherTimes with 2-player, 3-player, 5-player & Cyclic Graphs
console.log('  Testing calculateTogetherTimes under realistic & adversarial match graphs...')

// Scenario A: 2-player duo (P1, P2) played 3 matches together, with solo players
const matchesDuo = [
  { id: 'm1', players: ['P1', 'P2', 'R1', 'R2', 'R3'] },
  { id: 'm2', players: ['P1', 'P2', 'R4', 'R5', 'R6'] },
  { id: 'm3', players: ['P1', 'P2', 'R7', 'R8', 'R9'] }
]
const resDuo = calculateTogetherTimes(matchesDuo, ['P1', 'P2', 'R1', 'R4'], 3)
assert(resDuo.length === 1, `Expected 1 duo detected (got ${resDuo.length})`)
assert(resDuo[0].players.includes('P1') && resDuo[0].players.includes('P2'), 'P1 and P2 detected')
assert(resDuo[0].times === 3, `Duo played 3 times (got ${resDuo[0].times})`)

// Scenario B: 5-player full stack (5-premade) across 4 matches
const matches5Stack = [
  { id: 'g1', players: ['A', 'B', 'C', 'D', 'E'] },
  { id: 'g2', players: ['A', 'B', 'C', 'D', 'E'] },
  { id: 'g3', players: ['A', 'B', 'C', 'D', 'E'] },
  { id: 'g4', players: ['A', 'B', 'C', 'D', 'E'] }
]
const res5Stack = calculateTogetherTimes(matches5Stack, ['A', 'B', 'C', 'D', 'E'], 3)
assert(res5Stack.length === 1, `5-player stack should condense to exactly 1 team (got ${res5Stack.length})`)
assert(res5Stack[0].players.length === 5, `Expected 5 players in result (got ${res5Stack[0].players.length})`)
assert(res5Stack[0].times === 4, `Played together 4 times (got ${res5Stack[0].times})`)

// Scenario C: Cyclic False Premade Trap
// Game 1: P1 & P2
// Game 2: P2 & P3
// Game 3: P3 & P1
// All three NEVER played in the same game! A trio of 3 MUST NOT be detected!
const matchesCyclic = [
  { id: 'c1', players: ['P1', 'P2', 'X1'] },
  { id: 'c2', players: ['P2', 'P3', 'X2'] },
  { id: 'c3', players: ['P3', 'P1', 'X3'] }
]
const resCyclic = calculateTogetherTimes(matchesCyclic, ['P1', 'P2', 'P3'], 2)
// Since each pair only played together 1 time, with threshold=2, nothing should pass!
assert(resCyclic.length === 0, `Cyclic pair threshold 2 should detect 0 premades (got ${resCyclic.length})`)

const resCyclicT1 = calculateTogetherTimes(matchesCyclic, ['P1', 'P2', 'P3'], 1)
// With threshold=1: duos exist (P1-P2 in c1, P2-P3 in c2, P3-P1 in c3), BUT NO TRIO {P1, P2, P3} played together!
const trio = resCyclicT1.find((r) => r.players.length === 3)
assert(trio === undefined, 'A 3-player trio must NEVER be formed when all 3 never shared a match!')

// Scenario D: Threshold Variation Stress
const matchesThreshold = [
  { id: 't1', players: ['U1', 'U2'] },
  { id: 't2', players: ['U1', 'U2'] },
  { id: 't3', players: ['U1', 'U2'] },
  { id: 't4', players: ['U1', 'U2'] },
  { id: 't5', players: ['U1', 'U2'] }
]
assert(calculateTogetherTimes(matchesThreshold, ['U1', 'U2'], 1).length === 1, 'Threshold 1 passes')
assert(calculateTogetherTimes(matchesThreshold, ['U1', 'U2'], 5).length === 1, 'Threshold 5 passes')
assert(calculateTogetherTimes(matchesThreshold, ['U1', 'U2'], 6).length === 0, 'Threshold 6 filters out')

console.log('  ✅ Suite 2: All TeamUpGraph & UnionFind tests passed!')

// ======================================================================
// SUITE 3: 21 TACTICAL TAGS EVALUATOR TRIGGER INTEGRITY
// ======================================================================
console.log('\n>>> [SUITE 3] 21 Tactical Tags Evaluator Comprehensive Trigger Tests')

// Ensure exactly 21 tag definitions
assert(TACTICAL_TAG_DEFINITIONS.length === 21, `Must have exactly 21 tactical tag definitions (got ${TACTICAL_TAG_DEFINITIONS.length})`)

// Helper baseline context generator
function createBaseContext(puuid = 'puuid-base'): TacticalTagContext {
  return {
    puuid,
    selfPuuid: 'puuid-self',
    analysis: null,
    settings: { ...DEFAULT_TAG_SETTINGS }
  }
}

// 3.1 Tag 1: self
console.log('  Testing Tag 1: self...')
const tagSelfInactive = evaluateTacticalTags(createBaseContext('puuid-other')).find((t) => t.id === 'self')
assert(tagSelfInactive === undefined, 'self tag must be inactive for other puuid')

const ctxSelf = createBaseContext('puuid-self')
const tagSelfActive = evaluateTacticalTags(ctxSelf).find((t) => t.id === 'self')
assert(tagSelfActive !== undefined && tagSelfActive.label === '本人', 'self tag must be active for self puuid')

const tagSelfDisabled = evaluateTacticalTags(ctxSelf, { showSelfTag: false }).find((t) => t.id === 'self')
assert(tagSelfDisabled === undefined, 'self tag disabled by settings')

// 3.2 Tag 2: tagged
console.log('  Testing Tag 2: tagged...')
const ctxTagged = createBaseContext()
ctxTagged.savedInfo = { tags: [{ tag: '代练高玩', markedBySelf: true }] }
const tagTagged = evaluateTacticalTags(ctxTagged).find((t) => t.id === 'tagged')
assert(tagTagged !== undefined && tagTagged.label === '备注: 代练高玩', 'tagged tag must reflect note')

// 3.3 Tag 3: premade-team
console.log('  Testing Tag 3: premade-team...')
const ctxPremade = createBaseContext()
ctxPremade.premadeTeamId = 'B'
const tagPremade = evaluateTacticalTags(ctxPremade).find((t) => t.id === 'premade-team')
assert(tagPremade !== undefined && tagPremade.label === '车队 B', 'premade-team tag must reflect team id B')

// 3.4 Tag 4: high-win-rate (>= 16 games & >= 85% win rate)
console.log('  Testing Tag 4: high-win-rate...')
const ctxWinRate = createBaseContext()
ctxWinRate.analysis = {
  winLoss: { all: { count: 15, winRate: 0.9, wins: 14 } }
} as any
assert(evaluateTacticalTags(ctxWinRate).find((t) => t.id === 'high-win-rate') === undefined, 'Count 15 is below threshold 16')

ctxWinRate.analysis = {
  winLoss: { all: { count: 16, winRate: 0.84, wins: 13 } }
} as any
assert(evaluateTacticalTags(ctxWinRate).find((t) => t.id === 'high-win-rate') === undefined, 'Win rate 84% is below 85%')

ctxWinRate.analysis = {
  winLoss: { all: { count: 16, winRate: 0.85, wins: 14 } }
} as any
const tagWinRate = evaluateTacticalTags(ctxWinRate).find((t) => t.id === 'high-win-rate')
assert(tagWinRate !== undefined && tagWinRate.label === '绝活/胜率队', 'Win rate >= 85% with >= 16 games triggers tag')

// 3.5 Tag 5: met
console.log('  Testing Tag 5: met...')
const ctxMet = createBaseContext()
ctxMet.encounteredCount = 3
const tagMet = evaluateTacticalTags(ctxMet).find((t) => t.id === 'met')
assert(tagMet !== undefined && tagMet.label === '曾遇 3次', 'met tag triggers with count 3')

// 3.6 Tag 6: privacy
console.log('  Testing Tag 6: privacy...')
const ctxPriv = createBaseContext()
ctxPriv.summoner = { privacy: 'PRIVATE' }
const tagPriv = evaluateTacticalTags(ctxPriv).find((t) => t.id === 'privacy')
assert(tagPriv !== undefined && tagPriv.label === '隐藏战绩', 'privacy tag triggers on PRIVATE')

// 3.7 Tag 7: winning-streak (>= 3)
console.log('  Testing Tag 7: winning-streak...')
const ctxWinStreak = createBaseContext()
ctxWinStreak.analysis = { winLoss: { all: { winningStreak: 2 } } } as any
assert(evaluateTacticalTags(ctxWinStreak).find((t) => t.id === 'winning-streak') === undefined, '2 streak fails')
ctxWinStreak.analysis = { winLoss: { all: { winningStreak: 3 } } } as any
const tagWinStreak = evaluateTacticalTags(ctxWinStreak).find((t) => t.id === 'winning-streak')
assert(tagWinStreak !== undefined && tagWinStreak.label === '3连胜', '3 streak triggers')

// 3.8 Tag 8: losing-streak (>= 3)
console.log('  Testing Tag 8: losing-streak...')
const ctxLossStreak = createBaseContext()
ctxLossStreak.analysis = { winLoss: { all: { losingStreak: 2 } } } as any
assert(evaluateTacticalTags(ctxLossStreak).find((t) => t.id === 'losing-streak') === undefined, '2 loss streak fails')
ctxLossStreak.analysis = { winLoss: { all: { losingStreak: 4 } } } as any
const tagLossStreak = evaluateTacticalTags(ctxLossStreak).find((t) => t.id === 'losing-streak')
assert(tagLossStreak !== undefined && tagLossStreak.label === '4连败', '4 loss streak triggers')

// 3.9 Tag 9: great-performance (smurf / carry)
console.log('  Testing Tag 9: great-performance...')
const ctxGreat = createBaseContext()
ctxGreat.analysis = { akariScore: { total: 7.2, outstanding: true, extraordinary: false } } as any
const tagOutstanding = evaluateTacticalTags(ctxGreat).find((t) => t.id === 'great-performance')
assert(tagOutstanding !== undefined && tagOutstanding.label === '表现优异', 'outstanding triggers 表现优异')

ctxGreat.analysis = { akariScore: { total: 9.1, outstanding: true, extraordinary: true } } as any
const tagExtraordinary = evaluateTacticalTags(ctxGreat).find((t) => t.id === 'great-performance')
assert(tagExtraordinary !== undefined && tagExtraordinary.label === '通天代', 'extraordinary triggers 通天代')

// 3.10 Tag 10: suspicious-flash-position (flash key swap)
console.log('  Testing Tag 10: suspicious-flash-position...')
const ctxFlash = createBaseContext()
ctxFlash.analysis = { spells: { flashOnD: 5, flashOnF: 0 } } as any
assert(evaluateTacticalTags(ctxFlash).find((t) => t.id === 'suspicious-flash-position') === undefined, 'Only D flash fails')

ctxFlash.analysis = { spells: { flashOnD: 3, flashOnF: 7 } } as any
const tagFlash = evaluateTacticalTags(ctxFlash).find((t) => t.id === 'suspicious-flash-position')
assert(tagFlash !== undefined && tagFlash.label === '闪现异位', 'Both D and F flash triggers 闪现异位')

// 3.11 Tag 11: easy-gank (feeder detection & jungler suppression)
console.log('  Testing Tag 11: easy-gank...')
// Jungler suppression: assignedPosition === 'JUNGLE' or has smite spell
assert(getIsCurrentJungler('JUNGLE', undefined) === true, 'Assigned JUNGLE recognized as jungler')
assert(getIsCurrentJungler('MIDDLE', { spell1Id: 11, spell2Id: 4 }) === true, 'Smite on D recognized as jungler')
assert(getIsCurrentJungler('MIDDLE', { spell1Id: 4, spell2Id: 14 }) === false, 'Mid with Flash/Ignite is not jungler')

const ctxGank = createBaseContext()
ctxGank.assignedPosition = 'TOP'
ctxGank.analysis = {
  detailsCount: 10,
  details: { avgEarlyDeathsWithEnemyJunglerInvolved: 2.2 }
} as any
const tagVeryEasy = evaluateTacticalTags(ctxGank).find((t) => t.id === 'easy-gank')
assert(tagVeryEasy !== undefined && tagVeryEasy.label === '极易被抓', '>2 times early deaths triggers 极易被抓')

ctxGank.analysis = {
  detailsCount: 10,
  details: { avgEarlyDeathsWithEnemyJunglerInvolved: 1.6 }
} as any
const tagEasy = evaluateTacticalTags(ctxGank).find((t) => t.id === 'easy-gank')
assert(tagEasy !== undefined && tagEasy.label === '容易被抓', '1.6 times triggers 容易被抓')

ctxGank.analysis = {
  detailsCount: 10,
  details: { avgEarlyDeathsWithEnemyJunglerInvolved: 0.8 }
} as any
const tagHard = evaluateTacticalTags(ctxGank).find((t) => t.id === 'easy-gank')
assert(tagHard !== undefined && tagHard.label === '难被抓', '<1 time triggers 难被抓')

// Now make this player a Jungler: must suppress easy-gank
ctxGank.assignedPosition = 'JUNGLE'
assert(evaluateTacticalTags(ctxGank).find((t) => t.id === 'easy-gank') === undefined, 'Jungler must NOT have easy-gank tag')

// 3.12 Tag 12: solo-kills (>= 1.0)
console.log('  Testing Tag 12: solo-kills...')
const ctxSolo = createBaseContext()
ctxSolo.analysis = { summary: { avgSoloKills: 0.9 } } as any
assert(evaluateTacticalTags(ctxSolo).find((t) => t.id === 'solo-kills') === undefined, '0.9 solo kills fails')
ctxSolo.analysis = { summary: { avgSoloKills: 1.5 } } as any
const tagSolo = evaluateTacticalTags(ctxSolo).find((t) => t.id === 'solo-kills')
assert(tagSolo !== undefined && tagSolo.label === '单杀怪 1.5', '1.5 solo kills triggers 单杀怪')

// 3.13 Tag 13: average-team-damage (>= 0.25)
console.log('  Testing Tag 13: average-team-damage...')
const ctxDmgPct = createBaseContext()
ctxDmgPct.analysis = { summary: { avgChampionDamagePercentageOfTeam: 0.32 } } as any
const tagDmgPct = evaluateTacticalTags(ctxDmgPct, { showAverageTeamDamageTag: true }).find((t) => t.id === 'average-team-damage')
assert(tagDmgPct !== undefined && tagDmgPct.label === '输出 32%', '32% dmg triggers 输出 32%')

// 3.14 Tag 14: average-team-damage-taken (>= 0.25)
console.log('  Testing Tag 14: average-team-damage-taken...')
const ctxDmgTk = createBaseContext()
ctxDmgTk.analysis = { summary: { avgDamageTakenPercentageOfTeam: 0.28 } } as any
const tagDmgTk = evaluateTacticalTags(ctxDmgTk, { showAverageTeamDamageTakenTag: true }).find((t) => t.id === 'average-team-damage-taken')
assert(tagDmgTk !== undefined && tagDmgTk.label === '承伤 28%', '28% taken triggers 承伤 28%')

// 3.15 Tag 15: average-team-gold (>= 0.23)
console.log('  Testing Tag 15: average-team-gold...')
const ctxGold = createBaseContext()
ctxGold.analysis = { summary: { avgGoldPercentageOfTeam: 0.25 } } as any
const tagGold = evaluateTacticalTags(ctxGold, { showAverageTeamGoldTag: true }).find((t) => t.id === 'average-team-gold')
assert(tagGold !== undefined && tagGold.label === '经济 25%', '25% gold triggers 经济 25%')

// 3.16 Tag 16: average-cs-per-minute (>= 7.0)
console.log('  Testing Tag 16: average-cs-per-minute...')
const ctxCs = createBaseContext()
ctxCs.analysis = { summary: { avgCsPerMinute: 8.6 } } as any
const tagCs = evaluateTacticalTags(ctxCs, { showAverageCsPerMinuteTag: true }).find((t) => t.id === 'average-cs-per-minute')
assert(tagCs !== undefined && tagCs.label === '补刀 8.6/分', '8.6 CS/min triggers 补刀 8.6/分')

// 3.17 Tag 17: average-damage-gold-efficiency (>= 1.2)
console.log('  Testing Tag 17: average-damage-gold-efficiency...')
const ctxDge = createBaseContext()
ctxDge.analysis = { summary: { avgDamageGoldEfficiency: 1.45 } } as any
const tagDge = evaluateTacticalTags(ctxDge, { showAverageDamageGoldEfficiencyTag: true }).find((t) => t.id === 'average-damage-gold-efficiency')
assert(tagDge !== undefined && tagDge.label === '伤金转化 145%', '1.45 eff triggers 伤金转化 145%')

// 3.18 Tag 18: average-enemy-missing-pings (>= 2.0)
console.log('  Testing Tag 18: average-enemy-missing-pings...')
const ctxPings = createBaseContext()
ctxPings.analysis = { summary: { avgEnemyMissingPings: 3.2 } } as any
const tagPings = evaluateTacticalTags(ctxPings, { showAverageEnemyMissingPingsTag: true }).find((t) => t.id === 'average-enemy-missing-pings')
assert(tagPings !== undefined && tagPings.label === '问号标记 3.2', '3.2 pings triggers 问号标记 3.2')

// 3.19 Tag 19: average-vision-score (>= 25)
console.log('  Testing Tag 19: average-vision-score...')
const ctxVis = createBaseContext()
ctxVis.analysis = { summary: { avgVisionScore: 35 } } as any
const tagVis = evaluateTacticalTags(ctxVis, { showAverageVisionScoreTag: true }).find((t) => t.id === 'average-vision-score')
assert(tagVis !== undefined && tagVis.label === '视野控 35', '35 vision score triggers 视野控 35')

// 3.20 Tag 20: average-kill-damage-efficiency (KDE > 1.35 or < 0.65)
console.log('  Testing Tag 20: average-kill-damage-efficiency...')
const ctxKdeHigh = createBaseContext()
ctxKdeHigh.analysis = { summary: { avgKillDamageEfficiency: 1.5 } } as any
const tagKdeHigh = evaluateTacticalTags(ctxKdeHigh).find((t) => t.id === 'average-kill-damage-efficiency')
assert(tagKdeHigh !== undefined && tagKdeHigh.label === '人头收割', 'KDE 1.5 triggers 人头收割')

const ctxKdeLow = createBaseContext()
ctxKdeLow.analysis = { summary: { avgKillDamageEfficiency: 0.55 } } as any
const tagKdeLow = evaluateTacticalTags(ctxKdeLow).find((t) => t.id === 'average-kill-damage-efficiency')
assert(tagKdeLow !== undefined && tagKdeLow.label === '虚空伤害', 'KDE 0.55 triggers 虚空伤害')

const ctxKdeNormal = createBaseContext()
ctxKdeNormal.analysis = { summary: { avgKillDamageEfficiency: 1.0 } } as any
assert(evaluateTacticalTags(ctxKdeNormal).find((t) => t.id === 'average-kill-damage-efficiency') === undefined, 'KDE 1.0 normal is null')

// 3.21 Tag 21: akari-score
console.log('  Testing Tag 21: akari-score...')
const ctxAkari = createBaseContext()
ctxAkari.analysis = { akariScore: { total: 11.45 } } as any
const tagAkari = evaluateTacticalTags(ctxAkari, { showAkariScoreTag: true }).find((t) => t.id === 'akari-score')
assert(tagAkari !== undefined && tagAkari.label === 'Akari 11.45', 'Akari score tag triggers')

// 3.22 Priority Sorting Integrity Check
console.log('  Testing priority descending ordering across all active tags...')
const ctxAllActive: TacticalTagContext = {
  puuid: 'puuid-me',
  selfPuuid: 'puuid-me',
  premadeTeamId: 'A',
  assignedPosition: 'TOP',
  spells: { spell1Id: 4, spell2Id: 12 },
  summoner: { privacy: 'PRIVATE' },
  savedInfo: { tags: [{ tag: '好友', markedBySelf: true }] },
  encounteredCount: 2,
  analysis: {
    detailsCount: 10,
    winLoss: { all: { count: 20, winRate: 0.9, winningStreak: 5, losingStreak: 0 } } as any,
    akariScore: { total: 9.5, outstanding: true, extraordinary: true },
    spells: { flashOnD: 2, flashOnF: 8 } as any,
    details: { avgEarlyDeathsWithEnemyJunglerInvolved: 2.5 } as any,
    summary: {
      avgSoloKills: 2.0,
      avgChampionDamagePercentageOfTeam: 0.35,
      avgDamageTakenPercentageOfTeam: 0.3,
      avgGoldPercentageOfTeam: 0.28,
      avgCsPerMinute: 9.0,
      avgDamageGoldEfficiency: 1.5,
      avgEnemyMissingPings: 4.0,
      avgVisionScore: 40,
      avgKillDamageEfficiency: 1.6
    } as any
  } as any
}

// Enable all tags in settings
const allEnabledSettings = Object.keys(DEFAULT_TAG_SETTINGS).reduce((acc, k) => {
  acc[k as keyof typeof DEFAULT_TAG_SETTINGS] = true
  return acc
}, {} as any)

const allTags = evaluateTacticalTags(ctxAllActive, allEnabledSettings)
for (let i = 0; i < allTags.length - 1; i++) {
  assert(
    allTags[i].priority >= allTags[i + 1].priority,
    `Priority inversion: ${allTags[i].id} (${allTags[i].priority}) before ${allTags[i + 1].id} (${allTags[i + 1].priority})`
  )
}
console.log(`  All-enabled evaluated tag count: ${allTags.length} tags correctly sorted by priority!`)

// 3.23 Match Card Tags (MVP, SVP, Penta, Damage Leader, etc.)
console.log('  Testing Match Card tags (MVP, SVP, Multi-kills, Damage leaders)...')
const dummyTeamsResult: TeamsAdapterResult = {
  teamStatMap: {},
  teamStatsArr: [],
  allTeamStats: {
    maxDamageDealtToChampions: 45000,
    maxDamageTaken: 38000,
    maxGoldEarned: 18000
  } as any
}
const dummyBasicInfo: MatchBasicInfo = {
  gameDuration: 1800
} as any

const mvpParticipant: MatchParticipant = {
  isMvp: true,
  isSvp: false,
  pentaKills: 1,
  quadraKills: 0,
  tripleKills: 0,
  totalDamageDealtToChampions: 45000,
  totalDamageTaken: 20000,
  goldEarned: 18000,
  firstBloodKill: true,
  deaths: 0,
  kills: 12,
  assists: 8
} as any

const mvpTags = computePlayerTags(mvpParticipant, null, dummyTeamsResult, dummyBasicInfo)
assert(mvpTags.some((t) => t.label.includes('MVP')), 'MVP tag present')
assert(mvpTags.some((t) => t.label.includes('五杀')), 'Penta tag present')
assert(mvpTags.some((t) => t.label.includes('全场输出最高')), 'Damage leader tag present')
assert(mvpTags.some((t) => t.label.includes('经济第一')), 'Gold leader tag present')
assert(mvpTags.some((t) => t.label.includes('第一滴血')), 'First blood tag present')
assert(mvpTags.some((t) => t.label.includes('完美KDA')), 'Perfect KDA tag present')

const svpParticipant: MatchParticipant = {
  isMvp: false,
  isSvp: true,
  pentaKills: 0,
  quadraKills: 1,
  tripleKills: 0,
  totalDamageDealtToChampions: 30000,
  totalDamageTaken: 38000,
  goldEarned: 14000,
  firstBloodKill: false,
  deaths: 2,
  kills: 8,
  assists: 4
} as any

const svpTags = computePlayerTags(svpParticipant, null, dummyTeamsResult, dummyBasicInfo)
assert(svpTags.some((t) => t.label.includes('SVP')), 'SVP tag present')
assert(svpTags.some((t) => t.label.includes('四杀')), 'Quadra tag present')
assert(svpTags.some((t) => t.label.includes('全场承伤最高')), 'Damage taken leader tag present')

console.log('  ✅ Suite 3: All 21 tactical tag triggers and match card tags verified!')

// ======================================================================
// SUITE 4: PERFORMANCE & STRESS HARNESS
// ======================================================================
console.log('\n>>> [SUITE 4] High-Frequency Stress & Latency Benchmark')

const tStart = performance.now()
const benchmarkIterations = 5000
for (let i = 0; i < benchmarkIterations; i++) {
  evaluateTacticalTags(ctxAllActive, allEnabledSettings)
}
const elapsedMs = performance.now() - tStart
const usPerOp = (elapsedMs * 1000) / benchmarkIterations

console.log(`  Executed ${benchmarkIterations} tag evaluation cycles in ${elapsedMs.toFixed(2)} ms (~${usPerOp.toFixed(2)} μs per evaluation)`)
assert(usPerOp < 500, `Evaluation should take < 500 μs per cycle (took ${usPerOp.toFixed(2)} μs)`)

console.log('\n======================================================================')
console.log(`🎉 ADVERSARIAL VERIFICATION COMPLETE! Pass: ${passCount}, Fail: ${failCount}`)
console.log('======================================================================\n')
