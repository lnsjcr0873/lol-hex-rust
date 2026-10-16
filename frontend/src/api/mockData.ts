import { OngoingMatchData, SummonerProfile, MatchHistoryItem, MatchDetailFull, AutomationConfig } from './types';
import type {
  Game,
  GameTimeline,
  MatchHistory,
  Participant,
  ParticipantIdentity,
  Team,
  Stats,
  TimelineFrame,
  ParticipantFrames,
  GameEvent,
  ChampionKillEvent,
  BuildingKillEvent,
  EliteMonsterKillEvent
} from '@shared/types/league-client/match-history';

export const mockOngoingMatch: OngoingMatchData = {
  gameflow_phase: 'ChampSelect',
  queue_id: 420,
  queue_name: '排位赛 单双排 (5v5 召唤师峡谷)',
  is_my_team_blue: true,
  my_puuid: 'p-blue-1',
  chat_room_id: 'chat-room-12345',
  blue_team: [
    {
      puuid: 'p-blue-1',
      summoner_id: '1001',
      game_name: '飞天小女警',
      tag_line: 'CN1',
      profile_icon_id: 532,
      summoner_level: 285,
      team: 'blue',
      cell_id: 0,
      assigned_position: 'TOP',
      champion_id: 266,
      champion_name: '暗裔剑魔',
      champion_title: '亚托克斯',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/266.png',
      is_locked: true,
      spell1_id: 12,
      spell1_name: '传送',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'I',
      lp: 68,
      recent_kda: '9.2 / 3.4 / 7.1',
      recent_kda_ratio: 4.8,
      recent_win_rate: 68,
      recent_games_count: 20,
      champ_recent_games: 8,
      champ_recent_win_rate: 75,
      recent_matches: ['win', 'win', 'loss', 'win', 'win'],
      badges: [
        { id: 'b1', label: '绝活哥', icon: '👑', color: 'gold', description: '英雄近8场胜率75%' },
        { id: 'b2', label: '通天代', icon: '⚡', color: 'cyan', description: '近20场排位胜率68% KDA 4.8' },
        { id: 'b3', label: '连胜中', icon: '🔥', color: 'red', description: '近期二连胜' }
      ],
      premade_party: { party_id: 1, label: '🚗 双排车队', color_class: 'party-border-pink' },
      radar: { output: 88, kda: 85, join_rate: 74, survival: 79, gold: 86, vision: 65 }
    },
    {
      puuid: 'p-blue-2',
      summoner_id: '1002',
      game_name: '野区养猪场长',
      tag_line: 'EDG',
      profile_icon_id: 712,
      summoner_level: 412,
      team: 'blue',
      cell_id: 1,
      assigned_position: 'JUNGLE',
      champion_id: 64,
      champion_name: '盲僧',
      champion_title: '李青',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/64.png',
      is_locked: true,
      spell1_id: 11,
      spell1_name: '惩戒',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'II',
      lp: 42,
      recent_kda: '6.4 / 4.1 / 11.2',
      recent_kda_ratio: 4.3,
      recent_win_rate: 62,
      recent_games_count: 20,
      champ_recent_games: 12,
      champ_recent_win_rate: 67,
      recent_matches: ['win', 'loss', 'win', 'win', 'win'],
      badges: [
        { id: 'b4', label: '开团先锋', icon: '🎯', color: 'blue', description: '场均参团率达72%' }
      ],
      premade_party: { party_id: 1, label: '🚗 双排车队', color_class: 'party-border-pink' },
      radar: { output: 72, kda: 80, join_rate: 90, survival: 71, gold: 75, vision: 88 }
    },
    {
      puuid: 'p-blue-3',
      summoner_id: '1003',
      game_name: '中路防御塔本体',
      tag_line: 'MID',
      profile_icon_id: 1024,
      summoner_level: 198,
      team: 'blue',
      cell_id: 2,
      assigned_position: 'MIDDLE',
      champion_id: 103,
      champion_name: '九尾妖狐',
      champion_title: '阿狸',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/103.png',
      is_locked: true,
      spell1_id: 14,
      spell1_name: '引燃',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerDot.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'III',
      lp: 15,
      recent_kda: '7.8 / 3.9 / 6.5',
      recent_kda_ratio: 3.7,
      recent_win_rate: 55,
      recent_games_count: 20,
      champ_recent_games: 5,
      champ_recent_win_rate: 60,
      recent_matches: ['loss', 'win', 'win', 'loss', 'win'],
      badges: [],
      radar: { output: 81, kda: 74, join_rate: 68, survival: 76, gold: 80, vision: 62 }
    },
    {
      puuid: 'p-blue-4',
      summoner_id: '1004',
      game_name: '一枪一个小朋友',
      tag_line: 'ADC',
      profile_icon_id: 618,
      summoner_level: 320,
      team: 'blue',
      cell_id: 3,
      assigned_position: 'BOTTOM',
      champion_id: 222,
      champion_name: '暴走萝莉',
      champion_title: '金克丝',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/222.png',
      is_locked: false,
      spell1_id: 7,
      spell1_name: '治疗术',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerHeal.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'IV',
      lp: 88,
      recent_kda: '10.5 / 5.2 / 5.8',
      recent_kda_ratio: 3.1,
      recent_win_rate: 52,
      recent_games_count: 20,
      champ_recent_games: 14,
      champ_recent_win_rate: 57,
      recent_matches: ['win', 'loss', 'loss', 'win', 'loss'],
      badges: [
        { id: 'b5', label: '输出机器', icon: '💥', color: 'gold', description: '场均输出占比31%' }
      ],
      radar: { output: 95, kda: 68, join_rate: 65, survival: 54, gold: 92, vision: 45 }
    },
    {
      puuid: 'p-blue-5',
      summoner_id: '1005',
      game_name: '带刀侍卫璐璐酱',
      tag_line: 'SUP',
      profile_icon_id: 980,
      summoner_level: 164,
      team: 'blue',
      cell_id: 4,
      assigned_position: 'UTILITY',
      champion_id: 117,
      champion_name: '仙灵女巫',
      champion_title: '璐璐',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/117.png',
      is_locked: false,
      spell1_id: 3,
      spell1_name: '虚弱',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerExhaust.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'PLATINUM',
      division: 'I',
      lp: 92,
      recent_kda: '1.2 / 3.1 / 18.4',
      recent_kda_ratio: 6.3,
      recent_win_rate: 65,
      recent_games_count: 20,
      champ_recent_games: 9,
      champ_recent_win_rate: 67,
      recent_matches: ['win', 'win', 'win', 'win', 'loss'],
      badges: [
        { id: 'b6', label: '金牌辅助', icon: '🛡️', color: 'cyan', description: '视野得分超95%同段位玩家' },
        { id: 'b7', label: '四连胜', icon: '🔥', color: 'red', description: '状态极佳' }
      ],
      radar: { output: 35, kda: 92, join_rate: 88, survival: 82, gold: 50, vision: 98 }
    }
  ],
  red_team: [
    {
      puuid: 'p-red-1',
      summoner_id: '2001',
      game_name: '峡谷钢琴家杰斯',
      tag_line: 'RED',
      profile_icon_id: 812,
      summoner_level: 350,
      team: 'red',
      cell_id: 5,
      assigned_position: 'TOP',
      champion_id: 126,
      champion_name: '未来守护者',
      champion_title: '杰斯',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/126.png',
      is_locked: true,
      spell1_id: 12,
      spell1_name: '传送',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'I',
      lp: 84,
      recent_kda: '7.5 / 5.1 / 5.2',
      recent_kda_ratio: 2.5,
      recent_win_rate: 45,
      recent_games_count: 20,
      champ_recent_games: 11,
      champ_recent_win_rate: 45,
      recent_matches: ['loss', 'loss', 'loss', 'win', 'loss'],
      badges: [
        { id: 'b8', label: '三连败', icon: '❄️', color: 'blue', description: '近期心态可能波动' }
      ],
      radar: { output: 85, kda: 55, join_rate: 52, survival: 58, gold: 82, vision: 50 }
    },
    {
      puuid: 'p-red-2',
      summoner_id: '2002',
      game_name: '豹女代练不说话',
      tag_line: 'VIP',
      profile_icon_id: 742,
      summoner_level: 78,
      team: 'red',
      cell_id: 6,
      assigned_position: 'JUNGLE',
      champion_id: 76,
      champion_name: '狂野女猎手',
      champion_title: '奈德丽',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/76.png',
      is_locked: true,
      spell1_id: 11,
      spell1_name: '惩戒',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'MASTER',
      division: 'I',
      lp: 120,
      recent_kda: '12.4 / 2.8 / 8.9',
      recent_kda_ratio: 7.6,
      recent_win_rate: 80,
      recent_games_count: 20,
      champ_recent_games: 15,
      champ_recent_win_rate: 86,
      recent_matches: ['win', 'win', 'win', 'win', 'win'],
      badges: [
        { id: 'b9', label: '通天代', icon: '⚡', color: 'cyan', description: '胜率80% 胜点Master 120LP' },
        { id: 'b10', label: '五连胜', icon: '🔥', color: 'red', description: '连胜势不可挡' },
        { id: 'b11', label: '绝活哥', icon: '👑', color: 'gold', description: '豹女胜率 86%' }
      ],
      premade_party: { party_id: 2, label: '🚗 双排车队', color_class: 'party-border-yellow' },
      radar: { output: 96, kda: 98, join_rate: 85, survival: 89, gold: 98, vision: 84 }
    },
    {
      puuid: 'p-red-3',
      summoner_id: '2003',
      game_name: '沙皇推土机',
      tag_line: 'MID',
      profile_icon_id: 888,
      summoner_level: 215,
      team: 'red',
      cell_id: 7,
      assigned_position: 'MIDDLE',
      champion_id: 268,
      champion_name: '沙漠皇帝',
      champion_title: '阿兹尔',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/268.png',
      is_locked: true,
      spell1_id: 12,
      spell1_name: '传送',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'II',
      lp: 75,
      recent_kda: '6.1 / 3.8 / 7.2',
      recent_kda_ratio: 3.5,
      recent_win_rate: 60,
      recent_games_count: 20,
      champ_recent_games: 8,
      champ_recent_win_rate: 62,
      recent_matches: ['win', 'loss', 'win', 'win', 'loss'],
      badges: [],
      premade_party: { party_id: 2, label: '🚗 双排车队', color_class: 'party-border-yellow' },
      radar: { output: 86, kda: 72, join_rate: 68, survival: 74, gold: 84, vision: 65 }
    },
    {
      puuid: 'p-red-4',
      summoner_id: '2004',
      game_name: '狂暴小黄毛EZ',
      tag_line: 'ADC',
      profile_icon_id: 450,
      summoner_level: 180,
      team: 'red',
      cell_id: 8,
      assigned_position: 'BOTTOM',
      champion_id: 81,
      champion_name: '探险家',
      champion_title: '伊泽瑞尔',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/81.png',
      is_locked: false,
      spell1_id: 7,
      spell1_name: '治疗术',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerHeal.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'III',
      lp: 54,
      recent_kda: '5.8 / 4.4 / 6.1',
      recent_kda_ratio: 2.7,
      recent_win_rate: 48,
      recent_games_count: 20,
      champ_recent_games: 2,
      champ_recent_win_rate: 0,
      recent_matches: ['loss', 'loss', 'win', 'loss', 'win'],
      badges: [
        { id: 'b12', label: '试英雄', icon: '⚠️', color: 'purple', description: '近20场仅玩过2场且0胜' }
      ],
      radar: { output: 78, kda: 60, join_rate: 62, survival: 70, gold: 76, vision: 52 }
    },
    {
      puuid: 'p-red-5',
      summoner_id: '2005',
      game_name: '泰坦钩无虚发',
      tag_line: 'HOOK',
      profile_icon_id: 630,
      summoner_level: 260,
      team: 'red',
      cell_id: 9,
      assigned_position: 'UTILITY',
      champion_id: 111,
      champion_name: '深海泰坦',
      champion_title: '诺提勒斯',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/111.png',
      is_locked: false,
      spell1_id: 14,
      spell1_name: '引燃',
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerDot.png',
      spell2_id: 4,
      spell2_name: '闪现',
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND',
      division: 'IV',
      lp: 12,
      recent_kda: '2.1 / 6.2 / 12.8',
      recent_kda_ratio: 2.4,
      recent_win_rate: 50,
      recent_games_count: 20,
      champ_recent_games: 10,
      champ_recent_win_rate: 50,
      recent_matches: ['loss', 'win', 'loss', 'win', 'loss'],
      badges: [],
      radar: { output: 42, kda: 58, join_rate: 76, survival: 62, gold: 52, vision: 85 }
    }
  ]
};

