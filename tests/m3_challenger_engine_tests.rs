use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::decision::DecisionEngine;
use lol_hex_rust::engine::synergy::HexItemSynergyEngine;
use lol_hex_rust::engine::archetype::ArchetypeEngine;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn test_adversarial_classify_augments_boundaries() {
    let raw_cases = json!([
        // S tier exact boundary: delta = 0.035, games = 30 -> S
        { "id": "s_exact", "deltaWinRate": 0.035, "games": 30 },
        // S tier just below games: delta = 0.05, games = 29 -> B
        { "id": "s_below_games", "deltaWinRate": 0.05, "games": 29 },
        // S tier just below delta: delta = 0.0349, games = 50 -> A
        { "id": "s_below_delta", "deltaWinRate": 0.0349, "games": 50 },

        // A tier exact boundary: delta = 0.012, games = 30 -> A
        { "id": "a_exact", "deltaWinRate": 0.012, "games": 30 },
        // A tier just below games: delta = 0.02, games = 29 -> B
        { "id": "a_below_games", "deltaWinRate": 0.02, "games": 29 },
        // A tier just below delta: delta = 0.0119, games = 50 -> B
        { "id": "a_below_delta", "deltaWinRate": 0.0119, "games": 50 },

        // B tier neutral: delta = 0.0, games = 100 -> B
        { "id": "b_neutral", "deltaWinRate": 0.0, "games": 100 },
        // Trap boundary exact: delta = -0.005, games = 100 -> B (not < -0.005)
        { "id": "trap_exact_boundary", "deltaWinRate": -0.005, "games": 100 },

        // Trap low sample: delta = -0.00501, games = 1 -> TRAP
        { "id": "trap_low_sample_1", "deltaWinRate": -0.00501, "games": 1 },
        // Trap zero games: delta = -0.01, games = 0 -> TRAP
        { "id": "trap_zero_games", "deltaWinRate": -0.01, "games": 0 },
        // Severe trap low sample: delta = -0.20, games = 15 -> TRAP
        { "id": "trap_severe_low_sample", "deltaWinRate": -0.20, "games": 15 },
    ]);

    let res = DecisionEngine::classify_augments(&raw_cases, 0.50);

    let s_ids: Vec<&str> = res["sTier"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap()).collect();
    let a_ids: Vec<&str> = res["aTier"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap()).collect();
    let b_ids: Vec<&str> = res["bTier"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap()).collect();
    let trap_ids: Vec<&str> = res["traps"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap()).collect();

    assert_eq!(s_ids, vec!["s_exact"]);
    assert_eq!(a_ids, vec!["s_below_delta", "a_exact"]);
    assert!(b_ids.contains(&"s_below_games"));
    assert!(b_ids.contains(&"a_below_games"));
    assert!(b_ids.contains(&"a_below_delta"));
    assert!(b_ids.contains(&"b_neutral"));
    assert!(b_ids.contains(&"trap_exact_boundary"));

    assert!(trap_ids.contains(&"trap_low_sample_1"));
    assert!(trap_ids.contains(&"trap_zero_games"));
    assert!(trap_ids.contains(&"trap_severe_low_sample"));
}

