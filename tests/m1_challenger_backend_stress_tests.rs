//! Empirical Adversarial Stress Test Suite for HexBrain Phase 1 Backend Implementations
//!
//! Covers the 4 core focus areas specified in the dispatch:
//! 1. In-memory LRU cache: eviction order, capacity bounding, zero-capacity boundary, concurrent cache access, and hit consistency.
//! 2. Concurrency limiter: semaphore permit exhaustion, FIFO queuing behavior, concurrency race condition prevention, and permit leak immunity.
//! 3. Batch chunking: boundary ranges (0, 1, 19, 20, 21, 50, 100, 200), non-zero offsets, early termination on truncated history, and clamping.
//! 4. SGP URL resolution and tokens: unlisted shards, Tencent vs Global cluster handling, sub-ID placeholders, and token formatting.

#![recursion_limit = "512"]

#[path = "lcu_mock_harness/mod.rs"]
mod lcu_mock_harness;

use lcu_mock_harness::fixtures::canned_match_history;
use lcu_mock_harness::MockLcuServer;
use lol_hex_rust::lcu::client::{LcuClient, LruStore};
use lol_hex_rust::sgp::{
    get_sgp_server_id, resolve_server_config, resolve_sub_id, EntitlementsToken, SgpClient,
    SgpTokens,
};
use reqwest::header::HeaderValue;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

// ==============================================================================================
// 1. In-Memory LRU Cache Stress Tests
// ==============================================================================================

#[test]
fn test_lru_cache_strict_eviction_order() {
    let mut store = LruStore::<u64, String>::new(4);
    assert_eq!(store.capacity(), 4);
    assert_eq!(store.len(), 0);

    // Insert keys 1, 2, 3, 4
    store.insert(1, "A".to_string());
    store.insert(2, "B".to_string());
    store.insert(3, "C".to_string());
    store.insert(4, "D".to_string());
    assert_eq!(store.len(), 4);

    // Access key 1, then key 3 (order from oldest to newest: 2, 4, 1, 3)
    assert_eq!(store.get(&1), Some("A".to_string()));
    assert_eq!(store.get(&3), Some("C".to_string()));

    // Insert key 5: must evict the oldest key (2)
    store.insert(5, "E".to_string());
    assert_eq!(store.len(), 4);
    assert!(!store.contains_key(&2), "Key 2 must have been evicted first");
    assert!(store.contains_key(&1));
    assert!(store.contains_key(&3));
    assert!(store.contains_key(&4));
    assert!(store.contains_key(&5));

    // Insert key 6: must evict the next oldest key (4)
    store.insert(6, "F".to_string());
    assert_eq!(store.len(), 4);
    assert!(!store.contains_key(&4), "Key 4 must have been evicted");
    assert!(store.contains_key(&1));
    assert!(store.contains_key(&3));
    assert!(store.contains_key(&5));
    assert!(store.contains_key(&6));

    // Update existing key 1 with a new value: updates value and moves key 1 to MRU
    store.insert(1, "A_updated".to_string());
    assert_eq!(store.len(), 4);
    assert_eq!(store.get(&1), Some("A_updated".to_string()));

    // Insert key 7: must evict key 3 (since key 1 was updated to MRU, order was 3, 5, 6, 1)
    store.insert(7, "G".to_string());
    assert_eq!(store.len(), 4);
    assert!(!store.contains_key(&3), "Key 3 must have been evicted");
    assert_eq!(store.get(&1), Some("A_updated".to_string()));
    assert!(store.contains_key(&5));
    assert!(store.contains_key(&6));
    assert!(store.contains_key(&7));
}

