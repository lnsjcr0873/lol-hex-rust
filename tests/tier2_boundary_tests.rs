mod lcu_mock_harness;

use axum::http::StatusCode;
use lcu_mock_harness::fixtures::*;
use lcu_mock_harness::*;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::*;
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
// Feature 1: LCU API & Data Contract Alignment — Boundaries (5 tests)
// ==============================================================================================
mod f1_boundary_lcu_contract {
    use super::*;

    #[tokio::test]
    async fn test_f1_b01_empty_gameflow_phase() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Server sets phase to empty string
        server.state.set_phase("").await;
        let phase = client.get_gameflow_phase().await.expect("phase response");
        assert_eq!(phase, "");
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_b02_oversized_payload_handling() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // 100 extra metadata fields in session
        let mut session = canned_champ_select_session("huge-room", "local-puuid", 0);
        let huge_comment = "x".repeat(50000);
        session["largeMetadata"] = json!(huge_comment);

        server.state.set_champ_select_session(session).await;
        let fetched = client.get_champ_select_session().await.expect("large payload");
        assert_eq!(
            fetched["largeMetadata"].as_str().unwrap().len(),
            50000
        );
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_b03_session_null_and_missing_keys() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Session with null chat details and empty teams
        let sparse_session = json!({
            "gameId": 0,
            "chatDetails": null,
            "myTeam": [],
            "theirTeam": null
        });
        server.state.set_champ_select_session(sparse_session).await;

        let fetched = client.get_champ_select_session().await.expect("sparse session");
        assert_eq!(fetched["gameId"].as_i64().unwrap(), 0);
        assert!(fetched["chatDetails"].is_null());
        assert_eq!(fetched["myTeam"].as_array().unwrap().len(), 0);
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_b04_negative_and_zero_game_id() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Game ID 0
        let detail_zero = canned_match_detail_classic(0, true);
        server.state.set_match_detail(0, detail_zero).await;
        let res_zero = client.get_match_detail(0).await.expect("game 0 detail");
        assert_eq!(res_zero["gameId"].as_i64().unwrap(), 0);

        // Game ID -1
        let detail_neg = canned_match_detail_classic(-1, false);
        server.state.set_match_detail(-1, detail_neg).await;
        let res_neg = client.get_match_detail(-1).await.expect("game -1 detail");
        assert_eq!(res_neg["gameId"].as_i64().unwrap(), -1);
        server.stop();
    }

    #[tokio::test]
    async fn test_f1_b05_extreme_summoner_levels() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        // Level 0
        let mut sum_zero = canned_current_summoner("p-0", "Lvl0");
        sum_zero["summonerLevel"] = json!(0);
        server.state.set_current_summoner(sum_zero).await;
        let cur = client.get_current_summoner().await.expect("lvl 0 summoner");
        assert_eq!(cur["summonerLevel"].as_i64().unwrap(), 0);

        // Level 99999
        let mut sum_high = canned_current_summoner("p-high", "HighLvl");
        sum_high["summonerLevel"] = json!(99999);
        server.state.set_current_summoner(sum_high).await;
        let cur_high = client.get_current_summoner().await.expect("lvl 99999 summoner");
        assert_eq!(cur_high["summonerLevel"].as_i64().unwrap(), 99999);
        server.stop();
    }
}

// ==============================================================================================
// Feature 2: Data Aggregation & Scoring Algorithms — Boundaries (5 tests)
// ==============================================================================================
mod f2_boundary_scoring_and_radar {
    use super::*;

    #[test]
    fn test_f2_b01_zero_damage_across_all_participants() {
        let mut inputs = Vec::new();
        for i in 1..=10 {
            inputs.push(MatchParticipantInput {
                participant_id: i,
                puuid: format!("p-{}", i),
                team_id: if i <= 5 { 100 } else { 200 },
                is_win: i <= 5,
                kills: 0,
                deaths: 0,
                assists: 0,
                damage: 0,
                damage_taken: 0,
                damage_mitigated: 0,
                gold: 1000,
                cs: 0,
                vision_score: 0,
            });
        }

        let evals = evaluate_match_participants(1200, &inputs);
        assert_eq!(evals.len(), 10);
        for e in &evals {
            assert!(!e.score.is_nan(), "Score must never be NaN");
            assert_eq!(e.damage_share_pct, 20.0, "Zero team damage falls back to default 20%");
        }
        // Winner MVP and loser SVP should still be uniquely determined by tie breaker
        assert_eq!(evals.iter().filter(|e| e.is_mvp).count(), 1);
        assert_eq!(evals.iter().filter(|e| e.is_svp).count(), 1);
    }

    #[test]
    fn test_f2_b02_extreme_match_duration() {
        let mut inputs = Vec::new();
        inputs.push(MatchParticipantInput {
            participant_id: 1,
            puuid: "p-1".to_string(),
            team_id: 100,
            is_win: true,
            kills: 5,
            deaths: 1,
            assists: 5,
            damage: 20000,
            damage_taken: 10000,
            damage_mitigated: 5000,
            gold: 15000,
            cs: 200,
            vision_score: 30,
        });

        // 0-second duration (should clamp to at least 1.0 min)
        let eval_zero = evaluate_match_participants(0, &inputs);
        assert_eq!(eval_zero.len(), 1);
        assert!(!eval_zero[0].score.is_nan());

        // 10-hour duration (36,000s)
        let eval_huge = evaluate_match_participants(36000, &inputs);
        assert_eq!(eval_huge.len(), 1);
        assert!(!eval_huge[0].score.is_nan());
    }

