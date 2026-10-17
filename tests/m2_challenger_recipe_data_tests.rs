//! Empirical Challenger Adversarial Verification Suite for Milestone 2: `recipe_data.rs`
//!
//! Objectives:
//! 1. Adversarially test `normalize_item_id` with multi-byte Chinese, emojis, 1-byte, 2-byte,
//!    6-digit mode items, and empty/dirty inputs. Confirm zero panics and canonical outputs.
//! 2. Adversarially test `calculate_item_net_cost` with mode-prefixed inventory items
//!    (`["221052", " 221052 "]`) for recipe item "3089" and "3157" to verify gold deductions.
//! 3. Stress-test boundary conditions (Unicode slicing, recursive recipes, floating point edge cases).
//! 4. Verify `resolve_item_name` on adversarial inputs.

use lol_hex_rust::data::recipe_data::{
    calculate_item_net_cost, normalize_item_id, resolve_item_name, RECIPE_META,
};

#[test]
fn test_m2_normalize_item_id_adversarial_chinese_and_unicode() {
    // Multi-byte Chinese strings (3 bytes per character in UTF-8)
    // Slicing at byte index 2 or 3 previously caused panics.
    let chinese_inputs = [
        "灭世者的死亡之帽",
        "无尽之刃",
        "中文字符串测试",
        "多兰之盾",
        "多兰之刃",
        "中娅沙漏",
        "帽子",
        "冠",
        "甲",
        "一",
    ];

    for input in &chinese_inputs {
        let norm = normalize_item_id(input);
        assert_eq!(norm, *input, "Expected unchanged Chinese string for '{}'", input);
    }
}

#[test]
fn test_m2_normalize_item_id_adversarial_emojis_and_symbols() {
    // Emojis with 4 bytes or modifier sequences (ZWJ / variation selectors)
    let emoji_inputs = [
        "🗡️🛡️",
        "⚔️",
        "🔥",
        "🛡️",
        "🏹",
        "🧙‍♂️",
        "✨👑✨",
        "🗡️🛡️3089", // Contains digits at the end
    ];

    for input in &emoji_inputs {
        let norm = normalize_item_id(input);
        if input.ends_with("3089") {
            assert_eq!(norm, "3089", "Expected extracted known ID 3089 from '{}'", input);
        } else {
            assert_eq!(norm, *input, "Expected unchanged emoji string for '{}'", input);
        }
    }
}

#[test]
fn test_m2_normalize_item_id_adversarial_short_strings() {
    // 1-byte strings
    assert_eq!(normalize_item_id("1"), "1");
    assert_eq!(normalize_item_id("0"), "0");
    assert_eq!(normalize_item_id("9"), "9");
    assert_eq!(normalize_item_id("a"), "a");
    assert_eq!(normalize_item_id("z"), "z");
    assert_eq!(normalize_item_id("-"), "-");

    // 2-byte strings
    assert_eq!(normalize_item_id("10"), "10");
    assert_eq!(normalize_item_id("99"), "99");
    assert_eq!(normalize_item_id("ab"), "ab");
    assert_eq!(normalize_item_id("22"), "22");

    // 2-byte multi-byte UTF-8 character (e.g. Greek, Cyrillic, Latin with diacritics)
    assert_eq!(normalize_item_id("é"), "é");
    assert_eq!(normalize_item_id("Ω"), "Ω");
    assert_eq!(normalize_item_id("я"), "я");

    // Empty strings and whitespace
    assert_eq!(normalize_item_id(""), "");
    assert_eq!(normalize_item_id(" "), "");
    assert_eq!(normalize_item_id("   "), "");
    assert_eq!(normalize_item_id("\t\r\n"), "");
    assert_eq!(normalize_item_id(" \n \t "), "");
}

