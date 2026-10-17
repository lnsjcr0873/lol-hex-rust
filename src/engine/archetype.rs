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

    pub static ref HERO_DEFAULT_ARCHETYPE_MAP: HashMap<String, &'static str> = {
        let mut map: HashMap<String, &'static str> = HashMap::new();
        let raw = include_str!("../../data/json/heroes_summary.json");
        if let Ok(heroes) = serde_json::from_str::<Vec<Value>>(raw) {
            for h in heroes {
                let parse_role = |r: &str| -> Option<&'static str> {
                    let r_low = r.trim().to_lowercase();
                    match r_low.as_str() {
                        "mage" | "法师" => Some("AP_POKE_BURST"),
                        "marksman" | "射手" | "adc" => Some("CRIT_MARKSMAN"),
                        "assassin" | "刺客" => Some("LETHALITY_ASSASSIN"),
                        "tank" | "坦克" => Some("PURE_TANK"),
                        "support" | "辅助" | "enchanter" => Some("ENCHANTER_SUPPORT"),
                        "fighter" | "战士" | "bruiser" => Some("BRUISER_FIGHTER"),
                        _ => None,
                    }
                };

                let mut resolved_arch: Option<&'static str> = None;
                if let Some(roles_arr) = h.get("roles").and_then(|v| v.as_array()) {
                    for r in roles_arr {
                        if let Some(s) = r.as_str() {
                            if let Some(arch) = parse_role(s) {
                                resolved_arch = Some(arch);
                                break;
                            }
                        }
                    }
                }
                if resolved_arch.is_none() {
                    if let Some(tags_arr) = h.get("roleTags").and_then(|v| v.as_array()) {
                        for r in tags_arr {
                            if let Some(s) = r.as_str() {
                                if let Some(arch) = parse_role(s) {
                                    resolved_arch = Some(arch);
                                    break;
                                }
                            }
                        }
                    }
                }

                let final_arch = resolved_arch.unwrap_or("CRIT_MARKSMAN");

                if let Some(id) = h.get("id").and_then(|v| v.as_str()) {
                    if !id.is_empty() {
                        map.insert(id.to_string(), final_arch);
                    }
                }
                if let Some(id_num) = h.get("id").and_then(|v| v.as_i64()) {
                    map.insert(id_num.to_string(), final_arch);
                }
                if let Some(name) = h.get("name").and_then(|v| v.as_str()) {
                    map.insert(name.to_string(), final_arch);
                    map.insert(name.to_lowercase(), final_arch);
                }
                if let Some(slug) = h.get("seoSlug").and_then(|v| v.as_str()) {
                    map.insert(slug.to_string(), final_arch);
                    map.insert(slug.to_lowercase(), final_arch);
                }
                if let Some(search_terms) = h.get("searchTerms").and_then(|v| v.as_array()) {
                    for t in search_terms {
                        if let Some(s) = t.as_str() {
                            map.insert(s.to_string(), final_arch);
                            map.insert(s.to_lowercase(), final_arch);
                        }
                    }
                }
            }
        }
        map
    };
}