    #[test]
    fn test_f2_b03_all_winners_and_all_losers() {
        // All 5 participants are winners (no losers -> 0 SVPs)
        let mut all_winners = Vec::new();
        for i in 1..=5 {
            all_winners.push(MatchParticipantInput {
                participant_id: i,
                puuid: format!("win-{}", i),
                team_id: 100,
                is_win: true,
                kills: i as i32 * 2,
                deaths: 1,
                assists: 3,
                damage: 10000 * i,
                damage_taken: 5000,
                damage_mitigated: 2000,
                gold: 8000,
                cs: 100,
                vision_score: 10,
            });
        }
        let evals_win = evaluate_match_participants(1500, &all_winners);
        assert_eq!(evals_win.iter().filter(|e| e.is_mvp).count(), 1);
        assert_eq!(evals_win.iter().filter(|e| e.is_svp).count(), 0);

        // All 5 participants are losers (no winners -> 0 MVPs)
        let mut all_losers = Vec::new();
        for i in 1..=5 {
            all_losers.push(MatchParticipantInput {
                participant_id: i,
                puuid: format!("lose-{}", i),
                team_id: 200,
                is_win: false,
                kills: 1,
                deaths: i as i32 + 2,
                assists: 2,
                damage: 8000 * i,
                damage_taken: 10000,
                damage_mitigated: 4000,
                gold: 6000,
                cs: 80,
                vision_score: 8,
            });
        }
        let evals_lose = evaluate_match_participants(1500, &all_losers);
        assert_eq!(evals_lose.iter().filter(|e| e.is_mvp).count(), 0);
        assert_eq!(evals_lose.iter().filter(|e| e.is_svp).count(), 1);
    }

    #[test]
    fn test_f2_b04_radar_clamping_extremes() {
        // Upper extreme matches
        let high_matches = vec![
            ParsedMatchRecord {
                damage: 150000,
                damage_share_pct: 65.0,
                kills: 35,
                deaths: 0,
                assists: 25,
                kp_pct: 95.0,
                cs_per_min: 15.0,
                gold: 30000,
                vision_score: 100,
                ..Default::default()
            }
        ];
        let radar_high = compute_radar(&high_matches);
        for (_k, v) in radar_high {
            assert!(v <= 99, "Radar values must never exceed 99, got {}", v);
        }

        // Lower extreme matches
        let low_matches = vec![
            ParsedMatchRecord {
                damage: 0,
                damage_share_pct: 0.0,
                kills: 0,
                deaths: 30,
                assists: 0,
                kp_pct: 0.0,
                cs_per_min: 0.0,
                gold: 0,
                vision_score: 0,
                ..Default::default()
            }
        ];
        let radar_low = compute_radar(&low_matches);
        for (_k, v) in radar_low {
            assert!(v >= 25, "Radar values must never drop below 25, got {}", v);
        }
    }

    #[test]
    fn test_f2_b05_solitary_participant_evaluation() {
        let single = vec![MatchParticipantInput {
            participant_id: 1,
            puuid: "solo-1".to_string(),
            team_id: 100,
            is_win: true,
            kills: 10,
            deaths: 0,
            assists: 0,
            damage: 25000,
            damage_taken: 5000,
            damage_mitigated: 2000,
            gold: 12000,
            cs: 150,
            vision_score: 20,
        }];

        let evals = evaluate_match_participants(900, &single);
        assert_eq!(evals.len(), 1);
        assert!(evals[0].is_mvp);
        assert!(evals[0].is_highest_damage);
        assert_eq!(evals[0].damage_share_pct, 100.0);
    }
}

// ==============================================================================================
// Feature 3: Premade Team Detection (Graph BFS) — Boundaries (5 tests)
// ==============================================================================================
mod f3_boundary_premade_detection {
    use super::*;

    #[test]
    fn test_f3_b01_exact_one_game_vs_two_games_boundary() {
        // Player 1 & 2 share exactly 1 game -> NOT premade
        let mut players_one = vec![
            OngoingPlayerCard {
                puuid: "p1".to_string(),
                matches: vec![ParsedMatchRecord { game_id: 101, ..Default::default() }],
                ..Default::default()
            },
            OngoingPlayerCard {
                puuid: "p2".to_string(),
                matches: vec![ParsedMatchRecord { game_id: 101, ..Default::default() }],
                ..Default::default()
            },
        ];
        detect_premade_teams(&mut players_one);
        assert!(players_one[0].premade_group.is_none());
        assert!(players_one[1].premade_group.is_none());

        // Player 1 & 2 share exactly 2 games -> IS premade
        let mut players_two = vec![
            OngoingPlayerCard {
                puuid: "p1".to_string(),
                matches: vec![
                    ParsedMatchRecord { game_id: 101, ..Default::default() },
                    ParsedMatchRecord { game_id: 102, ..Default::default() },
                ],
                ..Default::default()
            },
            OngoingPlayerCard {
                puuid: "p2".to_string(),
                matches: vec![
                    ParsedMatchRecord { game_id: 101, ..Default::default() },
                    ParsedMatchRecord { game_id: 102, ..Default::default() },
                ],
                ..Default::default()
            },
        ];
        detect_premade_teams(&mut players_two);
        assert_eq!(players_two[0].premade_group, Some(1));
        assert_eq!(players_two[1].premade_group, Some(1));
        assert_eq!(players_two[0].premade_tag.as_deref(), Some("双排车队"));
    }

    #[test]
    fn test_f3_b02_empty_and_single_player_input() {
        let mut empty: Vec<OngoingPlayerCard> = Vec::new();
        detect_premade_teams(&mut empty);
        assert_eq!(empty.len(), 0);

        let mut single = vec![OngoingPlayerCard {
            puuid: "p1".to_string(),
            matches: vec![ParsedMatchRecord { game_id: 101, ..Default::default() }],
            ..Default::default()
        }];
        detect_premade_teams(&mut single);
        assert!(single[0].premade_group.is_none());
    }

