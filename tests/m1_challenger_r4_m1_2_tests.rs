//! Adversarial Empirical Challenge Suite for Milestone 1:
//! 1. LCU Client Robustness (Singleflight Negative Caching, URL encoding, 401 Unauthorized handling)
//! 2. Encounter Recording and IPC Storage Boundary & Edge Cases:
//!    - Edge Case 1: Empty or missing gameData in session (spectate mode or client crash)
//!    - Edge Case 2: Local player PUUID missing from participants (spectator / bot match)
//!    - Edge Case 3: Rapid session transitions (InProgress -> PreEndOfGame -> EndOfGame -> WaitingForStats) & concurrent deduplication
//!    - Edge Case 4: Tag phrase constraints (strings > 100 chars, lists > 20 items, whitespace trimming, UTF-8 boundaries)
//!    - Edge Case 5: Special characters, Unicode, emojis, and SQL injection strings ('; DROP TABLE;')

#[path = "lcu_mock_harness/mod.rs"]
mod lcu_mock_harness;

use lcu_mock_harness::MockLcuServer;
use lol_hex_rust::lcu::client::LcuClient;
use lol_hex_rust::storage::models::*;
use lol_hex_rust::storage::StorageManager;
use reqwest::StatusCode;
use serde_json::json;
use std::sync::Arc;

// ==============================================================================================
// Part 1: LCU Client Robustness & Network Boundary Tests
// ==============================================================================================

#[tokio::test]
async fn test_negative_caching_404_detail_single_and_concurrent() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;
    assert!(client.is_connected().await);

    let test_game_id = "9999901";
    let test_gid_u64 = 9999901u64;

    assert!(!client.is_negative_cached_detail(test_gid_u64));
    let req_before = server.state.get_request_count();

    let res = client.get_match_detail(test_game_id).await;
    assert!(res.is_none(), "404 detail must return None");
    assert!(client.is_negative_cached_detail(test_gid_u64));

    let req_after_first = server.state.get_request_count();
    let requests_made = req_after_first - req_before;
    assert!(requests_made > 0);

    for _ in 0..30 {
        let res2 = client.get_match_detail(test_game_id).await;
        assert!(res2.is_none());
    }

    assert_eq!(server.state.get_request_count(), req_after_first);

    let mut tasks = Vec::new();
    for _ in 0..50 {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            client_clone.get_match_detail(test_game_id).await
        }));
    }

    for task in tasks {
        let r = task.await.expect("task join");
        assert!(r.is_none());
    }

    assert_eq!(server.state.get_request_count(), req_after_first);
    assert!(client.get_raw_match_detail(test_gid_u64).await.is_none());
    assert_eq!(server.state.get_request_count(), req_after_first);
}

#[tokio::test]
async fn test_negative_caching_404_timeline_single_and_concurrent() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let test_gid = 8888801u64;
    assert!(!client.is_negative_cached_timeline(test_gid));

    let req_before = server.state.get_request_count();
    let res = client.get_raw_match_timeline(test_gid).await;
    assert!(res.is_none());
    assert!(client.is_negative_cached_timeline(test_gid));

    let req_after_first = server.state.get_request_count();
    assert!(req_after_first > req_before);

    for _ in 0..20 {
        assert!(client.get_raw_match_timeline(test_gid).await.is_none());
        assert!(client.get_game_timeline(test_gid).await.is_none());
        assert!(client.get_match_timeline(test_gid).await.is_none());
    }
    assert_eq!(server.state.get_request_count(), req_after_first);

    let mut tasks = Vec::new();
    for _ in 0..40 {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            client_clone.get_raw_match_timeline(test_gid).await
        }));
    }

    for task in tasks {
        let r = task.await.expect("task join");
        assert!(r.is_none());
    }
    assert_eq!(server.state.get_request_count(), req_after_first);
}

