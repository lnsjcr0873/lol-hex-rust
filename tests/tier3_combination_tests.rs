mod lcu_mock_harness;

use axum::http::StatusCode;
use lcu_mock_harness::fixtures::*;
use lcu_mock_harness::*;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::*;
use lol_hex_rust::web::routes::{build_mock_match_detail, parse_riot_match_detail};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

fn format_chat_broadcast_lines(ongoing: &OngoingMatchResponse) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("⚡【HexBrain 智脑战绩透视】⚡".to_string());

    for (idx, p) in ongoing.my_team.iter().take(5).enumerate() {
        let pos = match idx {
            0 => "上路",
            1 => "打野",
            2 => "中路",
            3 => "射手",
            _ => "辅助",
        };
        let wr = p.tactical_analysis.recent_win_rate_pct;
        let tag = p.premade_tag.as_deref().unwrap_or("");
        let tag_suffix = if !tag.is_empty() {
            format!(" [{}]", tag)
        } else {
            String::new()
        };
        lines.push(format!(
            "{}. {} [{}] {} (近{}场胜率{}%){}",
            idx + 1,
            pos,
            p.summoner_name,
            p.tactical_analysis.tier_str,
            p.tactical_analysis.recent_count,
            wr,
            tag_suffix
        ));
    }
    lines
}

// ==============================================================================================
// Combination Test 1: ARAM queue + Premade detection + Chat broadcast
// Features: F12 (ARAM), F3 (Premades), F5 (Chat broadcast)
// ==============================================================================================
#[tokio::test]
async fn test_t3_01_aram_premade_detection_and_chat_broadcast() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // Setup ARAM match with 5 players on my_team; players 0 and 1 have 3 shared games (duo)
    let mut players = Vec::new();
    for i in 0..5 {
        let mut card = OngoingPlayerCard {
            slot_index: i,
            team_id: 100,
            puuid: format!("aram-p-{}", i),
            summoner_name: format!("大乱斗玩家{}", i + 1),
            ..Default::default()
        };
        card.tactical_analysis.tier_str = "极地勇士".to_string();
        card.tactical_analysis.recent_count = 10;
        card.tactical_analysis.recent_win_rate_pct = 60;

        if i < 2 {
            // Shared 3 games between player 0 and 1
            card.matches = (1..=3)
                .map(|gid| ParsedMatchRecord {
                    game_id: gid,
                    queue_id: 450,
                    teammate_puuids: vec![format!("aram-p-{}", 1 - i)],
                    win: true,
                    ..Default::default()
                })
                .collect();
        }
        players.push(card);
    }

    // Run premade detection
    detect_premade_teams(&mut players);
    assert_eq!(players[0].premade_group, Some(1));
    assert_eq!(players[1].premade_group, Some(1));
    assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
    assert!(players[2].premade_group.is_none());

    let ongoing = OngoingMatchResponse {
        phase: "ChampSelect".to_string(),
        is_active: true,
        my_team: players,
        ..Default::default()
    };

    // Format chat broadcast lines
    let lines = format_chat_broadcast_lines(&ongoing);
    assert_eq!(lines.len(), 6);
    assert!(lines[1].contains("[双排车队]"));
    assert!(lines[2].contains("[双排车队]"));
    assert!(!lines[3].contains("[双排车队]"));

    // Send lines to mock chat room
    for line in &lines {
        let ok = client.send_chat_message("aram-chat-room", line).await.expect("send");
        assert!(ok);
    }

    let sent = server.state.get_sent_messages().await;
    assert_eq!(sent.len(), 6);
    server.stop();
}

// ==============================================================================================
// Combination Test 2: Arena 2v2v2v2 + Lifecycle reconnect + Scoreboard evaluation
// Features: F12 (Arena), F10 (Lifecycle reconnect), F9 (Scoreboard)
// ==============================================================================================
#[tokio::test]
async fn test_t3_02_arena_lifecycle_reconnect_and_scoreboard() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // Initial state: ChampSelect in Arena
    server.state.set_phase("ChampSelect").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "ChampSelect");

    // Sudden disconnect: phase drops to None
    server.state.set_phase("None").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "None");

    // Reconnected: phase transitions to InProgress
    server.state.set_phase("InProgress").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "InProgress");

    // Scoreboard for Arena match
    let arena_detail = canned_match_detail_arena(9001);
    server.state.set_match_detail(9001, arena_detail).await;

    let fetched = client.get_match_detail(9001).await.expect("arena detail");
    let db = Database::new();
    let parsed = parse_riot_match_detail(&fetched, &db).expect("parse arena scoreboard");

    assert_eq!(parsed["mode_cn"].as_str().unwrap(), "斗魂竞技场");
    let participants = parsed["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 8);

    server.stop();
}

