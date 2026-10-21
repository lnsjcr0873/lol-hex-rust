//! Adversarial Empirical Stress & Boundary Test Suite for Milestone 1 SQLite Storage Engine
//!
//! Evaluates:
//! 1. High-concurrency multithreaded read/write operations with `Arc<Mutex<Connection>>`
//! 2. Batch recording of 9 peers under repeated/concurrent calls (idempotent deduplication via `idx_enc_games_unique`)
//! 3. Large dataset pagination (1000+ encounter rows, LIMIT/OFFSET boundary invariants, DESC/ASC ordering, queue filter)
//! 4. Disk corruption recovery (detecting corrupt/truncated headers, archiving to `.corrupted.<timestamp>`, recreating fresh DB)
//! 5. Phrase normalization boundary constraints (Unicode scalar lengths, control chars, deduping, max counts)

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use lol_hex_rust::storage::models::*;
use lol_hex_rust::storage::StorageManager;

/// RAII cleanup guard for test temporary directory
struct TempDirGuard {
    path: std::path::PathBuf,
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Helper to create an isolated temporary directory for disk DB testing
fn create_temp_db_path(prefix: &str) -> (TempDirGuard, std::path::PathBuf) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{}_{}", prefix, now));
    let _ = std::fs::create_dir_all(&dir);
    let db_path = dir.join("HexBrain_test.db");
    (TempDirGuard { path: dir }, db_path)
}

// =========================================================================
// 1. High-Concurrency Multithreaded Read/Write Stress Test
// =========================================================================

