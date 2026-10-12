export function useGameResultName() {
  return function (placement?: number, winResult?: string, isSurrender?: boolean) {
    if (placement && placement > 0) {
      return `第${placement}名`
    }
    if (winResult === 'win') {
      return isSurrender ? '胜利 (敌方投降)' : '胜利'
    }
    if (winResult === 'loss') {
      return isSurrender ? '失败 (我方投降)' : '失败'
    }
    if (winResult === 'remake') return '重开'
    if (winResult === 'abort') return '中止'
    return '对局结束'
  }
}

export function useTeamName() {
  return function (teamIdentifier?: string) {
    if (teamIdentifier === '100' || teamIdentifier === 'blue') return '蓝方'
    if (teamIdentifier === '200' || teamIdentifier === 'red') return '红方'
    return `队伍 ${teamIdentifier || ''}`
  }
}

export function formatParticipantRole(role?: string): string {
  if (!role) return '未知'
  const r = role.toUpperCase()
  switch (r) {
    case 'TOP':
      return '上路'
    case 'JUNGLE':
    case 'JUG':
      return '打野'
    case 'MIDDLE':
    case 'MID':
      return '中路'
    case 'BOTTOM':
    case 'BOT':
    case 'ADC':
    case 'DUO_CARRY':
      return '下路'
    case 'UTILITY':
    case 'SUPPORT':
    case 'SUP':
    case 'DUO_SUPPORT':
      return '辅助'
    default:
      return role
  }
}

export const STATS_KEY_TRANSLATION_MAP: Record<string, string> = {
  // combat-stats
  kills: '击杀数',
  deaths: '阵亡数',
  assists: '助攻数',
  firstBloodKill: '获得一血',
  firstBloodAssist: '一血助攻',
  doubleKills: '双杀次数',
  tripleKills: '三杀次数',
  quadraKills: '四杀次数',
  pentaKills: '五杀次数',
  unrealKills: '六杀/超绝',
  killingSprees: '大杀特杀',
  largestKillingSpree: '最大连杀',
  largestMultiKill: '最大多杀',
  longestTimeSpentLiving: '最长存活(秒)',
  largestCriticalStrike: '最大暴击伤害',
  legendaryCount: '超神次数',
  highestChampionDamage: '全场伤害最高',
  damageGoldEfficiency: '金币转化率',
  // akari
  akariScore: 'Akari 评分',
  // damage
  totalDamageDealtToChampions: '对英雄总伤害',
  physicalDamageDealtToChampions: '对英雄物理伤害',
  magicDamageDealtToChampions: '对英雄魔法伤害',
  trueDamageDealtToChampions: '对英雄真实伤害',
  totalDamageDealt: '总输出伤害',
  physicalDamageDealt: '总物理伤害',
  magicDamageDealt: '总魔法伤害',
  trueDamageDealt: '总真实伤害',
  damageDealtToObjectives: '对大龙/野怪伤害',
  damageDealtToTurrets: '对防御塔伤害',
  damageDealtToBuildings: '对建筑物伤害',
  damageDealtToEpicMonsters: '对史诗野怪伤害',
  totalDamageTaken: '承受总伤害',
  physicalDamageTaken: '承受物理伤害',
  magicDamageTaken: '承受魔法伤害',
  trueDamageTaken: '承受真实伤害',
  damageSelfMitigated: '自我减免伤害',
  totalDamageShieldedOnTeammates: '队友护盾吸收伤害',
  // cc
  timeCCingOthers: '控制他人时长(秒)',
  totalTimeCCDealt: '施加控制总时长',
  totalTimeCrowdControlDealt: '控制综合时长',
  enemyChampionImmobilizations: '定身敌方次数',
  immobilizeAndKillWithAlly: '配合击杀定身敌人',
  knockEnemyIntoTeamAndKill: '击退击飞开团击杀',
  survivedThreeImmobilizesInFight: '团战承受3次控制存活',
  highestCrowdControlScore: '控制得分最高',
  // vision
  visionScore: '视野得分',
  wardsPlaced: '放置守卫',
  wardsKilled: '排查守卫',
  sightWardsBoughtInGame: '购买隐形守卫',
  visionWardsBoughtInGame: '购买控制守卫',
  detectorWardsPlaced: '放置真眼',
  visionScorePerMinute: '分均视野得分',
  wardTakedowns: '参与排眼',
  // buildings
  firstTowerKill: '摧毁一塔',
  turretTakedowns: '参与推塔',
  inhibitorTakedowns: '参与破水晶',
  turretKills: '推塔数',
  inhibitorKills: '破水晶数',
  // objectives
  dragonKills: '击杀小龙',
  baronKills: '击杀男爵',
  riftHeraldKills: '击杀峡谷先锋',
  voidGrubKills: '击杀虚空巢虫',
  epicMonsterSteals: '抢史诗级野怪',
  // economy
  goldEarned: '总金币获取',
  goldSpent: '消耗金币',
  cs: '补刀总数',
  totalMinionsKilled: '小兵补刀',
  neutralMinionsKilled: '野怪补刀',
  neutralMinionsKilledEnemyJungle: '敌方野区反野',
  // healing
  totalHeal: '总治疗量',
  totalHealsOnTeammates: '对友军治疗量',
  // combat-advanced
  soloKills: '单杀次数',
  bountyGold: '赏金金币',
  // abilities
  spell1Cast: 'Q 技能施放',
  spell2Cast: 'W 技能施放',
  spell3Cast: 'E 技能施放',
  spell4Cast: 'R 技能施放'
}

export function getStatKeyTranslation(key: string): string {
  return STATS_KEY_TRANSLATION_MAP[key] || key
}
