use lol_hex_rust::engine::player_history::{
    compute_radar, evaluate_match_participants, MatchParticipantInput, ParsedMatchRecord,
};

fn make_input(
    pid: i64,
    team_id: i64,
    is_win: bool,
    k: i32,
    d: i32,
    a: i32,
    dmg: i64,
    taken: i64,
    mit: i64,
    gold: i64,
    cs: i32,
    vis: i32,
) -> MatchParticipantInput {
    MatchParticipantInput {
        participant_id: pid,
        puuid: format!("puuid_{}", pid),
        team_id,
        is_win,
        kills: k,
        deaths: d,
        assists: a,
        damage: dmg,
        damage_taken: taken,
        damage_mitigated: mit,
        gold,
        cs,
        vision_score: vis,
    }
}

// ============================================================================
// Group 1: Radar Normalization & Clamping Tests
// ============================================================================

#[test]
fn test_radar_empty_matches() {
    let radar = compute_radar(&[]);
    assert_eq!(radar.len(), 6);
    let expected_keys = ["damage", "kda", "kp", "survival", "economy", "vision"];
    for key in &expected_keys {
        assert!(radar.contains_key(*key), "Radar missing key {}", key);
        let val = radar[*key];
        assert_eq!(val, 50, "Empty matches should default to 50 for {}", key);
        assert!((25..=99).contains(&val), "Value {} out of [25, 99]", val);
    }
}

#[test]
fn test_radar_all_zeros() {
    let zero_match = ParsedMatchRecord {
        game_id: 101,
        kills: 0,
        deaths: 0,
        assists: 0,
        damage: 0,
        damage_share_pct: 0.0,
        gold: 0,
        cs_per_min: 0.0,
        kp_pct: 0.0,
        vision_score: 0,
        ..Default::default()
    };

    let radar = compute_radar(&[zero_match]);
    assert_eq!(radar.len(), 6);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must be within [25, 99] for all zeros",
            axis,
            score
        );
    }
    // 0 deaths -> 100 - 0 = 100 -> clamped to 99
    assert_eq!(radar["survival"], 99);
    // Other axes with 0 -> clamped to 25
    assert_eq!(radar["damage"], 25);
    assert_eq!(radar["kda"], 25);
    assert_eq!(radar["kp"], 25);
    assert_eq!(radar["economy"], 25);
    assert_eq!(radar["vision"], 25);
}

#[test]
fn test_radar_extreme_high_values() {
    let hyper_match = ParsedMatchRecord {
        game_id: 999,
        kills: 1000,
        deaths: 0,
        assists: 500,
        damage: 10_000_000,
        damage_share_pct: 99.9,
        gold: 5_000_000,
        cs_per_min: 200.0,
        kp_pct: 100.0,
        vision_score: 5000,
        ..Default::default()
    };

    let radar = compute_radar(&[hyper_match]);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must strictly be in [25, 99] on hyper values",
            axis,
            score
        );
        assert_eq!(score, 99, "Axis {} should saturate to ceiling 99", axis);
    }
}

#[test]
fn test_radar_extreme_low_and_high_deaths() {
    let feeder_match = ParsedMatchRecord {
        game_id: 102,
        kills: 0,
        deaths: 100, // Massive deaths -> survival should saturate to 25
        assists: 0,
        damage: 10,
        damage_share_pct: 0.1,
        gold: 100,
        cs_per_min: 0.1,
        kp_pct: 0.0,
        vision_score: 0,
        ..Default::default()
    };

    let radar = compute_radar(&[feeder_match]);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must be within [25, 99]",
            axis,
            score
        );
    }
    assert_eq!(radar["survival"], 25, "Survival score must saturate to floor 25 on 100 deaths");
}

#[test]
fn test_radar_negative_values() {
    let neg_match = ParsedMatchRecord {
        game_id: 103,
        kills: -5,
        deaths: 50, // Positive high deaths to verify lower clamp for survival
        assists: -2,
        damage: -50_000,
        damage_share_pct: -20.0,
        gold: -10_000,
        cs_per_min: -5.0,
        kp_pct: -50.0,
        vision_score: -100,
        ..Default::default()
    };

    let radar = compute_radar(&[neg_match]);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must be within [25, 99] despite negative inputs",
            axis,
            score
        );
        assert_eq!(score, 25, "Axis {} score {} should clamp to floor 25 on negative stats", axis, score);
    }

    // Negative deaths test: -10 deaths produces 100 - (-10 * 9.5) = 195 -> clamps to 99
    let neg_deaths_match = ParsedMatchRecord {
        game_id: 104,
        deaths: -10,
        ..Default::default()
    };
    let radar_neg_d = compute_radar(&[neg_deaths_match]);
    assert_eq!(radar_neg_d["survival"], 99);
}

