# E2E Test Infra: HexBrain

## Test Philosophy
- Opaque-box, requirement-driven testing. No dependency on internal implementation details.
- Validates system behaviors from the perspective of an end user and LCU client integration.
- Methodology: Category-Partition + Boundary Value Analysis (BVA) + Pairwise Combinatorial Testing + Real-World Workload Testing.

## Feature Inventory
| # | Feature | Source (requirement) | Tier 1 | Tier 2 | Tier 3 |
|---|---------|----------------------|:------:|:------:|:------:|
| 1 | LCU API & Data Contract Alignment | ORIGINAL_REQUEST §R1 | 5 | 5 | ✓ |
| 2 | Data Aggregation & Scoring Algorithms (MVP/SVP, Radar) | ORIGINAL_REQUEST §R1 | 5 | 5 | ✓ |
| 3 | Premade Team Detection (Graph BFS) | ORIGINAL_REQUEST §R1 | 5 | 5 | ✓ |
| 4 | Tactical Badges Generation | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 5 | Chat Record Broadcast Formatting & Rate Limit | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 6 | Ongoing Match 5v5 Dual-Column Alignment | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 7 | Offline & Simulation Mode Transitions | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 8 | Summoner Profile & History Queue Filter | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 9 | 10-Player Scoreboard Contract & Dual Bars | ORIGINAL_REQUEST §R2 | 5 | 5 | ✓ |
| 10 | LCU Lifecycle Resilience & Credential Self-Healing | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |
| 11 | CN Anti-Anonymity & Account Switch (Relog) | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |
| 12 | Special Game Modes (ARAM, Arena 2v2v2v2, TFT) | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |
| 13 | Concurrency Throttling & Atomic Config Persistence | ORIGINAL_REQUEST §R3 | 5 | 5 | ✓ |

## Test Architecture
- **In-Process Mock LCU HTTP Server**: Simulates LCU REST API endpoints (`/lol-gameflow/v1/gameflow-phase`, `/lol-champ-select/v1/session`, `/lol-chat/v1/conversations`, `/lol-match-history/v1/products/lol/{puuid}/matches`, `/lol-match-history/v1/games/{id}`, `/lol-summoner/v1/current-summoner`).
- **Test Runner**: Standard Rust integration test harness invoked via `cargo test --test <name>`.
- **Pass/Fail Semantics**: All test assertions must pass with exit code 0. Zero panic tolerance.
- **Directory Layout**:
  - `tests/lcu_mock_harness/`: Reusable Mock LCU HTTP server fixture and mock data generators.
  - `tests/tier1_feature_tests.rs`: Tier 1 tests (>=65 feature coverage tests).
  - `tests/tier2_boundary_tests.rs`: Tier 2 tests (>=65 boundary & corner case tests).
  - `tests/tier3_combination_tests.rs`: Tier 3 tests (>=15 pairwise combinatorial interaction tests).
  - `tests/tier4_scenario_tests.rs`: Tier 4 tests (>=7 end-to-end realistic user workflows).

## Real-World Application Scenarios (Tier 4)
| # | Scenario | Features Exercised | Complexity |
|---|----------|--------------------|------------|
| 1 | Standard Ranked Solo/Duo Champ Select & Anti-Anonymity | F1, F5, F6, F10, F11 | High |
| 2 | Premade Trio Party in Flex Queue with Badges & Chat Broadcast | F2, F3, F4, F5, F6 | High |
| 3 | Full Match History Review & 10-Player Scoreboard Analysis | F1, F2, F7, F8 | Medium |
| 4 | Client Sudden Crash, Relog Different Account, and Reconnect | F9, F10, F11, F13 | High |
| 5 | High-Frequency Concurrent Requests with Semaphore Limiting | F1, F8, F13 | Medium |
| 6 | ARAM Match with Bench Rerolls and Non-Standard Team IDs | F5, F8, F12 | Medium |
| 7 | Arena 2v2v2v2 4-Team Match Scoreboard Isolation | F8, F12 | High |

## Coverage Thresholds
- Tier 1: >=5 test cases per feature (>=65 total)
- Tier 2: >=5 boundary/corner test cases per feature (>=65 total)
- Tier 3: Pairwise combinations across features (>=13 total)
- Tier 4: >=7 realistic end-to-end user application scenarios
- **Total Minimum Test Count: ~150 E2E tests**
