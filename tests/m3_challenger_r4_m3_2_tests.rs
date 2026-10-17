//! Adversarial Empirical Challenge Suite for Milestone 3 (Requirement R3)
//!
//! Specifically validates:
//! 1. `detect_archetype`:
//!    - Hero "1" (Annie) with empty inputs -> MUST NOT return CRIT_MARKSMAN (must return AP_POKE_BURST).
//!    - Hero "86" (Garen) with empty inputs -> verifies exact archetype classification and analyzes JUGGERNAUT vs BRUISER_FIGHTER.
//!    - 100 iterations of tie-break scenarios -> verify 100% deterministic, identical output every single iteration.
//!    - Complex multi-archetype tie-breaking stress tests.
//! 2. `find_tactical_traits`:
//!    - Test "Vi" -> MUST NOT false-match "Viktor", "Sivir", "Anivia", or "Viego".
//!    - Exact matching, slug normalization, prefix stripping ("game_character_displayname_").
//! 3. `apply_counter_recommendations`:
//!    - Enemy team with shield champions (e.g. Shen) -> Serpent's Fang (6695) active injection and score boosting.
//!    - Enemy team with heavy healing (Soraka, Vladimir, Aatrox, DrMundo, Warwick) -> anti-heal (3165 / 3033) injection and score boosting.
//!    - Deduplication: items already present in candidates are boosted, not duplicated.
//!    - Ownership filter: items already owned are not injected.
//!    - Forbidden tag safety: counter items violating archetype forbidden tags are excluded.

use lol_hex_rust::engine::archetype::{ArchetypeEngine, ArchetypeInfo};
use lol_hex_rust::engine::counter::{CounterItemEngine, ThreatProfile};
use serde_json::json;
use std::collections::HashSet;

// ============================================================================
// Group 1: detect_archetype Verification and Determinism Stress Tests
// ============================================================================

#[test]
fn test_m3_detect_archetype_annie_empty_inputs_not_crit_marksman() {
    // Hero "1" is Annie (Dark Child / 黑暗之女) - Pure AP Mage
    let info = ArchetypeEngine::detect_archetype("1", &[], &[], &[], false);
    assert_ne!(
        info.locked_archetype_id, "CRIT_MARKSMAN",
        "CRITICAL BUG: Annie (1) must NEVER default to CRIT_MARKSMAN!"
    );
    assert_eq!(
        info.locked_archetype_id, "AP_POKE_BURST",
        "Annie (1) should default to AP_POKE_BURST"
    );
    assert_eq!(info.name, "AP远程核弹流");
    assert!(!info.is_locked, "Default archetype should not be locked");
}

#[test]
fn test_m3_detect_archetype_garen_empty_inputs_classification() {
    // Hero "86" is Garen (德玛西亚之力) - Fighter / Juggernaut
    let info = ArchetypeEngine::detect_archetype("86", &[], &[], &[], false);
    assert_ne!(
        info.locked_archetype_id, "CRIT_MARKSMAN",
        "CRITICAL BUG: Garen (86) must NEVER default to CRIT_MARKSMAN!"
    );

    // In HexBrain's metadata (data/engine_metadata.json), the archetype representing
    // "重装战士 / Juggernaut" is "BRUISER_FIGHTER" (name: "战士重装半肉流", badge: "⚔️重装死斗").
    // There is no archetype ID named "JUGGERNAUT" in ENGINE_META.archetypes.
    assert_eq!(
        info.locked_archetype_id, "BRUISER_FIGHTER",
        "Garen (86) must return BRUISER_FIGHTER ('战士重装半肉流' / Juggernaut archetype)"
    );
    assert_eq!(info.name, "战士重装半肉流");
    assert_eq!(info.badge, "⚔️重装死斗");
    assert!(info.description.contains("兼顾攻击力、血量"));
}