pub fn augment_matches_favored_tag(name: &str, desc: &str, fav_tag: &str) -> bool {
    if name.contains(fav_tag) || desc.contains(fav_tag) {
        return true;
    }
    let keywords: &[&str] = match fav_tag {
        "AP" | "AP_BURST" | "MAGIC_PEN" | "SPELL_CRIT_SYNERGY" | "ABILITY_POWER" => {
            &["法术", "法强", "AP", "魔法", "巫师", "法术强度", "法穿", "法术穿透", "珠光"]
        }
        "MANA" | "MANA_REGEN" => &["法力", "法力值", "回蓝", "气定神闲"],
        "DOT_BURN" | "PERCENT_MAX_HP" | "SLOW_CC" => {
            &["灼烧", "流血", "最大生命值", "减速", "定身", "点燃"]
        }
        "CRIT" | "INFINITY" => &["暴击", "无尽", "暴伤", "暴击率", "暴击伤害"],
        "ATTACK_SPEED" | "DOUBLE_STRIKE" | "ON_HIT" | "PERCENT_CURRENT_HP" | "BUFF_ON_HIT" => {
            &["攻速", "攻击速度", "特效", "攻击特效", "双重", "当前生命值"]
        }
        "AD" | "ARMOR_PEN" | "LETHALITY" | "EXECUTE" | "ARMOR_SHRED" | "SPELL_BLADE" => {
            &["攻击力", "穿甲", "破甲", "物理", "斩杀", "咒刃", "耀光", "物理伤害"]
        }
        "ARMOR" | "MAGIC_RESIST" | "HEAVY_HP" | "HP_STACK" | "IMMOLATE_BURN" | "MR" => {
            &["护甲", "魔抗", "魔法抗性", "生命值", "坦度", "减伤", "最大生命", "献祭", "日炎", "巨像"]
        }
        "ABILITY_HASTE" | "BUFF_AP_HASTE" => &["技能急速", "急速", "冷却缩减", "冷却", "CD"],
        "HEAL_SHIELD_BOOST" | "CHAIN_HEAL" | "GLOBAL_HEAL" | "LIFELINE_SHIELD" => {
            &["护盾", "治疗", "救赎", "生命回复", "吸血", "全能吸血", "护盾值"]
        }
        _ => &[],
    };

    keywords.iter().any(|&kw| name.contains(kw) || desc.contains(kw))
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
    pub fn get_hero_archetype_info(hero_id: &str) -> ArchetypeInfo {
        Self::detect_archetype(hero_id, &[], &[], &[], false)
    }

    pub fn resolve_default_archetype_for_hero(hero_id: &str) -> &'static str {
        let hid = hero_id.trim();
        if hid.is_empty() {
            return "CRIT_MARKSMAN";
        }

        // 1. Check famous multi-archetype heroes (explicit overrides)
        if let Some(multi_hero) = ENGINE_META.famous_multi_archetype_heroes.get(hid) {
            if let Some(def) = multi_hero.get("default").and_then(|v| v.as_str()) {
                return def;
            }
        }

        // 2. Check full hero role mapping table from heroes_summary
        if let Some(&arch) = HERO_DEFAULT_ARCHETYPE_MAP.get(hid) {
            return arch;
        }
        let clean = hid.replace([' ', '.', '\'', '_'], "").to_lowercase();
        if let Some(&arch) = HERO_DEFAULT_ARCHETYPE_MAP.get(&clean) {
            return arch;
        }

        // 3. Check tactical traits
        if let Some(v) = ENGINE_META.champion_tactical_traits.get(hid) {
            if let Some(traits) = v.get("traits").and_then(|t| t.as_array()) {
                for t in traits {
                    if let Some(s) = t.as_str() {
                        match s {
                            "SUPER_TANK" => return "PURE_TANK",
                            "ENCHANTER" => return "ENCHANTER_SUPPORT",
                            "BURST_ASSASSIN" => return "LETHALITY_ASSASSIN",
                            "BRUISER" => return "BRUISER_FIGHTER",
                            "AP_BURST" => return "AP_POKE_BURST",
                            "AP_DOT" => return "AP_DOT_CONTROL",
                            "MARKSMAN" => return "CRIT_MARKSMAN",
                            _ => {}
                        }
                    }
                }
            }
        }

        // 4. Keyword heuristics as fallback
        if clean.contains("tank") || clean.contains("坦") {
            "PURE_TANK"
        } else if clean.contains("mage") || clean.contains("法") || clean.contains("ap") {
            "AP_POKE_BURST"
        } else if clean.contains("assassin") || clean.contains("刺") {
            "LETHALITY_ASSASSIN"
        } else if clean.contains("support") || clean.contains("辅") {
            "ENCHANTER_SUPPORT"
        } else if clean.contains("fighter") || clean.contains("战") || clean.contains("bruiser") {
            "BRUISER_FIGHTER"
        } else {
            "CRIT_MARKSMAN"
        }
    }

    pub fn detect_archetype(
        hero_id: &str,
        owned_items: &[String],
        active_augments: &[Value],
        candidate_items: &[Value],
        _resolve_conflicts: bool,
    ) -> ArchetypeInfo {
        let hid = hero_id.trim();
        let default_arch_id = Self::resolve_default_archetype_for_hero(hid);
        let multi_hero = ENGINE_META.famous_multi_archetype_heroes.get(hid);

        // 1. Check if user already owns major items
        let mut valid_owned = Vec::new();
        for i in owned_items {
            let s = i.trim();
            if !s.is_empty() && s != "0" && s != "None" {
                valid_owned.push(s.to_string());
            }
        }

        // Multi-archetype champion triggers (e.g. Kai'Sa, Varus, Kog'Maw, Sion, Vi, Lee Sin)
        if !valid_owned.is_empty() {
            if let Some(cfg) = multi_hero {
                let trigger_mappings: &[(&str, &str)] = &[
                    ("ap_trigger_items", "AP_POKE_BURST"),
                    ("crit_trigger_items", "CRIT_MARKSMAN"),
                    ("on_hit_trigger_items", "ON_HIT_ATTACK_SPEED"),
                    ("lethality_trigger_items", "LETHALITY_ASSASSIN"),
                    ("tank_trigger_items", "PURE_TANK"),
                    ("bruiser_trigger_items", "BRUISER_FIGHTER"),
                ];

                for &(key, target_arch) in trigger_mappings {
                    if let Some(arr) = cfg.get(key).and_then(|v| v.as_array()) {
                        for it in arr {
                            if let Some(s) = it.as_str() {
                                if valid_owned.iter().any(|o| o == s) {
                                    if let Some(arch) = ENGINE_META.archetypes.get(target_arch) {
                                        return ArchetypeInfo {
                                            locked_archetype_id: target_arch.to_string(),
                                            name: arch.name.clone(),
                                            badge: arch.badge.clone(),
                                            description: arch.description.clone(),
                                            is_locked: true,
                                            lock_reason: format!("检测到多流派英雄出装核心触发件[{}]", s),
                                            favored_tags: arch.favored_tags.clone(),
                                            forbidden_tags: arch.forbidden_tags.clone(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Check signature items matching with deterministic ordering
            let mut sorted_archetypes: Vec<(&String, &ArchetypeDefinition)> = ENGINE_META.archetypes.iter().collect();
            sorted_archetypes.sort_by_key(|(id, _)| id.as_str());

            for (arch_id, arch) in sorted_archetypes {
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
        let mut sorted_archetype_defs: Vec<(&String, &ArchetypeDefinition)> = ENGINE_META.archetypes.iter().collect();
        sorted_archetype_defs.sort_by_key(|(id, _)| id.as_str());

        for (arch_id, arch) in &sorted_archetype_defs {
            let mut score = 0;
            // Match candidate items
            for c in candidate_items.iter().take(8) {
                let iid = c.get("itemId")
                    .or_else(|| c.get("id"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();
                if let Some(tags) = ENGINE_META.item_feature_tags.get(&iid) {
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
            // Match active augments with Chinese keywords and English tags
            for a in active_augments {
                let aug_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let aug_desc = a.get("description").and_then(|v| v.as_str()).unwrap_or("");
                for fav in &arch.favored_tags {
                    if augment_matches_favored_tag(aug_name, aug_desc, fav) {
                        score += 5;
                    }
                }
            }
            arch_scores.insert(arch_id.as_str(), score);
        }

        let mut sorted_scores: Vec<(&str, i32)> = arch_scores.into_iter().collect();
        sorted_scores.sort_by(|(id_a, score_a), (id_b, score_b)| {
            score_b.cmp(score_a)
                .then_with(|| (id_a == &default_arch_id).cmp(&(id_b == &default_arch_id)).reverse())
                .then_with(|| id_a.cmp(id_b))
        });

        let arch_id = if let Some(&(best_id, best_score)) = sorted_scores.first() {
            if best_score > 0 {
                best_id
            } else {
                default_arch_id
            }
        } else {
            default_arch_id
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
            if !score.is_finite() {
                score = 50.0;
            }

            if has_forbidden && !is_counter_exempt {
                if resolve_conflicts && archetype_info.is_locked {
                    score *= (1.0 - strictness).max(0.1);
                    c["isArchetypeConflict"] = Value::Bool(true);
                }
            } else if tags.iter().any(|t| favored.contains(t.as_str())) {
                score *= 1.25;
                c["isArchetypeFavored"] = Value::Bool(true);
            }

            if !score.is_finite() {
                score = 50.0;
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
                let wr = trio.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let wr = if wr.is_finite() { wr } else { 0.0 };
                let games = trio.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
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

                if trio_mode == "STRICT_TRIO" && games > 0 {
                    // In strict trio mode, sample size and pure winrate carry extra weight
                    let smoothed_wr = (wr * games as f64 + 0.5 * 50.0) / (games as f64 + 50.0);
                    score += (smoothed_wr - 0.5) * 40.0;
                }

                if !score.is_finite() {
                    score = 0.0;
                }

                let mut enriched = trio.clone();
                enriched["matchScore"] = Value::from((score * 10.0).round() / 10.0);
                enriched["ownedMatches"] = Value::from(owned_match_count);

                (score, enriched)
            })
            .collect();

        scored_trios.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let ka = a.1.get("trioKey").and_then(|v| v.as_str()).unwrap_or("");
                    let kb = b.1.get("trioKey").and_then(|v| v.as_str()).unwrap_or("");
                    ka.cmp(kb)
                })
        });
        scored_trios.into_iter().next().map(|(_, val)| val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_detect_archetype_kaisa_default_and_crit_trigger() {
        // Kai'Sa (145) defaults to AP_POKE_BURST when no items owned
        let info = ArchetypeEngine::get_hero_archetype_info("145");
        assert_eq!(info.locked_archetype_id, "AP_POKE_BURST");
        assert!(!info.is_locked);

        // Kai'Sa with Infinity Edge (3031) locks to CRIT_MARKSMAN
        let crit_info = ArchetypeEngine::detect_archetype("145", &["3031".to_string()], &[], &[], false);
        assert_eq!(crit_info.locked_archetype_id, "CRIT_MARKSMAN");
        assert!(crit_info.is_locked);
    }

    #[test]
    fn test_detect_archetype_sion_default() {
        // Sion (14) defaults to PURE_TANK
        let info = ArchetypeEngine::get_hero_archetype_info("14");
        assert_eq!(info.locked_archetype_id, "PURE_TANK");
    }

    #[test]
    fn test_detect_archetype_lee_sin_default() {
        // Lee Sin (64) defaults to BRUISER_FIGHTER
        let info = ArchetypeEngine::get_hero_archetype_info("64");
        assert_eq!(info.locked_archetype_id, "BRUISER_FIGHTER");
    }

    #[test]
    fn test_detect_archetype_deterministic_empty_candidates() {
        let a1 = ArchetypeEngine::detect_archetype("9999", &[], &[], &[], false);
        let a2 = ArchetypeEngine::detect_archetype("9999", &[], &[], &[], false);
        assert_eq!(a1.locked_archetype_id, a2.locked_archetype_id);
    }

    #[test]
    fn test_resolve_default_archetype_for_heroes_from_classes() {
        // Annie (1) - Mage -> AP_POKE_BURST
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("1"), "AP_POKE_BURST");
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("Annie"), "AP_POKE_BURST");
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("黑暗之女"), "AP_POKE_BURST");

        // Leona (89) - Tank -> PURE_TANK
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("89"), "PURE_TANK");
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("Leona"), "PURE_TANK");

        // Zed (238) - Assassin -> LETHALITY_ASSASSIN
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("238"), "LETHALITY_ASSASSIN");
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("Zed"), "LETHALITY_ASSASSIN");

        // Soraka (16) - Support -> ENCHANTER_SUPPORT
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("16"), "ENCHANTER_SUPPORT");

        // Garen (86) - Fighter -> BRUISER_FIGHTER
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("86"), "BRUISER_FIGHTER");

        // Ashe (22) - Marksman -> CRIT_MARKSMAN
        assert_eq!(ArchetypeEngine::resolve_default_archetype_for_hero("22"), "CRIT_MARKSMAN");
    }

    #[test]
    fn test_detect_archetype_augment_chinese_tag_scoring() {
        // An active augment with Chinese name and description matching AP favored tags
        let augments = vec![json!({
            "name": "珠光莲华",
            "description": "你的技能可以造成暴击，并且获得大量法术强度与法术穿透。"
        })];

        // Annie with no items, but has "珠光莲华"
        let info = ArchetypeEngine::detect_archetype("1", &[], &augments, &[], false);
        assert_eq!(info.locked_archetype_id, "AP_POKE_BURST");

        // Tank augment: "巨像的勇气" matching armor and tank tags
        let tank_augments = vec![json!({
            "name": "巨像的勇气",
            "description": "获得大量护甲和最大生命值与减伤效果"
        })];
        let tank_info = ArchetypeEngine::detect_archetype("89", &[], &tank_augments, &[], false);
        assert_eq!(tank_info.locked_archetype_id, "PURE_TANK");
    }

    #[test]
    fn test_detect_archetype_deterministic_tie_breaking_prioritizes_hero_default() {
        // A generic hero with default "BRUISER_FIGHTER"
        // Even when scores tie, the hero's default archetype is prioritized
        let info = ArchetypeEngine::detect_archetype("86", &[], &[], &[], false);
        assert_eq!(info.locked_archetype_id, "BRUISER_FIGHTER");
    }
}