#[tokio::test]
async fn test_singleflight_stampede_protection_on_404() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(LcuClient::new());
    client.set_credentials(server.port, &server.auth_token).await;

    let stampede_gid_str = "7777701";
    let req_before = server.state.get_request_count();

    let mut tasks = Vec::new();
    for _ in 0..30 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.get_match_detail(stampede_gid_str).await
        }));
    }

    for t in tasks {
        let res = t.await.expect("join task");
        assert!(res.is_none());
    }

    let req_after = server.state.get_request_count();
    let requests_made = req_after - req_before;
    assert!(
        requests_made <= 2,
        "Singleflight must suppress stampede: expected <= 2 requests, got {}",
        requests_made
    );
    assert!(client.get_match_detail(stampede_gid_str).await.is_none());
    assert_eq!(server.state.get_request_count(), req_after);
}

#[tokio::test]
async fn test_singleflight_stampede_protection_on_timeline_404() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(LcuClient::new());
    client.set_credentials(server.port, &server.auth_token).await;

    let stampede_gid = 6666601u64;
    let req_before = server.state.get_request_count();

    let mut tasks = Vec::new();
    for _ in 0..30 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.get_raw_match_timeline(stampede_gid).await
        }));
    }

    for t in tasks {
        let res = t.await.expect("join task");
        assert!(res.is_none());
    }

    let req_after = server.state.get_request_count();
    let requests_made = req_after - req_before;
    assert!(
        requests_made <= 2,
        "Timeline singleflight must suppress stampede: expected <= 2 requests, got {}",
        requests_made
    );
}

fn mirror_percent_encode_path_segment(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{:02X}", b);
            }
        }
    }
    encoded
}

#[tokio::test]
async fn test_chat_room_id_url_encoding_special_characters_and_safety() {
    let client = LcuClient::new();
    assert!(client.get_chat_conversation_participants("").await.is_empty());
    assert!(!client.send_chat_message("", "hello").await);
    assert!(!client.send_chat_message("valid_room", "").await);
    assert!(!client.send_chat_message("", "").await);

    let test_cases = vec![
        ("champ-select#12345", "champ-select%2312345"),
        ("room?session=abc&mode=aram", "room%3Fsession%3Dabc%26mode%3Daram"),
        (
            "champ-select-123@champ-select.lol.riotgames.com",
            "champ-select-123%40champ-select.lol.riotgames.com",
        ),
        ("custom room with spaces", "custom%20room%20with%20spaces"),
        ("战队开黑房#2026", "%E6%88%98%E9%98%9F%E5%BC%80%E9%BB%91%E6%88%BF%232026"),
        ("nested/path/room", "nested%2Fpath%2Froom"),
    ];

    for (raw, expected_encoded) in test_cases {
        let encoded = mirror_percent_encode_path_segment(raw);
        assert_eq!(encoded, expected_encoded);

        let unencoded_url_str = format!("https://127.0.0.1:1234/lol-chat/v1/conversations/{}/participants", raw);
        if raw.contains('#') {
            if let Ok(bad_parsed) = reqwest::Url::parse(&unencoded_url_str) {
                assert!(bad_parsed.fragment().is_some());
                assert!(!bad_parsed.path().ends_with("/participants"));
            }
        }

        let encoded_url_str = format!("https://127.0.0.1:1234/lol-chat/v1/conversations/{}/participants", encoded);
        let parsed = reqwest::Url::parse(&encoded_url_str).expect("Encoded URL must parse successfully");
        assert!(parsed.fragment().is_none());
        assert!(parsed.query().is_none());
        assert!(parsed.path().ends_with("/participants"));
        let expected_path = format!("/lol-chat/v1/conversations/{}/participants", expected_encoded);
        assert_eq!(parsed.path(), expected_path);
    }
}

#[tokio::test]
async fn test_unauthorized_401_invalidates_credentials_via_fallback_candidate() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;
    *client.current_phase.write().await = "ChampSelect".to_string();
    *client.current_session.write().await = json!({"id": 100});
    assert!(client.is_connected().await);

    server.state.set_status_override(Some(StatusCode::UNAUTHORIZED)).await;
    let res = client.get_match_detail("55555").await;
    assert!(res.is_none());

    assert!(!client.is_connected().await);
    assert_eq!(*client.current_phase.read().await, "None");
    assert_eq!(*client.current_session.read().await, serde_json::Value::Null);

    client.set_credentials(server.port, &server.auth_token).await;
    assert!(client.is_connected().await);
    let res_timeline = client.get_raw_match_timeline(44444).await;
    assert!(res_timeline.is_none());
    assert!(!client.is_connected().await);
}

