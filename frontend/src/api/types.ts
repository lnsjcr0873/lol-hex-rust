export interface TacticalBadge {
  id: string;
  label: string;
  icon: string;
  color: 'gold' | 'cyan' | 'red' | 'blue' | 'purple' | 'green';
  description?: string;
}

export interface PremadeParty {
  party_id: number;
  label: string;
  color_class: string;
}

export interface RadarStat {
  output?: number;      // 0-100
  damage?: number;      // 0-100 (alias for output)
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
  top_champs?: TopChampItem[];
  recent_matches: ('win' | 'loss')[];
  matches?: any[];
  badges: TacticalBadge[];
  premade_party?: PremadeParty;
  radar: RadarStat;
  is_autofilled?: boolean;
  team_participant_id?: number | null;
}

export type BroadcastPreset = 'concise' | 'detailed' | 'jungle' | 'premade';

export interface ChatBroadcastResponse {
  success: boolean;
  message: string;
}

export interface TopChampItem {
  champion_id: number;
  champion_name: string;
  games: number;
  win_rate: number;
  win_rate_pct: number;
  kda: number;
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
  lcu_connected?: boolean;
  is_active?: boolean;
  is_mock?: boolean;
  message?: string;
}

export interface LcuStatus {
  is_connected: boolean;
  gameflow_phase: string;
  puuid: string;
  game_name: string;
  tag_line: string;
  profile_icon_id: number;
  summoner_level: number;
}

export interface SummonerProfile {
  is_connected?: boolean;
  is_mock?: boolean;
  privacy?: string;
  message?: string;
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
  recent_radar?: RadarStat;
  recent_summary?: {
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
  team_id?: number;
  is_blue?: boolean;
  champion_id: number;
  champion_name: string;
  champion_icon: string;
  role: string;
  role_cn?: string;
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
  spell1_id?: number;
  spell2_id?: number;
  physical_damage?: number;
  magic_damage?: number;
  true_damage?: number;
  primary_rune?: number;
  secondary_rune?: number;
  participants_preview: ParticipantPreviewItem[];
}

export interface ParticipantPreviewItem {
  participant_id?: number;
  puuid?: string;
  summoner_name?: string;
  game_name?: string;
  tag_line?: string;
  team_id?: number;
  is_blue?: boolean;
  champion_id: number;
  champion_name: string;
  champion_icon: string;
  is_win: boolean;
  kills?: number;
  deaths?: number;
  assists?: number;
  spell1_id?: number;
  spell2_id?: number;
}

export interface MatchDetailParticipant {
  puuid: string;
  summoner_name: string;
  game_name?: string;
  tag_line?: string;
  team: 'blue' | 'red';
  champion_id: number;
  champion_name: string;
  champion_title?: string;
  champion_icon?: string;
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
  kda_str: string;
  kda_ratio?: number;
  total_damage: number;
  total_damage_taken: number;
  damage_taken?: number;
  damage_share_pct: number;
  damage_taken_share_pct: number;
  is_highest_damage: boolean;
  is_mvp: boolean;
  is_svp: boolean;
  gold_earned: number;
  minions_killed: number;
  items: number[];
  item_trinket?: number;
  damage_mitigated?: number;
}

export interface MatchDetailFull {
  game_id: number;
  game_creation: number;
  game_duration: number;
  game_mode: string;
  game_mode_cn: string;
  queue_id: number;
  queue_name?: string;
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
  raw_game?: any;
}

export type MatchTimelineData = Record<string, any>;

export interface AutomationConfig {
  auto_accept_match: boolean;
  auto_accept_delay_seconds: number;
  auto_swap_bench: boolean;
  auto_apply_runes: boolean;
  bench_preferred_champions: string[];
}

export interface SavedPlayerTagRecord {
  puuid: string;
  selfPuuid: string;
  region: string;
  rsoPlatformId: string;
  tag: string | null;
  updateAt: number;
  lastMetAt?: number | null;
}

export interface SavedPlayerTagItem {
  puuid: string;
  selfPuuid: string;
  region: string;
  rsoPlatformId: string;
  tag: string;
  updateAt: number;
  lastMetAt?: number | null;
  markedBySelf: boolean;
}

export interface PaginatedPlayerTags {
  data: SavedPlayerTagRecord[];
  page: number;
  pageSize: number;
  total: number;
  totalPages?: number;
}

export interface UpdatePlayerTagDto {
  puuid: string;
  selfPuuid?: string;
  tag?: string | null;
  region?: string | null;
  rsoPlatformId?: string | null;
}

export interface QueryAllPlayerTagsDto {
  puuid?: string;
  selfPuuid?: string;
  region?: string;
  rsoPlatformId?: string;
  page?: number;
  pageSize?: number;
  timeOrder?: 'asc' | 'desc';
}

export interface EncounteredGameRecord {
  id: number;
  gameId: number;
  puuid: string;
  selfPuuid: string;
  region: string;
  rsoPlatformId: string;
  updateAt: number;
  queueType: string;
}

export interface PaginatedEncounteredGames {
  data: EncounteredGameRecord[];
  page: number;
  pageSize: number;
  total: number;
  totalPages?: number;
}

export interface QueryEncounteredGamesDto {
  puuid: string;
  selfPuuid?: string;
  queueType?: string;
  page?: number;
  pageSize?: number;
  timeOrder?: 'asc' | 'desc';
}

export interface GenericResult {
  success: boolean;
  message?: string | null;
}

export interface DraftOptions {
  gameModeKind: 'cherry' | 'normal';
  queueId: number;
  puuid: string | null;
  teams: Record<string, string[]>;
  championSelections: Record<string, number>;
  positions: Record<
    string,
    {
      selected: string;
      primary: string;
      secondary: string;
    }
  > | null;
}