#[test]
fn test_radar_nan_and_inf_inputs() {
    let nan_match = ParsedMatchRecord {
        game_id: 105,
        kills: 10,
        deaths: 2,
        assists: 5,
        damage: 20000,
        damage_share_pct: f64::NAN,
        gold: 12000,
        cs_per_min: f64::NAN,
        kp_pct: f64::NAN,
        vision_score: 30,
        ..Default::default()
    };

    let radar = compute_radar(&[nan_match]);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must be within [25, 99] under NaN",
            axis,
            score
        );
    }

    let inf_match = ParsedMatchRecord {
        game_id: 106,
        kills: 10,
        deaths: 2,
        assists: 5,
        damage: 20000,
        damage_share_pct: f64::INFINITY,
        gold: 12000,
        cs_per_min: f64::INFINITY,
        kp_pct: f64::NEG_INFINITY,
        vision_score: 30,
        ..Default::default()
    };

    let radar_inf = compute_radar(&[inf_match]);
    for (axis, &score) in &radar_inf {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} must be within [25, 99] under Inf",
            axis,
            score
        );
    }
}

#[test]
fn test_radar_multi_match_averaging() {
    let mut matches = Vec::new();
    for i in 0..20 {
        matches.push(ParsedMatchRecord {
            game_id: i + 1,
            kills: (i % 10) as i32,
            deaths: ((i + 1) % 8) as i32,
            assists: (i * 2 % 15) as i32,
            damage: ((i + 1) * 2000) as i64,
            damage_share_pct: (10.0 + (i as f64) * 1.5).min(40.0),
            gold: ((i + 1) * 1000) as i64,
            cs_per_min: 4.0 + (i as f64) * 0.2,
            kp_pct: 30.0 + (i as f64) * 2.0,
            vision_score: 10 + (i as i32) * 2,
            ..Default::default()
        });
    }

    let radar = compute_radar(&matches);
    assert_eq!(radar.len(), 6);
    for (axis, &score) in &radar {
        assert!(
            (25..=99).contains(&score),
            "Axis {} score {} in 20 matches must be in [25, 99]",
            axis,
            score
        );
    }
}

// ============================================================================
// Group 2: MVP/SVP Cardinality and Assignment Tests
// ============================================================================

#[test]
fn test_evaluate_empty_slice() {
    let result = evaluate_match_participants(1800, &[]);
    assert!(result.is_empty(), "Empty input must return empty evaluations");
}

#[test]
fn test_evaluate_standard_5v5_exact_cardinality() {
    let mut inputs = Vec::new();
    // Team 100 (winners)
    for i in 1..=5 {
        inputs.push(make_input(i, 100, true, i as i32 * 2, 2, 5, i * 6000, 10000, 5000, 10000 + i * 500, 150, 20));
    }
    // Team 200 (losers)
    for i in 6..=10 {
        inputs.push(make_input(i, 200, false, (11 - i) as i32, 5, 2, (11 - i) * 5000, 15000, 3000, 8000, 120, 15));
    }

    let evals = evaluate_match_participants(1800, &inputs);
    assert_eq!(evals.len(), 10);

    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 1, "Must have exactly 1 MVP");
    assert_eq!(svp_count, 1, "Must have exactly 1 SVP");

    let mvp = evals.iter().find(|e| e.is_mvp).unwrap();
    let svp = evals.iter().find(|e| e.is_svp).unwrap();

    assert_eq!(mvp.team_id, 100, "MVP must belong to the winning team");
    assert!(!mvp.is_svp, "MVP cannot simultaneously be SVP");

    assert_eq!(svp.team_id, 200, "SVP must belong to the losing team");
    assert!(!svp.is_mvp, "SVP cannot simultaneously be MVP");
}

#[test]
fn test_evaluate_all_winners_has_zero_svp() {
    let mut inputs = Vec::new();
    for i in 1..=10 {
        inputs.push(make_input(i, 100, true, i as i32, 2, 2, i * 4000, 8000, 2000, 10000, 150, 20));
    }

    let evals = evaluate_match_participants(1800, &inputs);
    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 1, "Must have exactly 1 MVP when all players win");
    assert_eq!(svp_count, 0, "Must have 0 SVP when no players lose");
}

#[test]
fn test_evaluate_all_losers_has_zero_mvp() {
    let mut inputs = Vec::new();
    for i in 1..=10 {
        inputs.push(make_input(i, 200, false, i as i32, 4, 1, i * 3000, 12000, 3000, 7000, 100, 15));
    }

    let evals = evaluate_match_participants(1800, &inputs);
    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 0, "Must have 0 MVP when no players win");
    assert_eq!(svp_count, 1, "Must have exactly 1 SVP when players lose");
}

