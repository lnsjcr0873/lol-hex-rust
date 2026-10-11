export interface TacticalBadge {
  id: string;
  label: string;
  icon: string;
  color: 'gold' | 'cyan' | 'red' | 'blue' | 'purple';
  description?: string;
}

export interface PremadeParty {
  party_id: number;
  label: string;
  color_class: string;
}

export interface RadarStat {
  output: number;      // 0-100
  kda: number;         // 0-100
  join_rate: number;   // 0-100
  survival: number;    // 0-100
  gold: number;        // 0-100
  vision: number;      // 0-100
}

export interface PlayerCard {
  puuid: string;
  summoner_id: string;
  game_name: string;
  tag_line: string;
  profile_icon_id: number;
  summoner_level: number;
  team: 'blue' | 'red';
  cell_id: number;
  assigned_position: string;
  champion_id: number;
  champion_name: string;
  champion_title: string;
  champion_icon: string;
  is_locked: boolean;
  spell1_id: number;
  spell1_name: string;
  spell1_icon: string;
  spell2_id: number;
  spell2_name: string;
  spell2_icon: string;
  tier: string;
  division: string;
  lp: number;
  recent_kda: string;
  recent_kda_ratio: number;
  recent_win_rate: number;
  recent_games_count: number;
  champ_recent_games: number;
  champ_recent_win_rate: number;
  recent_matches: ('win' | 'loss')[];
  badges: TacticalBadge[];
  premade_party?: PremadeParty;
  radar: RadarStat;
}

export interface OngoingMatchData {
  gameflow_phase: string;
  queue_id: number;
  queue_name: string;
  blue_team: PlayerCard[];
  red_team: PlayerCard[];
  is_my_team_blue: boolean;
  my_puuid: string;
  chat_room_id?: string;
}

export interface SummonerProfile {
  puuid: string;
  game_name: string;
  tag_line: string;
  profile_icon_id: number;
  summoner_level: number;
  tier_solo: string;
  division_solo: string;
  lp_solo: number;
  wins_solo: number;
  losses_solo: number;
  win_rate_solo: number;
  tier_flex: string;
  division_flex: string;
  lp_flex: number;
  wins_flex: number;
  losses_flex: number;
  win_rate_flex: number;
  recent_radar: RadarStat;
  recent_summary: {
    total_games: number;
    win_rate: number;
    avg_kda: string;
    avg_kda_ratio: number;
    top_champions: {
      champion_id: number;
      champion_name: string;
      champion_icon: string;
      games: number;
      win_rate: number;
      kda: string;
    }[];
  };
}

export interface MatchHistoryItem {
  game_id: number;
  game_creation: number;
  game_duration_seconds: number;
  game_mode: string;
  queue_id: number;
  queue_name: string;
  is_win: boolean;
  champion_id: number;
  champion_name: string;
  champion_icon: string;
  role: string;
  kills: number;
  deaths: number;
  assists: number;
  kda_ratio: number;
  is_mvp: boolean;
  is_svp: boolean;
  items: number[];
  total_damage: number;
  damage_share_pct: number;
  damage_taken: number;
  gold_earned: number;
  minions_killed: number;
  participants_preview: {
    champion_id: number;
    champion_name: string;
    champion_icon: string;
    game_name: string;
    is_win: boolean;
  }[];
}

export interface MatchDetailParticipant {
  puuid: string;
  game_name: string;
  tag_line: string;
  team: 'blue' | 'red';
  champion_id: number;
  champion_name: string;
  champion_title?: string;
  champion_icon: string;
  champion_level: number;
  role: string;
  spell1_id: number;
  spell1_icon: string;
  spell2_id: number;
  spell2_icon: string;
  tier: string;
  kills: number;
  deaths: number;
  assists: number;
  kda_ratio: number;
  is_mvp: boolean;
  is_svp: boolean;
  total_damage: number;
  damage_share_pct: number;
  is_highest_damage: boolean;
  damage_taken: number;
  damage_mitigated: number;
  gold_earned: number;
  minions_killed: number;
  items: number[];
  item_trinket: number;
}

export interface MatchDetailFull {
  game_id: number;
  game_creation: number;
  game_duration: number;
  queue_id: number;
  queue_name: string;
  blue_team_kills: number;
  blue_team_turrets: number;
  blue_team_dragons: number;
  blue_team_barons: number;
  blue_team_win: boolean;
  red_team_kills: number;
  red_team_turrets: number;
  red_team_dragons: number;
  red_team_barons: number;
  red_team_win: boolean;
  participants: MatchDetailParticipant[];
}

export interface AutomationConfig {
  auto_accept_match: boolean;
  auto_accept_delay_seconds: number;
  auto_swap_bench: boolean;
  auto_apply_runes: boolean;
  bench_preferred_champions: string[];
}
