// Resource resolution helper for LoL static game data with reliable CDNs & fallbacks
import { SPELL_NAME_MAP, type SummonerSpellInfo } from '@shared/constants/summoner-spells'
export { SPELL_NAME_MAP, type SummonerSpellInfo }

export const CHAMPION_NAMES_MAP: Record<number, string> = {
  266: '暗裔剑魔',
  103: '九尾妖狐',
  84: '离群之刺',
  12: '牛头酋长',
  32: '殇之木乃伊',
  34: '冰晶凤凰',
  1: '黑暗之女',
  523: '残月之肃',
  22: '寒冰射手',
  136: '铸星龙王',
  268: '沙漠皇帝',
  432: '星界游神',
  200: '虚空女皇',
  53: '蒸汽机器人',
  63: '复仇焰魂',
  201: '弗雷尔卓德之心',
  233: '狂厄蔷薇',
  51: '皮城女警',
  164: '青钢影',
  69: '魔蛇之拥',
  31: '虚空恐惧',
  42: '英勇投弹手',
  122: '诺克萨斯之手',
  131: '皎月女神',
  119: '荣耀行刑官',
  36: '祖安狂人',
  245: '时间刺客',
  60: '蜘蛛女皇',
  28: '痛苦之拥',
  81: '探险家',
  9: '远古恐惧',
  114: '无双剑姬',
  105: '潮汐海灵',
  3: '正义巨像',
  41: '海洋之灾',
  86: '德玛西亚之力',
  150: '迷失之牙',
  79: '酒桶',
  104: '法外狂徒',
  883: '灵罗娃娃',
  120: '战争之影',
  74: '大发明家',
  910: '异画师',
  420: '海兽祭司',
  39: '刀锋舞者',
  427: '翠神',
  40: '风暴之怒',
  59: '德玛西亚皇子',
  24: '武器大师',
  126: '未来守护者',
  202: '戏命师',
  222: '暴走萝莉',
  897: '奎桑提',
  145: '虚空之女',
  429: '复仇之矛',
  43: '天启者',
  30: '死亡颂唱者',
  38: '虚空行者',
  55: '不祥之刃',
  85: '狂暴之心',
  121: '虚空掠夺者',
  203: '永猎双子',
  240: '暴怒骑士',
  96: '深渊巨口',
  7: '诡术妖姬',
  64: '盲僧',
  89: '曙光女神',
  876: '含羞蓓蕾',
  127: '冰霜女巫',
  236: '圣枪游侠',
  117: '仙灵女巫',
  99: '光辉女郎',
  54: '熔岩巨兽',
  90: '虚空先知',
  57: '扭曲树精',
  11: '无极剑圣',
  902: '明烛',
  21: '赏金猎人',
  62: '齐天大圣',
  82: '铁男',
  25: '堕落天使',
  895: '纳亚菲',
  267: '唤潮鲛姬',
  75: '沙漠死神',
  111: '深海泰坦',
  76: '狂野女猎手',
  893: '百裂冥犬',
  56: '永恒梦魇',
  20: '雪原双子',
  2: '狂战士',
  61: '发条魔灵',
  516: '山隐之焰',
  80: '不屈之枪',
  78: '圣锤之毅',
  555: '血港鬼影',
  246: '元素女皇',
  133: '德玛西亚之翼',
  497: '幻翎',
  33: '披甲龙龟',
  421: '虚空遁地兽',
  526: '镕铁少女',
  884: '烈娜塔 · 玻拉斯克',
  58: '荒漠屠夫',
  107: '傲之追猎者',
  92: '放逐之刃',
  68: '机械公敌',
  13: '符文法师',
  360: '沙漠玫瑰',
  113: '北地之怒',
  235: '涤魂圣枪',
  147: '星籁歌姬',
  875: '腕豪',
  35: '恶魔小丑',
  98: '暮光之眼',
  102: '龙血武姬',
  27: '炼金术士',
  14: '亡灵战神',
  15: '战争女神',
  72: '水晶先锋',
  901: '斯莫德',
  37: '琴瑟仙女',
  16: '众星之子',
  50: '诺克萨斯统领',
  517: '解脱者',
  134: '暗黑元首',
  223: '河流之王',
  163: '岩雀',
  91: '刀锋之影',
  44: '瓦罗兰之盾',
  17: '迅捷斥候',
  412: '魂锁典狱长',
  18: '麦林炮手',
  48: '巨魔之王',
  23: '蛮族之王',
  4: '卡牌大师',
  29: '瘟疫之源',
  77: '兽灵行者',
  6: '无畏战车',
  110: '惩戒之箭',
  67: '暗夜猎手',
  45: '邪恶小法师',
  161: '虚空之眼',
  711: '愁云使者',
  254: '皮城执法官',
  234: '破败之王',
  112: '机械先驱',
  8: '猩红收割者',
  106: '不灭狂雷',
  19: '祖安怒兽',
  498: '逆羽',
  101: '远古巫灵',
  5: '德邦总管',
  157: '疾风剑豪',
  777: '封魔剑魂',
  83: '牧魂人',
  350: '魔法猫咪',
  154: '生化魔人',
  238: '影流之主',
  221: '祖安花火',
  115: '爆破鬼才',
  143: '荆棘之兴',
  142: '暮光星灵',
  711: '愁云使者',
  799: '铁血狼母',
  800: '议员',
  888: '炼金男爵',
  893: '奥秘术士',
  895: '不羁之悦',
  901: '炽炎雏龙',
  902: '明烛'
}

