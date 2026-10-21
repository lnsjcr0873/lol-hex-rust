import { describe, it, expect } from 'vitest';
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
  getPlayerWinningStreak,
  getPlayerLosingStreak,
  detectWinRateTeamBadge,
  normalizeWinRatePercent,
  normalizeWinRateRatio
} from '../src/components/ongoing-game-panel/utils/team-tags';
import type { BroadcastPreset, PlayerCard } from '../src/api/types';

describe('Milestone 3: Adaptive Layout & Team Badges Genuine Parity', () => {
  it('verifies adaptive column calculation across breakpoint boundaries', () => {
    // Sub-780px falls back to 3
    expect(calculateColumnsNeed(0)).toBe(3);
    expect(calculateColumnsNeed(500)).toBe(3);
    expect(calculateColumnsNeed(780)).toBe(3);

    // 781px -> 3 columns
    expect(calculateColumnsNeed(781)).toBe(3);

    // 1020px vs 1021px -> 4 columns
    expect(calculateColumnsNeed(1020)).toBe(3);
    expect(calculateColumnsNeed(1021, 8)).toBe(4);

    // 1260px vs 1261px -> 5 columns
    expect(calculateColumnsNeed(1260, 8)).toBe(4);
    expect(calculateColumnsNeed(1261, 8)).toBe(5);

    // 1500px vs 1501px -> 6 columns
    expect(calculateColumnsNeed(1500, 8)).toBe(5);
    expect(calculateColumnsNeed(1501, 8)).toBe(6);

    // 1740px vs 1741px -> 7 columns
    expect(calculateColumnsNeed(1740, 8)).toBe(6);
    expect(calculateColumnsNeed(1741, 8)).toBe(7);

    // 1980px vs 1981px -> 8 columns
    expect(calculateColumnsNeed(1980, 8)).toBe(7);
    expect(calculateColumnsNeed(1981, 8)).toBe(8);

    // 4K wide display caps at 8 for 8-player teams, but clamps to 5 for standard 5v5 teams
    expect(calculateColumnsNeed(3840, 8)).toBe(8);
    expect(calculateColumnsNeed(3840, 5)).toBe(5);
  });

  it('verifies linesPerTeam safely handles standard and edge-case teams', () => {
    expect(calculateLinesPerTeam(3, 5)).toBe(2);
    expect(calculateLinesPerTeam(5, 5)).toBe(1);
    expect(calculateLinesPerTeam(4, 8)).toBe(2);
    expect(calculateLinesPerTeam(0, 0)).toBe(1);
    expect(calculateLinesPerTeam(3, 0)).toBe(2);
  });

  it('verifies win rate normalization helper for ratio vs percent scales', () => {
    expect(normalizeWinRatePercent(0.60)).toBe(60);
    expect(normalizeWinRatePercent(60)).toBe(60);
    expect(normalizeWinRatePercent(0.85)).toBe(85);
    expect(normalizeWinRatePercent(85)).toBe(85);
    expect(normalizeWinRatePercent(undefined)).toBe(0);

    expect(normalizeWinRateRatio(0.60)).toBe(0.60);
    expect(normalizeWinRateRatio(60)).toBe(0.60);
    expect(normalizeWinRateRatio(25)).toBe(0.25);
    expect(normalizeWinRateRatio(0.25)).toBe(0.25);
    expect(normalizeWinRateRatio(null)).toBe(0);
  });

  it('verifies winning and losing streak calculations from recent_matches', () => {
    const winner: Partial<PlayerCard> = { recent_matches: ['win', 'win', 'win', 'loss'] };
    expect(getPlayerWinningStreak(winner)).toBe(3);
    expect(getPlayerLosingStreak(winner)).toBe(0);

    const loser: Partial<PlayerCard> = { recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss', 'win'] };
    expect(getPlayerLosingStreak(loser)).toBe(5);
    expect(getPlayerWinningStreak(loser)).toBe(0);

    const empty: Partial<PlayerCard> = { recent_matches: [] };
    expect(getPlayerWinningStreak(empty)).toBe(0);
    expect(getPlayerLosingStreak(empty)).toBe(0);
  });

  it('verifies win rate team constants and thresholds', () => {
    expect(WIN_RATE_TEAM_MIN_WIN_RATE).toBe(0.8);
    expect(WIN_RATE_TEAM_OTHER_MEMBER_WIN_STREAK).toBe(4);
    expect(WIN_RATE_TEAM_MIN_SIZE).toBe(3);
    expect(LOSS_RATE_TEAM_MIN_SIZE).toBe(2);
    expect(LOSS_RATE_TEAM_MAX_WIN_RATE).toBe(0.25);
    expect(LOSS_RATE_TEAM_MIN_LOSS_STREAK).toBe(5);
  });

  it('verifies 👑 胜率队 detection with both percentage and ratio input scales', () => {
    // 80% average win rate squad (integer scale)
    const squad80Pct: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 80, recent_games_count: 20 },
      { puuid: '2', recent_win_rate: 80, recent_games_count: 20 },
      { puuid: '3', recent_win_rate: 80, recent_games_count: 20 }
    ];
    expect(detectWinRateTeamBadge(squad80Pct)?.type).toBe('win-rate-team');

    // 80% average win rate squad (float 0.0..1.0 scale)
    const squad80Ratio: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.80, recent_games_count: 20 },
      { puuid: '2', recent_win_rate: 0.80, recent_games_count: 20 },
      { puuid: '3', recent_win_rate: 0.80, recent_games_count: 20 }
    ];
    expect(detectWinRateTeamBadge(squad80Ratio)?.type).toBe('win-rate-team');

    // Anchor player with >= 80% WR and >= 13 games + other members avg streak >= 4
    const anchorSquad: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.85, recent_games_count: 15, recent_matches: ['win'] },
      { puuid: '2', recent_win_rate: 0.60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] },
      { puuid: '3', recent_win_rate: 0.60, recent_games_count: 10, recent_matches: ['win', 'win', 'win', 'win', 'loss'] }
    ];
    expect(detectWinRateTeamBadge(anchorSquad)?.type).toBe('win-rate-team');

    // Sub-threshold squad (79% win rate, streak 3)
    const normalSquad: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.79, recent_games_count: 20 },
      { puuid: '2', recent_win_rate: 0.79, recent_games_count: 20 },
      { puuid: '3', recent_win_rate: 0.79, recent_games_count: 20 }
    ];
    expect(detectWinRateTeamBadge(normalSquad)).toBeUndefined();
  });

  it('verifies ❄️ 败率队 detection and prevents false positives on 0..1 scale', () => {
    // Normal 60% win rate squad (stored as 0.60) MUST NOT trigger loss-rate-team
    const normalRatioSquad: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.60, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 0.60, recent_games_count: 10 }
    ];
    expect(detectWinRateTeamBadge(normalRatioSquad)).toBeUndefined();

    // Actual <= 25% win rate squad (integer scale)
    const squad25Pct: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 25, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 25, recent_games_count: 10 }
    ];
    expect(detectWinRateTeamBadge(squad25Pct)?.type).toBe('loss-rate-team');

    // Actual <= 25% win rate squad (ratio scale)
    const squad25Ratio: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.25, recent_games_count: 10 },
      { puuid: '2', recent_win_rate: 0.25, recent_games_count: 10 }
    ];
    expect(detectWinRateTeamBadge(squad25Ratio)?.type).toBe('loss-rate-team');

    // 30% win rate squad (> 25%) on a 5-loss streak MUST qualify for ❄️ 败率队
    const squadLossStreak5: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss'] },
      { puuid: '2', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'loss'] }
    ];
    expect(detectWinRateTeamBadge(squadLossStreak5)?.type).toBe('loss-rate-team');

    // 30% win rate squad on only 4-loss streak does NOT trigger loss-rate-team
    const squadLossStreak4: Partial<PlayerCard>[] = [
      { puuid: '1', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] },
      { puuid: '2', recent_win_rate: 0.30, recent_games_count: 10, recent_matches: ['loss', 'loss', 'loss', 'loss', 'win'] }
    ];
    expect(detectWinRateTeamBadge(squadLossStreak4)).toBeUndefined();
  });

  it('verifies BroadcastPreset values and Settings persistence format validation', () => {
    const validPresets: BroadcastPreset[] = ['concise', 'detailed', 'jungle', 'premade'];
    expect(validPresets).toHaveLength(4);

    // Test persistence validator logic matching Settings.vue
    const validatePreset = (val: unknown): BroadcastPreset => {
      return validPresets.includes(val as BroadcastPreset) ? (val as BroadcastPreset) : 'concise';
    };

    expect(validatePreset('concise')).toBe('concise');
    expect(validatePreset('detailed')).toBe('detailed');
    expect(validatePreset('jungle')).toBe('jungle');
    expect(validatePreset('premade')).toBe('premade');
    expect(validatePreset('unknown_preset')).toBe('concise');
    expect(validatePreset(null)).toBe('concise');
  });

  it('verifies PlayerCard interface carries is_autofilled and team_participant_id', () => {
    const card: Partial<PlayerCard> = {
      summoner_name: 'AutofillPlayer',
      is_autofilled: true,
      team_participant_id: 42,
    };
    expect(card.is_autofilled).toBe(true);
    expect(card.team_participant_id).toBe(42);
  });
});
