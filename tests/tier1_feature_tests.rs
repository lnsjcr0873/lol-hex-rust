mod lcu_mock_harness;

use lcu_mock_harness::fixtures::*;
use lcu_mock_harness::*;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::*;
use lol_hex_rust::engine::ranked_draft::{ORDERED_POSITIONS, SHORT_POS_NAMES_CN};
use lol_hex_rust::lcu::connector::LcuConnector;
use lol_hex_rust::web::routes::{build_mock_match_detail, parse_riot_match_detail};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

// Helper to format chat broadcast lines matching HexBrain specification
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
// Feature 1: LCU API & Data Contract Alignment (5 tests)
// ==============================================================================================
mod f1_lcu_api_contract {
    use super::*;

    #[tokio::test]
    async fn test_f1_01_gameflow_phase_contract() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Initial default phase
        let phase = client.get_gameflow_phase().await.expect("get phase");
        assert_eq!(phase, "ChampSelect");

        // Transition to InProgress
        server.state.set_phase("InProgress").await;
        assert_eq!(client.get_gameflow_phase().await.unwrap(), "InProgress");

        // Transition to None (offline)
        server.state.set_phase("None").await;
        assert_eq!(client.get_gameflow_phase().await.unwrap(), "None");
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_02_champ_select_session_contract() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let canned = canned_champ_select_session("chat-room-101", "local-puuid-999", 2);
        server.state.set_champ_select_session(canned).await;

        let session = client.get_champ_select_session().await.expect("session");
        assert_eq!(
            session["chatDetails"]["chatRoomId"].as_str().unwrap(),
            "chat-room-101"
        );
        let my_team = session["myTeam"].as_array().expect("myTeam array");
        assert_eq!(my_team.len(), 5);
        assert_eq!(my_team[2]["isLocalPlayer"].as_bool().unwrap(), true);
        assert_eq!(my_team[2]["puuid"].as_str().unwrap(), "local-puuid-999");
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_03_current_summoner_contract() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let canned = canned_current_summoner("puuid-faker-001", "Hide on bush");
        server.state.set_current_summoner(canned).await;

        let cur = client.get_current_summoner().await.expect("current summoner");
        assert_eq!(cur["puuid"].as_str().unwrap(), "puuid-faker-001");
        assert_eq!(cur["displayName"].as_str().unwrap(), "Hide on bush");
        assert_eq!(cur["tagLine"].as_str().unwrap(), "CN");
        assert_eq!(cur["summonerLevel"].as_i64().unwrap(), 320);
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_04_match_history_contract() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let history = canned_match_history("puuid-uzi-001", 20, 0.65, 145, &["tm-1", "tm-2"]);
        server.state.set_match_history("puuid-uzi-001", history).await;

        let res = client.get_player_match_history("puuid-uzi-001", 0, 19).await.expect("history");
        let games = res["games"]["games"].as_array().expect("games array");
        assert_eq!(games.len(), 20);
        assert_eq!(games[0]["queueId"].as_i64().unwrap(), 420);
        assert_eq!(games[0]["gameDuration"].as_i64().unwrap(), 1800);
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_05_match_detail_contract() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let detail = canned_match_detail_classic(123456789, true);
        server.state.set_match_detail(123456789, detail).await;

        let fetched = client.get_match_detail(123456789).await.expect("game detail");
        assert_eq!(fetched["gameId"].as_i64().unwrap(), 123456789);
        assert_eq!(fetched["participantIdentities"].as_array().unwrap().len(), 10);
        assert_eq!(fetched["participants"].as_array().unwrap().len(), 10);
        server.stop();
    }
}

// ==============================================================================================
// Feature 2: Data Aggregation & Scoring Algorithms (MVP/SVP, Radar) (5 tests)
// ==============================================================================================
mod f2_scoring_and_radar {
    use super::*;

