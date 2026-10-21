//! Adversarial Empirical Stress & Boundary Test Suite for Milestone 2:
//! SQLite IPC Pagination, Deletion, and Error Resilience
//!
//! Evaluates:
//! 1. Pagination boundary handling: page 0, page_size 0, page > total, page_size huge
//! 2. Ordering edge cases: "asc", "ASC", "desc", "invalid_order"
//! 3. Deletion semantics: existing vs non-existing IDs, idempotent second delete
//! 4. Self-peer collision filtering: record_encountered_game where self_puuid is inside peer list
//! 5. Tag whitespace filtering in query_all_player_tags: only non-empty tags are returned
//! 6. Negative & zero game_id boundaries

use lol_hex_rust::storage::models::*;
use lol_hex_rust::storage::StorageManager;

#[test]
fn test_adversarial_pagination_boundaries() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage failed");
    let self_puuid = "self-test-puuid-1";
    let target_puuid = "peer-test-puuid-1";

    // Insert 25 encounters
    for i in 1..=25 {
        storage
            .save_encountered_game(&EncounteredGameSaveDto {
                game_id: i,
                puuid: target_puuid.to_string(),
                self_puuid: self_puuid.to_string(),
                region: "TENCENT".to_string(),
                rso_platform_id: "HN1".to_string(),
                queue_type: Some("RANKED_SOLO_5x5".to_string()),
            })
            .expect("Failed to save encounter");
    }

    // 1. page = 0, page_size = 0 should be safely clamped to page 1, page_size 1
    let query_clamp = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(0),
        page_size: Some(0),
        time_order: None,
    };
    let res_clamp = storage.query_encountered_games(&query_clamp).expect("Clamped query failed");
    assert_eq!(res_clamp.page, 1, "Page 0 must be clamped to 1");
    assert_eq!(res_clamp.page_size, 1, "PageSize 0 must be clamped to 1");
    assert_eq!(res_clamp.data.len(), 1);
    assert_eq!(res_clamp.total, 25);
    assert_eq!(res_clamp.total_pages, 25);

    // 2. page beyond total_pages (e.g. page 999) should return empty data, not panic
    let query_overflow = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(999),
        page_size: Some(10),
        time_order: None,
    };
    let res_overflow = storage.query_encountered_games(&query_overflow).expect("Overflow query failed");
    assert_eq!(res_overflow.page, 999);
    assert_eq!(res_overflow.total, 25);
    assert_eq!(res_overflow.total_pages, 3);
    assert!(res_overflow.data.is_empty(), "Page beyond total should yield empty data");

    // 3. huge page_size (e.g. 10000)
    let query_huge = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(1),
        page_size: Some(10000),
        time_order: None,
    };
    let res_huge = storage.query_encountered_games(&query_huge).expect("Huge query failed");
    assert_eq!(res_huge.data.len(), 25);
    assert_eq!(res_huge.total_pages, 1);
}

#[test]
fn test_adversarial_ordering_and_queue_filter() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage failed");
    let self_puuid = "self-test-puuid-2";
    let target_puuid = "peer-test-puuid-2";

    // Insert 10 encounters across 2 different queues
    for i in 1..=10 {
        let q = if i <= 6 { "RANKED_SOLO_5x5" } else { "ARAM" };
        storage
            .save_encountered_game(&EncounteredGameSaveDto {
                game_id: 1000 + i,
                puuid: target_puuid.to_string(),
                self_puuid: self_puuid.to_string(),
                region: "TENCENT".to_string(),
                rso_platform_id: "HN1".to_string(),
                queue_type: Some(q.to_string()),
            })
            .expect("Failed to save encounter");
    }

    // Ensure distinct update_at timestamps for deterministic ASC/DESC sorting verification
    let conn = storage.raw_connection();
    let locked = conn.lock().unwrap();
    locked.execute_batch(
        "UPDATE encountered_games SET update_at = 100000 + (game_id - 1000) * 1000 WHERE self_puuid = 'self-test-puuid-2';"
    ).expect("Set timestamps failed");
    drop(locked);

    // 1. time_order = "ASC" vs "DESC"
    let query_asc = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(1),
        page_size: Some(10),
        time_order: Some("ASC".to_string()),
    };
    let res_asc = storage.query_encountered_games(&query_asc).expect("ASC query failed");
    assert_eq!(res_asc.data.len(), 10);
    assert_eq!(res_asc.data[0].game_id, 1001, "First game in ASC order should be 1001");
    assert_eq!(res_asc.data[9].game_id, 1010, "Last game in ASC order should be 1010");

    // 2. invalid time_order should safely default to DESC
    let query_invalid_order = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: None,
        page: Some(1),
        page_size: Some(10),
        time_order: Some("NON_EXISTENT_ORDER".to_string()),
    };
    let res_invalid = storage.query_encountered_games(&query_invalid_order).expect("Invalid order query failed");
    assert_eq!(res_invalid.data.len(), 10);
    assert_eq!(res_invalid.data[0].game_id, 1010);

    // 3. Filter by queue_type = "ARAM"
    let query_aram = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: Some("ARAM".to_string()),
        page: Some(1),
        page_size: Some(10),
        time_order: None,
    };
    let res_aram = storage.query_encountered_games(&query_aram).expect("ARAM query failed");
    assert_eq!(res_aram.total, 4);
    assert_eq!(res_aram.data.len(), 4);

    // 4. Filter by non-existent queue_type
    let query_empty_q = EncounteredGamesQueryDto {
        puuid: target_puuid.to_string(),
        self_puuid: self_puuid.to_string(),
        queue_type: Some("CHERRY_ARENA_FAKE".to_string()),
        page: Some(1),
        page_size: Some(10),
        time_order: None,
    };
    let res_empty = storage.query_encountered_games(&query_empty_q).expect("Empty q query failed");
    assert_eq!(res_empty.total, 0);
    assert_eq!(res_empty.total_pages, 0);
    assert!(res_empty.data.is_empty());
}

