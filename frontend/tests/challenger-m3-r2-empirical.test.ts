import { describe, it, expect, beforeEach } from 'vitest';
import {
  FIXED_CARD_WIDTH_PX_NUMBER,
  WIN_RATE_TEAM_MIN_MATCHES,
  WIN_RATE_TEAM_MIN_SIZE,
  WIN_RATE_TEAM_MIN_WIN_RATE,
  WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK,
  LOSS_RATE_TEAM_MIN_SIZE,
  LOSS_RATE_TEAM_MAX_WIN_RATE,
  LOSS_RATE_TEAM_MIN_LOSS_STREAK
} from '../src/components/ongoing-game-panel/constants';
import {
  calculateColumnsNeed,
  calculateLinesPerTeam
} from '../src/components/ongoing-game-panel/utils/layout';
import {
  normalizeWinRatePercent,
  normalizeWinRateRatio,
  getPlayerWinningStreak,
  getPlayerLosingStreak,
  detectWinRateTeamBadge
} from '../src/components/ongoing-game-panel/utils/team-tags';
import type { BroadcastPreset, PlayerCard } from '../src/api/types';

describe('Empirical Challenger R2: Settings.vue Preset Persistence', () => {
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

  const ALL_PRESETS: BroadcastPreset[] = ['concise', 'detailed', 'jungle', 'premade'];

  // Mirror exact parsing logic of Settings.vue:348
  const parsePreferencesFromStorage = (rawJson: string | null) => {
    if (!rawJson) return { defaultPageSize: 20, broadcastFormat: 'concise' as BroadcastPreset };
    try {
      const parsed = JSON.parse(rawJson);
      return {
        defaultPageSize: [10, 20, 50, 100].includes(parsed?.defaultPageSize) ? parsed.defaultPageSize : 20,
        broadcastFormat: (ALL_PRESETS as BroadcastPreset[]).includes(parsed?.broadcastFormat)
          ? parsed.broadcastFormat
          : 'concise'
      };
    } catch {
      return { defaultPageSize: 20, broadcastFormat: 'concise' as BroadcastPreset };
    }
  };

  it('preserves all 4 presets across JSON save and reload cycle', () => {
    for (const preset of ALL_PRESETS) {
      const payload = {
        defaultPageSize: 50,
        broadcastFormat: preset
      };
      localStorage.setItem('hexbrain_preferences', JSON.stringify(payload));
      const loaded = parsePreferencesFromStorage(localStorage.getItem('hexbrain_preferences'));

      expect(loaded.broadcastFormat).toBe(preset);
      expect(loaded.defaultPageSize).toBe(50);
    }
  });

  it('gracefully falls back to "concise" on invalid or corrupted inputs', () => {
    const invalidInputs = [
      '',
      '{ invalid json }',
      JSON.stringify({ broadcastFormat: 'unknown_preset' }),
      JSON.stringify({ broadcastFormat: 'CONCISE' }), // case sensitive check
      JSON.stringify({ broadcastFormat: null }),
      JSON.stringify({ broadcastFormat: 123 }),
      JSON.stringify({ broadcastFormat: {} }),
      JSON.stringify({ broadcastFormat: [] }),
      JSON.stringify({})
    ];

    for (const input of invalidInputs) {
      const loaded = parsePreferencesFromStorage(input);
      expect(loaded.broadcastFormat).toBe('concise');
    }
  });
});