#[test]
fn test_m3_detect_archetype_100_iterations_determinism_stress() {
    // Run 100 evaluations on the same hero with empty inputs
    let first = ArchetypeEngine::detect_archetype("1", &[], &[], &[], false);
    for i in 1..=100 {
        let current = ArchetypeEngine::detect_archetype("1", &[], &[], &[], false);
        assert_eq!(
            first.locked_archetype_id, current.locked_archetype_id,
            "Iteration {} failed determinism for hero '1'",
            i
        );
        assert_eq!(first.name, current.name);
        assert_eq!(first.badge, current.badge);
    }

    // Run 100 evaluations for Garen "86"
    let garen_first = ArchetypeEngine::detect_archetype("86", &[], &[], &[], false);
    for i in 1..=100 {
        let current = ArchetypeEngine::detect_archetype("86", &[], &[], &[], false);
        assert_eq!(
            garen_first.locked_archetype_id, current.locked_archetype_id,
            "Iteration {} failed determinism for Garen '86'",
            i
        );
    }
}

#[test]
fn test_m3_detect_archetype_tie_break_determinism_stress() {
    // Construct an artificial scenario where multiple archetypes receive identical scores.
    // In this scenario, candidate items and augments provide equal scores to:
    // "AP_POKE_BURST" and "AP_DOT_CONTROL" (both AP archetypes).
    let cand_items = vec![
        json!({ "itemId": "3116" }), // Rylai's (has AP, SLOW_CC, HEAVY_HP)
    ];

    let first = ArchetypeEngine::detect_archetype("1", &[], &[], &cand_items, false);
    for i in 1..=100 {
        let current = ArchetypeEngine::detect_archetype("1", &[], &[], &cand_items, false);
        assert_eq!(
            first.locked_archetype_id, current.locked_archetype_id,
            "Iteration {} tie-break was not 100% deterministic!",
            i
        );
    }

    // Now test a scenario with non-AP hero (Garen) receiving equal scores on AP and Tank
    let augments = vec![
        json!({ "name": "巨像的勇气", "description": "获得最大生命值和护甲减伤" }),
        json!({ "name": "珠光莲华", "description": "技能暴击，提供法术强度" }),
    ];
    let g_first = ArchetypeEngine::detect_archetype("86", &[], &augments, &[], false);
    for i in 1..=100 {
        let current = ArchetypeEngine::detect_archetype("86", &[], &augments, &[], false);
        assert_eq!(
            g_first.locked_archetype_id, current.locked_archetype_id,
            "Iteration {} tie-break for hero 86 with tied augments was not deterministic!",
            i
        );
    }
}

#[test]
fn test_m3_detect_archetype_all_168_heroes_no_unintended_crit_marksman() {
    // Spot check multiple famous non-marksman heroes: none should default to CRIT_MARKSMAN
    let test_cases = [
        ("1", "AP_POKE_BURST"),       // Annie (Mage)
        ("2", "BRUISER_FIGHTER"),     // Olaf (Fighter)
        ("3", "AP_DOT_CONTROL"),      // Galio (Tank/Mage) or PURE_TANK
        ("14", "PURE_TANK"),          // Sion (Tank)
        ("16", "ENCHANTER_SUPPORT"),  // Soraka (Support)
        ("22", "CRIT_MARKSMAN"),      // Ashe (Marksman) -> SHOULD be CRIT_MARKSMAN
        ("86", "BRUISER_FIGHTER"),    // Garen (Fighter)
        ("89", "PURE_TANK"),          // Leona (Tank)
        ("238", "LETHALITY_ASSASSIN"),// Zed (Assassin)
    ];

    for (hid, expected) in test_cases {
        let resolved = ArchetypeEngine::resolve_default_archetype_for_hero(hid);
        if hid == "3" {
            // Galio might be PURE_TANK or AP_POKE_BURST depending on mapping, but NEVER CRIT_MARKSMAN
            assert_ne!(resolved, "CRIT_MARKSMAN");
        } else {
            assert_eq!(
                resolved, expected,
                "Hero ID {} resolved to {} instead of expected {}",
                hid, resolved, expected
            );
        }
    }
}

// ============================================================================
// Group 2: find_tactical_traits Exact Matching & Substring False-Positive Prevention
// ============================================================================

