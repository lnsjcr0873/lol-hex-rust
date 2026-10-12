mod lcu_mock_harness;

use lcu_mock_harness::fixtures::*;
use lcu_mock_harness::{MockLcuClient, MockLcuServer};
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::PlayerHistoryManager;
use lol_hex_rust::lcu::LcuClient;
use lol_hex_rust::overlay::{BenchOverlay, InGameOverlay, ItemOverlay};
use lol_hex_rust::web::routes::{
    build_mock_match_detail, build_mock_match_timeline, create_router, parse_riot_match_detail,
    AppState,
};
use lol_hex_rust::web::ws::WsBroadcaster;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Deserialize, PartialEq)]
pub struct MatchDetailParticipantContract {
    pub puuid: String,
    pub summoner_name: String,
    pub team: String,
    pub champion_id: i32,
    pub champion_name: String,
    pub champion_level: i32,
    pub spell1_id: i32,
    pub spell2_id: i32,
    pub spell1_icon: String,
    pub spell2_icon: String,
    pub tier: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda_ratio: f64,
    pub is_mvp: bool,
    pub is_svp: bool,
    pub total_damage: i64,
    pub is_highest_damage: bool,
    pub total_damage_taken: i64,
    pub gold_earned: i64,
    pub minions_killed: i32,
    pub items: Vec<i32>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MatchDetailFullContract {
    pub game_id: i64,
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_mode: String,
    pub game_mode_cn: String,
    pub queue_id: i32,
    pub queue_name: Option<String>,
    pub blue_team_kills: i32,
    pub blue_team_turrets: i32,
    pub blue_team_dragons: i32,
    pub blue_team_barons: i32,
    pub blue_team_win: bool,
    pub red_team_kills: i32,
    pub red_team_turrets: i32,
    pub red_team_dragons: i32,
    pub red_team_barons: i32,
    pub red_team_win: bool,
    pub participants: Vec<MatchDetailParticipantContract>,
}

#[tokio::test]
async fn test_lcu_client_get_game_timeline_success() {
    let server = MockLcuServer::start().await;
    let game_id = 9876543210u64;
    let canned_timeline = canned_game_timeline(game_id);
    server.state.set_match_timeline(game_id, canned_timeline.clone()).await;

    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let timeline = client.get_game_timeline(game_id).await;
    assert!(timeline.is_some(), "Timeline should be fetched successfully");
    let val = timeline.unwrap();

    assert_eq!(val["gameId"], game_id);
    let frames = val["frames"].as_array().expect("frames must be an array");
    assert_eq!(frames.len(), 16, "Should have 16 frames (minutes 0..15)");

    // Check frame 0 structure
    let f0 = &frames[0];
    assert_eq!(f0["timestamp"], 0);
    assert_eq!(f0["participantFrames"]["1"]["participantId"], 1);
    assert_eq!(f0["participantFrames"]["1"]["totalGold"], 500);

    // Check events present in frames
    let all_events: Vec<&Value> = frames
        .iter()
        .filter_map(|f| f.get("events").and_then(|e| e.as_array()))
        .flatten()
        .collect();

    let has_item = all_events.iter().any(|e| e.get("type").and_then(|t| t.as_str()) == Some("ITEM_PURCHASED"));
    let has_kill = all_events.iter().any(|e| e.get("type").and_then(|t| t.as_str()) == Some("CHAMPION_KILL"));
    let has_monster = all_events.iter().any(|e| e.get("type").and_then(|t| t.as_str()) == Some("ELITE_MONSTER_KILL"));
    let has_tower = all_events.iter().any(|e| e.get("type").and_then(|t| t.as_str()) == Some("BUILDING_KILL"));

    assert!(has_item, "Timeline should include ITEM_PURCHASED events");
    assert!(has_kill, "Timeline should include CHAMPION_KILL events");
    assert!(has_monster, "Timeline should include ELITE_MONSTER_KILL events");
    assert!(has_tower, "Timeline should include BUILDING_KILL events");

    // Also test alias method
    let alias_timeline = client.get_match_timeline(game_id).await;
    assert!(alias_timeline.is_some(), "get_match_timeline alias should also succeed");

    server.stop();
}

#[tokio::test]
async fn test_lcu_client_get_game_timeline_not_found() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    // Game ID not in mock server
    let timeline = client.get_game_timeline(404404404u64).await;
    assert!(timeline.is_none(), "Unseeded game timeline must return None (404)");

    server.stop();
}

#[tokio::test]
async fn test_lcu_client_get_game_timeline_zero_id() {
    let client = LcuClient::new();
    client.set_credentials(12345, "mock_token").await;

    // Passing 0 should immediately return None without calling network
    let timeline = client.get_game_timeline(0).await;
    assert!(timeline.is_none(), "Zero game ID must return None immediately");
}

#[tokio::test]
async fn test_mock_lcu_client_timeline_retrieval() {
    let server = MockLcuServer::start().await;
    let mock_client = MockLcuClient::from_server(&server);
    let game_id = 555666777u64;

    server.state.set_match_timeline(game_id, canned_game_timeline(game_id)).await;

    let res = mock_client.get_game_timeline(game_id).await;
    assert!(res.is_ok(), "MockLcuClient should retrieve timeline successfully");
    let val = res.unwrap();
    assert_eq!(val["gameId"], game_id);

    let err_res = mock_client.get_game_timeline(99999999).await;
    assert!(err_res.is_err(), "Missing timeline must return error");
    assert!(err_res.unwrap_err().contains("Status 404"));

    server.stop();
}