#[test]
fn test_lru_cache_capacity_bounding_and_zero_boundary() {
    // Boundary: capacity = 0 must be safely clamped to at least 1
    let mut zero_store = LruStore::<u64, i32>::new(0);
    assert_eq!(zero_store.capacity(), 1, "Capacity 0 must clamp to 1");

    for i in 0..50 {
        zero_store.insert(i, (i * 10) as i32);
        assert_eq!(zero_store.len(), 1, "Zero-clamped store must strictly hold at most 1 item");
        assert_eq!(zero_store.get(&i), Some((i * 10) as i32));
        if i > 0 {
            assert!(!zero_store.contains_key(&(i - 1)), "Previous key must be immediately evicted");
        }
    }

    // Capacity 256 mass insertion stress test
    let mut store = LruStore::<u64, usize>::new(256);
    assert_eq!(store.capacity(), 256);

    for i in 1..=10_000u64 {
        store.insert(i, (i * 2) as usize);
        assert!(store.len() <= 256, "Store len must NEVER exceed capacity 256");
        if i >= 256 {
            assert_eq!(store.len(), 256, "Store must remain fully saturated at capacity 256");
        }
    }

    // Verify only the most recent 256 items exist (9745..=10000)
    for i in 1..=9744 {
        assert!(!store.contains_key(&i), "Key {} must have been evicted", i);
    }
    for i in 9745..=10_000 {
        assert!(store.contains_key(&i), "Recent key {} must exist", i);
    }

    // Verify clear resets everything cleanly
    store.clear();
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
    assert!(!store.contains_key(&10_000));
}

#[tokio::test]
async fn test_lru_cache_high_concurrency_race_free() {
    let cache = Arc::new(std::sync::Mutex::new(LruStore::<u64, String>::new(100)));
    let mut handles = Vec::new();

    // Spawn 50 concurrent tasks performing 10,000 mixed operations
    for task_id in 0..50 {
        let cache_clone = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            for i in 0..200 {
                let key = (task_id * 10 + (i % 30)) as u64;
                if i % 3 == 0 {
                    // Write operation
                    let val = format!("val_{}_{}", task_id, i);
                    let mut lock = cache_clone.lock().unwrap();
                    lock.insert(key, val);
                } else {
                    // Read operation
                    let mut lock = cache_clone.lock().unwrap();
                    let _ = lock.get(&key);
                }
            }
        }));
    }

    for h in handles {
        h.await.expect("Task must not panic or deadlock");
    }

    let final_lock = cache.lock().unwrap();
    assert!(final_lock.len() <= 100, "Cache capacity bound 100 must hold after concurrent storm");
}

#[tokio::test]
async fn test_lcu_client_cache_hit_consistency_and_zero_network() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_game_id = 99881122u64;
    let raw_timeline = json!({
        "gameId": target_game_id,
        "frames": [{"timestamp": 0, "events": []}]
    });
    server.state.set_match_timeline(target_game_id, raw_timeline.clone()).await;

    let initial_reqs = server.state.get_request_count();

    // Concurrently trigger 30 lookups at the exact same millisecond
    let mut handles = Vec::new();
    for _ in 0..30 {
        let client_c = client.clone();
        handles.push(tokio::spawn(async move {
            client_c.get_raw_match_timeline(target_game_id).await
        }));
    }

    for h in handles {
        let res = h.await.unwrap().expect("Must fetch timeline successfully");
        assert_eq!(res["gameId"], target_game_id);
    }

    let reqs_after_first_wave = server.state.get_request_count();
    let wave_requests = reqs_after_first_wave - initial_reqs;
    println!(">>> Concurrent wave: {} requests made out of 30 tasks", wave_requests);
    assert!(
        wave_requests <= 30,
        "Must complete all tasks"
    );

    // Second wave: 100 sequential lookups must hit in-memory LRU cache with ZERO network requests!
    for _ in 0..100 {
        let res = client.get_raw_match_timeline(target_game_id).await.unwrap();
        assert_eq!(res["gameId"], target_game_id);
    }

    let reqs_after_second_wave = server.state.get_request_count();
    assert_eq!(
        reqs_after_second_wave, reqs_after_first_wave,
        "100 repeat lookups must yield 0 network calls (100% cache hit consistency)"
    );

    // Verify cache stats reporting
    let (details, timelines) = client.get_cache_stats();
    assert_eq!(details, 0);
    assert_eq!(timelines, 1);

    // Clear cache and verify next lookup queries the server again
    client.clear_lru_caches();
    let (details_after, timelines_after) = client.get_cache_stats();
    assert_eq!(details_after, 0);
    assert_eq!(timelines_after, 0);

    // Re-ensure credentials in case concurrent mock SSL failures triggered consecutive_failures threshold
    client.set_credentials(server.port, &server.auth_token).await;

    let res_after = client.get_raw_match_timeline(target_game_id).await;
    assert!(res_after.is_some(), "Must successfully query server after cache clear");
    let reqs_after_clear = server.state.get_request_count();
    assert_eq!(reqs_after_clear, reqs_after_second_wave + 1, "Must query server again after cache clear");

    server.stop();
}

