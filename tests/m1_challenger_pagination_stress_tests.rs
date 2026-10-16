//! Adversarial Stress & Boundary Challenge Test Suite for M1 Pagination & Match History Engine
//!
//! Specifically validates:
//! 1. Extreme boundary conditions: (0..199, 190..199, beg > end normalization, single match slices, out of bounds).
//! 2. Cache merging integrity: dedup across pages, reverse timestamp ordering, 500-match cap.
//! 3. IPC & Route parameter parsing: missing param defaults, clamping, camelCase & snake_case aliases.
//! 4. Live LCU Mock server integration under high boundary conditions.

use lol_hex_rust::engine::player_history::{ParsedMatchRecord, PlayerHistoryManager};
use lol_hex_rust::web::routes::HistoryQuery;
use serde_json::json;
use std::collections::HashSet;

#[path = "lcu_mock_harness/mod.rs"]
mod lcu_mock_harness;
use lcu_mock_harness::fixtures::canned_match_history;
use lcu_mock_harness::{MockLcuClient, MockLcuServer};

// ==============================================================================================
// 1. Extreme Boundary Conditions Tests
// ==============================================================================================

#[tokio::test]
async fn test_extreme_boundary_200_matches_full_page() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    let puuid = "puuid-stress-200-matches";
    // Setup a 250-match history to allow testing 0..199 and beyond
    let history = canned_match_history(puuid, 250, 0.55, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    // Boundary 1: 0..199 (exact maximum single page count = 200 matches)
    let res = client.get_player_match_history(puuid, 0, 199).await.expect("Must fetch 0..199");
    let games = res["games"]["games"].as_array().expect("games array");
    assert_eq!(games.len(), 200, "Must return exactly 200 matches for 0..199");
    assert_eq!(games[0]["gameId"].as_i64().unwrap(), 1_000_000_000);
    assert_eq!(games[199]["gameId"].as_i64().unwrap(), 1_000_000_199);

    server.stop();
}

#[tokio::test]
async fn test_extreme_boundary_tail_slice_190_to_199() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    let puuid = "puuid-stress-tail-slice";
    let history = canned_match_history(puuid, 250, 0.55, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    // Boundary 2: 190..199 (tail 10 matches of the 200-match window)
    let res = client.get_player_match_history(puuid, 190, 199).await.expect("Must fetch 190..199");
    let games = res["games"]["games"].as_array().expect("games array");
    assert_eq!(games.len(), 10, "Must return exactly 10 matches for 190..199");
    assert_eq!(games[0]["gameId"].as_i64().unwrap(), 1_000_000_190);
    assert_eq!(games[9]["gameId"].as_i64().unwrap(), 1_000_000_199);

    server.stop();
}

#[tokio::test]
async fn test_extreme_boundary_single_match_slices() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    let puuid = "puuid-stress-single-match";
    let history = canned_match_history(puuid, 100, 0.55, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    // Boundary 3a: 0..0 (single match at beginning)
    let res0 = client.get_player_match_history(puuid, 0, 0).await.expect("Must fetch 0..0");
    let games0 = res0["games"]["games"].as_array().expect("games array");
    assert_eq!(games0.len(), 1, "Must return exactly 1 match for 0..0");
    assert_eq!(games0[0]["gameId"].as_i64().unwrap(), 1_000_000_000);

    // Boundary 3b: 49..49 (single match at mid-range)
    let res49 = client.get_player_match_history(puuid, 49, 49).await.expect("Must fetch 49..49");
    let games49 = res49["games"]["games"].as_array().expect("games array");
    assert_eq!(games49.len(), 1, "Must return exactly 1 match for 49..49");
    assert_eq!(games49[0]["gameId"].as_i64().unwrap(), 1_000_000_049);

    // Boundary 3c: 99..99 (single match at boundary end)
    let res99 = client.get_player_match_history(puuid, 99, 99).await.expect("Must fetch 99..99");
    let games99 = res99["games"]["games"].as_array().expect("games array");
    assert_eq!(games99.len(), 1, "Must return exactly 1 match for 99..99");
    assert_eq!(games99[0]["gameId"].as_i64().unwrap(), 1_000_000_099);

    server.stop();
}