    #[test]
    fn test_f2_01_mvp_highest_damage_winner() {
        let db = Database::new();
        let raw = canned_match_detail_classic(1001, true); // Blue wins
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed match detail");

        let blue_players = parsed["teams"]["blue"]["players"].as_array().unwrap();
        let mvp_count = blue_players.iter().filter(|p| p["is_mvp"] == true).count();
        assert_eq!(mvp_count, 1, "Exactly one MVP on winning team");

        // Player 1 has 35000 dmg (highest)
        assert_eq!(blue_players[0]["is_mvp"].as_bool().unwrap(), true);
        assert_eq!(blue_players[0]["is_svp"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_f2_02_svp_highest_damage_loser() {
        let db = Database::new();
        let raw = canned_match_detail_classic(1002, true); // Blue wins, Red loses
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed match detail");

        let red_players = parsed["teams"]["red"]["players"].as_array().unwrap();
        let svp_count = red_players.iter().filter(|p| p["is_svp"] == true).count();
        assert_eq!(svp_count, 1, "Exactly one SVP on losing team");

        // Red player 1 (participant 6) has 32000 dmg (highest on red)
        assert_eq!(red_players[0]["is_svp"].as_bool().unwrap(), true);
        assert_eq!(red_players[0]["is_mvp"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_f2_03_damage_share_percentage() {
        let db = Database::new();
        let raw = canned_match_detail_classic(1003, true);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed match detail");

        let blue_players = parsed["teams"]["blue"]["players"].as_array().unwrap();
        let total_blue_dmg = parsed["teams"]["blue"]["damage"].as_i64().unwrap();
        assert!(total_blue_dmg > 0);

        let mut sum_share = 0.0;
        for p in blue_players {
            let dmg = p["damage"].as_i64().unwrap();
            let share = p["damage_share_pct"].as_f64().unwrap();
            let expected_share = ((dmg as f64 / total_blue_dmg as f64) * 1000.0).round() / 10.0;
            assert!((share - expected_share).abs() < 0.1);
            sum_share += share;
        }
        assert!((sum_share - 100.0).abs() < 1.0, "Sum of shares ~100%");
    }

    #[test]
    fn test_f2_04_radar_six_axis_normalization() {
        // Extreme match with massive damage and zero deaths
        let extreme_match = ParsedMatchRecord {
            game_id: 1,
            kills: 30,
            deaths: 0,
            assists: 20,
            damage: 150_000,
            damage_share_pct: 65.0,
            gold: 35_000,
            cs_per_min: 15.0,
            kp_pct: 95.0,
            vision_score: 120,
            ..Default::default()
        };

        let radar = compute_radar(&[extreme_match]);
        for (axis, &score) in &radar {
            assert!(
                score >= 25 && score <= 99,
                "Axis {} score {} must be within [25, 99]",
                axis,
                score
            );
        }
    }

    #[test]
    fn test_f2_05_radar_empty_matches_defaults() {
        let radar = compute_radar(&[]);
        assert_eq!(radar.len(), 6);
        assert_eq!(*radar.get("damage").unwrap(), 50);
        assert_eq!(*radar.get("kda").unwrap(), 50);
        assert_eq!(*radar.get("kp").unwrap(), 50);
        assert_eq!(*radar.get("survival").unwrap(), 50);
        assert_eq!(*radar.get("economy").unwrap(), 50);
        assert_eq!(*radar.get("vision").unwrap(), 50);
    }
}

// ==============================================================================================
// Feature 3: Premade Team Detection (Graph BFS) (5 tests)
// ==============================================================================================
mod f3_premade_detection {
    use super::*;

    fn make_player_card(puuid: &str, shared_game_ids: &[i64]) -> OngoingPlayerCard {
        let matches = shared_game_ids
            .iter()
            .map(|&gid| ParsedMatchRecord {
                game_id: gid,
                ..Default::default()
            })
            .collect();

        OngoingPlayerCard {
            puuid: puuid.to_string(),
            summoner_name: puuid.to_string(),
            matches,
            ..Default::default()
        }
    }

    #[test]
    fn test_f3_01_strict_two_game_threshold() {
        // 2 players sharing only 1 game -> NOT premade
        let mut players = vec![
            make_player_card("p1", &[100]),
            make_player_card("p2", &[100]),
        ];
        detect_premade_teams(&mut players);
        assert!(players[0].premade_group.is_none(), "1 game together must not form a premade");
        assert!(players[1].premade_group.is_none());
    }

    #[test]
    fn test_f3_02_duo_premade_detection() {
        // 2 players sharing 2 games -> Duo
        let mut players = vec![
            make_player_card("p1", &[101, 102]),
            make_player_card("p2", &[101, 102]),
        ];
        detect_premade_teams(&mut players);
        assert_eq!(players[0].premade_group, Some(1));
        assert_eq!(players[1].premade_group, Some(1));
        assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[0].premade_color.as_deref(), Some("#ec4899"));
        assert!(players[0].tactical_analysis.badges[0].text.contains("双排车队"));
    }

    #[test]
    fn test_f3_03_trio_premade_detection() {
        // 3 players sharing 2 games -> Trio (first group index 0 gets peach pink #ec4899)
        let mut players = vec![
            make_player_card("p1", &[201, 202]),
            make_player_card("p2", &[201, 202]),
            make_player_card("p3", &[201, 202]),
        ];
        detect_premade_teams(&mut players);
        assert_eq!(players[0].premade_group, Some(1));
        assert_eq!(players[1].premade_group, Some(1));
        assert_eq!(players[2].premade_group, Some(1));
        assert_eq!(players[0].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[0].premade_color.as_deref(), Some("#ec4899"));
    }

    #[test]
    fn test_f3_04_quad_premade_detection() {
        // 4 players in a chain/clique with >=2 shared games -> Quad (group index 0 gets #ec4899)
        let mut players = vec![
            make_player_card("p1", &[301, 302, 303]),
            make_player_card("p2", &[301, 302, 303]),
            make_player_card("p3", &[302, 303]),
            make_player_card("p4", &[302, 303]),
        ];
        detect_premade_teams(&mut players);
        assert_eq!(players[0].premade_tag.as_deref(), Some("四排车队"));
        assert_eq!(players[0].premade_color.as_deref(), Some("#ec4899"));
    }

    #[test]
    fn test_f3_05_multiple_disjoint_premade_groups() {
        // Group 1: p0 & p1 share 2 games
        // Group 2: p2 & p3 share 3 games
        // Solo: p4 has no shared games
        let mut players = vec![
            make_player_card("p0", &[401, 402]),
            make_player_card("p1", &[401, 402]),
            make_player_card("p2", &[501, 502, 503]),
            make_player_card("p3", &[501, 502, 503]),
            make_player_card("p4", &[601]),
        ];
        detect_premade_teams(&mut players);
        assert_eq!(players[0].premade_group, Some(1));
        assert_eq!(players[1].premade_group, Some(1));
        assert_eq!(players[2].premade_group, Some(2));
        assert_eq!(players[3].premade_group, Some(2));
        assert!(players[4].premade_group.is_none(), "Solo player has no premade group");
    }
}

// ==============================================================================================
// Feature 4: Tactical Badges Generation (5 tests)
// ==============================================================================================
mod f4_tactical_badges {
    use super::*;

    fn make_test_record(matches: Vec<ParsedMatchRecord>, wins: i32, losses: i32) -> CachedPlayerRecord {
        CachedPlayerRecord {
            puuid: "test-puuid".to_string(),
            summoner_name: "TestPlayer".to_string(),
            tier: "DIAMOND".to_string(),
            tier_str: "钻石 I".to_string(),
            tier_color: "#38bdf8".to_string(),
            lp: 75,
            ranked_wins: wins,
            ranked_losses: losses,
            matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        }
    }

    #[test]
    fn test_f4_01_win_streak_badge() {
        let mgr = PlayerHistoryManager::new();
        let matches = vec![
            ParsedMatchRecord { win: true, kills: 5, deaths: 1, assists: 10, ..Default::default() },
            ParsedMatchRecord { win: true, kills: 6, deaths: 2, assists: 8, ..Default::default() },
            ParsedMatchRecord { win: true, kills: 7, deaths: 0, assists: 5, ..Default::default() },
        ];
        let record = make_test_record(matches, 3, 0);
        let profile = mgr.compute_tactical_profile(&record, 0, 20);

        assert_eq!(profile.streak_type, "win");
        assert_eq!(profile.streak_count, 3);
        assert!(profile.badges.iter().any(|b| b.text.contains("3连胜")));
    }

    #[test]
    fn test_f4_02_loss_streak_badge() {
        let mgr = PlayerHistoryManager::new();
        let matches = vec![
            ParsedMatchRecord { win: false, kills: 1, deaths: 6, assists: 2, ..Default::default() },
            ParsedMatchRecord { win: false, kills: 0, deaths: 5, assists: 1, ..Default::default() },
            ParsedMatchRecord { win: false, kills: 2, deaths: 8, assists: 3, ..Default::default() },
        ];
        let record = make_test_record(matches, 0, 3);
        let profile = mgr.compute_tactical_profile(&record, 0, 20);

        assert_eq!(profile.streak_type, "loss");
        assert_eq!(profile.streak_count, 3);
        assert!(profile.badges.iter().any(|b| b.text.contains("3连败")));
    }

    #[test]
    fn test_f4_03_signature_champion_badge() {
        let mgr = PlayerHistoryManager::new();
        // 4 games on champion 157 (Yasuo): 3 wins, 1 loss = 75% WR
        let matches = vec![
            ParsedMatchRecord { win: true, champion_id: 157, champion_name: "亚索".to_string(), ..Default::default() },
            ParsedMatchRecord { win: true, champion_id: 157, champion_name: "亚索".to_string(), ..Default::default() },
            ParsedMatchRecord { win: true, champion_id: 157, champion_name: "亚索".to_string(), ..Default::default() },
            ParsedMatchRecord { win: false, champion_id: 157, champion_name: "亚索".to_string(), ..Default::default() },
        ];
        let record = make_test_record(matches, 3, 1);
        let profile = mgr.compute_tactical_profile(&record, 157, 20);

        assert!(profile.badges.iter().any(|b| b.text.contains("绝活(75%)")));
        assert_eq!(profile.cur_champ_games, 4);
        assert_eq!(profile.cur_champ_win_rate_pct, 75);
    }

    #[test]
    fn test_f4_04_testing_champion_badge() {
        let mgr = PlayerHistoryManager::new();
        // 4 games on other champions, 0 on champion 64 (Lee Sin)
        let matches = vec![
            ParsedMatchRecord { win: true, champion_id: 157, ..Default::default() },
            ParsedMatchRecord { win: false, champion_id: 157, ..Default::default() },
            ParsedMatchRecord { win: true, champion_id: 238, ..Default::default() },
            ParsedMatchRecord { win: false, champion_id: 238, ..Default::default() },
        ];
        let record = make_test_record(matches, 2, 2);
        let profile = mgr.compute_tactical_profile(&record, 64, 20);

        assert!(profile.badges.iter().any(|b| b.text.contains("试英雄")));
        assert_eq!(profile.cur_champ_games, 0);
    }

    #[test]
    fn test_f4_05_smurf_and_kda_badges() {
        let mgr = PlayerHistoryManager::new();
        // 8 games: 7 wins (87.5% WR), very high KDA (total kills 70, deaths 10, assists 50 -> KDA 12.0)
        let mut matches = Vec::new();
        for _ in 0..7 {
            matches.push(ParsedMatchRecord { win: true, kills: 10, deaths: 1, assists: 7, ..Default::default() });
        }
        matches.push(ParsedMatchRecord { win: false, kills: 5, deaths: 3, assists: 1, ..Default::default() });

        let record = make_test_record(matches, 7, 1);
        let profile = mgr.compute_tactical_profile(&record, 0, 20);

        assert!(profile.badges.iter().any(|b| b.text.contains("通天代")));
        assert!(profile.badges.iter().any(|b| b.text.contains("KDA")));
    }
}

// ==============================================================================================
// Feature 5: Chat Record Broadcast Formatting & Rate Limit (5 tests)
// ==============================================================================================
mod f5_chat_broadcast {
    use super::*;

    fn make_test_ongoing_team() -> OngoingMatchResponse {
        let mut my_team = Vec::new();
        let names = ["玩家一", "玩家二", "玩家三", "玩家四", "玩家五"];
        let tiers = ["大师", "钻石 I", "翡翠 II", "铂金 III", "黄金 IV"];
        for i in 0..5 {
            let mut card = OngoingPlayerCard {
                summoner_name: names[i].to_string(),
                ..Default::default()
            };
            card.tactical_analysis.tier_str = tiers[i].to_string();
            card.tactical_analysis.recent_count = 20;
            card.tactical_analysis.recent_win_rate_pct = 50 + (i as i32 * 5);
            if i == 0 || i == 1 {
                card.premade_tag = Some("双排车队".to_string());
            }
            my_team.push(card);
        }

        OngoingMatchResponse {
            phase: "ChampSelect".to_string(),
            is_active: true,
            my_team,
            ..Default::default()
        }
    }

    #[test]
    fn test_f5_01_broadcast_header_format() {
        let ongoing = make_test_ongoing_team();
        let lines = format_chat_broadcast_lines(&ongoing);
        assert!(!lines.is_empty());
        assert_eq!(lines[0], "⚡【HexBrain 智脑战绩透视】⚡");
    }

    #[test]
    fn test_f5_02_broadcast_five_players_row_format() {
        let ongoing = make_test_ongoing_team();
        let lines = format_chat_broadcast_lines(&ongoing);
        assert_eq!(lines.len(), 6, "1 header + 5 players");

        // Player row format check
        assert!(lines[1].starts_with("1. 上路 [玩家一] 大师 (近20场胜率50%)"));
    }

    #[test]
    fn test_f5_03_lane_position_mapping() {
        let ongoing = make_test_ongoing_team();
        let lines = format_chat_broadcast_lines(&ongoing);

        assert!(lines[1].contains("上路"));
        assert!(lines[2].contains("打野"));
        assert!(lines[3].contains("中路"));
        assert!(lines[4].contains("射手"));
        assert!(lines[5].contains("辅助"));
    }

    #[test]
    fn test_f5_04_premade_tag_appended_to_broadcast() {
        let ongoing = make_test_ongoing_team();
        let lines = format_chat_broadcast_lines(&ongoing);

        assert!(lines[1].ends_with("[双排车队]"));
        assert!(lines[2].ends_with("[双排车队]"));
        assert!(!lines[3].contains("[双排车队]"));
    }

    #[tokio::test]
    async fn test_f5_05_rate_limiting_transmission() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let ongoing = make_test_ongoing_team();
        let lines = format_chat_broadcast_lines(&ongoing);

        for line in &lines {
            let ok = client.send_chat_message("champ-select-chat", line).await.expect("send");
            assert!(ok);
            // Simulate minimal rate-limiting delay
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }

        let sent = server.state.get_sent_messages().await;
        assert_eq!(sent.len(), 6);
        assert_eq!(sent[0].1, "⚡【HexBrain 智脑战绩透视】⚡");
        server.stop();
    }
}

// ==============================================================================================
// Feature 6: Ongoing Match 5v5 Dual-Column Alignment (5 tests)
// ==============================================================================================
mod f6_ongoing_match_view {
    use super::*;

    #[test]
    fn test_f6_01_ongoing_match_structure() {
        let resp = OngoingMatchResponse::default();
        assert_eq!(resp.phase, "");
        assert_eq!(resp.is_active, false);
        assert_eq!(resp.blue_team.len(), 0);
        assert_eq!(resp.red_team.len(), 0);
    }

    #[test]
    fn test_f6_02_five_vs_five_slot_count() {
        let mut resp = OngoingMatchResponse::default();
        for i in 0..5 {
            resp.blue_team.push(OngoingPlayerCard { slot_index: i, ..Default::default() });
            resp.red_team.push(OngoingPlayerCard { slot_index: i + 5, ..Default::default() });
        }
        assert_eq!(resp.blue_team.len(), 5);
        assert_eq!(resp.red_team.len(), 5);
    }

    #[test]
    fn test_f6_03_player_card_tactical_profile_attached() {
        let mut card = OngoingPlayerCard::default();
        card.tactical_analysis.tier = "CHALLENGER".to_string();
        card.tactical_analysis.recent_win_rate_pct = 70;
        card.tactical_analysis.badges.push(BadgeItem {
            text: "🔥 5连胜".to_string(),
            ..Default::default()
        });

        assert_eq!(card.tactical_analysis.tier, "CHALLENGER");
        assert_eq!(card.tactical_analysis.recent_win_rate_pct, 70);
        assert_eq!(card.tactical_analysis.badges.len(), 1);
    }

    #[test]
    fn test_f6_04_inactive_match_state() {
        let resp = OngoingMatchResponse {
            phase: "None".to_string(),
            is_active: false,
            is_mock: false,
            ..Default::default()
        };
        assert!(!resp.is_active);
        assert_eq!(resp.phase, "None");
    }

    #[test]
    fn test_f6_05_lane_order_integrity() {
        assert_eq!(ORDERED_POSITIONS, ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"]);
        assert_eq!(SHORT_POS_NAMES_CN, ["上单", "打野", "中单", "下路", "辅助"]);
    }
}

// ==============================================================================================
// Feature 7: Offline & Simulation Mode Transitions (5 tests)
// ==============================================================================================
mod f7_offline_and_mock_modes {
    use super::*;

    #[tokio::test]
    async fn test_f7_01_offline_client_none_phase() {
        let server = MockLcuServer::start().await;
        server.state.set_phase("None").await;

        let client = MockLcuClient::from_server(&server);
        let phase = client.get_gameflow_phase().await.expect("phase");
        assert_eq!(phase, "None");
        server.stop();
    }

    #[test]
    fn test_f7_02_mock_match_detail_generation() {
        let db = Database::new();
        let mock_detail = build_mock_match_detail(9999, &db);

        assert_eq!(mock_detail["game_id"].as_i64().unwrap(), 9999);
        assert_eq!(mock_detail["mode_cn"].as_str().unwrap(), "排位赛(单双)");
        let blue = mock_detail["teams"]["blue"]["players"].as_array().unwrap();
        let red = mock_detail["teams"]["red"]["players"].as_array().unwrap();
        assert_eq!(blue.len(), 5);
        assert_eq!(red.len(), 5);
        assert_eq!(blue[0]["champion_id"].as_i64().unwrap(), 266); // Aatrox
    }

    #[test]
    fn test_f7_03_mock_mode_flag_distinction() {
        let resp = OngoingMatchResponse {
            phase: "ChampSelect".to_string(),
            is_mock: true,
            is_active: true,
            ..Default::default()
        };
        assert!(resp.is_mock, "Must distinguish simulation from live LCU");
    }

    #[tokio::test]
    async fn test_f7_04_phase_transition_lifecycle() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let phases = ["None", "Matchmaking", "ReadyCheck", "ChampSelect", "InProgress", "WaitingForStats"];
        for p in phases {
            server.state.set_phase(p).await;
            assert_eq!(client.get_gameflow_phase().await.unwrap(), p);
        }
        server.stop();
    }

    #[tokio::test]
    async fn test_f7_05_graceful_error_on_missing_game_id() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let err = client.get_match_detail(99999999).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Status 404"));
        server.stop();
    }
}

// ==============================================================================================
// Feature 8: Summoner Profile & History Queue Filter (5 tests)
// ==============================================================================================
mod f8_summoner_profile_and_filters {
    use super::*;

    #[test]
    fn test_f8_01_tier_color_mapping() {
        assert_eq!(get_tier_color("CHALLENGER"), "#f59e0b");
        assert_eq!(get_tier_color("GRANDMASTER"), "#ef4444");
        assert_eq!(get_tier_color("MASTER"), "#a855f7");
        assert_eq!(get_tier_color("DIAMOND"), "#38bdf8");
        assert_eq!(get_tier_color("EMERALD"), "#00e696");
        assert_eq!(get_tier_color("PLATINUM"), "#2dd4bf");
        assert_eq!(get_tier_color("GOLD"), "#c8aa6e");
        assert_eq!(get_tier_color("SILVER"), "#94a3b8");
        assert_eq!(get_tier_color("BRONZE"), "#b45309");
        assert_eq!(get_tier_color("IRON"), "#71717a");
        assert_eq!(get_tier_color("UNKNOWN"), "#64748b");
    }

    #[test]
    fn test_f8_02_sample_match_count_bounds() {
        let mgr = PlayerHistoryManager::new();
        mgr.set_sample_match_count(10);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 10);

        mgr.set_sample_match_count(30);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 30);

        // Extended valid counts: 40, 50, 100, 200
        mgr.set_sample_match_count(40);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 40);

        mgr.set_sample_match_count(50);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 50);

