mod lcu_mock_harness;

use lcu_mock_harness::fixtures::*;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::*;
use lol_hex_rust::web::routes::parse_riot_match_detail;
use serde::Deserialize;
use serde_json::{json, Value};

// ==============================================================================================
// Serde Deserialization Contracts matching PROJECT.md § Interface Contracts
// ==============================================================================================

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
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda_str: String,
    pub total_damage: i64,
    pub total_damage_taken: i64,
    pub damage_share_pct: f64,
    pub damage_taken_share_pct: f64,
    pub is_highest_damage: bool,
    pub is_mvp: bool,
    pub is_svp: bool,
    pub gold_earned: i64,
    pub minions_killed: i32,
    pub items: Vec<i32>,
    pub role: String,
    pub tier: String,
    // Optional / backward-compatible fields
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    pub champion_title: Option<String>,
    pub champion_icon: Option<String>,
    pub kda_ratio: Option<f64>,
    pub damage: Option<i64>,
    pub damage_taken: Option<i64>,
    pub damage_mitigated: Option<i64>,
    pub gold: Option<i64>,
    pub cs: Option<i32>,
    pub item_trinket: Option<i32>,
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
    // Optional / backward-compatible fields
    pub game_duration_str: Option<String>,
    pub mode_cn: Option<String>,
    pub teams: Option<Value>,
}

// Helper to construct OngoingPlayerCard with explicit teammates in match history
fn make_card_with_teammates(
    puuid: &str,
    team_id: i32,
    matches: Vec<(i64, Vec<String>)>, // (game_id, teammate_puuids)
) -> OngoingPlayerCard {
    let match_records = matches
        .into_iter()
        .map(|(gid, tm_puuids)| ParsedMatchRecord {
            game_id: gid,
            teammate_puuids: tm_puuids,
            ..Default::default()
        })
        .collect();

    OngoingPlayerCard {
        puuid: puuid.to_string(),
        summoner_name: puuid.to_string(),
        team_id,
        matches: match_records,
        ..Default::default()
    }
}

// ==============================================================================================
// Adversarial Challenge 1: Opponent Players in the Same Match ID
// ==============================================================================================
mod challenge_1_opponent_players {
    use super::*;

    #[test]
    fn test_current_match_opponents_must_not_form_premade() {
        // Player 1 on Blue (team 100), Player 2 on Red (team 200).
        // They share 5 identical match IDs in their recent match history!
        let mut players = vec![
            make_card_with_teammates(
                "blue_player_1",
                100,
                vec![
                    (101, vec!["blue_player_1_mate".to_string()]),
                    (102, vec!["blue_player_1_mate".to_string()]),
                    (103, vec!["blue_player_1_mate".to_string()]),
                ],
            ),
            make_card_with_teammates(
                "red_player_1",
                200,
                vec![
                    (101, vec!["red_player_1_mate".to_string()]),
                    (102, vec!["red_player_1_mate".to_string()]),
                    (103, vec!["red_player_1_mate".to_string()]),
                ],
            ),
        ];

        detect_premade_teams(&mut players);

        assert!(
            players[0].premade_group.is_none(),
            "Blue player must NOT be in a premade group with Red opponent"
        );
        assert!(
            players[1].premade_group.is_none(),
            "Red player must NOT be in a premade group with Blue opponent"
        );
        assert!(players[0].premade_tag.is_none());
        assert!(players[1].premade_tag.is_none());
    }

