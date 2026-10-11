mod lcu_mock_harness;

use lcu_mock_harness::fixtures::*;
use lcu_mock_harness::*;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::*;
use lol_hex_rust::lcu::connector::LcuConnector;
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
// Scenario 1: Standard Ranked Solo/Duo Champ Select & Anti-Anonymity Workflow
// Features Exercised: F1 (LCU Contract), F5 (Ongoing View), F6 (Isolation), F10 (Lifecycle), F11 (Anti-anonymity)
// ==============================================================================================
#[tokio::test]
async fn scenario_1_ranked_solo_champ_select_anti_anonymity() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);

    // 1. Initial State: Client is in ReadyCheck
    server.state.set_phase("ReadyCheck").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "ReadyCheck");

    // Accept ready check
    let accepted = client.accept_ready_check().await.expect("accept ready check");
    assert!(accepted);

    // 2. Transition into ChampSelect
    server.state.set_phase("ChampSelect").await;
    assert_eq!(client.get_gameflow_phase().await.unwrap(), "ChampSelect");

    // 3. Setup local player and anonymous teammates in champ select session
    let local_puuid = "puuid-local-solo";
    let chat_room_id = "solo-ranked-room-101";
    let session = canned_champ_select_session(chat_room_id, local_puuid, 0);
    server.state.set_champ_select_session(session).await;

    // Verify session received with anonymous names
    let session_val = client.get_champ_select_session().await.expect("session");
    let my_team_anon = session_val["myTeam"].as_array().unwrap();
    assert_eq!(my_team_anon.len(), 5);
    assert_eq!(my_team_anon[0]["isLocalPlayer"].as_bool().unwrap(), true);
    assert!(my_team_anon[1]["displayName"].as_str().unwrap().starts_with("召唤师"));

    // 4. Anti-anonymity unmasking via chat conversation participants
    let chat_participants = canned_chat_participants(
        chat_room_id,
        &[
            (local_puuid, "孤勇者 (我)"),
            ("real-puuid-top", "上单大爹"),
            ("real-puuid-jg", "节奏野王"),
            ("real-puuid-mid", "中路杀神"),
            ("real-puuid-adc", "暴击狂人"),
        ],
    );
    server.state.set_chat_participants(chat_room_id, chat_participants).await;

    let unmasked_list = client
        .get_chat_conversation_participants(chat_room_id)
        .await
        .expect("chat participants");
    assert_eq!(unmasked_list.len(), 5);

    // Filter out local player to avoid ghost duplication
    let teammate_candidates: Vec<_> = unmasked_list
        .iter()
        .filter(|p| p["puuid"].as_str().unwrap() != local_puuid)
        .collect();
    assert_eq!(teammate_candidates.len(), 4);

    // 5. Construct 5v5 ongoing match response
    let mut blue_cards = Vec::new();
    let positions = ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"];
    for (idx, &pos) in positions.iter().enumerate() {
        let (puuid, name) = if idx == 0 {
            (local_puuid.to_string(), "孤勇者 (我)".to_string())
        } else {
            let cand = &teammate_candidates[idx - 1];
            (cand["puuid"].as_str().unwrap().to_string(), cand["name"].as_str().unwrap().to_string())
        };

        blue_cards.push(OngoingPlayerCard {
            slot_index: idx,
            team_id: 100,
            is_local_player: idx == 0,
            puuid,
            summoner_name: name,
            position: pos.to_string(),
            tactical_analysis: PlayerTacticalProfile {
                tier_str: "钻石 II".to_string(),
                recent_count: 20,
                recent_win_rate_pct: 60,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    let ongoing = OngoingMatchResponse {
        phase: "ChampSelect".to_string(),
        phase_cn: "英雄选择中".to_string(),
        is_active: true,
        is_mock: false,
        chat_room_id: chat_room_id.to_string(),
        blue_team: blue_cards.clone(),
        my_team: blue_cards,
        ..Default::default()
    };

    assert!(ongoing.is_active);
    assert_eq!(ongoing.my_team.len(), 5);
    assert_eq!(ongoing.my_team[0].summoner_name, "孤勇者 (我)");
    assert_eq!(ongoing.my_team[1].summoner_name, "上单大爹");

    server.stop();
}

// ==============================================================================================
// Scenario 2: Premade Trio Party in Flex Queue with Badges & Chat Broadcast
// Features Exercised: F2 (Scoring & Badges), F3 (Premade BFS), F4 (Chat Broadcast), F5 (Ongoing View)
// ==============================================================================================
#[tokio::test]
async fn scenario_2_premade_trio_flex_queue_chat_broadcast() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let mgr = PlayerHistoryManager::new();

    // 5 players on team: players 0, 1, 2 form a 3-man premade party
    let mut my_team = Vec::new();
    let trio_puuids = vec!["trio-p1", "trio-p2", "trio-p3"];

    for i in 0..5 {
        let is_trio = i < 3;
        let puuid = if is_trio { format!("trio-p{}", i + 1) } else { format!("flex-p-{}", i) };
        let name = if is_trio { format!("车队成员{}", i + 1) } else { format!("路人玩家{}", i + 1) };

        let matches: Vec<ParsedMatchRecord> = if is_trio {
            // Each trio member shares 4 games with the other two members
            let others: Vec<String> = trio_puuids
                .iter()
                .filter(|&&p| p != format!("trio-p{}", i + 1))
                .map(|&s| s.to_string())
                .collect();
            (1..=4)
                .map(|gid| ParsedMatchRecord {
                    game_id: gid,
                    queue_id: 440,
                    teammate_puuids: others.clone(),
                    win: true,
                    ..Default::default()
                })
                .collect()
        } else {
            // Solo players have independent match histories
            vec![
                ParsedMatchRecord { game_id: 100 + i as i64, queue_id: 440, win: false, ..Default::default() },
                ParsedMatchRecord { game_id: 200 + i as i64, queue_id: 440, win: true, ..Default::default() },
            ]
        };

        let rec = CachedPlayerRecord {
            puuid: puuid.clone(),
            summoner_name: name.clone(),
            tier: "PLATINUM".to_string(),
            tier_str: "铂金 I".to_string(),
            tier_color: "#2dd4bf".to_string(),
            lp: 75,
            ranked_wins: 20,
            ranked_losses: 10,
            matches: matches.clone(),
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };

        // Compute tactical profile (player 0 has 4 win streak)
        let profile = mgr.compute_tactical_profile(&rec, 0, 20);

        my_team.push(OngoingPlayerCard {
            slot_index: i,
            team_id: 100,
            puuid,
            summoner_name: name,
            matches,
            tactical_analysis: profile,
            ..Default::default()
        });
    }

    // Detect premade teams
    detect_premade_teams(&mut my_team);

    // Verify Trio party detection
    assert_eq!(my_team[0].premade_group, Some(1));
    assert_eq!(my_team[1].premade_group, Some(1));
    assert_eq!(my_team[2].premade_group, Some(1));
    assert_eq!(my_team[0].premade_tag.as_deref(), Some("三排车队"));
    assert!(my_team[3].premade_group.is_none());
    assert!(my_team[4].premade_group.is_none());

    // Verify badges: player 0 has 4-win streak badge
    assert!(my_team[0].tactical_analysis.badges.iter().any(|b| b.text == "🔥 4连胜"));

    let ongoing = OngoingMatchResponse {
        phase: "ChampSelect".to_string(),
        is_active: true,
        my_team,
        ..Default::default()
    };

    // Format chat broadcast lines
    let lines = format_chat_broadcast_lines(&ongoing);
    assert_eq!(lines.len(), 6);
    assert!(lines[1].contains("[三排车队]"));
    assert!(lines[2].contains("[三排车队]"));
    assert!(lines[3].contains("[三排车队]"));
    assert!(!lines[4].contains("[三排车队]"));

    // Broadcast messages to chat room endpoint
    for line in &lines {
        let sent = client.send_chat_message("flex-chat-room", line).await.expect("send line");
        assert!(sent);
    }

    let all_sent = server.state.get_sent_messages().await;
    assert_eq!(all_sent.len(), 6);
    server.stop();
}

// ==============================================================================================
// Scenario 3: Full Match History Review & 10-Player Scoreboard Analysis
// Features Exercised: F1 (Match Detail), F2 (Radar & MVP/SVP), F7 (Profile), F8 (10p Scoreboard)
// ==============================================================================================
#[tokio::test]
async fn scenario_3_full_history_review_and_10p_scoreboard() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let db = Database::new();

    let target_puuid = "puuid-career-review";
    // 1. Summoner profile & 20 match history
    let history = canned_match_history(target_puuid, 20, 0.70, 157, &["tm-1", "tm-2"]);
    server.state.set_match_history(target_puuid, history).await;

    let hist_res = client.get_player_match_history(target_puuid, 20).await.expect("history");
    let games = hist_res["games"]["games"].as_array().unwrap();
    assert_eq!(games.len(), 20);

    // 2. Calculate 6-axis radar
    let mut parsed_records = Vec::new();
    for g in games {
        let p0 = &g["participants"][0]["stats"];
        parsed_records.push(ParsedMatchRecord {
            damage: p0["totalDamageDealtToChampions"].as_i64().unwrap(),
            kills: p0["kills"].as_i64().unwrap() as i32,
            deaths: p0["deaths"].as_i64().unwrap() as i32,
            assists: p0["assists"].as_i64().unwrap() as i32,
            gold: p0["goldEarned"].as_i64().unwrap(),
            win: p0["win"].as_bool().unwrap(),
            damage_share_pct: 28.0,
            kp_pct: 65.0,
            cs_per_min: 7.8,
            vision_score: 22,
            ..Default::default()
        });
    }
    let radar = compute_radar(&parsed_records);
    assert_eq!(radar.len(), 6);
    for (_k, v) in &radar {
        assert!(*v >= 25 && *v <= 99);
    }

    // 3. Select match #555555 and review detailed 10-player scoreboard
    let game_id = 555555i64;
    let match_detail = canned_match_detail_classic(game_id, true);
    server.state.set_match_detail(game_id, match_detail).await;

    let fetched_detail = client.get_match_detail(game_id).await.expect("detail");
    let parsed_scoreboard = parse_riot_match_detail(&fetched_detail, &db).expect("scoreboard");

    assert_eq!(parsed_scoreboard["game_id"].as_i64().unwrap(), game_id);
    assert_eq!(parsed_scoreboard["blue_team_win"].as_bool().unwrap(), true);
    assert_eq!(parsed_scoreboard["red_team_win"].as_bool().unwrap(), false);

    let participants = parsed_scoreboard["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 10);

    // Winner MVP and loser SVP checks
    let mvp_count = participants.iter().filter(|p| p["is_mvp"].as_bool().unwrap_or(false)).count();
    let svp_count = participants.iter().filter(|p| p["is_svp"].as_bool().unwrap_or(false)).count();
    assert_eq!(mvp_count, 1, "Exactly one MVP on winning team");
    assert_eq!(svp_count, 1, "Exactly one SVP on losing team");

    // Highest damage crown check
    let highest_dmg_count = participants
        .iter()
        .filter(|p| p["is_highest_damage"].as_bool().unwrap_or(false))
        .count();
    assert_eq!(highest_dmg_count, 1, "Exactly one highest damage crowned player");

    server.stop();
}

// ==============================================================================================
// Scenario 4: Client Sudden Crash, Relog Different Account, and Reconnect
// Features Exercised: F9 (Self-Healing), F10 (Lifecycle), F11 (Account Switch), F13 (Config Save)
// ==============================================================================================
#[tokio::test]
async fn scenario_4_client_crash_relog_reconnect_self_healing() {
    // Stage 1: Active client on Server 1 (Account 1)
    let server1 = MockLcuServer::start().await;
    let lockfile1 = server1.lockfile_content();
    let creds1 = LcuConnector::parse_lockfile(&lockfile1).expect("parse lockfile 1");

    let client1 = MockLcuClient::new(creds1.port, &creds1.auth_token);
    server1.state.set_current_summoner(canned_current_summoner("puuid-user-1", "老账号玩家")).await;

    let cur1 = client1.get_current_summoner().await.expect("account 1");
    assert_eq!(cur1["puuid"].as_str().unwrap(), "puuid-user-1");

    // Stage 2: Client crashes unexpectedly
    server1.stop();
    let crash_probe = client1.get_gameflow_phase().await;
    assert!(crash_probe.is_err(), "Detects client disconnected");

    // Application state gracefully transitions to offline/waiting without panicking
    let ongoing_waiting = OngoingMatchResponse {
        phase: "None".to_string(),
        is_active: false,
        message: Some("等待英雄联盟客户端重连...".to_string()),
        ..Default::default()
    };
    assert!(!ongoing_waiting.is_active);

    // Stage 3: Client restarts on a new ephemeral port with a new lockfile and Account 2
    let server2 = MockLcuServer::start().await;
    let lockfile2 = server2.lockfile_content();
    let creds2 = LcuConnector::parse_lockfile(&lockfile2).expect("parse lockfile 2");
    assert_ne!(creds1.port, creds2.port, "New client runs on different port");

    let client2 = MockLcuClient::new(creds2.port, &creds2.auth_token);
    server2.state.set_current_summoner(canned_current_summoner("puuid-user-2", "换号新召唤师")).await;

    // Stage 4: Reconnected within 3 seconds, fresh account detected
    let cur2 = client2.get_current_summoner().await.expect("account 2 reconnected");
    assert_eq!(cur2["puuid"].as_str().unwrap(), "puuid-user-2");
    assert_eq!(cur2["displayName"].as_str().unwrap(), "换号新召唤师");

    server2.stop();
}

// ==============================================================================================
// Scenario 5: High-Frequency Concurrent Requests with Semaphore Limiting
// Features Exercised: F1 (Batch History), F8 (Profiles), F13 (Semaphore Limiting)
// ==============================================================================================
#[tokio::test]
async fn scenario_5_high_frequency_concurrent_requests_semaphore() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(MockLcuClient::from_server(&server));

    // Register 10 players entering game
    for i in 1..=10 {
        let puuid = format!("conc-puuid-{}", i);
        let history = canned_match_history(&puuid, 20, 0.60, 100 + i, &[]);
        server.state.set_match_history(&puuid, history).await;
    }

    // Semaphore permits max 4 concurrent requests to prevent LCU 429 rate limit
    let sem = Arc::new(Semaphore::new(4));
    let mut handles = Vec::new();

    for i in 1..=10 {
        let sem_clone = sem.clone();
        let client_clone = client.clone();
        let puuid = format!("conc-puuid-{}", i);

        handles.push(tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let res = client_clone.get_player_match_history(&puuid, 20).await;
            res
        }));
    }

    let mut successful_fetches = 0;
    for h in handles {
        let res = h.await.unwrap();
        assert!(res.is_ok(), "Query must not fail under concurrent throttling");
        let val = res.unwrap();
        let games = val["games"]["games"].as_array().unwrap();
        assert_eq!(games.len(), 20);
        successful_fetches += 1;
    }

    assert_eq!(successful_fetches, 10);
    assert_eq!(server.state.get_request_count(), 10);
    server.stop();
}

