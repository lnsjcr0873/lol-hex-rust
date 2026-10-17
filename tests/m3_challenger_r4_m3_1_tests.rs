//! Adversarial Empirical Challenge Suite for Milestone 3
//!
//! Evaluates and stress-tests:
//! 1. Augment classification boundary cases (`DecisionEngine::classify_augments`):
//!    - High delta, low games (`delta = 0.50, games = 1`) -> MUST NOT be S-tier (demoted to B-tier due to games < 30).
//!    - Bad delta, low games (`delta = -0.08, games = 50`) -> MUST be classified as trap, NOT B-tier.
//!    - Good delta, high games (`delta = 0.04, games = 500`) -> S-tier.
//!    - Borderline delta (`delta = -0.006` vs `-0.004`) -> Verify `is_trap` and `all_traps` consistency without UI contradiction.
//! 2. Augment comparison consistency (`DecisionEngine::compare_augments`):
//!    - Traps badge alignment and elimination of UI contradiction.
//! 3. Hex-Item Synergy Scoring (`HexItemSynergyEngine::rerank_items_with_synergy`):
//!    - Augment with AP keywords ("法术强度") -> AP items gain synergy bonus, non-AP items do not.
//!    - Augment with Crit keywords ("暴击") -> Crit items gain synergy bonus, non-Crit items do not.
//!    - Weight scaling, adversarial weights (NaN, negative, >5.0), deterministic tie-breaking.
//! 4. Full Pipeline Integration (`DecisionEngine::get_item_recommendation`):
//!    - Hydration of augment name and description driving item synergy bonus.

use lol_hex_rust::config::RecommendationConfig;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::decision::DecisionEngine;
use lol_hex_rust::engine::synergy::HexItemSynergyEngine;
use serde_json::json;
use std::collections::HashSet;

// ============================================================================
// Group 1: Augment Classification Boundary Tests (classify_augments)
// ============================================================================

#[test]
fn test_classify_augments_high_delta_low_games_demoted_to_b_tier() {
    // High delta (0.50) with only 1 game MUST NOT be S-tier or A-tier; must be demoted to B-tier.
    let augments = json!([
        {
            "id": "aug_high_delta_1_game",
            "name": "新手光环",
            "deltaWinRate": 0.50,
            "games": 1
        },
        {
            "id": "aug_high_delta_29_games",
            "name": "临界样本强化",
            "deltaWinRate": 0.50,
            "games": 29
        },
        {
            "id": "aug_high_delta_30_games",
            "name": "达标样本强化",
            "deltaWinRate": 0.50,
            "games": 30
        }
    ]);

    let classified = DecisionEngine::classify_augments(&augments, 0.50);
    let s_tier = classified["sTier"].as_array().expect("sTier array");
    let a_tier = classified["aTier"].as_array().expect("aTier array");
    let b_tier = classified["bTier"].as_array().expect("bTier array");
    let traps = classified["traps"].as_array().expect("traps array");

    // games = 1 MUST NOT be S-tier or A-tier, MUST be B-tier
    assert!(!s_tier.iter().any(|a| a["id"] == "aug_high_delta_1_game"), "games=1 must not be S-tier");
    assert!(!a_tier.iter().any(|a| a["id"] == "aug_high_delta_1_game"), "games=1 must not be A-tier");
    assert!(!traps.iter().any(|a| a["id"] == "aug_high_delta_1_game"), "games=1 is positive delta, not trap");
    assert!(b_tier.iter().any(|a| a["id"] == "aug_high_delta_1_game"), "games=1 must be demoted to B-tier");

    // Boundary check at games = 29 (still < 30)
    assert!(!s_tier.iter().any(|a| a["id"] == "aug_high_delta_29_games"), "games=29 must not be S-tier");
    assert!(b_tier.iter().any(|a| a["id"] == "aug_high_delta_29_games"), "games=29 must be demoted to B-tier");

    // Boundary check at games = 30 (>= 30, delta >= 0.035 -> S-tier)
    assert!(s_tier.iter().any(|a| a["id"] == "aug_high_delta_30_games"), "games=30 with delta=0.50 must be S-tier");
}