#[tokio::test]
async fn test_cache_clearing_and_isolation() {
    let client = LcuClient::new();
    let gid = 123456u64;

    client.insert_negative_cached_detail(gid);
    assert!(client.is_negative_cached_detail(gid));
    assert!(!client.is_negative_cached_timeline(gid));

    client.insert_negative_cached_timeline(gid);
    assert!(client.is_negative_cached_timeline(gid));

    client.clear_lru_caches();
    assert!(!client.is_negative_cached_detail(gid));
    assert!(!client.is_negative_cached_timeline(gid));
}

// ==============================================================================================
// Part 2: Encounter Recording and IPC Storage Boundary & Edge Case Verification
// ==============================================================================================

/// Edge Case 1: Empty or missing gameData in session (spectate mode or client crash).
#[test]
fn test_edge_case_empty_or_missing_gamedata() {
    let storage = StorageManager::open_in_memory().expect("open in-memory db");

    // Case 1A: Null or empty game_id (<= 0)
    let res = storage.record_encountered_game(0, "self-p", &["p1".into()], "HN1", "HN1", "RANKED");
    assert!(res.is_ok(), "game_id <= 0 must gracefully return Ok without inserting");
    assert_eq!(storage.count_encountered_games("self-p", "p1").unwrap(), 0);

    let res_neg = storage.record_encountered_game(-100, "self-p", &["p1".into()], "HN1", "HN1", "RANKED");
    assert!(res_neg.is_ok(), "Negative game_id must return Ok without inserting");
    assert_eq!(storage.count_encountered_games("self-p", "p1").unwrap(), 0);

    // Case 1B: Empty self_puuid
    let res_empty_self = storage.record_encountered_game(1001, "", &["p1".into()], "HN1", "HN1", "RANKED");
    assert!(res_empty_self.is_ok(), "Empty self_puuid must return Ok without inserting");

    // Case 1C: Empty peer list
    let empty_peers: Vec<String> = vec![];
    let res_empty_peers = storage.record_encountered_game(1001, "self-p", &empty_peers, "HN1", "HN1", "RANKED");
    assert!(res_empty_peers.is_ok(), "Empty peers must return Ok without inserting");

    // Case 1D: Peer list containing only self_puuid or empty strings
    let invalid_peers = vec!["".to_string(), "self-p".to_string(), "   ".to_string()];
    let res_invalid = storage.record_encountered_game(1001, "self-p", &invalid_peers, "HN1", "HN1", "RANKED");
    assert!(res_invalid.is_ok());
    // Only trimmed non-empty and non-self peers should be inserted
    assert_eq!(storage.count_encountered_games("self-p", "self-p").unwrap(), 0);

    // Case 1E: Simulation of gameflow session JSON without gameData or with empty gameData
    let empty_session = json!({});
    let game_data_opt = empty_session.get("gameData");
    assert!(game_data_opt.is_none(), "Session without gameData must yield None");

    let null_session = serde_json::Value::Null;
    assert!(null_session.is_null());

    let session_empty_gd = json!({ "gameData": {} });
    let gid = session_empty_gd.get("gameData").and_then(|gd| gd.get("gameId")).and_then(|v| v.as_i64()).unwrap_or(0);
    assert_eq!(gid, 0, "Empty gameData must resolve gameId to 0");
}

