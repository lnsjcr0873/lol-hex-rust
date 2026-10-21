//! Milestone 3 Integration & Parity Verification Tests
//!
//! Validates:
//! 1. Tactical Broadcast Presets Generation (concise, detailed, jungle, premade)
//! 2. Champ-Select Handoff Snapshot & Ongoing Match Continuity
//! 3. SGP GSM Live Game Role & Autofill parsing metadata

use lol_hex_rust::engine::player_history::{
    generate_tactical_broadcast, OngoingPlayerCard, ParsedMatchRecord,
    TopChampItem,
};

#[test]
fn test_tactical_broadcast_concise_preset() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "PlayerOne".to_string();
    card.champion_id = 157;
    card.champion_name = "亚索".to_string();
    card.tactical_analysis.recent_count = 20;
    card.tactical_analysis.recent_win_rate_pct = 65;
    card.tactical_analysis.avg_kda_ratio = 3.25;
    card.tactical_analysis.top_champs = vec![
        TopChampItem {
            champion_id: 157,
            champion_name: "亚索".to_string(),
            games: 12,
            win_rate: 0.67,
            win_rate_pct: 67,
            kda: 3.5,
        },
    ];

    let lines = generate_tactical_broadcast("concise", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("亚索：20场对局，胜率65% KDA3.25"), "Actual: {}", line);
    assert!(line.contains("主玩英雄[亚索]"), "Actual: {}", line);
}

#[test]
fn test_tactical_broadcast_detailed_preset() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 0;
    card.summoner_name = "MidCarry".to_string();
    card.champion_id = 103;
    card.champion_name = "阿狸".to_string();
    card.tactical_analysis.recent_count = 15;
    card.tactical_analysis.recent_win_rate_pct = 73;
    card.tactical_analysis.avg_kda_ratio = 4.10;

    let mut m1 = ParsedMatchRecord::default();
    m1.damage_share_pct = 32.5;
    m1.damage_taken = 15000;
    m1.cs_per_min = 8.2;
    m1.kp_pct = 70.0;
    m1.vision_score = 28;
    m1.kills = 8;
    m1.assists = 4;
    card.matches.push(m1);

    let lines = generate_tactical_broadcast("detailed", &[card], &[], Some("preferChampionName"));
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("阿狸：15场对局，胜率73% KDA4.10"), "Actual: {}", line);
    assert!(line.contains("伤害"), "Actual: {}", line);
    assert!(line.contains("承伤"), "Actual: {}", line);
    assert!(line.contains("补兵"), "Actual: {}", line);
    assert!(line.contains("参团率"), "Actual: {}", line);
    assert!(line.contains("伤转比"), "Actual: {}", line);
}

#[test]
fn test_tactical_broadcast_jungle_preset_detection() {
    let mut mid = OngoingPlayerCard::default();
    mid.slot_index = 0;
    mid.position = "MIDDLE".to_string();
    mid.champion_id = 103;
    mid.champion_name = "阿狸".to_string();

    let mut jg = OngoingPlayerCard::default();
    jg.slot_index = 1;
    jg.position = "JUNGLE".to_string();
    jg.champion_id = 64;
    jg.champion_name = "盲僧".to_string();
    jg.spell1_id = 11; // Smite
    jg.spell2_id = 4;  // Flash
    jg.tactical_analysis.cur_champ_games = 18;

    let mut m1 = ParsedMatchRecord::default();
    m1.champion_id = 64;
    m1.lane = "JUNGLE".to_string();
    m1.win = true;
    m1.kills = 8;
    m1.deaths = 2;
    m1.assists = 6;
    m1.kp_pct = 70.0;
    m1.vision_score = 36;
    m1.dragon_kills = 3;
    m1.baron_kills = 1;
    jg.matches.push(m1);

    let my_team = vec![mid, jg];
    let lines = generate_tactical_broadcast("jungle", &my_team, &[], Some("preferChampionName"));

    // Should prioritize the jungler (盲僧) and calculate dynamic statistics from actual matches
    assert_eq!(lines.len(), 1);
    let line = &lines[0];
    assert!(line.contains("盲僧：本英雄打野样本1场"), "Actual: {}", line);
    assert!(line.contains("胜率100%"), "Actual: {}", line);
    assert!(line.contains("KDA7.00"), "Actual: {}", line);
    assert!(line.contains("场均参团率70%"), "Actual: {}", line);
    assert!(line.contains("场均视野得分36.0"), "Actual: {}", line);
    assert!(line.contains("场均小龙3.0"), "Actual: {}", line);
    assert!(line.contains("场均大龙1.0"), "Actual: {}", line);
}

