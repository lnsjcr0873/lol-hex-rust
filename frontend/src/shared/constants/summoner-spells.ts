/** 闪现 ID */
export const SUMMONER_SPELL_FLASH_ID = 4

/** 惩戒 ID */
export const SUMMONER_SPELL_SMITE_ID = 11

export interface SummonerSpellInfo {
  name: string
  iconKey: string
}

export const SPELL_NAME_MAP: Record<number, SummonerSpellInfo> = {
  1: { name: '净化', iconKey: 'summoner_boost' },
  3: { name: '虚弱', iconKey: 'summoner_exhaust' },
  4: { name: '闪现', iconKey: 'summoner_flash' },
  6: { name: '幽灵疾步', iconKey: 'summoner_haste' },
  7: { name: '治疗术', iconKey: 'summoner_heal' },
  11: { name: '惩戒', iconKey: 'summoner_smite' },
  12: { name: '传送', iconKey: 'summoner_teleport' },
  13: { name: '清晰术', iconKey: 'summoner_mana' },
  14: { name: '点燃', iconKey: 'summoner_ignite' },
  21: { name: '屏障', iconKey: 'summoner_barrier' },
  30: { name: '王者魄罗', iconKey: 'summoner_poro_recall' },
  31: { name: '魄罗投掷', iconKey: 'summoner_poro_throw' },
  32: { name: '雪球', iconKey: 'summoner_mark' },
  39: { name: '雪球标记', iconKey: 'summoner_mark' },
  2201: { name: '逃跑', iconKey: 'summoner_cherry_flee' },
  2202: { name: '闪现', iconKey: 'summoner_cherry_flash' }
}

export function getSpellName(spellId?: number | null): string {
  if (!spellId) return '未知技能'
  return SPELL_NAME_MAP[spellId]?.name ?? `技能 #${spellId}`
}