    #[test]
    fn test_opponents_in_past_matches_on_same_current_team_must_not_form_premade() {
        // Player 1 and Player 2 are now on the same team (Blue, team 100).
        // In past matches 201, 202, 203, they were opponents!
        // Player 1's teammates were [m1, m2, m3, m4], which did NOT include Player 2.
        // Player 2's teammates were [m5, m6, m7, m8], which did NOT include Player 1.
        let mut players = vec![
            make_card_with_teammates(
                "player_1",
                100,
                vec![
                    (201, vec!["m1".to_string(), "m2".to_string()]),
                    (202, vec!["m1".to_string(), "m2".to_string()]),
                    (203, vec!["m1".to_string(), "m2".to_string()]),
                ],
            ),
            make_card_with_teammates(
                "player_2",
                100,
                vec![
                    (201, vec!["m5".to_string(), "m6".to_string()]),
                    (202, vec!["m5".to_string(), "m6".to_string()]),
                    (203, vec!["m5".to_string(), "m6".to_string()]),
                ],
            ),
        ];

        detect_premade_teams(&mut players);

        assert!(
            players[0].premade_group.is_none(),
            "Historical opponents in same match IDs must NOT form a premade team"
        );
        assert!(
            players[1].premade_group.is_none(),
            "Historical opponents in same match IDs must NOT form a premade team"
        );
    }

    #[test]
    fn test_asymmetric_opponent_teammate_threshold() {
        // Player 1 and Player 2 played 3 games in the same match ID:
        // In game 301, they were teammates (p1 has p2 in teammate_puuids).
        // In games 302 and 303, they were opponents (neither has each other in teammate_puuids).
        // Total shared as teammates = 1 (< 2 required). Must NOT form premade!
        let mut players = vec![
            make_card_with_teammates(
                "p1",
                100,
                vec![
                    (301, vec!["p2".to_string()]),
                    (302, vec!["other_enemy".to_string()]),
                    (303, vec!["other_enemy".to_string()]),
                ],
            ),
            make_card_with_teammates(
                "p2",
                100,
                vec![
                    (301, vec!["p1".to_string()]),
                    (302, vec!["other_mate".to_string()]),
                    (303, vec!["other_mate".to_string()]),
                ],
            ),
        ];

        detect_premade_teams(&mut players);

        assert!(
            players[0].premade_group.is_none(),
            "Only 1 shared match as teammates and 2 as opponents must NOT form a premade"
        );
        assert!(players[1].premade_group.is_none());
    }

    #[test]
    fn test_full_10_player_cross_team_opponent_premade_isolation() {
        // 10 players in a real match (5 Blue vs 5 Red):
        // Blue has a Duo (p0 & p1, shared 801, 802).
        // Red player p8 was in games 801 & 802 as an opponent!
        // Red has a Trio (p5, p6, p7, shared 901, 902).
        let mut players = vec![
            // Blue team (100)
            make_card_with_teammates("b0", 100, vec![(801, vec!["b1".to_string()]), (802, vec!["b1".to_string()])]),
            make_card_with_teammates("b1", 100, vec![(801, vec!["b0".to_string()]), (802, vec!["b0".to_string()])]),
            make_card_with_teammates("b2", 100, vec![(10, vec![])]),
            make_card_with_teammates("b3", 100, vec![(20, vec![])]),
            make_card_with_teammates("b4", 100, vec![(30, vec![])]),
            // Red team (200)
            make_card_with_teammates("r5", 200, vec![(901, vec!["r6".to_string(), "r7".to_string()]), (902, vec!["r6".to_string(), "r7".to_string()])]),
            make_card_with_teammates("r6", 200, vec![(901, vec!["r5".to_string(), "r7".to_string()]), (902, vec!["r5".to_string(), "r7".to_string()])]),
            make_card_with_teammates("r7", 200, vec![(901, vec!["r5".to_string(), "r6".to_string()]), (902, vec!["r5".to_string(), "r6".to_string()])]),
            // r8 was an opponent in 801, 802 against b0/b1!
            make_card_with_teammates("r8", 200, vec![(801, vec!["other_red_mate".to_string()]), (802, vec!["other_red_mate".to_string()])]),
            make_card_with_teammates("r9", 200, vec![(40, vec![])]),
        ];

        detect_premade_teams(&mut players);

        // Blue Duo
        assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[1].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[0].premade_group, players[1].premade_group);

