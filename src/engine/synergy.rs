use super::archetype::ENGINE_META;
use serde_json::Value;
use std::collections::HashSet;

pub struct HexItemSynergyEngine;

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

        let is_ap_synergy = aug_text.contains("法术") || aug_text.contains("珠光") || aug_text.contains("AP") || aug_text.contains("巫师");
        let is_ad_crit_synergy = aug_text.contains("暴击") || aug_text.contains("无尽") || aug_text.contains("攻击力") || aug_text.contains("刃");
        let is_as_onhit_synergy = aug_text.contains("攻速") || aug_text.contains("攻击特效") || aug_text.contains("双重");

        for it in candidate_items {
            let mut c = it.clone();
            let iid = c.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
            let empty_tags = Vec::new();
            let tags = ENGINE_META.item_feature_tags.get(iid).unwrap_or(&empty_tags);

            let mut synergy_bonus = 0.0;
            if is_ap_synergy && tags.iter().any(|t| t == "AP" || t == "MAGIC_PEN") {
                synergy_bonus += 12.0 * synergy_weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_ad_crit_synergy && tags.iter().any(|t| t == "CRIT" || t == "AD") {
                synergy_bonus += 12.0 * synergy_weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }
            if is_as_onhit_synergy && tags.iter().any(|t| t == "ON_HIT" || t == "ATTACK_SPEED") {
                synergy_bonus += 12.0 * synergy_weight;
                c["hasAugmentSynergy"] = Value::Bool(true);
            }

            let base_score = c.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(50.0);
            let final_score = base_score + synergy_bonus;
            c["tacticalScore"] = Value::from((final_score * 10.0).round() / 10.0);

            scored_items.push(c);
        }

        // Sort descending by tacticalScore
        scored_items.sort_by(|a, b| {
            let sa = a.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let sb = b.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        scored_items
    }
}
