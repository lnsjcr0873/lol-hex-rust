use lol_hex_rust::engine::player_history::{ParsedMatchRecord, PlayerHistoryManager};
use serde_json::json;
use std::sync::atomic::Ordering;

#[path = "lcu_mock_harness/mod.rs"]
mod lcu_mock_harness;
use lcu_mock_harness::fixtures::canned_match_history;
use lcu_mock_harness::{MockLcuClient, MockLcuServer};

#[test]
fn test_sample_match_count_extended_whitelist() {
    let mgr = PlayerHistoryManager::new();

    // Verify all Akari-compatible page sizes are accepted
    for &size in &[10, 20, 30, 40, 50, 100, 200] {
        mgr.set_sample_match_count(size);
        assert_eq!(mgr.sample_count.load(Ordering::Relaxed), size, "Expected size {} to be valid", size);
    }

    // Verify invalid sizes fallback to 20
    for &invalid in &[-1, 0, 5, 15, 25, 75, 999] {
        mgr.set_sample_match_count(invalid);
        assert_eq!(mgr.sample_count.load(Ordering::Relaxed), 20, "Expected invalid {} to fallback to 20", invalid);
    }
}

#[tokio::test]
async fn test_mock_lcu_pagination_slicing() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    let puuid = "puuid-pagination-test-01";
    // Setup a 50-game history
    let history = canned_match_history(puuid, 50, 0.6, 157, &[]);
    server.state.set_match_history(puuid, history).await;

    // Page 1: 0..19 (20 games)
    let p1 = client.get_player_match_history(puuid, 0, 19).await.unwrap();
    let games_p1 = p1["games"]["games"].as_array().unwrap();
    assert_eq!(games_p1.len(), 20);
    assert_eq!(games_p1[0]["gameId"].as_i64().unwrap(), 1_000_000_000);
    assert_eq!(games_p1[19]["gameId"].as_i64().unwrap(), 1_000_000_019);

    // Page 2: 20..39 (20 games)
    let p2 = client.get_player_match_history(puuid, 20, 39).await.unwrap();
    let games_p2 = p2["games"]["games"].as_array().unwrap();
    assert_eq!(games_p2.len(), 20);
    assert_eq!(games_p2[0]["gameId"].as_i64().unwrap(), 1_000_000_020);
    assert_eq!(games_p2[19]["gameId"].as_i64().unwrap(), 1_000_000_039);

    // Page 3 partial: 40..49 (10 games)
    let p3 = client.get_player_match_history(puuid, 40, 59).await.unwrap();
    let games_p3 = p3["games"]["games"].as_array().unwrap();
    assert_eq!(games_p3.len(), 10);
    assert_eq!(games_p3[0]["gameId"].as_i64().unwrap(), 1_000_000_040);
    assert_eq!(games_p3[9]["gameId"].as_i64().unwrap(), 1_000_000_049);

    // Out of range: 50..69 (0 games)
    let p4 = client.get_player_match_history(puuid, 50, 69).await.unwrap();
    let games_p4 = p4["games"]["games"].as_array().unwrap();
    assert_eq!(games_p4.len(), 0);

    server.stop();
}

#[tokio::test]
async fn test_raw_cache_match_merging_across_pages() {
    let mgr = PlayerHistoryManager::new();
    let puuid = "puuid-merge-test";

    // Simulate first fetch: Page 1 with game IDs 1..=20
    let now_ms = 1_700_000_000_000i64;
    let page1_matches: Vec<ParsedMatchRecord> = (1..=20)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: now_ms + i * 1000,
            win: i % 2 == 0,
            ..Default::default()
        })
        .collect();

    // Insert page 1 into cache manually as if fetched
    {
        let mut guard = mgr.raw_cache.write().await;
        guard.insert(
            puuid.to_string(),
            lol_hex_rust::engine::player_history::CachedPlayerRecord {
                puuid: puuid.to_string(),
                summoner_name: "MergeTester".to_string(),
                tier_str: "王者".to_string(),
                tier_color: "#f59e0b".to_string(),
                tier: "CHALLENGER".to_string(),
            division: "".to_string(),
                lp: 888,
                ranked_wins: 50,
                ranked_losses: 20,
                matches: page1_matches.clone(),
                fetched_at: std::time::Instant::now(),
                raw_summoner: json!({"puuid": puuid, "name": "MergeTester"}),
                raw_ranked: json!({"tier": "CHALLENGER"}),
            },
        );
    }

    // Verify cache has 20 matches
    let cached1 = mgr.get_player_data(puuid).await.expect("cached1");
    assert_eq!(cached1.matches.len(), 20);

    // Simulate second fetch: Page 2 with game IDs 21..=40 and 5 overlapping game IDs (16..=20)
    let page2_matches: Vec<ParsedMatchRecord> = (16..=40)
        .map(|i| ParsedMatchRecord {
            game_id: i,
            game_creation: now_ms + i * 1000,
            win: i % 2 == 0,
            ..Default::default()
        })
        .collect();

    // Perform merge exactly as fetch_and_cache_player does
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

    // Verify merged cache: 20 + 20 (deduplicated 16..20) = 40 matches total!
    let cached2 = mgr.get_player_data(puuid).await.expect("cached2");
    assert_eq!(cached2.matches.len(), 40, "Overlapping games should be deduplicated, total should be 40");

    // Verify sorted descending by game_creation
    for i in 0..cached2.matches.len() - 1 {
        assert!(
            cached2.matches[i].game_creation >= cached2.matches[i + 1].game_creation,
            "Matches must remain sorted descending by game_creation"
        );
    }
}

#[tokio::test]
async fn test_mock_player_profile_generates_200_matches() {
    let mgr = PlayerHistoryManager::new();
    let mock = mgr.build_mock_player_profile(0, 266);
    assert_eq!(mock.matches.len(), 200, "build_mock_player_profile must generate 200 matches for dynamic pagination");

    // Verify game creation descending order
    for i in 0..mock.matches.len() - 1 {
        assert!(
            mock.matches[i].game_creation >= mock.matches[i + 1].game_creation,
            "Mock matches must be ordered newest to oldest"
        );
    }

    // Verify unique game IDs
    let mut gids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for m in &mock.matches {
        assert!(gids.insert(m.game_id), "All 200 mock matches must have unique game_id: {}", m.game_id);
    }
}