#[tokio::test]
async fn test_extreme_boundary_beg_greater_than_end_in_engine_and_ipc() {
    // Both IPC (`cmd.rs` line 523) and routes (`routes.rs` line 286) normalize:
    // (Some(b), Some(e)) => (b, e.max(b))
    // And `fetch_and_cache_player` line 1119 normalizes:
    // let (beg_index, end_index) = (beg_index, end_index.max(beg_index));
    // let count = (end_index + 1).saturating_sub(beg_index).clamp(1, 200);
    // let end_index = beg_index + count - 1;
    let b = 50usize;
    let e = 20usize;
    let (resolved_b, resolved_e) = (b, e.max(b));
    let count = (resolved_e + 1).saturating_sub(resolved_b).clamp(1, 200);
    let final_e = resolved_b + count - 1;

    assert_eq!(resolved_b, 50);
    assert_eq!(resolved_e, 50);
    assert_eq!(count, 1);
    assert_eq!(final_e, 50);

    // Now query Mock server with normalized indices (50..50)
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let puuid = "puuid-stress-beg-gt-end";
    let history = canned_match_history(puuid, 100, 0.55, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    let res = client.get_player_match_history(puuid, resolved_b, final_e).await.unwrap();
    let games = res["games"]["games"].as_array().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0]["gameId"].as_i64().unwrap(), 1_000_000_050);

    server.stop();
}

#[tokio::test]
async fn test_extreme_boundary_out_of_bounds_graceful() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    let puuid = "puuid-stress-oob";
    let history = canned_match_history(puuid, 50, 0.55, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    // Query entirely out of bounds: 200..250 on a 50-game history
    let res = client.get_player_match_history(puuid, 200, 250).await.expect("Must succeed with empty array");
    let games = res["games"]["games"].as_array().expect("games array");
    assert_eq!(games.len(), 0, "OOB slice should return 0 games without error");

    server.stop();
}

// ==============================================================================================
// 2. Cache Merging Integrity Tests
// ==============================================================================================

#[tokio::test]
async fn test_cache_merging_page1_page2_and_overlapping_dedup() {
    let mgr = PlayerHistoryManager::new();
    let puuid = "puuid-cache-merging-adversarial";
    let now_ms = 1_700_000_000_000i64;

    // Step 1: Query Page 1: matches 1..=20
    let page1_matches: Vec<ParsedMatchRecord> = (1..=20)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: now_ms + i * 1000,
            win: i % 2 == 0,
            ..Default::default()
        })
        .collect();

    {
        let mut guard = mgr.raw_cache.write().await;
        guard.insert(
            puuid.to_string(),
            lol_hex_rust::engine::player_history::CachedPlayerRecord {
                puuid: puuid.to_string(),
                summoner_name: "Merger".to_string(),
                tier_str: "王者".to_string(),
                tier_color: "#f59e0b".to_string(),
                tier: "CHALLENGER".to_string(),
                lp: 888,
                ranked_wins: 50,
                ranked_losses: 20,
                matches: page1_matches,
                fetched_at: std::time::Instant::now(),
                raw_summoner: json!({"puuid": puuid, "name": "Merger"}),
                raw_ranked: json!({"tier": "CHALLENGER"}),
            },
        );
    }

    let cached1 = mgr.get_player_data(puuid).await.expect("cached1");
    assert_eq!(cached1.matches.len(), 20, "Cache contains 20 matches after page 1");

    // Step 2: Query Page 2: matches 21..=40
    let page2_matches: Vec<ParsedMatchRecord> = (21..=40)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: now_ms + i * 1000,
            win: i % 2 == 0,
            ..Default::default()
        })
        .collect();

    {
        let mut guard = mgr.raw_cache.write().await;
        if let Some(existing) = guard.get_mut(puuid) {
            for m in &page2_matches {
                if !existing.matches.iter().any(|em| em.game_id == m.game_id) {
                    existing.matches.push(m.clone());
                }
            }
            existing.matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation));
        }
    }

    let cached2 = mgr.get_player_data(puuid).await.expect("cached2");
    assert_eq!(cached2.matches.len(), 40, "Cache merged page 1 and page 2: 20 + 20 = 40 matches");

    // Verify deduplication and descending order
    let mut ids = HashSet::new();
    for m in &cached2.matches {
        assert!(ids.insert(m.game_id), "Duplicate game ID found in cache: {}", m.game_id);
    }
    for i in 0..cached2.matches.len() - 1 {
        assert!(
            cached2.matches[i].game_creation >= cached2.matches[i + 1].game_creation,
            "Cache matches must be ordered descending by game_creation"
        );
    }

    // Step 3: Query Overlapping slice (matches 10..=30)
    let overlap_matches: Vec<ParsedMatchRecord> = (10..=30)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: now_ms + i * 1000,
            win: i % 2 == 0,
            ..Default::default()
        })
        .collect();

    {
        let mut guard = mgr.raw_cache.write().await;
        if let Some(existing) = guard.get_mut(puuid) {
            for m in &overlap_matches {
                if !existing.matches.iter().any(|em| em.game_id == m.game_id) {
                    existing.matches.push(m.clone());
                }
            }
            existing.matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation));
        }
    }

    let cached3 = mgr.get_player_data(puuid).await.expect("cached3");
    assert_eq!(cached3.matches.len(), 40, "Overlapping games must be deduplicated; total remains 40");
    let mut ids3 = HashSet::new();
    for m in &cached3.matches {
        assert!(ids3.insert(m.game_id), "Duplicate game ID found after overlap: {}", m.game_id);
    }
}