// ==============================================================================================
// 2. Concurrency Limiter Stress Tests
// ==============================================================================================

#[tokio::test]
async fn test_concurrency_limiter_strict_permit_exhaustion() {
    let client = LcuClient::new();
    assert_eq!(client.concurrency_limiter.available_permits(), 4);

    // Acquire all 4 permits
    let p1 = client.concurrency_limiter.acquire().await.unwrap();
    let p2 = client.concurrency_limiter.acquire().await.unwrap();
    let p3 = client.concurrency_limiter.acquire().await.unwrap();
    let p4 = client.concurrency_limiter.acquire().await.unwrap();
    assert_eq!(client.concurrency_limiter.available_permits(), 0);

    // 5th acquisition attempt must block and timeout
    let attempt = timeout(Duration::from_millis(40), client.concurrency_limiter.acquire()).await;
    assert!(attempt.is_err(), "Must block and timeout when all permits are exhausted");

    // Release permit 1: available permits becomes 1
    drop(p1);
    assert_eq!(client.concurrency_limiter.available_permits(), 1);

    // Acquisition now succeeds immediately
    let p5 = timeout(Duration::from_millis(40), client.concurrency_limiter.acquire())
        .await
        .expect("Must acquire immediately after release")
        .expect("Valid permit");
    assert_eq!(client.concurrency_limiter.available_permits(), 0);

    drop(p2);
    drop(p3);
    drop(p4);
    drop(p5);
    assert_eq!(client.concurrency_limiter.available_permits(), 4);
}

#[tokio::test]
async fn test_concurrency_limiter_fifo_queuing_fairness() {
    let client = Arc::new(LcuClient::new());

    // Exhaust all 4 permits
    let p1 = client.concurrency_limiter.clone().acquire_owned().await.unwrap();
    let p2 = client.concurrency_limiter.clone().acquire_owned().await.unwrap();
    let p3 = client.concurrency_limiter.clone().acquire_owned().await.unwrap();
    let p4 = client.concurrency_limiter.clone().acquire_owned().await.unwrap();
    assert_eq!(client.concurrency_limiter.available_permits(), 0);

    let (order_tx, mut order_rx) = tokio::sync::mpsc::unbounded_channel::<usize>();

    // Queue 5 waiters in strict sequential order
    let mut waiter_handles = Vec::new();
    for id in 0..5 {
        let client_c = Arc::clone(&client);
        let tx_c = order_tx.clone();
        let handle = tokio::spawn(async move {
            let _permit = client_c.concurrency_limiter.acquire().await.unwrap();
            let _ = tx_c.send(id);
        });
        waiter_handles.push(handle);
        // Small interval ensures tokio's semaphore FIFO wait queue registration order
        sleep(Duration::from_millis(15)).await;
    }

    // Release permits sequentially and observe acquisition order
    drop(p1);
    sleep(Duration::from_millis(20)).await;
    drop(p2);
    sleep(Duration::from_millis(20)).await;
    drop(p3);
    sleep(Duration::from_millis(20)).await;
    drop(p4);

    let mut acquired_order = Vec::new();
    for _ in 0..4 {
        if let Ok(Some(id)) = timeout(Duration::from_millis(100), order_rx.recv()).await {
            acquired_order.push(id);
        }
    }

    assert_eq!(
        acquired_order,
        vec![0, 1, 2, 3],
        "Waiters must be served in exact FIFO queuing order without starvation"
    );

    // Abort the remaining 5th waiter and verify permits return to 4
    for h in waiter_handles {
        h.abort();
    }
    sleep(Duration::from_millis(30)).await;
    assert_eq!(client.concurrency_limiter.available_permits(), 4);
}