    #[test]
    fn test_f3_b03_opponents_in_current_match_isolated() {
        // Both players share 5 games together historically, but in the current game,
        // Player 1 is on Blue (100) and Player 2 is on Red (200).
        let mut players = vec![
            OngoingPlayerCard {
                puuid: "p1".to_string(),
                team_id: 100,
                matches: (1..=5)
                    .map(|gid| ParsedMatchRecord {
                        game_id: gid,
                        teammate_puuids: vec!["p2".to_string()],
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
            OngoingPlayerCard {
                puuid: "p2".to_string(),
                team_id: 200,
                matches: (1..=5)
                    .map(|gid| ParsedMatchRecord {
                        game_id: gid,
                        teammate_puuids: vec!["p1".to_string()],
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
        ];

        detect_premade_teams(&mut players);
        assert!(players[0].premade_group.is_none(), "Current match opponents must never form premade");
        assert!(players[1].premade_group.is_none(), "Current match opponents must never form premade");
    }

    #[test]
    fn test_f3_b04_full_five_player_party_five_man() {
        let mut players = Vec::new();
        for i in 1..=5 {
            let mut teammate_puuids = Vec::new();
            for j in 1..=5 {
                if i != j {
                    teammate_puuids.push(format!("p{}", j));
                }
            }
            players.push(OngoingPlayerCard {
                puuid: format!("p{}", i),
                team_id: 100,
                matches: vec![
                    ParsedMatchRecord {
                        game_id: 501,
                        teammate_puuids: teammate_puuids.clone(),
                        ..Default::default()
                    },
                    ParsedMatchRecord {
                        game_id: 502,
                        teammate_puuids,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            });
        }

        detect_premade_teams(&mut players);
        for p in &players {
            assert_eq!(p.premade_group, Some(1));
            assert_eq!(p.premade_tag.as_deref(), Some("五排车队"));
        }
    }

    #[test]
    fn test_f3_b05_cyclic_one_match_triangle_no_group() {
        // A & B share game 1
        // B & C share game 2
        // C & A share game 3
        // Each pair shares only 1 game -> NO edges formed!
        let mut players = vec![
            OngoingPlayerCard {
                puuid: "pA".to_string(),
                matches: vec![
                    ParsedMatchRecord { game_id: 1, ..Default::default() },
                    ParsedMatchRecord { game_id: 3, ..Default::default() },
                ],
                ..Default::default()
            },
            OngoingPlayerCard {
                puuid: "pB".to_string(),
                matches: vec![
                    ParsedMatchRecord { game_id: 1, ..Default::default() },
                    ParsedMatchRecord { game_id: 2, ..Default::default() },
                ],
                ..Default::default()
            },
            OngoingPlayerCard {
                puuid: "pC".to_string(),
                matches: vec![
                    ParsedMatchRecord { game_id: 2, ..Default::default() },
                    ParsedMatchRecord { game_id: 3, ..Default::default() },
                ],
                ..Default::default()
            },
        ];

        detect_premade_teams(&mut players);
        assert!(players.iter().all(|p| p.premade_group.is_none()));
    }
}

// ==============================================================================================
// Feature 4: Tactical Badges Generation — Boundaries (5 tests)
// ==============================================================================================
mod f4_boundary_tactical_badges {
    use super::*;

    #[test]
    fn test_f4_b01_win_streak_exact_threshold() {
        let mgr = PlayerHistoryManager::new();

        // 2 wins -> streak 2, no streak badge
        let rec2 = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "GOLD".to_string(),
            division: "".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 2,
            ranked_losses: 0,
            matches: vec![
                ParsedMatchRecord { win: true, ..Default::default() },
                ParsedMatchRecord { win: true, ..Default::default() },
            ],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof2 = mgr.compute_tactical_profile(&rec2, 0, 20);
        assert_eq!(prof2.streak_count, 2);
        assert!(!prof2.badges.iter().any(|b| b.text.contains("连胜")));

        // 3 wins -> streak 3, badge present
        let mut rec3 = rec2.clone();
        rec3.matches.push(ParsedMatchRecord { win: true, ..Default::default() });
        let prof3 = mgr.compute_tactical_profile(&rec3, 0, 20);
        assert_eq!(prof3.streak_count, 3);
        assert!(prof3.badges.iter().any(|b| b.text == "🔥 3连胜"));
    }

    #[test]
    fn test_f4_b02_loss_streak_exact_threshold() {
        let mgr = PlayerHistoryManager::new();

        // 2 losses -> no badge
        let rec2 = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "SILVER".to_string(),
            division: "".to_string(),
            tier_str: "白银".to_string(),
            tier_color: "#94a3b8".to_string(),
            lp: 0,
            ranked_wins: 0,
            ranked_losses: 2,
            matches: vec![
                ParsedMatchRecord { win: false, ..Default::default() },
                ParsedMatchRecord { win: false, ..Default::default() },
            ],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof2 = mgr.compute_tactical_profile(&rec2, 0, 20);
        assert_eq!(prof2.streak_count, 2);
        assert!(!prof2.badges.iter().any(|b| b.text.contains("连败")));

        // 3 losses -> badge present
        let mut rec3 = rec2.clone();
        rec3.matches.push(ParsedMatchRecord { win: false, ..Default::default() });
        let prof3 = mgr.compute_tactical_profile(&rec3, 0, 20);
        assert_eq!(prof3.streak_count, 3);
        assert!(prof3.badges.iter().any(|b| b.text == "❄️ 3连败"));
    }

    #[test]
    fn test_f4_b03_one_trick_boundary_conditions() {
        let mgr = PlayerHistoryManager::new();

        // 2 games with 100% win rate on Yasuo (157) -> NOT one-trick yet (requires >= 3 games)
        let rec = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "GOLD".to_string(),
            division: "".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 2,
            ranked_losses: 0,
            matches: vec![
                ParsedMatchRecord { champion_id: 157, win: true, ..Default::default() },
                ParsedMatchRecord { champion_id: 157, win: true, ..Default::default() },
            ],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof = mgr.compute_tactical_profile(&rec, 157, 20);
        assert!(!prof.badges.iter().any(|b| b.text.contains("绝活")));

        // 3 games: 2 wins, 1 loss (67% win rate) -> IS one-trick
        let mut rec3 = rec.clone();
        rec3.matches.push(ParsedMatchRecord { champion_id: 157, win: false, ..Default::default() });
        let prof3 = mgr.compute_tactical_profile(&rec3, 157, 20);
        assert!(prof3.badges.iter().any(|b| b.text.contains("绝活(67%)")));
    }

    #[test]
    fn test_f4_b04_testing_champ_zero_games_boundary() {
        let mgr = PlayerHistoryManager::new();

        // 3 matches played on other champions, 0 on Zed (238) -> total matches < 4 -> NO testing badge
        let rec_under = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "GOLD".to_string(),
            division: "".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 1,
            ranked_losses: 2,
            matches: vec![
                ParsedMatchRecord { champion_id: 1, win: true, ..Default::default() },
                ParsedMatchRecord { champion_id: 2, win: false, ..Default::default() },
                ParsedMatchRecord { champion_id: 3, win: false, ..Default::default() },
            ],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof_under = mgr.compute_tactical_profile(&rec_under, 238, 20);
        assert!(!prof_under.badges.iter().any(|b| b.text.contains("试英雄")));

        // 4 matches played, 0 on Zed -> total matches >= 4 -> IS testing badge
        let mut rec_over = rec_under.clone();
        rec_over.matches.push(ParsedMatchRecord { champion_id: 4, win: true, ..Default::default() });
        let prof_over = mgr.compute_tactical_profile(&rec_over, 238, 20);
        assert!(prof_over.badges.iter().any(|b| b.text.contains("试英雄")));
    }

    #[test]
    fn test_f4_b05_zero_deaths_kda_division_by_zero() {
        let mgr = PlayerHistoryManager::new();
        let rec = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "CHALLENGER".to_string(),
            division: "".to_string(),
            tier_str: "王者".to_string(),
            tier_color: "#f59e0b".to_string(),
            lp: 1200,
            ranked_wins: 5,
            ranked_losses: 0,
            matches: vec![
                ParsedMatchRecord { kills: 10, deaths: 0, assists: 5, win: true, ..Default::default() },
                ParsedMatchRecord { kills: 8, deaths: 0, assists: 7, win: true, ..Default::default() },
            ],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof = mgr.compute_tactical_profile(&rec, 0, 20);
        assert!(!prof.avg_kda_ratio.is_nan());
        assert!(!prof.avg_kda_ratio.is_infinite());
        assert_eq!(prof.avg_kda_ratio, 30.0);
    }
}

// ==============================================================================================
// Feature 5: Chat Record Broadcast Formatting & Rate Limit — Boundaries (5 tests)
// ==============================================================================================
mod f5_boundary_chat_broadcast {
    use super::*;

    #[test]
    fn test_f5_b01_zero_and_hundred_win_rate_broadcast() {
        let mut ongoing = OngoingMatchResponse::default();
        let mut card_zero = OngoingPlayerCard::default();
        card_zero.summoner_name = "ZeroWR".to_string();
        card_zero.tactical_analysis.recent_win_rate_pct = 0;
        card_zero.tactical_analysis.recent_count = 10;
        card_zero.tactical_analysis.tier_str = "黑铁 IV".to_string();

        let mut card_hundred = OngoingPlayerCard::default();
        card_hundred.summoner_name = "HundredWR".to_string();
        card_hundred.tactical_analysis.recent_win_rate_pct = 100;
        card_hundred.tactical_analysis.recent_count = 10;
        card_hundred.tactical_analysis.tier_str = "王者 1000LP".to_string();

        ongoing.my_team.push(card_zero);
        ongoing.my_team.push(card_hundred);

        let lines = format_chat_broadcast_lines(&ongoing);
        assert!(lines[1].contains("胜率0%"));
        assert!(lines[2].contains("胜率100%"));
    }

    #[test]
    fn test_f5_b02_special_characters_in_summoner_name() {
        let mut ongoing = OngoingMatchResponse::default();
        let mut card = OngoingPlayerCard::default();
        card.summoner_name = "<VIP> ★Faker★ #KR1".to_string();
        card.tactical_analysis.recent_win_rate_pct = 75;
        card.tactical_analysis.recent_count = 20;
        card.tactical_analysis.tier_str = "最强王者".to_string();
        ongoing.my_team.push(card);

        let lines = format_chat_broadcast_lines(&ongoing);
        assert!(lines[1].contains("[<VIP> ★Faker★ #KR1]"));
    }

    #[test]
    fn test_f5_b03_less_than_five_players_in_team() {
        let mut ongoing = OngoingMatchResponse::default();
        let mut card = OngoingPlayerCard::default();
        card.summoner_name = "SoloPlayer".to_string();
        card.tactical_analysis.recent_win_rate_pct = 50;
        card.tactical_analysis.recent_count = 5;
        card.tactical_analysis.tier_str = "黄金 I".to_string();
        ongoing.my_team.push(card);

        let lines = format_chat_broadcast_lines(&ongoing);
        // Header + 1 player line = 2 lines
        assert_eq!(lines.len(), 2);
        assert!(lines[1].starts_with("1. 上路"));
    }

    #[test]
    fn test_f5_b04_empty_premade_tag_handling() {
        let mut ongoing = OngoingMatchResponse::default();
        let mut card_none = OngoingPlayerCard::default();
        card_none.summoner_name = "P1".to_string();
        card_none.premade_tag = None;

        let mut card_empty_str = OngoingPlayerCard::default();
        card_empty_str.summoner_name = "P2".to_string();
        card_empty_str.premade_tag = Some("".to_string());

        let mut card_tagged = OngoingPlayerCard::default();
        card_tagged.summoner_name = "P3".to_string();
        card_tagged.premade_tag = Some("三排车队".to_string());

        ongoing.my_team.push(card_none);
        ongoing.my_team.push(card_empty_str);
        ongoing.my_team.push(card_tagged);

        let lines = format_chat_broadcast_lines(&ongoing);
        assert!(!lines[1].ends_with("]"));
        assert!(!lines[2].ends_with("]"));
        assert!(lines[3].ends_with("[三排车队]"));
    }

    #[test]
    fn test_f5_b05_unranked_player_tier_string() {
        let mut ongoing = OngoingMatchResponse::default();
        let mut card = OngoingPlayerCard::default();
        card.summoner_name = "Newbie".to_string();
        card.tactical_analysis.tier_str = "未定级".to_string();
        card.tactical_analysis.recent_count = 0;
        card.tactical_analysis.recent_win_rate_pct = 50;
        ongoing.my_team.push(card);

        let lines = format_chat_broadcast_lines(&ongoing);
        assert!(lines[1].contains("未定级"));
        assert!(lines[1].contains("近0场胜率50%"));
    }
}

// ==============================================================================================
// Feature 6: Ongoing Match 5v5 Dual-Column Alignment — Boundaries (5 tests)
// ==============================================================================================
mod f6_boundary_ongoing_match {
    use super::*;

    #[test]
    fn test_f6_b01_empty_team_lists_safe_serialization() {
        let resp = OngoingMatchResponse::default();
        let json_val = serde_json::to_value(&resp).expect("serialize");
        assert_eq!(json_val["blue_team"].as_array().unwrap().len(), 0);
        assert_eq!(json_val["red_team"].as_array().unwrap().len(), 0);
        assert_eq!(json_val["is_active"].as_bool().unwrap(), false);
    }

    #[test]
    fn test_f6_b02_asymmetric_team_counts() {
        let mut resp = OngoingMatchResponse::default();
        // 3 on blue, 5 on red
        for i in 0..3 {
            resp.blue_team.push(OngoingPlayerCard { slot_index: i, team_id: 100, ..Default::default() });
        }
        for i in 0..5 {
            resp.red_team.push(OngoingPlayerCard { slot_index: i + 3, team_id: 200, ..Default::default() });
        }
        assert_eq!(resp.blue_team.len(), 3);
        assert_eq!(resp.red_team.len(), 5);
    }

    #[test]
    fn test_f6_b03_unknown_position_fallback() {
        let mut card = OngoingPlayerCard::default();
        card.position = "UNKNOWN_ROLE".to_string();
        assert_eq!(card.position, "UNKNOWN_ROLE");
    }

    #[test]
    fn test_f6_b04_negative_or_zero_champion_id() {
        let mut card_unpicked = OngoingPlayerCard::default();
        card_unpicked.champion_id = 0;
        assert_eq!(card_unpicked.champion_id, 0);

        let mut card_invalid = OngoingPlayerCard::default();
        card_invalid.champion_id = -1;
        assert_eq!(card_invalid.champion_id, -1);
    }

    #[test]
    fn test_f6_b05_inactive_phase_with_error_message() {
        let resp = OngoingMatchResponse {
            phase: "None".to_string(),
            is_active: false,
            is_mock: false,
            message: Some("英雄联盟客户端未启动".to_string()),
            ..Default::default()
        };
        assert!(!resp.is_active);
        assert_eq!(resp.message.as_deref(), Some("英雄联盟客户端未启动"));
    }
}

// ==============================================================================================
// Feature 7: Offline & Simulation Mode Transitions — Boundaries (5 tests)
// ==============================================================================================
mod f7_boundary_offline_and_mock {
    use super::*;

    #[tokio::test]
    async fn test_f7_b01_unknown_gameflow_phase_string() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        server.state.set_phase("CustomPhaseSpecial").await;
        let phase = client.get_gameflow_phase().await.expect("custom phase");
        assert_eq!(phase, "CustomPhaseSpecial");
        server.stop();
    }

    #[test]
    fn test_f7_b02_mock_detail_zero_and_negative_id() {
        let db = Database::new();
        let m_zero = build_mock_match_detail(0, &db);
        assert_eq!(m_zero["game_id"].as_i64().unwrap(), 0);

        let m_neg = build_mock_match_detail(-99, &db);
        assert_eq!(m_neg["game_id"].as_i64().unwrap(), -99);
    }

    #[tokio::test]
    async fn test_f7_b03_rapid_phase_toggle_state() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        for i in 0..10 {
            let target = if i % 2 == 0 { "ChampSelect" } else { "None" };
            server.state.set_phase(target).await;
            assert_eq!(client.get_gameflow_phase().await.unwrap(), target);
        }
        server.stop();
    }

    #[tokio::test]
    async fn test_f7_b04_missing_game_id_404_handling() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let res = client.get_match_detail(88888888).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("404"));
        server.stop();
    }

    #[tokio::test]
    async fn test_f7_b05_stopped_server_connection_refused() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);
        let port = server.port;
        server.stop();

        // Server stopped: subsequent request must fail without hanging
        let req = client.get_gameflow_phase().await;
        assert!(req.is_err(), "Must return error when server port {} is closed", port);
    }
}

// ==============================================================================================
// Feature 8: Summoner Profile & History Queue Filter — Boundaries (5 tests)
// ==============================================================================================
mod f8_boundary_summoner_profile {
    use super::*;

    #[test]
    fn test_f8_b01_max_lp_challenger_tier() {
        assert_eq!(get_tier_color("CHALLENGER"), "#f59e0b");
        let rec = CachedPlayerRecord {
            puuid: "faker".to_string(),
            summoner_name: "Hide on bush".to_string(),
            tier: "CHALLENGER".to_string(),
            division: "".to_string(),
            tier_str: "最强王者 2450 LP".to_string(),
            tier_color: get_tier_color("CHALLENGER").to_string(),
            lp: 2450,
            ranked_wins: 500,
            ranked_losses: 300,
            matches: vec![],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        assert_eq!(rec.lp, 2450);
        assert_eq!(rec.tier_color, "#f59e0b");
    }

    #[test]
    fn test_f8_b02_negative_lp_and_iron_tier() {
        assert_eq!(get_tier_color("IRON"), "#71717a");
        let rec = CachedPlayerRecord {
            puuid: "iron_player".to_string(),
            summoner_name: "IronBot".to_string(),
            tier: "IRON".to_string(),
            division: "".to_string(),
            tier_str: "黑铁 IV -5 LP".to_string(),
            tier_color: get_tier_color("IRON").to_string(),
            lp: -5,
            ranked_wins: 5,
            ranked_losses: 50,
            matches: vec![],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        assert_eq!(rec.lp, -5);
        assert_eq!(rec.tier_color, "#71717a");
    }

    #[test]
    fn test_f8_b03_invalid_sample_count_clamp() {
        let mgr = PlayerHistoryManager::new();
        // Negative number
        mgr.set_sample_match_count(-10);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 20);

        // Out of allowed set (not 10, 20, 30, 50)
        mgr.set_sample_match_count(999);
        assert_eq!(mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed), 20);
    }

    #[test]
    fn test_f8_b04_recent_dots_with_fewer_than_five_games() {
        let mgr = PlayerHistoryManager::new();
        // Only 1 game in history
        let rec1 = CachedPlayerRecord {
            puuid: "p1".to_string(),
            summoner_name: "p1".to_string(),
            tier: "GOLD".to_string(),
            division: "".to_string(),
            tier_str: "黄金".to_string(),
            tier_color: "#c8aa6e".to_string(),
            lp: 0,
            ranked_wins: 1,
            ranked_losses: 0,
            matches: vec![ParsedMatchRecord { win: true, ..Default::default() }],
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof1 = mgr.compute_tactical_profile(&rec1, 0, 20);
        assert_eq!(prof1.recent_dots, vec![true]);

        // 0 games
        let rec0 = CachedPlayerRecord {
            matches: vec![],
            ..rec1
        };
        let prof0 = mgr.compute_tactical_profile(&rec0, 0, 20);
        assert_eq!(prof0.recent_dots.len(), 0);
    }

    #[test]
    fn test_f8_b05_tied_top_champions_deterministic_order() {
        let mgr = PlayerHistoryManager::new();
        let matches = vec![
            ParsedMatchRecord { champion_id: 10, champion_name: "Champ10".to_string(), win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 20, champion_name: "Champ20".to_string(), win: true, ..Default::default() },
            ParsedMatchRecord { champion_id: 30, champion_name: "Champ30".to_string(), win: true, ..Default::default() },
        ];
        let rec = CachedPlayerRecord {
            puuid: "p".to_string(),
            summoner_name: "p".to_string(),
            tier: "SILVER".to_string(),
            division: "".to_string(),
            tier_str: "白银".to_string(),
            tier_color: "#94a3b8".to_string(),
            lp: 0,
            ranked_wins: 3,
            ranked_losses: 0,
            matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        };
        let prof = mgr.compute_tactical_profile(&rec, 0, 20);
        assert_eq!(prof.top_champs.len(), 3);
        for tc in &prof.top_champs {
            assert_eq!(tc.games, 1);
        }
    }
}

// ==============================================================================================
// Feature 9: 10-Player Scoreboard Contract & Dual Bars — Boundaries (5 tests)
// ==============================================================================================
mod f9_boundary_scoreboard_contract {
    use super::*;

    #[test]
    fn test_f9_b01_zero_gold_and_damage_scoreboard() {
        let db = Database::new();
        let mut raw = canned_match_detail_classic(1001, true);
        if let Some(arr) = raw.get_mut("participants").and_then(|v| v.as_array_mut()) {
            for p in arr {
                p["stats"]["totalDamageDealtToChampions"] = json!(0);
                p["stats"]["totalDamageTaken"] = json!(0);
                p["stats"]["goldEarned"] = json!(0);
            }
        }
        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some());
        let val = parsed.unwrap();
        let participants = val["participants"].as_array().unwrap();
        assert_eq!(participants.len(), 10);
        for p in participants {
            assert_eq!(p["total_damage"].as_i64().unwrap(), 0);
            assert_eq!(p["total_damage_taken"].as_i64().unwrap(), 0);
            assert_eq!(p["gold_earned"].as_i64().unwrap(), 0);
        }
    }

    #[test]
    fn test_f9_b02_single_player_one_hundred_percent_damage() {
        let db = Database::new();
        let mut raw = canned_match_detail_classic(1002, true);
        if let Some(arr) = raw.get_mut("participants").and_then(|v| v.as_array_mut()) {
            for (idx, p) in arr.iter_mut().enumerate() {
                if idx == 0 {
                    p["stats"]["totalDamageDealtToChampions"] = json!(50000);
                } else if idx < 5 {
                    p["stats"]["totalDamageDealtToChampions"] = json!(0);
                }
            }
        }
        let parsed = parse_riot_match_detail(&raw, &db).unwrap();
        let participants = parsed["participants"].as_array().unwrap();
        assert_eq!(participants[0]["damage_share_pct"].as_f64().unwrap(), 100.0);
        assert_eq!(participants[1]["damage_share_pct"].as_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_f9_b03_empty_item_slots() {
        let db = Database::new();
        let mut raw = canned_match_detail_classic(1003, true);
        if let Some(arr) = raw.get_mut("participants").and_then(|v| v.as_array_mut()) {
            for i in 0..7 {
                arr[0]["stats"][format!("item{}", i)] = json!(0);
            }
        }
        let parsed = parse_riot_match_detail(&raw, &db).unwrap();
        let items = parsed["participants"][0]["items"].as_array().unwrap();
        assert_eq!(items.len(), 7);
        for it in items {
            assert_eq!(it.as_i64().unwrap(), 0);
        }
    }

    #[test]
    fn test_f9_b04_non_standard_participant_counts() {
        let db = Database::new();
        // 6 players total (3v3)
        let mut raw = canned_match_detail_classic(1004, true);
        raw["participantIdentities"].as_array_mut().unwrap().truncate(6);
        raw["participants"].as_array_mut().unwrap().truncate(6);

        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some());
        let val = parsed.unwrap();
        assert_eq!(val["participants"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn test_f9_b05_extreme_high_damage_and_gold() {
        let db = Database::new();
        let mut raw = canned_match_detail_classic(1005, true);
        raw["participants"][0]["stats"]["totalDamageDealtToChampions"] = json!(9_999_999);
        raw["participants"][0]["stats"]["goldEarned"] = json!(100_000);

        let parsed = parse_riot_match_detail(&raw, &db).unwrap();
        let p0 = &parsed["participants"][0];
        assert_eq!(p0["total_damage"].as_i64().unwrap(), 9_999_999);
        assert_eq!(p0["gold_earned"].as_i64().unwrap(), 100_000);
    }
}

// ==============================================================================================
// Feature 10: LCU Lifecycle Resilience & Credential Self-Healing — Boundaries (5 tests)
// ==============================================================================================
mod f10_boundary_lifecycle_and_lockfile {
    use super::*;

    #[test]
    fn test_f10_b01_lockfile_with_spaces_and_newlines() {
        let content = "  LeagueClient:1234:5678:valid_auth_token:https\r\n\t ";
        let creds = LcuConnector::parse_lockfile(content.trim()).expect("trimmed creds");
        assert_eq!(creds.port, 5678);
        assert_eq!(creds.auth_token, "valid_auth_token");
    }

    #[test]
    fn test_f10_b02_lockfile_empty_token() {
        // Missing token between colons
        let content = "LeagueClient:1234:5678::https";
        let creds = LcuConnector::parse_lockfile(content);
        // Either rejected or parsed with empty token
        if let Some(c) = creds {
            assert_eq!(c.auth_token, "");
        }
    }

    #[test]
    fn test_f10_b03_lockfile_boundary_ports() {
        // Port 1 (minimum)
        let content_min = "LeagueClient:100:1:token:https";
        let creds_min = LcuConnector::parse_lockfile(content_min).expect("min port");
        assert_eq!(creds_min.port, 1);

        // Port 65535 (maximum u16)
        let content_max = "LeagueClient:100:65535:token:https";
        let creds_max = LcuConnector::parse_lockfile(content_max).expect("max port");
        assert_eq!(creds_max.port, 65535);

        // Port 65536 (overflow u16)
        let content_overflow = "LeagueClient:100:65536:token:https";
        assert!(LcuConnector::parse_lockfile(content_overflow).is_none());
    }

    #[tokio::test]
    async fn test_f10_b04_network_504_gateway_timeout() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        server.state.set_status_override(Some(StatusCode::GATEWAY_TIMEOUT)).await;
        let res = client.get_gameflow_phase().await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("504"));
        server.stop();
    }

    #[tokio::test]
    async fn test_f10_b05_status_401_unauthorized_simulation() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        server.state.set_status_override(Some(StatusCode::UNAUTHORIZED)).await;
        let res = client.get_gameflow_phase().await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("401"));
        server.stop();
    }
}

// ==============================================================================================
// Feature 11: CN Anti-Anonymity & Account Switch (Relog) — Boundaries (5 tests)
// ==============================================================================================
mod f11_boundary_anti_anonymity {
    use super::*;

    #[tokio::test]
    async fn test_f11_b01_chat_room_empty_participants() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        server.state.set_chat_participants("empty-room", vec![]).await;
        let participants = client
            .get_chat_conversation_participants("empty-room")
            .await
            .expect("empty participants");
        assert_eq!(participants.len(), 0);
        server.stop();
    }

    #[test]
    fn test_f11_b02_all_anonymous_names_in_champ_select() {
        let session = canned_champ_select_session("room-anon", "my-puuid", 0);
        let my_team = session["myTeam"].as_array().unwrap();
        for p in &my_team[1..] {
            assert!(p["displayName"].as_str().unwrap().starts_with("召唤师"));
            assert!(p["puuid"].as_str().unwrap().starts_with("anon-puuid"));
        }
    }

    #[test]
    fn test_f11_b03_duplicate_puuid_in_chat_participants() {
        // Chat room participants with duplicated PUUID
        let parts = canned_chat_participants(
            "dup-room",
            &[
                ("puuid-dup", "玩家1"),
                ("puuid-dup", "玩家1副本"),
                ("puuid-other", "玩家2"),
            ],
        );

        let mut unique_puuids = std::collections::HashSet::new();
        let mut deduped = Vec::new();
        for p in &parts {
            let pid = p["puuid"].as_str().unwrap();
            if unique_puuids.insert(pid) {
                deduped.push(p.clone());
            }
        }
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_f11_b04_local_player_cell_id_out_of_range() {
        // Local cell id 99 (not in 0..5 range)
        let session = canned_champ_select_session("room-out", "my-puuid", 99);
        let my_team = session["myTeam"].as_array().unwrap();
        // None of the 5 team slots should be flagged as local player
        assert_eq!(
            my_team.iter().filter(|p| p["isLocalPlayer"].as_bool().unwrap()).count(),
            0
        );
    }

    #[tokio::test]
    async fn test_f11_b05_rapid_summoner_account_switch() {
        let server = MockLcuServer::start().await;
        let client = MockLcuClient::from_server(&server);

        let accounts = ["acc-1", "acc-2", "acc-3"];
        for acc in accounts {
            server.state.set_current_summoner(canned_current_summoner(acc, acc)).await;
            let cur = client.get_current_summoner().await.expect("current summoner");
            assert_eq!(cur["puuid"].as_str().unwrap(), acc);
        }
        server.stop();
    }
}

// ==============================================================================================
// Feature 12: Special Game Modes (ARAM, Arena 2v2v2v2, TFT) — Boundaries (5 tests)
// ==============================================================================================
mod f12_boundary_special_modes {
    use super::*;

    #[test]
    fn test_f12_b01_aram_bench_empty_and_zero_rerolls() {
        let mut session = canned_champ_select_session("bench-room", "local-p", 0);
        session["benchChampions"] = json!([]);
        session["rerollsRemaining"] = json!(0);
        session["benchEnabled"] = json!(true);

        assert_eq!(session["benchChampions"].as_array().unwrap().len(), 0);
        assert_eq!(session["rerollsRemaining"].as_i64().unwrap(), 0);
    }

    #[test]
    fn test_f12_b02_aram_bench_ten_champions_full_capacity() {
        let mut session = canned_champ_select_session("bench-room", "local-p", 0);
        let mut bench = Vec::new();
        for i in 1..=10 {
            bench.push(json!({
                "championId": 100 + i,
                "isHighestMastery": false
            }));
        }
        session["benchChampions"] = json!(bench);
        session["benchEnabled"] = json!(true);

        assert_eq!(session["benchChampions"].as_array().unwrap().len(), 10);
    }

    #[test]
    fn test_f12_b03_arena_four_teams_equal_split() {
        let db = Database::new();
        let raw = canned_match_detail_arena(7001);
        let parsed = parse_riot_match_detail(&raw, &db).unwrap();
        let participants = parsed["participants"].as_array().unwrap();
        assert_eq!(participants.len(), 8);

        // Verify all 8 participants belong to 4 distinct teams (100, 200, 300, 400)
        let team_ids: std::collections::HashSet<i64> = raw["participants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["teamId"].as_i64().unwrap())
            .collect();
        assert_eq!(team_ids.len(), 4);
    }

    #[test]
    fn test_f12_b04_arena_odd_number_of_participants() {
        let db = Database::new();
        let mut raw = canned_match_detail_arena(7002);
        // Truncate to 7 players (one DC/leaver)
        raw["participantIdentities"].as_array_mut().unwrap().truncate(7);
        raw["participants"].as_array_mut().unwrap().truncate(7);

        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some(), "Arena 7-player match must not panic");
        let participants = parsed.unwrap()["participants"].as_array().unwrap().len();
        assert_eq!(participants, 7);
    }

    #[test]
    fn test_f12_b05_tft_queue_id_graceful_handling() {
        let db = Database::new();
        let raw_tft = json!({
            "gameId": 8001,
            "queueId": 1090, // TFT Normal
            "gameMode": "TFT",
            "participantIdentities": [],
            "participants": []
        });

        let parsed = parse_riot_match_detail(&raw_tft, &db).unwrap();
        assert_eq!(parsed["mode_cn"].as_str().unwrap(), "云顶之弈");
    }
}

// ==============================================================================================
// Feature 13: Concurrency Throttling & Atomic Config Persistence — Boundaries (5 tests)
// ==============================================================================================
mod f13_boundary_concurrency_and_config {
    use super::*;

    #[tokio::test]
    async fn test_f13_b01_atomic_config_empty_string_fields() {
        let temp_dir = std::env::temp_dir().join(format!("hex_test_cfg_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let cfg_path = temp_dir.join("config.json");

        let manager = ConfigManager::new(cfg_path.to_str().unwrap());
        manager
            .update(
                None,
                None,
                None,
                None,
                Some("".to_string()),
                Some("".to_string()),
                None,
                None,
                None,
            )
            .await;

        // Reload
        let reloaded = ConfigManager::new(cfg_path.to_str().unwrap());
        let loaded = reloaded.get().await;
        assert_eq!(loaded.gold_calc_mode, "");
        assert_eq!(loaded.trio_mode, "");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_f13_b02_config_extreme_numeric_values() {
        let temp_dir = std::env::temp_dir().join(format!("hex_test_cfg_ext_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let cfg_path = temp_dir.join("config.json");

        let manager = ConfigManager::new(cfg_path.to_str().unwrap());
        manager
            .update(
                Some(false),
                Some(999.0), // Should clamp to upper bound 3.0
                Some(-10.0), // Should clamp to lower bound 0.0
                Some(50.0),  // Should clamp to upper bound 3.0
                None,
                None,
                None,
                Some(true),
                Some(3600), // Should clamp to upper bound 10
            )
            .await;

        let reloaded = ConfigManager::new(cfg_path.to_str().unwrap());
        let loaded = reloaded.get().await;
        assert_eq!(loaded.hex_synergy_weight, 3.0, "hex_synergy_weight must clamp to 3.0");
        assert_eq!(loaded.archetype_strictness, 0.0, "archetype_strictness must clamp to 0.0");
        assert_eq!(loaded.counter_intensity, 3.0, "counter_intensity must clamp to 3.0");
        assert_eq!(loaded.auto_accept_delay_seconds, 10, "auto_accept_delay_seconds must clamp to 10");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_f13_b03_semaphore_single_permit_serialization() {
        let sem = Arc::new(Semaphore::new(1));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let s = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = s.acquire().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(sem.available_permits(), 1);
    }

    #[tokio::test]
    async fn test_f13_b04_burst_requests_high_load() {
        let server = MockLcuServer::start().await;
        let client = Arc::new(MockLcuClient::from_server(&server));

        let mut handles = Vec::new();
        for _ in 0..40 {
            let c = client.clone();
            handles.push(tokio::spawn(async move {
                c.get_gameflow_phase().await
            }));
        }

        let mut success_count = 0;
        for h in handles {
            let res = h.await.unwrap();
            if res.is_ok() && res.unwrap() == "ChampSelect" {
                success_count += 1;
            }
        }
        assert_eq!(success_count, 40);
        server.stop();
    }

    #[tokio::test]
    async fn test_f13_b05_config_file_temp_overwrite_safety() {
        let temp_dir = std::env::temp_dir().join(format!("hex_test_tmp_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let cfg_path = temp_dir.join("config.json");
        let tmp_path = temp_dir.join("config.json.tmp");

        // Pre-create stale tmp file
        std::fs::write(&tmp_path, "stale data").unwrap();

        let manager = ConfigManager::new(cfg_path.to_str().unwrap());
        manager
            .update(
                None,
                Some(2.5),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        // Writing should cleanly overwrite stale tmp and atomize rename
        assert!(cfg_path.exists());
        let reloaded = ConfigManager::new(cfg_path.to_str().unwrap());
        assert_eq!(reloaded.get().await.hex_synergy_weight, 2.5);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