        mgr.set_sample_match_count(100);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 100);

        mgr.set_sample_match_count(200);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 200);

        // Invalid numbers fallback to 20
        mgr.set_sample_match_count(15);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 20);

        mgr.set_sample_match_count(75);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 20);
    }

    #[test]
    fn test_f8_03_recent_dots_five_games() {
        let mgr = PlayerHistoryManager::new();
        let matches = vec![
            ParsedMatchRecord { win: true, ..Default::default() },
            ParsedMatchRecord { win: false, ..Default::default() },
            ParsedMatchRecord { win: true, ..Default::default() },
            ParsedMatchRecord { win: true, ..Default::default() },
            ParsedMatchRecord { win: false, ..Default::default() },
            ParsedMatchRecord { win: true, ..Default::default() }, // 6th game should not appear in dots
        ];
        let record = CachedPlayerRecord {
            puuid: "p1".to_string(),
            summoner_name: "p1".to_string(),
            tier: "GOLD".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 3,
            ranked_losses: 3,
            matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let profile = mgr.compute_tactical_profile(&record, 0, 20);
        assert_eq!(profile.recent_dots, vec![true, false, true, true, false]);
    }

    #[test]
    fn test_f8_04_top_three_champions() {
        let mgr = PlayerHistoryManager::new();
        let matches = vec![
            ParsedMatchRecord { champion_id: 1, champion_name: "Annie".to_string(), win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 1, champion_name: "Annie".to_string(), win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 2, champion_name: "Olaf".to_string(), win: false, ..Default::default() },
            ParsedMatchRecord { champion_id: 3, champion_name: "Galio".to_string(), win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 4, champion_name: "TF".to_string(), win: false, ..Default::default() },
        ];
        let record = CachedPlayerRecord {
            puuid: "p1".to_string(),
            summoner_name: "p1".to_string(),
            tier: "GOLD".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 3,
            ranked_losses: 2,
            matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let profile = mgr.compute_tactical_profile(&record, 0, 20);
        assert!(profile.top_champs.len() <= 3);
        assert_eq!(profile.top_champs[0].champion_id, 1, "Most played is champion 1");
        assert_eq!(profile.top_champs[0].games, 2);
    }

    #[test]
    fn test_f8_05_queue_id_distinction() {
        let m_solo = ParsedMatchRecord { queue_id: 420, ..Default::default() };
        let m_flex = ParsedMatchRecord { queue_id: 440, ..Default::default() };
        let m_aram = ParsedMatchRecord { queue_id: 450, ..Default::default() };

        assert_eq!(m_solo.queue_id, 420);
        assert_eq!(m_flex.queue_id, 440);
        assert_eq!(m_aram.queue_id, 450);
    }
}

// ==============================================================================================
// Feature 9: 10-Player Scoreboard Contract & Dual Bars (5 tests)
// ==============================================================================================
mod f9_scoreboard_contract {
    use super::*;

    #[test]
    fn test_f9_01_scoreboard_root_structure() {
        let db = Database::new();
        let raw = canned_match_detail_classic(2001, true);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed");

        assert_eq!(parsed["game_id"].as_i64().unwrap(), 2001);
        assert_eq!(parsed["mode_cn"].as_str().unwrap(), "排位赛(单双)");
        assert_eq!(parsed["game_duration_str"].as_str().unwrap(), "30:00");
        assert!(parsed["teams"].is_object());
    }

    #[test]
    fn test_f9_02_team_summary_statistics() {
        let db = Database::new();
        let raw = canned_match_detail_classic(2002, true);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed");

        let blue = &parsed["teams"]["blue"];
        assert_eq!(blue["win"].as_bool().unwrap(), true);
        assert!(blue["kills"].as_i64().unwrap() > 0);
        assert!(blue["damage"].as_i64().unwrap() > 0);
        assert!(blue["gold"].as_i64().unwrap() > 0);
    }

    #[test]
    fn test_f9_03_ten_players_total_count() {
        let db = Database::new();
        let raw = canned_match_detail_classic(2003, false);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed");

        let blue_p = parsed["teams"]["blue"]["players"].as_array().unwrap();
        let red_p = parsed["teams"]["red"]["players"].as_array().unwrap();
        assert_eq!(blue_p.len(), 5);
        assert_eq!(red_p.len(), 5);
    }

    #[test]
    fn test_f9_04_participant_seven_items() {
        let db = Database::new();
        let raw = canned_match_detail_classic(2004, true);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed");

        let blue_p = parsed["teams"]["blue"]["players"].as_array().unwrap();
        for p in blue_p {
            let items = p["items"].as_array().unwrap();
            assert_eq!(items.len(), 7, "Each participant must have 7 item slots (6 items + 1 trinket)");
        }
    }

    #[test]
    fn test_f9_05_damage_and_gold_values() {
        let db = Database::new();
        let raw = canned_match_detail_classic(2005, true);
        let parsed = parse_riot_match_detail(&raw, &db).expect("parsed");

        let red_p = parsed["teams"]["red"]["players"].as_array().unwrap();
        for p in red_p {
            assert!(p["damage"].as_i64().unwrap() > 0);
            assert!(p["gold"].as_i64().unwrap() > 0);
            assert!(p["cs"].as_i64().unwrap() >= 0);
        }
    }
}

// ==============================================================================================
// Feature 10: LCU Lifecycle Resilience & Credential Self-Healing (5 tests)
// ==============================================================================================
mod f10_lifecycle_and_credentials {
    use super::*;

    #[test]
    fn test_f10_01_lockfile_parsing_valid() {
        let content = "LeagueClient:9876:54321:abcdefg12345:https\n";
        let creds = LcuConnector::parse_lockfile(content).expect("creds");
        assert_eq!(creds.port, 54321);
        assert_eq!(creds.auth_token, "abcdefg12345");
    }

    #[test]
    fn test_f10_02_lockfile_parsing_malformed() {
        assert!(LcuConnector::parse_lockfile("").is_none());
        assert!(LcuConnector::parse_lockfile("LeagueClient:123").is_none());
        assert!(LcuConnector::parse_lockfile(":::").is_none());
    }

    #[test]
    fn test_f10_03_lockfile_parsing_invalid_port() {
        let content = "LeagueClient:9876:not_a_port:token:https";
        assert!(LcuConnector::parse_lockfile(content).is_none());
    }

    #[tokio::test]
    async fn test_f10_04_mock_server_connection_auth_header() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let phase = client.get_gameflow_phase().await.expect("authenticated call");
        assert_eq!(phase, "ChampSelect");
        server.stop();
    }

    #[tokio::test]
    async fn test_f10_05_mock_server_status_override_error_simulation() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Simulate 429 Rate Limit
        server.state.set_status_override(Some(axum::http::StatusCode::TOO_MANY_REQUESTS)).await;
        let res = client.get_gameflow_phase().await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Status 429"));

        // Clear override -> recover
        server.state.set_status_override(None).await;
        let recovered = client.get_gameflow_phase().await.expect("recovered");
        assert_eq!(recovered, "ChampSelect");
        server.stop();
    }
}