#[tokio::test]
async fn test_concurrency_limiter_max_in_flight_under_storm() {
    let client = Arc::new(LcuClient::new());
    let current_in_flight = Arc::new(AtomicUsize::new(0));
    let max_in_flight = Arc::new(AtomicUsize::new(0));
    let concurrency_violation = Arc::new(AtomicBool::new(false));

    let mut handles = Vec::new();
    for _ in 0..60 {
        let client_c = Arc::clone(&client);
        let cur_c = Arc::clone(&current_in_flight);
        let max_c = Arc::clone(&max_in_flight);
        let viol_c = Arc::clone(&concurrency_violation);

        handles.push(tokio::spawn(async move {
            let _permit = client_c.concurrency_limiter.acquire().await.unwrap();
            let cur = cur_c.fetch_add(1, Ordering::SeqCst) + 1;
            max_c.fetch_max(cur, Ordering::SeqCst);
            if cur > 4 {
                viol_c.store(true, Ordering::SeqCst);
            }
            // Simulate variable network I/O latency
            sleep(Duration::from_millis(10)).await;
            cur_c.fetch_sub(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    assert!(!concurrency_violation.load(Ordering::SeqCst), "In-flight concurrency must NEVER exceed 4");
    assert!(max_in_flight.load(Ordering::SeqCst) <= 4, "Observed max in-flight must be <= 4");
    assert_eq!(client.concurrency_limiter.available_permits(), 4, "All permits must be returned");
}

#[tokio::test]
async fn test_concurrency_limiter_permit_leak_immunity_on_errors() {
    let client = LcuClient::new();
    // Target a dead port where connections will immediately fail
    client.set_credentials(19999, "fake_token").await;

    // Send 10 failing requests
    for i in 1..=10 {
        let res = client.get_raw_match_timeline(i).await;
        assert!(res.is_none(), "Connection failure must return None");
    }

    assert_eq!(
        client.concurrency_limiter.available_permits(),
        4,
        "Permits must NEVER leak even when network requests fail"
    );

    // Abort a future while holding permit
    let limiter_c = client.concurrency_limiter.clone();
    let task = tokio::spawn(async move {
        let _permit = limiter_c.acquire().await.unwrap();
        sleep(Duration::from_secs(10)).await;
    });

    sleep(Duration::from_millis(20)).await;
    assert_eq!(client.concurrency_limiter.available_permits(), 3);
    task.abort();
    sleep(Duration::from_millis(20)).await;
    assert_eq!(client.concurrency_limiter.available_permits(), 4, "Aborted task must release permit on drop");
}

// ==============================================================================================
// 3. Batch Chunking Stress Tests for `get_raw_match_history`
// ==============================================================================================

#[tokio::test]
async fn test_batch_chunking_all_boundary_ranges() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "chunking-comprehensive-puuid";
    // Setup a 250-match dataset (game IDs 1000..=1249)
    let mut full_history = canned_match_history(target_puuid, 250, 0.52, 103, &[]);
    full_history["accountId"] = json!(123456789);
    server.state.set_match_history(target_puuid, full_history).await;

    // Boundary Test Cases: (beg_index, end_index, expected_count, expected_requests, first_id, last_id)
    let test_cases = vec![
        // 1. Boundary 0 / inverted (beg=10, end=9 -> normalized to 10..10, count=1)
        (10usize, 9usize, 1usize, 1usize, 1_000_000_010i64, 1_000_000_010i64),
        // 2. Boundary 1 (0..0 -> count=1, 1 request)
        (0, 0, 1, 1, 1_000_000_000, 1_000_000_000),
        // 3. Boundary 19 (0..18 -> count=19, 1 request)
        (0, 18, 19, 1, 1_000_000_000, 1_000_000_018),
        // 4. Boundary 20 (0..19 -> count=20, 1 request, single request threshold)
        (0, 19, 20, 1, 1_000_000_000, 1_000_000_019),
        // 5. Boundary 21 (0..20 -> count=21, 2 requests: 0..19, 20..20)
        (0, 20, 21, 2, 1_000_000_000, 1_000_000_020),
        // 6. Boundary 50 (0..49 -> count=50, 3 requests: 0..19, 20..39, 40..49)
        (0, 49, 50, 3, 1_000_000_000, 1_000_000_049),
        // 7. Boundary 100 (0..99 -> count=100, 5 requests: 20 per chunk)
        (0, 99, 100, 5, 1_000_000_000, 1_000_000_099),
        // 8. Boundary 200 (0..199 -> count=200, 10 requests: 20 per chunk, max single page)
        (0, 199, 200, 10, 1_000_000_000, 1_000_000_199),
        // Non-zero offset boundary (180..199 -> count=20, 1 request)
        (180, 199, 20, 1, 1_000_000_180, 1_000_000_199),
        // Non-zero offset cross-chunk boundary (180..200 -> count=21, 2 requests)
        (180, 200, 21, 2, 1_000_000_180, 1_000_000_200),
    ];

    for (beg, end, exp_count, exp_reqs, exp_first_id, exp_last_id) in test_cases {
        let reqs_before = server.state.get_request_count();
        let res = client
            .get_raw_match_history(target_puuid, beg, end)
            .await
            .unwrap_or_else(|| panic!("Failed fetching beg={} end={}", beg, end));

        let reqs_after = server.state.get_request_count();
        let reqs_made = reqs_after - reqs_before;

        let games = res["games"]["games"]
            .as_array()
            .expect("games.games must be a JSON array");

        assert_eq!(
            games.len(),
            exp_count,
            "Count mismatch for beg={} end={}: expected {}, got {}",
            beg,
            end,
            exp_count,
            games.len()
        );

        assert_eq!(
            reqs_made,
            exp_reqs,
            "Request count mismatch for beg={} end={}: expected {} requests, made {}",
            beg,
            end,
            exp_reqs,
            reqs_made
        );

        assert_eq!(
            games[0]["gameId"].as_i64().unwrap(),
            exp_first_id,
            "First game ID mismatch for beg={} end={}",
            beg,
            end
        );

        assert_eq!(
            games[exp_count - 1]["gameId"].as_i64().unwrap(),
            exp_last_id,
            "Last game ID mismatch for beg={} end={}",
            beg,
            end
        );

        // Verify root accountId preservation
        assert_eq!(res["accountId"], 123456789);
    }

    server.stop();
}

#[tokio::test]
async fn test_batch_chunking_early_termination_on_short_history() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "short-history-puuid";
    // Server only has 14 games available
    let short_history = canned_match_history(target_puuid, 14, 0.50, 157, &[]);
    server.state.set_match_history(target_puuid, short_history).await;

    let reqs_before = server.state.get_request_count();

    // Client requests 100 matches (0..99)
    let res = client
        .get_raw_match_history(target_puuid, 0, 99)
        .await
        .expect("Must fetch available games");

    let reqs_after = server.state.get_request_count();
    let reqs_made = reqs_after - reqs_before;

    let games = res["games"]["games"].as_array().expect("games array");
    assert_eq!(games.len(), 14, "Must return all 14 available matches");

    // Because first batch returned 14 (< 20), client MUST detect end-of-history and stop immediately!
    assert_eq!(
        reqs_made, 1,
        "Must NOT issue redundant requests when client hits end-of-history"
    );

    server.stop();
}

#[tokio::test]
async fn test_batch_chunking_clamp_exceeding_200() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "oversize-puuid";
    let full_history = canned_match_history(target_puuid, 250, 0.50, 157, &[]);
    server.state.set_match_history(target_puuid, full_history).await;

    let reqs_before = server.state.get_request_count();

    // Request 500 matches (0..499) -> must clamp to max 200 matches
    let res = client
        .get_raw_match_history(target_puuid, 0, 499)
        .await
        .expect("Must succeed");

    let reqs_after = server.state.get_request_count();
    let reqs_made = reqs_after - reqs_before;

    let games = res["games"]["games"].as_array().expect("games array");
    assert_eq!(games.len(), 200, "Must be clamped to exactly 200 matches");
    assert_eq!(reqs_made, 10, "Must make at most 10 requests (200 / 20 = 10)");

    server.stop();
}

#[tokio::test]
async fn test_batch_chunking_empty_history_and_invalid_puuid() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    // Empty PUUID must immediately return None with 0 network calls
    let reqs_before = server.state.get_request_count();
    let res_empty_puuid = client.get_raw_match_history("", 0, 19).await;
    assert!(res_empty_puuid.is_none());
    assert_eq!(server.state.get_request_count(), reqs_before);

    // Empty matches array on server
    let target_puuid = "empty-matches-puuid";
    let empty_history = json!({
        "accountId": 999,
        "games": { "games": [] }
    });
    server.state.set_match_history(target_puuid, empty_history).await;

    let res_empty = client.get_raw_match_history(target_puuid, 0, 50).await;
    println!(">>> Empty matches history returned: {:?}", res_empty.is_some());
    assert!(res_empty.is_some(), "Empty match history (0..50) must return Some");
    let res_val = res_empty.unwrap();
    assert_eq!(res_val["accountId"], 999);
    assert_eq!(res_val["games"]["games"], json!([]));

    server.stop();
}

// ==============================================================================================
// 4. SGP URL Resolution and Token Stress Tests
// ==============================================================================================

#[test]
fn test_sgp_unlisted_shards_and_dynamic_resolution() {
    // 1. Unlisted Tencent Shards: Must dynamically construct correct k8s endpoint
    let unlisted_tencent = vec![
        ("TENCENT_WT1", "https://wt1-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_WT7", "https://wt7-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_HN2", "https://hn2-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_BGP1", "https://bgp1-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_EDU1", "https://edu1-k8s-sgp.lol.qq.com:21019"),
    ];

    for (server_id, expected_url) in unlisted_tencent {
        let cfg = resolve_server_config(server_id)
            .unwrap_or_else(|| panic!("Dynamic resolution failed for {}", server_id));
        assert_eq!(cfg.match_history, expected_url);
        assert_eq!(cfg.common, expected_url);
        assert!(cfg.is_tencent);
        assert!(cfg.region_path_param.is_none());

        // Sub-ID resolution for unlisted shard
        let sub_id = resolve_sub_id(server_id, &cfg, None);
        let expected_sub = server_id.strip_prefix("TENCENT_").unwrap();
        assert_eq!(sub_id, expected_sub);
    }

    // 2. Unlisted Global Shards: Must return None safely without panic
    let invalid_shards = vec!["UNKNOWN_SHARD", "EUW99", "NOT_A_REGION", "", "   "];
    for invalid in invalid_shards {
        assert!(
            resolve_server_config(invalid).is_none(),
            "Invalid shard '{}' must return None",
            invalid
        );
    }
}

#[test]
fn test_sgp_tencent_vs_global_cluster_handling() {
    // 1. Tencent cluster server ID casing and whitespace resilience
    assert_eq!(get_sgp_server_id("tencent", Some("hn1")), "TENCENT_HN1");
    assert_eq!(get_sgp_server_id("TENCENT", Some("  hn10  ")), "TENCENT_HN10");
    assert_eq!(get_sgp_server_id("Tencent", Some("bgp2")), "TENCENT_BGP2");
    assert_eq!(get_sgp_server_id("tencent", None), "TENCENT_");

    // 2. Global cluster region normalization & aliases
    let global_mappings = vec![
        ("NA", "NA1"),
        ("na", "NA1"),
        ("BR", "BR1"),
        ("TR", "TR1"),
        ("LAN", "LA1"),
        ("LAS", "LA2"),
        ("OCE", "OC1"),
        ("EUW1", "EUW"),
        ("euw1", "EUW"),
        ("JP1", "JP"),
        ("jp1", "JP"),
        ("KR", "KR"),
        ("kr", "KR"),
        ("TW2", "TW2"),
        ("SG2", "SG2"),
        ("PH2", "PH2"),
        ("VN2", "VN2"),
        ("RU", "RU"),
        ("TH2", "TH2"),
    ];

    for (input, expected) in global_mappings {
        assert_eq!(
            get_sgp_server_id(input, None),
            expected,
            "Mapping failed for region '{}'",
            input
        );
    }

    // 3. Sub-ID resolution distinction:
    // Global with region_path_param:
    let euw_cfg = resolve_server_config("EUW").unwrap();
    assert_eq!(resolve_sub_id("EUW", &euw_cfg, None), "EUW1");
    let jp_cfg = resolve_server_config("JP").unwrap();
    assert_eq!(resolve_sub_id("JP", &jp_cfg, None), "JP1");
    let pbe_cfg = resolve_server_config("PBE").unwrap();
    assert_eq!(resolve_sub_id("PBE", &pbe_cfg, None), "PBE1");

    // Global without region_path_param (uses server_id):
    let kr_cfg = resolve_server_config("KR").unwrap();
    assert_eq!(resolve_sub_id("KR", &kr_cfg, None), "KR");
    let na1_cfg = resolve_server_config("NA1").unwrap();
    assert_eq!(resolve_sub_id("NA1", &na1_cfg, None), "NA1");

    // Tencent (strips TENCENT_ or uses rso_platform_id):
    let hn1_cfg = resolve_server_config("TENCENT_HN1").unwrap();
    assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, Some("HN1")), "HN1");
    assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, Some("  hn1  ")), "HN1");
    assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, None), "HN1");
}

