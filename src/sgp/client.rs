use super::gsm::{query_gsm_by_game_id, query_gsm_by_puuid, GsmLedgeRegionResponse};
use super::servers::{get_sgp_server_id, resolve_server_config, resolve_sub_id, SgpServerConfig};
use super::tokens::{extract_sgp_tokens, SgpTokens};
use crate::lcu::LcuClient;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct SgpClient {
    client: reqwest::Client,
    lcu: Option<Arc<LcuClient>>,
    tokens: Arc<RwLock<SgpTokens>>,
    server_id: Arc<RwLock<String>>,
    rso_platform_id: Arc<RwLock<Option<String>>>,
    server_config: Arc<RwLock<Option<SgpServerConfig>>>,
}

impl SgpClient {
    /// Creates a new SGP client attached to an LCU client.
    pub fn new(lcu: Arc<LcuClient>) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_millis(5000))
            .build()
            .unwrap_or_default();

        Self {
            client,
            lcu: Some(lcu),
            tokens: Arc::new(RwLock::new(SgpTokens::default())),
            server_id: Arc::new(RwLock::new(String::new())),
            rso_platform_id: Arc::new(RwLock::new(None)),
            server_config: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a standalone SGP client with explicit tokens and server configuration.
    pub fn with_credentials(server_id: &str, rso_platform_id: Option<&str>, tokens: SgpTokens) -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_millis(5000))
            .build()
            .unwrap_or_default();

        let cfg = resolve_server_config(server_id);

        Self {
            client,
            lcu: None,
            tokens: Arc::new(RwLock::new(tokens)),
            server_id: Arc::new(RwLock::new(server_id.to_string())),
            rso_platform_id: Arc::new(RwLock::new(rso_platform_id.map(|s| s.to_string()))),
            server_config: Arc::new(RwLock::new(cfg)),
        }
    }
}

impl Default for SgpClient {
    fn default() -> Self {
        Self::with_credentials("", None, SgpTokens::default())
    }
}

impl SgpClient {

    /// Configures the active server ID and resolves server configuration.
    pub async fn set_server(&self, region: &str, rso_platform_id: Option<&str>) {
        let id = get_sgp_server_id(region, rso_platform_id);
        let cfg = resolve_server_config(&id);

        let mut sid_guard = self.server_id.write().await;
        *sid_guard = id;

        let mut pid_guard = self.rso_platform_id.write().await;
        *pid_guard = rso_platform_id.map(|s| s.to_string());

        let mut cfg_guard = self.server_config.write().await;
        *cfg_guard = cfg;
    }

    /// Sets explicit SGP server configuration.
    pub async fn set_server_config(&self, config: SgpServerConfig) {
        let mut cfg_guard = self.server_config.write().await;
        *cfg_guard = Some(config);
    }

    /// Ensures SGP server configuration is populated, auto-discovering from LCU if unconfigured.
    pub async fn ensure_server_configured(&self) -> Result<(), String> {
        {
            let cfg_guard = self.server_config.read().await;
            if cfg_guard.is_some() {
                return Ok(());
            }
        }

        if let Some(ref lcu) = self.lcu {
            let (region, platform_id) = lcu.get_region_and_platform().await;
            let (eff_region, eff_platform) = match (region.trim(), platform_id.as_deref().map(|s| s.trim())) {
                ("", None) | ("", Some("")) => ("TENCENT", Some("HN1")),
                ("", Some(p)) => {
                    let p_upper = p.to_uppercase();
                    if p_upper.starts_with("HN")
                        || p_upper.starts_with("WT")
                        || p_upper.starts_with("BGP")
                        || p_upper.starts_with("EDU")
                        || p_upper.starts_with("TJ")
                        || p_upper.starts_with("NJ")
                        || p_upper.starts_with("GZ")
                        || p_upper.starts_with("CQ")
                    {
                        ("TENCENT", Some(p))
                    } else {
                        ("", Some(p))
                    }
                }
                (r, None) | (r, Some("")) => {
                    let r_upper = r.to_uppercase();
                    if r_upper == "TENCENT" {
                        ("TENCENT", Some("HN1"))
                    } else {
                        (r, None)
                    }
                }
                (r, Some(p)) => {
                    let r_upper = r.to_uppercase();
                    let p_upper = p.to_uppercase();
                    if r_upper == "TENCENT" && p_upper.is_empty() {
                        ("TENCENT", Some("HN1"))
                    } else {
                        (r, Some(p))
                    }
                }
            };
            self.set_server(eff_region, eff_platform).await;
            return Ok(());
        }

        Err("SGP server not configured".to_string())
    }