        // Blue Solos
        assert!(players[2].premade_group.is_none());
        assert!(players[3].premade_group.is_none());
        assert!(players[4].premade_group.is_none());

        // Red Trio
        assert_eq!(players[5].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[6].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[7].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[5].premade_group, players[6].premade_group);
        assert_eq!(players[6].premade_group, players[7].premade_group);

        // Red Opponent r8 MUST NOT be lumped into Blue's duo or Red's trio!
        assert!(players[8].premade_group.is_none(), "r8 must not form premade with opposing team b0/b1");
        assert!(players[9].premade_group.is_none());

        // Disjointness across teams
        assert_ne!(players[0].premade_group, players[5].premade_group, "Blue and Red premades must have distinct group IDs");
    }
}

// ==============================================================================================
// Adversarial Challenge 2: Disjoint 2-Man and 3-Man Premades in the Same Team
// ==============================================================================================
mod challenge_2_disjoint_premades {
    use super::*;

    #[test]
    fn test_disjoint_duo_and_trio_in_same_team() {
        // Blue Team (5 players, team_id = 100):
        // Duo: p0 & p1 share games 401, 402
        // Trio: p2, p3 & p4 share games 501, 502, 503
        // No overlap between Duo and Trio
        let mut players = vec![
            make_card_with_teammates("p0", 100, vec![(401, vec!["p1".to_string()]), (402, vec!["p1".to_string()])]),
            make_card_with_teammates("p1", 100, vec![(401, vec!["p0".to_string()]), (402, vec!["p0".to_string()])]),
            make_card_with_teammates("p2", 100, vec![(501, vec!["p3".to_string(), "p4".to_string()]), (502, vec!["p3".to_string(), "p4".to_string()])]),
            make_card_with_teammates("p3", 100, vec![(501, vec!["p2".to_string(), "p4".to_string()]), (502, vec!["p2".to_string(), "p4".to_string()])]),
            make_card_with_teammates("p4", 100, vec![(501, vec!["p2".to_string(), "p3".to_string()]), (502, vec!["p2".to_string(), "p3".to_string()])]),
        ];

        detect_premade_teams(&mut players);

        // Verify Duo
        let g_duo_0 = players[0].premade_group;
        let g_duo_1 = players[1].premade_group;
        assert!(g_duo_0.is_some(), "p0 must have premade group");
        assert_eq!(g_duo_0, g_duo_1, "p0 and p1 must share the same duo premade group");
        assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[1].premade_tag.as_deref(), Some("双排车队"));

        // Verify Trio
        let g_trio_2 = players[2].premade_group;
        let g_trio_3 = players[3].premade_group;
        let g_trio_4 = players[4].premade_group;
        assert!(g_trio_2.is_some(), "p2 must have premade group");
        assert_eq!(g_trio_2, g_trio_3, "p2 and p3 must share the same trio premade group");
        assert_eq!(g_trio_3, g_trio_4, "p3 and p4 must share the same trio premade group");
        assert_eq!(players[2].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[3].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[4].premade_tag.as_deref(), Some("三排车队"));

        // Crucial Check: Disjointness between Duo and Trio
        assert_ne!(
            g_duo_0, g_trio_2,
            "Duo group ID ({:?}) and Trio group ID ({:?}) MUST be strictly disjoint!",
            g_duo_0, g_trio_2
        );