/// Edge Case 2: Local player PUUID missing from participants (spectator mode / bot match).
///
/// EMPIRICAL AUDIT FINDING:
/// When spectating a match, the spectator's PUUID is NOT in teamOne or teamTwo.
/// If runtime.rs extracts participants without verifying that self_puuid is among them,
/// `all_players.filter(|p| p != &self_puuid)` retains all 10 players!
/// This test proves the discrepancy and demonstrates the exact condition that must be guarded.
#[test]
fn test_edge_case_spectator_mode_local_player_missing_from_participants() {
    let storage = StorageManager::open_in_memory().expect("open in-memory db");

    let self_puuid = "spectator_puuid_abc";
    let team_one = vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string()];
    let team_two = vec!["p6".to_string(), "p7".to_string(), "p8".to_string(), "p9".to_string(), "p10".to_string()];

    let mut all_players = Vec::new();
    all_players.extend(team_one);
    all_players.extend(team_two);

    assert_eq!(all_players.len(), 10);

    // Check whether self_puuid is in the participants
    let is_self_participant = all_players.iter().any(|p| p == self_puuid);
    assert!(!is_self_participant, "In spectator mode, local player is NOT in teamOne or teamTwo");

    // Naive filter as in runtime.rs lines 785-788:
    let peer_puuids: Vec<String> = all_players
        .iter()
        .filter(|p| p.as_str() != self_puuid)
        .cloned()
        .collect();

    // ADVERSARIAL OBSERVATION:
    // Because self_puuid was missing, peer_puuids.len() is 10 (all participants), NOT 9!
    assert_eq!(peer_puuids.len(), 10, "Without participant check, 10 peers are extracted instead of 9");

    // If caller properly checks participant status:
    let should_record = is_self_participant && !peer_puuids.is_empty();
    assert!(!should_record, "Spectator mode MUST NOT record encounters");

    // If improperly called anyway, verify storage behavior:
    let res = storage.record_encountered_game(55555, self_puuid, &peer_puuids, "HN1", "HN1", "RANKED_SOLO_5x5");
    assert!(res.is_ok());

    // Storage accepted 10 peers because StorageManager trusts caller's list:
    let query = EncounteredGamesQueryDto {
        puuid: "p1".to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(1),
        page_size: Some(10),
        time_order: Some("desc".to_string()),
    };
    let page = storage.query_encountered_games(&query).unwrap();
    assert_eq!(page.total, 1, "If unvalidated, p1 was falsely recorded as encountered");

    // Clean up
    assert!(storage.delete_encountered_game(page.data[0].id).unwrap());
}

/// Edge Case 3: Session transitions rapidly between InProgress -> PreEndOfGame -> EndOfGame -> WaitingForStats
/// Verifies triple deduplication: in-memory HashSet, last_recorded_game_id, and SQLite UNIQUE constraint.
#[tokio::test]
async fn test_edge_case_rapid_session_transitions_and_concurrent_deduplication() {
    let storage = Arc::new(StorageManager::open_in_memory().expect("open in-memory db"));
    let game_id = 777123456i64;
    let self_puuid = "local_player_puuid";
    let peers: Vec<String> = (1..=9).map(|i| format!("peer_player_{}", i)).collect();

    // 1. First transition: PreEndOfGame
    storage.record_encountered_game(game_id, self_puuid, &peers, "HN1", "HN1", "RANKED_SOLO_5x5")
        .expect("PreEndOfGame record");

    // Verify 9 peers recorded
    for peer in &peers {
        assert_eq!(storage.count_encountered_games(self_puuid, peer).unwrap(), 1);
    }

    // 2. Second transition: EndOfGame (same game_id)
    storage.record_encountered_game(game_id, self_puuid, &peers, "HN1", "HN1", "RANKED_SOLO_5x5")
        .expect("EndOfGame record (duplicate)");

    // Counts must STILL be 1 (no duplicate rows in encountered_games)
    for peer in &peers {
        assert_eq!(storage.count_encountered_games(self_puuid, peer).unwrap(), 1);
    }

    // 3. Third transition: WaitingForStats (same game_id)
    storage.record_encountered_game(game_id, self_puuid, &peers, "HN1", "HN1", "RANKED_SOLO_5x5")
        .expect("WaitingForStats record (duplicate)");

    for peer in &peers {
        assert_eq!(storage.count_encountered_games(self_puuid, peer).unwrap(), 1);
    }

    // 4. Concurrent stampede stress: 50 concurrent tasks calling record_encountered_game on the same game_id
    let mut tasks = Vec::new();
    for _ in 0..50 {
        let storage_c = storage.clone();
        let peers_c = peers.clone();
        tasks.push(tokio::spawn(async move {
            storage_c.record_encountered_game(game_id, self_puuid, &peers_c, "HN1", "HN1", "RANKED_SOLO_5x5")
        }));
    }

    for t in tasks {
        let r = t.await.expect("join");
        assert!(r.is_ok(), "Concurrent records must succeed without deadlock or constraint failure");
    }

    // Must still have exactly 1 record per peer (9 total rows for this game)
    for peer in &peers {
        assert_eq!(storage.count_encountered_games(self_puuid, peer).unwrap(), 1);
    }

    let conn = storage.raw_connection();
    let conn_guard = conn.lock().unwrap();
    let total_rows: i64 = conn_guard.query_row(
        "SELECT count(*) FROM encountered_games WHERE game_id = ?1;",
        [game_id],
        |r| r.get(0)
    ).unwrap();
    assert_eq!(total_rows, 9, "Total rows in encountered_games must be exactly 9, never duplicated");
}