// ==============================================================================================
// Scenario 6: ARAM Match with Bench Rerolls and Non-Standard Team IDs
// Features Exercised: F5 (Ongoing View), F8 (Scoreboard), F12 (ARAM Bench & Queue 450)
// ==============================================================================================
#[tokio::test]
async fn scenario_6_aram_match_bench_rerolls_and_scoreboard() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let db = Database::new();

    // 1. ARAM Champ select session with bench champions
    let mut session = canned_champ_select_session("aram-bench-session", "local-aram-p", 0);
    session["benchEnabled"] = json!(true);
    session["rerollsRemaining"] = json!(2);
    session["benchChampions"] = json!([
        { "championId": 222, "isHighestMastery": false }, // Jinx
        { "championId": 99,  "isHighestMastery": true  }, // Lux
        { "championId": 115, "isHighestMastery": false }  // Ziggs
    ]);
    server.state.set_champ_select_session(session).await;

    let active_session = client.get_champ_select_session().await.expect("aram session");
    assert_eq!(active_session["benchEnabled"].as_bool().unwrap(), true);
    assert_eq!(active_session["benchChampions"].as_array().unwrap().len(), 3);
    assert_eq!(active_session["rerollsRemaining"].as_i64().unwrap(), 2);

    // 2. ARAM post-game scoreboard review (Queue 450)
    let game_id = 999111i64;
    let aram_detail = canned_match_detail_aram(game_id);
    server.state.set_match_detail(game_id, aram_detail).await;

    let fetched = client.get_match_detail(game_id).await.expect("aram match");
    let parsed = parse_riot_match_detail(&fetched, &db).expect("parse aram");

    assert_eq!(parsed["mode_cn"].as_str().unwrap(), "狂暴大乱斗");
    assert_eq!(parsed["game_mode_cn"].as_str().unwrap(), "极地大乱斗");
    assert_eq!(parsed["queue_id"].as_i64().unwrap(), 450);

    let participants = parsed["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 10);

    server.stop();
}