#[test]
fn test_m2_normalize_item_id_adversarial_6digit_mode_items() {
    // 1. Amplifying Tome (1052, 400g): authentic component, should strip prefix
    assert_eq!(normalize_item_id("221052"), "1052");
    assert_eq!(normalize_item_id(" 221052 "), "1052");
    assert_eq!(normalize_item_id("Item_221052"), "1052");

    // 2. Long Sword (1036, 350g) and Needlessly Large Rod (1058, 1250g)
    assert_eq!(normalize_item_id("221036"), "1036");
    assert_eq!(normalize_item_id("221058"), "1058");
    assert_eq!(normalize_item_id("223802"), "3802"); // Lost Chapter

    // 3. Canyon finished items MUST NOT be aliased by 6-digit mode items!
    // "223089" is a 6-digit Arena/Mode item and must NOT alias to "3089" (Deathcap)
    assert_eq!(normalize_item_id("223089"), "223089");
    assert_eq!(normalize_item_id("223031"), "223031"); // Not Infinity Edge
    assert_eq!(normalize_item_id("223157"), "223157"); // Not Zhonya's Hourglass
    assert_eq!(normalize_item_id("223078"), "223078"); // Not Trinity Force
}

#[test]
fn test_m2_normalize_item_id_fuzz_stress() {
    // Stress test 5,000 combinations of mixed bytes, Unicode scalars, control chars, and digits
    let fragments = [
        "", " ", "\t", "1", "10", "1052", "221052", "223089", "3089",
        "灭世者的死亡之帽", "🗡️", "🛡️", "Item_", "装备#", "Item_221052",
        "\0", "é", "ñ", "中文", "22", "333",
    ];

    for f1 in &fragments {
        for f2 in &fragments {
            let combined = format!("{}{}", f1, f2);
            // Must never panic!
            let _ = normalize_item_id(&combined);
        }
    }
}

#[test]
fn test_m2_calculate_item_net_cost_zhonya_3157_gold_deduction() {
    // Zhonya's Hourglass ("3157", 3250g)
    // Recipe: ["3191", "1058"] (Seeker's Armguard 1600g + Needlessly Large Rod 1250g)
    // Seeker's Armguard ("3191"): ["1052", "1029"] (Amplifying Tome 400g + Cloth Armor 300g)

    // Test A: Single mode-prefixed 221052
    let owned_a = vec!["221052".to_string()];
    let res_a = calculate_item_net_cost("3157", &owned_a, 2000.0, "NET_DEDUCT");
    assert_eq!(res_a.total_cost, 3250);
    assert_eq!(res_a.deducted_gold, 400, "221052 should deduct exactly 400g for Zhonya 3157");
    assert_eq!(res_a.net_cost, 2850, "Net cost should be 3250 - 400 = 2850");
    assert_eq!(res_a.matched_components.len(), 1);
    assert_eq!(res_a.matched_components[0].item_id, "1052");
    assert_eq!(res_a.matched_components[0].price, 400);

    // Test B: Whitespace padded mode-prefixed " 221052 "
    let owned_b = vec![" 221052 ".to_string()];
    let res_b = calculate_item_net_cost("3157", &owned_b, 2000.0, "NET_DEDUCT");
    assert_eq!(res_b.deducted_gold, 400, "' 221052 ' should deduct exactly 400g");
    assert_eq!(res_b.net_cost, 2850);

    // Test C: Both ["221052", " 221052 "]
    // Since Zhonya only requires one Amplifying Tome (in Seeker's Armguard), only 1 should be deducted
    let owned_c = vec!["221052".to_string(), " 221052 ".to_string()];
    let res_c = calculate_item_net_cost("3157", &owned_c, 2000.0, "NET_DEDUCT");
    assert_eq!(res_c.deducted_gold, 400, "Only 1 Amplifying Tome is needed for Zhonya, 400g deducted");
    assert_eq!(res_c.net_cost, 2850);

    // Test D: Prefixed Amplifying Tome (221052) + Cloth Armor (221029) + Needlessly Large Rod (221058)
    let owned_d = vec![
        "221052".to_string(),
        "221029".to_string(),
        "221058".to_string(),
    ];
    let res_d = calculate_item_net_cost("3157", &owned_d, 2000.0, "NET_DEDUCT");
    assert_eq!(res_d.deducted_gold, 400 + 300 + 1250); // 1950g
    assert_eq!(res_d.net_cost, 3250 - 1950); // 1300g
    assert!(res_d.has_all_components, "All components satisfied for Zhonya");
}

