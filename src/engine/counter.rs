use super::archetype::{ArchetypeInfo, ENGINE_META};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThreatProfile {
    pub has_high_heal: bool,
    pub has_shield_heavy: bool,
    pub has_super_tank: bool,
    pub has_high_cc: bool,
    pub physical_threat_count: usize,
    pub magic_threat_count: usize,
    pub summary: String,
}

pub const TAG_ABILITY_POWER: &str = "ABILITY_POWER";
pub const TAG_MAGIC_RESIST: &str = "MAGIC_RESIST";
pub const TAG_ARMOR: &str = "ARMOR";
pub const TAG_ATTACK_DAMAGE: &str = "AD";
pub const TAG_CRIT: &str = "CRIT";
pub const TAG_LETHALITY: &str = "LETHALITY";
pub const TAG_ATTACK_SPEED: &str = "ATTACK_SPEED";
pub const TAG_HEAVY_HP: &str = "HEAVY_HP";
pub const TAG_ABILITY_HASTE: &str = "ABILITY_HASTE";

pub struct CounterItemEngine;

impl CounterItemEngine {
    pub fn find_tactical_traits(cname: &str) -> Option<&'static Value> {
        let mut trimmed = cname.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(stripped) = trimmed.strip_prefix("game_character_displayname_") {
            trimmed = stripped;
        }
        if let Some(v) = ENGINE_META.champion_tactical_traits.get(trimmed) {
            return Some(v);
        }
        let clean = trimmed.replace([' ', '.', '\'', '_'], "").to_lowercase();
        for (k, v) in &ENGINE_META.champion_tactical_traits {
            let k_clean = k.replace([' ', '.', '\'', '_'], "").to_lowercase();
            if k.eq_ignore_ascii_case(trimmed) || k_clean == clean {
                return Some(v);
            }
            if let Some(n) = v.get("name").and_then(|v| v.as_str()) {
                if n == trimmed || n.eq_ignore_ascii_case(trimmed) {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn analyze_enemy_threats(enemy_team: &[Value]) -> ThreatProfile {
        let mut profile = ThreatProfile::default();
        if enemy_team.is_empty() {
            profile.summary = "敌方阵容未知，维持通用出装".to_string();
            return profile;
        }

        let mut healer_names = Vec::new();
        let mut tank_names = Vec::new();
        let mut shield_names = Vec::new();

        for e in enemy_team {
            let cname = e
                .get("championName")
                .or_else(|| e.get("champion_name"))
                .or_else(|| e.get("name"))
                .or_else(|| e.get("hero_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if let Some(trait_val) = Self::find_tactical_traits(cname) {
                let disp_name = trait_val.get("name").and_then(|v| v.as_str()).unwrap_or(cname);
                if let Some(traits) = trait_val.get("traits").and_then(|v| v.as_array()) {
                    for t in traits {
                        if let Some(t_str) = t.as_str() {
                            match t_str {
                                "HIGH_HEAL" => {
                                    profile.has_high_heal = true;
                                    if !healer_names.contains(&disp_name) {
                                        healer_names.push(disp_name);
                                    }
                                }
                                "SUPER_TANK" => {
                                    profile.has_super_tank = true;
                                    if !tank_names.contains(&disp_name) {
                                        tank_names.push(disp_name);
                                    }
                                }
                                "SHIELD_HEAVY" | "SHIELD" => {
                                    profile.has_shield_heavy = true;
                                    if !shield_names.contains(&disp_name) {
                                        shield_names.push(disp_name);
                                    }
                                }
                                "HIGH_CC" | "HARD_CC" => profile.has_high_cc = true,
                                _ => {}
                            }
                        }
                    }
                }
                if let Some(dmg) = trait_val.get("damage").and_then(|v| v.as_str()) {
                    if dmg.contains("PHYSICAL") {
                        profile.physical_threat_count += 1;
                    }
                    if dmg.contains("MAGIC") {
                        profile.magic_threat_count += 1;
                    }
                }
            }
        }

        let mut notes = Vec::new();
        if profile.has_high_heal {
            notes.push(format!("强回血({})需重伤", healer_names.join("/")));
        }
        if profile.has_shield_heavy {
            notes.push(format!("多重护盾({})需巨蛇破盾", shield_names.join("/")));
        }
        if profile.has_super_tank {
            notes.push(format!("重装前排({})需百分比穿透", tank_names.join("/")));
        }
        if profile.has_high_cc {
            notes.push("强控阵容需水银/护盾女妖应对".to_string());
        }
        if profile.physical_threat_count >= 3 {
            notes.push("高额物理伤害需护甲应对".to_string());
        }
        if profile.magic_threat_count >= 3 {
            notes.push("多法系爆发需魔抗应对".to_string());
        }

        if notes.is_empty() {
            profile.summary = "敌方阵容攻守均衡".to_string();
        } else {
            profile.summary = notes.join("; ");
        }

        profile
    }

    pub fn get_urgent_counter_ids(
        threat: &ThreatProfile,
        arch: &ArchetypeInfo,
    ) -> HashSet<String> {
        let is_ap_oriented = arch.favored_tags.iter().any(|t| {
            t == TAG_ABILITY_POWER || t == "AP" || t == "MAGIC" || t == "AP_BURST"
        }) || arch.forbidden_tags.iter().any(|t| t == TAG_ATTACK_DAMAGE || t == TAG_CRIT);

        let is_ad_oriented = arch.favored_tags.iter().any(|t| {
            t == TAG_ATTACK_DAMAGE || t == TAG_CRIT || t == TAG_LETHALITY || t == TAG_ATTACK_SPEED
        }) || arch.forbidden_tags.iter().any(|t| t == TAG_ABILITY_POWER || t == "AP");

        let is_tank_oriented = arch.favored_tags.iter().any(|t| {
            t == "TANK" || t == TAG_ARMOR || t == TAG_MAGIC_RESIST || t == "MR" || t == TAG_HEAVY_HP
        });

        let mut candidate_ids = Vec::new();

        if threat.has_high_heal {
            // Anti-heal items
            if is_ap_oriented {
                candidate_ids.push("3165"); // Morellonomicon
            }
            if is_ad_oriented {
                candidate_ids.push("3033"); // Mortal Reminder
                candidate_ids.push("6609"); // Chempunk Chainsword
            }
            if is_tank_oriented || (!is_ap_oriented && !is_ad_oriented) {
                candidate_ids.push("3075"); // Thornmail
            }
            if !is_ap_oriented && !is_ad_oriented && !is_tank_oriented {
                candidate_ids.extend(&["3165", "3033", "6609", "3075"]);
            }
        }

        if threat.has_shield_heavy {
            // Serpent's Fang (6695) is AD-lethality based
            if is_ad_oriented || (!is_ap_oriented && !is_tank_oriented) {
                candidate_ids.push("6695");
            }
        }

        if threat.has_super_tank {
            // Anti-tank penetration
            if is_ap_oriented {
                candidate_ids.push("3135"); // Void Staff
                candidate_ids.push("3151"); // Liandry's Torment
            }
            if is_ad_oriented {
                candidate_ids.push("3036"); // Lord Dominik's Regards
                candidate_ids.push("3153"); // Blade of the Ruined King
            }
            if !is_ap_oriented && !is_ad_oriented {
                candidate_ids.extend(&["3135", "3036", "3151", "3153"]);
            }
        }

        if threat.has_high_cc {
            // Anti-CC
            if is_ap_oriented {
                candidate_ids.push("3102"); // Banshee's Veil
            }
            if is_ad_oriented {
                candidate_ids.push("3139"); // Mercurial Scimitar
                candidate_ids.push("3814"); // Edge of Night
            }
            if !is_ap_oriented && !is_ad_oriented {
                candidate_ids.extend(&["3139", "3102", "3814"]);
            }
        }

        let mut urgent = HashSet::new();
        for id in candidate_ids {
            if let Some(item_tags) = ENGINE_META.item_feature_tags.get(id) {
                let has_forbidden = item_tags.iter().any(|tag| arch.forbidden_tags.contains(tag));
                if has_forbidden {
                    continue;
                }
            }
            urgent.insert(id.to_string());
        }

        urgent
    }

    pub fn apply_counter_recommendations(
        candidate_items: &[Value],
        threat: &ThreatProfile,
        arch: &ArchetypeInfo,
        _owned: &HashSet<String>,
        intensity: f64,
        _resolve_conflicts: bool,
    ) -> Vec<Value> {
        let urgent = Self::get_urgent_counter_ids(threat, arch);
        let mut out = Vec::new();

        for it in candidate_items {
            let mut c = it.clone();
            let iid = c.get("itemId")
                .or_else(|| c.get("id"))
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                .unwrap_or_default();

            if urgent.contains(&iid) {
                let current_score = c.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(50.0);
                let boosted = current_score + (15.0 * intensity);
                c["tacticalScore"] = Value::from((boosted * 10.0).round() / 10.0);
                c["isCounterItem"] = Value::Bool(true);
                c["counterReason"] = Value::String(threat.summary.clone());
            }

            out.push(c);
        }

        // Actively inject urgent counter items that are not already present in the candidate pool
        let mut sorted_urgent: Vec<String> = urgent.into_iter().collect();
        sorted_urgent.sort();

        for urgent_id in &sorted_urgent {
            let already_present = out.iter().any(|it| {
                let id = it.get("itemId")
                    .or_else(|| it.get("id"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();
                &id == urgent_id
            });

            if !already_present && !_owned.contains(urgent_id) {
                let item_name = crate::data::recipe_data::resolve_item_name(urgent_id);
                out.push(serde_json::json!({
                    "itemId": urgent_id,
                    "itemName": item_name,
                    "tacticalScore": ((60.0 + (15.0 * intensity)) * 10.0).round() / 10.0,
                    "isCounterItem": true,
                    "counterReason": threat.summary.clone(),
                    "isUrgentInjected": true
                }));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_analyze_enemy_threats_deduplication_and_name_fields() {
        let enemies = vec![
            json!({ "championName": "Soraka" }),
            json!({ "champion_name": "Soraka" }),
            json!({ "name": "Aatrox" }),
        ];

        let threat = CounterItemEngine::analyze_enemy_threats(&enemies);
        assert!(threat.has_high_heal);
        // "众星之子" and "暗裔剑魔" should each appear only once
        let heal_note = threat.summary;
        assert_eq!(heal_note.matches("众星之子").count(), 1);
        assert_eq!(heal_note.matches("暗裔剑魔").count(), 1);
    }

    #[test]
    fn test_apply_counter_recommendations_numeric_item_id() {
        let threat = ThreatProfile {
            has_high_heal: true,
            summary: "强回血需重伤".to_string(),
            ..Default::default()
        };

        let arch = ArchetypeInfo {
            favored_tags: vec!["AP".to_string()],
            forbidden_tags: vec!["AD".to_string()],
            ..Default::default()
        };

        // Item with numeric itemId: 3165 (Morellonomicon)
        let candidate_items = vec![
            json!({
                "itemId": 3165,
                "itemName": "莫雷洛秘典",
                "tacticalScore": 60.0
            }),
            json!({
                "id": "3089",
                "itemName": "灭世者的死亡之帽",
                "tacticalScore": 80.0
            })
        ];

        let out = CounterItemEngine::apply_counter_recommendations(
            &candidate_items,
            &threat,
            &arch,
            &HashSet::new(),
            1.0,
            true,
        );

        let morello = out.iter().find(|it| it["itemId"] == 3165).unwrap();
        assert_eq!(morello["isCounterItem"], true);
        assert_eq!(morello["tacticalScore"], 75.0); // 60.0 + 15.0
    }

    #[test]
    fn test_find_tactical_traits_vi_exact_matching_avoids_substring_false_positives() {
        // "Vi" should NOT match "Viktor", "Sivir", or "Anivia"
        // In champion_tactical_traits, "Vi" is not explicitly defined, so it must return None
        // rather than false-matching Viktor or another champion.
        let vi_trait = CounterItemEngine::find_tactical_traits("Vi");
        assert!(vi_trait.is_none(), "Short name 'Vi' must NOT substring-match Viktor or Sivir");

        // Exact match for Soraka works
        let soraka = CounterItemEngine::find_tactical_traits("Soraka");
        assert!(soraka.is_some());
        assert_eq!(soraka.unwrap().get("name").unwrap().as_str().unwrap(), "众星之子");
    }

    #[test]
    fn test_analyze_enemy_threats_shen_shield_trait() {
        let enemies = vec![json!({ "championName": "Shen" })];
        let threat = CounterItemEngine::analyze_enemy_threats(&enemies);
        assert!(threat.has_shield_heavy, "Shen's SHIELD trait must trigger has_shield_heavy");
        assert!(threat.summary.contains("多重护盾"));
    }

    #[test]
    fn test_apply_counter_recommendations_urgent_injection_when_missing() {
        let threat = ThreatProfile {
            has_shield_heavy: true,
            summary: "多重护盾需巨蛇破盾".to_string(),
            ..Default::default()
        };

        let arch = ArchetypeInfo {
            favored_tags: vec!["AD".to_string(), "LETHALITY".to_string()],
            forbidden_tags: vec!["AP".to_string()],
            ..Default::default()
        };

        // Candidates lack Serpent's Fang (6695)
        let candidate_items = vec![
            json!({
                "itemId": "3142", // Youmuu's
                "itemName": "幽梦之灵",
                "tacticalScore": 60.0
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

        // Serpent's Fang (6695) should be injected
        let serpent = out.iter().find(|it| {
            it.get("itemId").and_then(|v| v.as_str()) == Some("6695")
        });
        assert!(serpent.is_some(), "Serpent's Fang (6695) must be actively injected into candidates");
        let serpent = serpent.unwrap();
        assert_eq!(serpent["isCounterItem"], true);
        assert_eq!(serpent["isUrgentInjected"], true);
        assert_eq!(serpent["tacticalScore"], 75.0);
    }
}