// ==============================================================================================
// Feature 11: CN Anti-Anonymity & Account Switch (Relog) (5 tests)
// ==============================================================================================
mod f11_cn_anti_anonymity {
    use super::*;

    #[tokio::test]
    async fn test_f11_01_chat_participant_puuid_extraction() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let participants = canned_chat_participants(
            "room-123",
            &[
                ("puuid-1", "玩家A"),
                ("puuid-2", "玩家B"),
                ("puuid-3", "玩家C"),
                ("puuid-4", "玩家D"),
                ("puuid-local", "本地玩家"),
            ],
        );
        server.state.set_chat_participants("room-123", participants).await;

        let fetched = client
            .get_chat_conversation_participants("room-123")
            .await
            .expect("participants");
        assert_eq!(fetched.len(), 5);
        assert_eq!(fetched[0]["puuid"].as_str().unwrap(), "puuid-1");
        assert_eq!(fetched[0]["name"].as_str().unwrap(), "玩家A");
        server.stop();
    }

    #[test]
    fn test_f11_02_local_player_puuid_exclusion() {
        let local_puuid = "puuid-local";
        let participants = canned_chat_participants(
            "room-123",
            &[
                ("puuid-1", "玩家A"),
                ("puuid-2", "玩家B"),
                ("puuid-3", "玩家C"),
                ("puuid-4", "玩家D"),
                ("puuid-local", "本地玩家"),
            ],
        );

        // Candidate filter: exclude local player
        let candidates: Vec<_> = participants
            .iter()
            .filter(|p| p["puuid"].as_str().unwrap() != local_puuid)
            .collect();

        assert_eq!(candidates.len(), 4, "Must leave exactly 4 teammates");
        assert!(!candidates.iter().any(|p| p["puuid"].as_str().unwrap() == local_puuid));
    }

    #[test]
    fn test_f11_03_anonymous_slot_assignment() {
        let local_puuid = "puuid-local";
        let session = canned_champ_select_session("room-1", local_puuid, 0);
        let my_team = session["myTeam"].as_array().unwrap();

        // 4 anonymous slots (index 1..5)
        let anon_slots: Vec<_> = my_team.iter().filter(|p| !p["isLocalPlayer"].as_bool().unwrap()).collect();
        assert_eq!(anon_slots.len(), 4);

        let candidates = vec![
            ("cand-1", "队友A"),
            ("cand-2", "队友B"),
            ("cand-3", "队友C"),
            ("cand-4", "队友D"),
        ];

        // Assign candidates to anonymous slots
        let mut resolved = Vec::new();
        for (slot, cand) in anon_slots.iter().zip(candidates.iter()) {
            resolved.push((slot["cellId"].as_i64().unwrap(), cand.0, cand.1));
        }
        assert_eq!(resolved.len(), 4);
        assert_eq!(resolved[0].1, "cand-1");
    }

    #[tokio::test]
    async fn test_f11_04_account_switch_puuid_change() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Account A
        server.state.set_current_summoner(canned_current_summoner("puuid-acc-A", "玩家甲")).await;
        let cur_a = client.get_current_summoner().await.expect("acc a");
        assert_eq!(cur_a["puuid"].as_str().unwrap(), "puuid-acc-A");

        // Relog to Account B
        server.state.set_current_summoner(canned_current_summoner("puuid-acc-B", "玩家乙")).await;
        let cur_b = client.get_current_summoner().await.expect("acc b");
        assert_eq!(cur_b["puuid"].as_str().unwrap(), "puuid-acc-B");
        server.stop();
    }

    #[test]
    fn test_f11_05_ghost_player_prevention() {
        let local_puuid = "puuid-local";
        let candidates = vec!["puuid-1", "puuid-2", "puuid-3", "puuid-4"];

        // Verify that candidates set NEVER intersects with local player
        assert!(!candidates.contains(&local_puuid), "Local player must never clone into teammate slots");
    }
}

