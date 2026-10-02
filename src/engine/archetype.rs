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
    #[serde(default)]
    pub archetypes: HashMap<String, ArchetypeDefinition>,
    #[serde(default)]
    pub famous_multi_archetype_heroes: HashMap<String, Value>,
    #[serde(default)]
    pub champion_tactical_traits: HashMap<String, Value>,
    #[serde(default)]
    pub counter_solutions: HashMap<String, Value>,
    #[serde(default)]
    pub item_feature_tags: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub semantic_patterns: Vec<Value>,
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

        // 2. Infer from augments and candidate items against all archetypes
        let mut arch_scores: HashMap<&str, i32> = HashMap::new();
        for (arch_id, arch) in &ENGINE_META.archetypes {
            let mut score = 0;
            // Match candidate items
            for c in candidate_items.iter().take(8) {
                let iid = c.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(tags) = ENGINE_META.item_feature_tags.get(iid) {
                    for t in tags {
                        if arch.favored_tags.contains(t) {
                            score += 3;
                        }
                        if arch.forbidden_tags.contains(t) {
                            score -= 3;
                        }
                    }
                }
            }
            // Match active augments
            for a in active_augments {
                let aug_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                for fav in &arch.favored_tags {
                    if aug_name.contains(fav) {
                        score += 5;
                    }
                }
            }
            arch_scores.insert(arch_id.as_str(), score);
        }

        let arch_id = arch_scores
            .into_iter()
            .max_by_key(|&(_, score)| score)
            .map(|(id, _)| id)
            .unwrap_or("CRIT_MARKSMAN");

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

    fn extract_trio_item_ids(trio: &Value) -> Vec<String> {
        let mut ids = Vec::new();
        let candidates = [
            trio.get("itemIds"),
            trio.get("items"),
            trio.get("itemIdList"),
        ];
        for cand in candidates.into_iter().flatten() {
            if let Some(arr) = cand.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        ids.push(s.to_string());
                    } else if let Some(n) = v.as_i64() {
                        ids.push(n.to_string());
                    }
                }
                if !ids.is_empty() {
                    return ids;
                }
            }
        }
        ids
    }

    pub fn align_terminal_trio_with_archetype(
        all_trios: &[Value],
        archetype_info: &ArchetypeInfo,
        owned_item_ids: &HashSet<String>,
        counter_items: &[Value],
        resolve_conflicts: bool,
        trio_mode: &str,
    ) -> Option<Value> {
        if all_trios.is_empty() {
            return None;
        }

        let favored: HashSet<&str> = archetype_info.favored_tags.iter().map(|s| s.as_str()).collect();
        let forbidden: HashSet<&str> = archetype_info.forbidden_tags.iter().map(|s| s.as_str()).collect();
        let counter_set: HashSet<String> = counter_items
            .iter()
            .filter_map(|it| it.get("itemId").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let mut scored_trios: Vec<(f64, Value)> = all_trios
            .iter()
            .map(|trio| {
                let wr = trio.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let games = trio.get("games").and_then(|v| v.as_i64()).unwrap_or(100);
                let mut score = wr * 100.0;

                let items = Self::extract_trio_item_ids(trio);
                let mut owned_match_count = 0;

                for iid in &items {
                    // 1. Owned item match
                    if owned_item_ids.contains(iid) {
                        score += 20.0;
                        owned_match_count += 1;
                    }

                    // 2. Archetype tags
                    if let Some(tags) = ENGINE_META.item_feature_tags.get(iid) {
                        if tags.iter().any(|t| favored.contains(t.as_str())) {
                            score += 5.0;
                        }
                        if resolve_conflicts && archetype_info.is_locked && tags.iter().any(|t| forbidden.contains(t.as_str())) {
                            score -= 15.0;
                        }
                    }

                    // 3. Counter item synergy
                    if (trio_mode == "DYNAMIC_COUNTER" || trio_mode == "IRON_COUNTER") && counter_set.contains(iid) {
                        score += 8.0;
                    }
                }

                if trio_mode == "STRICT_TRIO" {
                    // In strict trio mode, sample size and pure winrate carry extra weight
                    let smoothed_wr = (wr * games as f64 + 0.5 * 50.0) / (games as f64 + 50.0);
                    score += (smoothed_wr - 0.5) * 40.0;
                }

                let mut enriched = trio.clone();
                enriched["matchScore"] = Value::from((score * 10.0).round() / 10.0);
                enriched["ownedMatches"] = Value::from(owned_match_count);

                (score, enriched)
            })
            .collect();

        scored_trios.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored_trios.into_iter().next().map(|(_, val)| val)
    }
}

