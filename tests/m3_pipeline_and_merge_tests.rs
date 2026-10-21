//! Integration Tests for Milestone 3 Pipeline, Snapshot Stitching, and GSM Merge Logic

use lol_hex_rust::engine::player_history::{
    convert_ongoing_cards_to_slots, member_merge, ChampSelectHandoffSnapshot,
    GameflowSlotInfo, OngoingPlayerCard, PlayerHistoryManager,
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

#[test]
fn test_member_merge_lcs_alignment() {
    // 1. Origin is subset of Extra (e.g. 2 players revealed in gameflow, 5 in snapshot)
    let origin = vec!["puuid_1".to_string(), "puuid_3".to_string()];
    let extra = vec![
        "puuid_1".to_string(),
        "puuid_2".to_string(),
        "puuid_3".to_string(),
        "puuid_4".to_string(),
        "puuid_5".to_string(),
    ];

    let merged = member_merge(&origin, &extra);
    assert_eq!(merged.len(), 5);
    assert_eq!(merged, extra);

    // 2. Both equal
    let merged_equal = member_merge(&extra, &extra);
    assert_eq!(merged_equal, extra);

    // 3. Origin is empty
    let merged_empty_origin = member_merge(&[], &extra);
    assert_eq!(merged_empty_origin, extra);

    // 4. Extra is empty
    let merged_empty_extra = member_merge(&origin, &[]);
    assert_eq!(merged_empty_extra, origin);
}

#[tokio::test]
async fn test_stitch_team_with_snapshot_populates_missing_slots() {
    let engine = PlayerHistoryManager::new();

    let mut current_slots = vec![
        GameflowSlotInfo {
            cell_id: 0,
            puuid: "puuid_A".to_string(),
            summoner_name: "PlayerA".to_string(),
            champion_id: 157,
            position: "MID".to_string(),
            position_cn: "中单".to_string(),
            is_local_player: true,
            profile: None,
            team_id: 100,
            spell1_id: 4,
            spell2_id: 14,
        },
    ];

    let mut card_a = OngoingPlayerCard::default();
    card_a.slot_index = 0;
    card_a.puuid = "puuid_A".to_string();
    card_a.summoner_name = "PlayerA".to_string();
    card_a.champion_id = 157;
    card_a.position = "MID".to_string();
    card_a.is_local_player = true;

    let mut card_b = OngoingPlayerCard::default();
    card_b.slot_index = 1;
    card_b.puuid = "puuid_B".to_string();
    card_b.summoner_name = "PlayerB".to_string();
    card_b.champion_id = 64;
    card_b.position = "JUNGLE".to_string();
    card_b.spell1_id = 11;
    card_b.spell2_id = 4;

    let mut card_c = OngoingPlayerCard::default();
    card_c.slot_index = 2;
    card_c.puuid = "puuid_C".to_string();
    card_c.summoner_name = "PlayerC".to_string();
    card_c.champion_id = 51;
    card_c.position = "BOTTOM".to_string();

    let snapshot_cards = vec![card_a, card_b, card_c];

    engine.stitch_team_with_snapshot(&mut current_slots, &snapshot_cards);

    assert_eq!(current_slots.len(), 3, "Stitching must expand slots to 3 members");
    assert_eq!(current_slots[0].puuid, "puuid_A");
    assert_eq!(current_slots[1].puuid, "puuid_B");
    assert_eq!(current_slots[1].position, "JUNGLE");
    assert_eq!(current_slots[1].spell1_id, 11);
    assert_eq!(current_slots[2].puuid, "puuid_C");
    assert_eq!(current_slots[2].position, "BOTTOM");
}

#[tokio::test]
async fn test_convert_ongoing_cards_to_slots_mapping() {
    let mut card = OngoingPlayerCard::default();
    card.slot_index = 3;
    card.puuid = "puuid_test_123".to_string();
    card.summoner_name = "Tester".to_string();
    card.champion_id = 89;
    card.position = "UTILITY".to_string();
    card.is_local_player = false;
    card.spell1_id = 4;
    card.spell2_id = 3;

    let slots = convert_ongoing_cards_to_slots(&[card]);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].cell_id, 3);
    assert_eq!(slots[0].puuid, "puuid_test_123");
    assert_eq!(slots[0].summoner_name, "Tester");
    assert_eq!(slots[0].champion_id, 89);
    assert_eq!(slots[0].position, "UTILITY");
    assert_eq!(slots[0].spell1_id, 4);
    assert_eq!(slots[0].spell2_id, 3);
}

#[tokio::test]
async fn test_clear_champ_select_snapshot_memory_and_storage() {
    let engine = PlayerHistoryManager::new();
    let storage = Arc::new(StorageManager::open_in_memory().expect("open test storage"));
    engine.set_storage(storage.clone()).await;

    let snap = ChampSelectHandoffSnapshot {
        game_id: 8888,
        queue_id: 420,
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        is_my_team_blue: true,
        chat_room_id: "test-room".to_string(),
        created_at_ms: current_time_ms(),
    };

    engine.save_champ_select_snapshot(snap, Some(&storage)).await;
    assert!(engine.get_valid_champ_select_snapshot(Some(8888), Some(&storage)).await.is_some());

    engine.clear_champ_select_snapshot(Some(&storage)).await;
    assert!(engine.get_valid_champ_select_snapshot(Some(8888), Some(&storage)).await.is_none());
}