#[test]
fn test_evaluate_arena_mode_2v2v2v2() {
    let mut inputs = Vec::new();
    // 1st place team (winners)
    inputs.push(make_input(1, 100, true, 8, 2, 4, 25000, 12000, 4000, 12000, 10, 5));
    inputs.push(make_input(2, 100, true, 6, 1, 6, 22000, 10000, 3000, 11000, 12, 5));

    // 2nd, 3rd, 4th place teams (losers)
    for i in 3..=8 {
        inputs.push(make_input(i, 200 + (i / 2), false, (9 - i) as i32, 5, 2, (10 - i) * 3000, 15000, 2000, 8000, 5, 2));
    }

    let evals = evaluate_match_participants(900, &inputs);
    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 1, "Arena mode must have exactly 1 MVP");
    assert_eq!(svp_count, 1, "Arena mode must have exactly 1 SVP");
}

#[test]
fn test_evaluate_1v1_duel() {
    let inputs = vec![
        make_input(1, 100, true, 3, 0, 0, 15000, 5000, 1000, 8000, 100, 10),
        make_input(2, 200, false, 0, 3, 0, 6000, 14000, 500, 5000, 70, 5),
    ];

    let evals = evaluate_match_participants(600, &inputs);
    assert_eq!(evals.iter().filter(|e| e.is_mvp).count(), 1);
    assert_eq!(evals.iter().filter(|e| e.is_svp).count(), 1);
    assert!(evals[0].is_mvp);
    assert!(evals[1].is_svp);
}

#[test]
fn test_evaluate_single_player_winner() {
    let inputs = vec![
        make_input(1, 100, true, 5, 0, 2, 18000, 6000, 2000, 9000, 120, 15),
    ];
    let evals = evaluate_match_participants(1200, &inputs);
    assert_eq!(evals.len(), 1);
    assert!(evals[0].is_mvp);
    assert!(!evals[0].is_svp);
}

#[test]
fn test_evaluate_single_player_loser() {
    let inputs = vec![
        make_input(1, 200, false, 1, 5, 0, 7000, 15000, 1000, 5000, 80, 5),
    ];
    let evals = evaluate_match_participants(1200, &inputs);
    assert_eq!(evals.len(), 1);
    assert!(!evals[0].is_mvp);
    assert!(evals[0].is_svp);
}

// ============================================================================
// Group 3: Edge Cases (0 kills, 0 dmg, 0 deaths, negatives, zeros)
// ============================================================================

#[test]
fn test_evaluate_all_zero_statistics_and_duration() {
    let mut inputs = Vec::new();
    for i in 1..=5 {
        inputs.push(make_input(i, 100, true, 0, 0, 0, 0, 0, 0, 0, 0, 0));
    }
    for i in 6..=10 {
        inputs.push(make_input(i, 200, false, 0, 0, 0, 0, 0, 0, 0, 0, 0));
    }

    // Remake game or zero duration: duration_sec = 0
    let evals = evaluate_match_participants(0, &inputs);
    assert_eq!(evals.len(), 10);

    for e in &evals {
        assert!(!e.score.is_nan(), "Score must never be NaN");
        assert!(!e.score.is_infinite(), "Score must never be infinite");
        assert!(!e.damage_share_pct.is_nan(), "damage_share_pct must never be NaN");
        assert!(!e.damage_taken_share_pct.is_nan(), "damage_taken_share_pct must never be NaN");
        assert!(!e.kp_pct.is_nan(), "kp_pct must never be NaN");
        assert!(!e.is_highest_damage, "No player should have highest damage crown when damage is 0");
    }

    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();
    assert_eq!(mvp_count, 1, "Exactly 1 MVP even when all stats are zero");
    assert_eq!(svp_count, 1, "Exactly 1 SVP even when all stats are zero");

    // Tie-breaking by participant_id selects participant 1 as MVP and participant 6 as SVP
    assert!(evals[0].is_mvp, "Participant 1 should win tie-break on winning team");
    assert!(evals[5].is_svp, "Participant 6 should win tie-break on losing team");
}

#[test]
fn test_evaluate_negative_values_and_duration() {
    let mut inputs = Vec::new();
    for i in 1..=5 {
        inputs.push(make_input(i, 100, true, -1, -2, -1, -1000, -500, -200, -1000, -20, -5));
    }
    for i in 6..=10 {
        inputs.push(make_input(i, 200, false, -2, -5, -1, -2000, -1000, -300, -2000, -30, -10));
    }

    let evals = evaluate_match_participants(-600, &inputs);
    assert_eq!(evals.len(), 10);

    for e in &evals {
        assert!(!e.score.is_nan());
        assert!(!e.score.is_infinite());
    }

    assert_eq!(evals.iter().filter(|e| e.is_mvp).count(), 1);
    assert_eq!(evals.iter().filter(|e| e.is_svp).count(), 1);
}

// ============================================================================
// Group 4: Tie-Breakers Stress Tests
// ============================================================================