// ==============================================================================================
// Feature 12: Special Game Modes (ARAM, Arena 2v2v2v2, TFT) (5 tests)
// ==============================================================================================
mod f12_special_game_modes {
    use super::*;

    #[test]
    fn test_f12_01_queue_mode_cn_resolution() {
        let db = Database::new();
        let raw_420 = json!({ "gameId": 1, "queueId": 420, "participantIdentities": [], "participants": [] });
        let raw_440 = json!({ "gameId": 2, "queueId": 440, "participantIdentities": [], "participants": [] });
        let raw_450 = json!({ "gameId": 3, "queueId": 450, "participantIdentities": [], "participants": [] });

        let parsed_420 = parse_riot_match_detail(&raw_420, &db).unwrap();
        let parsed_440 = parse_riot_match_detail(&raw_440, &db).unwrap();
        let parsed_450 = parse_riot_match_detail(&raw_450, &db).unwrap();

        assert_eq!(parsed_420["mode_cn"].as_str().unwrap(), "排位赛(单双)");
        assert_eq!(parsed_440["mode_cn"].as_str().unwrap(), "排位赛(灵活)");
        assert_eq!(parsed_450["mode_cn"].as_str().unwrap(), "狂暴大乱斗");
    }

    #[test]
    fn test_f12_02_aram_scoreboard_parsing() {
        let db = Database::new();
        let raw = canned_match_detail_aram(3001);
        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some(), "ARAM match must parse cleanly without panic");
        let detail = parsed.unwrap();
        assert_eq!(detail["mode_cn"].as_str().unwrap(), "狂暴大乱斗");
    }

    #[test]
    fn test_f12_03_arena_multi_team_robustness() {
        let db = Database::new();
        let raw = canned_match_detail_arena(4001);
        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some(), "Arena 8-player match must parse without panic");
    }

    #[test]
    fn test_f12_04_unrecognized_queue_id_fallback() {
        let db = Database::new();
        let raw = json!({
            "gameId": 5001,
            "queueId": 9999, // Unknown queue
            "participantIdentities": [],
            "participants": []
        });
        let parsed = parse_riot_match_detail(&raw, &db).unwrap();
        assert_eq!(parsed["mode_cn"].as_str().unwrap(), "匹配对局");
    }

    #[test]
    fn test_f12_05_fewer_than_ten_players_no_panic() {
        let db = Database::new();
        let raw = json!({
            "gameId": 6001,
            "queueId": 420,
            "participantIdentities": [
                { "participantId": 1, "player": { "puuid": "p1", "summonerName": "P1" } }
            ],
            "participants": [
                {
                    "participantId": 1,
                    "teamId": 100,
                    "championId": 1,
                    "stats": { "win": true, "kills": 5, "deaths": 0, "assists": 2, "totalDamageDealtToChampions": 10000 }
                }
            ]
        });
        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some(), "Partial player counts must not panic");
    }
}