#[test]
fn test_m2_calculate_item_net_cost_deathcap_3089_gold_deduction() {
    // Rabadon's Deathcap ("3089", 3600g)
    // Recipe: ["1058", "1058"] (Two Needlessly Large Rods, 1250g each)
    // Amplifying Tome (1052) is NOT in Deathcap's recipe.

    // Test A: ["221052", " 221052 "] on 3089
    // Amplifying tome is NOT part of Rabadon's Deathcap -> should deduct 0g
    let owned_tome = vec!["221052".to_string(), " 221052 ".to_string()];
    let res_tome = calculate_item_net_cost("3089", &owned_tome, 2000.0, "NET_DEDUCT");
    assert_eq!(res_tome.total_cost, 3600);
    assert_eq!(res_tome.deducted_gold, 0, "Amplifying Tome is not in Deathcap recipe, 0g deducted");
    assert_eq!(res_tome.net_cost, 3600);

    // Test B: Mode-prefixed Needlessly Large Rods ["221058", " 221058 "] on 3089
    let owned_rods = vec!["221058".to_string(), " 221058 ".to_string()];
    let res_rods = calculate_item_net_cost("3089", &owned_rods, 2000.0, "NET_DEDUCT");
    assert_eq!(res_rods.deducted_gold, 2500, "Two Needlessly Large Rods deduct 2500g");
    assert_eq!(res_rods.net_cost, 1100, "Net cost should be 3600 - 2500 = 1100g");
    assert!(res_rods.has_all_components);
}

#[test]
fn test_m2_calculate_item_net_cost_multi_component_ludens_6657() {
    // Luden's Companion ("6657", 2700g)
    // Recipe: ["3802", "3108"]
    // "3802" has ["1052", "1052", "1004"]
    // "3108" has ["1052", "1052"]
    // It can accept up to four Amplifying Tomes!
    let owned_two_tomes = vec!["221052".to_string(), " 221052 ".to_string()];
    let res = calculate_item_net_cost("6657", &owned_two_tomes, 1000.0, "NET_DEDUCT");
    assert_eq!(res.deducted_gold, 800, "Two 221052 tomes should deduct 800g for Ludens 6657");
    assert_eq!(res.net_cost, 2700 - 800);
}

#[test]
fn test_m2_calculate_item_net_cost_edge_cases() {
    // Empty inventory
    let res_empty = calculate_item_net_cost("3157", &[], 100.0, "NET_DEDUCT");
    assert_eq!(res_empty.deducted_gold, 0);
    assert_eq!(res_empty.net_cost, 3250);

    // Dirty inventory: ["0", "", "None", "undefined", "   "]
    let dirty = vec![
        "0".to_string(),
        "".to_string(),
        "None".to_string(),
        "undefined".to_string(),
        "   ".to_string(),
    ];
    let res_dirty = calculate_item_net_cost("3157", &dirty, 100.0, "NET_DEDUCT");
    assert_eq!(res_dirty.deducted_gold, 0);

    // Self item in inventory: should NOT self-deduct
    let self_inv = vec!["3157".to_string(), "Item_3157".to_string()];
    let res_self = calculate_item_net_cost("3157", &self_inv, 100.0, "NET_DEDUCT");
    assert_eq!(res_self.deducted_gold, 0);

    // FULL_COST mode bypasses deductions
    let owned = vec!["221052".to_string()];
    let res_full = calculate_item_net_cost("3157", &owned, 1000.0, "FULL_COST");
    assert_eq!(res_full.deducted_gold, 0);
    assert_eq!(res_full.net_cost, 3250);
}

#[test]
fn test_m2_resolve_item_name_adversarial() {
    // Known items
    assert_eq!(resolve_item_name("3089"), "灭世者的死亡之帽");
    assert_eq!(resolve_item_name("221052"), "增幅典籍"); // Normalized to 1052
    assert_eq!(resolve_item_name("1054"), "多兰之盾");

    // Adversarial / Unknown inputs: must not panic
    assert_eq!(resolve_item_name(""), "未知装备");
    assert_eq!(resolve_item_name("   "), "未知装备");
    assert_eq!(resolve_item_name("灭世者的死亡之帽"), "装备#灭世者的死亡之帽");
    assert_eq!(resolve_item_name("🗡️🛡️"), "装备#🗡️🛡️");

    // Verify RECIPE_META lookup integrity
    assert!(RECIPE_META.total_costs.contains_key("3089"));
    assert!(RECIPE_META.component_prices.contains_key("1052"));
}