#[test]
fn test_adversarial_deletion_and_idempotency() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage failed");
    let self_puuid = "self-test-puuid-3";
    let target_puuid = "peer-test-puuid-3";

    let id = storage
        .save_encountered_game(&EncounteredGameSaveDto {
            game_id: 9999,
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            region: "TENCENT".to_string(),
            rso_platform_id: "HN1".to_string(),
            queue_type: Some("RANKED".to_string()),
        })
        .expect("Save failed");

    // Deleting existing encounter ID succeeds
    let del1 = storage.delete_encountered_game(id).expect("Delete 1 failed");
    assert!(del1, "First delete should return true");

    // Deleting same encounter ID second time returns false (not found), does not error
    let del2 = storage.delete_encountered_game(id).expect("Delete 2 failed");
    assert!(!del2, "Second delete should return false");

    // Deleting non-existent ID
    let del_fake = storage.delete_encountered_game(999_999_999).expect("Delete fake failed");
    assert!(!del_fake, "Non-existent delete should return false");

    // Tag deletion idempotency
    storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            region: None,
            rso_platform_id: None,
            tag: Some("绝活哥".to_string()),
        })
        .expect("Save tag failed");

    let del_tag1 = storage.delete_saved_player(target_puuid, self_puuid).expect("Delete tag failed");
    assert!(del_tag1, "First tag delete should return true");

    let del_tag2 = storage.delete_saved_player(target_puuid, self_puuid).expect("Delete tag 2 failed");
    assert!(!del_tag2, "Second tag delete should return false");
}

#[test]
fn test_adversarial_record_encountered_game_edge_cases() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage failed");
    let self_puuid = "self-hero";

    // 1. Peer list includes self_puuid and empty strings: self and empty must be stripped!
    let peers = vec![
        "self-hero".to_string(), // self should be filtered
        "".to_string(),          // empty should be filtered
        "peer-1".to_string(),
        "peer-2".to_string(),
    ];

    storage
        .record_encountered_game(5001, self_puuid, &peers, "TENCENT", "HN1", "NORMAL")
        .expect("Record encounter with self/empty peers failed");

    // Self should NOT have encounter row against itself
    let count_self = storage.count_encountered_games(self_puuid, self_puuid).expect("Count self failed");
    assert_eq!(count_self, 0, "Self player should never be recorded as encountered peer");

    // peer-1 and peer-2 must have 1 encounter each
    let count_p1 = storage.count_encountered_games(self_puuid, "peer-1").expect("Count p1 failed");
    assert_eq!(count_p1, 1);
    let count_p2 = storage.count_encountered_games(self_puuid, "peer-2").expect("Count p2 failed");
    assert_eq!(count_p2, 1);

    // 2. Negative or 0 game_id should be ignored as no-op
    storage
        .record_encountered_game(0, self_puuid, &peers, "TENCENT", "HN1", "NORMAL")
        .expect("0 game_id should no-op");
    storage
        .record_encountered_game(-100, self_puuid, &peers, "TENCENT", "HN1", "NORMAL")
        .expect("Negative game_id should no-op");

    // Counts remain 1
    assert_eq!(storage.count_encountered_games(self_puuid, "peer-1").unwrap(), 1);
}

#[test]
fn test_adversarial_query_all_player_tags_filters_empty_or_whitespace() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage failed");
    let self_puuid = "self-admin";

    // Insert tag with content
    storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: "valid-player-1".to_string(),
            self_puuid: self_puuid.to_string(),
            region: None,
            rso_platform_id: None,
            tag: Some("通天代".to_string()),
        })
        .expect("Save tag 1 failed");

    // Insert tag with whitespace only
    storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: "whitespace-player".to_string(),
            self_puuid: self_puuid.to_string(),
            region: None,
            rso_platform_id: None,
            tag: Some("   ".to_string()),
        })
        .expect("Save tag 2 failed");

    // Insert tag with None
    storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: "none-player".to_string(),
            self_puuid: self_puuid.to_string(),
            region: None,
            rso_platform_id: None,
            tag: None,
        })
        .expect("Save tag 3 failed");

    let tags_res = storage
        .query_all_player_tags(&QueryAllPlayerTagsDto {
            puuid: None,
            self_puuid: None,
            region: None,
            rso_platform_id: None,
            page: Some(1),
            page_size: Some(20),
            time_order: None,
        })
        .expect("Query all tags failed");

    // Whitespace-only or None tags must NOT appear in query_all_player_tags
    assert_eq!(tags_res.total, 1);
    assert_eq!(tags_res.data.len(), 1);
    assert_eq!(tags_res.data[0].puuid, "valid-player-1");
    assert_eq!(tags_res.data[0].tag.as_deref(), Some("通天代"));
}