#[tokio::test]
async fn test_sgp_sub_id_placeholders_and_url_formatting() {
    let tokens = SgpTokens {
        entitlements: Some(EntitlementsToken {
            access_token: "test_access_jwt".to_string(),
            entitlements: vec![],
            issuer: None,
            subject: None,
            token: None,
        }),
        league_session_token: Some("test_session_jwt".to_string()),
    };

    // 1. EUW Client sub-id check
    let euw_client = SgpClient::with_credentials("EUW", None, tokens.clone());
    assert_eq!(euw_client.current_sub_id().await, "EUW1");

    // 2. Tencent Client sub-id check
    let tencent_client = SgpClient::with_credentials("TENCENT_HN1", Some("HN1"), tokens.clone());
    assert_eq!(tencent_client.current_sub_id().await, "HN1");

    // 3. Dynamic Tencent Client sub-id check
    let dynamic_client = SgpClient::with_credentials("TENCENT_WT1", None, tokens.clone());
    assert_eq!(dynamic_client.current_sub_id().await, "WT1");

    // Verify URL patterns manually with exact format verification
    let euw_cfg = resolve_server_config("EUW").unwrap();
    let game_id = 99887766u64;
    let expected_euw_summary_url = format!(
        "{}/match-history-query/v1/products/lol/{}_{}/SUMMARY",
        euw_cfg.match_history.trim_end_matches('/'),
        "EUW1",
        game_id
    );
    assert_eq!(
        expected_euw_summary_url,
        "https://euc1-red.pp.sgp.pvp.net/match-history-query/v1/products/lol/EUW1_99887766/SUMMARY"
    );

    let expected_euw_gsm_url = format!(
        "{}/gsm/v1/ledge/region/{}/puuid/{}",
        euw_cfg.common.trim_end_matches('/'),
        "EUW1",
        "sample-puuid"
    );
    assert_eq!(
        expected_euw_gsm_url,
        "https://euw-red.lol.sgp.pvp.net/gsm/v1/ledge/region/EUW1/puuid/sample-puuid"
    );
}

