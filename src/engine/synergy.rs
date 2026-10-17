use super::archetype::ENGINE_META;
use serde_json::Value;
use std::collections::HashSet;

pub struct HexItemSynergyEngine;

/// Matches a short token (like "AP", "AD", "CD") ensuring it is not part of a larger ASCII word (e.g. "TRAP", "ESCAPE", "BROADCAST").
pub fn contains_isolated_token(text: &str, token: &str) -> bool {
    let mut start = 0;
    while let Some(idx) = text[start..].find(token) {
        let abs_idx = start + idx;
        let before_ok = if abs_idx == 0 {
            true
        } else {
            let prev = text[..abs_idx].chars().last().unwrap();
            !prev.is_ascii_alphabetic()
        };
        let after_idx = abs_idx + token.len();
        let after_ok = if after_idx >= text.len() {
            true
        } else {
            let next = text[after_idx..].chars().next().unwrap();
            !next.is_ascii_alphabetic()
        };
        if before_ok && after_ok {
            return true;
        }
        start = abs_idx + token.len();
    }
    false
}

impl HexItemSynergyEngine {
    pub fn rerank_items_with_synergy(
        candidate_items: &[Value],
        active_augments: &[Value],
        _owned_items: &HashSet<String>,
        _hero_id: &str,
        synergy_weight: f64,
        _resolve_conflicts: bool,
    ) -> Vec<Value> {
        let mut scored_items = Vec::new();

        let weight = if synergy_weight.is_nan() || synergy_weight.is_infinite() || synergy_weight < 0.0 {
            1.0
        } else {
            synergy_weight.min(5.0)
        };

        let mut aug_text = String::new();
        for a in active_augments {
            if let Some(name) = a.get("name").and_then(|v| v.as_str()) {
                aug_text.push_str(name);
                aug_text.push(' ');
            }
            if let Some(desc) = a.get("description").and_then(|v| v.as_str()) {
                aug_text.push_str(desc);
                aug_text.push(' ');
            }
        }

        let is_ap_synergy = aug_text.contains("法术")
            || aug_text.contains("法强")
            || aug_text.contains("法术强度")
            || aug_text.contains("法穿")
            || aug_text.contains("魔法伤害")
            || aug_text.contains("珠光")
            || contains_isolated_token(&aug_text, "AP")
            || aug_text.contains("巫师")
            || aug_text.contains("帽子")
            || aug_text.contains("秘术");
        let is_ad_crit_synergy = aug_text.contains("暴击")
            || aug_text.contains("无尽")
            || aug_text.contains("攻击力")
            || aug_text.contains("利刃")
            || aug_text.contains("刀刃")
            || aug_text.contains("锋刃")
            || aug_text.contains("巨刃")
            || aug_text.contains("穿甲")
            || aug_text.contains("破甲")
            || aug_text.contains("物理伤害")
            || aug_text.contains("暴伤")
            || aug_text.contains("暴击率")
            || contains_isolated_token(&aug_text, "AD");
        let is_as_onhit_synergy = aug_text.contains("攻速")
            || aug_text.contains("攻击速度")
            || aug_text.contains("攻击特效")
            || aug_text.contains("特效")
            || aug_text.contains("双重")
            || aug_text.contains("附带")
            || aug_text.contains("普攻");
        let is_tank_synergy = aug_text.contains("生命值")
            || aug_text.contains("护甲")
            || aug_text.contains("魔抗")
            || aug_text.contains("魔法抗性")
            || aug_text.contains("巨像")
            || aug_text.contains("减伤")
            || aug_text.contains("最大生命")
            || aug_text.contains("坦度")
            || aug_text.contains("坚韧");
        let is_haste_synergy = aug_text.contains("技能急速")
            || aug_text.contains("冷却")
            || aug_text.contains("急速")
            || aug_text.contains("冷却缩减")
            || contains_isolated_token(&aug_text, "CD");

        for it in candidate_items {
            let mut c = it.clone();
            let iid = c.get("itemId")
                .or_else(|| c.get("id"))
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                .unwrap_or_default();
            let empty_tags = Vec::new();
            let tags = ENGINE_META.item_feature_tags.get(&iid).unwrap_or(&empty_tags);

            let mut synergy_bonus = 0.0;
            if is_ap_synergy && tags.iter().any(|t| t == "AP" || t == "MAGIC_PEN" || t == "ABILITY_POWER") {
                synergy_bonus += 12.0 * weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_ad_crit_synergy && tags.iter().any(|t| t == "CRIT" || t == "AD" || t == "ARMOR_PEN" || t == "LETHALITY") {
                synergy_bonus += 12.0 * weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_as_onhit_synergy && tags.iter().any(|t| t == "ON_HIT" || t == "ATTACK_SPEED") {
                synergy_bonus += 12.0 * weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_tank_synergy && tags.iter().any(|t| t == "TANK" || t == "ARMOR" || t == "MR" || t == "MAGIC_RESIST" || t == "HEALTH" || t == "HEAVY_HP" || t == "HP_STACK") {
                synergy_bonus += 12.0 * weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_haste_synergy && tags.iter().any(|t| t == "ABILITY_HASTE") {
                synergy_bonus += 8.0 * weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }

            let base_score = c.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let final_score = if base_score.is_nan() { 50.0 } else { base_score } + synergy_bonus;
            c["tacticalScore"] = Value::from((final_score * 10.0).round() / 10.0);

            scored_items.push(c);
        }

        // Sort descending by tacticalScore safely with deterministic secondary tie-breaker
        scored_items.sort_by(|a, b| {
            let sa = a.get("tacticalScore").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
            let sb = b.get("tacticalScore").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let ida = a.get("itemId").or_else(|| a.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                    let idb = b.get("itemId").or_else(|| b.get("id")).and_then(|v| v.as_str()).unwrap_or("");
                    ida.cmp(idb)
                })
        });

        scored_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rerank_items_with_synergy_numeric_item_id() {
        let active_augments = vec![json!({
            "name": "珠光护手",
            "description": "法术强度和暴击几率提升"
        })];

        // 3089 = Rabadon's Deathcap (has AP tag in item_feature_tags)
        let candidate_items = vec![
            json!({
                "itemId": 3089,
                "tacticalScore": 50.0
            }),
            json!({
                "itemId": 3031, // Infinity Edge
                "tacticalScore": 50.0
            })
        ];

        let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
            &candidate_items,
            &active_augments,
            &HashSet::new(),
            "103",
            1.0,
            true,
        );

        let cap = reranked.iter().find(|it| it["itemId"] == 3089).unwrap();
        assert_eq!(cap["hasAugmentSynergy"], true);
        assert!(cap["tacticalScore"].as_f64().unwrap() >= 62.0); // 50.0 + 12.0
    }

    #[test]
    fn test_rerank_items_tank_synergy_with_chinese_description() {
        let active_augments = vec![json!({
            "name": "巨像之力",
            "description": "提供大量生命值与护甲坦度，提升生存能力"
        })];

        // 3075 = Thornmail (has ARMOR, TANK)
        let candidate_items = vec![
            json!({ "itemId": "3075", "tacticalScore": 50.0 }),
            json!({ "itemId": "3089", "tacticalScore": 50.0 }),
        ];

        let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
            &candidate_items,
            &active_augments,
            &HashSet::new(),
            "14",
            1.0,
            true,
        );

        let thornmail = reranked.iter().find(|it| it["itemId"] == "3075").unwrap();
        assert_eq!(thornmail["hasAugmentSynergy"], true);
        assert!(thornmail["tacticalScore"].as_f64().unwrap() >= 62.0);
    }

    #[test]
    fn test_rerank_items_ad_crit_and_haste_synergies() {
        let active_augments = vec![
            json!({
                "name": "战争律动",
                "description": "攻击力提升且暴击造成致命撕裂"
            }),
            json!({
                "name": "极速流转",
                "description": "获得额外技能急速与冷却缩减"
            }),
        ];

        let candidate_items = vec![
            json!({ "itemId": "3031", "tacticalScore": 50.0 }), // Infinity Edge (CRIT, AD)
            json!({ "itemId": "3158", "tacticalScore": 50.0 }), // Ionian Boots (ABILITY_HASTE)
        ];

        let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
            &candidate_items,
            &active_augments,
            &HashSet::new(),
            "238",
            1.0,
            true,
        );

        let ie = reranked.iter().find(|it| it["itemId"] == "3031").unwrap();
        assert_eq!(ie["hasAugmentSynergy"], true);
        assert!(ie["tacticalScore"].as_f64().unwrap() >= 62.0);

        let boots = reranked.iter().find(|it| it["itemId"] == "3158").unwrap();
        assert_eq!(boots["hasAugmentSynergy"], true);
        assert!(boots["tacticalScore"].as_f64().unwrap() >= 58.0);
    }

    #[test]
    fn test_tightened_stem_matching_no_false_positives() {
        assert!(!contains_isolated_token("TRAP_CARD", "AP"));
        assert!(!contains_isolated_token("ESCAPE_ROUTE", "AP"));
        assert!(!contains_isolated_token("BROADCAST", "CD"));
        assert!(contains_isolated_token("获得额外 AP 加成", "AP"));
        assert!(contains_isolated_token("+30AP", "AP"));
        assert!(contains_isolated_token("CD缩短", "CD"));

        // Augment contains "迎刃而解" and English word "TRAP"
        // Neither single "刃" nor "TRAP" should trigger AD crit or AP synergy!
        let active_augments = vec![json!({
            "name": "思维飞跃 TRAP",
            "description": "所有困境迎刃而解"
        })];

        let candidate_items = vec![
            json!({ "itemId": "3089", "tacticalScore": 50.0 }), // Rabadon's (AP)
            json!({ "itemId": "3031", "tacticalScore": 50.0 }), // IE (AD, CRIT)
        ];

        let reranked = HexItemSynergyEngine::rerank_items_with_synergy(
            &candidate_items,
            &active_augments,
            &HashSet::new(),
            "103",
            1.0,
            true,
        );

        for item in reranked {
            assert_ne!(item.get("hasAugmentSynergy"), Some(&Value::Bool(true)));
            assert_eq!(item["tacticalScore"], 50.0);
        }
    }
}