export const mockSummonerProfile: SummonerProfile = {
  puuid: 'p-blue-1',
  game_name: '飞天小女警',
  tag_line: 'CN1',
  profile_icon_id: 532,
  summoner_level: 285,
  tier_solo: 'DIAMOND',
  division_solo: 'I',
  lp_solo: 68,
  wins_solo: 142,
  losses_solo: 98,
  win_rate_solo: 59,
  tier_flex: 'EMERALD',
  division_flex: 'II',
  lp_flex: 45,
  wins_flex: 38,
  losses_flex: 22,
  win_rate_flex: 63,
  recent_radar: { output: 88, kda: 85, join_rate: 74, survival: 79, gold: 86, vision: 65 },
  recent_summary: {
    total_games: 20,
    win_rate: 65,
    avg_kda: '8.4 / 3.8 / 7.2',
    avg_kda_ratio: 4.1,
    top_champions: [
      {
        champion_id: 266,
        champion_name: '暗裔剑魔',
        champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/266.png',
        games: 8,
        win_rate: 75,
        kda: '9.2 / 3.4 / 7.1'
      },
      {
        champion_id: 84,
        champion_name: '离群之刺',
        champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/84.png',
        games: 6,
        win_rate: 67,
        kda: '11.0 / 4.1 / 5.5'
      },
      {
        champion_id: 39,
        champion_name: '刀锋舞者',
        champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/39.png',
        games: 4,
        win_rate: 50,
        kda: '6.5 / 5.0 / 4.8'
      }
    ]
  }
};