#[tokio::test]
async fn test_sgp_token_formatting_and_error_paths() {
    // 1. Token model deserialization: full fields
    let full_json = json!({
        "accessToken": "eyJh...valid_jwt",
        "entitlements": ["lol", "sgp"],
        "issuer": "https://auth.riotgames.com",
        "subject": "test-subject",
        "token": "raw_token"
    });
    let ent: EntitlementsToken = serde_json::from_value(full_json).unwrap();
    assert_eq!(ent.access_token, "eyJh...valid_jwt");
    assert_eq!(ent.entitlements.len(), 2);
    assert_eq!(ent.subject.as_deref(), Some("test-subject"));

    // 2. Token model deserialization: minimal fields (omitted defaults)
    let minimal_json = json!({
        "accessToken": "eyJh...minimal_jwt"
    });
    let min_ent: EntitlementsToken = serde_json::from_value(minimal_json).unwrap();
    assert_eq!(min_ent.access_token, "eyJh...minimal_jwt");
    assert!(min_ent.entitlements.is_empty());
    assert!(min_ent.issuer.is_none());

    // 3. Token model deserialization: missing required accessToken
    let invalid_json = json!({
        "issuer": "https://auth.riotgames.com"
    });
    let parse_err = serde_json::from_value::<EntitlementsToken>(invalid_json);
    assert!(parse_err.is_err(), "Must reject JSON missing accessToken");

    // 4. Header injection protection & Bearer formatting
    let valid_token = "valid_token_12345";
    let bearer = format!("Bearer {}", valid_token);
    let header_val = HeaderValue::from_str(&bearer);
    assert!(header_val.is_ok());

    // Hostile token with CRLF injection attempt
    let hostile_token = "valid_token\r\nInjected-Header: evil\r\n";
    let hostile_bearer = format!("Bearer {}", hostile_token);
    let hostile_val = HeaderValue::from_str(&hostile_bearer);
    assert!(hostile_val.is_err(), "Must reject CRLF injection in authorization header");

    // 5. SgpClient error handling when tokens are absent
    let empty_tokens_client = SgpClient::with_credentials("EUW", None, SgpTokens::default());
    let mh_res = empty_tokens_client.get_match_history_summary("test_puuid", 0, 10).await;
    assert!(mh_res.is_err());
    assert_eq!(mh_res.unwrap_err(), "Entitlements token missing");

    let gsm_res = empty_tokens_client.get_gsm_by_puuid("test_puuid").await;
    assert!(gsm_res.is_err());
    assert_eq!(gsm_res.unwrap_err(), "League session token missing");

    // 6. SgpClient error handling when server is unconfigured
    let unconfigured_client = SgpClient::with_credentials("NON_EXISTENT_SHARD", None, SgpTokens::default());
    let mh_unconfigured = unconfigured_client.get_match_history_summary("test_puuid", 0, 10).await;
    assert!(mh_unconfigured.is_err());
    assert_eq!(mh_unconfigured.unwrap_err(), "SGP server not configured");
}

