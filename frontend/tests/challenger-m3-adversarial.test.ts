import { describe, it, expect, beforeEach } from 'vitest';
import {
  FIXED_CARD_WIDTH_PX_NUMBER,
  WIN_RATE_TEAM_MIN_MATCHES,
  WIN_RATE_TEAM_MIN_SIZE,
  WIN_RATE_TEAM_MIN_WIN_RATE,
  WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK,
  LOSS_RATE_TEAM_MIN_SIZE,
  LOSS_RATE_TEAM_MAX_WIN_RATE
} from '../src/components/ongoing-game-panel/constants';
import type { PlayerCard, BroadcastPreset } from '../src/api/types';

describe('Adversarial M3 Challenger: Multi-Column Calculation at Breakpoint Boundaries', () => {
  // Akari and HexBrain formula:
  // [8, 7, 6, 5, 4, 3].find(col => contentWidth > FIXED_CARD_WIDTH_PX_NUMBER * (col + 0.25))
  // FIXED_CARD_WIDTH_PX_NUMBER = 240
  // Thresholds:
  // col 8: 240 * 8.25 = 1980px
  // col 7: 240 * 7.25 = 1740px
  // col 6: 240 * 6.25 = 1500px
  // col 5: 240 * 5.25 = 1260px
  // col 4: 240 * 4.25 = 1020px
  // col 3: 240 * 3.25 = 780px

  const calculateColumns = (contentWidth: number, teamSize = 8) => {
    const maxTeamMembers = Math.max(teamSize, 5);
    const maxAllowed = [8, 7, 6, 5, 4, 3].find(
      (col) => contentWidth > FIXED_CARD_WIDTH_PX_NUMBER * (col + 0.25)
    );
    return Math.min(maxAllowed || 3, maxTeamMembers);
  };

  it('verifies exact breakpoint boundaries for 8-player teams', () => {
    // Extreme sub-780px
    expect(calculateColumns(0)).toBe(3);
    expect(calculateColumns(500)).toBe(3);
    expect(calculateColumns(780)).toBe(3); // exactly at 780px (> 780 is false)

    // 780px vs 781px
    expect(calculateColumns(781)).toBe(3); // > 780 is true -> 3 columns

    // 1020px vs 1021px
    expect(calculateColumns(1020)).toBe(3); // <= 1020 is 3
    expect(calculateColumns(1021)).toBe(4); // > 1020 is 4

    // 1260px vs 1261px
    expect(calculateColumns(1260)).toBe(4); // <= 1260 is 4
    expect(calculateColumns(1261)).toBe(5); // > 1260 is 5

    // 1500px vs 1501px
    expect(calculateColumns(1500)).toBe(5); // <= 1500 is 5
    expect(calculateColumns(1501)).toBe(6); // > 1500 is 6

    // 1740px vs 1741px
    expect(calculateColumns(1740)).toBe(6); // <= 1740 is 6
    expect(calculateColumns(1741)).toBe(7); // > 1741 is 7

    // 1980px vs 1981px
    expect(calculateColumns(1980)).toBe(7); // <= 1980 is 7
    expect(calculateColumns(1981)).toBe(8); // > 1980 is 8

    // Ultra-wide 4K monitor (3840px)
    expect(calculateColumns(3840)).toBe(8);
  });

  it('evaluates standard 5v5 team behavior against wide displays', () => {
    // In standard 5v5, teamSize is 5.
    // If team has only 5 players, what does calculateColumns return at 4K?
    const colsAt4k = calculateColumns(3840, 5);
    // Because maxTeamMembers is 5, Math.min(8, 5) is 5.
    expect(colsAt4k).toBe(5);
  });

  it('guarantees 0-player arrays do not cause NaN or division by zero in linesPerTeam', () => {
    const computeLinesPerTeam = (blueLen: number, redLen: number, width: number) => {
      const maxMembers = Math.max(blueLen, redLen, 5);
      const cols = calculateColumns(width, Math.max(blueLen, redLen));
      if (!cols) return 1;
      return Math.ceil(maxMembers / cols);
    };

    expect(computeLinesPerTeam(0, 0, 500)).toBe(2); // 5 / 3 = 1.66 -> 2
    expect(computeLinesPerTeam(0, 0, 1981)).toBe(1); // 5 / 5 = 1
    expect(Number.isFinite(computeLinesPerTeam(0, 0, 0))).toBe(true);
  });
});