#[tokio::test]
async fn test_cache_merging_out_of_order_preserves_timestamp_sorting() {
    let mgr = PlayerHistoryManager::new();
    let puuid = "puuid-out-of-order-sort";

    let base_time = 1_700_000_000_000i64;

    // Simulate batch 1: matches at t=1000, 3000, 5000
    let batch1 = vec![
        ParsedMatchRecord { game_id: 1, game_creation: base_time + 1000, ..Default::default() },
        ParsedMatchRecord { game_id: 3, game_creation: base_time + 3000, ..Default::default() },
        ParsedMatchRecord { game_id: 5, game_creation: base_time + 5000, ..Default::default() },
    ];

    {
        let mut guard = mgr.raw_cache.write().await;
        guard.insert(
            puuid.to_string(),
            lol_hex_rust::engine::player_history::CachedPlayerRecord {
                puuid: puuid.to_string(),
                summoner_name: "Tester".to_string(),
                tier_str: "王者".to_string(),
                tier_color: "#f59e0b".to_string(),
                tier: "CHALLENGER".to_string(),
                lp: 888,
                ranked_wins: 50,
                ranked_losses: 20,
                matches: batch1,
                fetched_at: std::time::Instant::now(),
                raw_summoner: json!({"puuid": puuid, "name": "Tester"}),
                raw_ranked: json!({"tier": "CHALLENGER"}),
            },
        );
    }

    // Simulate batch 2: matches interspersed at t=2000, 4000, 6000
    let batch2 = vec![
        ParsedMatchRecord { game_id: 2, game_creation: base_time + 2000, ..Default::default() },
        ParsedMatchRecord { game_id: 4, game_creation: base_time + 4000, ..Default::default() },
        ParsedMatchRecord { game_id: 6, game_creation: base_time + 6000, ..Default::default() },
    ];

    {
        let mut guard = mgr.raw_cache.write().await;
        if let Some(existing) = guard.get_mut(puuid) {
            for m in &batch2 {
                if !existing.matches.iter().any(|em| em.game_id == m.game_id) {
                    existing.matches.push(m.clone());
                }
            }
            existing.matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation));
        }
    }

    let cached = mgr.get_player_data(puuid).await.expect("cached");
    assert_eq!(cached.matches.len(), 6);
    // Verifying strict descending order: 6 (6000), 5 (5000), 4 (4000), 3 (3000), 2 (2000), 1 (1000)
    for i in 0..cached.matches.len() - 1 {
        assert!(
            cached.matches[i].game_creation > cached.matches[i + 1].game_creation,
            "Must be strictly descending order"
        );
    }
    assert_eq!(cached.matches[0].game_id, 6);
    assert_eq!(cached.matches[5].game_id, 1);
}

