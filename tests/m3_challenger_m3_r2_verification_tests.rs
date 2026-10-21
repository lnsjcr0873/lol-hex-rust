//! Milestone 3 Iteration 2 Challenger Verification Test Suite
//!
//! Author: challenger_m3_r2_1
//! Purpose: Adversarial empirical verification of:
//! 1. Snapshot 180s TTL boundary (179s vs 180.001s vs 181s) and persistent storage eviction
//! 2. 0-game and multi-role jungle preset calculations (strict anti-fabrication check)
//! 3. Dynamic metrics calculation accuracy across both jungle and detailed presets
//! 4. GSM enrichment and background snapshot pipeline edge cases

use lol_hex_rust::engine::player_history::{
    generate_tactical_broadcast, member_merge, ChampSelectHandoffSnapshot,
    OngoingPlayerCard, ParsedMatchRecord, PlayerHistoryManager,
    CHAMP_SELECT_SNAPSHOT_TTL_MS,
};
use lol_hex_rust::storage::StorageManager;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// 1. Snapshot TTL Boundary & Storage Eviction Tests
// ============================================================================

#[tokio::test]
async fn test_snapshot_ttl_strict_boundary_179s_vs_181s() {
    assert_eq!(
        CHAMP_SELECT_SNAPSHOT_TTL_MS,
        180 * 1000,
        "Snapshot TTL constant must be exactly 180,000 ms (180s)"
    );

    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open in-memory storage"));
    engine.set_storage(storage.clone()).await;

    let now_ms = current_time_ms();

    // Boundary 1: Snapshot created 179 seconds ago (179,000 ms < 180,000 ms) -> MUST BE VALID
    let snap_179s = ChampSelectHandoffSnapshot {
        game_id: 3001,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-179s".to_string(),
        created_at_ms: now_ms.saturating_sub(179 * 1000),
    };
    engine.save_champ_select_snapshot(snap_179s, Some(&storage)).await;

    let res_179s = engine.get_valid_champ_select_snapshot(Some(3001), Some(&storage)).await;
    assert!(
        res_179s.is_some(),
        "Snapshot created 179s ago must be valid (< 180s)"
    );
    assert_eq!(res_179s.unwrap().chat_room_id, "chat-179s");

    // Boundary 2: Snapshot created 180 seconds + 500 ms ago (> 180,000 ms) -> MUST BE EXPIRED
    let snap_180_5s = ChampSelectHandoffSnapshot {
        game_id: 3002,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-180-5s".to_string(),
        created_at_ms: now_ms.saturating_sub(180_500),
    };
    engine.save_champ_select_snapshot(snap_180_5s, Some(&storage)).await;

    let res_180_5s = engine.get_valid_champ_select_snapshot(Some(3002), Some(&storage)).await;
    assert!(
        res_180_5s.is_none(),
        "Snapshot created 180.5s ago must be expired (> 180s TTL)"
    );

    // Boundary 3: Verify storage was evicted upon expiry
    let persisted_setting = storage.get_setting("champ_select_handoff_snapshot").unwrap();
    assert!(
        persisted_setting.is_none() || persisted_setting.as_deref() == Some(""),
        "Expired snapshot must be evicted from persistent storage"
    );

    // Boundary 4: Snapshot created 181 seconds ago -> MUST BE EXPIRED
    let snap_181s = ChampSelectHandoffSnapshot {
        game_id: 3003,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-181s".to_string(),
        created_at_ms: now_ms.saturating_sub(181 * 1000),
    };
    engine.save_champ_select_snapshot(snap_181s, Some(&storage)).await;

    let res_181s = engine.get_valid_champ_select_snapshot(Some(3003), Some(&storage)).await;
    assert!(
        res_181s.is_none(),
        "Snapshot created 181s ago must be expired (> 180s)"
    );
}

#[tokio::test]
async fn test_snapshot_future_timestamp_safety() {
    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open storage"));
    engine.set_storage(storage.clone()).await;

    let now_ms = current_time_ms();
    // Clock skew / NTP drift: created_at_ms slightly in future
    let snap_future = ChampSelectHandoffSnapshot {
        game_id: 3004,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-future".to_string(),
        created_at_ms: now_ms + 5000,
    };
    engine.save_champ_select_snapshot(snap_future, Some(&storage)).await;

    // Must not panic on saturating_sub
    let res = engine.get_valid_champ_select_snapshot(Some(3004), Some(&storage)).await;
    assert!(res.is_some(), "Snapshot with slight future clock skew must not underflow/panic");
}