    /// Refreshes SGP credentials from LCU.
    pub async fn refresh_tokens(&self) -> Result<SgpTokens, String> {
        let lcu = self.lcu.as_ref().ok_or_else(|| "LCU client not configured".to_string())?;
        let tokens = extract_sgp_tokens(lcu).await;
        let mut guard = self.tokens.write().await;
        *guard = tokens.clone();
        Ok(tokens)
    }

    /// Gets current tokens.
    pub async fn get_tokens(&self) -> SgpTokens {
        self.tokens.read().await.clone()
    }

    /// Resolves the current sub-ID for URL placeholders.
    pub async fn current_sub_id(&self) -> String {
        let sid = self.server_id.read().await;
        let pid = self.rso_platform_id.read().await;
        let cfg = self.server_config.read().await;

        if let Some(ref c) = *cfg {
            resolve_sub_id(&sid, c, pid.as_deref())
        } else {
            sid.clone()
        }
    }

    /// Helper to execute an SGP HTTP GET with auto-refresh on 401/403.
    async fn request_with_auth_retry(&self, url: &str) -> Result<reqwest::Response, String> {
        let token_opt = {
            let tokens_guard = self.tokens.read().await;
            tokens_guard.access_token().map(|s| s.to_string())
        };

        let token = match token_opt {
            Some(t) => t,
            None => {
                if self.lcu.is_some() {
                    let refreshed = self.refresh_tokens().await?;
                    refreshed
                        .access_token()
                        .ok_or_else(|| "Entitlements token missing after refresh".to_string())?
                        .to_string()
                } else {
                    return Err("Entitlements token missing".to_string());
                }
            }
        };

        let make_headers = |tok: &str| -> HeaderMap {
            let mut headers = HeaderMap::new();
            let bearer = format!("Bearer {}", tok);
            if let Ok(val) = HeaderValue::from_str(&bearer) {
                headers.insert(AUTHORIZATION, val);
            }
            headers
        };

        let mut res = self
            .client
            .get(url)
            .headers(make_headers(&token))
            .send()
            .await
            .map_err(|e| format!("SGP HTTP request failed: {}", e))?;

        if (res.status() == reqwest::StatusCode::UNAUTHORIZED || res.status() == reqwest::StatusCode::FORBIDDEN)
            && self.lcu.is_some()
        {
            if let Ok(refreshed) = self.refresh_tokens().await {
                if let Some(new_tok) = refreshed.access_token() {
                    if let Ok(retry_res) = self.client.get(url).headers(make_headers(new_tok)).send().await {
                        res = retry_res;
                    }
                }
            }
        }

        Ok(res)
    }

