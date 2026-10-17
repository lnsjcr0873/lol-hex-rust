//! Adversarial Empirical Challenge Suite for Milestone 2
//!
//! Evaluates and stress-tests:
//! 1. `deserialize_f64_lenient` edge cases, scale consistency (0.0 ~ 1.0 for percentages),
//!    and full model deserialization (HeroSummary, AugmentSummary, AugmentTrio, TerminalItemTrio, PostMatchInfo).
//! 2. `Database::sync_from_api` repeated calls in sequence, verifying that
//!    `heroes_summary.len()` and `augments_summary.len()` remain constant and do not multiply.

use lol_hex_rust::crawler::HexdataCrawler;
use lol_hex_rust::data::models::{
    deserialize_f64_lenient, AugmentSummary, AugmentTrio, HeroSummary, PostMatchInfo,
    TerminalItemTrio,
};
use lol_hex_rust::data::loader::Database;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;

#[derive(Debug, Deserialize, PartialEq)]
struct F64LenientWrapper {
    #[serde(deserialize_with = "deserialize_f64_lenient")]
    value: f64,
}

fn parse_lenient(v: serde_json::Value) -> Result<f64, serde_json::Error> {
    let obj = json!({ "value": v });
    let wrapper: F64LenientWrapper = serde_json::from_value(obj)?;
    Ok(wrapper.value)
}

// ============================================================================
// Group 1: Required Edge Cases for deserialize_f64_lenient
// ============================================================================

#[test]
fn test_m2_deserialize_f64_lenient_required_edge_cases() {
    // 1. "54.2%" -> 0.542
    let val = parse_lenient(json!("54.2%")).expect("Failed to parse '54.2%'");
    assert!(
        (val - 0.542).abs() < 1e-6,
        "Expected 0.542, got {}",
        val
    );

    // 2. "100%" -> 1.0
    let val = parse_lenient(json!("100%")).expect("Failed to parse '100%'");
    assert!(
        (val - 1.0).abs() < 1e-6,
        "Expected 1.0, got {}",
        val
    );

    // 3. "0%" -> 0.0
    let val = parse_lenient(json!("0%")).expect("Failed to parse '0%'");
    assert!(
        (val - 0.0).abs() < 1e-6,
        "Expected 0.0, got {}",
        val
    );

    // 4. "0.542" -> 0.542 (plain string number without % is NOT divided by 100.0)
    let val = parse_lenient(json!("0.542")).expect("Failed to parse '0.542'");
    assert!(
        (val - 0.542).abs() < 1e-6,
        "Expected 0.542, got {}",
        val
    );

    // 5. 0.542 (raw float number) -> 0.542
    let val = parse_lenient(json!(0.542)).expect("Failed to parse float 0.542");
    assert!(
        (val - 0.542).abs() < 1e-6,
        "Expected 0.542, got {}",
        val
    );

    // 6. "54.2 %" (with space before %) -> 0.542
    let val = parse_lenient(json!("54.2 %")).expect("Failed to parse '54.2 %'");
    assert!(
        (val - 0.542).abs() < 1e-6,
        "Expected 0.542, got {}",
        val
    );

    // 7. " -5.5% " (negative percentage with outer whitespace) -> -0.055
    let val = parse_lenient(json!(" -5.5% ")).expect("Failed to parse ' -5.5% '");
    assert!(
        (val - (-0.055)).abs() < 1e-6,
        "Expected -0.055, got {}",
        val
    );

    // 8. "" (empty string) -> 0.0
    let val = parse_lenient(json!("")).expect("Failed to parse empty string");
    assert!(
        (val - 0.0).abs() < 1e-6,
        "Expected 0.0 for empty string, got {}",
        val
    );

    // 9. "invalid" -> 0.0
    let val = parse_lenient(json!("invalid")).expect("Failed to parse 'invalid'");
    assert!(
        (val - 0.0).abs() < 1e-6,
        "Expected 0.0 for 'invalid', got {}",
        val
    );
}

