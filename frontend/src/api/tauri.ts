import { 
  OngoingMatchData, 
  SummonerProfile, 
  MatchHistoryItem, 
  MatchDetailFull, 
  AutomationConfig,
  MatchTimelineData
} from './types';
import type { MatchHistory, GameTimeline } from '@shared/types/league-client/match-history';
import { 
  mockOngoingMatch, 
  mockSummonerProfile, 
  mockMatchHistory, 
  mockMatchDetail, 
  mockAutomationConfig,
  getMockRawMatchHistory,
  getMockRawMatchTimeline
} from './mockData';

// Safe check whether we are running inside Tauri
export const isTauri = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

async function safeInvoke<T>(command: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (isTauri()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(command, args);
    } catch (err) {
      console.warn(`[Tauri IPC] '${command}' failed, falling back to mock:`, err);
      if (fallback !== undefined) return fallback;
      throw err;
    }
  } else {
    // Web dev mode fallback
    if (fallback !== undefined) {
      return fallback;
    }
    throw new Error(`Tauri environment not detected and no fallback provided for '${command}'`);
  }
}

// Window actions
export async function minimizeWindow(): Promise<void> {
  if (isTauri()) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().minimize();
  } else {
    console.log('[Dev] minimize window');
  }
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (isTauri()) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().toggleMaximize();
  } else {
    console.log('[Dev] toggle maximize window');
  }
}

export async function closeWindow(): Promise<void> {
  if (isTauri()) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  } else {
    console.log('[Dev] close window');
  }
}

export async function startDragging(): Promise<void> {
  if (isTauri()) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().startDragging();
  }
}

// LCU & Gameplay Commands
export async function getOngoingMatch(forceMock?: boolean): Promise<OngoingMatchData> {
  return safeInvoke<OngoingMatchData>(
    'get_ongoing_match',
    { forceMock: forceMock ?? null },
    forceMock ? mockOngoingMatch : {
      gameflow_phase: 'None',
      queue_id: 0,
      queue_name: '英雄联盟未连接',
      blue_team: [],
      red_team: [],
      is_my_team_blue: true,
      my_puuid: '',
      lcu_connected: false,
      is_active: false,
      is_mock: false,
      message: '英雄联盟客户端未连接（请先启动游戏并登录）'
    }
  );
}

export const emptySummonerProfile: SummonerProfile = {
  puuid: '',
  game_name: '未连接客户端',
  tag_line: '',
  profile_icon_id: 29,
  summoner_level: 0,
  tier_solo: 'UNRANKED',
  division_solo: '',
  lp_solo: 0,
  wins_solo: 0,
  losses_solo: 0,
  win_rate_solo: 0,
  tier_flex: 'UNRANKED',
  division_flex: '',
  lp_flex: 0,
  wins_flex: 0,
  losses_flex: 0,
  win_rate_flex: 0,
  is_connected: false,
  message: '客户端未连接'
};

export async function getSummonerProfile(puuid?: string): Promise<SummonerProfile> {
  return safeInvoke<SummonerProfile>('get_summoner_profile', { puuid }, emptySummonerProfile);
}

export async function getMatchHistory(
  puuid?: string,
  queueId?: number,
  begIndex?: number,
  endIndex?: number,
  page?: number,
  pageSize?: number
): Promise<MatchHistoryItem[]> {
  return safeInvoke<MatchHistoryItem[]>(
    'get_match_history',
    {
      puuid,
      queueId: queueId ?? null,
      begIndex,
      endIndex,
      page,
      pageSize,
      queue_id: queueId ?? null,
      beg_index: begIndex,
      end_index: endIndex,
      page_size: pageSize
    },
    []
  );
}

export async function getRawMatchHistory(
  puuid?: string,
  queueId?: number,
  begIndex?: number,
  endIndex?: number
): Promise<MatchHistory> {
  return safeInvoke<MatchHistory>(
    'get_raw_match_history',
    {
      puuid,
      queueId: queueId ?? null,
      queue_id: queueId ?? null,
      begIndex,
      beg_index: begIndex,
      endIndex,
      end_index: endIndex
    },
    getMockRawMatchHistory(queueId, begIndex, endIndex)
  );
}