export function getChampionName(champId: number): string {
  return CHAMPION_NAMES_MAP[champId] || `英雄${champId}`
}

export function getChampionTitle(champId: number): string {
  return CHAMPION_NAMES_MAP[champId] || ''
}


export function getChampionIconUrl(champId: number): string {
  if (!champId || champId <= 0) {
    return 'https://ddragon.leagueoflegends.com/cdn/14.24.1/img/profileicon/29.png'
  }
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${champId}.png`
}

export function getChampionIconFallbackUrl(champId: number): string {
  if (!champId || champId <= 0) return 'https://game.gtimg.cn/images/lol/act/img/skinloading/142000.jpg'
  return `https://game.gtimg.cn/images/lol/act/img/skinloading/${champId}000.jpg`
}

export function getProfileIconUrl(iconId: number): string {
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${iconId || 1}.jpg`
}

export function getItemIconUrl(itemId: number): string {
  if (!itemId || itemId <= 0) return ''
  return `https://game.gtimg.cn/images/lol/act/img/item/${itemId}.png`
}

export function getItemFallbackUrl(itemId: number): string {
  if (!itemId || itemId <= 0) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/item-icons/${itemId}.png`
}


export const SPELL_GTIMG_MAP: Record<number, string> = {
  1: 'SummonerBoost.png',
  3: 'SummonerExhaust.png',
  4: 'SummonerFlash.png',
  6: 'SummonerHaste.png',
  7: 'SummonerHeal.png',
  11: 'SummonerSmite.png',
  12: 'SummonerTeleport.png',
  13: 'SummonerMana.png',
  14: 'SummonerDot.png',
  21: 'SummonerBarrier.png',
  30: 'SummonerPoroRecall.png',
  31: 'SummonerPoroThrow.png',
  32: 'SummonerSnowball.png',
  39: 'SummonerSnowball.png',
  54: 'SummonerSmite.png',
  55: 'SummonerSmite.png',
  2201: 'SummonerCherryFlee.png',
  2202: 'SummonerCherryFlash.png'
}

export function getSpellName(spellId: number): string {
  return SPELL_NAME_MAP[spellId]?.name || `技能${spellId}`
}

export function getSpellIconUrl(spellId: number): string {
  const gtFileName = SPELL_GTIMG_MAP[spellId]
  if (gtFileName) {
    return `https://game.gtimg.cn/images/lol/act/img/spell/${gtFileName}`
  }
  const meta = SPELL_NAME_MAP[spellId]
  if (meta?.iconKey) {
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/${meta.iconKey}.png`
  }
  return `https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png`
}

export const PERK_NAME_MAP: Record<number, string> = {
  // 精密
  8000: '精密',
  8005: '强攻',
  8008: '致命节奏',
  8021: '迅捷步法',
  8010: '征服者',
  9101: '过量治疗',
  9111: '凯旋',
  8009: '气定神闲',
  9104: '传说：欢欣',
  9105: '传说：韧性',
  9103: '传说：血统',
  8014: '致命一击',
  8017: '砍倒',
  8299: '坚毅不倒',
  // 主宰
  8100: '主宰',
  8112: '电刑',
  8124: '掠食者',
  8128: '黑暗收割',
  9923: '丛刃',
  8126: '恶意中伤',
  8139: '血之滋味',
  8143: '猛然冲击',
  8136: '幽灵魄罗',
  8120: '僵尸守卫',
  8138: '眼球收集器',
  8135: '寻宝猎人',
  8134: '灵性猎手',
  8105: '无情猎手',
  8106: '终极猎人',
  // 巫术
  8200: '巫术',
  8214: '召唤：艾黎',
  8229: '奥术彗星',
  8230: '相位猛冲',
  8224: '无效化之法球',
  8226: '法力流系带',
  8275: '灵光披风',
  8210: '超然',
  8234: '迅捷',
  8233: '绝对专注',
  8237: '焦灼',
  8232: '水上行走',
  8236: '风暴聚集',
  // 坚决
  8400: '坚决',
  8437: '不灭之握',
  8439: '余震',
  8465: '守护者',
  8446: '爆破',
  8463: '生命源泉',
  8401: '护盾猛击',
  8429: '调节',
  8444: '复风之风',
  8473: '骸骨镀层',
  8451: '过度生长',
  8453: '复苏',
  8242: '坚定',
  // 启迪
  8300: '启迪',
  8351: '冰川增幅',
  8360: '启封的秘籍',
  8369: '先攻',
  8306: '海克斯科技闪现罗网',
  8304: '神奇之鞋',
  8313: '完美时机',
  8321: '未来市场',
  8316: '小兵去质器',
  8345: '饼干配送',
  8347: '星界洞悉',
  8410: '行近速率',
  8352: '时间扭曲补药',
  // 属性碎片
  5001: '生命值',
  5002: '护甲',
  5003: '魔抗',
  5005: '攻击速度',
  5007: '技能急速',
  5008: '自适应之力',
  5010: '移动速度',
  5011: '成长生命值',
  5012: '抗性',
  5013: '韧性'
}

export const PERK_ICON_PATH_MAP: Record<number, string> = {
  5001: 'v1/perk-images/statmods/statmodshealthplusicon.png',
  5002: 'v1/perk-images/statmods/statmodsarmoricon.png',
  5003: 'v1/perk-images/statmods/statmodsmagicresicon.png',
  5005: 'v1/perk-images/statmods/statmodsattackspeedicon.png',
  5007: 'v1/perk-images/statmods/statmodscdrscalingicon.png',
  5008: 'v1/perk-images/statmods/statmodsadaptiveforceicon.png',
  5010: 'v1/perk-images/statmods/statmodsmovementspeedicon.png',
  5011: 'v1/perk-images/statmods/statmodshealthscalingicon.png',
  5012: 'v1/perk-images/statmods/statmodsadaptiveforcescalingicon.png',
  5013: 'v1/perk-images/statmods/statmodstenacityicon.png',
  7000: 'v1/perk-images/template/7000.png',
  8004: 'v1/perk-images/styles/runesicon.png',
  8005: 'v1/perk-images/styles/precision/presstheattack/presstheattack.png',
  8006: 'v1/perk-images/styles/runesicon.png',
  8007: 'v1/perk-images/styles/runesicon.png',
  8008: 'v1/perk-images/styles/precision/lethaltempo/lethaltempotemp.png',
  8009: 'v1/perk-images/styles/precision/presenceofmind/presenceofmind.png',
  8010: 'v1/perk-images/styles/precision/conqueror/conqueror.png',
  8014: 'v1/perk-images/styles/precision/coupdegrace/coupdegrace.png',
  8016: 'v1/perk-images/styles/runesicon.png',
  8017: 'v1/perk-images/styles/precision/cutdown/cutdown.png',
  8021: 'v1/perk-images/styles/precision/fleetfootwork/fleetfootwork.png',
  8105: 'v1/perk-images/styles/domination/relentlesshunter/relentlesshunter.png',
  8106: 'v1/perk-images/styles/domination/ultimatehunter/ultimatehunter.png',
  8109: 'v1/perk-images/styles/runesicon.png',
  8112: 'v1/perk-images/styles/domination/electrocute/electrocute.png',
  8114: 'v1/perk-images/styles/runesicon.png',
  8115: 'v1/perk-images/styles/runesicon.png',
  8120: 'v1/perk-images/styles/domination/ghostporo/ghostporo.png',
  8124: 'v1/perk-images/styles/domination/predator/predator.png',
  8126: 'v1/perk-images/styles/domination/cheapshot/cheapshot.png',
  8127: 'v1/perk-images/styles/runesicon.png',
  8128: 'v1/perk-images/styles/domination/darkharvest/darkharvest.png',
  8134: 'v1/perk-images/styles/domination/ingenioushunter/ingenioushunter.png',
  8135: 'v1/perk-images/styles/domination/treasurehunter/treasurehunter.png',
  8136: 'v1/perk-images/styles/domination/zombieward/zombieward.png',
  8137: 'v1/perk-images/styles/domination/sixthsense/sixthsense.png',
  8138: 'v1/perk-images/styles/domination/eyeballcollection/eyeballcollection.png',
  8139: 'v1/perk-images/styles/domination/tasteofblood/greenterror_tasteofblood.png',
  8140: 'v1/perk-images/styles/domination/grislymementos/grislymementos.png',
  8141: 'v1/perk-images/styles/domination/deepward/deepward.png',
  8143: 'v1/perk-images/styles/domination/suddenimpact/suddenimpact.png',
  8205: 'v1/perk-images/styles/runesicon.png',
  8207: 'v1/perk-images/styles/runesicon.png',
  8208: 'v1/perk-images/styles/runesicon.png',
  8210: 'v1/perk-images/styles/sorcery/transcendence/transcendence.png',
  8214: 'v1/perk-images/styles/sorcery/summonaery/summonaery.png',
  8220: 'v1/perk-images/styles/runesicon.png',
  8224: 'v1/perk-images/styles/sorcery/nullifyingorb/axiom_arcanist.png',
  8226: 'v1/perk-images/styles/sorcery/manaflowband/manaflowband.png',
  8229: 'v1/perk-images/styles/sorcery/arcanecomet/arcanecomet.png',
  8230: 'v1/perk-images/styles/sorcery/phaserush/stormraiderssurgeruneicon2.png',
  8232: 'v1/perk-images/styles/sorcery/waterwalking/waterwalking.png',
  8233: 'v1/perk-images/styles/sorcery/absolutefocus/absolutefocus.png',
  8234: 'v1/perk-images/styles/sorcery/celerity/celeritytemp.png',
  8236: 'v1/perk-images/styles/sorcery/gatheringstorm/gatheringstorm.png',
  8237: 'v1/perk-images/styles/sorcery/scorch/scorch.png',
  8242: 'v1/perk-images/styles/sorcery/unflinching/unflinching.png',
  8275: 'v1/perk-images/styles/sorcery/nimbuscloak/6361.png',
  8299: 'v1/perk-images/styles/sorcery/laststand/laststand.png',
  8304: 'v1/perk-images/styles/inspiration/magicalfootwear/magicalfootwear.png',
  8306: 'v1/perk-images/styles/inspiration/hextechflashtraption/hextechflashtraption.png',
  8313: 'v1/perk-images/styles/inspiration/perfecttiming/alchemistcabinet.png',
  8316: 'v1/perk-images/styles/inspiration/jackofalltrades/jackofalltrades2.png',
  8318: 'v1/perk-images/styles/runesicon.png',
  8319: 'v1/perk-images/styles/runesicon.png',
  8320: 'v1/perk-images/styles/runesicon.png',
  8321: 'v1/perk-images/styles/inspiration/cashback/cashback2.png',
  8339: 'v1/perk-images/styles/inspiration/celestialbody/celestialbody.png',
  8344: 'v1/perk-images/styles/runesicon.png',
  8345: 'v1/perk-images/styles/inspiration/biscuitdelivery/biscuitdelivery.png',
  8347: 'v1/perk-images/styles/inspiration/cosmicinsight/cosmicinsight.png',
  8351: 'v1/perk-images/styles/inspiration/glacialaugment/glacialaugment.png',
  8352: 'v1/perk-images/styles/inspiration/timewarptonic/timewarptonic.png',
  8359: 'v1/perk-images/styles/inspiration/kleptomancy/kleptomancy.png',
  8360: 'v1/perk-images/styles/inspiration/unsealedspellbook/unsealedspellbook.png',
  8369: 'v1/perk-images/styles/inspiration/firststrike/firststrike.png',
  8401: 'v1/perk-images/styles/resolve/mirrorshell/mirrorshell.png',
  8410: 'v1/perk-images/styles/resolve/approachvelocity/approachvelocity.png',
  8414: 'v1/perk-images/styles/runesicon.png',
  8415: 'v1/perk-images/styles/runesicon.png',
  8416: 'v1/perk-images/styles/runesicon.png',
  8429: 'v1/perk-images/styles/resolve/conditioning/conditioning.png',
  8430: 'v1/perk-images/styles/resolve/ironskin/ironskin.png',
  8435: 'v1/perk-images/styles/resolve/mirrorshell/mirrorshell.png',
  8437: 'v1/perk-images/styles/resolve/graspoftheundying/graspoftheundying.png',
  8439: 'v1/perk-images/styles/resolve/veteranaftershock/veteranaftershock.png',
  8444: 'v1/perk-images/styles/resolve/secondwind/secondwind.png',
  8446: 'v1/perk-images/styles/resolve/demolish/demolish.png',
  8451: 'v1/perk-images/styles/resolve/overgrowth/overgrowth.png',
  8453: 'v1/perk-images/styles/resolve/revitalize/revitalize.png',
  8454: 'v1/perk-images/styles/runesicon.png',
  8463: 'v1/perk-images/styles/resolve/fontoflife/fontoflife.png',
  8465: 'v1/perk-images/styles/resolve/guardian/guardian.png',
  8472: 'v1/perk-images/styles/resolve/chrysalis/chrysalis.png',
  8473: 'v1/perk-images/styles/resolve/boneplating/boneplating.png',
  8992: 'v1/perk-images/styles/sorcery/deathfiretouch/deathfire_touch_keystone.png',
  9101: 'v1/perk-images/styles/precision/absorblife/absorblife.png',
  9103: 'v1/perk-images/styles/precision/legendbloodline/legendbloodline.png',
  9104: 'v1/perk-images/styles/precision/legendalacrity/legendalacrity.png',
  9105: 'v1/perk-images/styles/precision/legendhaste/legendhaste.png',
  9111: 'v1/perk-images/styles/precision/triumph.png',
  9923: 'v1/perk-images/styles/domination/hailofblades/hailofblades.png',
}

export function getPerkName(perkId: number): string {
  return PERK_NAME_MAP[perkId] || `符文${perkId}`
}

export function getPerkIconUrl(perkId: number): string {
  if (!perkId || perkId <= 0) return ''
  // 兼容主系/副系风格ID直接传入的情况
  if ([8000, 8100, 8200, 8300, 8400].includes(perkId)) {
    return getPerkStyleIconUrl(perkId)
  }
  const relPath = PERK_ICON_PATH_MAP[perkId]
  if (relPath) {
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/${relPath}`
  }
  return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/runesicon.png'
}

export function getPerkStyleIconUrl(styleId: number): string {
  switch (styleId) {
    case 8000:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/7201_precision.png'
    case 8100:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/7200_domination.png'
    case 8200:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/7202_sorcery.png'
    case 8400:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/7204_resolve.png'
    case 8300:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/7203_whimsy.png'
    default:
      return 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/runesicon.png'
  }
}

export const QUEUE_NAME_MAP: Record<number, string> = {
  420: '排位赛(单双)',
  440: '排位赛(灵活)',
  450: '极地大乱斗',
  400: '匹配模式(征召)',
  430: '匹配模式(自选)',
  490: '快速对局',
  1700: '斗魂竞技场',
  1710: '斗魂竞技场',
  1900: '无限火力',
  900: '无限乱斗',
  1090: '云顶之弈(匹配)',
  1100: '云顶之弈(排位)'
}

export function getQueueName(queueId: number): string {
  return QUEUE_NAME_MAP[queueId] || '匹配对局'
}

export function getMapName(mapId: number): string {
  switch (mapId) {
    case 11:
      return "召唤师峡谷"
    case 12:
      return "极地大乱斗"
    case 21:
      return "斗魂竞技场"
    default:
      return "召唤师峡谷"
  }
}