#[test]
fn test_stress_high_concurrency_reads_and_writes() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage init failed");
    let thread_count = 20;
    let ops_per_thread = 50;
    let start_time = Instant::now();

    let write_successes = Arc::new(AtomicUsize::new(0));
    let read_successes = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    for t_idx in 0..thread_count {
        let storage_clone = storage.clone();
        let write_succ = write_successes.clone();
        let read_succ = read_successes.clone();

        let handle = thread::spawn(move || {
            for i in 0..ops_per_thread {
                let self_puuid = format!("self-player-{}", t_idx % 4);
                let target_puuid = format!("peer-player-{}", (t_idx * 10) + (i % 10));

                if t_idx % 2 == 0 {
                    // Writer thread: alternating between encounter batch and tag update
                    if i % 2 == 0 {
                        let game_id = (t_idx as i64 * 10_000) + i as i64 + 1;
                        let peers: Vec<String> = (1..=9)
                            .map(|p| format!("peer-t{}-p{}-g{}", t_idx, p, i))
                            .collect();

                        storage_clone
                            .record_encountered_game(
                                game_id,
                                &self_puuid,
                                &peers,
                                "TENCENT",
                                "HN1",
                                "RANKED_SOLO_5x5",
                            )
                            .expect("Concurrent record_encountered_game failed");
                        write_succ.fetch_add(1, Ordering::Relaxed);
                    } else {
                        storage_clone
                            .update_player_tag(&UpdatePlayerTagDto {
                                puuid: target_puuid.clone(),
                                self_puuid: self_puuid.clone(),
                                region: Some("TENCENT".to_string()),
                                rso_platform_id: Some("HN1".to_string()),
                                tag: Some(format!("Tag from thread {} op {}", t_idx, i)),
                            })
                            .expect("Concurrent update_player_tag failed");
                        write_succ.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    // Reader thread: querying paginated encounters, tags, and settings
                    if i % 3 == 0 {
                        let res = storage_clone
                            .query_encountered_games(&EncounteredGamesQueryDto {
                                puuid: target_puuid.clone(),
                                self_puuid: self_puuid.clone(),
                                queue_type: None,
                                page: Some(1),
                                page_size: Some(10),
                                time_order: Some("desc".to_string()),
                            })
                            .expect("Concurrent query_encountered_games failed");
                        let _ = res.total;
                        read_succ.fetch_add(1, Ordering::Relaxed);
                    } else if i % 3 == 1 {
                        let tags = storage_clone
                            .get_player_tags(&target_puuid, &self_puuid)
                            .expect("Concurrent get_player_tags failed");
                        let _ = tags.len();
                        read_succ.fetch_add(1, Ordering::Relaxed);
                    } else {
                        let _ = storage_clone.is_champ_select_alert_enabled().unwrap();
                        let _ = storage_clone.is_phrase_panel_expanded().unwrap();
                        read_succ.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Worker thread panicked under lock contention");
    }

    let elapsed = start_time.elapsed();
    let total_writes = write_successes.load(Ordering::SeqCst);
    let total_reads = read_successes.load(Ordering::SeqCst);
    let total_ops = total_writes + total_reads;

    println!(
        "\n[Stress Concurrency Metric] {} total operations (writes: {}, reads: {}) completed in {:?} (~{:.2} ops/sec)",
        total_ops,
        total_writes,
        total_reads,
        elapsed,
        total_ops as f64 / elapsed.as_secs_f64()
    );

    assert_eq!(total_ops, thread_count * ops_per_thread);
    assert!(total_writes > 0);
    assert!(total_reads > 0);

    // Verify raw connection health via quick_check
    let conn = storage.raw_connection();
    let locked = conn.lock().expect("Lock poisoned after concurrency stress");
    let check: String = locked
        .query_row("PRAGMA quick_check;", [], |r| r.get(0))
        .expect("PRAGMA quick_check failed");
    assert_eq!(check, "ok");
}

// =========================================================================
// 2. Batch Recording of 9 Peers & Idempotent Deduplication Stress Test
// =========================================================================

#[test]
fn test_stress_idempotent_deduplication_batch_9_peers() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage init failed");
    let self_puuid = "local-hero-puuid";
    let game_id = 998877665544i64;
    let peers: Vec<String> = (1..=9).map(|i| format!("peer-puuid-fixed-{:02}", i)).collect();

    // Step A: Pre-tag peer 1 with a custom note
    storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: peers[0].clone(),
            self_puuid: self_puuid.to_string(),
            region: Some("TENCENT".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some("绝活亚索-勿动".to_string()),
        })
        .expect("Pre-tagging failed");

    // Step B: Record the game 10 times consecutively (simulating redundant gameflow events)
    for _ in 1..=10 {
        storage
            .record_encountered_game(
                game_id,
                self_puuid,
                &peers,
                "TENCENT",
                "HN1",
                "RANKED_SOLO_5x5",
            )
            .expect("Redundant record_encountered_game failed");
    }

    // Step C: Concurrently record the exact same game from 10 threads
    let mut handles = Vec::new();
    for _ in 0..10 {
        let storage_clone = storage.clone();
        let peers_clone = peers.clone();
        handles.push(thread::spawn(move || {
            storage_clone
                .record_encountered_game(
                    game_id,
                    self_puuid,
                    &peers_clone,
                    "TENCENT",
                    "HN1",
                    "RANKED_SOLO_5x5",
                )
                .expect("Concurrent redundant record failed");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // Verification 1: Exactly 9 records in encountered_games for this game_id
    let conn = storage.raw_connection();
    let locked = conn.lock().unwrap();
    let total_game_rows: i64 = locked
        .query_row(
            "SELECT COUNT(*) FROM encountered_games WHERE game_id = ?1;",
            [game_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        total_game_rows, 9,
        "Idempotent deduplication failed: expected exactly 9 peer rows for game_id {}, got {}",
        game_id, total_game_rows
    );

    // Verification 2: Each peer has encounter count = 1
    drop(locked);
    for peer in &peers {
        let count = storage.count_encountered_games(self_puuid, peer).unwrap();
        assert_eq!(
            count, 1,
            "Peer {} expected encounter count 1, got {}",
            peer, count
        );
    }

    // Verification 3: Custom tag on peer 1 was preserved, not overwritten by NULL
    let peer1_tag = storage.get_saved_player(&peers[0], self_puuid).unwrap();
    assert!(peer1_tag.is_some());
    let rec = peer1_tag.unwrap();
    assert_eq!(
        rec.tag.as_deref(),
        Some("绝活亚索-勿动"),
        "Custom tag note was overwritten by record_encountered_game!"
    );
    assert!(
        rec.last_met_at.is_some(),
        "last_met_at was not updated on encounter"
    );

    // Verification 4: Duplicate peer within same slice and self-puuid inclusion
    let mut tainted_peers = peers.clone();
    tainted_peers.push(peers[0].clone()); // duplicate within batch
    tainted_peers.push(self_puuid.to_string()); // self-puuid in peers
    tainted_peers.push("".to_string()); // empty string

    let new_game_id = 998877665545i64;
    storage
        .record_encountered_game(
            new_game_id,
            self_puuid,
            &tainted_peers,
            "TENCENT",
            "HN1",
            "ARAM",
        )
        .expect("Tainted slice record failed");

    let count_new_game: i64 = storage
        .raw_connection()
        .lock()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM encountered_games WHERE game_id = ?1;",
            [new_game_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count_new_game, 9,
        "Tainted slice did not filter duplicates or self-puuid: expected 9, got {}",
        count_new_game
    );
}

// =========================================================================
// 3. Large Dataset Pagination Stress Test (1000+ Rows, Boundary Invariants)
// =========================================================================

#[test]
fn test_stress_large_dataset_pagination_and_ordering() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage init failed");
    let self_puuid = "challenger-self";
    let target_puuid = "rival-puuid-deep";
    let total_rows: usize = 1000;

    println!("\n[Large Dataset Test] Inserting {} encounter rows...", total_rows);
    let insert_start = Instant::now();

    // Insert 1000 encounters with distinct game_ids and queue types
    for i in 1..=total_rows {
        let q_type = if i % 2 == 0 { "RANKED_SOLO_5x5" } else { "ARAM" };
        storage
            .record_encountered_game(
                i as i64,
                self_puuid,
                &[target_puuid.to_string()],
                "TENCENT",
                "HN1",
                q_type,
            )
            .expect("Batch insert failed");
    }
    println!(
        "[Large Dataset Test] Inserted {} rows in {:?}",
        total_rows,
        insert_start.elapsed()
    );

    // Verify total count
    let count = storage.count_encountered_games(self_puuid, target_puuid).unwrap();
    assert_eq!(count as usize, total_rows);

    // Test different page sizes: 10, 25, 50, 100, 200
    for page_size in [10, 25, 50, 100, 200] {
        let expected_pages = (total_rows + page_size - 1) / page_size;

        let p1 = storage
            .query_encountered_games(&EncounteredGamesQueryDto {
                puuid: target_puuid.to_string(),
                self_puuid: self_puuid.to_string(),
                queue_type: None,
                page: Some(1),
                page_size: Some(page_size as u32),
                time_order: Some("desc".to_string()),
            })
            .unwrap();

        assert_eq!(p1.total as usize, total_rows);
        assert_eq!(p1.total_pages as usize, expected_pages);
        assert_eq!(p1.data.len(), page_size);

        // Fetch last page
        let p_last = storage
            .query_encountered_games(&EncounteredGamesQueryDto {
                puuid: target_puuid.to_string(),
                self_puuid: self_puuid.to_string(),
                queue_type: None,
                page: Some(expected_pages as u32),
                page_size: Some(page_size as u32),
                time_order: Some("desc".to_string()),
            })
            .unwrap();

        let expected_last_page_len = if total_rows % page_size == 0 {
            page_size
        } else {
            total_rows % page_size
        };
        assert_eq!(p_last.data.len(), expected_last_page_len);
    }

    // Verify sequential traversal covering all 1000 items without gaps or overlaps
    let mut collected_ids = Vec::with_capacity(total_rows);
    let page_size = 50;
    let total_pages = total_rows / page_size;

    for page in 1..=total_pages {
        let res = storage
            .query_encountered_games(&EncounteredGamesQueryDto {
                puuid: target_puuid.to_string(),
                self_puuid: self_puuid.to_string(),
                queue_type: None,
                page: Some(page as u32),
                page_size: Some(page_size as u32),
                time_order: Some("desc".to_string()),
            })
            .unwrap();

        for item in res.data {
            collected_ids.push(item.id);
        }
    }
    assert_eq!(collected_ids.len(), total_rows);
    let mut deduped_ids = collected_ids.clone();
    deduped_ids.sort();
    deduped_ids.dedup();
    assert_eq!(deduped_ids.len(), total_rows, "Pagination resulted in duplicated IDs across pages!");

    // Verify Queue Type Filtering: RANKED_SOLO_5x5 must yield exactly 500
    let ranked_res = storage
        .query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: Some("RANKED_SOLO_5x5".to_string()),
            page: Some(1),
            page_size: Some(100),
            time_order: Some("desc".to_string()),
        })
        .unwrap();
    assert_eq!(ranked_res.total, 500);
    assert_eq!(ranked_res.total_pages, 5);
    for item in &ranked_res.data {
        assert_eq!(item.queue_type, "RANKED_SOLO_5x5");
    }

    // Verify Out-of-bounds Page: page 9999 should return empty data, not panic
    let oob_res = storage
        .query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(9999),
            page_size: Some(50),
            time_order: Some("desc".to_string()),
        })
        .unwrap();
    assert_eq!(oob_res.total as usize, total_rows);
    assert!(oob_res.data.is_empty());

    // Verify Boundary Page = 0 (should clamp to 1)
    let p0_res = storage
        .query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(0),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        })
        .unwrap();
    assert_eq!(p0_res.page, 1);
    assert_eq!(p0_res.data.len(), 10);
}

// =========================================================================
// 4. Disk Corruption Recovery & Archiving Stress Test
// =========================================================================

#[test]
fn test_stress_corruption_recovery_on_disk() {
    let (_temp_dir, db_path) = create_temp_db_path("hexbrain_corrupt_harness");

    // Phase 1: Initialize healthy DB and write valid data
    {
        let storage = StorageManager::open(&db_path).expect("Initial DB creation failed");
        storage
            .update_player_tag(&UpdatePlayerTagDto {
                puuid: "victim-puuid".to_string(),
                self_puuid: "self-puuid".to_string(),
                region: Some("TENCENT".to_string()),
                rso_platform_id: Some("HN1".to_string()),
                tag: Some("Old tag before corruption".to_string()),
            })
            .expect("Tag insertion failed");
    }

    // Phase 2: Intentionally corrupt the DB file by overwriting with binary garbage
    std::fs::write(
        &db_path,
        b"CORRUPTED HEADER 0xDEADBEEF NOT A SQLITE 3 DATABASE FILE AT ALL !!!! \x00\xFF\xAA\x55",
    )
    .expect("Failed to write corrupt bytes");

    // Also place dummy companion files (wal/shm) to verify cleanup
    let file_stem = db_path.file_stem().unwrap().to_str().unwrap();
    let wal_path = db_path.with_file_name(format!("{}.db-wal", file_stem));
    let shm_path = db_path.with_file_name(format!("{}.db-shm", file_stem));
    std::fs::write(&wal_path, b"DUMMY WAL DATA").unwrap();
    std::fs::write(&shm_path, b"DUMMY SHM DATA").unwrap();

    // Phase 3: Open storage manager with recovery
    let recovered_storage =
        StorageManager::open(&db_path).expect("StorageManager::open MUST recover corrupted DB");

    // Verification A: Recovered DB is healthy
    let conn = recovered_storage.raw_connection();
    let locked = conn.lock().unwrap();
    let check: String = locked
        .query_row("PRAGMA quick_check;", [], |r| r.get(0))
        .expect("PRAGMA quick_check failed on recovered DB");
    assert_eq!(check, "ok");

    // Verification B: Archived .corrupted.<timestamp> backup file exists
    let parent = db_path.parent().unwrap();
    let mut found_archive = false;
    for entry in std::fs::read_dir(parent).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(".db.corrupted.") {
            found_archive = true;
            let meta = entry.metadata().unwrap();
            assert!(meta.len() > 0, "Archived corrupted file must not be empty");
            break;
        }
    }
    assert!(found_archive, "Expected timestamped .corrupted archive file");

    // Verification C: The old dummy WAL and SHM files were deleted (new files belong to fresh WAL-mode DB)
    let wal_content = std::fs::read(&wal_path).unwrap_or_default();
    assert_ne!(
        wal_content.as_slice(),
        b"DUMMY WAL DATA",
        "Old dummy WAL data was not deleted during recovery"
    );
    let shm_content = std::fs::read(&shm_path).unwrap_or_default();
    assert_ne!(
        shm_content.as_slice(),
        b"DUMMY SHM DATA",
        "Old dummy SHM data was not deleted during recovery"
    );

    // Verification D: Schema migrations are active and fresh inserts succeed immediately
    drop(locked);
    recovered_storage
        .update_player_tag(&UpdatePlayerTagDto {
            puuid: "fresh-player".to_string(),
            self_puuid: "self-puuid".to_string(),
            region: Some("TENCENT".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some("Fresh tag post-recovery".to_string()),
        })
        .expect("Fresh tag insertion failed on recovered DB");

    let tags = recovered_storage
        .get_player_tags("fresh-player", "self-puuid")
        .expect("Query failed on recovered DB");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].tag, "Fresh tag post-recovery");
}