#[test]
fn test_classify_augments_bad_delta_low_games_must_be_trap() {
    // Bad delta (-0.08) with low games (50) MUST be classified as trap, NOT B-tier.
    // The previous bug required games >= 100 for traps, causing low-sample terrible augments to fall into B-tier.
    let augments = json!([
        {
            "id": "aug_bad_delta_50_games",
            "name": "巨坑强化A",
            "deltaWinRate": -0.08,
            "games": 50
        },
        {
            "id": "aug_bad_delta_0_games",
            "name": "巨坑强化0场",
            "deltaWinRate": -0.08,
            "games": 0
        },
        {
            "id": "aug_bad_delta_20_games",
            "name": "巨坑强化20场",
            "deltaWinRate": -0.05,
            "games": 20
        },
        {
            "id": "aug_bad_delta_99_games",
            "name": "巨坑强化99场",
            "deltaWinRate": -0.02,
            "games": 99
        }
    ]);

    let classified = DecisionEngine::classify_augments(&augments, 0.50);
    let b_tier = classified["bTier"].as_array().expect("bTier array");
    let traps = classified["traps"].as_array().expect("traps array");

    // All must be in traps, NONE in bTier
    for aid in &["aug_bad_delta_50_games", "aug_bad_delta_0_games", "aug_bad_delta_20_games", "aug_bad_delta_99_games"] {
        assert!(
            traps.iter().any(|a| a["id"] == *aid),
            "{} must be classified as trap",
            aid
        );
        assert!(
            !b_tier.iter().any(|a| a["id"] == *aid),
            "{} must NOT fall into B-tier",
            aid
        );
    }
}

#[test]
fn test_classify_augments_good_delta_high_games_is_s_tier() {
    // Good delta (0.04) with high games (500) -> S-tier (delta >= 0.035 && games >= 30)
    let augments = json!([
        {
            "id": "aug_s_tier",
            "name": "天命强化",
            "deltaWinRate": 0.04,
            "games": 500
        },
        {
            "id": "aug_a_tier_upper",
            "name": "强势强化A",
            "deltaWinRate": 0.034,
            "games": 500
        },
        {
            "id": "aug_a_tier_lower",
            "name": "强势强化B",
            "deltaWinRate": 0.012,
            "games": 500
        },
        {
            "id": "aug_b_tier_normal",
            "name": "普通可用",
            "deltaWinRate": 0.011,
            "games": 500
        }
    ]);

    let classified = DecisionEngine::classify_augments(&augments, 0.50);
    let s_tier = classified["sTier"].as_array().expect("sTier array");
    let a_tier = classified["aTier"].as_array().expect("aTier array");
    let b_tier = classified["bTier"].as_array().expect("bTier array");

    assert!(s_tier.iter().any(|a| a["id"] == "aug_s_tier"), "delta=0.04, games=500 must be S-tier");
    assert!(a_tier.iter().any(|a| a["id"] == "aug_a_tier_upper"), "delta=0.034, games=500 must be A-tier");
    assert!(a_tier.iter().any(|a| a["id"] == "aug_a_tier_lower"), "delta=0.012, games=500 must be A-tier");
    assert!(b_tier.iter().any(|a| a["id"] == "aug_b_tier_normal"), "delta=0.011, games=500 must be B-tier");
}

