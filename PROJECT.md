# Project: HexBrain

## Architecture
HexBrain is a high-performance desktop application for League of Legends players, built on Tauri 2 + Vue 3 + Rust. It integrates directly with the League Client Update (LCU) local REST and WebSocket interfaces to provide real-time match analysis, summoner career profiling, post-game scoreboard reviews, and automated gameplay utilities.

```
                      ┌───────────────────────────────────────────────┐
                      │              League Client (LCU)              │
                      │   - REST API (127.0.0.1:port / wamp ws)      │
                      │   - Champ Select, Chat Rooms, Match History   │
                      └──────────────────────▲────────────────────────┘
                                             │ HTTPS (skip SSL) / WSS
                      ┌──────────────────────▼────────────────────────┐
                      │        HexBrain Rust Backend (Core Engine)    │
                      │                                               │
                      │  - lcu/client.rs: Resilient LCU Connector     │
                      │  - engine/player_history.rs: Stats & Badges   │
                      │  - web/routes.rs & src-tauri/src/cmd.rs: IPC  │
                      │  - config.rs: Atomic Persistence              │
                      └──────────────────────▲────────────────────────┘
                                             │ Tauri IPC / HTTP REST
                      ┌──────────────────────▼────────────────────────┐
                      │          HexBrain Frontend (Vue 3 + TS)       │
                      │                                               │
                      │  - OngoingMatch.vue: 5v5 Dual-Column View     │
                      │  - MatchHistory.vue: Profile, 10p Scoreboard  │
                      │  - Automation.vue & Settings.vue              │
                      └───────────────────────────────────────────────┘
```

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| F1 | LCU/SGP API & Data Contract Alignment | Align endpoints, request/response structures for gameflow, champ select, match history, and summoner profiles | M1 | ORIGINAL_REQUEST §R1 |
| F2 | Data Aggregation & Scoring Algorithms | Multi-dimensional scoring, MVP/SVP calculation, recent 20 match statistics, 6-axis radar normalization [25-99] | M1 | ORIGINAL_REQUEST §R1 |
| F3 | Premade Detection & Tactical Badges | Graph BFS connected component detection for 2-5 player parties (>=2 games), tactical badges (streaks, one-trick, smurf, testing) | M1 | ORIGINAL_REQUEST §R1 |
| F4 | 1-Click Chat Record Broadcast | Fast team history formatting and rate-limited posting to champ-select chat room | M2 | ORIGINAL_REQUEST §R2 |
| F5 | Ongoing Match 5v5 Dual-Column Panoramic View | 5v5 dual perspective with horizontal lane alignment (TOP vs TOP, JG vs JG), ranks, win rates, tactical badges | M2 | ORIGINAL_REQUEST §R2 |
| F6 | Offline & Simulation Mode Isolation | Smooth state machine handling for client not running, champ select, loading, in-game, and mock mode | M2 | ORIGINAL_REQUEST §R2 |
| F7 | Summoner Career Profile & Match History List | Dual ranked badges, recent 20 matches with lane & queue filters (Solo/Duo, Flex, ARAM), signature champions | M3 | ORIGINAL_REQUEST §R2 |
| F8 | 10-Player Detailed Scoreboard & Match Review | Fix P0 contract mismatch (flat participant structure), dual damage dealt/taken comparison bars, highest damage crowns, MVP/SVP | M3 | ORIGINAL_REQUEST §R2 |
| F9 | LCU Lifecycle Resilience & Credential Self-Healing | Global failure tracking across all endpoints, auto-reconnect within 3s, clear cached local PUUID on account switch/relog | M4 | ORIGINAL_REQUEST §R3 |
| F10 | CN Anti-Anonymity Robustness | Chat room participant polling with async retry and exponential backoff, skip local player PUUID, prevent ghost teammate duplicates | M4 | ORIGINAL_REQUEST §R3 |
| F11 | Special Game Modes Boundary Protection | ARAM bench reroll safety, Arena 2v2v2v2 multi-team isolation (prevent lumping into red team), TFT graceful degradation | M4 | ORIGINAL_REQUEST §R3 |
| F12 | Concurrency Rate Limiting & Atomic Config Persistence | Semaphore-based concurrency limiter on batch history, in-memory hot cache, atomic config file write via tempfile + rename | M4 | ORIGINAL_REQUEST §R3 |
| F13 | E2E Test Suite & Mock LCU Infrastructure | In-process Mock LCU HTTP harness simulating gameflow, champ-select, anti-anonymity, match history, scoreboard, special modes | E2E Track | ORIGINAL_REQUEST §Acceptance |
| F14 | Final Milestone Verification & Adversarial Hardening | Pass 100% E2E test suite (Tiers 1-4) and execute Tier 5 adversarial coverage hardening | M5 | ORIGINAL_REQUEST §Acceptance |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Core Engine & Data Contract Alignment | F1, F2, F3: Normalize backend routes/IPC schemas, MVP/SVP calculation, graph premade clustering, badge generation | none | DONE |
| M2 | Ongoing Match High-Fidelity Experience | F4, F5, F6: 5v5 dual-column layout with horizontal lane alignment, badge rendering, chat broadcast, offline/mock isolation | M1 | IN_PROGRESS |
| M3 | Match History & Scoreboard Full Restoration | F7, F8: Fix P0 contract mismatch, flat MatchDetailParticipant structure, dual damage/damage-taken bars, crowns, items, radar chart | M1 | IN_PROGRESS |
| M4 | LCU Robustness, Lifecycle Resilience & Special Modes | F9, F10, F11, F12: Self-healing LCU client, account switch relog, chat retry backoff, Arena/ARAM/TFT protection, atomic config persistence, semaphore throttling | M1 | IN_PROGRESS |
| M5 | Final E2E Integration & Adversarial Hardening | F14: Pass 100% E2E test suite (Tiers 1-4) and execute Tier 5 adversarial coverage hardening | M2, M3, M4, TEST_READY.md | PLANNED |

