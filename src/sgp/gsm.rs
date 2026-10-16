use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GsmPlayerChampionSelection {
    pub puuid: String,
    #[serde(rename = "championId")]
    pub champion_id: i64,
    #[serde(rename = "selectedSkinIndex", default)]
    pub selected_skin_index: Option<i64>,
    #[serde(rename = "spell1Id")]
    pub spell1_id: i64,
    #[serde(rename = "spell2Id")]
    pub spell2_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GsmTeamMember {
    pub puuid: String,
    #[serde(rename = "summonerId", default)]
    pub summoner_id: Option<i64>,
    #[serde(rename = "teamParticipantId", default)]
    pub team_participant_id: Option<i64>,
    #[serde(rename = "championId", default)]
    pub champion_id: Option<i64>,
    #[serde(rename = "selectedRole", default)]
    pub selected_role: Option<String>,
    #[serde(rename = "selectedPosition", default)]
    pub selected_position: Option<String>,
    #[serde(rename = "summonerName", default)]
    pub summoner_name: Option<String>,
    #[serde(rename = "profileIconId", default)]
    pub profile_icon_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GsmGameInfo {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(rename = "gameState", default)]
    pub game_state: Option<String>,
    #[serde(rename = "gameMode", default)]
    pub game_mode: Option<String>,
    #[serde(rename = "queueTypeName", default)]
    pub queue_type_name: Option<String>,
    #[serde(rename = "teamOne", default)]
    pub team_one: Vec<GsmTeamMember>,
    #[serde(rename = "teamTwo", default)]
    pub team_two: Vec<GsmTeamMember>,
    #[serde(rename = "playerChampionSelections", default)]
    pub player_champion_selections: Vec<GsmPlayerChampionSelection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GsmLedgeRegionResponse {
    #[serde(default)]
    pub game: Option<GsmGameInfo>,
    #[serde(rename = "gameName", default)]
    pub game_name: Option<String>,
    #[serde(rename = "reconnectDelay", default)]
    pub reconnect_delay: Option<i64>,
}

/// Executes a GSM Live Query by PUUID.
pub async fn query_gsm_by_puuid(
    client: &reqwest::Client,
    common_base_url: &str,
    sub_id: &str,
    puuid: &str,
    session_token: &str,
) -> Result<GsmLedgeRegionResponse, String> {
    let p = puuid.trim();
    if p.is_empty() {
        return Err("GSM query: PUUID cannot be empty".to_string());
    }
    let sub = sub_id.trim();
    if sub.is_empty() {
        return Err("GSM query: sub_id cannot be empty".to_string());
    }
    let base = common_base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("GSM query: common_base_url cannot be empty".to_string());
    }

    let url = format!(
        "{}/gsm/v1/ledge/region/{}/puuid/{}",
        base,
        sub,
        p
    );

    let mut headers = HeaderMap::new();
    let bearer = format!("Bearer {}", session_token.trim());
    if let Ok(val) = HeaderValue::from_str(&bearer) {
        headers.insert(AUTHORIZATION, val);
    }

    let res = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("GSM query by PUUID failed: {}", e))?;

    let status = res.status();
    if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::NO_CONTENT {
        return Ok(GsmLedgeRegionResponse::default());
    }

    if !status.is_success() {
        return Err(format!("GSM query returned status {}", status));
    }

    let text = res
        .text()
        .await
        .map_err(|e| format!("Failed to read GSM response: {}", e))?;
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "null" || trimmed == "{}" {
        return Ok(GsmLedgeRegionResponse::default());
    }
    if trimmed.starts_with('<') {
        let snippet: String = trimmed.chars().take(120).collect();
        return Err(format!(
            "GSM query returned HTML error page instead of JSON: {}",
            snippet
        ));
    }

    serde_json::from_str::<GsmLedgeRegionResponse>(trimmed)
        .map_err(|e| format!("Failed to parse GSM response: {}", e))
}

/// Executes a GSM Live Query by Game ID.
pub async fn query_gsm_by_game_id(
    client: &reqwest::Client,
    common_base_url: &str,
    sub_id: &str,
    game_id: u64,
    session_token: &str,
) -> Result<Value, String> {
    if game_id == 0 {
        return Err("GSM query: game_id cannot be 0".to_string());
    }
    let sub = sub_id.trim();
    if sub.is_empty() {
        return Err("GSM query: sub_id cannot be empty".to_string());
    }
    let base = common_base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("GSM query: common_base_url cannot be empty".to_string());
    }

    let url = format!(
        "{}/gsm/v1/ledge/region/{}/gameId/{}",
        base,
        sub,
        game_id
    );

    let mut headers = HeaderMap::new();
    let bearer = format!("Bearer {}", session_token.trim());
    if let Ok(val) = HeaderValue::from_str(&bearer) {
        headers.insert(AUTHORIZATION, val);
    }

    let res = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("GSM query by Game ID failed: {}", e))?;

    let status = res.status();
    if status == reqwest::StatusCode::NOT_FOUND || status == reqwest::StatusCode::NO_CONTENT {
        return Ok(Value::Null);
    }

    if !status.is_success() {
        return Err(format!("GSM query returned status {}", status));
    }

    let text = res
        .text()
        .await
        .map_err(|e| format!("Failed to read GSM JSON: {}", e))?;
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Value::Null);
    }
    if trimmed.starts_with('<') {
        let snippet: String = trimmed.chars().take(120).collect();
        return Err(format!(
            "GSM query returned HTML error page instead of JSON: {}",
            snippet
        ));
    }

    serde_json::from_str::<Value>(trimmed)
        .map_err(|e| format!("Failed to parse GSM JSON: {}", e))
}