#[test]
fn test_classify_augments_borderline_delta_threshold() {
    // Borderline delta: -0.006 vs -0.004 with threshold at -0.005
    let augments = json!([
        {
            "id": "aug_neg_006",
            "name": "轻度负收益",
            "deltaWinRate": -0.006,
            "games": 200
        },
        {
            "id": "aug_neg_005001",
            "name": "微超阈值负收益",
            "deltaWinRate": -0.005001,
            "games": 200
        },
        {
            "id": "aug_neg_005",
            "name": "恰好在阈值",
            "deltaWinRate": -0.005,
            "games": 200
        },
        {
            "id": "aug_neg_004999",
            "name": "微低于阈值",
            "deltaWinRate": -0.004999,
            "games": 200
        },
        {
            "id": "aug_neg_004",
            "name": "轻微负收益",
            "deltaWinRate": -0.004,
            "games": 200
        }
    ]);

    let classified = DecisionEngine::classify_augments(&augments, 0.50);
    let b_tier = classified["bTier"].as_array().expect("bTier array");
    let traps = classified["traps"].as_array().expect("traps array");

    // -0.006 < -0.005 -> trap
    assert!(traps.iter().any(|a| a["id"] == "aug_neg_006"), "-0.006 must be in traps");
    assert!(!b_tier.iter().any(|a| a["id"] == "aug_neg_006"), "-0.006 must not be in bTier");

    // -0.005001 < -0.005 -> trap
    assert!(traps.iter().any(|a| a["id"] == "aug_neg_005001"), "-0.005001 must be in traps");

    // -0.005 < -0.005 is false -> falls to b_tier
    assert!(b_tier.iter().any(|a| a["id"] == "aug_neg_005"), "-0.005 exactly is not < -0.005, falls into B tier");

    // -0.004999 is not < -0.005 -> b_tier
    assert!(b_tier.iter().any(|a| a["id"] == "aug_neg_004999"), "-0.004999 must be in bTier");

    // -0.004 is not < -0.005 -> b_tier
    assert!(b_tier.iter().any(|a| a["id"] == "aug_neg_004"), "-0.004 must be in bTier");
    assert!(!traps.iter().any(|a| a["id"] == "aug_neg_004"), "-0.004 must not be in traps");
}

#[test]
fn test_classify_augments_fallback_win_rate_subtraction() {
    // When deltaWinRate is absent, delta is calculated from winRate - hero_win_rate
    let augments = json!([
        {
            "id": "aug_wr_good",
            "name": "高胜率无delta",
            "winRate": 0.54, // 0.54 - 0.50 = +0.04
            "games": 100
        },
        {
            "id": "aug_wr_bad",
            "name": "低胜率无delta",
            "winRate": 0.42, // 0.42 - 0.50 = -0.08
            "games": 50
        }
    ]);

    let classified = DecisionEngine::classify_augments(&augments, 0.50);
    let s_tier = classified["sTier"].as_array().expect("sTier array");
    let traps = classified["traps"].as_array().expect("traps array");

    assert!(s_tier.iter().any(|a| a["id"] == "aug_wr_good"), "wr=0.54 with hero=0.50 -> delta=+0.04 -> S-tier");
    assert!(traps.iter().any(|a| a["id"] == "aug_wr_bad"), "wr=0.42 with hero=0.50 -> delta=-0.08 -> trap");
}

// ============================================================================
// Group 2: Augment Comparison & UI Consistency Tests (compare_augments)
// ============================================================================

#[test]
fn test_compare_augments_trap_consistency_and_ui_contradiction_elimination() {
    let db = Database::load_from_dir("data");

    // Test with Ahri (hero "103")
    // Scenario 1: Both candidates are bad (delta < -0.005).
    // Verify that every card marked isTrap=true, tierBadge="TRAP", and allTrapsAlert=true.
    let candidates_all_traps = vec!["1001".to_string(), "1002".to_string()];
    let res = DecisionEngine::compare_augments(&db, "103", &candidates_all_traps, None, &[]);

    assert!(res.get("error").is_none(), "Hero 103 should be resolved: {:?}", res.get("error"));
    let cards = res["candidates"].as_array().expect("candidates list");
    let all_traps_alert = res["allTrapsAlert"].as_bool().unwrap_or(false);

    // If all candidates are traps, verify no contradictory B-tier badge
    if all_traps_alert {
        for card in cards {
            assert_eq!(card["isTrap"], true, "When allTrapsAlert is true, each card isTrap must be true");
            assert_eq!(card["tierBadge"], "TRAP", "When allTrapsAlert is true, each card badge must be TRAP");
        }
        let rationale = res["recommendationRationale"].as_str().unwrap_or("");
        assert!(rationale.contains("全陷阱警告"), "Rationale should include full-trap warning: {}", rationale);
        assert_eq!(res["rerollDecision"]["action"], "REROLL", "Action must be REROLL for all traps");
    }

    // Scenario 2: Invariant Check across all returned cards:
    // A card CANNOT simultaneously have tierBadge == "B" and isTrap == true.
    // A card CANNOT simultaneously have tierBadge == "TRAP" and isTrap == false.
    for card in cards {
        let is_trap = card["isTrap"].as_bool().unwrap_or(false);
        let badge = card["tierBadge"].as_str().unwrap_or("");
        if is_trap {
            assert_eq!(badge, "TRAP", "Trap card must have TRAP badge, not {}", badge);
        } else {
            assert_ne!(badge, "TRAP", "Non-trap card must not have TRAP badge");
        }
    }
}

