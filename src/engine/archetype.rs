use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchetypeDefinition {
    pub id: String,
    pub name: String,
    pub badge: String,
    pub description: String,
    pub favored_tags: Vec<String>,
    pub forbidden_tags: Vec<String>,
    pub signature_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineMetadata {
    pub archetypes: HashMap<String, ArchetypeDefinition>,
    pub famous_multi_archetype_heroes: HashMap<String, Value>,
    pub champion_tactical_traits: HashMap<String, Value>,
    pub counter_solutions: HashMap<String, Value>,
    pub item_feature_tags: HashMap<String, Vec<String>>,
    pub semantic_patterns: HashMap<String, Value>,
}

lazy_static! {
    pub static ref ENGINE_META: EngineMetadata = {
        let raw = include_str!("../../data/engine_metadata.json");
        serde_json::from_str(raw).expect("Failed to parse engine_metadata.json")
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchetypeInfo {
    pub locked_archetype_id: String,
    pub name: String,
    pub badge: String,
    pub description: String,
    pub is_locked: bool,
    pub lock_reason: String,
    pub favored_tags: Vec<String>,
    pub forbidden_tags: Vec<String>,
}

pub struct ArchetypeEngine;

impl ArchetypeEngine {
    pub fn detect_archetype(
        hero_id: &str,
        owned_items: &[String],
        active_augments: &[Value],
        candidate_items: &[Value],
        _resolve_conflicts: bool,
    ) -> ArchetypeInfo {
        let _hid = hero_id.trim();

        // 1. Check if user already owns major items
        let mut valid_owned = Vec::new();
        for i in owned_items {
            let s = i.trim();
            if !s.is_empty() && s != "0" && s != "None" {
                valid_owned.push(s.to_string());
            }
        }

        if !valid_owned.is_empty() {
            // Check signature items matching
            for (arch_id, arch) in &ENGINE_META.archetypes {
                for sig in &arch.signature_items {
                    if valid_owned.iter().any(|o| o == sig) {
                        return ArchetypeInfo {
                            locked_archetype_id: arch_id.clone(),
                            name: arch.name.clone(),
                            badge: arch.badge.clone(),
                            description: arch.description.clone(),
                            is_locked: true,
                            lock_reason: format!("根据已出核心装备[{}]锁定流派", sig),
                            favored_tags: arch.favored_tags.clone(),
                            forbidden_tags: arch.forbidden_tags.clone(),
                        };
                    }
                }
            }
        }

        // 2. Infer from augments or candidate items
        // Default to AP_POKE_BURST or CRIT_MARKSMAN based on candidate items top stats
        let mut ap_score = 0;
        let mut ad_score = 0;
        for c in candidate_items.iter().take(5) {
            let iid = c.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(tags) = ENGINE_META.item_feature_tags.get(iid) {
                if tags.iter().any(|t| t == "AP" || t == "MANA") {
                    ap_score += 1;
                }
                if tags.iter().any(|t| t == "AD" || t == "CRIT" || t == "LETHALITY") {
                    ad_score += 1;
                }
            }
        }

        let arch_id = if ap_score >= ad_score {
            "AP_POKE_BURST"
        } else {
            "CRIT_MARKSMAN"
        };

        if let Some(arch) = ENGINE_META.archetypes.get(arch_id) {
            ArchetypeInfo {
                locked_archetype_id: arch_id.to_string(),
                name: arch.name.clone(),
                badge: arch.badge.clone(),
                description: arch.description.clone(),
                is_locked: false,
                lock_reason: "基于英雄定位推荐默认流派".to_string(),
                favored_tags: arch.favored_tags.clone(),
                forbidden_tags: arch.forbidden_tags.clone(),
            }
        } else {
            ArchetypeInfo::default()
        }
    }

    pub fn filter_and_boost_by_archetype(
        candidate_items: &[Value],
        archetype_info: &ArchetypeInfo,
        urgent_counter_ids: &HashSet<String>,
        strictness: f64,
        resolve_conflicts: bool,
    ) -> Vec<Value> {
        let favored: HashSet<&str> = archetype_info.favored_tags.iter().map(|s| s.as_str()).collect();
        let forbidden: HashSet<&str> = archetype_info.forbidden_tags.iter().map(|s| s.as_str()).collect();

        let mut refined = Vec::new();

        for it in candidate_items {
            let mut c = it.clone();
            let iid = c.get("itemId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let empty_tags = Vec::new();
            let tags = ENGINE_META.item_feature_tags.get(&iid).unwrap_or(&empty_tags);

            let is_counter_exempt = resolve_conflicts && urgent_counter_ids.contains(&iid);
            let has_forbidden = tags.iter().any(|t| forbidden.contains(t.as_str()));

            let mut score = c.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(50.0);

            if has_forbidden && !is_counter_exempt {
                if resolve_conflicts && archetype_info.is_locked {
                    score *= (1.0 - strictness).max(0.1);
                    c["isArchetypeConflict"] = Value::Bool(true);
                }
            } else if tags.iter().any(|t| favored.contains(t.as_str())) {
                score *= 1.25;
                c["isArchetypeFavored"] = Value::Bool(true);
            }

            c["tacticalScore"] = Value::from((score * 10.0).round() / 10.0);
            refined.push(c);
        }

        refined
    }

    pub fn align_terminal_trio_with_archetype(
        all_trios: &[Value],
        _archetype_info: &ArchetypeInfo,
        _owned_item_ids: &HashSet<String>,
        _counter_items: &[Value],
        _resolve_conflicts: bool,
        _trio_mode: &str,
    ) -> Option<Value> {
        all_trios.first().cloned()
    }
}