/// Edge Case 4: Tag phrase constraints:
/// - strings exceeding 100 characters (Unicode scalar counts)
/// - lists exceeding 20 items
/// - whitespace trimming and exclusion of empty strings
/// - UTF-8 character boundary safety (Chinese, emojis)
#[test]
fn test_edge_case_tag_phrase_constraints_and_normalization() {
    let storage = StorageManager::open_in_memory().expect("open in-memory db");

    // 4A: Whitespace trimming and empty exclusion
    let raw_whitespace = vec![
        "   ".to_string(),
        "\t\t\n".to_string(),
        "  通天代  ".to_string(),
        "".to_string(),
        "  绝活哥  \r\n".to_string(),
    ];
    let norm1 = normalize_saved_player_tag_phrases(&raw_whitespace);
    assert_eq!(norm1, vec!["通天代".to_string(), "绝活哥".to_string()]);

    // 4B: Strings exceeding 100 characters (ASCII)
    let long_ascii = "A".repeat(150);
    let norm_ascii = normalize_saved_player_tag_phrase(&long_ascii);
    assert_eq!(norm_ascii.len(), 100);
    assert_eq!(norm_ascii.chars().count(), 100);

    // 4C: Strings exceeding 100 characters (Chinese multibyte chars: 3 bytes each)
    let long_cjk = "通天代".repeat(40); // 120 Chinese characters = 360 bytes
    assert_eq!(long_cjk.chars().count(), 120);
    let norm_cjk = normalize_saved_player_tag_phrase(&long_cjk);
    assert_eq!(norm_cjk.chars().count(), 100, "Must be capped at exactly 100 Unicode characters");
    assert_eq!(norm_cjk.len(), 300, "100 Chinese characters = 300 UTF-8 bytes");

    // 4D: Strings exceeding 100 characters (4-byte emojis)
    let long_emoji = "🔥".repeat(120); // 120 emojis = 480 bytes
    let norm_emoji = normalize_saved_player_tag_phrase(&long_emoji);
    assert_eq!(norm_emoji.chars().count(), 100);
    assert_eq!(norm_emoji.len(), 400);

    // 4E: Lists exceeding 20 items (SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS)
    let mut thirty_phrases = Vec::new();
    for i in 1..=30 {
        thirty_phrases.push(format!("Phrase_{}", i));
    }
    let norm_list = normalize_saved_player_tag_phrases(&thirty_phrases);
    assert_eq!(norm_list.len(), 20, "Phrase list must be clamped to at most 20 items");
    assert_eq!(norm_list[0], "Phrase_1");
    assert_eq!(norm_list[19], "Phrase_20");

    // 4F: Deduplication within list
    let duplicate_phrases = vec![
        "演员".to_string(),
        "  演员  ".to_string(),
        "绝活哥".to_string(),
        "演员".to_string(),
        "带崩全队".to_string(),
    ];
    let norm_dedup = normalize_saved_player_tag_phrases(&duplicate_phrases);
    assert_eq!(norm_dedup, vec!["演员".to_string(), "绝活哥".to_string(), "带崩全队".to_string()]);

    // 4G: Persistence in SQLite
    let updated = storage.update_player_tag_phrases(&thirty_phrases).expect("update phrases");
    assert_eq!(updated.len(), 20);

    let retrieved = storage.get_player_tag_phrases().expect("get phrases");
    assert_eq!(retrieved.len(), 20);
    assert_eq!(retrieved[0], "Phrase_1");
    assert_eq!(retrieved[19], "Phrase_20");
}