describe('Empirical Challenger R2: Win Rate Scale Invariance & Loss Streak', () => {
  describe('Normalization functions (normalizeWinRatePercent & normalizeWinRateRatio)', () => {
    it('normalizes ratio scale [0.0, 1.0] to percentage scale [0, 100]', () => {
      expect(normalizeWinRatePercent(0.0)).toBe(0);
      expect(normalizeWinRatePercent(0.15)).toBeCloseTo(15);
      expect(normalizeWinRatePercent(0.25)).toBeCloseTo(25);
      expect(normalizeWinRatePercent(0.60)).toBeCloseTo(60);
      expect(normalizeWinRatePercent(0.80)).toBeCloseTo(80);
      expect(normalizeWinRatePercent(0.85)).toBeCloseTo(85);
      expect(normalizeWinRatePercent(1.0)).toBe(100);
    });

    it('preserves integer/percent scale [0, 100] as percentages', () => {
      expect(normalizeWinRatePercent(0)).toBe(0);
      expect(normalizeWinRatePercent(15)).toBe(15);
      expect(normalizeWinRatePercent(25)).toBe(25);
      expect(normalizeWinRatePercent(60)).toBe(60);
      expect(normalizeWinRatePercent(80)).toBe(80);
      expect(normalizeWinRatePercent(85)).toBe(85);
      expect(normalizeWinRatePercent(100)).toBe(100);
    });

    it('safely handles non-finite and edge values', () => {
      expect(normalizeWinRatePercent(null)).toBe(0);
      expect(normalizeWinRatePercent(undefined)).toBe(0);
      expect(normalizeWinRatePercent(NaN)).toBe(0);
      expect(normalizeWinRatePercent(Infinity)).toBe(0);
      expect(normalizeWinRateRatio(null)).toBe(0);
      expect(normalizeWinRateRatio(undefined)).toBe(0);
      expect(normalizeWinRateRatio(NaN)).toBe(0);
    });

    it('normalizes percentage scale to ratio scale [0.0, 1.0]', () => {
      expect(normalizeWinRateRatio(60)).toBeCloseTo(0.60);
      expect(normalizeWinRateRatio(0.60)).toBeCloseTo(0.60);
      expect(normalizeWinRateRatio(85)).toBeCloseTo(0.85);
      expect(normalizeWinRateRatio(0.85)).toBeCloseTo(0.85);
      expect(normalizeWinRateRatio(15)).toBeCloseTo(0.15);
      expect(normalizeWinRateRatio(0.15)).toBeCloseTo(0.15);
    });
  });

  describe('Streak detection (winning & losing)', () => {
    it('correctly counts consecutive winning streak', () => {
      expect(getPlayerWinningStreak({ recent_matches: ['win', 'win', 'win', 'win', 'loss'] })).toBe(4);
      expect(getPlayerWinningStreak({ recent_matches: ['loss', 'win', 'win'] })).toBe(0);
      expect(getPlayerWinningStreak({ recent_matches: [] })).toBe(0);
      expect(getPlayerWinningStreak({ recent_matches: undefined })).toBe(0);
    });

    it('correctly counts consecutive losing streak', () => {
      expect(getPlayerLosingStreak({ recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss', 'win'] })).toBe(5);
      expect(getPlayerLosingStreak({ recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] })).toBe(4);
      expect(getPlayerLosingStreak({ recent_matches: ['win', 'loss', 'loss'] })).toBe(0);
      expect(getPlayerLosingStreak({ recent_matches: [] })).toBe(0);
      expect(getPlayerLosingStreak({ recent_matches: undefined })).toBe(0);
    });
  });

  describe('detectWinRateTeamBadge scale invariance and detection accuracy', () => {
    it('R2-TEST-1: 60% win rate squad (0.60 vs 60) evaluates to squadAvgWr=60 and is NOT flagged as ❄️ 败率队', () => {
      // Float ratio scale (0.60)
      const squadRatio60: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.60, recent_games_count: 20 },
        { puuid: 'p2', recent_win_rate: 0.60, recent_games_count: 20 },
        { puuid: 'p3', recent_win_rate: 0.60, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(squadRatio60)).toBeUndefined();

      // Integer percent scale (60)
      const squadInt60: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 60, recent_games_count: 20 },
        { puuid: 'p2', recent_win_rate: 60, recent_games_count: 20 },
        { puuid: 'p3', recent_win_rate: 60, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(squadInt60)).toBeUndefined();

      // Mixed scale (0.60 and 60)
      const squadMixed60: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.60, recent_games_count: 20 },
        { puuid: 'p2', recent_win_rate: 60, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(squadMixed60)).toBeUndefined();
    });

    it('R2-TEST-2: 85% win rate squad (0.85 vs 85) with streak >= 4 flags as 👑 胜率队', () => {
      // 85% anchor (0.85 ratio) with 15 matches + other members with streak 4
      const anchorSquadRatio: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.85, recent_games_count: 15, recent_matches: ['win'] },
        { puuid: 'p2', recent_win_rate: 0.60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] },
        { puuid: 'p3', recent_win_rate: 0.60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] }
      ];
      expect(detectWinRateTeamBadge(anchorSquadRatio)?.type).toBe('win-rate-team');

      // 85% anchor (85 percent) with 15 matches + other members with streak 4
      const anchorSquadInt: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 85, recent_games_count: 15, recent_matches: ['win'] },
        { puuid: 'p2', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] },
        { puuid: 'p3', recent_win_rate: 60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] }
      ];
      expect(detectWinRateTeamBadge(anchorSquadInt)?.type).toBe('win-rate-team');

      // Pure 85% squad across the board (both scales)
      const pureSquad85: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.85, recent_games_count: 20 },
        { puuid: 'p2', recent_win_rate: 85, recent_games_count: 20 },
        { puuid: 'p3', recent_win_rate: 0.85, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(pureSquad85)?.type).toBe('win-rate-team');
    });

    it('R2-TEST-3: 15% win rate squad (0.15 vs 15) flags as ❄️ 败率队', () => {
      // Ratio scale (0.15)
      const squadRatio15: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.15, recent_games_count: 10 },
        { puuid: 'p2', recent_win_rate: 0.15, recent_games_count: 10 }
      ];
      expect(detectWinRateTeamBadge(squadRatio15)?.type).toBe('loss-rate-team');

      // Integer percent scale (15)
      const squadInt15: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 15, recent_games_count: 10 },
        { puuid: 'p2', recent_win_rate: 15, recent_games_count: 10 }
      ];
      expect(detectWinRateTeamBadge(squadInt15)?.type).toBe('loss-rate-team');

      // Mixed scale
      const squadMixed15: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.15, recent_games_count: 10 },
        { puuid: 'p2', recent_win_rate: 15, recent_games_count: 10 }
      ];
      expect(detectWinRateTeamBadge(squadMixed15)?.type).toBe('loss-rate-team');
    });

    it('R2-TEST-4: Loss streak >= 5 flags as ❄️ 败率队 even if win rate > 25%', () => {
      // 30% win rate (well above 25% threshold) with 5-loss streak on one member
      const squadLossStreak5: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss'] },
        { puuid: 'p2', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] }
      ];
      expect(detectWinRateTeamBadge(squadLossStreak5)?.type).toBe('loss-rate-team');

      // 4-loss streak with 30% win rate does NOT trigger loss-rate-team
      const squadLossStreak4: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] },
        { puuid: 'p2', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] }
      ];
      expect(detectWinRateTeamBadge(squadLossStreak4)).toBeUndefined();
    });

    it('respects squad size thresholds (win-rate min 3, loss-rate min 2)', () => {
      // 2 players with 90% win rate -> cannot be win-rate-team (min 3 required)
      const duo90: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 90, recent_games_count: 20 },
        { puuid: 'p2', recent_win_rate: 90, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(duo90)).toBeUndefined();

      // 1 player with 10% win rate -> cannot be loss-rate-team (min 2 required)
      const solo10: Partial<PlayerCard>[] = [
        { puuid: 'p1', recent_win_rate: 10, recent_games_count: 20 }
      ];
      expect(detectWinRateTeamBadge(solo10)).toBeUndefined();
    });
  });
});