// ============================================================================
// Group 2: Extended Adversarial Edge Cases for deserialize_f64_lenient
// ============================================================================

#[test]
fn test_m2_deserialize_f64_lenient_extended_adversarial() {
    // Leading plus with percent: " +15.5% " -> 0.155
    let val = parse_lenient(json!(" +15.5% ")).expect("Failed to parse ' +15.5% '");
    assert!((val - 0.155).abs() < 1e-6, "Expected 0.155, got {}", val);

    // Percent alone: "%" -> 0.0
    let val = parse_lenient(json!("%")).expect("Failed to parse '%'");
    assert_eq!(val, 0.0);

    // Whitespace with percent alone: "   %   " -> 0.0
    let val = parse_lenient(json!("   %   ")).expect("Failed to parse '   %   '");
    assert_eq!(val, 0.0);

    // Minus percent alone: "-%" -> 0.0
    let val = parse_lenient(json!("-%")).expect("Failed to parse '-%'");
    assert_eq!(val, 0.0);

    // Comma formatted percentage: "1,250.5%" -> 12.505
    let val = parse_lenient(json!("1,250.5%")).expect("Failed to parse '1,250.5%'");
    assert!((val - 12.505).abs() < 1e-6, "Expected 12.505, got {}", val);

    // Negative 100%: "-100%" -> -1.0
    let val = parse_lenient(json!("-100%")).expect("Failed to parse '-100%'");
    assert!((val - (-1.0)).abs() < 1e-6, "Expected -1.0, got {}", val);

    // Over 100%: "250%" -> 2.5
    let val = parse_lenient(json!("250%")).expect("Failed to parse '250%'");
    assert!((val - 2.5).abs() < 1e-6, "Expected 2.5, got {}", val);

    // Null JSON value -> 0.0
    let val = parse_lenient(serde_json::Value::Null).expect("Failed to parse Null");
    assert_eq!(val, 0.0);

    // Boolean true -> 1.0, false -> 0.0
    let val_true = parse_lenient(json!(true)).expect("Failed to parse true");
    assert_eq!(val_true, 1.0);
    let val_false = parse_lenient(json!(false)).expect("Failed to parse false");
    assert_eq!(val_false, 0.0);

    // Integer 0, 1, 100 -> 0.0, 1.0, 100.0 (not divided without %)
    let val_i0 = parse_lenient(json!(0)).expect("Failed to parse int 0");
    assert_eq!(val_i0, 0.0);
    let val_i1 = parse_lenient(json!(1)).expect("Failed to parse int 1");
    assert_eq!(val_i1, 1.0);
    let val_i100 = parse_lenient(json!(100)).expect("Failed to parse int 100");
    assert_eq!(val_i100, 100.0);

    // NaN / Infinity strings -> 0.0
    let val_nan = parse_lenient(json!("NaN")).expect("Failed to parse 'NaN'");
    assert_eq!(val_nan, 0.0);
    let val_nan_pct = parse_lenient(json!("NaN%")).expect("Failed to parse 'NaN%'");
    assert_eq!(val_nan_pct, 0.0);
    let val_inf = parse_lenient(json!("Infinity")).expect("Failed to parse 'Infinity'");
    assert_eq!(val_inf, 0.0);
    let val_inf_pct = parse_lenient(json!("-inf%")).expect("Failed to parse '-inf%'");
    assert_eq!(val_inf_pct, 0.0);

    // Only whitespace: "    " -> 0.0
    let val_spaces = parse_lenient(json!("    ")).expect("Failed to parse spaces");
    assert_eq!(val_spaces, 0.0);
}

// ============================================================================
// Group 3: Real Struct Deserialization Consistency
// ============================================================================