#[test]
fn test_evaluate_identical_damage_across_teammates() {
    let mut inputs = Vec::new();
    // 5 winning teammates with EXACT SAME damage, but different kills
    for i in 1..=5 {
        inputs.push(make_input(
            i, 100, true,
            i as i32, // varying kills
            2,
            5,
            25000, // identical damage!
            10000,
            3000,
            12000,
            150,
            20,
        ));
    }
    // 5 losing teammates with EXACT SAME damage, but different kills
    for i in 6..=10 {
        inputs.push(make_input(
            i, 200, false,
            (11 - i) as i32,
            4,
            2,
            18000, // identical damage!
            12000,
            2000,
            9000,
            120,
            15,
        ));
    }

    let evals = evaluate_match_participants(1800, &inputs);

    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 1, "Must select exactly 1 MVP when damage is identical");
    assert_eq!(svp_count, 1, "Must select exactly 1 SVP when damage is identical");

    // Participant 5 has the highest kills (5) among identical damage -> should win MVP
    assert!(evals[4].is_mvp, "Player 5 with most kills should break tie for MVP");
}

#[test]
fn test_evaluate_cascade_tie_breaker_all_metrics_identical() {
    let mut inputs = Vec::new();
    // 5 teammates with 100% IDENTICAL stats in every field
    for i in 1..=5 {
        inputs.push(make_input(i, 100, true, 5, 2, 8, 20000, 10000, 5000, 11000, 150, 25));
    }
    for i in 6..=10 {
        inputs.push(make_input(i, 200, false, 3, 5, 3, 14000, 12000, 3000, 8000, 120, 15));
    }

    let evals = evaluate_match_participants(1800, &inputs);

    let mvp_count = evals.iter().filter(|e| e.is_mvp).count();
    let svp_count = evals.iter().filter(|e| e.is_svp).count();

    assert_eq!(mvp_count, 1, "Exactly 1 MVP must be selected among clones");
    assert_eq!(svp_count, 1, "Exactly 1 SVP must be selected among clones");

    // Deterministic tie-breaker selects participant with lowest participant_id
    assert!(evals[0].is_mvp, "Participant 1 must win tie-break (lowest pid)");
    assert!(evals[5].is_svp, "Participant 6 must win tie-break (lowest pid on losing team)");
}

#[test]
fn test_evaluate_tie_breaker_deaths_preference() {
    // Two players with nearly identical scores:
    // Player 1: 5 kills, 1 death, 5 assists, 20000 dmg
    // Player 2: 5 kills, 3 deaths, 5 assists, 20000 dmg
    let p1 = make_input(1, 100, true, 5, 1, 5, 20000, 8000, 2000, 10000, 150, 20);
    let p2 = make_input(2, 100, true, 5, 3, 5, 20000, 8000, 2000, 10000, 150, 20);

    let evals = evaluate_match_participants(1800, &[p1, p2]);
    assert_eq!(evals.iter().filter(|e| e.is_mvp).count(), 1);
    assert!(evals[0].is_mvp, "Player 1 with fewer deaths (1 vs 3) should win MVP");
}

#[test]
fn test_evaluate_tie_breaker_pid_cascade() {
    // Two players with exactly identical stats, different pid
    let p1 = make_input(10, 100, true, 5, 2, 5, 20000, 8000, 2000, 10000, 150, 20);
    let p2 = make_input(20, 100, true, 5, 2, 5, 20000, 8000, 2000, 10000, 150, 20);

    let evals = evaluate_match_participants(1800, &[p1, p2]);
    assert_eq!(evals.iter().filter(|e| e.is_mvp).count(), 1);
    assert!(evals[0].is_mvp, "Player with smaller pid (10 vs 20) should win MVP");
}

#[test]
fn test_highest_damage_crown_with_ties() {
    // Two players on opposing teams tie for highest game damage (30,000)
    let p1 = make_input(1, 100, true, 8, 2, 5, 30000, 10000, 2000, 12000, 180, 20);
    let p2 = make_input(2, 100, true, 4, 3, 4, 15000, 8000, 1000, 9000, 120, 15);
    let p3 = make_input(3, 200, false, 7, 3, 3, 30000, 12000, 3000, 11000, 170, 18);
    let p4 = make_input(4, 200, false, 2, 5, 2, 12000, 10000, 1000, 8000, 100, 10);

    let evals = evaluate_match_participants(1800, &[p1, p2, p3, p4]);
    // Both p1 and p3 achieved 30,000 damage (the game maximum > 0)
    assert!(evals[0].is_highest_damage, "p1 has highest damage");
    assert!(!evals[1].is_highest_damage, "p2 does not have highest damage");
    assert!(evals[2].is_highest_damage, "p3 also has highest damage");
    assert!(!evals[3].is_highest_damage, "p4 does not have highest damage");
}