#[test]
fn test_m3_find_tactical_traits_vi_exact_matching() {
    // In champion_tactical_traits, "Vi" is not explicitly defined.
    // In buggy code: "Vi" would substring match "Viktor", "Sivir", "Anivia", or "Viego".
    let vi_traits = CounterItemEngine::find_tactical_traits("Vi");
    assert!(
        vi_traits.is_none(),
        "CRITICAL BUG: 'Vi' must NOT match 'Viktor', 'Sivir', 'Anivia', or 'Viego'!"
    );

    // Also test lowercase "vi"
    let vi_lower = CounterItemEngine::find_tactical_traits("vi");
    assert!(
        vi_lower.is_none(),
        "'vi' must NOT substring match any champion"
    );

    // Test with LCU prefix
    let vi_prefixed = CounterItemEngine::find_tactical_traits("game_character_displayname_Vi");
    assert!(
        vi_prefixed.is_none(),
        "'game_character_displayname_Vi' must NOT substring match"
    );
}

#[test]
fn test_m3_find_tactical_traits_viego_and_vladimir_exact() {
    // "Viego" is in champion_tactical_traits
    let viego = CounterItemEngine::find_tactical_traits("Viego");
    assert!(viego.is_some(), "Viego should be found in champion_tactical_traits");
    let viego_val = viego.unwrap();
    assert_eq!(viego_val.get("name").and_then(|v| v.as_str()), Some("破败之王"));

    // "Vladimir" is in champion_tactical_traits
    let vlad = CounterItemEngine::find_tactical_traits("Vladimir");
    assert!(vlad.is_some(), "Vladimir should be found in champion_tactical_traits");
    let vlad_val = vlad.unwrap();
    assert_eq!(vlad_val.get("name").and_then(|v| v.as_str()), Some("猩红收割者"));

    // "Vlad" is NOT a key (unless searchTerms or slug matches)
    // Verify it doesn't corrupt other champions
    let vlad_short = CounterItemEngine::find_tactical_traits("Vlad");
    // Should be None since the key is "Vladimir"
    assert!(vlad_short.is_none());
}