#[tokio::test]
async fn test_snapshot_expiry_ttl_180s() {
    use lol_hex_rust::engine::player_history::{ChampSelectHandoffSnapshot, PlayerHistoryManager, CHAMP_SELECT_SNAPSHOT_TTL_MS};
    use lol_hex_rust::storage::StorageManager;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    assert_eq!(CHAMP_SELECT_SNAPSHOT_TTL_MS, 180 * 1000);

    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open test storage"));
    engine.set_storage(storage.clone()).await;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let valid_snap = ChampSelectHandoffSnapshot {
        game_id: 2001,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-valid".to_string(),
        created_at_ms: now_ms.saturating_sub(120 * 1000),
    };
    engine.save_champ_select_snapshot(valid_snap, None).await;
    let res = engine.get_valid_champ_select_snapshot(Some(2001), None).await;
    assert!(res.is_some(), "Snapshot under 180s must be valid");

    let expired_snap = ChampSelectHandoffSnapshot {
        game_id: 2002,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "chat-expired".to_string(),
        created_at_ms: now_ms.saturating_sub(190 * 1000),
    };
    engine.save_champ_select_snapshot(expired_snap, None).await;
    let res_expired = engine.get_valid_champ_select_snapshot(Some(2002), None).await;
    assert!(res_expired.is_none(), "Snapshot over 180s must be expired");
}

#[test]
fn test_tactical_broadcast_premade_preset_clustering() {
    let mut p1 = OngoingPlayerCard::default();
    p1.slot_index = 0;
    p1.summoner_name = "PlayerA".to_string();
    p1.premade_group = Some(1);

    let mut p2 = OngoingPlayerCard::default();
    p2.slot_index = 1;
    p2.summoner_name = "PlayerB".to_string();
    p2.premade_group = Some(1);

    let mut p3 = OngoingPlayerCard::default();
    p3.slot_index = 2;
    p3.summoner_name = "SoloPlayer".to_string();
    p3.premade_group = None;

    let mut enemy1 = OngoingPlayerCard::default();
    enemy1.slot_index = 0;
    enemy1.summoner_name = "Enemy1".to_string();
    enemy1.premade_group = Some(2);

    let mut enemy2 = OngoingPlayerCard::default();
    enemy2.slot_index = 1;
    enemy2.summoner_name = "Enemy2".to_string();
    enemy2.premade_group = Some(2);

    let my_team = vec![p1, p2, p3];
    let their_team = vec![enemy1, enemy2];

    let lines = generate_tactical_broadcast("premade", &my_team, &their_team, Some("preferName"));
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("蓝方开黑：[PlayerA, PlayerB]"), "Actual: {}", lines[0]);
    assert!(lines[1].contains("红方开黑：[Enemy1, Enemy2]"), "Actual: {}", lines[1]);
}

#[test]
fn test_ongoing_player_card_autofill_and_team_participant_fields() {
    let mut card = OngoingPlayerCard::default();
    assert!(!card.is_autofilled, "Default is_autofilled must be false");
    assert_eq!(card.team_participant_id, None, "Default team_participant_id must be None");

    card.is_autofilled = true;
    card.team_participant_id = Some(105);

    let serialized = serde_json::to_string(&card).expect("serialization succeeds");
    let deserialized: OngoingPlayerCard = serde_json::from_str(&serialized).expect("deserialization succeeds");

    assert!(deserialized.is_autofilled);
    assert_eq!(deserialized.team_participant_id, Some(105));
}