/// Edge Case 5: Special characters, Unicode, emojis, and SQL injection strings ('; DROP TABLE;')
/// Verifies parameterized query safety, Unicode persistence, and database schema integrity.
#[test]
fn test_edge_case_special_characters_unicode_and_sql_injection() {
    let storage = StorageManager::open_in_memory().expect("open in-memory db");

    let test_vectors = vec![
        // 1. Classical SQL Injection
        "'; DROP TABLE saved_player_tags; --",
        "'; DROP TABLE encountered_games; --",
        "' OR '1'='1",
        "Robert'); DROP TABLE app_settings;--",
        "\" UNION SELECT * FROM app_settings --",
        "admin'--",
        // 2. Multilingual & Unicode & CJK
        "绝活哥·峡谷第一亚索 🐉⚡",
        "مرحبا بالعالم (Arabic RTL)",
        "Привет мир (Russian Cyrillic)",
        "こんにちは世界 (Japanese)",
        // 3. Emojis and Symbols
        "🤡👑💩🔥💯✨🎯🚀⚠️",
        // 4. Quotes and escapes
        "Let's play \"League\" \\ and \t newline \n test",
        // 5. HTML / XSS Injection
        "<script>alert('xss')</script>",
        "<img src=x onerror=alert(1)>",
    ];

    let self_puuid = "tester_self_puuid";

    for (idx, payload) in test_vectors.iter().enumerate() {
        let target_puuid = format!("target_puuid_{}", idx);

        // A. Upsert tag with payload
        let dto = UpdatePlayerTagDto {
            puuid: target_puuid.clone(),
            self_puuid: self_puuid.to_string(),
            region: Some("HN1".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some(payload.to_string()),
        };

        let saved = storage.update_player_tag(&dto)
            .expect(&format!("Saving payload '{}' must succeed without SQL syntax error", payload));
        assert_eq!(saved.tag.as_deref(), Some(*payload));

        // B. Query single tag
        let retrieved = storage.get_saved_player(&target_puuid, self_puuid)
            .expect("get_saved_player")
            .expect("record exists");
        assert_eq!(retrieved.tag.as_deref(), Some(*payload));

        // C. Check tagged players hook
        let tags = storage.get_player_tags(&target_puuid, self_puuid).expect("get_player_tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag, *payload);
        assert!(tags[0].marked_by_self);
    }

    // Verify all tables still exist and are NOT dropped
    {
        let conn = storage.raw_connection();
        let conn_guard = conn.lock().unwrap();

        let check_table = |table_name: &str| {
            let exists: bool = conn_guard.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1;",
                [table_name],
                |r| r.get::<_, i32>(0).map(|c| c > 0)
            ).unwrap();
            assert!(exists, "Table '{}' MUST NOT have been dropped by SQL injection payloads!", table_name);
        };

        check_table("saved_player_tags");
        check_table("encountered_games");
        check_table("player_tag_phrases");
        check_table("app_settings");

        // PRAGMA quick_check to verify database integrity
        let integrity: String = conn_guard.query_row("PRAGMA quick_check;", [], |r| r.get(0)).unwrap();
        assert_eq!(integrity, "ok", "Database file integrity must remain 'ok'");
    }

    // Verify query_all_player_tags
    let all_tags = storage.query_all_player_tags(&QueryAllPlayerTagsDto::default()).unwrap();
    assert_eq!(all_tags.total as usize, test_vectors.len());

    // Test SQL injection in query_encountered_games time_order
    let enc_query = EncounteredGamesQueryDto {
        puuid: "p1".to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: Some("'; DROP TABLE encountered_games; --".to_string()),
        page: Some(1),
        page_size: Some(10),
        time_order: Some("'; DROP TABLE encountered_games; --".to_string()),
    };
    let enc_res = storage.query_encountered_games(&enc_query);
    assert!(enc_res.is_ok(), "query_encountered_games must resist SQL injection");

    // Verify table encountered_games is still intact
    {
        let conn = storage.raw_connection();
        let conn_guard = conn.lock().unwrap();
        let exists: bool = conn_guard.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='encountered_games';",
            [],
            |r| r.get::<_, i32>(0).map(|c| c > 0)
        ).unwrap();
        assert!(exists);
    }
}