export async function getRawMatchTimeline(gameId: number): Promise<GameTimeline> {
  return safeInvoke<GameTimeline>(
    'get_raw_match_timeline',
    {
      gameId,
      game_id: gameId
    },
    getMockRawMatchTimeline(gameId)
  );
}

export async function getRawMatchDetail(gameId: number): Promise<any> {
  return safeInvoke<any>(
    'get_raw_match_detail',
    {
      gameId,
      game_id: gameId
    },
    null
  );
}

export async function getMatchDetail(gameId: number): Promise<MatchDetailFull | null> {
  return safeInvoke<MatchDetailFull | null>('get_match_detail', { gameId }, null);
}

export async function getMatchTimeline(gameId: number): Promise<MatchTimelineData | null> {
  return safeInvoke<MatchTimelineData | null>('get_match_timeline', { gameId }, null);
}

export { getMockRawMatchHistory, getMockRawMatchTimeline };

export function getMockSummonerProfile(): SummonerProfile {
  return mockSummonerProfile;
}

export function getMockMatchHistory(): MatchHistoryItem[] {
  return mockMatchHistory;
}

export function getMockMatchDetail(): MatchDetailFull {
  return mockMatchDetail;
}

export async function sendChatRecords(roomId: string, format: 'concise' | 'detailed'): Promise<{ success: boolean; message: string }> {
  return safeInvoke('send_chat_records', { roomId, format }, { success: false, message: '客户端未连接，无法发送战绩' });
}

export async function getAutomationConfig(): Promise<AutomationConfig> {
  return safeInvoke<AutomationConfig>('get_automation_config', undefined, mockAutomationConfig);
}

export async function updateAutomationConfig(config: AutomationConfig): Promise<{ success: boolean }> {
  return safeInvoke('update_automation_config', { config }, { success: true });
}

export async function getChampionMastery(puuid?: string): Promise<any[]> {
  const res = await safeInvoke<any[]>('get_champion_mastery', { puuid: puuid || '' }, []);
  if (Array.isArray(res) && res.length > 0) {
    return res;
  }
  // If demo puuid or empty result while running in demo mode
  if (puuid && (puuid.startsWith('mock_') || puuid.includes('demo'))) {
    const { getMockChampionMastery } = await import('./mockData');
    return getMockChampionMastery();
  }
  return res || [];
}

export async function getPlayerChallenges(puuid?: string): Promise<any> {
  const fallback = {
    playerChallenges: [
      { id: 505001, currentValue: 168 },
      { id: 510001, currentValue: 145 },
      { id: 510011, currentValue: 38 },
      { id: 504003, currentValue: 12 },
      { id: 504002, currentValue: 88 },
      { id: 504004, currentValue: 45 }
    ]
  };
  return safeInvoke<any>('get_player_challenges', { puuid: puuid || '' }, fallback);
}

export async function searchSummoner(name: string): Promise<any> {
  return safeInvoke('search_summoner', { name }, null);
}

export async function applyRunesAndSpells(championId: number): Promise<{ success: boolean; message: string }> {
  return safeInvoke('apply_runes_and_spells', { championId }, { success: false, message: '客户端未连接，无法配置符文' });
}

export async function toggleOngoingOverlay(): Promise<boolean> {
  return safeInvoke<boolean>('toggle_ongoing_overlay', undefined, false);
}

export async function setOngoingOverlayVisible(visible: boolean): Promise<void> {
  return safeInvoke<void>('set_ongoing_overlay_visible', { visible }, null as any);
}

export async function setOngoingOverlayAlwaysOnTop(alwaysOnTop: boolean): Promise<void> {
  return safeInvoke<void>('set_ongoing_overlay_always_on_top', { alwaysOnTop }, null as any);
}

export async function isOngoingOverlayVisible(): Promise<boolean> {
  return safeInvoke<boolean>('is_ongoing_overlay_visible', undefined, false);
}

export { getMockChampionMastery } from './mockData';

