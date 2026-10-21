use serde::{Deserialize, Serialize};

/// Current timestamp in epoch milliseconds.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub const SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS: usize = 20;
pub const SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH: usize = 100;

/// Normalizes a single tag phrase: trims whitespace and caps at 100 Unicode scalar values.
pub fn normalize_saved_player_tag_phrase(phrase: &str) -> String {
    let trimmed = phrase.trim();
    if trimmed.chars().count() > SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH {
        trimmed.chars().take(SAVED_PLAYER_TAG_PHRASE_MAX_LENGTH).collect()
    } else {
        trimmed.to_string()
    }
}

/// Normalizes a list of preset phrases: trims, deduplicates, bounds items <= 20, and excludes empty phrases.
pub fn normalize_saved_player_tag_phrases(phrases: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS.min(phrases.len()));
    let mut seen = std::collections::HashSet::new();

    for p in phrases {
        let normalized = normalize_saved_player_tag_phrase(p);
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            result.push(normalized);
            if result.len() >= SAVED_PLAYER_TAG_PHRASE_MAX_ITEMS {
                break;
            }
        }
    }

    result
}

// ==========================================
// Encountered Games Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredGameRecord {
    pub id: i64,
    pub game_id: i64,
    pub puuid: String,
    pub self_puuid: String,
    pub region: String,
    pub rso_platform_id: String,
    pub update_at: i64,
    pub queue_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredGameSaveDto {
    pub game_id: i64,
    pub puuid: String,
    pub self_puuid: String,
    pub region: String,
    pub rso_platform_id: String,
    pub queue_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredGamesQueryDto {
    pub puuid: String,
    pub self_puuid: String,
    pub queue_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub time_order: Option<String>, // "asc" or "desc" (default "desc")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedEncounteredGames {
    pub data: Vec<EncounteredGameRecord>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u32,
}

// ==========================================
// Saved Player Tags Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedPlayerTagRecord {
    pub puuid: String,
    pub self_puuid: String,
    pub region: String,
    pub rso_platform_id: String,
    pub tag: Option<String>,
    pub update_at: i64,
    pub last_met_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedPlayerTagItem {
    pub puuid: String,
    pub self_puuid: String,
    pub region: String,
    pub rso_platform_id: String,
    pub tag: String,
    pub update_at: i64,
    pub last_met_at: Option<i64>,
    pub marked_by_self: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerTagDto {
    pub puuid: String,
    pub self_puuid: String,
    pub region: Option<String>,
    pub rso_platform_id: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryAllPlayerTagsDto {
    pub puuid: Option<String>,
    pub self_puuid: Option<String>,
    pub region: Option<String>,
    pub rso_platform_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub time_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedPlayerTags {
    pub data: Vec<SavedPlayerTagRecord>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPlayerWithGamesResult {
    pub player: Option<SavedPlayerTagRecord>,
    pub tags: Vec<SavedPlayerTagItem>,
    pub encountered_games: PaginatedEncounteredGames,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericResult {
    pub success: bool,
    pub message: Option<String>,
}
