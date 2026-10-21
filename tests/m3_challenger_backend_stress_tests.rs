//! Empirical Adversarial Stress Test Suite for Milestone 3 (Backend Logic & Edge Cases)
//!
//! Validates:
//! 1. Champ-Select Handoff Snapshot:
//!    - Snapshot expiry (180s timeout window vs actual implementation).
//!    - Snapshot deserialization when JSON is corrupted or key is absent.
//!    - Zero-blackout fallback behavior during simulated gameflow transition from ChampSelect to InProgress.
//! 2. SGP GSM Live Game Parsing:
//!    - `parse_gsm_selected_role`: Diverse role string variants (MIDDLE.PRIMARY, TOP.AUTOFILL, lowercase, malformed, nulls).
//!    - Missing fields, empty teams, invalid PUUIDs in GSM response.
//! 3. Tactical Broadcast Presets:
//!    - 0 games / empty history records across all 4 presets ("concise", "detailed", "jungle", "premade").
//!    - Missing metrics (None/null/zeros).
//!    - Single-player vs multi-player premade parties.
//!    - Unselected champions (champion_id == 0).

use lol_hex_rust::engine::player_history::{
    generate_tactical_broadcast, parse_gsm_selected_role,
    ChampSelectHandoffSnapshot, OngoingPlayerCard,
    PlayerHistoryManager,
};
use lol_hex_rust::sgp::gsm::{GsmLedgeRegionResponse, GsmTeamMember};
use lol_hex_rust::storage::StorageManager;
use serde_json::json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// Group 1: Champ-Select Handoff Snapshot Expiry & Storage Corruption Tests
// ============================================================================

#[tokio::test]
async fn test_snapshot_expiry_180s_boundary_check() {
    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open test storage"));
    engine.set_storage(storage.clone()).await;

    let now_ms = current_time_ms();

    // 1. Snapshot created 60s ago (< 180s window) -> MUST be valid
    let valid_snap = ChampSelectHandoffSnapshot {
        game_id: 1001,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-test-valid".to_string(),
        created_at_ms: now_ms.saturating_sub(60 * 1000),
    };
    engine.save_champ_select_snapshot(valid_snap, None).await;

    let res = engine.get_valid_champ_select_snapshot(Some(1001), None).await;
    assert!(res.is_some(), "Snapshot created 60s ago must be valid");
    assert_eq!(res.unwrap().chat_room_id, "chat-test-valid");

    // 2. Snapshot created 300s ago (5 minutes ago > 180s window)
    // NOTE (EMPIRICAL AUDIT FINDING): The specification (§R3) and worker handoff claim
    // a 180-second timeout window. However, src/engine/player_history.rs:1200 checks:
    // `now_ms.saturating_sub(snap.created_at_ms) > 2 * 3600 * 1000` (7200 seconds / 2 hours).
    let expired_snap = ChampSelectHandoffSnapshot {
        game_id: 1002,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-test-expired".to_string(),
        created_at_ms: now_ms.saturating_sub(300 * 1000), // 300s ago
    };
    engine.save_champ_select_snapshot(expired_snap, None).await;

    let res_expired = engine.get_valid_champ_select_snapshot(Some(1002), None).await;
    // We document the empirical behavior:
    // Under strict 180s expiry, res_expired must be None.
    let is_expired_under_180s = res_expired.is_none();
    println!(
        "[EMPIRICAL OBSERVATION] 300s-old snapshot returned is_none: {} (Actual snapshot returned: {:?})",
        is_expired_under_180s,
        res_expired.as_ref().map(|s| &s.chat_room_id)
    );
    assert!(is_expired_under_180s, "Snapshot older than 180s must be expired under 180s TTL rule");

    // 3. Absolute timeout (> 2 hours = 7300s) -> MUST be evicted regardless
    let ancient_snap = ChampSelectHandoffSnapshot {
        game_id: 1003,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-test-ancient".to_string(),
        created_at_ms: now_ms.saturating_sub(7300 * 1000), // > 2 hours
    };
    engine.save_champ_select_snapshot(ancient_snap, None).await;
    let res_ancient = engine.get_valid_champ_select_snapshot(Some(1003), None).await;
    assert!(res_ancient.is_none(), "Snapshot older than 2 hours must always be expired and cleared");
}