export const mockMatchHistory: MatchHistoryItem[] = [
  {
    game_id: 7012345671,
    game_creation: Date.now() - 1000 * 60 * 35,
    game_duration_seconds: 1720,
    game_mode: 'CLASSIC',
    queue_id: 420,
    queue_name: '排位赛 单双排',
    is_win: true,
    champion_id: 266,
    champion_name: '暗裔剑魔',
    champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/266.png',
    role: '上单',
    kills: 12,
    deaths: 2,
    assists: 9,
    kda_ratio: 10.5,
    is_mvp: true,
    is_svp: false,
    items: [3074, 3053, 3111, 6631, 3071, 3026, 3340],
    total_damage: 34820,
    damage_share_pct: 32,
    damage_taken: 28400,
    gold_earned: 15400,
    minions_killed: 245,
    participants_preview: [
      { champion_id: 266, champion_name: '暗裔剑魔', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/266.png', game_name: '飞天小女警', is_win: true },
      { champion_id: 64, champion_name: '盲僧', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/64.png', game_name: '野区养猪场长', is_win: true },
      { champion_id: 103, champion_name: '阿狸', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/103.png', game_name: '中路防御塔本体', is_win: true },
      { champion_id: 222, champion_name: '金克丝', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/222.png', game_name: '一枪一个小朋友', is_win: true },
      { champion_id: 117, champion_name: '璐璐', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/117.png', game_name: '带刀侍卫璐璐酱', is_win: true },
      { champion_id: 126, champion_name: '杰斯', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/126.png', game_name: '峡谷钢琴家杰斯', is_win: false },
      { champion_id: 76, champion_name: '奈德丽', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/76.png', game_name: '豹女代练不说话', is_win: false },
      { champion_id: 268, champion_name: '阿兹尔', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/268.png', game_name: '沙皇推土机', is_win: false },
      { champion_id: 81, champion_name: '伊泽瑞尔', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/81.png', game_name: '狂暴小黄毛EZ', is_win: false },
      { champion_id: 111, champion_name: '诺提勒斯', champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/111.png', game_name: '泰坦钩无虚发', is_win: false }
    ]
  },
  {
    game_id: 7012345672,
    game_creation: Date.now() - 1000 * 60 * 120,
    game_duration_seconds: 1980,
    game_mode: 'ARAM',
    queue_id: 450,
    queue_name: '极地大乱斗',
    is_win: true,
    champion_id: 84,
    champion_name: '离群之刺',
    champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/84.png',
    role: '中单',
    kills: 19,
    deaths: 6,
    assists: 14,
    kda_ratio: 5.5,
    is_mvp: true,
    is_svp: false,
    items: [3157, 3089, 3152, 4645, 3020, 3165, 0],
    total_damage: 48600,
    damage_share_pct: 36,
    damage_taken: 31200,
    gold_earned: 18200,
    minions_killed: 68,
    participants_preview: []
  },
  {
    game_id: 7012345673,
    game_creation: Date.now() - 1000 * 60 * 240,
    game_duration_seconds: 1540,
    game_mode: 'CLASSIC',
    queue_id: 420,
    queue_name: '排位赛 单双排',
    is_win: false,
    champion_id: 39,
    champion_name: '刀锋舞者',
    champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/39.png',
    role: '中单',
    kills: 8,
    deaths: 3,
    assists: 5,
    kda_ratio: 4.3,
    is_mvp: false,
    is_svp: true,
    items: [3153, 3078, 3053, 3111, 0, 0, 3340],
    total_damage: 22100,
    damage_share_pct: 28,
    damage_taken: 19800,
    gold_earned: 11400,
    minions_killed: 198,
    participants_preview: []
  }
];

export const mockMatchDetail: MatchDetailFull = {
  game_id: 7012345671,
  game_creation: Date.now() - 1000 * 60 * 35,
  game_duration: 1720,
  game_mode: 'CLASSIC',
  game_mode_cn: '召唤师峡谷',
  queue_id: 420,
  queue_name: '排位赛 单双排 (5v5 召唤师峡谷)',
  blue_team_kills: 35,
  blue_team_turrets: 9,
  blue_team_dragons: 3,
  blue_team_barons: 1,
  blue_team_win: true,
  red_team_kills: 18,
  red_team_turrets: 2,
  red_team_dragons: 1,
  red_team_barons: 0,
  red_team_win: false,
  participants: [
    {
      puuid: 'p-blue-1',
      summoner_name: '飞天小女警',
      game_name: '飞天小女警',
      tag_line: 'CN1',
      team: 'blue',
      champion_id: 266,
      champion_name: '暗裔剑魔',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/266.png',
      champion_level: 16,
      role: 'TOP',
      spell1_id: 12,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND I',
      kills: 12,
      deaths: 2,
      assists: 9,
      kda_str: '10.5',
      kda_ratio: 10.5,
      is_mvp: true,
      is_svp: false,
      total_damage: 34820,
      total_damage_taken: 28400,
      damage_taken: 28400,
      damage_share_pct: 32,
      damage_taken_share_pct: 29.5,
      is_highest_damage: true,
      damage_mitigated: 35000,
      gold_earned: 15400,
      minions_killed: 245,
      items: [3074, 3053, 3111, 6631, 3071, 3026, 3340],
      item_trinket: 3340
    },
    {
      puuid: 'p-blue-2',
      summoner_name: '野区养猪场长',
      game_name: '野区养猪场长',
      tag_line: 'EDG',
      team: 'blue',
      champion_id: 64,
      champion_name: '盲僧',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/64.png',
      champion_level: 15,
      role: 'JUNGLE',
      spell1_id: 11,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND II',
      kills: 7,
      deaths: 3,
      assists: 14,
      kda_str: '7.0',
      kda_ratio: 7.0,
      is_mvp: false,
      is_svp: false,
      total_damage: 18500,
      total_damage_taken: 24100,
      damage_taken: 24100,
      damage_share_pct: 17,
      damage_taken_share_pct: 25.0,
      is_highest_damage: false,
      damage_mitigated: 21000,
      gold_earned: 12800,
      minions_killed: 160,
      items: [3078, 3053, 3111, 3075, 0, 0, 3364],
      item_trinket: 3364
    },
    {
      puuid: 'p-blue-3',
      summoner_name: '中路防御塔本体',
      game_name: '中路防御塔本体',
      tag_line: 'MID',
      team: 'blue',
      champion_id: 103,
      champion_name: '九尾妖狐',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/103.png',
      champion_level: 16,
      role: 'MIDDLE',
      spell1_id: 14,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerDot.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND III',
      kills: 8,
      deaths: 4,
      assists: 8,
      kda_str: '4.0',
      kda_ratio: 4.0,
      is_mvp: false,
      is_svp: false,
      total_damage: 24200,
      total_damage_taken: 16200,
      damage_taken: 16200,
      damage_share_pct: 22,
      damage_taken_share_pct: 16.8,
      is_highest_damage: false,
      damage_mitigated: 12000,
      gold_earned: 13500,
      minions_killed: 220,
      items: [3089, 3157, 3020, 6655, 3165, 0, 3340],
      item_trinket: 3340
    },
    {
      puuid: 'p-blue-4',
      summoner_name: '一枪一个小朋友',
      game_name: '一枪一个小朋友',
      tag_line: 'ADC',
      team: 'blue',
      champion_id: 222,
      champion_name: '暴走萝莉',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/222.png',
      champion_level: 15,
      role: 'BOTTOM',
      spell1_id: 7,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerHeal.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND IV',
      kills: 6,
      deaths: 5,
      assists: 10,
      kda_str: '3.2',
      kda_ratio: 3.2,
      is_mvp: false,
      is_svp: false,
      total_damage: 26500,
      total_damage_taken: 15300,
      damage_taken: 15300,
      damage_share_pct: 24,
      damage_taken_share_pct: 15.9,
      is_highest_damage: false,
      damage_mitigated: 8000,
      gold_earned: 14200,
      minions_killed: 260,
      items: [3031, 3085, 3006, 3046, 3036, 0, 3363],
      item_trinket: 3363
    },
    {
      puuid: 'p-blue-5',
      summoner_name: '带刀侍卫璐璐酱',
      game_name: '带刀侍卫璐璐酱',
      tag_line: 'SUP',
      team: 'blue',
      champion_id: 117,
      champion_name: '仙灵女巫',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/117.png',
      champion_level: 13,
      role: 'UTILITY',
      spell1_id: 3,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerExhaust.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'PLATINUM I',
      kills: 2,
      deaths: 4,
      assists: 22,
      kda_str: '6.0',
      kda_ratio: 6.0,
      is_mvp: false,
      is_svp: false,
      total_damage: 5100,
      total_damage_taken: 12400,
      damage_taken: 12400,
      damage_share_pct: 5,
      damage_taken_share_pct: 12.8,
      is_highest_damage: false,
      damage_mitigated: 18000,
      gold_earned: 8900,
      minions_killed: 32,
      items: [3865, 3107, 3117, 3504, 0, 0, 3364],
      item_trinket: 3364
    },
    // Red Team
    {
      puuid: 'p-red-1',
      summoner_name: '峡谷钢琴家杰斯',
      game_name: '峡谷钢琴家杰斯',
      tag_line: 'RED',
      team: 'red',
      champion_id: 126,
      champion_name: '未来守护者',
      champion_title: '杰斯',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/126.png',
      champion_level: 14,
      role: 'TOP',
      spell1_id: 12,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND I',
      kills: 3,
      deaths: 8,
      assists: 4,
      kda_str: '0.9',
      kda_ratio: 0.9,
      is_mvp: false,
      is_svp: false,
      total_damage: 17800,
      total_damage_taken: 26800,
      damage_taken: 26800,
      damage_share_pct: 22,
      damage_taken_share_pct: 25.5,
      is_highest_damage: false,
      damage_mitigated: 14000,
      gold_earned: 9800,
      minions_killed: 202,
      items: [3142, 3004, 3158, 0, 0, 0, 3340],
      item_trinket: 3340
    },
    {
      puuid: 'p-red-2',
      summoner_name: '豹女代练不说话',
      game_name: '豹女代练不说话',
      tag_line: 'VIP',
      team: 'red',
      champion_id: 76,
      champion_name: '狂野女猎手',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/76.png',
      champion_level: 15,
      role: 'JUNGLE',
      spell1_id: 11,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'MASTER I',
      kills: 9,
      deaths: 4,
      assists: 5,
      kda_str: '3.5',
      kda_ratio: 3.5,
      is_mvp: false,
      is_svp: true,
      total_damage: 28400,
      total_damage_taken: 21000,
      damage_taken: 21000,
      damage_share_pct: 35,
      damage_taken_share_pct: 20.0,
      is_highest_damage: true,
      damage_mitigated: 11000,
      gold_earned: 13200,
      minions_killed: 175,
      items: [3152, 3089, 3020, 3100, 0, 0, 3364],
      item_trinket: 3364
    },
    {
      puuid: 'p-red-3',
      summoner_name: '沙皇推土机',
      game_name: '沙皇推土机',
      tag_line: 'MID',
      team: 'red',
      champion_id: 268,
      champion_name: '沙漠皇帝',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/268.png',
      champion_level: 15,
      role: 'MIDDLE',
      spell1_id: 12,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND II',
      kills: 4,
      deaths: 7,
      assists: 6,
      kda_str: '1.4',
      kda_ratio: 1.4,
      is_mvp: false,
      is_svp: false,
      total_damage: 19500,
      total_damage_taken: 18400,
      damage_taken: 18400,
      damage_share_pct: 24,
      damage_taken_share_pct: 17.5,
      is_highest_damage: false,
      damage_mitigated: 15000,
      gold_earned: 11500,
      minions_killed: 230,
      items: [3115, 3020, 3157, 0, 0, 0, 3340],
      item_trinket: 3340
    },
    {
      puuid: 'p-red-4',
      summoner_name: '狂暴小黄毛EZ',
      game_name: '狂暴小黄毛EZ',
      tag_line: 'ADC',
      team: 'red',
      champion_id: 81,
      champion_name: '探险家',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/81.png',
      champion_level: 14,
      role: 'BOTTOM',
      spell1_id: 7,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerHeal.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND III',
      kills: 2,
      deaths: 7,
      assists: 7,
      kda_str: '1.3',
      kda_ratio: 1.3,
      is_mvp: false,
      is_svp: false,
      total_damage: 12400,
      total_damage_taken: 16900,
      damage_taken: 16900,
      damage_share_pct: 15,
      damage_taken_share_pct: 16.1,
      is_highest_damage: false,
      damage_mitigated: 9000,
      gold_earned: 9600,
      minions_killed: 195,
      items: [3078, 3042, 3158, 0, 0, 0, 3363],
      item_trinket: 3363
    },
    {
      puuid: 'p-red-5',
      summoner_name: '泰坦钩无虚发',
      game_name: '泰坦钩无虚发',
      tag_line: 'HOOK',
      team: 'red',
      champion_id: 111,
      champion_name: '深海泰坦',
      champion_icon: 'https://game.gtimg.cn/images/lol/act/img/champion/111.png',
      champion_level: 12,
      role: 'UTILITY',
      spell1_id: 14,
      spell1_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerDot.png',
      spell2_id: 4,
      spell2_icon: 'https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png',
      tier: 'DIAMOND IV',
      kills: 0,
      deaths: 9,
      assists: 8,
      kda_str: '0.9',
      kda_ratio: 0.9,
      is_mvp: false,
      is_svp: false,
      total_damage: 3200,
      total_damage_taken: 22100,
      damage_taken: 22100,
      damage_share_pct: 4,
      damage_taken_share_pct: 21.0,
      is_highest_damage: false,
      damage_mitigated: 26000,
      gold_earned: 6800,
      minions_killed: 28,
      items: [3865, 3111, 3075, 0, 0, 0, 3364],
      item_trinket: 3364
    }
  ]
};

export const mockAutomationConfig: AutomationConfig = {
  auto_accept_match: false,
  auto_accept_delay_seconds: 2,
  auto_swap_bench: false,
  auto_apply_runes: false,
  bench_preferred_champions: []
};

// ============================================================================
// LOSSLESS RAW LCU MOCK DATA PIPELINE (MILESTONE 2)
// Provides 100% faithful Riot LCU structures with genuine keystone runes,
// 1-18 skill level-up timestamps, and event kill positions.
// ============================================================================

export function buildRealisticMockRawGame(
  gameId: number,
  queueId = 420,
  isWin = true,
  duration = 1720,
  creation = Date.now(),
  seed = 0
): Game {
  const isAram = queueId === 450;
  const gameMode = isAram ? 'ARAM' : 'CLASSIC';
  const mapId = isAram ? 12 : 11;

  const blueCids = [266, 64, 103, 222, 117];
  const redCids = [126, 76, 268, 81, 111];
  const blueNames = ['飞天小女警', '野区养猪场长', '中路防御塔本体', '一枪一个小朋友', '带刀侍卫璐璐酱'];
  const redNames = ['峡谷钢琴家杰斯', '豹女代练不说话', '沙皇推土机', '狂暴小黄毛EZ', '泰坦钩无虚发'];
  const blueTags = ['CN1', 'EDG', 'MID', 'ADC', 'SUP'];
  const redTags = ['RED', 'VIP', 'MID', 'ADC', 'HOOK'];
  const roles = ['TOP', 'JUNGLE', 'MIDDLE', 'BOTTOM', 'UTILITY'];
  const lanes = ['TOP', 'JUNGLE', 'MIDDLE', 'BOTTOM', 'BOTTOM'];

  const identities: ParticipantIdentity[] = [];
  const participants: Participant[] = [];

  // Team 100 (Blue) - Participants 1..5
  for (let i = 0; i < 5; i++) {
    const pid = i + 1;
    const cid = isAram && i === 0 ? 84 : blueCids[i];
    const puuid = `p-blue-${pid}`;
    const name = blueNames[i];
    const tag = blueTags[i];

    identities.push({
      participantId: pid,
      player: {
        accountId: 1000 + pid,
        currentAccountId: 1000 + pid,
        currentPlatformId: 'HN1',
        matchHistoryUri: '',
        platformId: 'HN1',
        profileIcon: 500 + pid * 20,
        summonerId: 1000 + pid,
        puuid,
        gameName: name,
        tagLine: tag,
        summonerName: name
      }
    });

    const kills = isWin ? (8 + ((i + seed) % 4) * 2) : (2 + ((i + seed) % 3));
    const deaths = isWin ? (1 + (i % 2)) : (5 + (i % 3));
    const assists = isWin ? (7 + ((i + seed) % 5)) : (3 + (i % 3));
    const totalDmg = 18000 + (i * 4200) + (seed % 5000);
    const physDmg = Math.round(totalDmg * 0.7);
    const magicDmg = Math.round(totalDmg * 0.2);
    const trueDmg = totalDmg - physDmg - magicDmg;
    const dmgTaken = 16000 + (i * 3100);
    const gold = 11000 + (i * 1200);
    const cs = isAram ? (45 + i * 15) : (180 + i * 25);

    // Realistic keystone rune stats with genuine non-zero var1, var2, var3
    let perk0 = 8010; // Conqueror
    let perk0Var1 = 1845;
    let perk0Var2 = 920;
    let perk0Var3 = 0;
    let perk1 = 9111;
    let perk1Var1 = 1250;
    let perkPrimaryStyle = 8000;
    let perkSubStyle = 8400;

    if (i === 1) {
      perk0 = 8005; // PTA
      perk0Var1 = 1650;
      perk0Var2 = 820;
      perk0Var3 = 310;
      perkPrimaryStyle = 8000;
      perkSubStyle = 8100;
    } else if (i === 2) {
      perk0 = 8112; // Electrocute
      perk0Var1 = 2180;
      perk0Var2 = 0;
      perk0Var3 = 0;
      perkPrimaryStyle = 8100;
      perkSubStyle = 8200;
    } else if (i === 3) {
      perk0 = 8008; // Lethal Tempo
      perk0Var1 = 2450;
      perk0Var2 = 1120;
      perk0Var3 = 0;
      perkPrimaryStyle = 8000;
      perkSubStyle = 8200;
    } else if (i === 4) {
      perk0 = 8214; // Summon Aery
      perk0Var1 = 1890;
      perk0Var2 = 2420;
      perk0Var3 = 0;
      perkPrimaryStyle = 8200;
      perkSubStyle = 8400;
    }

    const items = [3074, 3053, 3111, 6631, 3071, 3026, 3340];

    const stats: Stats = {
      assists,
      causedEarlySurrender: false,
      champLevel: 14 + (i % 3),
      combatPlayerScore: 0,
      damageDealtToObjectives: 18500 - i * 1500,
      damageDealtToTurrets: 8200 - i * 1000,
      damageSelfMitigated: 35000 - i * 3000,
      deaths,
      doubleKills: isWin ? 2 : 0,
      earlySurrenderAccomplice: false,
      firstBloodAssist: isWin && i === 1,
      firstBloodKill: isWin && i === 0,
      firstInhibitorAssist: false,
      firstInhibitorKill: isWin && i === 0,
      firstTowerAssist: false,
      firstTowerKill: isWin && i === 0,
      gameEndedInEarlySurrender: false,
      gameEndedInSurrender: false,
      goldEarned: gold,
      goldSpent: gold - 800,
      inhibitorKills: isWin && i === 0 ? 1 : 0,
      item0: items[0],
      item1: items[1],
      item2: items[2],
      item3: items[3],
      item4: items[4],
      item5: items[5],
      item6: items[6],
      killingSprees: isWin ? 2 : 0,
      kills,
      largestCriticalStrike: 0,
      largestKillingSpree: isWin ? 5 : 1,
      largestMultiKill: isWin ? 3 : 1,
      longestTimeSpentLiving: Math.round(duration / 2),
      magicDamageDealt: magicDmg * 2,
      magicDamageDealtToChampions: magicDmg,
      magicalDamageTaken: Math.round(dmgTaken * 0.35),
      neutralMinionsKilled: i === 1 ? 120 : 12,
      neutralMinionsKilledEnemyJungle: i === 1 ? 24 : 0,
      neutralMinionsKilledTeamJungle: i === 1 ? 96 : 12,
      objectivePlayerScore: 0,
      participantId: pid,
      pentaKills: 0,
      perk0,
      perk0Var1,
      perk0Var2,
      perk0Var3,
      perk1,
      perk1Var1,
      perk1Var2: 300,
      perk1Var3: 0,
      perk2: 9104,
      perk2Var1: 18,
      perk2Var2: 0,
      perk2Var3: 0,
      perk3: 8299,
      perk3Var1: 1150,
      perk3Var2: 0,
      perk3Var3: 0,
      perk4: 8444,
      perk4Var1: 890,
      perk4Var2: 0,
      perk4Var3: 0,
      perk5: 8451,
      perk5Var1: 220,
      perk5Var2: 0,
      perk5Var3: 0,
      perkPrimaryStyle,
      perkSubStyle,
      physicalDamageDealt: physDmg * 2,
      physicalDamageDealtToChampions: physDmg,
      physicalDamageTaken: Math.round(dmgTaken * 0.55),
      playerAugment1: 0,
      playerAugment2: 0,
      playerAugment3: 0,
      playerAugment4: 0,
      playerAugment5: 0,
      playerAugment6: 0,
      playerScore0: 0,
      playerScore1: 0,
      playerScore2: 0,
      playerScore3: 0,
      playerScore4: 0,
      playerScore5: 0,
      playerScore6: 0,
      playerScore7: 0,
      playerScore8: 0,
      playerScore9: 0,
      playerSubteamId: 0,
      quadraKills: 0,
      roleBoundItem: 0,
      sightWardsBoughtInGame: 0,
      subteamPlacement: 0,
      teamEarlySurrendered: false,
      timeCCingOthers: 28 + i * 4,
      totalDamageDealt: totalDmg * 2,
      totalDamageDealtToChampions: totalDmg,
      totalDamageTaken: dmgTaken,
      totalHeal: 9800 - i * 1000,
      totalMinionsKilled: cs,
      totalPlayerScore: 0,
      totalScoreRank: 0,
      totalTimeCrowdControlDealt: 145 + i * 20,
      totalUnitsHealed: 1,
      tripleKills: 0,
      trueDamageDealt: trueDmg * 2,
      trueDamageDealtToChampions: trueDmg,
      trueDamageTaken: Math.round(dmgTaken * 0.1),
      turretKills: isWin && i === 0 ? 3 : 0,
      unrealKills: 0,
      visionScore: 38 - i * 2,
      visionWardsBoughtInGame: 3,
      wardsKilled: 5,
      wardsPlaced: 14,
      win: isWin
    };

    participants.push({
      championId: cid,
      highestAchievedSeasonTier: 'DIAMOND I',
      participantId: pid,
      spell1Id: i === 1 ? 11 : 12,
      spell2Id: 4,
      stats,
      teamId: 100,
      timeline: {
        lane: lanes[i],
        role: roles[i],
        participantId: pid,
        creepsPerMinDeltas: { '0-10': 7.5, '10-20': 8.2 },
        csDiffPerMinDeltas: { '0-10': 1.2, '10-20': 2.1 },
        damageTakenDiffPerMinDeltas: { '0-10': -50, '10-20': -80 },
        damageTakenPerMinDeltas: { '0-10': 450, '10-20': 620 },
        goldPerMinDeltas: { '0-10': 380, '10-20': 460 },
        xpDiffPerMinDeltas: { '0-10': 80, '10-20': 120 },
        xpPerMinDeltas: { '0-10': 480, '10-20': 550 }
      }
    });
  }

  // Team 200 (Red) - Participants 6..10
  for (let i = 0; i < 5; i++) {
    const pid = i + 6;
    const cid = redCids[i];
    const puuid = `p-red-${i + 1}`;
    const name = redNames[i];
    const tag = redTags[i];

    identities.push({
      participantId: pid,
      player: {
        accountId: 2000 + pid,
        currentAccountId: 2000 + pid,
        currentPlatformId: 'HN1',
        matchHistoryUri: '',
        platformId: 'HN1',
        profileIcon: 600 + pid * 15,
        summonerId: 2000 + pid,
        puuid,
        gameName: name,
        tagLine: tag,
        summonerName: name
      }
    });

    const kills = !isWin ? (7 + ((i + seed) % 4) * 2) : (2 + ((i + seed) % 2));
    const deaths = !isWin ? (1 + (i % 2)) : (6 + (i % 3));
    const assists = !isWin ? (6 + (i % 4)) : (2 + (i % 3));
    const totalDmg = 15000 + (i * 3800);
    const physDmg = Math.round(totalDmg * 0.65);
    const magicDmg = Math.round(totalDmg * 0.25);
    const trueDmg = totalDmg - physDmg - magicDmg;
    const dmgTaken = 17000 + (i * 2800);
    const gold = 9800 + (i * 1000);
    const cs = isAram ? (40 + i * 12) : (165 + i * 20);

    const stats: Stats = {
      assists,
      causedEarlySurrender: false,
      champLevel: 13 + (i % 3),
      combatPlayerScore: 0,
      damageDealtToObjectives: !isWin ? 14000 : 3000,
      damageDealtToTurrets: !isWin ? 6500 : 1200,
      damageSelfMitigated: 24000,
      deaths,
      doubleKills: !isWin ? 1 : 0,
      earlySurrenderAccomplice: false,
      firstBloodAssist: false,
      firstBloodKill: !isWin && i === 0,
      firstInhibitorAssist: false,
      firstInhibitorKill: false,
      firstTowerAssist: false,
      firstTowerKill: !isWin && i === 0,
      gameEndedInEarlySurrender: false,
      gameEndedInSurrender: false,
      goldEarned: gold,
      goldSpent: gold - 600,
      inhibitorKills: !isWin ? 1 : 0,
      item0: 3142,
      item1: 3004,
      item2: 3158,
      item3: 0,
      item4: 0,
      item5: 0,
      item6: 3340,
      killingSprees: !isWin ? 2 : 0,
      kills,
      largestCriticalStrike: 0,
      largestKillingSpree: !isWin ? 4 : 1,
      largestMultiKill: 1,
      longestTimeSpentLiving: Math.round(duration / 2.5),
      magicDamageDealt: magicDmg * 2,
      magicDamageDealtToChampions: magicDmg,
      magicalDamageTaken: Math.round(dmgTaken * 0.4),
      neutralMinionsKilled: i === 1 ? 95 : 8,
      neutralMinionsKilledEnemyJungle: 0,
      neutralMinionsKilledTeamJungle: i === 1 ? 95 : 8,
      objectivePlayerScore: 0,
      participantId: pid,
      pentaKills: 0,
      perk0: 8230,
      perk0Var1: 14,
      perk0Var2: 0,
      perk0Var3: 0,
      perk1: 8226,
      perk1Var1: 250,
      perk1Var2: 0,
      perk1Var3: 0,
      perk2: 8210,
      perk2Var1: 10,
      perk2Var2: 0,
      perk2Var3: 0,
      perk3: 8236,
      perk3Var1: 24,
      perk3Var2: 0,
      perk3Var3: 0,
      perk4: 8009,
      perk4Var1: 320,
      perk4Var2: 0,
      perk4Var3: 0,
      perk5: 8014,
      perk5Var1: 410,
      perk5Var2: 0,
      perk5Var3: 0,
      perkPrimaryStyle: 8200,
      perkSubStyle: 8000,
      physicalDamageDealt: physDmg * 2,
      physicalDamageDealtToChampions: physDmg,
      physicalDamageTaken: Math.round(dmgTaken * 0.5),
      playerAugment1: 0,
      playerAugment2: 0,
      playerAugment3: 0,
      playerAugment4: 0,
      playerAugment5: 0,
      playerAugment6: 0,
      playerScore0: 0,
      playerScore1: 0,
      playerScore2: 0,
      playerScore3: 0,
      playerScore4: 0,
      playerScore5: 0,
      playerScore6: 0,
      playerScore7: 0,
      playerScore8: 0,
      playerScore9: 0,
      playerSubteamId: 0,
      quadraKills: 0,
      roleBoundItem: 0,
      sightWardsBoughtInGame: 0,
      subteamPlacement: 0,
      teamEarlySurrendered: false,
      timeCCingOthers: 22,
      totalDamageDealt: totalDmg * 2,
      totalDamageDealtToChampions: totalDmg,
      totalDamageTaken: dmgTaken,
      totalHeal: 4500,
      totalMinionsKilled: cs,
      totalPlayerScore: 0,
      totalScoreRank: 0,
      totalTimeCrowdControlDealt: 110,
      totalUnitsHealed: 1,
      tripleKills: 0,
      trueDamageDealt: trueDmg * 2,
      trueDamageDealtToChampions: trueDmg,
      trueDamageTaken: Math.round(dmgTaken * 0.1),
      turretKills: !isWin ? 2 : 0,
      unrealKills: 0,
      visionScore: 24,
      visionWardsBoughtInGame: 1,
      wardsKilled: 2,
      wardsPlaced: 8,
      win: !isWin
    };

    participants.push({
      championId: cid,
      highestAchievedSeasonTier: 'DIAMOND II',
      participantId: pid,
      spell1Id: i === 1 ? 11 : 12,
      spell2Id: 4,
      stats,
      teamId: 200,
      timeline: {
        lane: lanes[i],
        role: roles[i],
        participantId: pid,
        creepsPerMinDeltas: { '0-10': 6.8, '10-20': 7.4 },
        csDiffPerMinDeltas: { '0-10': -1.2, '10-20': -2.1 },
        damageTakenDiffPerMinDeltas: { '0-10': 50, '10-20': 80 },
        damageTakenPerMinDeltas: { '0-10': 520, '10-20': 680 },
        goldPerMinDeltas: { '0-10': 340, '10-20': 410 },
        xpDiffPerMinDeltas: { '0-10': -80, '10-20': -120 },
        xpPerMinDeltas: { '0-10': 440, '10-20': 510 }
      }
    });
  }

  const teams: Team[] = [
    {
      bans: [{ championId: 119, pickTurn: 1 }, { championId: 157, pickTurn: 2 }, { championId: 238, pickTurn: 3 }],
      baronKills: isWin ? 1 : 0,
      dominionVictoryScore: 0,
      dragonKills: isWin ? 3 : 1,
      hordeKills: isWin ? 5 : 1,
      firstBaron: isWin,
      firstBlood: isWin,
      firstDargon: isWin,
      firstInhibitor: isWin,
      firstTower: isWin,
      inhibitorKills: isWin ? 2 : 0,
      riftHeraldKills: isWin ? 1 : 0,
      teamId: 100,
      towerKills: isWin ? 9 : 2,
      vilemawKills: 0,
      win: isWin ? 'Win' : 'Fail'
    },
    {
      bans: [{ championId: 104, pickTurn: 4 }, { championId: 122, pickTurn: 5 }, { championId: 84, pickTurn: 6 }],
      baronKills: !isWin ? 1 : 0,
      dominionVictoryScore: 0,
      dragonKills: !isWin ? 3 : 1,
      hordeKills: !isWin ? 5 : 1,
      firstBaron: !isWin,
      firstBlood: !isWin,
      firstDargon: !isWin,
      firstInhibitor: !isWin,
      firstTower: !isWin,
      inhibitorKills: !isWin ? 2 : 0,
      riftHeraldKills: !isWin ? 1 : 0,
      teamId: 200,
      towerKills: !isWin ? 9 : 2,
      vilemawKills: 0,
      win: !isWin ? 'Win' : 'Fail'
    }
  ];

  return {
    endOfGameResult: 'GameComplete',
    gameCreation: creation,
    gameCreationDate: new Date(creation).toISOString(),
    gameDuration: duration,
    gameId,
    gameMode,
    gameType: 'MATCHED_GAME',
    gameVersion: '14.20.1',
    mapId,
    participantIdentities: identities,
    participants,
    platformId: 'HN1',
    queueId,
    seasonId: 14,
    teams,
    gameModeMutators: []
  };
}

export const mockRawGames: Game[] = [
  buildRealisticMockRawGame(7012345671, 420, true, 1720, Date.now() - 1000 * 60 * 35, 1),
  buildRealisticMockRawGame(7012345672, 450, true, 1980, Date.now() - 1000 * 60 * 120, 2),
  buildRealisticMockRawGame(7012345673, 420, false, 1540, Date.now() - 1000 * 60 * 240, 3),
  buildRealisticMockRawGame(7012345674, 440, true, 1820, Date.now() - 1000 * 60 * 360, 4),
  buildRealisticMockRawGame(7012345675, 430, true, 1650, Date.now() - 1000 * 60 * 480, 5)
];

// Ensure mockMatchDetail contains raw_game
if (mockMatchDetail) {
  mockMatchDetail.raw_game = mockRawGames[0];
}

export const mockRawMatchHistory: MatchHistory = {
  accountId: 1001,
  platformId: 'HN1',
  games: {
    gameBeginDate: '',
    gameCount: mockRawGames.length,
    gameEndDate: '',
    gameIndexBegin: 0,
    gameIndexEnd: mockRawGames.length - 1,
    games: mockRawGames
  }
};

export function getMockRawMatchHistory(
  queueId?: number,
  begIndex = 0,
  endIndex = 19
): MatchHistory {
  const targetCount = Math.max(1, endIndex - begIndex + 1);
  const pool: Game[] = [];

  // Generate a large pool of games matching the request
  const countToGenerate = Math.max(targetCount + begIndex, 200);
  for (let idx = 0; idx < countToGenerate; idx++) {
    const gid = 7012345671 + idx;
    const qid = queueId ? queueId : (idx % 3 === 0 ? 450 : (idx % 4 === 0 ? 440 : 420));
    const isWin = (idx % 3 !== 0);
    const creation = Date.now() - idx * 3600000 * 2;
    pool.push(buildRealisticMockRawGame(gid, qid, isWin, 1600 + (idx % 400), creation, idx));
  }

  const filtered = queueId ? pool.filter(g => g.queueId === queueId) : pool;
  const sliced = filtered.slice(begIndex, endIndex + 1);

  return {
    accountId: 1001,
    platformId: 'HN1',
    games: {
      gameBeginDate: '',
      gameCount: sliced.length,
      gameEndDate: '',
      gameIndexBegin: begIndex,
      gameIndexEnd: begIndex + sliced.length - 1,
      games: sliced
    }
  };
}

export function buildRealisticMockRawTimeline(gameId: number): GameTimeline {
  const frames: TimelineFrame[] = [];
  const aatroxSkillSlots = [1, 3, 2, 1, 1, 4, 1, 3, 1, 3, 4, 3, 3, 2, 2, 4, 2, 2]; // Q, E, W, Q, Q, R, Q, E, Q, E, R, E, E, W, W, R, W, W

  for (let minute = 0; minute <= 18; minute++) {
    const timestamp = minute * 60000;
    const participantFrames: ParticipantFrames = {};

    for (let pid = 1; pid <= 10; pid++) {
      const isBlue = pid <= 5;
      const roleIdx = (pid - 1) % 5;
      const posX = isBlue ? (1000 + minute * 500 + roleIdx * 200) : (14000 - minute * 450 - roleIdx * 200);
      const posY = isBlue ? (1200 + minute * 400 + roleIdx * 300) : (13800 - minute * 400 - roleIdx * 300);

      participantFrames[String(pid)] = {
        participantId: pid,
        currentGold: (500 + minute * 420) % 1000,
        totalGold: 500 + minute * 450 + (isBlue ? minute * 40 : 0),
        level: 1 + Math.min(17, Math.floor((minute * 17) / 18)),
        xp: minute * 650,
        minionsKilled: roleIdx === 4 ? minute * 1 : (roleIdx === 1 ? minute * 4 : minute * 8),
        jungleMinionsKilled: roleIdx === 1 ? minute * 4 : 0,
        dominionScore: 0,
        teamScore: 0,
        position: { x: posX, y: posY }
      };
    }

    const events: GameEvent[] = [];

    // 1-18 Skill Level Ups for Participant 1 (and teammates)
    if (minute >= 1 && minute <= 18) {
      for (let pid = 1; pid <= 10; pid++) {
        events.push({
          type: 'SKILL_LEVEL_UP' as any,
          timestamp: timestamp + pid * 500,
          participantId: pid,
          skillSlot: aatroxSkillSlots[minute - 1] || 1,
          levelUpType: 'NORMAL' as any,
          position: { x: 0, y: 0 },
          killerId: 0,
          victimId: 0,
          assistingParticipantIds: [],
          teamId: pid <= 5 ? 100 : 200,
          itemId: 0,
          buildingType: '',
          towerType: '',
          laneType: '',
          monsterType: '',
          monsterSubType: ''
        } as any);
      }
    }

    // Item purchases at strategic minutes
    if (minute === 1) {
      for (let pid = 1; pid <= 10; pid++) {
        events.push({
          type: 'ITEM_PURCHASED' as any,
          timestamp: 65000 + pid * 1000,
          participantId: pid,
          itemId: (pid - 1) % 5 === 1 ? 1101 : 1055,
          position: { x: 500, y: 500 }
        } as any);
      }
    } else if (minute === 5) {
      events.push({
        type: 'ITEM_PURCHASED' as any,
        timestamp: 310000,
        participantId: 1,
        itemId: 1001,
        position: { x: 500, y: 500 }
      } as any);
    } else if (minute === 9) {
      events.push({
        type: 'ITEM_PURCHASED' as any,
        timestamp: 550000,
        participantId: 1,
        itemId: 3074,
        position: { x: 500, y: 500 }
      } as any);
    } else if (minute === 13) {
      events.push({
        type: 'ITEM_PURCHASED' as any,
        timestamp: 790000,
        participantId: 1,
        itemId: 3053,
        position: { x: 500, y: 500 }
      } as any);
    } else if (minute === 17) {
      events.push({
        type: 'ITEM_PURCHASED' as any,
        timestamp: 1030000,
        participantId: 1,
        itemId: 6631,
        position: { x: 500, y: 500 }
      } as any);
    }

    // Champion Kill Events with Authentic Kill Positions
    if (minute === 3) {
      events.push({
        type: 'CHAMPION_KILL',
        timestamp: 195000,
        participantId: 1,
        killerId: 1,
        victimId: 6,
        assistingParticipantIds: [2],
        position: { x: 2150, y: 11400 },
        skillSlot: 1,
        teamId: 100,
        itemId: 0,
        buildingType: '',
        towerType: '',
        laneType: '',
        monsterType: '',
        monsterSubType: ''
      });
    } else if (minute === 7) {
      events.push({
        type: 'CHAMPION_KILL',
        timestamp: 435000,
        participantId: 4,
        killerId: 4,
        victimId: 9,
        assistingParticipantIds: [2, 5],
        position: { x: 10400, y: 4900 },
        skillSlot: 1,
        teamId: 100,
        itemId: 0,
        buildingType: '',
        towerType: '',
        laneType: '',
        monsterType: '',
        monsterSubType: ''
      });
    } else if (minute === 11) {
      events.push({
        type: 'CHAMPION_KILL',
        timestamp: 675000,
        participantId: 1,
        killerId: 1,
        victimId: 7,
        assistingParticipantIds: [3],
        position: { x: 7200, y: 7100 },
        skillSlot: 3,
        teamId: 100,
        itemId: 0,
        buildingType: '',
        towerType: '',
        laneType: '',
        monsterType: '',
        monsterSubType: ''
      });
    } else if (minute === 15) {
      events.push({
        type: 'CHAMPION_KILL',
        timestamp: 915000,
        participantId: 4,
        killerId: 4,
        victimId: 8,
        assistingParticipantIds: [1, 2],
        position: { x: 8900, y: 9200 },
        skillSlot: 1,
        teamId: 100,
        itemId: 0,
        buildingType: '',
        towerType: '',
        laneType: '',
        monsterType: '',
        monsterSubType: ''
      });
    }

    // Building Kill Events
    if (minute === 12) {
      events.push({
        type: 'BUILDING_KILL',
        timestamp: 730000,
        participantId: 1,
        killerId: 1,
        victimId: 0,
        assistingParticipantIds: [2],
        buildingType: 'TOWER_BUILDING',
        towerType: 'OUTER_TURRET',
        laneType: 'TOP_LANE',
        teamId: 200,
        skillSlot: 0,
        itemId: 0,
        monsterType: '',
        monsterSubType: '',
        position: { x: 4318, y: 13875 }
      });
    }

    // Elite Monster Kill Events
    if (minute === 6) {
      events.push({
        type: 'ELITE_MONSTER_KILL',
        timestamp: 385000,
        killerId: 2,
        killerTeamId: 100,
        monsterType: 'DRAGON',
        position: { x: 9866, y: 4414 },
        assistingParticipantIds: [4, 5]
      });
    } else if (minute === 14) {
      events.push({
        type: 'ELITE_MONSTER_KILL',
        timestamp: 850000,
        killerId: 2,
        killerTeamId: 100,
        monsterType: 'RIFTHERALD',
        position: { x: 4900, y: 10400 },
        assistingParticipantIds: [1]
      });
    }

    frames.push({
      timestamp,
      events,
      participantFrames
    });
  }

  return {
    frames
  };
}

export const mockRawMatchTimeline: GameTimeline = buildRealisticMockRawTimeline(7012345671);

export function getMockRawMatchTimeline(gameId?: number): GameTimeline {
  return buildRealisticMockRawTimeline(gameId ?? 7012345671);
}

export const mockChampionMasteries = [
  {
    championId: 157, // 亚索
    championLevel: 7,
    championPoints: 268420,
    championPointsSinceLastLevel: 45000,
    championPointsUntilNextLevel: 0,
    chestGranted: true,
    formattedChampionPoints: '268,420',
    formattedMasteryGoal: '0',
    highestGrade: 'S+',
    lastPlayTime: 1729000000000,
    tokensEarned: 0,
    masteryMilestone: 8
  },
  {
    championId: 64, // 盲僧
    championLevel: 7,
    championPoints: 184500,
    championPointsSinceLastLevel: 22000,
    championPointsUntilNextLevel: 0,
    chestGranted: true,
    formattedChampionPoints: '184,500',
    formattedMasteryGoal: '0',
    highestGrade: 'S',
    lastPlayTime: 1728900000000,
    tokensEarned: 0,
    masteryMilestone: 6
  },
  {
    championId: 238, // 劫
    championLevel: 7,
    championPoints: 152300,
    championPointsSinceLastLevel: 15000,
    championPointsUntilNextLevel: 0,
    chestGranted: false,
    formattedChampionPoints: '152,300',
    formattedMasteryGoal: '0',
    highestGrade: 'S+',
    lastPlayTime: 1728800000000,
    tokensEarned: 0,
    masteryMilestone: 5
  },
  {
    championId: 81, // 伊泽瑞尔
    championLevel: 6,
    championPoints: 98400,
    championPointsSinceLastLevel: 18000,
    championPointsUntilNextLevel: 3600,
    chestGranted: true,
    formattedChampionPoints: '98,400',
    formattedMasteryGoal: '3,600',
    highestGrade: 'S-',
    lastPlayTime: 1728700000000,
    tokensEarned: 2,
    masteryMilestone: 4
  },
  {
    championId: 103, // 阿狸
    championLevel: 6,
    championPoints: 86200,
    championPointsSinceLastLevel: 12000,
    championPointsUntilNextLevel: 9800,
    chestGranted: false,
    formattedChampionPoints: '86,200',
    formattedMasteryGoal: '9,800',
    highestGrade: 'A+',
    lastPlayTime: 1728600000000,
    tokensEarned: 1,
    masteryMilestone: 3
  },
  {
    championId: 412, // 锤石
    championLevel: 5,
    championPoints: 64500,
    championPointsSinceLastLevel: 8000,
    championPointsUntilNextLevel: 12000,
    chestGranted: true,
    formattedChampionPoints: '64,500',
    formattedMasteryGoal: '12,000',
    highestGrade: 'S',
    lastPlayTime: 1728500000000,
    tokensEarned: 0,
    masteryMilestone: 3
  },
  {
    championId: 222, // 金克丝
    championLevel: 5,
    championPoints: 51200,
    championPointsSinceLastLevel: 5000,
    championPointsUntilNextLevel: 15000,
    chestGranted: false,
    formattedChampionPoints: '51,200',
    formattedMasteryGoal: '15,000',
    highestGrade: 'A',
    lastPlayTime: 1728400000000,
    tokensEarned: 0,
    masteryMilestone: 2
  },
  {
    championId: 266, // 亚托克斯
    championLevel: 5,
    championPoints: 42100,
    championPointsSinceLastLevel: 3000,
    championPointsUntilNextLevel: 17000,
    chestGranted: true,
    formattedChampionPoints: '42,100',
    formattedMasteryGoal: '17,000',
    highestGrade: 'S-',
    lastPlayTime: 1728300000000,
    tokensEarned: 0,
    masteryMilestone: 2
  },
  {
    championId: 777, // 永恩
    championLevel: 4,
    championPoints: 34800,
    championPointsSinceLastLevel: 4000,
    championPointsUntilNextLevel: 8000,
    chestGranted: false,
    formattedChampionPoints: '34,800',
    formattedMasteryGoal: '8,000',
    highestGrade: 'B+',
    lastPlayTime: 1728200000000,
    tokensEarned: 0,
    masteryMilestone: 1
  },
  {
    championId: 145, // 卡莎
    championLevel: 4,
    championPoints: 29600,
    championPointsSinceLastLevel: 2500,
    championPointsUntilNextLevel: 9500,
    chestGranted: true,
    formattedChampionPoints: '29,600',
    formattedMasteryGoal: '9,500',
    highestGrade: 'S',
    lastPlayTime: 1728100000000,
    tokensEarned: 0,
    masteryMilestone: 1
  },
  {
    championId: 92, // 锐雯
    championLevel: 4,
    championPoints: 24100,
    championPointsSinceLastLevel: 1200,
    championPointsUntilNextLevel: 10800,
    chestGranted: false,
    formattedChampionPoints: '24,100',
    formattedMasteryGoal: '10,800',
    highestGrade: 'A-',
    lastPlayTime: 1728000000000,
    tokensEarned: 0,
    masteryMilestone: 1
  },
  {
    championId: 84, // 阿卡丽
    championLevel: 3,
    championPoints: 16500,
    championPointsSinceLastLevel: 3500,
    championPointsUntilNextLevel: 4500,
    chestGranted: false,
    formattedChampionPoints: '16,500',
    formattedMasteryGoal: '4,500',
    highestGrade: 'B',
    lastPlayTime: 1727900000000,
    tokensEarned: 0,
    masteryMilestone: 0
  },
  {
    championId: 517, // 塞拉斯
    championLevel: 3,
    championPoints: 14200,
    championPointsSinceLastLevel: 1200,
    championPointsUntilNextLevel: 6800,
    chestGranted: false,
    formattedChampionPoints: '14,200',
    formattedMasteryGoal: '6,800',
    highestGrade: 'A',
    lastPlayTime: 1727800000000,
    tokensEarned: 0,
    masteryMilestone: 0
  },
  {
    championId: 99, // 拉克丝
    championLevel: 3,
    championPoints: 12100,
    championPointsSinceLastLevel: 800,
    championPointsUntilNextLevel: 7200,
    chestGranted: true,
    formattedChampionPoints: '12,100',
    formattedMasteryGoal: '7,200',
    highestGrade: 'S-',
    lastPlayTime: 1727700000000,
    tokensEarned: 0,
    masteryMilestone: 0
  },
  {
    championId: 67, // 薇恩
    championLevel: 2,
    championPoints: 7800,
    championPointsSinceLastLevel: 1800,
    championPointsUntilNextLevel: 4200,
    chestGranted: false,
    formattedChampionPoints: '7,800',
    formattedMasteryGoal: '4,200',
    highestGrade: 'C+',
    lastPlayTime: 1727600000000,
    tokensEarned: 0,
    masteryMilestone: 0
  }
];

export function getMockChampionMastery(): any[] {
  return mockChampionMasteries;
}