describe('Empirical Challenger R2: Adaptive Layout Bounds (500px to 4K)', () => {
  it('tests full continuum from 500px to 4K for 8-player squads', () => {
    // Under 780px: always 3 columns
    expect(calculateColumnsNeed(500, 8)).toBe(3);
    expect(calculateColumnsNeed(600, 8)).toBe(3);
    expect(calculateColumnsNeed(780, 8)).toBe(3);

    // 780px -> 3 cols
    expect(calculateColumnsNeed(781, 8)).toBe(3);

    // Breakpoint: 1020px vs 1021px (col 4: 240 * 4.25 = 1020)
    expect(calculateColumnsNeed(1020, 8)).toBe(3);
    expect(calculateColumnsNeed(1021, 8)).toBe(4);

    // Breakpoint: 1260px vs 1261px (col 5: 240 * 5.25 = 1260)
    expect(calculateColumnsNeed(1260, 8)).toBe(4);
    expect(calculateColumnsNeed(1261, 8)).toBe(5);

    // Breakpoint: 1500px vs 1501px (col 6: 240 * 6.25 = 1500)
    expect(calculateColumnsNeed(1500, 8)).toBe(5);
    expect(calculateColumnsNeed(1501, 8)).toBe(6);

    // Breakpoint: 1740px vs 1741px (col 7: 240 * 7.25 = 1740)
    expect(calculateColumnsNeed(1740, 8)).toBe(6);
    expect(calculateColumnsNeed(1741, 8)).toBe(7);

    // Breakpoint: 1980px vs 1981px (col 8: 240 * 8.25 = 1980)
    expect(calculateColumnsNeed(1980, 8)).toBe(7);
    expect(calculateColumnsNeed(1981, 8)).toBe(8);

    // 2K (2560px) and 4K (3840px)
    expect(calculateColumnsNeed(2560, 8)).toBe(8);
    expect(calculateColumnsNeed(3840, 8)).toBe(8);
  });

  it('verifies standard 5v5 team layout clamps columns to team size (max 5)', () => {
    // For a standard 5v5 team (teamSize = 5), maxTeamMembers is 5.
    // Even on 4K display, it must not display 6, 7, 8 columns with empty slots.
    expect(calculateColumnsNeed(500, 5)).toBe(3);
    expect(calculateColumnsNeed(1021, 5)).toBe(4);
    expect(calculateColumnsNeed(1261, 5)).toBe(5);
    expect(calculateColumnsNeed(1501, 5)).toBe(5); // clamped to 5!
    expect(calculateColumnsNeed(1981, 5)).toBe(5); // clamped to 5!
    expect(calculateColumnsNeed(3840, 5)).toBe(5); // clamped to 5!
  });

  it('verifies calculateLinesPerTeam under normal and boundary conditions', () => {
    // 5 players on 3 columns -> 2 lines
    expect(calculateLinesPerTeam(3, 5)).toBe(2);
    // 5 players on 4 columns -> 2 lines
    expect(calculateLinesPerTeam(4, 5)).toBe(2);
    // 5 players on 5 columns -> 1 line
    expect(calculateLinesPerTeam(5, 5)).toBe(1);
    // 8 players on 4 columns -> 2 lines
    expect(calculateLinesPerTeam(4, 8)).toBe(2);
    // 8 players on 8 columns -> 1 line
    expect(calculateLinesPerTeam(8, 8)).toBe(1);

    // Safe zero/negative handling
    expect(calculateLinesPerTeam(0, 5)).toBe(1);
    expect(calculateLinesPerTeam(-2, 5)).toBe(1);
  });
});