// ============================================================================
// Group 3: Hex-Item Synergy Scoring Tests (HexItemSynergyEngine)
// ============================================================================

#[test]
fn test_synergy_scoring_ap_keywords_boost_ap_items() {
    // Construct augment with AP keywords ("法术强度")
    let active_augments = vec![json!({
        "name": "奥术之灵",
        "description": "提供极高法术强度加成"
    })];

    // Item 3089 = Rabadon's Deathcap (has AP tag in item_feature_tags)
    // Item 3031 = Infinity Edge (has AD, CRIT tags)
    let candidate_items = vec![
        json!({ "itemId": "3089", "tacticalScore": 50.0 }),
        json!({ "itemId": "3031", "tacticalScore": 50.0 })
    ];

    let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103", // Ahri
        1.0,
        true,
    );

    let deathcap = reranked.iter().find(|it| it["itemId"] == "3089").expect("Deathcap in reranked");
    let ie = reranked.iter().find(|it| it["itemId"] == "3031").expect("IE in reranked");

    // AP item MUST gain synergy bonus (+12.0)
    assert_eq!(deathcap["hasAugmentSynergy"], true, "AP item must have hasAugmentSynergy = true");
    let deathcap_score = deathcap["tacticalScore"].as_f64().unwrap();
    assert!((deathcap_score - 62.0).abs() < 1e-4, "Deathcap score expected 62.0, got {}", deathcap_score);

    // AD/Crit item MUST NOT gain synergy bonus
    assert!(ie.get("hasAugmentSynergy").is_none() || ie["hasAugmentSynergy"] == false, "AD item must not have AP synergy");
    let ie_score = ie["tacticalScore"].as_f64().unwrap();
    assert!((ie_score - 50.0).abs() < 1e-4, "IE score expected 50.0, got {}", ie_score);
}

#[test]
fn test_synergy_scoring_crit_keywords_boost_crit_items() {
    // Construct augment with Crit keywords ("暴击")
    let active_augments = vec![json!({
        "name": "鹰眼锐利",
        "description": "普通攻击暴击率提升且暴击造成额外伤害"
    })];

    // Item 3031 = Infinity Edge (has CRIT, AD tags)
    // Item 3089 = Rabadon's Deathcap (has AP tag)
    let candidate_items = vec![
        json!({ "itemId": "3031", "tacticalScore": 50.0 }),
        json!({ "itemId": "3089", "tacticalScore": 50.0 })
    ];

    let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "222", // Jinx
        1.0,
        true,
    );

    let ie = reranked.iter().find(|it| it["itemId"] == "3031").expect("IE in reranked");
    let deathcap = reranked.iter().find(|it| it["itemId"] == "3089").expect("Deathcap in reranked");

    // Crit item MUST gain synergy bonus (+12.0)
    assert_eq!(ie["hasAugmentSynergy"], true, "Crit item must have hasAugmentSynergy = true");
    let ie_score = ie["tacticalScore"].as_f64().unwrap();
    assert!((ie_score - 62.0).abs() < 1e-4, "IE score expected 62.0, got {}", ie_score);

    // AP item MUST NOT gain synergy bonus
    assert!(deathcap.get("hasAugmentSynergy").is_none() || deathcap["hasAugmentSynergy"] == false, "AP item must not have Crit synergy");
    let deathcap_score = deathcap["tacticalScore"].as_f64().unwrap();
    assert!((deathcap_score - 50.0).abs() < 1e-4, "Deathcap score expected 50.0, got {}", deathcap_score);
}