#[test]
fn test_parse_riot_match_detail_preserves_raw_game_and_raw() {
    let db = Database::new();
    let raw = canned_match_detail_classic(12345678, true);

    let parsed = parse_riot_match_detail(&raw, &db).expect("Should parse classic match");

    // 1. Verify backward compatibility with MatchDetailFullContract
    let contract_res: Result<MatchDetailFullContract, _> = serde_json::from_value(parsed.clone());
    assert!(contract_res.is_ok(), "Must strictly deserialize into MatchDetailFullContract: {:?}", contract_res.err());

    let contract = contract_res.unwrap();
    assert_eq!(contract.game_id, 12345678);
    assert_eq!(contract.participants.len(), 10);

    // 2. Verify raw_game and raw are preserved
    assert!(parsed.get("raw_game").is_some(), "Must contain raw_game field");
    assert!(parsed.get("raw").is_some(), "Must contain raw field");

    let raw_game = &parsed["raw_game"];
    assert_eq!(raw_game["gameId"], 12345678);
    assert!(raw_game.get("participants").is_some());
    assert!(raw_game.get("participantIdentities").is_some());
    assert_eq!(raw_game["gameMode"], "CLASSIC");
}

#[test]
fn test_build_mock_match_detail_preserves_raw_game_with_runes() {
    let db = Database::new();
    let mock = build_mock_match_detail(99998888, &db);

    // 1. Verify backward compatibility with MatchDetailFullContract
    let contract_res: Result<MatchDetailFullContract, _> = serde_json::from_value(mock.clone());
    assert!(contract_res.is_ok(), "build_mock_match_detail must deserialize into MatchDetailFullContract");

    // 2. Verify raw_game contains Riot-format participants with stats & runes
    assert!(mock.get("raw_game").is_some(), "build_mock_match_detail must contain raw_game");
    assert!(mock.get("raw").is_some(), "build_mock_match_detail must contain raw");

    let raw_game = &mock["raw_game"];
    assert_eq!(raw_game["gameId"], 99998888);
    let participants = raw_game["participants"].as_array().expect("raw_game participants array");
    assert_eq!(participants.len(), 10);

    let p0_stats = &participants[0]["stats"];
    assert_eq!(p0_stats["perkPrimaryStyle"], 8000, "Primary rune style must be present");
    assert_eq!(p0_stats["perkSubStyle"], 8400, "Sub rune style must be present");
    assert_eq!(p0_stats["perk0"], 8010, "Keystone perk must be present");
    assert_eq!(p0_stats["perk0Var1"], 1250, "Perk var1 must be present");
    assert!(p0_stats.get("physicalDamageDealtToChampions").is_some());
    assert!(p0_stats.get("magicDamageDealtToChampions").is_some());
    assert!(p0_stats.get("trueDamageDealtToChampions").is_some());
}

#[test]
fn test_build_mock_match_timeline_realism() {
    let timeline = build_mock_match_timeline(777888999);

    assert_eq!(timeline["gameId"], 777888999);
    let frames = timeline["frames"].as_array().expect("frames array");
    assert_eq!(frames.len(), 16);

    // Frame 1 check
    let f1 = &frames[1];
    assert_eq!(f1["timestamp"], 60_000);
    assert_eq!(f1["participantFrames"]["1"]["level"], 1);
    assert!(f1["participantFrames"]["1"]["totalGold"].as_i64().unwrap() >= 500);

    // Minute 3 first blood kill check
    let f3_events = frames[3]["events"].as_array().expect("frame 3 events");
    let fb = f3_events.iter().find(|e| e["type"] == "CHAMPION_KILL").expect("Must have first blood kill");
    assert_eq!(fb["killerId"], 1);
    assert_eq!(fb["victimId"], 6);

    // Minute 6 dragon kill check
    let f6_events = frames[6]["events"].as_array().expect("frame 6 events");
    let dragon = f6_events.iter().find(|e| e["type"] == "ELITE_MONSTER_KILL").expect("Must have dragon kill");
    assert_eq!(dragon["monsterType"], "DRAGON");
    assert_eq!(dragon["killerTeamId"], 100);

    // Minute 12 tower kill check
    let f12_events = frames[12]["events"].as_array().expect("frame 12 events");
    let tower = f12_events.iter().find(|e| e["type"] == "BUILDING_KILL").expect("Must have tower kill");
    assert_eq!(tower["buildingType"], "TOWER_BUILDING");
}

#[tokio::test]
async fn test_web_routes_timeline_endpoint() {
    let db = Arc::new(Database::new());
    let lcu = Arc::new(LcuClient::new());
    let history = Arc::new(PlayerHistoryManager::new());
    let config = Arc::new(ConfigManager::new("test_config.json"));
    let broadcaster = Arc::new(WsBroadcaster::new());

    let state = AppState {
        db,
        lcu,
        history,
        config,
        item_overlay: Arc::new(ItemOverlay::new()),
        ingame_overlay: Arc::new(InGameOverlay::new()),
        bench_overlay: Arc::new(BenchOverlay::new()),
        broadcaster,
    };

    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    let resp = reqwest::get(format!("http://{}/api/match/timeline/100200300", addr))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let json_resp: Value = resp.json().await.unwrap();
    assert_eq!(json_resp["gameId"], 100200300);
    assert!(json_resp["frames"].is_array());
    assert_eq!(json_resp["frames"].as_array().unwrap().len(), 16);
}