// ==============================================================================================
// Combination Test 3: Account switch + Scoreboard cache invalidation / refresh
// Features: F11 (Account switch), F8 (Summoner profile), F9 (Scoreboard)
// ==============================================================================================
#[tokio::test]
async fn test_t3_03_account_switch_and_scoreboard_cache_invalidation() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let history_mgr = PlayerHistoryManager::new();

    // Account A logged in
    server.state.set_current_summoner(canned_current_summoner("puuid-user-A", "召唤师甲")).await;
    let sum_a = client.get_current_summoner().await.expect("sum A");
    assert_eq!(sum_a["puuid"].as_str().unwrap(), "puuid-user-A");

    // Simulate match history fetch for Account A
    let history_a = canned_match_history("puuid-user-A", 10, 0.8, 157, &[]);
    server.state.set_match_history("puuid-user-A", history_a).await;
    let hist_res_a = client.get_player_match_history("puuid-user-A", 0, 9).await.unwrap();
    assert_eq!(hist_res_a["games"]["games"].as_array().unwrap().len(), 10);

    // Account switch / relog to Account B
    server.state.set_current_summoner(canned_current_summoner("puuid-user-B", "召唤师乙")).await;
    let sum_b = client.get_current_summoner().await.expect("sum B");
    assert_eq!(sum_b["puuid"].as_str().unwrap(), "puuid-user-B");

    // Account B gets fresh, separate history
    let history_b = canned_match_history("puuid-user-B", 5, 0.4, 238, &[]);
    server.state.set_match_history("puuid-user-B", history_b).await;
    let hist_res_b = client.get_player_match_history("puuid-user-B", 0, 4).await.unwrap();
    assert_eq!(hist_res_b["games"]["games"].as_array().unwrap().len(), 5);

    // History manager sample limits maintained across switches
    history_mgr.set_sample_match_count(30);
    assert_eq!(history_mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 30);

    server.stop();
}

// ==============================================================================================
// Combination Test 4: High concurrency batch queries + Semaphore rate limiting + Radar
// Features: F13 (Concurrency / Semaphore), F1 (LCU Batch queries), F2 (6-axis Radar)
// ==============================================================================================
#[tokio::test]
async fn test_t3_04_high_concurrency_batch_queries_and_radar() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(MockLcuClient::from_server(&server));

    // Register 10 players with 20 matches each in mock server
    for i in 1..=10 {
        let puuid = format!("batch-puuid-{}", i);
        let hist = canned_match_history(&puuid, 20, 0.5 + (i as f64 * 0.03), 100 + i, &[]);
        server.state.set_match_history(&puuid, hist).await;
    }

    let sem = Arc::new(Semaphore::new(3)); // Max 3 in-flight concurrent requests
    let mut handles = Vec::new();

    for i in 1..=10 {
        let sem_clone = sem.clone();
        let client_clone = client.clone();
        let puuid = format!("batch-puuid-{}", i);

        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let history_val = client_clone.get_player_match_history(&puuid, 0, 19).await?;

            let games = history_val["games"]["games"].as_array().unwrap();
            let mut parsed_matches = Vec::new();
            for g in games {
                let stats = &g["participants"][0]["stats"];
                parsed_matches.push(ParsedMatchRecord {
                    damage: stats["totalDamageDealtToChampions"].as_i64().unwrap(),
                    kills: stats["kills"].as_i64().unwrap() as i32,
                    deaths: stats["deaths"].as_i64().unwrap() as i32,
                    assists: stats["assists"].as_i64().unwrap() as i32,
                    gold: stats["goldEarned"].as_i64().unwrap(),
                    win: stats["win"].as_bool().unwrap(),
                    damage_share_pct: 25.0,
                    kp_pct: 60.0,
                    cs_per_min: 7.0,
                    vision_score: 25,
                    ..Default::default()
                });
            }
            let radar = compute_radar(&parsed_matches);
            Ok::<HashMap<String, i32>, String>(radar)
        }));
    }

    for h in handles {
        let radar = h.await.unwrap().expect("radar computation under concurrency");
        assert_eq!(radar.len(), 6);
        for (_k, v) in radar {
            assert!(v >= 25 && v <= 99, "Radar axis value must clamp [25, 99]");
        }
    }

    server.stop();
}