// ============================================================================
// 2. Tactical Broadcast Jungle Preset Anti-Fabrication & Role Hierarchy Tests
// ============================================================================

#[test]
fn test_jungle_preset_0_matches_0_history_never_fabricates_16_games() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 1;
    card.position = "JUNGLE".to_string();
    card.summoner_name = "RealNovice".to_string();
    card.champion_id = 64;
    card.champion_name = "盲僧".to_string();
    card.spell1_id = 11; // Smite
    card.tactical_analysis.recent_count = 0;
    card.tactical_analysis.cur_champ_games = 0;
    card.matches = vec![];

    let lines = generate_tactical_broadcast("jungle", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    assert!(!line.contains("16场"), "NEVER fabricate 16 games: {}", line);
    assert!(!line.contains("样本16"), "NEVER fabricate 16 samples: {}", line);
    assert_eq!(line, "盲僧：近期没有对局记录");
}

#[test]
fn test_jungle_preset_other_role_only_no_jungle_games() {
    // Player has 10 matches on Ahri in MID lane, 0 games on Lee Sin and 0 jungle games
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 1;
    card.position = "JUNGLE".to_string();
    card.summoner_name = "AutofilledMidToJungle".to_string();
    card.champion_id = 64; // Locked Lee Sin
    card.champion_name = "盲僧".to_string();
    card.spell1_id = 11;
    card.tactical_analysis.recent_count = 10;
    card.tactical_analysis.cur_champ_games = 0;

    for _ in 0..10 {
        let mut m = ParsedMatchRecord::default();
        m.champion_id = 103; // Ahri
        m.lane = "MIDDLE".to_string();
        m.role = "SOLO".to_string();
        m.spell1_id = 4;
        m.spell2_id = 14;
        card.matches.push(m);
    }

    let lines = generate_tactical_broadcast("jungle", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    assert!(!line.contains("16"), "Must not fabricate 16 games: {}", line);
    assert_eq!(
        line,
        "盲僧：没有本英雄打野明细记录",
        "Must correctly indicate zero champion jungle detail without fabrication: {}",
        line
    );
}

#[test]
fn test_jungle_preset_fallback_to_other_jungle_champions_when_no_champ_games() {
    // Player locked Lee Sin (champion_id = 64), 0 games on Lee Sin,
    // but has 3 matches on Jarvan IV (champion_id = 59) in JUNGLE lane.
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 1;
    card.position = "JUNGLE".to_string();
    card.summoner_name = "JarvanMainOnLeeSin".to_string();
    card.champion_id = 64; // Lee Sin
    card.champion_name = "盲僧".to_string();
    card.spell1_id = 11;
    card.tactical_analysis.recent_count = 3;
    card.tactical_analysis.cur_champ_games = 0;

    for i in 0..3 {
        let mut m = ParsedMatchRecord::default();
        m.champion_id = 59; // Jarvan IV
        m.lane = "JUNGLE".to_string();
        m.role = "JUNGLE".to_string();
        m.spell1_id = 11;
        m.spell2_id = 4;
        m.win = i < 2; // 2 wins, 1 loss = 67% WR
        m.kills = 4;
        m.deaths = 2;
        m.assists = 8;
        m.kp_pct = 60.0;
        m.vision_score = 30;
        m.dragon_kills = 2;
        m.baron_kills = 1;
        card.matches.push(m);
    }

    let lines = generate_tactical_broadcast("jungle", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    // Must output generic jungle stats with sample count = 3 (NOT 16, NOT "本英雄")
    assert!(line.contains("盲僧：打野样本3场"), "Actual: {}", line);
    assert!(line.contains("胜率67%"), "Actual: {}", line);
    assert!(line.contains("KDA6.00"), "Actual: {}", line);
    assert!(line.contains("场均参团率60%"), "Actual: {}", line);
    assert!(line.contains("场均视野得分30.0"), "Actual: {}", line);
    assert!(line.contains("场均小龙2.0"), "Actual: {}", line);
    assert!(line.contains("场均大龙1.0"), "Actual: {}", line);
}

// ============================================================================
// 3. Dynamic Metrics Accuracy Verification
// ============================================================================

#[test]
fn test_dynamic_metrics_detailed_preset_mathematical_precision() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "PrecisionADC".to_string();
    card.champion_id = 222;
    card.champion_name = "金克丝".to_string();
    card.tactical_analysis.recent_count = 2;
    card.tactical_analysis.recent_win_rate_pct = 100;
    card.tactical_analysis.avg_kda_ratio = 7.5;

    // Match 1:
    let mut m1 = ParsedMatchRecord::default();
    m1.damage_share_pct = 30.0;
    m1.damage_taken_share_pct = 15.0;
    m1.gold_share_pct = 25.0;
    m1.cs_per_min = 9.0;
    m1.kp_pct = 80.0;
    m1.vision_score = 20;
    m1.solo_kills = 2;
    card.matches.push(m1);

    // Match 2:
    let mut m2 = ParsedMatchRecord::default();
    m2.damage_share_pct = 40.0;
    m2.damage_taken_share_pct = 25.0;
    m2.gold_share_pct = 35.0;
    m2.cs_per_min = 10.0;
    m2.kp_pct = 60.0;
    m2.vision_score = 40;
    m2.solo_kills = 4;
    card.matches.push(m2);

    // Expected averages:
    // dmg = (30 + 40)/2 = 35%
    // taken = (15 + 25)/2 = 20%
    // gold = (25 + 35)/2 = 30%
    // cs = (9.0 + 10.0)/2 = 9.5
    // kp = (80 + 60)/2 = 70%
    // vis = (20 + 40)/2 = 30.0
    // solo = (2 + 4)/2 = 3.0
    // efficiency = 35 / 30 * 100 = 116.666... -> 117%

    let lines = generate_tactical_broadcast("detailed", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    assert!(line.contains("金克丝：2场对局，胜率100% KDA7.50"), "Actual: {}", line);
    assert!(line.contains("场均单杀3.0"), "Actual: {}", line);
    assert!(line.contains("场均视野得分30.0"), "Actual: {}", line);
    assert!(line.contains("伤害35%"), "Actual: {}", line);
    assert!(line.contains("承伤20%"), "Actual: {}", line);
    assert!(line.contains("经济30%"), "Actual: {}", line);
    assert!(line.contains("补兵9.5/分"), "Actual: {}", line);
    assert!(line.contains("参团率70%"), "Actual: {}", line);
    assert!(line.contains("伤转比117%"), "Actual: {}", line);
}

#[test]
fn test_dynamic_metrics_zero_death_kda_calculation_safety() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 1;
    card.position = "JUNGLE".to_string();
    card.champion_id = 64;
    card.champion_name = "盲僧".to_string();
    card.spell1_id = 11;

    let mut m = ParsedMatchRecord::default();
    m.champion_id = 64;
    m.lane = "JUNGLE".to_string();
    m.win = true;
    m.kills = 10;
    m.deaths = 0; // ZERO DEATHS -> must use max(1.0)
    m.assists = 5;
    m.kp_pct = 75.0;
    m.vision_score = 25;
    m.dragon_kills = 4;
    card.matches.push(m);

    let lines = generate_tactical_broadcast("jungle", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    // KDA should be (10 + 5) / 1.0 = 15.00
    assert!(line.contains("KDA15.00"), "Actual: {}", line);
    assert!(line.contains("场均小龙4.0"), "Actual: {}", line);
}

// ============================================================================
// 4. Pipeline & LCS Snapshot Alignment Integration
// ============================================================================

#[test]
fn test_member_merge_preserves_unique_order() {
    let origin = vec!["puuid_1".to_string(), "puuid_2".to_string()];
    let extra = vec!["puuid_2".to_string(), "puuid_3".to_string(), "puuid_4".to_string()];

    let merged = member_merge(&origin, &extra);
    assert_eq!(merged.len(), 4);
    assert_eq!(merged, vec!["puuid_1", "puuid_2", "puuid_3", "puuid_4"]);
}