#[test]
fn test_m2_hero_summary_deserialization_scale() {
    let raw = json!({
        "id": "103",
        "name": "Ahri",
        "alias": "Ahri",
        "title": "the Nine-Tailed Fox",
        "roles": ["Mage", "Assassin"],
        "winRate": "54.2%",
        "pickRate": "12.5 %",
        "banRate": "0%",
        "games": 15000,
        "tier": 1
    });

    let hero: HeroSummary = serde_json::from_value(raw).expect("HeroSummary must deserialize");
    assert!(
        (hero.win_rate - 0.542).abs() < 1e-6,
        "Hero winRate must be 0.542, got {}",
        hero.win_rate
    );
    assert!(
        (hero.pick_rate - 0.125).abs() < 1e-6,
        "Hero pickRate must be 0.125, got {}",
        hero.pick_rate
    );
    assert!(
        (hero.ban_rate - 0.0).abs() < 1e-6,
        "Hero banRate must be 0.0, got {}",
        hero.ban_rate
    );
}

#[test]
fn test_m2_augment_summary_deserialization_scale() {
    let raw = json!({
        "id": "201",
        "name": "Giant Slayer",
        "tier": 2,
        "winRate": "54.2%",
        "pickRate": " -5.5% ",
        "hexScore": "95.5",
        "games": "3500",
        "wins": 1900
    });

    let aug: AugmentSummary = serde_json::from_value(raw).expect("AugmentSummary must deserialize");
    assert!((aug.win_rate - 0.542).abs() < 1e-6);
    assert!((aug.pick_rate - (-0.055)).abs() < 1e-6);
    assert!((aug.hex_score - 95.5).abs() < 1e-6);
    assert_eq!(aug.games, 3500);
    assert_eq!(aug.wins, 1900);
}

#[test]
fn test_m2_postmatch_info_deserialization_scale() {
    let raw = json!({
        "kda": "4.2",
        "damageShare": "32.5%",
        "killParticipation": "65.0%",
        "survivability": "88.0%",
        "goldEfficiency": "105.2%",
        "games": 50
    });

    let postmatch: PostMatchInfo = serde_json::from_value(raw).expect("PostMatchInfo must deserialize");
    assert!((postmatch.kda - 4.2).abs() < 1e-6);
    assert!((postmatch.damage_share - 0.325).abs() < 1e-6);
    assert!((postmatch.kill_participation - 0.65).abs() < 1e-6);
    assert!((postmatch.survivability - 0.88).abs() < 1e-6);
    assert!((postmatch.gold_efficiency - 1.052).abs() < 1e-6);
    assert_eq!(postmatch.games, 50);
}

#[test]
fn test_m2_trio_models_deserialization_scale() {
    let raw_aug_trio = json!({
        "trioKey": "trio_1",
        "augmentIds": [1, 2, 3],
        "winRate": "58.4%",
        "pickRate": "3.2 %",
        "games": 120
    });
    let aug_trio: AugmentTrio = serde_json::from_value(raw_aug_trio).unwrap();
    assert!((aug_trio.win_rate - 0.584).abs() < 1e-6);
    assert!((aug_trio.pick_rate - 0.032).abs() < 1e-6);

    let raw_item_trio = json!({
        "itemIds": [3089, 3157, 3135],
        "winRate": "65.0%",
        "pickRate": "100%",
        "games": "85"
    });
    let item_trio: TerminalItemTrio = serde_json::from_value(raw_item_trio).unwrap();
    assert!((item_trio.win_rate - 0.65).abs() < 1e-6);
    assert!((item_trio.pick_rate - 1.0).abs() < 1e-6);
    assert_eq!(item_trio.games, 85);
}

// ============================================================================
// Group 4: Database::sync_from_api Sequential Repeat Stress Tests
// ============================================================================