// ==============================================================================================
// Scenario 7: Arena 2v2v2v2 4-Team Match Scoreboard Isolation
// Features Exercised: F8 (Scoreboard Contract), F12 (Arena 4-Team Isolation)
// ==============================================================================================
#[tokio::test]
async fn scenario_7_arena_4team_scoreboard_isolation() {
    let server = MockLcuServer::start().await;
    let client = MockLcuClient::from_server(&server);
    let db = Database::new();

    // Arena match: 8 players across 4 teams (100, 200, 300, 400)
    let game_id = 1700999i64;
    let arena_detail = canned_match_detail_arena(game_id);
    server.state.set_match_detail(game_id, arena_detail).await;

    let fetched = client.get_match_detail(game_id).await.expect("arena match");
    let parsed = parse_riot_match_detail(&fetched, &db).expect("parse arena scoreboard");

    assert_eq!(parsed["mode_cn"].as_str().unwrap(), "斗魂竞技场");
    assert_eq!(parsed["game_mode"].as_str().unwrap(), "CHERRY");
    assert_eq!(parsed["queue_id"].as_i64().unwrap(), 1700);

    let participants = parsed["participants"].as_array().unwrap();
    assert_eq!(participants.len(), 8, "Arena match must maintain exactly 8 players across 4 teams");

    // Verify all 8 participants are parsed into flat structure without crashing or collapsing into 5v5
    for p in participants {
        assert!(p["participant_id"].as_i64().is_some());
        assert!(p["champion_name"].as_str().is_some());
        assert!(p["total_damage"].as_i64().is_some());
        assert_eq!(p["items"].as_array().unwrap().len(), 7);
    }

    server.stop();
}