describe('Adversarial M3 Challenger: Win-Rate & Loss-Rate Team Badges Empirical Logic', () => {
  // Mirroring OngoingGamePanel.vue's exact internal detection logic:
  function getPlayerWinningStreak(player: Partial<PlayerCard>): number {
    if (Array.isArray(player.recent_matches)) {
      let streak = 0;
      for (const m of player.recent_matches) {
        if (m === 'win') streak++;
        else break;
      }
      return streak;
    }
    return 0;
  }

  function detectTeamBadgesCurrentImplementation(members: Partial<PlayerCard>[]) {
    let winRateTeamInfo: { type: 'win-rate-team' | 'loss-rate-team' } | undefined = undefined;

    // 1. Check "胜率队"
    if (members.length >= WIN_RATE_TEAM_MIN_SIZE) {
      let hasOneHighWinRateMember = false;
      let otherMembersWinTotalStreak = 0;

      for (const m of members) {
        const wr = (m.recent_win_rate ?? 0) > 1 ? (m.recent_win_rate ?? 0) / 100 : (m.recent_win_rate ?? 0);
        const count = m.recent_games_count ?? 0;
        const streak = getPlayerWinningStreak(m);

        if (!hasOneHighWinRateMember && wr >= WIN_RATE_TEAM_MIN_WIN_RATE && count >= WIN_RATE_TEAM_MIN_MATCHES) {
          hasOneHighWinRateMember = true;
        } else {
          otherMembersWinTotalStreak += streak;
        }
      }

      const avgStreak = members.length > 1 ? otherMembersWinTotalStreak / (members.length - 1) : 0;
      const squadAvgWr = members.reduce((sum, m) => sum + (m.recent_win_rate ?? 0), 0) / members.length;

      if ((hasOneHighWinRateMember && avgStreak >= WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK) || squadAvgWr >= 80) {
        winRateTeamInfo = { type: 'win-rate-team' };
      }
    }

    // 2. Check "败率队"
    if (!winRateTeamInfo && members.length >= LOSS_RATE_TEAM_MIN_SIZE) {
      let allLossQualified = true;
      for (const m of members) {
        const wr = (m.recent_win_rate ?? 0) > 1 ? (m.recent_win_rate ?? 0) / 100 : (m.recent_win_rate ?? 0);
        if (wr > LOSS_RATE_TEAM_MAX_WIN_RATE) {
          allLossQualified = false;
          break;
        }
      }
      const squadAvgWr = members.reduce((sum, m) => sum + (m.recent_win_rate ?? 0), 0) / members.length;
      if (allLossQualified || squadAvgWr <= 25) {
        winRateTeamInfo = { type: 'loss-rate-team' };
      }
    }

    return winRateTeamInfo;
  }

  it('tests win rate team boundary: 79% vs 80% win rate', () => {
    // 79% win rate squad (no high streak)
    const squad79: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 79, recent_games_count: 20, recent_matches: ['loss'] },
      { puuid: '2', recent_win_rate: 79, recent_games_count: 20, recent_matches: ['loss'] },
      { puuid: '3', recent_win_rate: 79, recent_games_count: 20, recent_matches: ['loss'] },
    ];
    expect(detectTeamBadgesCurrentImplementation(squad79)).toBeUndefined();

    // 80% win rate squad
    const squad80: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 80, recent_games_count: 20, recent_matches: ['loss'] },
      { puuid: '2', recent_win_rate: 80, recent_games_count: 20, recent_matches: ['loss'] },
      { puuid: '3', recent_win_rate: 80, recent_games_count: 20, recent_matches: ['loss'] },
    ];
    expect(detectTeamBadgesCurrentImplementation(squad80)?.type).toBe('win-rate-team');
  });

  it('tests win rate team boundary: win streak 3 vs 4', () => {
    // Member 1 is anchor (85% WR, 15 games). Other members have avg streak = 3
    const squadStreak3: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 85, recent_games_count: 15, recent_matches: ['win', 'win'] },
      { puuid: '2', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'loss'] }, // streak 3
      { puuid: '3', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'loss'] }, // streak 3
    ];
    // squadAvgWr = (85+60+60)/3 = 68.33% (< 80)
    // otherMembers avgStreak = 3 (< 4)
    expect(detectTeamBadgesCurrentImplementation(squadStreak3)).toBeUndefined();

    // Other members have avg streak = 4
    const squadStreak4: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 85, recent_games_count: 15, recent_matches: ['win', 'win'] },
      { puuid: '2', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] }, // streak 4
      { puuid: '3', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] }, // streak 4
    ];
    expect(detectTeamBadgesCurrentImplementation(squadStreak4)?.type).toBe('win-rate-team');
  });

  it('tests loss rate team boundary: win rate 25% vs 26%', () => {
    // 25% win rate squad
    const squad25: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 25, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 25, recent_games_count: 10 },
    ];
    expect(detectTeamBadgesCurrentImplementation(squad25)?.type).toBe('loss-rate-team');

    // 26% win rate squad
    const squad26: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 26, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 26, recent_games_count: 10 },
    ];
    expect(detectTeamBadgesCurrentImplementation(squad26)).toBeUndefined();
  });

  it('EXPOSES BUG: loss streak (>=5) is not handled when win rate > 25%', () => {
    // Players with 30% win rate but on a 5-loss streak
    const squadLossStreak5: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss'] },
      { puuid: '2', recent_win_rate: 30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss'] },
    ];
    // In current implementation:
    // wr = 30 > 25 -> allLossQualified = false
    // squadAvgWr = 30 > 25 -> false
    // Loss streak is NOT checked, so it returns undefined!
    const result = detectTeamBadgesCurrentImplementation(squadLossStreak5);
    // We document this as an empirical finding:
    expect(result).toBeUndefined(); // Should have been 'loss-rate-team' according to worker claims and DISPATCH!
  });

  it('EXPOSES BUG: 0.0~1.0 normalized win rate falsely triggers loss-rate-team', () => {
    // Suppose a data source provides recent_win_rate in [0.0, 1.0] scale, e.g. 0.60 (60% win rate)
    const squadNormalized60: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.60, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 0.60, recent_games_count: 10 },
    ];
    // In current implementation:
    // for m: wr = 0.60 > 0.25 -> allLossQualified = false
    // BUT: squadAvgWr = (0.60 + 0.60) / 2 = 0.60
    // squadAvgWr <= 25 -> 0.60 <= 25 is TRUE!
    // So a 60% win rate team is classified as 'loss-rate-team'!
    const result = detectTeamBadgesCurrentImplementation(squadNormalized60);
    expect(result?.type).toBe('loss-rate-team'); // BUG: 60% winrate team marked as loss team!
  });
});