#[tokio::test]
async fn test_snapshot_corrupted_storage_recovery_and_absent_keys() {
    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open test storage"));
    engine.set_storage(storage.clone()).await;

    // Case 1: Storage key is completely absent -> returns None cleanly, no panic
    let res_empty = engine.get_valid_champ_select_snapshot(None, None).await;
    assert!(res_empty.is_none(), "Absent storage key must return None without panic");

    // Case 2: Storage key contains corrupted / malformed JSON
    storage.set_setting("champ_select_handoff_snapshot", "{corrupted_json_syntax: [missing_brace").unwrap();
    let res_corrupt = engine.get_valid_champ_select_snapshot(None, None).await;
    assert!(res_corrupt.is_none(), "Malformed JSON must safely return None without panic");

    // Case 3: Storage key contains empty string ""
    storage.set_setting("champ_select_handoff_snapshot", "").unwrap();
    let res_empty_str = engine.get_valid_champ_select_snapshot(None, None).await;
    assert!(res_empty_str.is_none(), "Empty string setting must return None");

    // Case 4: Storage key contains valid JSON but mismatched schema
    storage.set_setting("champ_select_handoff_snapshot", r#"{"foo": "bar", "number": 123}"#).unwrap();
    let res_mismatch = engine.get_valid_champ_select_snapshot(None, None).await;
    assert!(res_mismatch.is_none(), "Schema mismatch must safely return None");
}

#[tokio::test]
async fn test_snapshot_zero_blackout_transition_and_game_id_isolation() {
    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open test storage"));
    engine.set_storage(storage.clone()).await;

    let now_ms = current_time_ms();

    // In LCU ChampSelect, game_id is often 0 or unassigned before the game server boots
    let mut card1 = OngoingPlayerCard::default();
    card1.slot_index = 0;
    card1.summoner_name = "PlayerOne".to_string();
    card1.champion_id = 103;

    let pre_game_snap = ChampSelectHandoffSnapshot {
        game_id: 0, // Unassigned during champ select
        queue_id: 420,
        blue_team: vec![card1.clone()],
        red_team: vec![],
        my_team: vec![card1.clone()],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "room-abc".to_string(),
        created_at_ms: now_ms,
    };
    engine.save_champ_select_snapshot(pre_game_snap, None).await;

    // Transition to InProgress: active_gid is now assigned by game server (e.g. 987654321)
    // The snapshot should NOT be rejected just because snap.game_id was 0 during champ select!
    let retrieved = engine.get_valid_champ_select_snapshot(Some(987654321), None).await;
    assert!(retrieved.is_some(), "Snapshot with game_id=0 must be accepted during transition to live game_id");
    assert_eq!(retrieved.unwrap().my_team[0].champion_id, 103);

    // Cross-game isolation: If snap.game_id > 0 (e.g. 11111) and new game has 22222 -> MUST reject and clear
    let stamped_snap = ChampSelectHandoffSnapshot {
        game_id: 11111,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "room-game-1".to_string(),
        created_at_ms: now_ms,
    };
    engine.save_champ_select_snapshot(stamped_snap, None).await;

    let mismatch = engine.get_valid_champ_select_snapshot(Some(22222), None).await;
    assert!(mismatch.is_none(), "Snapshot from different game_id must be discarded");

    // Verify it was cleared
    let cleared = engine.get_valid_champ_select_snapshot(None, None).await;
    assert!(cleared.is_none(), "Snapshot must be cleared from storage upon mismatch");
}

// ============================================================================
// Group 2: SGP GSM Live Game Role Parsing & Payload Edge Cases
// ============================================================================

#[test]
fn test_parse_gsm_selected_role_matrix() {
    // 1. Standard cases
    let (pos, reason, is_autofill) = parse_gsm_selected_role("MIDDLE.PRIMARY.MIDDLE.TOP.NONE");
    assert_eq!(pos, "MIDDLE");
    assert_eq!(reason, "PRIMARY");
    assert!(!is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("TOP.AUTOFILL.MIDDLE.BOTTOM.FILL");
    assert_eq!(pos, "TOP");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("JUNGLE.SECONDARY.TOP.JUNGLE.NONE");
    assert_eq!(pos, "JUNGLE");
    assert_eq!(reason, "SECONDARY");
    assert!(!is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("UTILITY.AUTOFILL.MIDDLE.TOP.FILL");
    assert_eq!(pos, "UTILITY");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    // 2. Case variations (lowercase & mixed case)
    let (pos, reason, is_autofill) = parse_gsm_selected_role("top.autofill.middle.bottom.fill");
    assert_eq!(pos, "TOP");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("Bottom.Primary.Bottom.Utility.None");
    assert_eq!(pos, "BOTTOM");
    assert_eq!(reason, "PRIMARY");
    assert!(!is_autofill);

    // 3. Short 2-part role strings
    let (pos, reason, is_autofill) = parse_gsm_selected_role("TOP.AUTOFILL");
    assert_eq!(pos, "TOP");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("BOTTOM.PRIMARY");
    assert_eq!(pos, "BOTTOM");
    assert_eq!(reason, "PRIMARY");
    assert!(!is_autofill);

    // 4. Single-part strings without dots
    let (pos, reason, is_autofill) = parse_gsm_selected_role("MIDDLE");
    assert_eq!(pos, "MIDDLE");
    assert_eq!(reason, "NONE");
    assert!(!is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("AUTOFILL");
    assert_eq!(pos, "AUTOFILL");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    // 5. Malformed, empty, and special strings (must not panic)
    let (pos, reason, is_autofill) = parse_gsm_selected_role("");
    assert_eq!(pos, "");
    assert_eq!(reason, "NONE");
    assert!(!is_autofill);

    let (pos, _, _) = parse_gsm_selected_role("...");
    assert_eq!(pos, "");

    let (pos, reason, is_autofill) = parse_gsm_selected_role(".AUTOFILL");
    assert_eq!(pos, "");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("TOP.autofill.");
    assert_eq!(pos, "TOP");
    assert_eq!(reason, "AUTOFILL");
    assert!(is_autofill);

    let (pos, reason, is_autofill) = parse_gsm_selected_role("UNKNOWN_ROLE_DATA");
    assert_eq!(pos, "UNKNOWN_ROLE_DATA");
    assert_eq!(reason, "NONE");
    assert!(!is_autofill);
}

#[test]
fn test_gsm_payload_deserialization_edge_cases() {
    // 1. Completely empty GSM response JSON
    let empty_json = "{}";
    let resp: GsmLedgeRegionResponse = serde_json::from_str(empty_json).expect("deserialize empty json");
    assert!(resp.game.is_none());

    // 2. Game with empty teams and no player selections
    let empty_game_json = json!({
        "game": {
            "id": 12345,
            "gameState": "IN_PROGRESS",
            "teamOne": [],
            "teamTwo": [],
            "playerChampionSelections": []
        }
    }).to_string();
    let resp2: GsmLedgeRegionResponse = serde_json::from_str(&empty_game_json).expect("deserialize empty game");
    let game = resp2.game.unwrap();
    assert_eq!(game.id, Some(12345));
    assert!(game.team_one.is_empty());
    assert!(game.team_two.is_empty());

    // 3. Team member with missing optional fields
    let partial_member_json = json!({
        "puuid": "test-puuid-123",
        "selectedRole": null,
        "selectedPosition": null,
        "teamParticipantId": null
    }).to_string();
    let member: GsmTeamMember = serde_json::from_str(&partial_member_json).expect("deserialize partial member");
    assert_eq!(member.puuid, "test-puuid-123");
    assert_eq!(member.selected_role, None);
    assert_eq!(member.selected_position, None);
    assert_eq!(member.team_participant_id, None);
}

// ============================================================================
// Group 3: Tactical Broadcast Presets Edge Cases & Stress Tests
// ============================================================================

#[test]
fn test_tactical_broadcast_concise_preset_zero_games() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "NovicePlayer".to_string();
    card.champion_id = 157;
    card.champion_name = "亚索".to_string();
    card.tactical_analysis.recent_count = 0; // 0 games

    let lines = generate_tactical_broadcast("concise", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "亚索：近期没有对局记录");
}

#[test]
fn test_tactical_broadcast_detailed_preset_zero_games() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "NoviceMid".to_string();
    card.champion_id = 103;
    card.champion_name = "阿狸".to_string();
    card.tactical_analysis.recent_count = 0; // 0 games

    let lines = generate_tactical_broadcast("detailed", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "阿狸：近期没有对局记录");
}

#[test]
fn test_tactical_broadcast_jungle_preset_zero_games_anomaly() {
    // EMPIRICAL TEST: When a jungler has 0 games / empty history records
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 1;
    card.position = "JUNGLE".to_string();
    card.summoner_name = "FirstTimeJungle".to_string();
    card.champion_id = 64;
    card.champion_name = "盲僧".to_string();
    card.spell1_id = 11; // Smite
    card.tactical_analysis.recent_count = 0;
    card.tactical_analysis.cur_champ_games = 0;
    card.matches = vec![]; // Empty matches

    let lines = generate_tactical_broadcast("jungle", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    // Check what sample count is reported:
    // In src/engine/player_history.rs:4019, `p.matches.len().max(16)` defaults to 16!
    println!("[EMPIRICAL OBSERVATION] Jungle broadcast for 0-game player: {}", line);
    // If the bug exists, line contains "样本16场" despite 0 matches!
    let fabricates_16_games = line.contains("样本16场");
    println!("[EMPIRICAL OBSERVATION] Does jungle preset fabricate 16 games? {}", fabricates_16_games);
    assert!(!fabricates_16_games, "Must not fabricate 16 games for player with 0 games");
    assert_eq!(line, "盲僧：近期没有对局记录");
}

#[test]
fn test_tactical_broadcast_unselected_champions() {
    // Test unselected champion (champion_id == 0, name == "选择中...")
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 2;
    card.summoner_name = "UnselectedPlayer".to_string();
    card.champion_id = 0;
    card.champion_name = "选择中...".to_string();
    card.tactical_analysis.recent_count = 10;
    card.tactical_analysis.recent_win_rate_pct = 50;
    card.tactical_analysis.avg_kda_ratio = 2.0;

    // 1. Concise
    let lines_concise = generate_tactical_broadcast("concise", &[card.clone()], &[], Some("preferChampionName"));
    assert_eq!(lines_concise.len(), 1);
    assert!(!lines_concise[0].contains("选择中..."), "Must not display '选择中...' as champion name: {}", lines_concise[0]);
    assert!(lines_concise[0].contains("UnselectedPlayer"), "Must fallback to summoner name: {}", lines_concise[0]);

    // 2. Detailed
    let lines_detailed = generate_tactical_broadcast("detailed", &[card.clone()], &[], Some("preferChampionName"));
    assert_eq!(lines_detailed.len(), 1);
    assert!(!lines_detailed[0].contains("选择中..."));

    // 3. Jungle
    let mut jg_card = card.clone();
    jg_card.position = "JUNGLE".to_string();
    let lines_jungle = generate_tactical_broadcast("jungle", &[jg_card], &[], Some("preferChampionName"));
    assert_eq!(lines_jungle.len(), 1);
    assert!(lines_jungle[0].contains("尚未选择英雄"), "Jungle preset must indicate hero not yet chosen: {}", lines_jungle[0]);
}

#[test]
fn test_tactical_broadcast_premade_single_player_party_isolated() {
    // A single player having premade_group = Some(1) with no other party members
    // MUST NOT be treated as a premade party (LeagueAkari requires >= 2 members).
    let mut p1 = OngoingPlayerCard::default();
    p1.slot_index = 0;
    p1.summoner_name = "SoloPartyPlayer".to_string();
    p1.premade_group = Some(1); // Lone party member

    let mut p2 = OngoingPlayerCard::default();
    p2.slot_index = 1;
    p2.summoner_name = "RealSoloPlayer".to_string();
    p2.premade_group = None;

    let my_team = vec![p1, p2];
    let lines = generate_tactical_broadcast("premade", &my_team, &[], Some("preferName"));
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "蓝方无开黑", "Lone player in party must not trigger premade party line");
}

#[test]
fn test_tactical_broadcast_empty_teams_and_missing_metrics() {
    // 1. Empty teams must not panic
    let lines_empty = generate_tactical_broadcast("concise", &[], &[], None);
    assert!(lines_empty.is_empty());

    let lines_empty_premade = generate_tactical_broadcast("premade", &[], &[], None);
    assert_eq!(lines_empty_premade.len(), 1);
    assert_eq!(lines_empty_premade[0], "蓝方无开黑");

    // 2. Missing metrics (kda = 0.0, zero shares)
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "ZeroMetricPlayer".to_string();
    card.champion_id = 1;
    card.champion_name = "安妮".to_string();
    card.tactical_analysis.recent_count = 5;
    card.tactical_analysis.recent_win_rate_pct = 0;
    card.tactical_analysis.avg_kda_ratio = 0.0; // 0.0 KDA

    let lines_zero = generate_tactical_broadcast("detailed", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines_zero.len(), 1);
    assert!(lines_zero[0].contains("KDA0.00"), "0.0 KDA must format safely as KDA0.00: {}", lines_zero[0]);
}
