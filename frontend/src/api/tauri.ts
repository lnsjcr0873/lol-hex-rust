import { 
  OngoingMatchData, 
  SummonerProfile, 
  MatchHistoryItem, 
  MatchDetailFull, 
  AutomationConfig 
} from './types';
import { 
  mockOngoingMatch, 
  mockSummonerProfile, 
  mockMatchHistory, 
  mockMatchDetail, 
  mockAutomationConfig 
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

export async function getSummonerProfile(puuid?: string): Promise<SummonerProfile> {
  return safeInvoke<SummonerProfile>('get_summoner_profile', { puuid }, mockSummonerProfile);
}

export async function getMatchHistory(puuid?: string, queueId?: number): Promise<MatchHistoryItem[]> {
  return safeInvoke<MatchHistoryItem[]>('get_match_history', { puuid, queueId }, []);
}

export async function getMatchDetail(gameId: number): Promise<MatchDetailFull | null> {
  return safeInvoke<MatchDetailFull | null>('get_match_detail', { gameId }, null);
}

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
  return safeInvoke('send_chat_records', { roomId, format }, { success: true, message: '战绩已广播至选人聊天室' });
}

export async function getAutomationConfig(): Promise<AutomationConfig> {
  return safeInvoke<AutomationConfig>('get_automation_config', undefined, mockAutomationConfig);
}

export async function updateAutomationConfig(config: AutomationConfig): Promise<{ success: boolean }> {
  return safeInvoke('update_automation_config', { config }, { success: true });
}

export async function applyRunesAndSpells(championId: number): Promise<{ success: boolean; message: string }> {
  return safeInvoke('apply_runes_and_spells', { championId }, { success: true, message: '推荐符文与召唤师技能已自动应用' });
}