describe('Adversarial M3 Challenger: Tactical Broadcast UI & LocalStorage Persistence', () => {
  const storeMap = new Map<string, string>();
  const mockLocalStorage = {
    getItem: (key: string) => storeMap.get(key) || null,
    setItem: (key: string, val: string) => storeMap.set(key, val),
    removeItem: (key: string) => storeMap.delete(key),
    clear: () => storeMap.clear()
  };

  beforeEach(() => {
    // @ts-ignore
    globalThis.localStorage = mockLocalStorage;
    storeMap.clear();
  });

  it('EXPOSES BUG: Settings.vue onMounted resets "jungle" and "premade" presets to "concise"', () => {
    // Simulate user selecting "jungle" and saving to localStorage
    const userPrefs = {
      defaultPageSize: 20,
      broadcastFormat: 'jungle' as BroadcastPreset
    };
    localStorage.setItem('hexbrain_preferences', JSON.stringify(userPrefs));

    // Verify localStorage has 'jungle'
    expect(JSON.parse(localStorage.getItem('hexbrain_preferences')!).broadcastFormat).toBe('jungle');

    // Simulate Settings.vue onMounted parsing (Settings.vue:348):
    const parsed = JSON.parse(localStorage.getItem('hexbrain_preferences')!);
    const loadedBroadcastFormat = parsed.broadcastFormat === 'detailed' ? 'detailed' : 'concise';

    // Because Settings.vue uses `parsed.broadcastFormat === 'detailed' ? 'detailed' : 'concise'`,
    // 'jungle' gets transformed to 'concise'!
    expect(loadedBroadcastFormat).toBe('concise'); // BUG: 'jungle' is lost!

    // Same for 'premade'
    const userPrefsPremade = {
      defaultPageSize: 20,
      broadcastFormat: 'premade' as BroadcastPreset
    };
    localStorage.setItem('hexbrain_preferences', JSON.stringify(userPrefsPremade));
    const parsedPremade = JSON.parse(localStorage.getItem('hexbrain_preferences')!);
    const loadedPremade = parsedPremade.broadcastFormat === 'detailed' ? 'detailed' : 'concise';
    expect(loadedPremade).toBe('concise'); // BUG: 'premade' is lost!
  });

  it('OngoingMatch.vue correctly parses all 4 presets from localStorage', () => {
    const validPresets: BroadcastPreset[] = ['concise', 'detailed', 'jungle', 'premade'];

    for (const preset of validPresets) {
      localStorage.setItem('hexbrain_preferences', JSON.stringify({ broadcastFormat: preset }));
      const raw = localStorage.getItem('hexbrain_preferences');
      const parsed = JSON.parse(raw!);
      let format = 'concise';
      if (['concise', 'detailed', 'jungle', 'premade'].includes(parsed?.broadcastFormat)) {
        format = parsed.broadcastFormat;
      }
      expect(format).toBe(preset);
    }
  });
});