// ==============================================================================================
// 5. Empirical Bug Demonstrations & Edge Case Probes
// ==============================================================================================

#[tokio::test]
async fn test_empirical_finding_cache_stampede_under_concurrency() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let game_id = 77112233u64;
    server
        .state
        .set_match_timeline(
            game_id,
            json!({ "gameId": game_id, "frames": [] }),
        )
        .await;

    let reqs_before = server.state.get_request_count();

    // 8 concurrent requests for the exact same uncached game_id
    let mut handles = Vec::new();
    for _ in 0..8 {
        let client_c = client.clone();
        handles.push(tokio::spawn(async move {
            client_c.get_raw_match_timeline(game_id).await
        }));
    }

    for h in handles {
        assert!(h.await.unwrap().is_some());
    }

    let reqs_after = server.state.get_request_count();
    let requests_made = reqs_after - reqs_before;
    println!(
        ">>> Cache Stampede Proof: {} network requests made for 8 concurrent lookups of SAME game",
        requests_made
    );

    // Verify cache stampede is prevented: single network request for concurrent lookups
    assert_eq!(
        requests_made, 1,
        "Stampede prevented: exactly 1 network request was made for identical key under concurrency"
    );

    server.stop();
}

#[tokio::test]
async fn test_empirical_finding_empty_match_history_returns_none() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "newbie-zero-matches-puuid";
    let empty_payload = json!({
        "accountId": 987654,
        "platformId": "HN1",
        "games": {
            "games": []
        }
    });
    server.state.set_match_history(target_puuid, empty_payload).await;

    // A fresh account with 0 matches:
    let res = client.get_raw_match_history(target_puuid, 0, 19).await;

    // Verify empty match history acceptance:
    assert!(
        res.is_some(),
        "Accounts with 0 matches legitimately receive Some with empty games array"
    );
    let val = res.unwrap();
    assert_eq!(val["accountId"], 987654);
    assert_eq!(val["games"]["games"], json!([]));

    server.stop();
}