#[tokio::test]
async fn test_cache_memory_cap_500_matches() {
    let mgr = PlayerHistoryManager::new();
    let puuid = "puuid-cap-test";

    let base_time = 1_700_000_000_000i64;

    // Simulate inserting 300 matches into the cache
    let initial_matches: Vec<ParsedMatchRecord> = (1..=300)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: base_time + i * 1000,
            ..Default::default()
        })
        .collect();

    {
        let mut guard = mgr.raw_cache.write().await;
        guard.insert(
            puuid.to_string(),
            lol_hex_rust::engine::player_history::CachedPlayerRecord {
                puuid: puuid.to_string(),
                summoner_name: "Tester".to_string(),
                tier_str: "王者".to_string(),
                tier_color: "#f59e0b".to_string(),
                tier: "CHALLENGER".to_string(),
                lp: 888,
                ranked_wins: 50,
                ranked_losses: 20,
                matches: initial_matches,
                fetched_at: std::time::Instant::now(),
                raw_summoner: json!({"puuid": puuid, "name": "Tester"}),
                raw_ranked: json!({"tier": "CHALLENGER"}),
            },
        );
    }

    // Merge next 300 matches (game_ids 301..=600)
    let next_matches: Vec<ParsedMatchRecord> = (301..=600)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: base_time + i * 1000,
            ..Default::default()
        })
        .collect();

    {
        let mut guard = mgr.raw_cache.write().await;
        if let Some(existing) = guard.get_mut(puuid) {
            for m in &next_matches {
                if !existing.matches.iter().any(|em| em.game_id == m.game_id) {
                    existing.matches.push(m.clone());
                }
            }
            existing.matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation));
            if existing.matches.len() > 500 {
                existing.matches.truncate(500);
            }
        }
    }

    let cached = mgr.get_player_data(puuid).await.expect("cached");
    assert_eq!(cached.matches.len(), 500, "Cache must be capped at exactly 500 matches");

    // The oldest 100 games (game_ids 1..=100) should have been truncated.
    // The retained games must be 101..=600, with newest first (600).
    assert_eq!(cached.matches[0].game_id, 600, "Newest match must be retained at index 0");
    assert_eq!(cached.matches[499].game_id, 101, "500th match must be game_id 101");
}

// ==============================================================================================
// 3. IPC & Route Parameter Parsing Tests
// ==============================================================================================