    /// Helper to parse SGP responses, handling 404s, 204 No Content, HTML gateway errors, and invalid JSON.
    async fn parse_sgp_response(res: reqwest::Response, endpoint_name: &str) -> Result<Value, String> {
        let status = res.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("{} not found in SGP (404)", endpoint_name));
        }
        if status == reqwest::StatusCode::NO_CONTENT {
            return Ok(Value::Null);
        }
        if !status.is_success() {
            return Err(format!("SGP {} query returned status {}", endpoint_name, status));
        }

        let body = res
            .text()
            .await
            .map_err(|e| format!("Failed to read SGP {} response text: {}", endpoint_name, e))?;
        let trimmed = body.trim();
        if trimmed.is_empty() || trimmed == "null" {
            return Ok(Value::Null);
        }
        if trimmed.starts_with('<') {
            let snippet: String = trimmed.chars().take(120).collect();
            return Err(format!(
                "SGP {} returned HTML error page instead of JSON: {}",
                endpoint_name, snippet
            ));
        }

        serde_json::from_str::<Value>(trimmed)
            .map_err(|e| format!("Failed to parse SGP {} JSON: {}", endpoint_name, e))
    }

    /// Queries SGP Match History Summary for a player PUUID.
    pub async fn get_match_history_summary(
        &self,
        puuid: &str,
        start_index: usize,
        count: usize,
    ) -> Result<Value, String> {
        self.get_match_history_summary_with_tag(puuid, start_index, count, None).await
    }

    /// Queries SGP Match History Summary for a player PUUID with optional queue filter tag (e.g. "q_420", "q_450").
    pub async fn get_match_history_summary_with_tag(
        &self,
        puuid: &str,
        start_index: usize,
        count: usize,
        tag: Option<&str>,
    ) -> Result<Value, String> {
        let p = puuid.trim();
        if p.is_empty() {
            return Err("PUUID cannot be empty".to_string());
        }
        let count = count.clamp(1, 200);

        self.ensure_server_configured().await.ok();

        let base_url = {
            let cfg_guard = self.server_config.read().await;
            let cfg = cfg_guard.as_ref().ok_or_else(|| "SGP server not configured".to_string())?;
            cfg.match_history.clone()
        };

        let tag_param = match tag {
            Some(t) if !t.trim().is_empty() => format!("&tag={}", t.trim()),
            _ => String::new(),
        };

        let url = format!(
            "{}/match-history-query/v1/products/lol/player/{}/SUMMARY?startIndex={}&count={}{}",
            base_url.trim_end_matches('/'),
            p,
            start_index,
            count,
            tag_param
        );

        let res = self.request_with_auth_retry(&url).await?;
        Self::parse_sgp_response(res, "match history").await
    }

    /// Queries SGP Game Summary for a specific game ID.
    pub async fn get_game_summary(&self, game_id: u64) -> Result<Value, String> {
        if game_id == 0 {
            return Err("Invalid game_id: 0".to_string());
        }

        let sub_id = self.current_sub_id().await;
        let base_url = {
            let cfg_guard = self.server_config.read().await;
            let cfg = cfg_guard.as_ref().ok_or_else(|| "SGP server not configured".to_string())?;
            cfg.match_history.clone()
        };

        let url = format!(
            "{}/match-history-query/v1/products/lol/{}_{}/SUMMARY",
            base_url.trim_end_matches('/'),
            sub_id,
            game_id
        );

        let res = self.request_with_auth_retry(&url).await?;
        Self::parse_sgp_response(res, "game summary").await
    }

    /// Queries SGP Game Details (timeline) for a specific game ID.
    pub async fn get_game_details(&self, game_id: u64) -> Result<Value, String> {
        if game_id == 0 {
            return Err("Invalid game_id: 0".to_string());
        }

        let sub_id = self.current_sub_id().await;
        let base_url = {
            let cfg_guard = self.server_config.read().await;
            let cfg = cfg_guard.as_ref().ok_or_else(|| "SGP server not configured".to_string())?;
            cfg.match_history.clone()
        };

        let url = format!(
            "{}/match-history-query/v1/products/lol/{}_{}/DETAILS",
            base_url.trim_end_matches('/'),
            sub_id,
            game_id
        );

        let res = self.request_with_auth_retry(&url).await?;
        Self::parse_sgp_response(res, "game details").await
    }

    /// Queries GSM Live Game State for a player PUUID.
    pub async fn get_gsm_by_puuid(&self, puuid: &str) -> Result<GsmLedgeRegionResponse, String> {
        let p = puuid.trim();
        if p.is_empty() {
            return Err("PUUID cannot be empty".to_string());
        }

        self.ensure_server_configured().await.ok();

        let sub_id = self.current_sub_id().await;
        let (common_base, token_opt) = {
            let cfg_guard = self.server_config.read().await;
            let cfg = cfg_guard.as_ref().ok_or_else(|| "SGP server not configured".to_string())?;
            let tokens_guard = self.tokens.read().await;
            let token = tokens_guard.session_token().map(|s| s.to_string());
            (cfg.common.clone(), token)
        };

        let token = match token_opt {
            Some(t) => t,
            None => {
                if self.lcu.is_some() {
                    let refreshed = self.refresh_tokens().await?;
                    refreshed
                        .session_token()
                        .ok_or_else(|| "League session token missing after refresh".to_string())?
                        .to_string()
                } else {
                    return Err("League session token missing".to_string());
                }
            }
        };

        match query_gsm_by_puuid(&self.client, &common_base, &sub_id, p, &token).await {
            Ok(resp) => Ok(resp),
            Err(e) if (e.contains("401") || e.contains("403")) && self.lcu.is_some() => {
                if let Ok(refreshed) = self.refresh_tokens().await {
                    if let Some(new_token) = refreshed.session_token() {
                        return query_gsm_by_puuid(&self.client, &common_base, &sub_id, p, new_token).await;
                    }
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }

    /// Queries GSM Live Game State for a specific game ID.
    pub async fn get_gsm_by_game_id(&self, game_id: u64) -> Result<Value, String> {
        if game_id == 0 {
            return Err("Invalid game_id: 0".to_string());
        }

        self.ensure_server_configured().await.ok();

        let sub_id = self.current_sub_id().await;
        let (common_base, token_opt) = {
            let cfg_guard = self.server_config.read().await;
            let cfg = cfg_guard.as_ref().ok_or_else(|| "SGP server not configured".to_string())?;
            let tokens_guard = self.tokens.read().await;
            let token = tokens_guard.session_token().map(|s| s.to_string());
            (cfg.common.clone(), token)
        };

        let token = match token_opt {
            Some(t) => t,
            None => {
                if self.lcu.is_some() {
                    let refreshed = self.refresh_tokens().await?;
                    refreshed
                        .session_token()
                        .ok_or_else(|| "League session token missing after refresh".to_string())?
                        .to_string()
                } else {
                    return Err("League session token missing".to_string());
                }
            }
        };

        match query_gsm_by_game_id(&self.client, &common_base, &sub_id, game_id, &token).await {
            Ok(resp) => Ok(resp),
            Err(e) if (e.contains("401") || e.contains("403")) && self.lcu.is_some() => {
                if let Ok(refreshed) = self.refresh_tokens().await {
                    if let Some(new_token) = refreshed.session_token() {
                        return query_gsm_by_game_id(&self.client, &common_base, &sub_id, game_id, new_token).await;
                    }
                }
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_overseas_sgp_server_mapping() {
        let client = SgpClient::default();
        // Overseas NA platform
        client.set_server("NA", None).await;
        let cfg = client.server_config.read().await.clone();
        assert!(cfg.is_some());
        let cfg = cfg.unwrap();
        assert!(!cfg.is_tencent);
        assert!(cfg.match_history.contains("usw2-red") || cfg.match_history.contains("pvp.net"));

        // Overseas EUW platform
        client.set_server("EUW", Some("EUW1")).await;
        let cfg = client.server_config.read().await.clone().unwrap();
        assert!(!cfg.is_tencent);
        assert!(cfg.match_history.contains("euc1-red") || cfg.match_history.contains("pvp.net"));

        // Tencent server
        client.set_server("TENCENT", Some("HN1")).await;
        let cfg = client.server_config.read().await.clone().unwrap();
        assert!(cfg.is_tencent);
        assert!(cfg.match_history.contains("hn1-k8s-sgp"));
    }
}