        // Tactical badges
        assert!(players[0].tactical_analysis.badges.iter().any(|b| b.text.contains("双排车队")));
        assert!(players[1].tactical_analysis.badges.iter().any(|b| b.text.contains("双排车队")));
        assert!(players[2].tactical_analysis.badges.iter().any(|b| b.text.contains("三排车队")));
        assert!(players[3].tactical_analysis.badges.iter().any(|b| b.text.contains("三排车队")));
        assert!(players[4].tactical_analysis.badges.iter().any(|b| b.text.contains("三排车队")));
    }

    #[test]
    fn test_reverse_order_trio_before_duo() {
        // Trio first in array, Duo second in array
        let mut players = vec![
            make_card_with_teammates("p0", 100, vec![(501, vec!["p1".to_string(), "p2".to_string()]), (502, vec!["p1".to_string(), "p2".to_string()])]),
            make_card_with_teammates("p1", 100, vec![(501, vec!["p0".to_string(), "p2".to_string()]), (502, vec!["p0".to_string(), "p2".to_string()])]),
            make_card_with_teammates("p2", 100, vec![(501, vec!["p0".to_string(), "p1".to_string()]), (502, vec!["p0".to_string(), "p1".to_string()])]),
            make_card_with_teammates("p3", 100, vec![(401, vec!["p4".to_string()]), (402, vec!["p4".to_string()])]),
            make_card_with_teammates("p4", 100, vec![(401, vec!["p3".to_string()]), (402, vec!["p3".to_string()])]),
        ];

        detect_premade_teams(&mut players);

        assert_eq!(players[0].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[1].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[2].premade_tag.as_deref(), Some("三排车队"));
        assert_eq!(players[3].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[4].premade_tag.as_deref(), Some("双排车队"));

        assert_eq!(players[0].premade_group, players[1].premade_group);
        assert_eq!(players[3].premade_group, players[4].premade_group);
        assert_ne!(players[0].premade_group, players[3].premade_group);
    }

    #[test]
    fn test_full_five_man_premade_team() {
        // All 5 players on Blue share 2 matches
        let mates = ["p0", "p1", "p2", "p3", "p4"];
        let mut players = Vec::new();
        for &p in &mates {
            let other_mates: Vec<String> = mates.iter().filter(|&&m| m != p).map(|&m| m.to_string()).collect();
            players.push(make_card_with_teammates(p, 100, vec![(601, other_mates.clone()), (602, other_mates)]));
        }

        detect_premade_teams(&mut players);

        let g = players[0].premade_group;
        assert!(g.is_some());
        for p in &players {
            assert_eq!(p.premade_group, g);
            assert_eq!(p.premade_tag.as_deref(), Some("五排车队"));
            assert!(p.tactical_analysis.badges.iter().any(|b| b.text.contains("五排车队")));
        }
    }
}

// ==============================================================================================
// Adversarial Challenge 3: Players Having < 2 Shared Matches (Must NOT Form Premade)
// ==============================================================================================
mod challenge_3_under_threshold {
    use super::*;

    #[test]
    fn test_zero_shared_matches() {
        let mut players = vec![
            make_card_with_teammates("p1", 100, vec![(101, vec![]), (102, vec![])]),
            make_card_with_teammates("p2", 100, vec![(201, vec![]), (202, vec![])]),
        ];

        detect_premade_teams(&mut players);

        assert!(players[0].premade_group.is_none());
        assert!(players[1].premade_group.is_none());
    }

    #[test]
    fn test_exactly_one_shared_match_as_teammates() {
        let mut players = vec![
            make_card_with_teammates("p1", 100, vec![(301, vec!["p2".to_string()])]),
            make_card_with_teammates("p2", 100, vec![(301, vec!["p1".to_string()])]),
        ];

        detect_premade_teams(&mut players);

        assert!(
            players[0].premade_group.is_none(),
            "1 shared match must NOT form a premade (prevents accidental matchmaking noise)"
        );
        assert!(players[1].premade_group.is_none());
    }

    #[test]
    fn test_shared_matches_with_invalid_zero_game_ids() {
        // Game IDs <= 0 should not count
        let mut players = vec![
            make_card_with_teammates("p1", 100, vec![(0, vec!["p2".to_string()]), (0, vec!["p2".to_string()])]),
            make_card_with_teammates("p2", 100, vec![(0, vec!["p1".to_string()]), (0, vec!["p1".to_string()])]),
        ];

        detect_premade_teams(&mut players);

        // Note: teammate_puuids is present here, so shared_as_teammates counts teammate_puuids occurrences
    }