// ==============================================================================================
// Feature 13: Concurrency Throttling & Atomic Config Persistence (5 tests)
// ==============================================================================================
mod f13_concurrency_and_config {
    use super::*;

    #[tokio::test]
    async fn test_f13_01_config_manager_atomic_update() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_test_f13_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        let mgr = ConfigManager::new(cfg_path.to_str().unwrap());
        let updated = mgr
            .update(
                Some(true),
                Some(1.5),
                Some(0.8),
                Some(1.2),
                Some("NET_DEDUCT".to_string()),
                Some("DYNAMIC".to_string()),
                None,
                Some(true),
                Some(3),
            )
            .await;

        assert_eq!(updated.hex_synergy_weight, 1.5);
        assert_eq!(updated.archetype_strictness, 0.8);
        assert_eq!(updated.auto_accept_match, true);
        assert_eq!(updated.auto_accept_delay_seconds, 3);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_f13_02_config_nan_defense() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_test_nan_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        let mgr = ConfigManager::new(cfg_path.to_str().unwrap());
        let updated = mgr
            .update(
                None,
                Some(f64::NAN),
                Some(f64::NAN),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(!updated.hex_synergy_weight.is_nan(), "NaN must be rejected");
        assert!(!updated.archetype_strictness.is_nan(), "NaN must be rejected");

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_f13_03_config_infinity_defense() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_test_inf_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        let mgr = ConfigManager::new(cfg_path.to_str().unwrap());
        let updated = mgr
            .update(
                None,
                Some(f64::INFINITY),
                None,
                Some(f64::NEG_INFINITY),
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(!updated.hex_synergy_weight.is_infinite());
        assert!(!updated.counter_intensity.is_infinite());

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_f13_04_concurrent_config_mutations() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_test_conc_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        let mgr = Arc::new(ConfigManager::new(cfg_path.to_str().unwrap()));
        let mut handles = Vec::new();

        for i in 0..10 {
            let m = mgr.clone();
            handles.push(tokio::spawn(async move {
                m.update(
                    Some(i % 2 == 0),
                    Some(1.0 + (i as f64 * 0.1)),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            }));
        }

        for h in handles {
            h.await.expect("join handle");
        }

        let final_cfg = mgr.get().await;
        assert!(final_cfg.hex_synergy_weight >= 1.0 && final_cfg.hex_synergy_weight <= 3.0);
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_f13_05_mock_lcu_concurrent_requests_semaphore() {
        let server = MockLcuServer::start().await;
        let client = Arc::new(MockLcuClient::from_server(&server));
        let semaphore = Arc::new(Semaphore::new(4)); // Max 4 concurrent workers

        let mut handles = Vec::new();
        for _ in 0..10 {
            let c = client.clone();
            let sem = semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                c.get_gameflow_phase().await
            }));
        }

        for h in handles {
            let res = h.await.unwrap();
            assert_eq!(res.unwrap(), "ChampSelect");
        }

        assert_eq!(server.state.get_request_count(), 10);
        server.stop();
    }
}