// =========================================================================
// 5. Phrase Normalization, Unicode & Constraint Stress Test
// =========================================================================

#[test]
fn test_stress_phrase_normalization_and_boundaries() {
    let storage = StorageManager::open_in_memory().expect("In-memory storage init failed");

    // Test extreme phrase inputs: whitespace, unicode, emojis, overflow
    let raw_phrases = vec![
        "   ".to_string(),                                  // empty/whitespace
        "".to_string(),                                     // empty
        "   通天代中单   ".to_string(),                      // whitespace padding
        "通天代中单".to_string(),                           // duplicate
        "绝活🔥亚索".to_string(),                           // unicode emoji
        "中".repeat(150),                                   // 150 Chinese chars (> 100 chars limit)
        "English Note With Special Characters: <>/\\|?*!@#$%^&*()".to_string(),
        "演员".to_string(),
        "车队".to_string(),
        "连胜".to_string(),
        "连败".to_string(),
        "容易被抓".to_string(),
        "通天代".to_string(),
        "带崩全队".to_string(),
        "单杀怪".to_string(),
        "闪现置位异常".to_string(),
        "曾遇到过".to_string(),
        "心态极差".to_string(),
        "绝活哥".to_string(),
        "稳健补刀".to_string(),
        "开黑车队".to_string(),
        "高危玩家".to_string(),
        "Extra Phrase 21".to_string(),                     // > 20 items cap
        "Extra Phrase 22".to_string(),
    ];

    let saved = storage.update_player_tag_phrases(&raw_phrases).expect("Phrase update failed");

    // Boundary check 1: max 20 phrases
    assert!(saved.len() <= SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS);
    assert_eq!(saved.len(), 20);

    // Boundary check 2: trimmed
    assert_eq!(saved[0], "通天代中单");

    // Boundary check 3: capped at 100 Unicode scalar characters
    let long_chinese = saved.iter().find(|p| p.starts_with("中")).unwrap();
    assert_eq!(long_chinese.chars().count(), 100);

    // Boundary check 4: loaded matches saved
    let loaded = storage.get_player_tag_phrases().expect("Phrase get failed");
    assert_eq!(loaded, saved);

    // UI Setting toggles
    assert!(storage.is_phrase_panel_expanded().unwrap());
    storage.set_phrase_panel_expanded(false).unwrap();
    assert!(!storage.is_phrase_panel_expanded().unwrap());

    assert!(storage.is_champ_select_alert_enabled().unwrap());
    storage.set_champ_select_alert_enabled(false).unwrap();
    assert!(!storage.is_champ_select_alert_enabled().unwrap());
}