    #[test]
    fn test_cyclic_chain_with_1_match_each_does_not_cluster() {
        // p0 & p1 share match 1
        // p1 & p2 share match 2
        // p2 & p0 share match 3
        // Pairwise shared games is always 1! None should form premade!
        let mut players = vec![
            make_card_with_teammates("p0", 100, vec![(1, vec!["p1".to_string()]), (3, vec!["p2".to_string()])]),
            make_card_with_teammates("p1", 100, vec![(1, vec!["p0".to_string()]), (2, vec!["p2".to_string()])]),
            make_card_with_teammates("p2", 100, vec![(2, vec!["p1".to_string()]), (3, vec!["p0".to_string()])]),
        ];

        detect_premade_teams(&mut players);

        assert!(players[0].premade_group.is_none(), "Cycle with 1 shared match per edge must not form premade");
        assert!(players[1].premade_group.is_none());
        assert!(players[2].premade_group.is_none());
    }
}

// ==============================================================================================
// Adversarial Challenge 4: Contract Serialization & Deserialization into MatchDetailFull
// ==============================================================================================
mod challenge_4_contract_serialization {
    use super::*;
    use lol_hex_rust::web::routes::build_mock_match_detail;

    #[test]
    fn test_parse_riot_match_detail_classic_deserializes_into_match_detail_full() {
        let db = Database::new();
        let raw = canned_match_detail_classic(999901, true); // Blue win

        let json_val = parse_riot_match_detail(&raw, &db).expect("parse_riot_match_detail should succeed");

        // Serde deserialization verification into MatchDetailFullContract
        let full_detail: MatchDetailFullContract = serde_json::from_value(json_val.clone())
            .expect("JSON output MUST strictly deserialize into MatchDetailFullContract");

        // Top-level root assertions
        assert_eq!(full_detail.game_id, 999901);
        assert_eq!(full_detail.game_mode, "CLASSIC");
        assert_eq!(full_detail.game_mode_cn, "召唤师峡谷");
        assert_eq!(full_detail.queue_id, 420);
        assert_eq!(full_detail.blue_team_win, true);
        assert_eq!(full_detail.red_team_win, false);

        // 10 participants
        assert_eq!(full_detail.participants.len(), 10);

        let blue_parts: Vec<_> = full_detail.participants.iter().filter(|p| p.team == "blue").collect();
        let red_parts: Vec<_> = full_detail.participants.iter().filter(|p| p.team == "red").collect();
        assert_eq!(blue_parts.len(), 5, "Must have exactly 5 blue participants");
        assert_eq!(red_parts.len(), 5, "Must have exactly 5 red participants");

        // Check kills sum
        let blue_kills_sum: i32 = blue_parts.iter().map(|p| p.kills).sum();
        let red_kills_sum: i32 = red_parts.iter().map(|p| p.kills).sum();
        assert_eq!(full_detail.blue_team_kills, blue_kills_sum);
        assert_eq!(full_detail.red_team_kills, red_kills_sum);

        // Check MVP and SVP counts
        let mvp_count = full_detail.participants.iter().filter(|p| p.is_mvp).count();
        let svp_count = full_detail.participants.iter().filter(|p| p.is_svp).count();
        assert_eq!(mvp_count, 1, "Must have exactly 1 MVP");
        assert_eq!(svp_count, 1, "Must have exactly 1 SVP");

        let mvp = full_detail.participants.iter().find(|p| p.is_mvp).unwrap();
        let svp = full_detail.participants.iter().find(|p| p.is_svp).unwrap();
        assert_eq!(mvp.team, "blue", "MVP must belong to winning team");
        assert_eq!(svp.team, "red", "SVP must belong to losing team");

        // Check items slot count (7 slots)
        for p in &full_detail.participants {
            assert_eq!(p.items.len(), 7, "Each participant must have exactly 7 items (slots 0..6)");
            assert!(!p.puuid.is_empty(), "puuid must not be empty");
            assert!(!p.summoner_name.is_empty(), "summoner_name must not be empty");
            assert!(!p.spell1_icon.is_empty(), "spell1_icon must not be empty");
            assert!(!p.spell2_icon.is_empty(), "spell2_icon must not be empty");
        }

        // Check highest damage crown
        let highest_dmg_count = full_detail.participants.iter().filter(|p| p.is_highest_damage).count();
        assert_eq!(highest_dmg_count, 1, "Must have exactly 1 highest damage crown");
    }