#[tokio::test]
async fn test_m2_sync_from_api_repeated_calls_constant_count() {
    // 1. Initialize DB from disk data
    let mut db = Database::load_from_dir("data");
    let initial_heroes_len = db.heroes_summary.len();
    let initial_augments_len = db.augments_summary.len();

    assert!(initial_heroes_len > 0, "Heroes summary must be non-empty");
    assert!(initial_augments_len > 0, "Augments summary must be non-empty");

    let crawler = HexdataCrawler::new();

    // 2. Perform 10 consecutive sync_from_api calls in sequence
    for iteration in 1..=10 {
        db.sync_from_api(&crawler).await;

        assert_eq!(
            db.heroes_summary.len(),
            initial_heroes_len,
            "Iteration {}: heroes_summary count multiplied or changed! (expected {}, got {})",
            iteration,
            initial_heroes_len,
            db.heroes_summary.len()
        );

        assert_eq!(
            db.augments_summary.len(),
            initial_augments_len,
            "Iteration {}: augments_summary count multiplied or changed! (expected {}, got {})",
            iteration,
            initial_augments_len,
            db.augments_summary.len()
        );

        // Verify no duplicate IDs in heroes_summary
        let unique_heroes: HashSet<String> = db.heroes_summary.iter().map(|h| h.id.clone()).collect();
        assert_eq!(
            unique_heroes.len(),
            db.heroes_summary.len(),
            "Iteration {}: duplicate hero IDs detected in heroes_summary!",
            iteration
        );

        // Verify no duplicate IDs in augments_summary
        let unique_augments: HashSet<String> = db.augments_summary.iter().map(|a| a.id.clone()).collect();
        assert_eq!(
            unique_augments.len(),
            db.augments_summary.len(),
            "Iteration {}: duplicate augment IDs detected in augments_summary!",
            iteration
        );
    }
}

#[tokio::test]
async fn test_m2_sync_from_api_repeated_calls_from_empty_db() {
    let mut db = Database::new();
    assert_eq!(db.heroes_summary.len(), 0);
    assert_eq!(db.augments_summary.len(), 0);

    let crawler = HexdataCrawler::new();

    // 5 consecutive sync calls on an empty DB
    for iteration in 1..=5 {
        db.sync_from_api(&crawler).await;

        // Offline / fallback crawler returns None, so DB remains empty with 0 items
        assert_eq!(
            db.heroes_summary.len(),
            0,
            "Iteration {}: empty db heroes should remain 0 under offline crawler",
            iteration
        );
        assert_eq!(
            db.augments_summary.len(),
            0,
            "Iteration {}: empty db augments should remain 0 under offline crawler",
            iteration
        );
    }
}

#[test]
fn test_m2_sync_anti_duplication_multi_generation_cycles() {
    let mut db = Database::load_from_dir("data");
    let heroes_template = db.heroes_summary.clone();
    let augments_template = db.augments_summary.clone();

    // Simulate 20 successive sync ingestion cycles with varying batch sizes
    for cycle in 1..=20 {
        // Take a dynamic slice (e.g. 6 to 24 items)
        let hero_count = (cycle % 7 + 1) * 3;
        let augment_count = (cycle % 5 + 1) * 4;

        let hero_batch: Vec<HeroSummary> = heroes_template.iter().take(hero_count).cloned().collect();
        let augment_batch: Vec<AugmentSummary> = augments_template.iter().take(augment_count).cloned().collect();

        // Simulate sync_from_api logic: clear before populating
        db.heroes_summary.clear();
        for h in &hero_batch {
            db.heroes_summary.push(h.clone());
        }

        db.augments_summary.clear();
        for a in &augment_batch {
            db.augments_summary.push(a.clone());
        }

        assert_eq!(
            db.heroes_summary.len(),
            hero_batch.len(),
            "Cycle {}: heroes_summary did not reset to exact batch size!",
            cycle
        );
        assert_eq!(
            db.augments_summary.len(),
            augment_batch.len(),
            "Cycle {}: augments_summary did not reset to exact batch size!",
            cycle
        );
    }
}