## E2E Testing Track
- Sub-Orchestrator: E2E Testing Orchestrator
- Scope: F13 (In-process Mock LCU HTTP test harness, Tier 1-4 requirement-driven test cases, publishes `TEST_READY.md`)
- Status: PLANNED

## Interface Contracts

### 1. `get_match_detail` Backend ↔ Frontend Contract
Frontend expects `MatchDetailFull` in `frontend/src/api/types.ts`:
```json
{
  "game_id": 1234567890,
  "game_creation": 1712000000000,
  "game_duration": 1800,
  "game_mode": "CLASSIC",
  "game_mode_cn": "召唤师峡谷",
  "queue_id": 420,
  "blue_team_kills": 25,
  "blue_team_turrets": 8,
  "blue_team_dragons": 3,
  "blue_team_barons": 1,
  "blue_team_win": true,
  "red_team_kills": 18,
  "red_team_turrets": 2,
  "red_team_dragons": 1,
  "red_team_barons": 0,
  "red_team_win": false,
  "participants": [
    {
      "puuid": "...",
      "summoner_name": "...",
      "team": "blue",
      "champion_id": 157,
      "champion_name": "亚索",
      "champion_level": 16,
      "spell1_id": 4,
      "spell2_id": 14,
      "spell1_icon": "...",
      "spell2_icon": "...",
      "kills": 10,
      "deaths": 3,
      "assists": 8,
      "kda_str": "6.0",
      "total_damage": 28500,
      "total_damage_taken": 19200,
      "damage_share_pct": 28.5,
      "damage_taken_share_pct": 24.2,
      "is_highest_damage": true,
      "is_mvp": true,
      "is_svp": false,
      "gold_earned": 14500,
      "minions_killed": 210,
      "items": [3031, 3046, 3006, 0, 0, 0, 3340],
      "role": "MIDDLE",
      "tier": "CHALLENGER"
    }
  ]
}
```

### 2. `get_ongoing_match` Backend ↔ Frontend Contract
Both teams (`blue_team` and `red_team`) provide `PlayerCard[]`. In 5v5 standard games, players are aligned or sortable by lane role (`TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY`).

### 3. Config Atomic Persistence
Writes to configuration must write to a temporary file in the same directory (`config.json.tmp`) and then atomically rename to `config.json`.

## Code Layout
- `src/`: Root Rust crate (`lol-hex-rust`)
  - `src/lcu/`: LCU client, connection sniffing, credentials, WebSocket/REST
  - `src/engine/`: Data calculation (player history, radar, badges, premade detection, champ-select anti-anonymity)
  - `src/web/`: Embedded web server and REST route handlers
  - `src/config.rs`: Configuration management and atomic file storage
- `src-tauri/`: Tauri 2 desktop application wrapper
  - `src-tauri/src/cmd.rs`: Tauri IPC commands exposed to frontend
  - `src-tauri/src/lib.rs`: Tauri builder, plugin registration, and command bindings
- `frontend/`: Vue 3 + TypeScript + Tailwind frontend
  - `frontend/src/views/`: Primary views (`OngoingMatch.vue`, `MatchHistory.vue`, `Automation.vue`, `Settings.vue`)
  - `frontend/src/components/`: Reusable UI elements (radar chart, badges, player cards, scoreboard rows)
  - `frontend/src/api/`: IPC wrappers (`tauri.ts`) and TypeScript types (`types.ts`)
- `tests/`: Integration & E2E Testing
  - `tests/lcu_mock_harness/`: Mock LCU HTTP test server
  - `tests/e2e_tier1_features/`: Feature coverage test suites
  - `tests/e2e_tier2_boundaries/`: Edge case and boundary test suites
  - `tests/e2e_tier3_combinations/`: Cross-feature combinatorial test suites
  - `tests/e2e_tier4_scenarios/`: Real-world scenario end-to-end test suites
