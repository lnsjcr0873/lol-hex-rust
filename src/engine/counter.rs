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

pub struct CounterItemEngine;

impl CounterItemEngine {
    pub fn find_tactical_traits(cname: &str) -> Option<&'static Value> {
        let trimmed = cname.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(v) = ENGINE_META.champion_tactical_traits.get(trimmed) {
            return Some(v);
        }
        let clean = trimmed.replace([' ', '.', '\'', '_'], "").to_lowercase();
        for (k, v) in &ENGINE_META.champion_tactical_traits {
            if k.to_lowercase() == clean || k.replace([' ', '.', '\'', '_'], "").to_lowercase() == clean {
                return Some(v);
            }
            if let Some(n) = v.get("name").and_then(|v| v.as_str()) {
                if n == trimmed || n.contains(trimmed) || trimmed.contains(n) {
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
                .or_else(|| e.get("name"))
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
                                    healer_names.push(disp_name);
                                }
                                "SUPER_TANK" => {
                                    profile.has_super_tank = true;
                                    tank_names.push(disp_name);
                                }
                                "SHIELD_HEAVY" => {
                                    profile.has_shield_heavy = true;
                                    shield_names.push(disp_name);
                                }
                                "HIGH_CC" => profile.has_high_cc = true,
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
        _arch: &ArchetypeInfo,
    ) -> HashSet<String> {
        let mut urgent = HashSet::new();

        if threat.has_high_heal {
            // Anti-heal items (Morello 3165, Mortal Reminder 3033, Chempunk Chainsword 6609, Thornmail 3075)
            for id in ["3165", "3033", "6609", "3075"] {
                urgent.insert(id.to_string());
            }
        }

        if threat.has_shield_heavy {
            // Serpent's Fang (6695 巨蛇之牙)
            urgent.insert("6695".to_string());
        }

        if threat.has_super_tank {
            // Anti-tank penetration (Void Staff 3135, LDR 3036, Liandry 3151, BotRK 3153)
            for id in ["3135", "3036", "3151", "3153"] {
                urgent.insert(id.to_string());
            }
        }

        if threat.has_high_cc {
            // Anti-CC (Mercurial 3139, Banshee 3102, Edge of Night 3814)
            for id in ["3139", "3102", "3814"] {
                urgent.insert(id.to_string());
            }
        }

        urgent
    }

    pub fn apply_counter_recommendations(
        candidate_items: &[Value],
        threat: &ThreatProfile,
        _arch: &ArchetypeInfo,
        _owned: &HashSet<String>,
        intensity: f64,
        _resolve_conflicts: bool,
    ) -> Vec<Value> {
        let urgent = Self::get_urgent_counter_ids(threat, _arch);
        let mut out = Vec::new();

        for it in candidate_items {
            let mut c = it.clone();
            let iid = c.get("itemId").and_then(|v| v.as_str()).unwrap_or("");

            if urgent.contains(iid) {
                let current_score = c.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(50.0);
                let boosted = current_score + (15.0 * intensity);
                c["tacticalScore"] = Value::from((boosted * 10.0).round() / 10.0);
                c["isCounterItem"] = Value::Bool(true);
                c["counterReason"] = Value::String(threat.summary.clone());
            }

            out.push(c);
        }

        out
    }
}
