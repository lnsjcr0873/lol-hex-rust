// HexBrain Vue 3 Live Full-Stack Engine (Professional Esports Edition)
const { createApp, ref, computed, onMounted, nextTick, watch } = Vue;

const app = createApp({
  setup() {
    const activeTab = ref('dashboard');
    const isLoading = ref(true);
    const clientConnected = ref(false);
    const wsConnected = ref(false);
    const gamePhase = ref('None');
    const liveBenchSlots = ref([]);
    const liveBenchRanked = ref([]);
    const diceCount = ref(2);
    const isDiceRolling = ref(false);

    const gamePhaseText = computed(() => {
      if (!clientConnected.value) return '等待游戏启动';
      switch (gamePhase.value) {
        case 'ChampSelect': return '英雄选择中';
        case 'InProgress': return '游戏中';
        case 'Lobby': return '大厅';
        case 'Matchmaking': return '匹配中';
        case 'ReadyCheck': return '就绪确认';
        case 'EndOfGame': return '对局结算';
        default: return '已连接';
      }
    });

    // Toast Notification System
    const toasts = ref([]);
    const showToast = (message, icon = '✨') => {
      const id = Date.now() + Math.random();
      toasts.value.push({ id, message, icon });
      setTimeout(() => {
        toasts.value = toasts.value.filter(t => t.id !== id);
      }, 3000);
    };

    // Meta Information
    const meta = ref({
      patch: '16.17',
      reportDate: '2026-08-29',
      heroCount: 64,
      augmentCount: 213,
      totalMatches: '12.8w',
      comboCount: '8.7k'
    });

    // Default pre-populated heroes (Figma Arcane Design 100% Match)
    const defaultHeroes = [
      { id: '142', name: '暮光星灵', title: '佐伊', tier: 1, winRate: 0.546, pickRate: 0.187, banRate: 0.123, games: 1870000, kda: 3.94, roles: ['法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/142.png' },
      { id: '136', name: '铸星龙王', title: '奥瑞利安·索尔', tier: 1, winRate: 0.532, pickRate: 0.152, banRate: 0.087, games: 1690000, kda: 3.21, roles: ['法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/136.png' },
      { id: '145', name: '虚空之女', title: '卡莎', tier: 1, winRate: 0.528, pickRate: 0.161, banRate: 0.041, games: 1790000, kda: 2.76, roles: ['射手', '法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/145.png' },
      { id: '202', name: '戏命师', title: '烬', tier: 2, winRate: 0.516, pickRate: 0.148, banRate: 0.029, games: 1480000, kda: 3.10, roles: ['射手'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/202.png' },
      { id: '80', name: '不屈之枪', title: '潘森', tier: 2, winRate: 0.513, pickRate: 0.123, banRate: 0.016, games: 1370000, kda: 2.65, roles: ['战士', '刺客'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/80.png' },
      { id: '89', name: '曙光女神', title: '蕾欧娜', tier: 2, winRate: 0.508, pickRate: 0.115, banRate: 0.012, games: 1280000, kda: 3.01, roles: ['坦克', '辅助'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/89.png' },
      { id: '157', name: '疾风剑豪', title: '亚索', tier: 1, winRate: 0.568, pickRate: 0.105, banRate: 0.095, games: 1169338, kda: 2.63, roles: ['战士', '刺客'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/157.png' },
      { id: '887', name: '灵罗娃娃', title: '格温', tier: 1, winRate: 0.567, pickRate: 0.040, banRate: 0.035, games: 446721, kda: 2.81, roles: ['战士', '法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/887.png' },
      { id: '876', name: '含羞蓓蕾', title: '莉莉娅', tier: 1, winRate: 0.562, pickRate: 0.089, banRate: 0.048, games: 992813, kda: 3.12, roles: ['战士', '法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/876.png' },
      { id: '777', name: '封魔剑魂', title: '永恩', tier: 1, winRate: 0.560, pickRate: 0.080, banRate: 0.072, games: 893120, kda: 2.54, roles: ['战士', '刺客'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/777.png' },
      { id: '4', name: '卡牌大师', title: '崔斯特', tier: 1, winRate: 0.559, pickRate: 0.089, banRate: 0.016, games: 993120, kda: 3.45, roles: ['法师'], avatarUrl: 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/4.png' }
    ];

    const allHeroes = ref(defaultHeroes);
    const allAugments = ref([]);
    const globalSearch = ref('');
    const showSearchDropdown = ref(false);
    const hotTags = ['皮城女警', '金克丝', '约德尔人', '投机者之心', '复苏之风'];

    // Ranking Filters
    const heroFilterRole = ref('全部');
    const heroSortBy = ref('tier');
    const heroSortAsc = ref(false);
    const rolesList = ['全部', '法师', '射手', '战士', '坦克', '刺客', '辅助'];

    // Selected Hero Profile
    const currentHeroId = ref('142');
    const heroDecisionSubTab = ref('海克斯推荐');
    const heroDetailData = ref(null);

    // Simulator State (3-Pick 100% Design Match)
    const simStage = ref('2-1');
    const simAugmentIds = ref(['1048', '1388', '1138']);
    const simChoices = ref([
      { id: '1048', slot: 'A', name: '珠光护手', rarity: '棱彩', score: 92, delta: '+4.6%', isDestiny: true, isTrap: false, alpha: '+4.6%', itemAdvice: '灭世者的死亡之帽 + 影焰', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/jeweledgauntlet_small.png', icon: '🌸', desc: '你的技能可以造成暴击。获得 额外暴击几率。' },
      { id: '1388', slot: 'B', name: '无限循环往复', rarity: '棱彩', score: 87, delta: '+3.8%', isDestiny: false, isTrap: false, alpha: '+3.8%', itemAdvice: '卢登的伙伴 + 虚空之杖', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/recursion_small.png', icon: '⚡', desc: '获得60技能急速，外加每次参与击杀3技能急速。' },
      { id: '1138', slot: 'C', name: '渴血', rarity: '白银', score: 71, delta: '+2.1%', isDestiny: false, isTrap: false, alpha: '+2.1%', itemAdvice: '中娅沙漏 + 峡谷制造者', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/vulnerability_small.png', icon: '🌱', desc: '获得15%全能吸血。' }
    ]);
    const simRationale = ref('首选【珠光护手】！为当前英雄带来高达 +4.6% 的净胜率暴增。');
    const simRerollAdvice = ref({
      action: 'LOCK',
      label: '👑 天命已至 · 立即锁定',
      desc: '首选【珠光护手】带来高达 +4.6% 净胜率暴增，强烈建议保留骰子！'
    });

    // Team Lab State (5-man composition)
    const teamHeroIds = ref(['142', '136', '145', '80', '89']);
    const teamSynergyData = ref({
      score: 92,
      evaluation: '极佳',
      synergyFit: '8 / 10',
      damageDist: { physical: 40, magic: 45, trueDmg: 15 },
      counterStats: { winAdvantage: 68, winDisadvantage: 32 },
      synergyPairs: [
        { heroA: { name: '暮光星灵' }, heroB: { name: '疾风剑豪' }, synergyDelta: 0.033, adjustedWinRate: 0.552, isGoldenPair: true },
        { heroA: { name: '暮光星灵' }, heroB: { name: '暴走萝莉' }, synergyDelta: 0.021, adjustedWinRate: 0.538, isGoldenPair: false },
        { heroA: { name: '暮光星灵' }, heroB: { name: '深海泰坦' }, synergyDelta: 0.028, adjustedWinRate: 0.545, isGoldenPair: true },
        { heroA: { name: '疾风剑豪' }, heroB: { name: '深海泰坦' }, synergyDelta: 0.045, adjustedWinRate: 0.568, isGoldenPair: true },
        { heroA: { name: '暴走萝莉' }, heroB: { name: '深海泰坦' }, synergyDelta: 0.031, adjustedWinRate: 0.549, isGoldenPair: true }
      ],
      topThreats: [
        { opponentName: '复仇焰魂', threatIndex: 6, reason: '大乱斗团战AOE高额爆发' },
        { opponentName: '蒸汽机器人', threatIndex: 5, reason: '先手抽奖强控破坏阵型' },
        { opponentName: '熔岩巨兽', threatIndex: 4, reason: '大招击飞强开团克制脆皮' },
        { opponentName: '虚空之眼', threatIndex: 4, reason: '超远距离真实伤害Poke' },
        { opponentName: '齐天大圣', threatIndex: 3, reason: '进场双重击飞与团控' }
      ]
    });

    // ---------------- Augment Icon Resolution & Image Safe Guard ----------------
    const getAugmentIconUrl = (aug) => {
      if (!aug) return 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png';
      if (aug.cdnIconUrl && typeof aug.cdnIconUrl === 'string' && aug.cdnIconUrl.startsWith('http') && !aug.cdnIconUrl.includes('undefined')) {
        return aug.cdnIconUrl;
      }
      const rawUrl = aug.augmentIconUrl || aug.iconUrl;
      if (rawUrl && typeof rawUrl === 'string') {
        const fn = rawUrl.split('/').pop();
        if (fn) return `https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/${fn.toLowerCase()}`;
      }
      const rarity = aug.rarity || '黄金';
      if (rarity === '棱彩') return 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png';
      if (rarity === '白银') return 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png';
      return 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutegold_small.png';
    };

    const handleAugmentImgError = (e, rarity = '黄金') => {
      if (!e || !e.target) return;
      if (e.target.dataset.failed) {
        e.target.style.opacity = '0.35';
        return;
      }
      e.target.dataset.failed = 'true';
      if (rarity === '棱彩') {
        e.target.src = 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png';
      } else if (rarity === '白银') {
        e.target.src = 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png';
      } else {
        e.target.src = 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutegold_small.png';
      }
    };

    const parsePctOrFloat = (val) => {
      if (val === null || val === undefined) return 0;
      if (typeof val === 'string' && val.includes('%')) {
        const num = parseFloat(val.replace('%', '').trim());
        return isNaN(num) ? 0 : num;
      }
      if (typeof val === 'number') {
        if (Math.abs(val) <= 1.0 && val !== 0) {
          return val * 100;
        }
        return val;
      }
      const num = parseFloat(String(val).trim());
      if (isNaN(num)) return 0;
      if (Math.abs(num) <= 1.0 && num !== 0) {
        return num * 100;
      }
      return num;
    };

    // Hero Augments Explorer State
    const dexActiveMode = ref('hero'); // 'hero' (当前英雄专属) or 'global' (全图鉴)
    const heroAugmentsTab = ref('全部'); // '全部', 'S级天命', 'A级强势', 'B级可用', '避雷陷阱'
    const heroAugmentSearch = ref('');
    const heroAugmentRarity = ref('全部');
    const heroAugmentSortBy = ref('deltaWinRate'); // 'deltaWinRate', 'pairWinRate', 'games', 'hexScore'
    const heroAugmentSortAsc = ref(false);
    const showHeroAugmentsModal = ref(false);

    // All evaluated augments for current hero (106+ items from 03_英雄海克斯表现.csv)
    const allHeroAugments = computed(() => {
      let list = [];
      if (heroDetailData.value && heroDetailData.value.augments) {
        const augs = heroDetailData.value.augments;
        if (Array.isArray(augs)) {
          augs.forEach(a => {
            if (!a) return;
            const item = typeof a === 'object' ? a : { augmentId: String(a), name: String(a) };
            const delta = parsePctOrFloat(item.deltaWinRate);
            const tier = delta >= 3.5 ? 'S' : delta >= 1.2 ? 'A' : delta < -0.5 ? 'TRAP' : 'B';
            const badgeClass = tier === 'S'
              ? 'bg-emerald-500/20 text-[#4ad1a0] border border-emerald-500/40'
              : tier === 'A'
                ? 'bg-blue-500/20 text-blue-300 border border-blue-500/40'
                : tier === 'TRAP'
                  ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40'
                  : 'bg-slate-700/40 text-slate-300 border border-slate-600/40';
            const tierName = tier === 'S' ? '👑 英雄天命 S级' : tier === 'A' ? '✨ 强势优选 A级' : tier === 'TRAP' ? '⚠️ 避雷陷阱' : '⚖️ 常规可用 B级';
            list.push({ ...item, cdnIconUrl: item.cdnIconUrl || getAugmentIconUrl(item), tierLevel: tier, tierName, tierBadgeClass: badgeClass });
          });
        } else if (typeof augs === 'object') {
          (augs.sTier || []).forEach(a => { if (a) list.push({ ...(typeof a === 'object' ? a : { augmentId: String(a) }), cdnIconUrl: (typeof a === 'object' && a.cdnIconUrl) || getAugmentIconUrl(a), tierLevel: 'S', tierName: '👑 英雄天命 S级', tierBadgeClass: 'bg-emerald-500/20 text-[#4ad1a0] border border-emerald-500/40' }); });
          (augs.aTier || []).forEach(a => { if (a) list.push({ ...(typeof a === 'object' ? a : { augmentId: String(a) }), cdnIconUrl: (typeof a === 'object' && a.cdnIconUrl) || getAugmentIconUrl(a), tierLevel: 'A', tierName: '✨ 强势优选 A级', tierBadgeClass: 'bg-blue-500/20 text-blue-300 border border-blue-500/40' }); });
          (augs.bTier || []).forEach(a => { if (a) list.push({ ...(typeof a === 'object' ? a : { augmentId: String(a) }), cdnIconUrl: (typeof a === 'object' && a.cdnIconUrl) || getAugmentIconUrl(a), tierLevel: 'B', tierName: '⚖️ 常规可用 B级', tierBadgeClass: 'bg-slate-700/40 text-slate-300 border border-slate-600/40' }); });
          (augs.traps || []).forEach(a => { if (a) list.push({ ...(typeof a === 'object' ? a : { augmentId: String(a) }), cdnIconUrl: (typeof a === 'object' && a.cdnIconUrl) || getAugmentIconUrl(a), tierLevel: 'TRAP', tierName: '⚠️ 避雷陷阱', tierBadgeClass: 'bg-rose-500/20 text-rose-400 border border-rose-500/40' }); });
        }
      } else {
        // High quality default fallback (03_英雄海克斯表现.csv)
        list = [
          { augmentId: '1048', name: '珠光护手', rarity: '棱彩', tierLevel: 'S', tierBadgeClass: 'bg-purple-500/20 text-purple-300 border border-purple-500/40', pairWinRate: 0.563, deltaWinRate: 0.046, pickRate: 0.189, games: 3200000, hexScore: 92, description: '你的技能可以造成暴击。获得 额外暴击几率。', stages: [{stage: 1, winRate: 0.55, deltaWinRate: 0.04}, {stage: 2, winRate: 0.56, deltaWinRate: 0.05}, {stage: 3, winRate: 0.57, deltaWinRate: 0.05}] },
          { augmentId: '1388', name: '无限循环往复', rarity: '棱彩', tierLevel: 'S', tierBadgeClass: 'bg-purple-500/20 text-purple-300 border border-purple-500/40', pairWinRate: 0.548, deltaWinRate: 0.038, pickRate: 0.162, games: 2800000, hexScore: 87, description: '获得60技能急速，外加每次参与击杀3技能急速。', stages: [{stage: 1, winRate: 0.54, deltaWinRate: 0.03}, {stage: 2, winRate: 0.55, deltaWinRate: 0.04}] },
          { augmentId: '1030', name: '尤里卡', rarity: '棱彩', tierLevel: 'A', tierBadgeClass: 'bg-blue-500/20 text-blue-300 border border-blue-500/40', pairWinRate: 0.537, deltaWinRate: 0.028, pickRate: 0.121, games: 1950000, hexScore: 81, description: '获得相当于30%法术强度的技能急速。' },
          { augmentId: '1138', name: '渴血', rarity: '白银', tierLevel: 'A', tierBadgeClass: 'bg-blue-500/20 text-blue-300 border border-blue-500/40', pairWinRate: 0.531, deltaWinRate: 0.021, pickRate: 0.145, games: 2400000, hexScore: 78, description: '获得15%全能吸血。' },
          { augmentId: '1038', name: '有始有终', rarity: '黄金', tierLevel: 'A', tierBadgeClass: 'bg-blue-500/20 text-blue-300 border border-blue-500/40', pairWinRate: 0.526, deltaWinRate: 0.016, pickRate: 0.098, games: 1600000, hexScore: 75, description: '获得先攻和黑暗收割基石符文。' },
          { augmentId: '1058', name: '秘术冲拳', rarity: '棱彩', tierLevel: 'TRAP', tierBadgeClass: 'bg-rose-500/20 text-rose-400 border border-rose-500/40', pairWinRate: 0.420, deltaWinRate: -0.053, pickRate: 0.014, games: 320000, hexScore: 32, description: '攻击特效使技能冷却时间缩减（与远程法师普攻机制冲突）。' },
          { augmentId: '1013', name: '星界躯体', rarity: '黄金', tierLevel: 'TRAP', tierBadgeClass: 'bg-rose-500/20 text-rose-400 border border-rose-500/40', pairWinRate: 0.455, deltaWinRate: -0.018, pickRate: 0.063, games: 980000, hexScore: 35, description: '获得1500生命值，但造成的伤害降低10%（严重削弱法师爆发）。' },
          { augmentId: '1206', name: '魔法转物理', rarity: '白银', tierLevel: 'TRAP', tierBadgeClass: 'bg-rose-500/20 text-rose-400 border border-rose-500/40', pairWinRate: 0.458, deltaWinRate: -0.015, pickRate: 0.042, games: 720000, hexScore: 38, description: '将法术强度转化为额外攻击力（损失AP法强收益）。' }
        ];
      }

      if (heroAugmentSearch.value.trim()) {
        const q = heroAugmentSearch.value.trim().toLowerCase();
        list = list.filter(a => (a.name && a.name.toLowerCase().includes(q)) || (a.description && a.description.toLowerCase().includes(q)));
      }
      if (heroAugmentsTab.value !== '全部') {
        list = list.filter(a => {
          if (heroAugmentsTab.value === 'S级天命') return a.tierLevel === 'S';
          if (heroAugmentsTab.value === 'A级强势') return a.tierLevel === 'A';
          if (heroAugmentsTab.value === 'B级可用') return a.tierLevel === 'B';
          if (heroAugmentsTab.value === '避雷陷阱') return a.tierLevel === 'TRAP';
          return true;
        });
      }
      if (heroAugmentRarity.value !== '全部') {
        list = list.filter(a => a.rarity === heroAugmentRarity.value);
      }

      list.sort((a, b) => {
        const key = heroAugmentSortBy.value;
        let valA = a[key];
        let valB = b[key];
        if (key === 'pairWinRate' || key === 'deltaWinRate' || key === 'pickRate') {
          valA = parsePctOrFloat(valA);
          valB = parsePctOrFloat(valB);
        } else {
          valA = valA || 0;
          valB = valB || 0;
        }
        return heroAugmentSortAsc.value ? valA - valB : valB - valA;
      });

      return list.map(a => ({
        ...a,
        cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a)
      }));
    });
    // Global Augment Dex State (209 Augments)
    const dexSearch = ref('');
    const dexRarity = ref('全部');
    const dexTierFilter = ref('全部');
    const dexSortBy = ref('winRate'); // 'winRate', 'pickRate', 'score', 'games'
    const dexSortAsc = ref(false);

    // Filtered Global Augments for View 4 (augmentDex)
    const filteredDexAugments = computed(() => {
      let list = (allAugments.value && allAugments.value.length > 0) ? [...allAugments.value] : [];
      if (list.length === 0) {
        list = [
          { id: '1134', name: '亮出你的剑', rarity: '棱彩', tier: 1, score: 92, delta: '+10.6%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png', icon: '👑', desc: '变为近战状态，并且获得攻击力、生命值、攻速、吸血与移速', winRate: '57.9%', pickRate: '4.1%', rawWinRate: 0.579, rawPickRate: 0.041, games: 489000 },
          { id: '2095', name: '掷骰狂人', rarity: '棱彩', tier: 1, score: 88, delta: '+2.5%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/highroller_small.png', icon: '🌸', desc: '附近敌人在阵亡时有几率掉落【属性锻造器】', winRate: '57.0%', pickRate: '1.2%', rawWinRate: 0.570, rawPickRate: 0.012, games: 4993528 },
          { id: '1058', name: '秘术冲拳', rarity: '棱彩', tier: 3, score: 45, delta: '-5.3%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png', icon: '🌱', desc: '攻击特效使你的各个技能的冷却时间缩减1.25秒', winRate: '42.0%', pickRate: '1.4%', rawWinRate: 0.420, rawPickRate: 0.014, games: 320000 },
          { id: '1048', name: '珠光护手', rarity: '棱彩', tier: 1, score: 86, delta: '+4.2%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/jeweledgauntlet_small.png', icon: '🌸', desc: '你的技能可以造成暴击。获得 额外暴击几率。', winRate: '56.3%', pickRate: '18.9%', rawWinRate: 0.563, rawPickRate: 0.189, games: 3200000 },
          { id: '1388', name: '无限循环往复', rarity: '棱彩', tier: 2, score: 82, delta: '+3.5%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/recursion_small.png', icon: '⚡', desc: '获得60技能急速，外加每次参与击杀3技能急速。', winRate: '54.8%', pickRate: '16.2%', rawWinRate: 0.548, rawPickRate: 0.162, games: 2800000 },
          { id: '1206', name: '魔法转物理', rarity: '白银', tier: 2, score: 78, delta: '+3.1%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png', icon: '🗡️', desc: '法术强度转化为额外攻击力', winRate: '53.2%', pickRate: '12.0%', rawWinRate: 0.532, rawPickRate: 0.120, games: 1900000 }
        ];
      }

      if (dexSearch.value.trim()) {
        const q = dexSearch.value.trim().toLowerCase();
        list = list.filter(a => (a.name && a.name.toLowerCase().includes(q)) || (a.desc && a.desc.toLowerCase().includes(q)));
      }
      if (dexRarity.value !== '全部') {
        list = list.filter(a => a.rarity === dexRarity.value);
      }
      if (dexTierFilter.value !== '全部') {
        const tNum = parseInt(dexTierFilter.value.replace('T', ''));
        list = list.filter(a => a.tier === tNum);
      }

      list.sort((a, b) => {
        let valA = a.score || 0;
        let valB = b.score || 0;
        if (dexSortBy.value === 'winRate') {
          valA = parsePctOrFloat(a.winRate !== undefined ? a.winRate : a.rawWinRate);
          valB = parsePctOrFloat(b.winRate !== undefined ? b.winRate : b.rawWinRate);
        } else if (dexSortBy.value === 'pickRate') {
          valA = parsePctOrFloat(a.pickRate !== undefined ? a.pickRate : a.rawPickRate);
          valB = parsePctOrFloat(b.pickRate !== undefined ? b.pickRate : b.rawPickRate);
        } else if (dexSortBy.value === 'deltaWinRate' || dexSortBy.value === 'delta') {
          valA = parsePctOrFloat(a.delta !== undefined ? a.delta : (a.avgDeltaWinRate || a.deltaWinRate || 0));
          valB = parsePctOrFloat(b.delta !== undefined ? b.delta : (b.avgDeltaWinRate || b.deltaWinRate || 0));
        } else if (dexSortBy.value === 'games') {
          valA = a.games || 0;
          valB = b.games || 0;
        } else if (dexSortBy.value === 'score') {
          valA = a.score !== undefined ? a.score : (a.globalHexScore || 0);
          valB = b.score !== undefined ? b.score : (b.globalHexScore || 0);
        }
        return dexSortAsc.value ? valA - valB : valB - valA;
      });

      return list.map(a => ({
        ...a,
        cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a)
      }));
    });

    const selectedDexAugment = ref({
      id: '1048',
      name: '珠光护手',
      rarity: '棱彩',
      tier: 1,
      cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/jeweledgauntlet_small.png',
      description: '你的技能可以造成暴击。获得 额外暴击几率。',
      winRate: '56.3%',
      pickRate: '18.9%',
      avgDeltaWinRate: 0.046,
      topHeroes: []
    });

    // Modals
    const showFullRankModal = ref(false);
    const showFullAugmentsModal = ref(false);
    const showSimDetailModal = ref(false);
    const showTeamDetailModal = ref(false);
    const showDexDetailModal = ref(false);
    
    // Live Swap Modals
    const showSwapAugmentModal = ref(false);
    const currentSwapSlot = ref(0);
    const swapAugmentSearch = ref('');

    const showSwapChampionModal = ref(false);
    const currentSwapTeamSlot = ref(0);
    const swapChampionSearch = ref('');

    // Computed: Search Suggestions
    const searchSuggestions = computed(() => {
      const q = globalSearch.value.trim().toLowerCase();
      if (!q) return [];
      const heroes = allHeroes.value || defaultHeroes;
      return heroes.filter(h => 
        (h.name && h.name.toLowerCase().includes(q)) || 
        (h.title && h.title.toLowerCase().includes(q)) ||
        (Array.isArray(h.searchTerms) && h.searchTerms.some(st => String(st).toLowerCase().includes(q)))
      ).slice(0, 5);
    });

    // Computed: Filtered Heroes in Rankings
    const filteredRankHeroes = computed(() => {
      let list = (allHeroes.value && allHeroes.value.length > 0) ? [...allHeroes.value] : [...defaultHeroes];
      if (globalSearch.value && globalSearch.value.trim()) {
        const q = globalSearch.value.trim().toLowerCase();
        list = list.filter(h => 
          (h.name && h.name.toLowerCase().includes(q)) || 
          (h.title && h.title.toLowerCase().includes(q)) ||
          (Array.isArray(h.searchTerms) && h.searchTerms.some(st => String(st).toLowerCase().includes(q)))
        );
      }
      if (heroFilterRole.value && heroFilterRole.value !== '全部') {
        list = list.filter(h => {
          const roles = h.roles || [];
          return roles.includes(heroFilterRole.value);
        });
      }
      list.sort((a, b) => {
        if (heroSortBy.value === 'tier') {
          const tierA = a.tier || 5;
          const tierB = b.tier || 5;
          if (tierA !== tierB) {
            return heroSortAsc.value ? tierB - tierA : tierA - tierB;
          }
          return (b.winRate || 0) - (a.winRate || 0);
        }
        const valA = a[heroSortBy.value] || 0;
        const valB = b[heroSortBy.value] || 0;
        return heroSortAsc.value ? valA - valB : valB - valA;
      });
      return list;
    });

    const topRankHeroes = computed(() => {
      return filteredRankHeroes.value.slice(0, 5);
    });

    const currentHero = computed(() => {
      const heroes = (allHeroes.value && allHeroes.value.length > 0) ? allHeroes.value : defaultHeroes;
      const found = heroes.find(h => String(h.id) === String(currentHeroId.value));
      return found || heroes[0] || defaultHeroes[0];
    });

    const teamHeroList = computed(() => {
      const heroes = (allHeroes.value && allHeroes.value.length > 0) ? allHeroes.value : defaultHeroes;
      return teamHeroIds.value.map(id => {
        const h = heroes.find(hero => String(hero.id) === String(id));
        return h || {
          id: id,
          name: '英雄',
          title: '英雄',
          avatarUrl: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${id}.png`,
          roles: ['法师']
        };
      });
    });

    // Items View State (04_装备表现 + 08_三件套 + 09_鞋子)
    const itemSubTab = ref('trios'); // 'trios', 'single', 'shoes'
    const itemSearch = ref('');
    const trioTierFilter = ref('全部'); // '全部', 'T1', 'T2', 'T3'
    const trioDeltaFilter = ref('全部'); // '全部', 'positive', 'high'
    const trioMinGames = ref(0);
    const trioSortBy = ref('winRate'); // 'name', 'winRate', 'deltaWinRate', 'pickRate', 'games', 'tier'
    const trioSortAsc = ref(false);

    const singleTierFilter = ref('全部');
    const singleDeltaFilter = ref('全部');
    const singleItemSortBy = ref('deltaWinRate'); // 'name', 'averageIndex', 'winRate', 'deltaWinRate', 'pickRate', 'hexScore', 'games', 'tier'
    const singleItemSortAsc = ref(false);

    const shoesSortBy = ref('winRate');
    const shoesSortAsc = ref(false);

    const toggleTrioSort = (field) => {
      if (trioSortBy.value === field) {
        trioSortAsc.value = !trioSortAsc.value;
      } else {
        trioSortBy.value = field;
        trioSortAsc.value = (field === 'tier' || field === 'name') ? true : false;
      }
      showToast(`三件套已按【${field === 'winRate' ? '终局胜率' : field === 'deltaWinRate' ? '净胜率加成' : field === 'pickRate' ? '选择率' : field === 'games' ? '实战场次' : field === 'tier' ? '评级' : '装备名称'}】${trioSortAsc.value ? '升序 ▲' : '降序 ▼'}排列`, '⚔️');
    };

    const toggleSingleSort = (field) => {
      if (singleItemSortBy.value === field) {
        singleItemSortAsc.value = !singleItemSortAsc.value;
      } else {
        singleItemSortBy.value = field;
        singleItemSortAsc.value = (field === 'averageIndex' || field === 'name' || field === 'tier') ? true : false;
      }
      showToast(`单件装备已按【${field === 'winRate' ? '搭配胜率' : field === 'deltaWinRate' ? '净加成' : field === 'averageIndex' ? '出装均序' : field === 'hexScore' ? 'HexScore' : field === 'games' ? '场次' : field === 'tier' ? '评级' : '装备名称'}】${singleItemSortAsc.value ? '升序 ▲' : '降序 ▼'}排列`, '🗡️');
    };

    const toggleShoesSort = (field) => {
      if (shoesSortBy.value === field) {
        shoesSortAsc.value = !shoesSortAsc.value;
      } else {
        shoesSortBy.value = field;
        shoesSortAsc.value = (field === 'name') ? true : false;
      }
    };

    // All Terminal Trios for current hero (from 08_英雄终局三件套.csv - up to 285 builds)
    const allHeroTrios = computed(() => {
      if (heroDetailData.value && heroDetailData.value.terminalTrios) {
        let list = [...heroDetailData.value.terminalTrios];
        // 1. Text search
        if (itemSearch.value.trim()) {
          const q = itemSearch.value.trim().toLowerCase();
          list = list.filter(t => (t.itemNames || []).some(n => n.toLowerCase().includes(q)));
        }
        // 2. Tier filter
        if (trioTierFilter.value !== '全部') {
          const targetTier = parseInt(trioTierFilter.value.replace('T', ''));
          list = list.filter(t => (t.tier || 1) === targetTier);
        }
        // 3. Delta winrate filter
        if (trioDeltaFilter.value === 'positive') {
          list = list.filter(t => (t.deltaWinRate || 0) > 0);
        } else if (trioDeltaFilter.value === 'high') {
          list = list.filter(t => (t.deltaWinRate || 0) >= 0.03);
        }
        // 4. Min games filter
        if (trioMinGames.value > 0) {
          list = list.filter(t => (t.games || 0) >= trioMinGames.value);
        }
        // 5. Sorting
        list.sort((a, b) => {
          let valA = 0, valB = 0;
          if (trioSortBy.value === 'name') {
            valA = (a.itemNames || []).join('');
            valB = (b.itemNames || []).join('');
            return trioSortAsc.value ? String(valA).localeCompare(String(valB)) : String(valB).localeCompare(String(valA));
          } else if (trioSortBy.value === 'tier') {
            valA = a.tier || 5;
            valB = b.tier || 5;
          } else {
            valA = a[trioSortBy.value] || 0;
            valB = b[trioSortBy.value] || 0;
          }
          return trioSortAsc.value ? valA - valB : valB - valA;
        });
        return list;
      }
      return [];
    });

    // All Single Items for current hero (from 04_英雄装备表现.csv - up to 109 items)
    const allHeroSingleItems = computed(() => {
      if (heroDetailData.value && heroDetailData.value.items) {
        const itemsObj = heroDetailData.value.items;
        let list = (itemsObj.allItems && itemsObj.allItems.length > 0)
          ? [...itemsObj.allItems]
          : (itemsObj.coreItems && itemsObj.coreItems.length > 0)
            ? [...itemsObj.coreItems]
            : Array.isArray(itemsObj) ? [...itemsObj] : [];
        // 1. Text search
        if (itemSearch.value.trim()) {
          const q = itemSearch.value.trim().toLowerCase();
          list = list.filter(it => (it.itemName && it.itemName.toLowerCase().includes(q)) || (it.name && it.name.toLowerCase().includes(q)));
        }
        // 2. Tier filter
        if (singleTierFilter.value !== '全部') {
          const targetTier = parseInt(singleTierFilter.value.replace('T', ''));
          list = list.filter(it => (it.tier || 1) === targetTier);
        }
        // 3. Delta winrate filter
        if (singleDeltaFilter.value === 'positive') {
          list = list.filter(it => (it.deltaWinRate || it.coreDelta || 0) > 0);
        } else if (singleDeltaFilter.value === 'high') {
          list = list.filter(it => (it.deltaWinRate || it.coreDelta || 0) >= 0.03);
        }
        // 4. Sorting
        list.sort((a, b) => {
          let valA = 0, valB = 0;
          if (singleItemSortBy.value === 'name') {
            valA = a.itemName || a.name || '';
            valB = b.itemName || b.name || '';
            return singleItemSortAsc.value ? String(valA).localeCompare(String(valB)) : String(valB).localeCompare(String(valA));
          } else if (singleItemSortBy.value === 'tier') {
            valA = a.tier || 5;
            valB = b.tier || 5;
          } else if (singleItemSortBy.value === 'deltaWinRate') {
            valA = a.coreDelta !== undefined ? a.coreDelta : (a.deltaWinRate || 0);
            valB = b.coreDelta !== undefined ? b.coreDelta : (b.deltaWinRate || 0);
          } else {
            valA = a[singleItemSortBy.value] || 0;
            valB = b[singleItemSortBy.value] || 0;
          }
          return singleItemSortAsc.value ? valA - valB : valB - valA;
        });
        return list;
      }
      return [];
    });

    // Favorites State
    const savedFavorites = ref([
      { id: '142', name: '暮光星灵', title: '星灵少女', winRate: '47.3%', tag: 'AP核弹流', coreAugment: '亮出你的剑 + 珠光护手', coreItems: '灭世者之帽 + 影焰 + 卢登的伙伴' },
      { id: '157', name: '疾风剑豪', title: '快乐风男', winRate: '51.8%', tag: '暴击飓风流', coreAugment: '闪电打击 + 亮出你的剑', coreItems: '破败王者之刃 + 无尽之刃 + 焚天' },
      { id: '222', name: '暴走萝莉', title: '罪恶快感', winRate: '53.2%', tag: '超频收割流', coreAugment: '超频攻速 + 巨兽杀手', coreItems: '海妖杀手 + 卢安娜的飓风 + 无尽之刃' }
    ]);

    // Dynamic Team Calculation
    const dynamicTeamData = computed(() => {
      const data = teamSynergyData.value || {};
      const dd = data.damageDist || {};
      return {
        score: data.score || 92,
        evaluation: data.evaluation || '极佳',
        synergyFit: data.synergyFit || '8 / 10',
        damageDist: {
          physical: dd.physical !== undefined ? dd.physical : 40,
          magic: dd.magic !== undefined ? dd.magic : 45,
          trueDmg: dd.trueDmg !== undefined ? dd.trueDmg : 15
        },
        counterStats: data.counterStats || { winAdvantage: 68, winDisadvantage: 32 }
      };
    });

    // Dice ROI Calculation
    const rerollEvaluation = computed(() => {
      const hero = currentHero.value || defaultHeroes[0];
      const wr = hero.winRate || 0.5;
      const tier = hero.tier || 2;
      const count = diceCount.value || 2;
      
      let roi = (2.38 * (count / 2)).toFixed(2);
      let label = '极度推荐重随';
      let desc = `基于当前T级和胜率，重随期望收益极高`;
      let thumbLeft = '88%';

      if (tier === 1 && wr >= 0.53) {
        roi = `+${roi}`;
        label = '极度推荐重随';
        desc = `持有 ${count} 颗骰子，可冲刺顶级专属天命强化`;
        thumbLeft = '90%';
      } else if (tier <= 2) {
        roi = `+${(1.42 * (count / 2)).toFixed(2)}`;
        label = '推荐重随';
        desc = `有较高概率摇出 T1 核心海克斯与英雄`;
        thumbLeft = '68%';
      } else {
        roi = `+${(0.65 * (count / 2)).toFixed(2)}`;
        label = '建议重随';
        desc = `当前英雄梯队偏弱，建议优先重摇`;
        thumbLeft = '45%';
      }

      return { roi, label, desc, thumbLeft };
    });

    // 1. Recommended Augments (Top 5 from sTier/aTier/bTier)
    const heroAugmentsRecommend = computed(() => {
      if (heroDetailData.value && heroDetailData.value.augments) {
        const augs = heroDetailData.value.augments;
        let list = [];
        if (Array.isArray(augs)) {
          list = augs;
        } else if (typeof augs === 'object') {
          list = [
            ...(augs.sTier || []),
            ...(augs.aTier || []),
            ...(augs.bTier || [])
          ];
        }
        if (list.length > 0) {
          return list.slice(0, 5).map(a => {
            const pwr = parsePctOrFloat(a.pairWinRate !== undefined ? a.pairWinRate : 0.563);
            const pkr = parsePctOrFloat(a.pickRate !== undefined ? a.pickRate : 0.189);
            const dwr = parsePctOrFloat(a.deltaWinRate !== undefined ? a.deltaWinRate : (a.delta !== undefined ? a.delta : 0.042));
            return {
              id: String(a.augmentId || a.id || ''),
              name: a.name || a.augmentName || '海克斯强化',
              rarity: a.rarity || '黄金',
              cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a),
              winRate: `${pwr.toFixed(1)}%`,
              pickRate: `${pkr.toFixed(1)}%`,
              delta: `${dwr >= 0 ? '+' : ''}${dwr.toFixed(1)}%`,
              score: a.hexScore || 85
            };
          });
        }
      }
      return [
        { id: '1048', name: '珠光护手', rarity: '棱彩', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/jeweledgauntlet_small.png', winRate: '56.3%', pickRate: '18.9%', delta: '+4.2%', score: 88 },
        { id: '1388', name: '无限循环往复', rarity: '棱彩', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/recursion_small.png', winRate: '54.8%', pickRate: '16.2%', delta: '+3.5%', score: 82 },
        { id: '1030', name: '尤里卡', rarity: '棱彩', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/eureka_small.png', winRate: '53.7%', pickRate: '12.1%', delta: '+2.8%', score: 79 },
        { id: '1138', name: '渴血', rarity: '白银', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/vulnerability_small.png', winRate: '53.1%', pickRate: '14.5%', delta: '+2.1%', score: 76 },
        { id: '1038', name: '有始有终', rarity: '黄金', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutegold_small.png', winRate: '52.6%', pickRate: '9.8%', delta: '+1.6%', score: 74 }
      ];
    });

    // 2. Avoid Augments (from traps)
    const heroAugmentsAvoid = computed(() => {
      if (heroDetailData.value && heroDetailData.value.augments) {
        const augs = heroDetailData.value.augments;
        let list = [];
        if (typeof augs === 'object' && !Array.isArray(augs) && augs.traps) {
          list = augs.traps;
        } else if (Array.isArray(augs)) {
          list = augs.filter(a => parsePctOrFloat(a.deltaWinRate) < 0);
        }
        if (list.length > 0) {
          return list.slice(0, 3).map(a => {
            const dwr = parsePctOrFloat(a.deltaWinRate !== undefined ? a.deltaWinRate : -0.02);
            const pkr = parsePctOrFloat(a.pickRate !== undefined ? a.pickRate : 0.087);
            return {
              id: String(a.augmentId || a.id || ''),
              name: a.name || a.augmentName || '避雷强化',
              rarity: a.rarity || '白银',
              cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a),
              deltaWinRate: `${dwr >= 0 ? '+' : ''}${dwr.toFixed(1)}%`,
              pickRate: `${pkr.toFixed(1)}%`
            };
          });
        }
      }
      return [
        { id: '101', name: '狩猎律动 II', rarity: '白银', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png', deltaWinRate: '-2.1%', pickRate: '8.7%' },
        { id: '102', name: '战斗法师', rarity: '黄金', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutegold_small.png', deltaWinRate: '-1.8%', pickRate: '6.3%' },
        { id: '103', name: '短兵之利 II', rarity: '白银', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png', deltaWinRate: '-1.5%', pickRate: '4.2%' }
      ];
    });

    // 3. Core Items & Boots (from 08_英雄终局三件套.csv & 09_英雄鞋子胜率.csv)
    const heroCoreItems = computed(() => {
      if (heroDetailData.value && heroDetailData.value.terminalTrios && heroDetailData.value.terminalTrios.length > 0) {
        return heroDetailData.value.terminalTrios.slice(0, 3).map(t => ({
          names: (t.itemNames || []).join(' + ') || '核心神装',
          winRate: `${parsePctOrFloat(t.winRate || 0.55).toFixed(1)}%`,
          pickRate: `${parsePctOrFloat(t.pickRate || 0.3).toFixed(1)}%`,
          games: t.games || 71000
        }));
      }
      return [
        { names: '灭世者的死亡之帽 + 影焰 + 卢登的回声', winRate: '45.3%', pickRate: '22.2%', games: 71040 },
        { names: '卢登的激荡 + 影焰 + 虚空之杖', winRate: '46.1%', pickRate: '18.4%', games: 58900 },
        { names: '时光之杖 + 大天使之杖 + 灭世者之帽', winRate: '44.8%', pickRate: '12.6%', games: 40200 }
      ];
    });

    // Best Boots from 09_英雄鞋子胜率.csv
    const heroBoots = computed(() => {
      if (heroDetailData.value && heroDetailData.value.shoes) {
        const shoes = heroDetailData.value.shoes;
        const list = (shoes.bootsRank || []).map(b => {
          const deltaVal = parsePctOrFloat((b.coreDelta !== undefined && b.coreDelta !== null) ? b.coreDelta : (b.deltaWinRate || 0));
          const deltaSign = deltaVal >= 0 ? "+" : "";
          const icon = b.cdnIconUrl || b.itemImageUrl || (b.itemId ? `https://ddragon.leagueoflegends.com/cdn/14.16.1/img/item/${b.itemId}.png` : '');
          return {
            itemId: b.itemId,
            name: b.itemName || '鞋子',
            winRate: `${parsePctOrFloat(b.winRate || 0.5).toFixed(1)}%`,
            pickRate: `${parsePctOrFloat(b.pickRate || 0.3).toFixed(1)}%`,
            delta: `${deltaSign}${deltaVal.toFixed(1)}%`,
            hexLabel: b.hexLabel || '推荐',
            hexScore: b.hexScore || 70,
            cdnIconUrl: icon
          };
        });
        const advice = shoes.sellBootsAdvice || '大乱斗18级金币充裕时，可卖鞋换第六件神装';
        return {
          list,
          advice,
          sellBootsAdvice: advice
        };
      }
      return {
        list: [
          { itemId: '3020', name: '法师之靴', winRate: '46.7%', pickRate: '57.2%', delta: '-0.6%', hexLabel: '主流', hexScore: 59, cdnIconUrl: 'https://ddragon.leagueoflegends.com/cdn/14.16.1/img/item/3020.png' },
          { itemId: '3158', name: '明朗之靴', winRate: '50.9%', pickRate: '6.8%', delta: '+3.6%', hexLabel: '顶级', hexScore: 82.5, cdnIconUrl: 'https://ddragon.leagueoflegends.com/cdn/14.16.1/img/item/3158.png' },
          { itemId: '3009', name: '轻灵之靴', winRate: '49.5%', pickRate: '0.4%', delta: '+2.2%', hexLabel: '人上人', hexScore: 68.5, cdnIconUrl: 'https://ddragon.leagueoflegends.com/cdn/14.16.1/img/item/3009.png' }
        ],
        advice: '大乱斗18级金币充裕时，可卖鞋换第六件神装',
        sellBootsAdvice: '大乱斗18级金币充裕时，可卖鞋换第六件神装'
      };
    });

    // 4. Summoner Spells (from 07_英雄召唤师技能.csv)
    const heroSummonerSpells = computed(() => {
      if (heroDetailData.value && heroDetailData.value.summonerSpells && heroDetailData.value.summonerSpells.length > 0) {
        return heroDetailData.value.summonerSpells.slice(0, 2).map(s => ({
          names: (s.spellNames || []).join(' + ') || '闪现 + 标记',
          desc: (s.spellNames || []).includes('标记') ? '雪球突进爆发流' : (s.spellNames || []).includes('疾跑') ? '极致拉扯流' : '常规爆发流',
          winRate: `${parsePctOrFloat(s.winRate || 0.54).toFixed(1)}%`,
          pickRate: `${parsePctOrFloat(s.pickRate || 0.5).toFixed(1)}%`,
          games: s.games || 271300
        }));
      }
      return [
        { names: '闪现 + 标记', desc: '雪球突进爆发流', winRate: '45.6%', pickRate: '84.6%', games: 271300 },
        { names: '闪现 + 疾跑', desc: '极致拉扯流', winRate: '47.2%', pickRate: '34.8%', games: 72278 }
      ];
    });

    // 5. Counter & Teammates (from 06_英雄关系_克制与队友协同.csv)
    const heroMatchups = computed(() => {
      const rels = (heroDetailData.value && heroDetailData.value.relationships) || {};
      const strong = (rels.strongAgainst || []).slice(0, 2).map(s => {
        const wr = parsePctOrFloat(s.adjustedWinRate || 0.6);
        const cd = parsePctOrFloat(s.counterDelta || 0.03);
        return {
          name: s.opponentName || '对手',
          winRate: `${wr.toFixed(1)}%`,
          delta: `${cd >= 0 ? '+' : ''}${cd.toFixed(1)}%`
        };
      });
      const weak = (rels.weakAgainst || []).slice(0, 2).map(w => {
        const wr = parsePctOrFloat(w.adjustedWinRate || 0.4);
        const cd = parsePctOrFloat(w.counterDelta || -0.04);
        return {
          name: w.opponentName || '对手',
          winRate: `${wr.toFixed(1)}%`,
          delta: `${cd >= 0 ? '+' : ''}${cd.toFixed(1)}%`
        };
      });
      const teammates = (rels.teammates || []).slice(0, 3).map(t => {
        const wr = parsePctOrFloat(t.adjustedWinRate || 0.58);
        const syn = parsePctOrFloat(t.synergyDelta || 0.03);
        return {
          name: t.teammateName || '队友',
          winRate: `${wr.toFixed(1)}%`,
          synergy: `${syn >= 0 ? '+' : ''}${syn.toFixed(1)}%`
        };
      });

      return { strong, weak, teammates };
    });

    // 6. Combat Radar & Post-Match Deep Stats (from 05_英雄赛后表现.csv)
    const combatStats = computed(() => {
      if (heroDetailData.value && heroDetailData.value.combatRadar) {
        const radar = heroDetailData.value.combatRadar;
        const raw = radar.raw || {};
        return {
          radar: {
            damage: radar.damage || 67,
            survivability: radar.survivability || 69,
            utility: radar.utility || 30,
            economy: radar.economy || 80,
            kp: radar.kp || 93
          },
          kda: (raw.kda || 3.94).toFixed(2),
          avgKills: (raw.avgKills || 9.8).toFixed(1),
          avgDeaths: (raw.avgDeaths || 8.9).toFixed(1),
          avgAssists: (raw.avgAssists || 25.4).toFixed(1),
          avgDamage: Math.round(raw.avgDamage || 38182).toLocaleString(),
          damageShare: `${((raw.damageShare || 0.201) * 100).toFixed(1)}%`,
          avgDamageTaken: Math.round(raw.avgDamageTaken || 23651).toLocaleString(),
          avgDmgMitigated: Math.round(raw.avgDmgMitigated || 14030).toLocaleString(),
          avgGold: Math.round(raw.avgGold || 15799).toLocaleString(),
          avgCs: (raw.avgCs || 32.4).toFixed(1),
          killParticipation: `${((raw.killParticipation || 0.798) * 100).toFixed(1)}%`,
          goldEfficiency: (raw.goldEfficiency || 2.42).toFixed(2),
          avgCcTime: `${(raw.avgCcTime || 39.1).toFixed(1)}s`,
          avgTurretDmg: Math.round(raw.avgTurretDmg || 1504).toLocaleString(),
          pentaKills: (raw.pentaKills || 434).toLocaleString(),
          quadraKills: (raw.quadraKills || 3920).toLocaleString(),
          tripleKills: (raw.tripleKills || 32390).toLocaleString(),
          doubleKills: (raw.doubleKills || 231369).toLocaleString()
        };
      }
      return {
        radar: { damage: 67, survivability: 69, utility: 30, economy: 80, kp: 93 },
        kda: '3.94',
        avgKills: '9.9',
        avgDeaths: '9.0',
        avgAssists: '25.4',
        avgDamage: '38,182',
        damageShare: '20.1%',
        avgDamageTaken: '23,651',
        avgDmgMitigated: '14,030',
        avgGold: '15,799',
        avgCs: '32.4',
        killParticipation: '79.8%',
        goldEfficiency: '2.42',
        avgCcTime: '39.1s',
        avgTurretDmg: '1,504',
        pentaKills: '434',
        quadraKills: '3,920',
        tripleKills: '32,390',
        doubleKills: '231,369'
      };
    });

    // Filtered Augments for Swap Modal
    const filteredAugmentsForSwap = computed(() => {
      let list = (allAugments.value && allAugments.value.length > 0) ? allAugments.value : [
        { id: '1134', name: '亮出你的剑', rarity: '棱彩', score: 82, delta: '+10.6%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png', icon: '👑', desc: '变为近战状态并获得高额攻击属性', winRate: '57.9%', pickRate: '4.1%' },
        { id: '2095', name: '掷骰狂人', rarity: '棱彩', score: 60, delta: '+2.5%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/highroller_small.png', icon: '🌸', desc: '敌方阵亡掉落属性锻造器', winRate: '49.9%', pickRate: '8.2%' },
        { id: '1058', name: '秘术冲拳', rarity: '棱彩', score: 29, delta: '-5.3%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmuteprismatic_small.png', icon: '🌱', desc: '攻击特效缩减技能冷却时间', winRate: '42.0%', pickRate: '1.4%' },
        { id: '1206', name: '魔法转物理', rarity: '白银', score: 75, delta: '+3.1%', cdnIconUrl: 'https://raw.communitydragon.org/latest/game/assets/ux/cherry/augments/icons/transmutesilver_small.png', icon: '⚡', desc: '法术强度转化为额外攻击力', winRate: '53.2%', pickRate: '12.0%' }
      ];
      if (dexSearch.value.trim() || swapAugmentSearch.value.trim()) {
        const q = (dexSearch.value.trim() || swapAugmentSearch.value.trim()).toLowerCase();
        list = list.filter(a => (a.name && a.name.toLowerCase().includes(q)) || (a.desc && a.desc.toLowerCase().includes(q)));
      }
      if (dexRarity.value && dexRarity.value !== '全部') {
        list = list.filter(a => a.rarity === dexRarity.value);
      }
      return list.map(a => ({
        ...a,
        cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a)
      }));
    });

    // Filtered Champions for Team Lab Swap Modal
    const filteredChampionsForSwap = computed(() => {
      let list = (allHeroes.value && allHeroes.value.length > 0) ? [...allHeroes.value] : [...defaultHeroes];
      const q = swapChampionSearch.value.trim().toLowerCase();
      if (q) {
        list = list.filter(h => 
          (h.name && h.name.toLowerCase().includes(q)) || 
          (h.title && h.title.toLowerCase().includes(q)) ||
          (Array.isArray(h.searchTerms) && h.searchTerms.some(st => String(st).toLowerCase().includes(q)))
        );
      }
      return list;
    });

    // ---------------- Actions & Methods ----------------
    const selectHero = (heroId) => {
      currentHeroId.value = String(heroId);
      showSearchDropdown.value = false;
      globalSearch.value = '';
      fetchHeroDetail(heroId);
      const hero = allHeroes.value.find(h => String(h.id) === String(heroId));
      if (hero) showToast(`已切换至【${hero.name}】· 决策画像与推演已重算`, '🎯');
    };

    const applySearchTag = (tag) => {
      globalSearch.value = tag;
      const heroes = allHeroes.value || defaultHeroes;
      const found = heroes.find(h => (h.name && h.name.includes(tag)) || (h.title && h.title.includes(tag)));
      if (found) selectHero(found.id);
    };

    const handleSearchEnter = () => {
      const q = globalSearch.value.trim().toLowerCase();
      if (!q) return;
      const heroes = allHeroes.value || defaultHeroes;
      const found = heroes.find(h => (h.name && h.name.toLowerCase().includes(q)) || (h.title && h.title.toLowerCase().includes(q)));
      if (found) {
        selectHero(found.id);
      } else {
        activeTab.value = 'tierList';
      }
      showSearchDropdown.value = false;
    };

    const triggerOCRScan = async () => {
      try {
        showToast('正在实时分析游戏屏幕与海克斯三选一...', '📷');
        const stageNum = simStage.value === '2-1' ? 1 : simStage.value === '3-2' ? 2 : 3;
        const res = await fetch('/api/ocr/scan-and-compare', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            heroId: String(currentHeroId.value),
            currentStage: stageNum
          })
        }).then(r => r.json());

        if (res && res.candidates) {
          simRationale.value = res.recommendationRationale || '已根据屏幕识别海克斯完成推演！';
          simChoices.value = res.candidates.map((c, idx) => ({
            id: c.augmentId,
            slot: idx === 0 ? 'A' : idx === 1 ? 'B' : 'C',
            name: c.name,
            rarity: c.rarity || '黄金',
            score: Math.round(c.score || 75),
            delta: c.deltaWinRate ? `${(c.deltaWinRate >= 0 ? '+' : '')}${(c.deltaWinRate * 100).toFixed(1)}%` : '+3.5%',
            isDestiny: c.isDestiny || (c.deltaWinRate >= 0.035),
            isTrap: c.isTrap || (c.deltaWinRate < -0.005),
            alpha: c.alphaDelta ? `${(c.alphaDelta >= 0 ? '+' : '')}${(c.alphaDelta * 100).toFixed(1)}%` : '+2.4%',
            itemAdvice: c.itemAdvice || '核心神装三件套',
            stages: c.stages || [],
            stageNote: c.stageNote || '',
            cdnIconUrl: c.cdnIconUrl || getAugmentIconUrl(c),
            icon: c.rarity === '棱彩' ? '👑' : c.rarity === '黄金' ? '🌸' : '🌱',
            desc: c.description || '提供强化增益'
          }));
          activeTab.value = 'recommend';
          showToast('海克斯识图推演完成！已锁定最优解', '👑');
        }
      } catch (e) {
        console.warn('OCR Scan error:', e);
        showToast('未检测到活跃游戏画面，已采用离线推演方案', '⚠️');
      }
    };

    // ---------------- WebSocket 实时全双工感知连接 ----------------
    let liveSocket = null;
    let wsReconnectTimer = null;

    const connectLiveWebSocket = () => {
      if (liveSocket && (liveSocket.readyState === WebSocket.OPEN || liveSocket.readyState === WebSocket.CONNECTING)) {
        return;
      }
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const host = window.location.host;
      const wsUrl = `${protocol}//${host}/ws/live`;

      try {
        liveSocket = new WebSocket(wsUrl);

        liveSocket.onopen = () => {
          wsConnected.value = true;
          if (wsReconnectTimer) {
            clearTimeout(wsReconnectTimer);
            wsReconnectTimer = null;
          }
        };

        liveSocket.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            handleLiveWebSocketMessage(data);
          } catch (err) {
            console.warn('[WS] 解析实时数据失败:', err);
          }
        };

        liveSocket.onclose = () => {
          wsConnected.value = false;
          liveSocket = null;
          if (!wsReconnectTimer) {
            wsReconnectTimer = setTimeout(() => {
              wsReconnectTimer = null;
              connectLiveWebSocket();
            }, 3000);
          }
        };

        liveSocket.onerror = () => {
          wsConnected.value = false;
        };
      } catch (e) {
        console.warn('[WS] 无法创建 WebSocket 连接:', e);
        if (!wsReconnectTimer) {
          wsReconnectTimer = setTimeout(() => {
            wsReconnectTimer = null;
            connectLiveWebSocket();
          }, 4000);
        }
      }
    };

    const handleLiveWebSocketMessage = (data) => {
      if (!data) return;
      const event = data.event;

      if (event === 'LCU_CONNECTED') {
        clientConnected.value = true;
        showToast('🟢 已成功连接至英雄联盟客户端', '🎮');
      } else if (event === 'LCU_DISCONNECTED') {
        clientConnected.value = false;
        gamePhase.value = 'None';
        showToast('🔴 英雄联盟客户端已断开', '🔌');
      } else if (event === 'INIT_STATE' || event === 'LIVE_SYNC') {
        clientConnected.value = Boolean(data.lcuConnected);
        if (data.gamePhase) {
          gamePhase.value = data.gamePhase;
        }
        if (typeof data.rerollsRemaining === 'number') {
          diceCount.value = data.rerollsRemaining;
        }
        if (data.benchSlots) {
          liveBenchSlots.value = data.benchSlots;
        }
        if (data.benchRanked) {
          liveBenchRanked.value = data.benchRanked;
        }

        // 选人或对局中自动感知当前我方锁定的英雄并切换详情展示
        if (data.myHero && data.myHero.id) {
          const newHeroId = String(data.myHero.id);
          if (newHeroId && newHeroId !== '0' && newHeroId !== currentHeroId.value) {
            currentHeroId.value = newHeroId;
            fetchHeroDetail(newHeroId);
            const hName = data.myHero.name || (currentHero.value && currentHero.value.name) || '当前英雄';
            showToast(`已自动同步当局英雄：${hName}`, '🎯');
          }
        }

        // 自动同步当前 5 人队伍阵容至开黑实验室
        if (data.teamMembers && Array.isArray(data.teamMembers) && data.teamMembers.length > 0) {
          const memberIds = data.teamMembers.map(m => String(m.id)).filter(id => id && id !== '0' && id !== 'undefined');
          if (memberIds.length > 0) {
            const updatedTeam = [...memberIds];
            while (updatedTeam.length < 5) {
              const fallbackId = teamHeroIds.value[updatedTeam.length] || '142';
              updatedTeam.push(fallbackId);
            }
            const isDifferent = updatedTeam.slice(0, 5).some((id, idx) => id !== teamHeroIds.value[idx]);
            if (isDifferent) {
              teamHeroIds.value = updatedTeam.slice(0, 5);
              if (data.teamSynergy && data.teamSynergy.synergyScore) {
                const syn = data.teamSynergy;
                teamSynergyData.value = {
                  score: syn.synergyScore || 88,
                  evaluation: syn.synergyScore >= 85 ? '极佳' : syn.synergyScore >= 70 ? '良好' : '一般',
                  synergyFit: syn.synergyRating || '8 / 10',
                  damageDist: syn.damageDist || teamSynergyData.value.damageDist,
                  counterStats: teamSynergyData.value.counterStats,
                  synergyPairs: (syn.synergyPairs && syn.synergyPairs.length > 0) ? syn.synergyPairs : teamSynergyData.value.synergyPairs,
                  topThreats: (syn.topThreats && syn.topThreats.length > 0) ? syn.topThreats : teamSynergyData.value.topThreats
                };
              } else {
                calculateTeamSynergy();
              }
            }
          }
        }
      }
    };

    const rollDiceInteractive = async () => {
      isDiceRolling.value = true;
      if (clientConnected.value && gamePhase.value === 'ChampSelect') {
        try {
          const res = await fetch('/api/champ-select/reroll', { method: 'POST' }).then(r => r.json());
          if (res && res.status === 'ok') {
            showToast('🎲 骰子重掷指令已发送至英雄联盟客户端', '🎲');
          } else {
            diceCount.value = (diceCount.value % 3) + 1;
            showToast(`已调整持有骰子数至: ${diceCount.value} 颗 · ROI 已更新`, '🎲');
          }
        } catch (e) {
          diceCount.value = (diceCount.value % 3) + 1;
          showToast(`已调整持有骰子数至: ${diceCount.value} 颗 · ROI 已更新`, '🎲');
        }
      } else {
        diceCount.value = (diceCount.value % 3) + 1;
        showToast(`已调整持有骰子数至: ${diceCount.value} 颗 · ROI 已更新`, '🎲');
      }
      setTimeout(() => { isDiceRolling.value = false; }, 500);
    };

    const inspectAugmentDetail = (aug) => {
      let topHeroes = aug.topHeroes || [];
      if ((!topHeroes || topHeroes.length === 0) && aug.id && allAugments.value) {
        const found = allAugments.value.find(a => String(a.id) === String(aug.id));
        if (found && found.topHeroes && found.topHeroes.length > 0) {
          topHeroes = found.topHeroes;
        }
      }
      selectedDexAugment.value = {
        ...aug,
        topHeroes: topHeroes,
        cdnIconUrl: aug.cdnIconUrl || getAugmentIconUrl(aug)
      };
      showDexDetailModal.value = true;
    };

    // In-Game Overlay & Auto Sensing Settings
    const overlayAutoSensing = ref(true);
    const overlayVoice = ref(true);
    const overlayHotkeyInput = ref('');
    const overlayStatus = ref({
      completedMilestones: [],
      inGamePhase: false,
      currentWindowReason: '',
      isWindowActive: false,
      hotkey: '['
    });

    const fetchOverlayStatus = async () => {
      try {
        const res = await fetch('/api/overlay/status').then(r => r.json());
        if (res && res.status === 'ok') {
          overlayAutoSensing.value = res.autoSensingEnabled;
          overlayVoice.value = res.voiceEnabled;
          overlayStatus.value = res;
        }
      } catch (e) {}
    };

    const toggleAutoSensing = async () => {
      overlayAutoSensing.value = !overlayAutoSensing.value;
      try {
        await fetch('/api/overlay/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ autoSensingEnabled: overlayAutoSensing.value })
        });
        showToast(`时序自动感知已${overlayAutoSensing.value ? '开启' : '关闭'}`, '⚡');
        fetchOverlayStatus();
      } catch (e) {}
    };

    const toggleVoice = async () => {
      overlayVoice.value = !overlayVoice.value;
      try {
        await fetch('/api/overlay/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ voiceEnabled: overlayVoice.value })
        });
        showToast(`语音智能报牌已${overlayVoice.value ? '开启' : '关闭'}`, '🎧');
        fetchOverlayStatus();
      } catch (e) {}
    };

    const saveHotkey = async () => {
      const hotkey = (overlayHotkeyInput.value || '').trim();
      if (!hotkey) {
        showToast('请输入要设置的快捷键键名', '⌨️');
        return;
      }
      try {
        const res = await fetch('/api/overlay/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ hotkey })
        }).then(r => r.json());
        overlayHotkeyInput.value = '';
        if (res && res.status === 'updated') {
          overlayStatus.value.hotkey = res.hotkey;
          showToast(`全局快捷键已更新为 ${res.hotkey}`, '⌨️');
          fetchOverlayStatus();
        }
      } catch (e) {
        showToast('快捷键更新失败，请检查后端服务', '⚠️');
      }
    };

    // Item Overlay (左下角装备推荐 HUD)
    const itemOverlayEnabled = ref(true);
    const itemOverlayVoice = ref(false);
    const itemOverlayStatus = ref({ hotkey: ']', lastRecommendation: null });

    const fetchItemOverlayStatus = async () => {
      try {
        const res = await fetch('/api/item-overlay/status').then(r => r.json());
        if (res && res.status === 'ok') {
          itemOverlayEnabled.value = res.enabled;
          itemOverlayVoice.value = res.voiceEnabled;
          itemOverlayStatus.value = res;
        }
      } catch (e) {}
    };

    const toggleItemOverlay = async () => {
      itemOverlayEnabled.value = !itemOverlayEnabled.value;
      try {
        await fetch('/api/item-overlay/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enabled: itemOverlayEnabled.value })
        });
        showToast(`装备推荐 HUD 已${itemOverlayEnabled.value ? '开启' : '暂停'}`, '🛡️');
        fetchItemOverlayStatus();
      } catch (e) {}
    };

    const toggleItemVoice = async () => {
      itemOverlayVoice.value = !itemOverlayVoice.value;
      try {
        await fetch('/api/item-overlay/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ voiceEnabled: itemOverlayVoice.value })
        });
        showToast(`装备语音播报已${itemOverlayVoice.value ? '开启' : '关闭'}`, '🎧');
        fetchItemOverlayStatus();
      } catch (e) {}
    };

    const triggerItemOverlay = async () => {
      try {
        await fetch('/api/item-overlay/trigger', { method: 'POST' });
        showToast('已触发装备推荐 HUD 弹出！', '🛡️');
      } catch (e) {
        showToast('装备 HUD 触发失败', '⚠️');
      }
    };

    // 推荐偏好与冲突协调控制 (Conflict Resolution & Priority Weights)
    const itemOverlayPreferences = ref({
      resolve_conflicts: true,
      hex_synergy_weight: 1.0,
      archetype_strictness: 0.85,
      counter_intensity: 1.0,
      gold_calc_mode: 'NET_REMAINING',
      trio_mode: 'DYNAMIC_ADAPTIVE',
      current_preset: 'SMART',
      preset_info: { name: '👑 智能竞技托管', desc: '全自动协调流派、海克斯与克制' },
      available_presets: {}
    });

    const isSavingPreferences = ref(false);

    const fetchItemOverlayPreferences = async () => {
      try {
        const res = await fetch('/api/item-overlay/preferences').then(r => r.json());
        if (res && res.status === 'ok' && res.preferences) {
          itemOverlayPreferences.value = res.preferences;
        }
      } catch (e) {
        console.warn('Could not fetch preferences:', e);
      }
    };

    const updateItemOverlayPreferences = async (partialUpdates = {}) => {
      try {
        isSavingPreferences.value = true;
        const current = itemOverlayPreferences.value;
        const payload = {
          resolveConflicts: partialUpdates.resolve_conflicts !== undefined ? partialUpdates.resolve_conflicts : current.resolve_conflicts,
          hexSynergyWeight: partialUpdates.hex_synergy_weight !== undefined ? parseFloat(partialUpdates.hex_synergy_weight) : parseFloat(current.hex_synergy_weight),
          archetypeStrictness: partialUpdates.archetype_strictness !== undefined ? parseFloat(partialUpdates.archetype_strictness) : parseFloat(current.archetype_strictness),
          counterIntensity: partialUpdates.counter_intensity !== undefined ? parseFloat(partialUpdates.counter_intensity) : parseFloat(current.counter_intensity),
          goldCalcMode: partialUpdates.gold_calc_mode || current.gold_calc_mode,
          trioMode: partialUpdates.trio_mode || current.trio_mode,
          preset: partialUpdates.preset || null
        };
        const res = await fetch('/api/item-overlay/preferences', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        }).then(r => r.json());
        if (res && res.status === 'updated' && res.preferences) {
          itemOverlayPreferences.value = res.preferences;
          const pName = res.preferences.preset_info ? res.preferences.preset_info.name : '自定义配置';
          showToast(`偏好已同步生效 [${pName}]`, '⚙️');
          fetchItemOverlayStatus();
        }
      } catch (e) {
        showToast('推荐偏好同步失败', '⚠️');
      } finally {
        isSavingPreferences.value = false;
      }
    };

    const toggleConflictResolution = async () => {
      const nextVal = !itemOverlayPreferences.value.resolve_conflicts;
      await updateItemOverlayPreferences({ resolve_conflicts: nextVal, preset: nextVal ? 'SMART' : null });
    };

    const applyRecommendationPreset = async (presetKey) => {
      await updateItemOverlayPreferences({ preset: presetKey });
    };

    const testHUDPopup = async () => {
      try {
        await fetch('/api/overlay/trigger', { method: 'POST' });
        showToast('已向游戏画面发送置顶 HUD 测试弹窗！', '🖥️');
      } catch (e) {
        showToast('HUD 触发测试失败', '⚠️');
      }
    };

    // Bench Overlay (大乱斗选人上方英雄池胜率置顶 HUD)
    const benchOverlayEnabled = ref(true);
    const benchClickToSwap = ref(true);
    const benchBadgePosition = ref('bottom');
    const benchVerticalOffset = ref(0);
    const benchHorizontalOffset = ref(0);
    const benchOverlayStatus = ref({ isShowing: false, slotsCount: 0 });
    const benchMockActive = ref(false);

    const fetchBenchOverlayStatus = async () => {
      try {
        const res = await fetch('/api/overlay/bench/status').then(r => r.json());
        if (res) {
          benchOverlayEnabled.value = res.enabled;
          benchClickToSwap.value = res.clickToSwap;
          benchBadgePosition.value = res.badgePosition || 'bottom';
          benchVerticalOffset.value = res.verticalOffset || 0;
          benchHorizontalOffset.value = res.horizontalOffset || 0;
          benchOverlayStatus.value = res;
          benchMockActive.value = !!res.mockMode;
        }
      } catch (e) {}
    };

    const updateBenchOverlayConfig = async (partialUpdates = {}) => {
      try {
        const payload = {
          enabled: partialUpdates.enabled !== undefined ? partialUpdates.enabled : benchOverlayEnabled.value,
          clickToSwap: partialUpdates.clickToSwap !== undefined ? partialUpdates.clickToSwap : benchClickToSwap.value,
          badgePosition: partialUpdates.badgePosition || benchBadgePosition.value,
          verticalOffset: partialUpdates.verticalOffset !== undefined ? partialUpdates.verticalOffset : benchVerticalOffset.value,
          horizontalOffset: partialUpdates.horizontalOffset !== undefined ? partialUpdates.horizontalOffset : benchHorizontalOffset.value
        };
        const res = await fetch('/api/overlay/bench/config', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        }).then(r => r.json());
        if (res && res.config) {
          fetchBenchOverlayStatus();
          showToast('英雄池胜率置顶配置已更新', '🎯');
        }
      } catch (e) {
        showToast('更新失败', '⚠️');
      }
    };

    const triggerBenchMock = async () => {
      try {
        // 截图中 10 个英雄: 安妮(1), 狮子狗(107), 小法(45), 猫咪(350), 波比(78), 拉克丝(99), 冰鸟(34), 塔姆(223), 炼金(27), 滑板鞋(429)
        const testHeroes = ['1', '107', '45', '350', '78', '99', '34', '223', '27', '429'];
        await fetch('/api/champ-select/mock', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ heroIds: testHeroes, myHeroId: '804' })
        });
        benchMockActive.value = true;
        showToast('已在屏幕对应位置启动选人英雄池胜率模拟！', '👑');
        fetchBenchOverlayStatus();
      } catch (e) {
        showToast('模拟启动失败', '⚠️');
      }
    };

    const stopBenchMock = async () => {
      try {
        await fetch('/api/champ-select/stop-mock', { method: 'POST' });
        benchMockActive.value = false;
        showToast('已关闭选人模拟预览', '🛑');
        fetchBenchOverlayStatus();
      } catch (e) {}
    };

    const fetchHeroDetail = async (heroId) => {
      try {
        const res = await fetch(`/api/hero/${heroId}`).then(r => r.json());
        if (res) {
          heroDetailData.value = res;
          
          // Dynamically populate 3-pick simulator candidates with real distinct augments from this hero!
          const sList = (res.augments && res.augments.sTier) || [];
          const aList = (res.augments && res.augments.aTier) || [];
          const bList = (res.augments && res.augments.bTier) || [];
          const trapList = (res.augments && res.augments.traps) || [];
          const pool = [...sList, ...aList, ...bList];

          const seenIds = new Set();
          const pickUnique = (list) => {
            for (const item of (list || [])) {
              if (!item) continue;
              const id = typeof item === 'object' ? String(item.augmentId || item.id || '') : String(item);
              if (id && !seenIds.has(id)) {
                seenIds.add(id);
                return id;
              }
            }
            return null;
          };

          const s1 = pickUnique(sList) || pickUnique(pool) || '1134';
          const s2 = pickUnique(aList) || pickUnique(pool) || '2095';
          const s3 = pickUnique(trapList) || pickUnique(bList) || pickUnique(pool) || '1058';

          simAugmentIds.value = [s1, s2, s3];
          runAugmentCompare();
        }
      } catch (e) {
        console.warn('Could not fetch hero detail:', e);
      }
    };

    const runAugmentCompare = async () => {
      try {
        const stageNum = simStage.value === '2-1' ? 1 : simStage.value === '3-2' ? 2 : 3;
        const res = await fetch('/api/augment/compare', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            heroId: String(currentHeroId.value),
            augmentIds: simAugmentIds.value,
            currentStage: stageNum
          })
        }).then(r => r.json());

        if (res && res.candidates) {
          simRationale.value = res.recommendationRationale || '';
          if (res.rerollDecision) {
            simRerollAdvice.value = res.rerollDecision;
          }
          simChoices.value = res.candidates.map((c, idx) => ({
            id: c.augmentId,
            slot: idx === 0 ? 'A' : idx === 1 ? 'B' : 'C',
            name: c.name,
            rarity: c.rarity || '黄金',
            score: Math.round(c.score || 75),
            delta: c.deltaWinRate ? `${(c.deltaWinRate >= 0 ? '+' : '')}${(c.deltaWinRate * 100).toFixed(1)}%` : '+3.5%',
            isDestiny: c.isDestiny || (c.deltaWinRate >= 0.035),
            isTrap: c.isTrap || (c.deltaWinRate < -0.005),
            alpha: c.alphaDelta ? `${(c.alphaDelta >= 0 ? '+' : '')}${(c.alphaDelta * 100).toFixed(1)}%` : '+2.4%',
            itemAdvice: c.itemAdvice || '核心神装三件套',
            stages: c.stages || [],
            stageNote: c.stageNote || '',
            cdnIconUrl: c.cdnIconUrl || getAugmentIconUrl(c),
            icon: c.rarity === '棱彩' ? '👑' : c.rarity === '黄金' ? '🌸' : '🌱',
            desc: c.description || '提供强化增益'
          }));
        }
      } catch (e) {
        console.warn('Augment compare fallback:', e);
      }
    };

    const openSwapAugment = (idx) => {
      currentSwapSlot.value = idx;
      showSwapAugmentModal.value = true;
    };

    const selectSwapAugment = (aug) => {
      if (!aug) return;
      const augId = String(aug.id || aug.augmentId || '1134');
      const augName = aug.name || aug.augmentName || '海克斯强化';
      const slotIdx = Math.max(0, Math.min(2, parseInt(currentSwapSlot.value) || 0));
      simAugmentIds.value[slotIdx] = augId;
      showSwapAugmentModal.value = false;
      runAugmentCompare();
      showToast(`已更换卡槽 ${slotIdx === 0 ? 'A' : slotIdx === 1 ? 'B' : 'C'} 为【${augName}】`, '🎴');
    };

    const calculateTeamSynergy = async () => {
      try {
        const res = await fetch('/api/team/synergy', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ heroIds: teamHeroIds.value })
        }).then(r => r.json());

        if (res) {
          const heroes = teamHeroList.value;
          let magicWeight = 0, physWeight = 0, trueWeight = 10;
          heroes.forEach(h => {
            const roles = h.roles || [];
            if (roles.includes('法师') || roles.includes('辅助')) magicWeight += 20;
            else if (roles.includes('射手') || roles.includes('刺客')) physWeight += 20;
            else if (roles.includes('战士')) { physWeight += 12; magicWeight += 8; }
            else if (roles.includes('坦克')) { magicWeight += 10; physWeight += 10; }
            else { magicWeight += 10; physWeight += 10; }
          });
          const total = magicWeight + physWeight + trueWeight;
          const mPct = Math.round((magicWeight / total) * 100);
          const pPct = Math.round((physWeight / total) * 100);
          const tPct = 100 - mPct - pPct;

          const score = res.synergyScore || 88;
          const ratingText = score >= 85 ? '极佳' : score >= 70 ? '良好' : '一般';
          const distinctRoles = new Set(heroes.flatMap(h => h.roles || [])).size;
          const fitText = `${Math.min(10, Math.floor(distinctRoles * 1.6))} / 10`;

          teamSynergyData.value = {
            score: score,
            evaluation: ratingText,
            synergyFit: fitText,
            damageDist: { physical: pPct, magic: mPct, trueDmg: tPct },
            counterStats: { winAdvantage: Math.min(85, 55 + distinctRoles * 3), winDisadvantage: 100 - Math.min(85, 55 + distinctRoles * 3) },
            synergyPairs: (res.synergyPairs && res.synergyPairs.length > 0) ? res.synergyPairs : teamSynergyData.value.synergyPairs,
            topThreats: (res.topThreats && res.topThreats.length > 0) ? res.topThreats : teamSynergyData.value.topThreats
          };
        }
      } catch (e) {
        console.warn('Team synergy fallback:', e);
      }
    };

    const openSwapTeamChampion = (slotIdx) => {
      currentSwapTeamSlot.value = slotIdx;
      showSwapChampionModal.value = true;
    };

    const selectSwapTeamChampion = (hero) => {
      if (!hero) return;
      const heroId = String(hero.id || '142');
      const heroName = hero.name || hero.title || '英雄';
      const slotIdx = Math.max(0, Math.min(4, parseInt(currentSwapTeamSlot.value) || 0));
      teamHeroIds.value[slotIdx] = heroId;
      showSwapChampionModal.value = false;
      calculateTeamSynergy();
      showToast(`已更换位置 ${slotIdx + 1} 为【${heroName}】· 协同度已重算`, '👥');
    };

    const handleImgError = (e) => {
      e.target.src = 'https://game.gtimg.cn/images/lol/act/img/champion/Zoe.png';
    };

    // ---------------- Global Data Initialization ----------------
    const fetchData = async () => {
      try {
        const [metaRes, heroesRes, augsRes] = await Promise.all([
          fetch('/api/meta').then(r => r.json()).catch(() => null),
          fetch('/api/heroes').then(r => r.json()).catch(() => null),
          fetch('/api/augments').then(r => r.json()).catch(() => null)
        ]);

        if (metaRes) {
          meta.value = {
            patch: metaRes.patch || '16.17',
            reportDate: metaRes.reportDate || '2026-08-29',
            heroCount: metaRes.heroCount || 173,
            augmentCount: metaRes.augmentCount || 209,
            totalMatches: metaRes.totalMatches ? `${(metaRes.totalMatches / 10000).toFixed(1)}w` : '1157.2w',
            comboCount: '8.7k'
          };
        }

        if (heroesRes) {
          const rawHeroes = Array.isArray(heroesRes) ? heroesRes : (heroesRes.heroes || []);
          if (rawHeroes.length > 0) {
            allHeroes.value = rawHeroes;
          }
        }

        if (augsRes) {
          const rawAugs = Array.isArray(augsRes) ? augsRes : (augsRes.augments || []);
          if (rawAugs.length > 0) {
            allAugments.value = rawAugs.map(a => {
              const wr = parsePctOrFloat(a.winRate !== undefined ? a.winRate : 0.542);
              const pr = parsePctOrFloat(a.pickRate !== undefined ? a.pickRate : 0.150);
              const dwr = parsePctOrFloat(a.avgDeltaWinRate !== undefined ? a.avgDeltaWinRate : (a.deltaWinRate !== undefined ? a.deltaWinRate : 0.035));
              return {
                id: String(a.id || a.augmentId || ''),
                name: a.name || a.augmentName || '海克斯强化',
                rarity: a.rarity || (a.tier === 1 ? '白银' : a.tier === 2 ? '黄金' : '棱彩'),
                score: Math.round(a.globalHexScore || (75 + (wr / 100) * 35)),
                delta: `${dwr >= 0 ? '+' : ''}${dwr.toFixed(1)}%`,
                cdnIconUrl: a.cdnIconUrl || getAugmentIconUrl(a),
                icon: a.rarity === '棱彩' ? '👑' : a.rarity === '黄金' ? '🌸' : '🌱',
                desc: a.description || '海克斯强化效果',
                winRate: `${wr.toFixed(1)}%`,
                pickRate: `${pr.toFixed(1)}%`,
                tier: a.tier || 2,
                topHeroes: (a.topHeroes || []).map(th => ({
                  heroName: th.heroName,
                  deltaWinRate: parsePctOrFloat(th.deltaWinRate || 0.05),
                  pairWinRate: parsePctOrFloat(th.pairWinRate || 0.56),
                  avatarUrl: th.avatarUrl || 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/142.png'
                }))
              };
            });
          }
        }

        // Trigger initial hero and team synergy fetch
        fetchHeroDetail(currentHeroId.value);
        runAugmentCompare();
        calculateTeamSynergy();
      } catch (err) {
        console.warn('Initialization error:', err);
      } finally {
        isLoading.value = false;
      }
    };

    onMounted(() => {
      fetchData();
      fetchOverlayStatus();
      fetchBenchOverlayStatus();
      fetchItemOverlayPreferences();
      connectLiveWebSocket();
      setInterval(fetchOverlayStatus, 3000);
      setInterval(fetchBenchOverlayStatus, 3000);
    });

    watch(simStage, () => {
      runAugmentCompare();
    });

    return {
      activeTab,
      dexActiveMode,
      heroAugmentsTab,
      heroAugmentSearch,
      heroAugmentRarity,
      heroAugmentSortBy,
      heroAugmentSortAsc,
      showHeroAugmentsModal,
      allHeroAugments,
      itemSubTab,
      itemSearch,
      trioTierFilter,
      trioDeltaFilter,
      trioMinGames,
      trioSortBy,
      trioSortAsc,
      singleTierFilter,
      singleDeltaFilter,
      singleItemSortBy,
      singleItemSortAsc,
      shoesSortBy,
      shoesSortAsc,
      toggleTrioSort,
      toggleSingleSort,
      toggleShoesSort,
      allHeroTrios,
      allHeroSingleItems,
      savedFavorites,
      isLoading,
      clientConnected,
      wsConnected,
      gamePhase,
      gamePhaseText,
      liveBenchSlots,
      liveBenchRanked,
      diceCount,
      isDiceRolling,
      toasts,
      meta,
      allHeroes,
      allAugments,
      globalSearch,
      showSearchDropdown,
      searchSuggestions,
      hotTags,
      rolesList,
      heroFilterRole,
      heroSortBy,
      heroSortAsc,
      currentHeroId,
      currentHero,
      heroDecisionSubTab,
      heroDetailData,
      simStage,
      simChoices,
      simRationale,
      simRerollAdvice,
      teamHeroIds,
      teamHeroList,
      teamSynergyData,
      currentSwapSlot,
      currentSwapTeamSlot,
      dynamicTeamData,
      rerollEvaluation,
      heroAugmentsRecommend,
      heroAugmentsAvoid,
      heroCoreItems,
      heroBoots,
      heroSummonerSpells,
      heroMatchups,
      combatStats,
      dexSearch,
      dexRarity,
      dexTierFilter,
      dexSortBy,
      dexSortAsc,
      filteredDexAugments,
      selectedDexAugment,
      filteredRankHeroes,
      topRankHeroes,
      filteredAugmentsForSwap,
      filteredChampionsForSwap,
      showFullRankModal,
      showFullAugmentsModal,
      showSimDetailModal,
      showTeamDetailModal,
      showDexDetailModal,
      showSwapAugmentModal,
      showSwapChampionModal,
      swapAugmentSearch,
      swapChampionSearch,
      selectHero,
      applySearchTag,
      handleSearchEnter,
      triggerOCRScan,
      rollDiceInteractive,
      inspectAugmentDetail,
      openSwapAugment,
      selectSwapAugment,
      openSwapTeamChampion,
      selectSwapTeamChampion,
      calculateTeamSynergy,
      handleImgError,
      getAugmentIconUrl,
      handleAugmentImgError,
      overlayAutoSensing,
      overlayVoice,
      overlayStatus,
      overlayHotkeyInput,
      fetchOverlayStatus,
      toggleAutoSensing,
      toggleVoice,
      saveHotkey,
      testHUDPopup,
      itemOverlayEnabled,
      itemOverlayVoice,
      itemOverlayStatus,
      fetchItemOverlayStatus,
      toggleItemOverlay,
      toggleItemVoice,
      triggerItemOverlay,
      itemOverlayPreferences,
      isSavingPreferences,
      fetchItemOverlayPreferences,
      updateItemOverlayPreferences,
      toggleConflictResolution,
      applyRecommendationPreset,
      benchOverlayEnabled,
      benchClickToSwap,
      benchBadgePosition,
      benchVerticalOffset,
      benchHorizontalOffset,
      benchOverlayStatus,
      benchMockActive,
      fetchBenchOverlayStatus,
      updateBenchOverlayConfig,
      triggerBenchMock,
      stopBenchMock,
      showToast
    };
  }
});

// Global Error Handler Guard
app.config.errorHandler = (err, vm, info) => {
  console.warn('[HexBrain Vue Safe Boundary]:', err, info);
};

app.mount('#app');
window.hexBrainAppMounted = true;