#[test]
fn test_adversarial_compare_augments_trap_and_fluke_handling() {
    let db = Database::load_from_dir("data");

    // Test fluke augment with 1 game 100% win rate
    // Candidates: 1068 (recursion) vs 1238 (prismatic)
    let candidates = vec!["1068".to_string(), "1238".to_string()];
    let res = DecisionEngine::compare_augments(&db, "103", &candidates, None, &[]);

    let cards = res["candidates"].as_array().unwrap();
    assert_eq!(cards.len(), 2);

    let all_traps = res["allTrapsAlert"].as_bool().unwrap();
    assert_eq!(all_traps, cards.iter().all(|c| c["isTrap"].as_bool().unwrap_or(false)));

    for card in cards {
        let delta = card["deltaWinRate"].as_f64().unwrap();
        let games = card["games"].as_i64().unwrap();
        let is_destiny = card["isDestiny"].as_bool().unwrap();
        let is_trap = card["isTrap"].as_bool().unwrap();
        let tier = card["tierBadge"].as_str().unwrap();

        if is_trap {
            assert!(delta < -0.005);
            assert_eq!(tier, "TRAP");
        } else if is_destiny {
            assert!(delta >= 0.035 && games >= 30);
            assert_eq!(tier, "S");
        } else if delta >= 0.012 && games >= 30 {
            assert_eq!(tier, "A");
        } else {
            assert_eq!(tier, "B");
        }
    }

    // Now test candidates that are ALL negative delta (traps)
    let trap_candidates = vec!["1001".to_string(), "1002".to_string()];
    let trap_res = DecisionEngine::compare_augments(&db, "103", &trap_candidates, None, &[]);
    let trap_cards = trap_res["candidates"].as_array().unwrap();
    let trap_all_alert = trap_res["allTrapsAlert"].as_bool().unwrap();
    assert!(trap_all_alert, "When all candidates are negative delta, allTrapsAlert must be true");
    for card in trap_cards {
        assert_eq!(card["isTrap"], true, "Each card must be marked isTrap = true");
        assert_eq!(card["tierBadge"], "TRAP", "Each card must display TRAP tier badge");
    }
}

#[test]
fn test_adversarial_hex_item_synergy_stress() {
    // 1. Extreme synergy weights: NaN, Inf, negative, huge
    let candidate_items = vec![
        json!({ "itemId": "3089", "tacticalScore": 50.0 }), // AP
        json!({ "itemId": "3031", "tacticalScore": 50.0 }), // AD/Crit
        json!({ "itemId": "3075", "tacticalScore": 50.0 }), // Tank
    ];

    let active_augments = vec![
        json!({
            "name": "超级法术与暴击",
            "description": "法强提升50，暴击率提升20%，并且拥有大量技能急速"
        })
    ];

    // NaN weight should clamp gracefully to 1.0
    let res_nan = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "1",
        f64::NAN,
        false,
    );
    assert!(!res_nan.is_empty());
    let ap_item = res_nan.iter().find(|i| i["itemId"] == "3089").unwrap();
    assert!(ap_item["tacticalScore"].as_f64().unwrap().is_finite());

    // 2. Deterministic secondary tie-breaker
    // When items have identical score, they must sort deterministically by ID
    let identical_items = vec![
        json!({ "itemId": "3089", "tacticalScore": 50.0 }),
        json!({ "itemId": "3031", "tacticalScore": 50.0 }),
        json!({ "itemId": "3075", "tacticalScore": 50.0 }),
    ];
    let res_tied_1 = HexItemSynergyEngine::rerank_items_with_synergy(
        &identical_items,
        &[],
        &HashSet::new(),
        "1",
        1.0,
        false,
    );
    let res_tied_2 = HexItemSynergyEngine::rerank_items_with_synergy(
        &identical_items,
        &[],
        &HashSet::new(),
        "1",
        1.0,
        false,
    );
    let ids_1: Vec<&str> = res_tied_1.iter().map(|i| i["itemId"].as_str().unwrap()).collect();
    let ids_2: Vec<&str> = res_tied_2.iter().map(|i| i["itemId"].as_str().unwrap()).collect();
    assert_eq!(ids_1, ids_2, "Tied items must sort deterministically");
}

#[test]
fn test_adversarial_augment_hydration_and_synergy_activation() {
    let db = Database::load_from_dir("data");

    // Augment 1068 is "循环往复", desc: "获得60技能急速。"
    let rec_cfg = lol_hex_rust::config::RecommendationConfig::default();
    let rec = DecisionEngine::get_item_recommendation(
        &db,
        "103", // Ahri
        &[],
        3000.0,
        1,
        &["1068".to_string()], // Pass augment ID
        &[],
        &rec_cfg,
    );

    let build_path = rec["buildPath"].as_array().expect("buildPath must be present");
    // Verify haste items receive synergy
    let ionian_boots = build_path.iter().find(|it| {
        it.get("itemId").and_then(|v| v.as_str()) == Some("3158")
    });
    if let Some(boots) = ionian_boots {
        assert_eq!(boots["hasAugmentSynergy"], true, "Ionian boots should have augment synergy with 循环往复");
    }
}