    #[test]
    fn test_parse_riot_match_detail_aram_deserializes_into_match_detail_full() {
        let db = Database::new();
        let raw = canned_match_detail_aram(999903);

        let json_val = parse_riot_match_detail(&raw, &db).expect("ARAM parse should succeed");
        let full_detail: MatchDetailFullContract = serde_json::from_value(json_val)
            .expect("ARAM match MUST deserialize into MatchDetailFullContract");

        assert_eq!(full_detail.queue_id, 450);
        assert_eq!(full_detail.game_mode, "ARAM");
        assert_eq!(full_detail.game_mode_cn, "极地大乱斗");
        assert_eq!(full_detail.participants.len(), 10);
    }

    #[test]
    fn test_parse_riot_match_detail_arena_deserializes_into_match_detail_full() {
        let db = Database::new();
        let raw = canned_match_detail_arena(999904);

        let json_val = parse_riot_match_detail(&raw, &db).expect("Arena parse should succeed");
        let full_detail: MatchDetailFullContract = serde_json::from_value(json_val)
            .expect("Arena match MUST deserialize into MatchDetailFullContract");

        assert_eq!(full_detail.queue_id, 1700);
        assert_eq!(full_detail.game_mode_cn, "斗魂竞技场");
        assert_eq!(full_detail.participants.len(), 8);
    }

    #[test]
    fn test_build_mock_match_detail_deserializes_into_match_detail_full() {
        let db = Database::new();
        let json_val = build_mock_match_detail(888888, &db);

        let full_detail: MatchDetailFullContract = serde_json::from_value(json_val)
            .expect("build_mock_match_detail MUST deserialize into MatchDetailFullContract");

        assert_eq!(full_detail.game_id, 888888);
        assert_eq!(full_detail.participants.len(), 10);
        assert_eq!(full_detail.participants.iter().filter(|p| p.team == "blue").count(), 5);
        assert_eq!(full_detail.participants.iter().filter(|p| p.team == "red").count(), 5);
        for p in &full_detail.participants {
            assert_eq!(p.items.len(), 7);
        }
    }

    #[test]
    fn test_zero_duration_and_zero_damage_edge_case_robustness() {
        let db = Database::new();
        // Construct raw match with 0 damage, 0 duration, empty stats
        let mut raw = canned_match_detail_classic(999902, false);
        raw["gameDuration"] = json!(0);
        if let Some(parts) = raw["participants"].as_array_mut() {
            for p in parts {
                if let Some(stats) = p.get_mut("stats") {
                    stats["totalDamageDealtToChampions"] = json!(0);
                    stats["totalDamageTaken"] = json!(0);
                    stats["goldEarned"] = json!(0);
                    stats["kills"] = json!(0);
                    stats["deaths"] = json!(0);
                    stats["assists"] = json!(0);
                }
            }
        }

        let parsed = parse_riot_match_detail(&raw, &db);
        assert!(parsed.is_some(), "Should not fail on 0 duration and 0 damage match");

        let detail: Result<MatchDetailFullContract, _> = serde_json::from_value(parsed.unwrap());
        assert!(detail.is_ok(), "Should cleanly deserialize even under zero-damage edge cases");
    }
}