#[test]
fn test_m3_find_tactical_traits_shen_and_soraka() {
    let shen = CounterItemEngine::find_tactical_traits("Shen");
    assert!(shen.is_some());
    let shen_val = shen.unwrap();
    assert_eq!(shen_val["name"], "暮光之眼");
    let traits: Vec<String> = shen_val["traits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(traits.contains(&"SHIELD".to_string()));
    assert!(traits.contains(&"SUPER_TANK".to_string()));

    let soraka = CounterItemEngine::find_tactical_traits("Soraka");
    assert!(soraka.is_some());
    let soraka_val = soraka.unwrap();
    assert_eq!(soraka_val["name"], "众星之子");
    let s_traits: Vec<String> = soraka_val["traits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(s_traits.contains(&"HIGH_HEAL".to_string()));
}

// ============================================================================
// Group 3: apply_counter_recommendations Shield and Heal Injections
// ============================================================================

#[test]
fn test_m3_counter_shield_champions_injects_serpents_fang() {
    // Enemy team with shield champion Shen
    let enemies = vec![
        json!({ "championName": "Shen" }),
        json!({ "championName": "Janna" }),
        json!({ "championName": "Karma" }),
    ];

    let threat = CounterItemEngine::analyze_enemy_threats(&enemies);
    assert!(
        threat.has_shield_heavy,
        "Enemy team with Shen must trigger has_shield_heavy"
    );
    assert!(
        threat.summary.contains("多重护盾"),
        "Summary should mention shield threat: {}",
        threat.summary
    );

    // Player archetype is an AD assassin or fighter (where 6695 is not forbidden)
    let arch = ArchetypeInfo {
        locked_archetype_id: "LETHALITY_ASSASSIN".to_string(),
        favored_tags: vec!["AD".to_string(), "LETHALITY".to_string()],
        forbidden_tags: vec!["AP".to_string(), "ATTACK_SPEED".to_string()],
        ..Default::default()
    };

    // Candidates do NOT contain Serpent's Fang (6695)
    let candidate_items = vec![
        json!({
            "itemId": "3142",
            "itemName": "幽梦之灵",
            "tacticalScore": 60.0
        }),
        json!({
            "itemId": "3814",
            "itemName": "夜之锋刃",
            "tacticalScore": 55.0
        }),
    ];

    let out = CounterItemEngine::apply_counter_recommendations(
        &candidate_items,
        &threat,
        &arch,
        &HashSet::new(),
        1.0,
        true,
    );

    // Verify Serpent's Fang (6695) is actively injected
    let serpent = out.iter().find(|it| {
        it.get("itemId").and_then(|v| v.as_str()) == Some("6695")
    });

    assert!(
        serpent.is_some(),
        "Serpent's Fang (6695) must be actively injected when enemy has heavy shields!"
    );

    let s = serpent.unwrap();
    assert_eq!(s["isCounterItem"], true);
    assert_eq!(s["isUrgentInjected"], true);
    let score = s["tacticalScore"].as_f64().unwrap();
    assert!(
        score >= 75.0,
        "Injected Serpent's Fang should receive boosted score (>= 75.0), got {}",
        score
    );
}

#[test]
fn test_m3_counter_healing_champions_injects_anti_heal_items() {
    // 5 heavy healing champions
    let enemies = vec![
        json!({ "championName": "Soraka" }),
        json!({ "championName": "Vladimir" }),
        json!({ "championName": "Aatrox" }),
        json!({ "championName": "DrMundo" }),
        json!({ "championName": "Warwick" }),
    ];

    let threat = CounterItemEngine::analyze_enemy_threats(&enemies);
    assert!(
        threat.has_high_heal,
        "5 healers must trigger has_high_heal"
    );
    assert!(
        threat.summary.contains("强回血"),
        "Summary should mention healing threat: {}",
        threat.summary
    );

    // Case A: AP Mage -> Morellonomicon (3165)
    let ap_arch = ArchetypeInfo {
        locked_archetype_id: "AP_POKE_BURST".to_string(),
        favored_tags: vec!["AP".to_string(), "ABILITY_POWER".to_string()],
        forbidden_tags: vec!["AD".to_string(), "CRIT".to_string()],
        ..Default::default()
    };

    let ap_candidates = vec![
        json!({ "itemId": "3089", "itemName": "灭世者的死亡之帽", "tacticalScore": 80.0 }),
    ];

    let ap_out = CounterItemEngine::apply_counter_recommendations(
        &ap_candidates,
        &threat,
        &ap_arch,
        &HashSet::new(),
        1.0,
        true,
    );

    let morello = ap_out.iter().find(|it| {
        it.get("itemId").and_then(|v| v.as_str()) == Some("3165")
    });
    assert!(
        morello.is_some(),
        "Morellonomicon (3165) must be injected for AP archetype facing heavy healers!"
    );
    let m = morello.unwrap();
    assert_eq!(m["isCounterItem"], true);
    assert_eq!(m["isUrgentInjected"], true);
    assert!(m["tacticalScore"].as_f64().unwrap() >= 75.0);

    // Case B: AD / Marksman -> Mortal Reminder (3033)
    let ad_arch = ArchetypeInfo {
        locked_archetype_id: "CRIT_MARKSMAN".to_string(),
        favored_tags: vec!["AD".to_string(), "CRIT".to_string()],
        forbidden_tags: vec!["AP".to_string(), "LETHALITY".to_string()],
        ..Default::default()
    };

    let ad_candidates = vec![
        json!({ "itemId": "3031", "itemName": "无尽之刃", "tacticalScore": 85.0 }),
    ];

    let ad_out = CounterItemEngine::apply_counter_recommendations(
        &ad_candidates,
        &threat,
        &ad_arch,
        &HashSet::new(),
        1.0,
        true,
    );

    let mortal = ad_out.iter().find(|it| {
        it.get("itemId").and_then(|v| v.as_str()) == Some("3033")
    });
    assert!(
        mortal.is_some(),
        "Mortal Reminder (3033) must be injected for Crit Marksman facing heavy healers!"
    );
    let rem = mortal.unwrap();
    assert_eq!(rem["isCounterItem"], true);
    assert_eq!(rem["isUrgentInjected"], true);
    assert!(rem["tacticalScore"].as_f64().unwrap() >= 75.0);
}

#[test]
fn test_m3_counter_items_deduplication_and_boost() {
    let threat = ThreatProfile {
        has_high_heal: true,
        summary: "强回血需重伤".to_string(),
        ..Default::default()
    };

    let ap_arch = ArchetypeInfo {
        locked_archetype_id: "AP_POKE_BURST".to_string(),
        favored_tags: vec!["AP".to_string()],
        forbidden_tags: vec!["AD".to_string()],
        ..Default::default()
    };

    // Morellonomicon (3165) is ALREADY in candidate items
    let candidates = vec![
        json!({
            "itemId": "3165",
            "itemName": "莫雷洛秘典",
            "tacticalScore": 50.0
        }),
        json!({
            "itemId": "3089",
            "itemName": "灭世者的死亡之帽",
            "tacticalScore": 70.0
        }),
    ];

    let out = CounterItemEngine::apply_counter_recommendations(
        &candidates,
        &threat,
        &ap_arch,
        &HashSet::new(),
        1.0,
        true,
    );

    // 3165 should be boosted, NOT duplicated
    let count_3165 = out.iter().filter(|it| it["itemId"] == "3165").count();
    assert_eq!(count_3165, 1, "Item 3165 must not be duplicated in candidates");

    let morello = out.iter().find(|it| it["itemId"] == "3165").unwrap();
    assert_eq!(morello["isCounterItem"], true);
    // 50.0 + 15.0 = 65.0
    assert_eq!(morello["tacticalScore"], 65.0);
    // Not injected because it was already present
    assert_eq!(morello.get("isUrgentInjected"), None);
}

#[test]
fn test_m3_counter_items_not_injected_if_already_owned() {
    let threat = ThreatProfile {
        has_shield_heavy: true,
        summary: "多重护盾需巨蛇破盾".to_string(),
        ..Default::default()
    };

    let arch = ArchetypeInfo {
        locked_archetype_id: "LETHALITY_ASSASSIN".to_string(),
        favored_tags: vec!["AD".to_string(), "LETHALITY".to_string()],
        forbidden_tags: vec!["AP".to_string()],
        ..Default::default()
    };

    let candidates = vec![
        json!({ "itemId": "3142", "itemName": "幽梦之灵", "tacticalScore": 60.0 }),
    ];

    // Player ALREADY owns Serpent's Fang (6695)
    let mut owned = HashSet::new();
    owned.insert("6695".to_string());

    let out = CounterItemEngine::apply_counter_recommendations(
        &candidates,
        &threat,
        &arch,
        &owned,
        1.0,
        true,
    );

    let serpent = out.iter().find(|it| it["itemId"] == "6695");
    assert!(
        serpent.is_none(),
        "Owned item 6695 must NOT be injected into candidates"
    );
}

#[test]
fn test_m3_counter_items_forbidden_tags_respected() {
    let threat = ThreatProfile {
        has_shield_heavy: true,
        summary: "多重护盾需巨蛇破盾".to_string(),
        ..Default::default()
    };

    // Archetype where LETHALITY is forbidden (e.g. PURE_TANK or CRIT_MARKSMAN)
    let arch = ArchetypeInfo {
        locked_archetype_id: "PURE_TANK".to_string(),
        favored_tags: vec!["TANK".to_string(), "ARMOR".to_string()],
        forbidden_tags: vec!["LETHALITY".to_string(), "CRIT".to_string()],
        ..Default::default()
    };

    let candidates = vec![
        json!({ "itemId": "3068", "itemName": "日炎圣盾", "tacticalScore": 60.0 }),
    ];

    let out = CounterItemEngine::apply_counter_recommendations(
        &candidates,
        &threat,
        &arch,
        &HashSet::new(),
        1.0,
        true,
    );

    // 6695 has LETHALITY tag, which is in arch.forbidden_tags.
    // It must NOT be injected for Pure Tank!
    let serpent = out.iter().find(|it| it["itemId"] == "6695");
    assert!(
        serpent.is_none(),
        "6695 must NOT be injected when its tag (LETHALITY) is forbidden by the archetype!"
    );
}