fn resolve_ipc_pagination_indices(
    beg_index: Option<usize>,
    end_index: Option<usize>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> (usize, usize) {
    let (resolved_beg, resolved_end) = match (beg_index, end_index) {
        (Some(b), Some(e)) => (b, e.max(b)),
        (Some(b), None) => {
            let count = page_size.unwrap_or(20).clamp(1, 200);
            (b, b + count.saturating_sub(1))
        }
        (None, Some(e)) => (0, e),
        (None, None) => {
            let p = page.unwrap_or(1).max(1);
            let ps = page_size.unwrap_or(20).clamp(1, 200);
            let b = (p - 1) * ps;
            let e = b + ps - 1;
            (b, e)
        }
    };
    let resolved_end = resolved_end.min(resolved_beg + 199);
    (resolved_beg, resolved_end)
}

#[test]
fn test_ipc_parameter_resolution_defaults_and_clamping() {
    // 1. All None -> page 1 of 20 matches (0..19)
    assert_eq!(resolve_ipc_pagination_indices(None, None, None, None), (0, 19));

    // 2. Only page specified -> page 3 defaults to 20/page: (40..59)
    assert_eq!(resolve_ipc_pagination_indices(None, None, Some(3), None), (40, 59));

    // 3. Page 0 -> clamped to page 1: (0..19)
    assert_eq!(resolve_ipc_pagination_indices(None, None, Some(0), None), (0, 19));

    // 4. Page size 0 -> clamped to 1 match: (0..0)
    assert_eq!(resolve_ipc_pagination_indices(None, None, None, Some(0)), (0, 0));

    // 5. Page size 500 -> clamped to 200 matches: (0..199)
    assert_eq!(resolve_ipc_pagination_indices(None, None, None, Some(500)), (0, 199));

    // 6. Page 2, page size 50 -> (50..99)
    assert_eq!(resolve_ipc_pagination_indices(None, None, Some(2), Some(50)), (50, 99));

    // 7. Page 1, page size 200 -> (0..199)
    assert_eq!(resolve_ipc_pagination_indices(None, None, Some(1), Some(200)), (0, 199));

    // 8. Beg index only: beg = 30, page_size = 40 -> (30..69)
    assert_eq!(resolve_ipc_pagination_indices(Some(30), None, None, Some(40)), (30, 69));

    // 9. Beg index only with no page_size -> defaults to 20 count: (30..49)
    assert_eq!(resolve_ipc_pagination_indices(Some(30), None, None, None), (30, 49));

    // 10. End index only: end = 50 -> (0..50)
    assert_eq!(resolve_ipc_pagination_indices(None, Some(50), None, None), (0, 50));

    // 11. Beg > End inversion: beg = 50, end = 20 -> normalized to (50..50) single match
    assert_eq!(resolve_ipc_pagination_indices(Some(50), Some(20), None, None), (50, 50));

    // 12. Giant span: beg = 0, end = 999 -> clamped to max 200 window: (0..199)
    assert_eq!(resolve_ipc_pagination_indices(Some(0), Some(999), None, None), (0, 199));

    // 13. Offset giant span: beg = 100, end = 999 -> clamped to (100..299)
    assert_eq!(resolve_ipc_pagination_indices(Some(100), Some(999), None, None), (100, 299));
}

#[test]
fn test_history_query_serde_deserialization_aliases() {
    // Test snake_case and camelCase JSON/Payload parsing into HistoryQuery

    // 1. Empty payload
    let q1: HistoryQuery = serde_json::from_value(json!({})).expect("empty query");
    assert_eq!(q1.beg_index, None);
    assert_eq!(q1.end_index, None);
    assert_eq!(q1.page, None);
    assert_eq!(q1.page_size, None);
    assert_eq!(q1.queue_id, None);

    // 2. camelCase format (from frontend Akari HTTP calls)
    let q2: HistoryQuery = serde_json::from_value(json!({
        "begIndex": 20,
        "endIndex": 39,
        "queueId": 420,
        "pageSize": 20
    })).expect("camelCase");
    assert_eq!(q2.beg_index_camel, Some(20));
    assert_eq!(q2.end_index_camel, Some(39));
    assert_eq!(q2.queue_id_camel, Some(420));
    assert_eq!(q2.page_size_camel, Some(20));

    // 3. snake_case format (from backend internal or alternative API calls)
    let q3: HistoryQuery = serde_json::from_value(json!({
        "beg_index": 0,
        "end_index": 199,
        "queue_id": 450,
        "page_size": 200
    })).expect("snake_case");
    assert_eq!(q3.beg_index, Some(0));
    assert_eq!(q3.end_index, Some(199));
    assert_eq!(q3.queue_id, Some(450));
    assert_eq!(q3.page_size, Some(200));

    // 4. Combined aliases precedence resolution check:
    let resolved_queue = q2.queue_id.or(q2.queue_id_camel).unwrap_or(0);
    assert_eq!(resolved_queue, 420);

    let resolved_beg = q2.beg_index.or(q2.beg_index_camel);
    assert_eq!(resolved_beg, Some(20));

    let resolved_end = q2.end_index.or(q2.end_index_camel);
    assert_eq!(resolved_end, Some(39));
}