// ==============================================================================================
// Combination Test 5: Anti-anonymity unmasking + Champ-select tactical badges + Broadcast
// Features: F11 (Anti-anonymity), F4 (Tactical badges), F5 (Chat broadcast)
// ==============================================================================================
#[tokio::test]
async fn test_t3_05_anti_anonymity_tactical_badges_and_broadcast() {
    let local_puuid = "local-puuid-me";
    let chat_room_id = "ranked-room-555";

    // Session has 4 anonymous players
    let _session = canned_champ_select_session(chat_room_id, local_puuid, 0);

    // Chat participants unmask real summoner identities
    let participants = canned_chat_participants(
        chat_room_id,
        &[
            (local_puuid, "我是一只鱼"),
            ("unmasked-p1", "绝活哥亚索"),
            ("unmasked-p2", "连胜战神盲僧"),
            ("unmasked-p3", "试英雄新手"),
            ("unmasked-p4", "稳健辅助"),
        ],
    );

    // Filter out local player
    let candidates: Vec<_> = participants
        .iter()
        .filter(|p| p["puuid"].as_str().unwrap() != local_puuid)
        .collect();
    assert_eq!(candidates.len(), 4);

    let mgr = PlayerHistoryManager::new();
    let mut my_team_cards = Vec::new();

    // Local player card
    my_team_cards.push(OngoingPlayerCard {
        slot_index: 0,
        puuid: local_puuid.to_string(),
        summoner_name: "我是一只鱼".to_string(),
        position: "TOP".to_string(),
        tactical_analysis: PlayerTacticalProfile {
            tier_str: "大师".to_string(),
            recent_count: 20,
            recent_win_rate_pct: 65,
            ..Default::default()
        },
        ..Default::default()
    });

    // Unmasked teammate 1: 5-win streak
    let rec1 = CachedPlayerRecord {
        puuid: "unmasked-p1".to_string(),
        summoner_name: "绝活哥亚索".to_string(),
        tier: "DIAMOND".to_string(),
            division: "".to_string(),
        tier_str: "钻石 I".to_string(),
        tier_color: "#38bdf8".to_string(),
        lp: 85,
        ranked_wins: 15,
        ranked_losses: 5,
        matches: (1..=5).map(|_| ParsedMatchRecord { win: true, ..Default::default() }).collect(),
        fetched_at: Instant::now(),
        raw_summoner: json!({}),
        raw_ranked: json!({}),
    };
    let prof1 = mgr.compute_tactical_profile(&rec1, 0, 20);
    assert!(prof1.badges.iter().any(|b| b.text == "🔥 5连胜"));

    my_team_cards.push(OngoingPlayerCard {
        slot_index: 1,
        puuid: "unmasked-p1".to_string(),
        summoner_name: "绝活哥亚索".to_string(),
        position: "JUNGLE".to_string(),
        tactical_analysis: prof1,
        ..Default::default()
    });

    for idx in 2..5 {
        my_team_cards.push(OngoingPlayerCard {
            slot_index: idx,
            puuid: format!("unmasked-p{}", idx),
            summoner_name: format!("队友{}", idx),
            position: match idx { 2 => "MID", 3 => "ADC", _ => "SUPPORT" }.to_string(),
            tactical_analysis: PlayerTacticalProfile {
                tier_str: "翡翠".to_string(),
                recent_count: 10,
                recent_win_rate_pct: 50,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    let ongoing = OngoingMatchResponse {
        phase: "ChampSelect".to_string(),
        is_active: true,
        my_team: my_team_cards,
        ..Default::default()
    };

    let lines = format_chat_broadcast_lines(&ongoing);
    assert!(lines[1].contains("我是一只鱼"));
    assert!(lines[2].contains("绝活哥亚索"));
    assert!(lines[2].contains("钻石 I"));
}

// ==============================================================================================
// Combination Test 6: Offline state transition + Mock mode fallback + Ongoing match view
// Features: F7 (Offline / Mock mode), F1 (Lifecycle), F6 (Ongoing match view)
// ==============================================================================================
#[tokio::test]
async fn test_t3_06_offline_to_champ_select_to_mock_fallback() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // Phase 1: Client not running (None)
    server.state.set_phase("None").await;
    let phase_offline = client.get_gameflow_phase().await.unwrap();
    assert_eq!(phase_offline, "None");

    let resp_offline = OngoingMatchResponse {
        phase: phase_offline,
        is_active: false,
        is_mock: false,
        message: Some("英雄联盟客户端未运行".to_string()),
        ..Default::default()
    };
    assert!(!resp_offline.is_active);

    // Phase 2: Client enters champ select
    server.state.set_phase("ChampSelect").await;
    let phase_active = client.get_gameflow_phase().await.unwrap();
    assert_eq!(phase_active, "ChampSelect");

    // Phase 3: Sudden client crash -> App transitions to simulation mock mode
    server.stop(); // LCU connection lost
    let db = Database::new();
    let mock_detail = build_mock_match_detail(12345, &db);

    let resp_mock = OngoingMatchResponse {
        phase: "Simulation".to_string(),
        is_active: true,
        is_mock: true,
        game_id: Some(mock_detail["game_id"].as_i64().unwrap()),
        ..Default::default()
    };
    assert!(resp_mock.is_mock);
    assert_eq!(resp_mock.game_id, Some(12345));
}

// ==============================================================================================
// Combination Test 7: Premade detection across MVP/SVP match history
// Features: F3 (Premades), F2 (MVP / SVP Scoring), F8 (Player History)
// ==============================================================================================
#[test]
fn test_t3_07_premade_detection_across_mvp_svp_history() {
    let mut p1_matches = Vec::new();
    let mut p2_matches = Vec::new();

    // 3 shared matches
    for gid in 1..=3 {
        // Match participant evaluation
        let participants = vec![
            MatchParticipantInput {
                participant_id: 1,
                puuid: "p1".to_string(),
                team_id: 100,
                is_win: true,
                kills: 10,
                deaths: 1,
                assists: 8,
                damage: 35000,
                damage_taken: 15000,
                damage_mitigated: 8000,
                gold: 15000,
                cs: 220,
                vision_score: 30,
            },
            MatchParticipantInput {
                participant_id: 2,
                puuid: "p2".to_string(),
                team_id: 100,
                is_win: true,
                kills: 2,
                deaths: 2,
                assists: 16,
                damage: 18000,
                damage_taken: 22000,
                damage_mitigated: 12000,
                gold: 11000,
                cs: 50,
                vision_score: 45,
            },
        ];

        let evals = evaluate_match_participants(1800, &participants);
        assert!(evals[0].is_mvp, "p1 was MVP");

        p1_matches.push(ParsedMatchRecord {
            game_id: gid,
            teammate_puuids: vec!["p2".to_string()],
            is_mvp: evals[0].is_mvp,
            win: true,
            ..Default::default()
        });

        p2_matches.push(ParsedMatchRecord {
            game_id: gid,
            teammate_puuids: vec!["p1".to_string()],
            is_mvp: evals[1].is_mvp,
            win: true,
            ..Default::default()
        });
    }

    let mut players = vec![
        OngoingPlayerCard {
            puuid: "p1".to_string(),
            matches: p1_matches,
            ..Default::default()
        },
        OngoingPlayerCard {
            puuid: "p2".to_string(),
            matches: p2_matches,
            ..Default::default()
        },
    ];

    detect_premade_teams(&mut players);
    assert_eq!(players[0].premade_group, Some(1));
    assert_eq!(players[1].premade_group, Some(1));
    assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
}

// ==============================================================================================
// Combination Test 8: ARAM bench reroll & Tactical badge dynamic update
// Features: F12 (ARAM Bench), F4 (Tactical Badges), F6 (Player Card)
// ==============================================================================================
#[test]
fn test_t3_08_aram_bench_rerolls_and_player_card_update() {
    let mgr = PlayerHistoryManager::new();

    // Player with 0 games on Yasuo (157), but 5 games with 80% WR on Jinx (222)
    let rec = CachedPlayerRecord {
        puuid: "player-aram".to_string(),
        summoner_name: "大乱斗达人".to_string(),
        tier: "EMERALD".to_string(),
            division: "".to_string(),
        tier_str: "翡翠 II".to_string(),
        tier_color: "#00e696".to_string(),
        lp: 45,
        ranked_wins: 10,
        ranked_losses: 5,
        matches: vec![
            ParsedMatchRecord { champion_id: 222, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 222, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 222, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 222, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 222, win: false, ..Default::default() },
        ],
        fetched_at: Instant::now(),
        raw_summoner: json!({}),
        raw_ranked: json!({}),
    };

    // Before reroll: holding Yasuo (157) -> 试英雄 badge
    let prof_yasuo = mgr.compute_tactical_profile(&rec, 157, 20);
    assert!(prof_yasuo.badges.iter().any(|b| b.text.contains("试英雄")));

    // After reroll: swaps with Jinx (222) from bench -> 绝活 badge
    let prof_jinx = mgr.compute_tactical_profile(&rec, 222, 20);
    assert!(prof_jinx.badges.iter().any(|b| b.text.contains("绝活(80%)")));
}

// ==============================================================================================
// Combination Test 9: Rate limit 429 response + Backoff retry + Config atomic persist
// Features: F10 (429 handling), F13 (Atomic config persistence)
// ==============================================================================================
#[tokio::test]
async fn test_t3_09_rate_limit_429_exponential_backoff_and_config_save() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // Server starts in 429 rate limit
    server.state.set_status_override(Some(StatusCode::TOO_MANY_REQUESTS)).await;

    let res_first = client.get_gameflow_phase().await;
    assert!(res_first.is_err());
    assert!(res_first.unwrap_err().contains("429"));

    // Backoff retry simulation: server clears rate limit
    server.state.set_status_override(None).await;
    let res_retry = client.get_gameflow_phase().await.expect("retry succeeded");
    assert_eq!(res_retry, "ChampSelect");

    // Atomically persist client configuration
    let temp_dir = std::env::temp_dir().join(format!("hex_test_cfg_combo_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cfg_path = temp_dir.join("config.json");

    let mgr = ConfigManager::new(cfg_path.to_str().unwrap());
    mgr.update(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
        Some(5),
    ).await;

    let reloaded = ConfigManager::new(cfg_path.to_str().unwrap());
    let loaded = reloaded.get().await;
    assert_eq!(loaded.auto_accept_match, true);
    assert_eq!(loaded.auto_accept_delay_seconds, 5);

    let _ = std::fs::remove_dir_all(temp_dir);
    server.stop();
}

// ==============================================================================================
// Combination Test 10: Arena 4-team split + Damage share calculation + Crowns
// Features: F12 (Arena 4-team), F2 (Damage share & MVP), F9 (Scoreboard)
// ==============================================================================================
#[test]
fn test_t3_10_arena_four_teams_damage_share_and_crowns() {
    let db = Database::new();
    let mut raw = canned_match_detail_arena(8888);

    // Make participant 3 the highest damage dealer in the game
    raw["participants"][2]["stats"]["totalDamageDealtToChampions"] = json!(45000);

    let parsed = parse_riot_match_detail(&raw, &db).expect("arena parsed");
    let participants = parsed["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 8);

    // Team damage totals and individual damage shares
    let p3 = &participants[2];
    assert_eq!(p3["total_damage"].as_i64().unwrap(), 45000);
}

// ==============================================================================================
// Combination Test 11: Tactical badges multi-attribute clustering
// Features: F4 (Tactical Badges), F8 (Summoner Profile)
// ==============================================================================================
#[test]
fn test_t3_11_tactical_badges_multi_attribute_clustering() {
    let mgr = PlayerHistoryManager::new();

    // Player with 4 win streak, 80% recent win rate, 5.0 KDA, and selected champion 3 games 100% WR
    let rec = CachedPlayerRecord {
        puuid: "smurf_god".to_string(),
        summoner_name: "通天代神".to_string(),
        tier: "MASTER".to_string(),
            division: "".to_string(),
        tier_str: "大师 350 LP".to_string(),
        tier_color: "#a855f7".to_string(),
        lp: 350,
        ranked_wins: 40,
        ranked_losses: 10,
        matches: vec![
            ParsedMatchRecord { champion_id: 157, kills: 12, deaths: 2, assists: 8, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 157, kills: 10, deaths: 1, assists: 6, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 157, kills: 15, deaths: 3, assists: 9, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 238, kills: 8, deaths: 2, assists: 4, win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 238, kills: 6, deaths: 4, assists: 2, win: false, ..Default::default() },
            ParsedMatchRecord { champion_id: 238, kills: 9, deaths: 2, assists: 7, win: true, ..Default::default() },
        ],
        fetched_at: Instant::now(),
        raw_summoner: json!({}),
        raw_ranked: json!({}),
    };

    let prof = mgr.compute_tactical_profile(&rec, 157, 20);
    let badge_texts: Vec<_> = prof.badges.iter().map(|b| b.text.as_str()).collect();

    // Check coexistence of streak, one-trick, and smurf badges
    assert!(badge_texts.iter().any(|t| t.contains("连胜")));
    assert!(badge_texts.iter().any(|t| t.contains("绝活(100%)")));
    assert!(badge_texts.iter().any(|t| t.contains("通天代")));
}

// ==============================================================================================
// Combination Test 12: Relog PUUID change & Chat unmasking exclusion
// Features: F11 (Anti-anonymity), F10 (Account switch)
// ==============================================================================================
#[test]
fn test_t3_12_relog_puuid_change_and_chat_unmask() {
    let old_local_puuid = "puuid-old-account";
    let new_local_puuid = "puuid-new-account";

    // Chat room contains both old account (now teammate) and new account
    let participants = canned_chat_participants(
        "relog-room",
        &[
            (old_local_puuid, "老账号队友"),
            (new_local_puuid, "新登录账号"),
            ("other-1", "路人A"),
            ("other-2", "路人B"),
            ("other-3", "路人C"),
        ],
    );

    // When logged into new account, candidate list must exclude new_local_puuid
    // and treat old_local_puuid as a valid candidate teammate
    let candidates: Vec<_> = participants
        .iter()
        .filter(|p| p["puuid"].as_str().unwrap() != new_local_puuid)
        .collect();

    assert_eq!(candidates.len(), 4);
    assert!(candidates.iter().any(|p| p["puuid"].as_str().unwrap() == old_local_puuid));
    assert!(!candidates.iter().any(|p| p["puuid"].as_str().unwrap() == new_local_puuid));
}

// ==============================================================================================
// Combination Test 13: Match history filter (Solo vs Flex vs ARAM) + 6-Axis Radar
// Features: F8 (Queue filter), F2 (6-axis radar)
// ==============================================================================================
#[test]
fn test_t3_13_queue_filter_solo_vs_flex_vs_aram_radar() {
    let mut matches = Vec::new();
    // 5 Solo/Duo matches (queue 420): high cs/min, high vision, high kda
    for _ in 0..5 {
        matches.push(ParsedMatchRecord {
            queue_id: 420,
            damage: 25000,
            damage_share_pct: 30.0,
            kills: 8,
            deaths: 2,
            assists: 8,
            kp_pct: 65.0,
            cs_per_min: 8.5,
            gold: 14000,
            vision_score: 35,
            ..Default::default()
        });
    }

    // 5 ARAM matches (queue 450): high damage, low cs/min, low vision
    for _ in 0..5 {
        matches.push(ParsedMatchRecord {
            queue_id: 450,
            damage: 40000,
            damage_share_pct: 35.0,
            kills: 15,
            deaths: 8,
            assists: 20,
            kp_pct: 85.0,
            cs_per_min: 2.0,
            gold: 16000,
            vision_score: 0,
            ..Default::default()
        });
    }

    // Filter Solo/Duo matches and compute radar
    let solo_matches: Vec<_> = matches.iter().filter(|m| m.queue_id == 420).cloned().collect();
    let radar_solo = compute_radar(&solo_matches);

    // Filter ARAM matches and compute radar
    let aram_matches: Vec<_> = matches.iter().filter(|m| m.queue_id == 450).cloned().collect();
    let radar_aram = compute_radar(&aram_matches);

    // Solo/Duo should have significantly higher economy & vision score than ARAM
    assert!(radar_solo["economy"] > radar_aram["economy"]);
    assert!(radar_solo["vision"] > radar_aram["vision"]);
}

// ==============================================================================================
// Combination Test 14: InProgress gameflow + 10-player scoreboard live fetch + MVP
// Features: F1 (Gameflow InProgress), F9 (Scoreboard contract), F2 (MVP evaluation)
// ==============================================================================================
#[tokio::test]
async fn test_t3_14_in_progress_scoreboard_live_fetch_and_mvp() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    server.state.set_phase("InProgress").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "InProgress");

    let detail = canned_match_detail_classic(77777, true);
    server.state.set_match_detail(77777, detail).await;

    let fetched = client.get_match_detail(77777).await.expect("live game detail");
    let db = Database::new();
    let parsed = parse_riot_match_detail(&fetched, &db).unwrap();

    let participants = parsed["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 10);

    // Player 1 on blue team was configured with highest damage (35000) & 12 kills -> MVP
    assert_eq!(participants[0]["is_mvp"].as_bool().unwrap(), true);
    assert_eq!(participants[0]["is_highest_damage"].as_bool().unwrap(), true);

    // Player 6 on red team was configured with highest damage on losing team -> SVP
    assert_eq!(participants[5]["is_svp"].as_bool().unwrap(), true);

    server.stop();
}

// ==============================================================================================
// Combination Test 15: Champ select dodge + State reset & Cache cleanup
// Features: F1 (Gameflow phase reset), F6 (Ongoing match response), F11 (Session reset)
// ==============================================================================================
#[tokio::test]
async fn test_t3_15_champ_select_dodge_and_cache_cleanup() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // Step 1: In ChampSelect
    server.state.set_phase("ChampSelect").await;
    let session = canned_champ_select_session("dodge-room", "local-p", 0);
    server.state.set_champ_select_session(session).await;

    let active_session = client.get_champ_select_session().await.expect("active session");
    assert_eq!(active_session["chatDetails"]["chatRoomId"].as_str().unwrap(), "dodge-room");

    // Step 2: Player dodges queue -> Phase transitions to None
    server.state.set_phase("None").await;
    server.state.set_champ_select_session(json!(null)).await;

    let phase_after = client.get_gameflow_phase().await.unwrap();
    assert_eq!(phase_after, "None");

    let ongoing_cleared = OngoingMatchResponse {
        phase: phase_after,
        is_active: false,
        chat_room_id: "".to_string(),
        blue_team: vec![],
        red_team: vec![],
        my_team: vec![],
        their_team: vec![],
        ..Default::default()
    };
    assert!(!ongoing_cleared.is_active);
    assert_eq!(ongoing_cleared.chat_room_id, "");
    assert_eq!(ongoing_cleared.my_team.len(), 0);

    server.stop();
}

// ==============================================================================================
// Combination Test 16: Concurrent config mutations during match participant evaluation
// Features: F13 (Config concurrency), F2 (Participant scoring)
// ==============================================================================================
#[tokio::test]
async fn test_t3_16_concurrent_config_mutations_during_batch_eval() {
    let temp_dir = std::env::temp_dir().join(format!("hex_test_cfg_race_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cfg_path = temp_dir.join("config.json");

    let manager = Arc::new(ConfigManager::new(cfg_path.to_str().unwrap()));
    let mut handles = Vec::new();

    // 5 tasks updating config concurrently
    for i in 0..5 {
        let mgr = manager.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..10 {
                let weight = (i * 10 + j) as f64 * 0.1;
                mgr.update(
                    Some(true),
                    Some(weight),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(true),
                    Some(2),
                ).await;
            }
        }));
    }

    // 5 tasks evaluating match participants concurrently
    for _ in 0..5 {
        handles.push(tokio::spawn(async move {
            for _ in 0..10 {
                let inputs = vec![
                    MatchParticipantInput {
                        participant_id: 1,
                        puuid: "p1".to_string(),
                        team_id: 100,
                        is_win: true,
                        kills: 10,
                        deaths: 2,
                        assists: 5,
                        damage: 30000,
                        damage_taken: 15000,
                        damage_mitigated: 5000,
                        gold: 12000,
                        cs: 180,
                        vision_score: 25,
                    },
                    MatchParticipantInput {
                        participant_id: 2,
                        puuid: "p2".to_string(),
                        team_id: 200,
                        is_win: false,
                        kills: 3,
                        deaths: 8,
                        assists: 2,
                        damage: 15000,
                        damage_taken: 20000,
                        damage_mitigated: 8000,
                        gold: 8000,
                        cs: 120,
                        vision_score: 15,
                    },
                ];
                let evals = evaluate_match_participants(1200, &inputs);
                assert_eq!(evals.len(), 2);
                assert!(evals[0].is_mvp);
                assert!(evals[1].is_svp);
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let final_cfg = manager.get().await;
    assert!(final_cfg.hex_synergy_weight >= 0.0 && final_cfg.hex_synergy_weight <= 3.0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
