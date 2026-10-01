use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrNum {
        Str(String),
        Num(i64),
        Float(f64),
    }

    match StrOrNum::deserialize(deserializer)? {
        StrOrNum::Str(s) => Ok(s),
        StrOrNum::Num(n) => Ok(n.to_string()),
        StrOrNum::Float(f) => Ok(f.to_string()),
    }
}

pub fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Num(f64),
        Int(i64),
        Str(String),
        Null,
    }

    match NumOrStr::deserialize(deserializer)? {
        NumOrStr::Num(f) => Ok(f),
        NumOrStr::Int(i) => Ok(i as f64),
        NumOrStr::Str(s) => Ok(s.parse::<f64>().unwrap_or(0.0)),
        NumOrStr::Null => Ok(0.0),
    }
}

pub fn deserialize_i64_lenient<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Int(i64),
        Num(f64),
        Str(String),
        Null,
    }

    match NumOrStr::deserialize(deserializer)? {
        NumOrStr::Int(i) => Ok(i),
        NumOrStr::Num(f) => Ok(f as i64),
        NumOrStr::Str(s) => Ok(s.parse::<i64>().unwrap_or(0)),
        NumOrStr::Null => Ok(0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MetaInfo {
    #[serde(default)]
    pub report_patch: String,
    #[serde(default)]
    pub report_date: String,
    #[serde(default)]
    pub sample_scope: SampleScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SampleScope {
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub snapshot_matches: i64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub eligible_matches: i64,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HeroSummary {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub role_tags: Vec<String>,
    #[serde(default)]
    pub tier: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub win_rate: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub pick_rate: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub ban_rate: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub games: i64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub kda: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_cs: f64,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub top_augments: Vec<Value>,
    #[serde(default)]
    pub search_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AugmentSummary {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub rarity: String,
    #[serde(default)]
    pub tier: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub win_rate: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub pick_rate: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub games: i64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub wins: i64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub hex_score: f64,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cdn_icon_url: String,
    #[serde(default)]
    pub seo_slug: Option<String>,
    #[serde(default)]
    pub avg_delta_win_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchInfo {
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub kda: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_kills: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_deaths: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_assists: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_damage: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub damage_share: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_damage_taken: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_dmg_mitigated: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_gold: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_cs: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub kill_participation: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_turret_dmg: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub avg_cc_time: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub penta_kills: i64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub quadra_kills: i64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub triple_kills: i64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub double_kills: i64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub survivability: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub gold_efficiency: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub games: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HeroDetail {
    #[serde(default)]
    pub hero_id: Value,
    #[serde(default)]
    pub hero_name: String,
    #[serde(default)]
    pub augments: Value,
    #[serde(default)]
    pub items: Value,
    #[serde(default)]
    pub terminal_item_trios: Vec<Value>,
    #[serde(default)]
    pub trios: Vec<Value>,
    #[serde(default)]
    pub summoner_spell_pairs: Vec<Value>,
    #[serde(default)]
    pub strong_against: Vec<Value>,
    #[serde(default)]
    pub weak_against: Vec<Value>,
    #[serde(default)]
    pub teammate_synergies: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AugmentTrio {
    #[serde(default)]
    pub trio_key: String,
    #[serde(default)]
    pub augment_ids: Vec<Value>,
    #[serde(default)]
    pub augment_names: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub win_rate: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub pick_rate: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub games: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TerminalItemTrio {
    #[serde(default)]
    pub item_ids: Vec<Value>,
    #[serde(default)]
    pub item_names: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub win_rate: f64,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub pick_rate: f64,
    #[serde(default, deserialize_with = "deserialize_i64_lenient")]
    pub games: i64,
}