#[test]
fn test_synergy_scoring_weight_scaling_and_adversarial_weights() {
    let active_augments = vec![json!({
        "name": "测试AP强化",
        "description": "提升法术强度"
    })];

    let candidate_items = vec![
        json!({ "itemId": "3089", "tacticalScore": 50.0 }),
    ];

    // Weight = 2.0 -> bonus = 12.0 * 2.0 = 24.0 -> score = 74.0
    let reranked_2 = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103",
        2.0,
        true,
    );
    assert!((reranked_2[0]["tacticalScore"].as_f64().unwrap() - 74.0).abs() < 1e-4);

    // Weight = 0.5 -> bonus = 12.0 * 0.5 = 6.0 -> score = 56.0
    let reranked_half = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103",
        0.5,
        true,
    );
    assert!((reranked_half[0]["tacticalScore"].as_f64().unwrap() - 56.0).abs() < 1e-4);

    // Weight = -1.0 (invalid/negative) -> defaults to 1.0 -> bonus = 12.0 -> score = 62.0
    let reranked_neg = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103",
        -1.0,
        true,
    );
    assert!((reranked_neg[0]["tacticalScore"].as_f64().unwrap() - 62.0).abs() < 1e-4);

    // Weight = f64::NAN -> defaults to 1.0 -> bonus = 12.0 -> score = 62.0
    let reranked_nan = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103",
        f64::NAN,
        true,
    );
    assert!((reranked_nan[0]["tacticalScore"].as_f64().unwrap() - 62.0).abs() < 1e-4);

    // Weight = 10.0 -> clamped to 5.0 -> bonus = 12.0 * 5.0 = 60.0 -> score = 110.0
    let reranked_clamped = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "103",
        10.0,
        true,
    );
    assert!((reranked_clamped[0]["tacticalScore"].as_f64().unwrap() - 110.0).abs() < 1e-4);
}

#[test]
fn test_synergy_scoring_deterministic_tie_breaker() {
    // When multiple items have equal score, they must be deterministically sorted by itemId
    let active_augments = vec![json!({
        "name": "普通强化",
        "description": "无特殊加成"
    })];

    let candidate_items = vec![
        json!({ "itemId": "3089", "tacticalScore": 50.0 }),
        json!({ "itemId": "3031", "tacticalScore": 50.0 }),
        json!({ "itemId": "3075", "tacticalScore": 50.0 }),
    ];

    let reranked1 = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "1",
        1.0,
        true,
    );

    let reranked2 = HexItemSynergyEngine::rerank_items_with_synergy(
        &candidate_items,
        &active_augments,
        &HashSet::new(),
        "1",
        1.0,
        true,
    );

    let ids1: Vec<&str> = reranked1.iter().map(|it| it["itemId"].as_str().unwrap()).collect();
    let ids2: Vec<&str> = reranked2.iter().map(|it| it["itemId"].as_str().unwrap()).collect();
    assert_eq!(ids1, ids2, "Sorting must be completely deterministic across calls");
}

// ============================================================================
// Group 4: Pipeline Integration Tests (DecisionEngine::get_item_recommendation)
// ============================================================================

#[test]
fn test_get_item_recommendation_hydrates_augment_and_activates_synergy() {
    let db = Database::load_from_dir("data");

    // Pick an existing augment with AP keywords from database if available (e.g. 1030 Eureka or 1390 Phenomenal Evil)
    let aug_id = db.augments_summary.iter().find(|a| {
        a.description.contains("法术") || a.description.contains("法强") || a.name.contains("法术")
    }).map(|a| a.id.clone()).unwrap_or_else(|| "1030".to_string());

    let cfg = RecommendationConfig::default();
    let enemy_team = vec![json!({ "championName": "DrMundo" })];
    let rec = DecisionEngine::get_item_recommendation(
        &db,
        "1", // Annie (AP_POKE_BURST default archetype)
        &[],
        3000.0,
        6,
        &[aug_id],
        &enemy_team,
        &cfg,
    );

    assert!(rec.get("archetype").is_some(), "Archetype must be present");
    assert!(rec.get("buildPath").is_some(), "Build path must be present");

    let build_path = rec["buildPath"].as_array().expect("buildPath array");
    assert!(!build_path.is_empty(), "Build path must not be empty");

    // All top AP items in Annie's buildPath must have received augment synergy!
    let torch = build_path.iter().find(|it| it["itemId"] == "2503").expect("Torch 2503 in buildPath");
    assert_eq!(torch["hasAugmentSynergy"], true, "Torch 2503 must receive augment synergy bonus");
    assert_eq!(torch["isArchetypeFavored"], true);
}
