import type { RenderGroup } from './types'

export const RENDER_GROUP_DISPLAY_ORDER = [
  'combat-stats',
  'akari',
  'game-state',
  'misc',
  'damage',
  'cc',
  'vision',
  'buildings',
  'objectives',
  'economy',
  'healing',
  'pings',
  'combat-advanced',
  'statistics',
  'abilities',
  'survival-skills',
  'teamfight',
  'undocumented'
]

export const GROUP_NAME_MAP: Record<string, string> = {
  'combat-stats': '核心战斗数据',
  akari: 'Akari 综合评分',
  'game-state': '局内胜负状态',
  misc: '基础与位置',
  damage: '伤害与承受',
  cc: '控制与定身',
  vision: '视野得分统计',
  buildings: '防御塔与建筑',
  objectives: '史诗级大龙野怪',
  economy: '经济与补刀',
  healing: '治疗与回复',
  pings: '战术信号',
  'combat-advanced': '深度战斗进阶',
  statistics: '占比与转化',
  abilities: '技能施放统计',
  'survival-skills': '生存走位逃脱',
  teamfight: '团战输出参与',
  undocumented: '其他指标'
}

export const RENDER_GROUPS: RenderGroup[] = [
  {
    group: 'combat-stats',
    name: '核心战斗数据',
    items: [
      { key: 'kills', render: 'integer' },
      { key: 'deaths', render: 'integer' },
      { key: 'assists', render: 'integer' },
      { key: 'firstBloodKill', render: 'boolean' },
      { key: 'firstBloodAssist', render: 'boolean' },
      { key: 'doubleKills', render: 'integer' },
      { key: 'tripleKills', render: 'integer' },
      { key: 'quadraKills', render: 'integer' },
      { key: 'pentaKills', render: 'integer' },
      { key: 'killingSprees', render: 'integer' },
      { key: 'largestKillingSpree', render: 'integer' },
      { key: 'largestMultiKill', render: 'integer' },
      { key: 'longestTimeSpentLiving', render: 'integer' },
      { key: 'largestCriticalStrike', render: 'integer' },
      { key: 'legendaryCount', render: 'integer' },
      { key: 'highestChampionDamage', render: 'boolean' },
      { key: 'damageGoldEfficiency', render: 'percentage' }
    ]
  },
  {
    group: 'akari',
    name: 'Akari 综合评分',
    items: [{ key: 'akariScore', render: 'akari-score' }]
  },
  {
    group: 'damage',
    name: '伤害与承受',
    items: [
      { key: 'totalDamageDealtToChampions', render: 'compat' },
      { key: 'physicalDamageDealtToChampions', render: 'compat' },
      { key: 'magicDamageDealtToChampions', render: 'compat' },
      { key: 'trueDamageDealtToChampions', render: 'compat' },
      { key: 'totalDamageDealt', render: 'compat' },
      { key: 'physicalDamageDealt', render: 'compat' },
      { key: 'magicDamageDealt', render: 'compat' },
      { key: 'trueDamageDealt', render: 'compat' },
      { key: 'damageDealtToObjectives', render: 'compat' },
      { key: 'damageDealtToTurrets', render: 'compat' },
      { key: 'damageDealtToBuildings', render: 'compat' },
      { key: 'damageDealtToEpicMonsters', render: 'compat' },
      { key: 'totalDamageTaken', render: 'compat' },
      { key: 'physicalDamageTaken', render: 'compat' },
      { key: 'magicDamageTaken', render: 'compat' },
      { key: 'trueDamageTaken', render: 'compat' },
      { key: 'damageSelfMitigated', render: 'compat' },
      { key: 'totalDamageShieldedOnTeammates', render: 'compat' }
    ]
  },
  {
    group: 'cc',
    name: '控制与定身',
    items: [
      { key: 'timeCCingOthers', render: 'integer' },
      { key: 'totalTimeCCDealt', render: 'integer' },
      { key: 'totalTimeCrowdControlDealt', render: 'integer' },
      { key: 'enemyChampionImmobilizations', render: 'integer' },
      { key: 'immobilizeAndKillWithAlly', render: 'integer' },
      { key: 'knockEnemyIntoTeamAndKill', render: 'integer' },
      { key: 'survivedThreeImmobilizesInFight', render: 'integer' },
      { key: 'highestCrowdControlScore', render: 'integer' }
    ]
  },
  {
    group: 'vision',
    name: '视野得分统计',
    items: [
      { key: 'visionScore', render: 'integer' },
      { key: 'wardsPlaced', render: 'integer' },
      { key: 'wardsKilled', render: 'integer' },
      { key: 'sightWardsBoughtInGame', render: 'integer' },
      { key: 'visionWardsBoughtInGame', render: 'integer' },
      { key: 'detectorWardsPlaced', render: 'integer' },
      { key: 'visionScorePerMinute', render: 'integer' },
      { key: 'wardTakedowns', render: 'integer' }
    ]
  },
  {
    group: 'buildings',
    name: '防御塔与建筑',
    items: [
      { key: 'firstTowerKill', render: 'boolean' },
      { key: 'turretTakedowns', render: 'integer' },
      { key: 'inhibitorTakedowns', render: 'integer' },
      { key: 'turretKills', render: 'integer' },
      { key: 'inhibitorKills', render: 'integer' }
    ]
  },
  {
    group: 'objectives',
    name: '史诗级大龙野怪',
    items: [
      { key: 'dragonKills', render: 'integer' },
      { key: 'baronKills', render: 'integer' },
      { key: 'riftHeraldKills', render: 'integer' },
      { key: 'voidGrubKills', render: 'integer' },
      { key: 'epicMonsterSteals', render: 'integer' }
    ]
  },
  {
    group: 'economy',
    name: '经济与补刀',
    items: [
      { key: 'goldEarned', render: 'compat' },
      { key: 'goldSpent', render: 'compat' },
      { key: 'cs', render: 'integer' },
      { key: 'totalMinionsKilled', render: 'integer' },
      { key: 'neutralMinionsKilled', render: 'integer' },
      { key: 'neutralMinionsKilledEnemyJungle', render: 'integer' }
    ]
  },
  {
    group: 'healing',
    name: '治疗与回复',
    items: [
      { key: 'totalHeal', render: 'compat' },
      { key: 'totalHealsOnTeammates', render: 'compat' }
    ]
  },
  {
    group: 'combat-advanced',
    name: '深度战斗进阶',
    items: [
      { key: 'soloKills', render: 'integer' },
      { key: 'bountyGold', render: 'integer' }
    ]
  },
  {
    group: 'abilities',
    name: '技能施放统计',
    items: [
      { key: 'spell1Cast', render: 'integer' },
      { key: 'spell2Cast', render: 'integer' },
      { key: 'spell3Cast', render: 'integer' },
      { key: 'spell4Cast', render: 'integer' }
    ]
  }
]
