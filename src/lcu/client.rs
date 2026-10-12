use super::connector::{LcuConnector, LcuCredentials};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone)]
pub struct LcuClient {
    client: reqwest::Client,
    creds: Arc<RwLock<Option<LcuCredentials>>>,
    pub current_phase: Arc<RwLock<String>>,
    pub current_session: Arc<RwLock<Value>>,
    consecutive_failures: Arc<AtomicU32>,
}

impl Default for LcuClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LcuClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_millis(2500))
            .build()
            .unwrap_or_default();

        Self {
            client,
            creds: Arc::new(RwLock::new(None)),
            current_phase: Arc::new(RwLock::new("None".to_string())),
            current_session: Arc::new(RwLock::new(Value::Null)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn ensure_credentials(&self) -> bool {
        {
            let guard = self.creds.read().await;
            if guard.is_some() {
                return true;
            }
        }
        if let Some(creds) = LcuConnector::sniff_credentials() {
            let mut guard = self.creds.write().await;
            *guard = Some(creds);
            self.consecutive_failures.store(0, Ordering::Relaxed);
            return true;
        }
        false
    }

    pub async fn invalidate_credentials(&self) {
        let mut guard = self.creds.write().await;
        *guard = None;
        let mut p_guard = self.current_phase.write().await;
        *p_guard = "None".to_string();
        let mut s_guard = self.current_session.write().await;
        *s_guard = Value::Null;
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    pub async fn record_failure(&self) -> u32 {
        let fails = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        if fails >= 3 {
            let mut guard = self.creds.write().await;
            *guard = None;
            let mut p_guard = self.current_phase.write().await;
            *p_guard = "None".to_string();
            let mut s_guard = self.current_session.write().await;
            *s_guard = Value::Null;
        }
        fails
    }

    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    pub fn get_consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    pub async fn is_connected(&self) -> bool {
        self.creds.read().await.is_some()
    }

    pub async fn set_credentials(&self, port: u16, auth_token: &str) {
        let mut guard = self.creds.write().await;
        *guard = Some(LcuCredentials {
            port,
            auth_token: auth_token.to_string(),
        });
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    async fn handle_response(&self, res_result: Result<reqwest::Response, reqwest::Error>) -> Option<reqwest::Response> {
        match res_result {
            Ok(res) => {
                let status = res.status();
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    self.invalidate_credentials().await;
                    None
                } else if status.is_server_error() {
                    self.record_failure().await;
                    None
                } else if status.is_success() {
                    self.record_success();
                    Some(res)
                } else {
                    self.record_success();
                    None
                }
            }
            Err(_) => {
                self.record_failure().await;
                None
            }
        }
    }

    async fn get_auth_header(&self) -> Option<HeaderMap> {
        let guard = self.creds.read().await;
        let creds = guard.as_ref()?;
        let basic = format!("riot:{}", creds.auth_token);
        // Base64 encoding
        let encoded = {
            const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut result = String::new();
            let bytes = basic.as_bytes();
            for chunk in bytes.chunks(3) {
                let b0 = chunk[0] as usize;
                let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
                let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

                let n = (b0 << 16) | (b1 << 8) | b2;
                result.push(CHARSET[(n >> 18) & 63] as char);
                result.push(CHARSET[(n >> 12) & 63] as char);
                if chunk.len() > 1 {
                    result.push(CHARSET[(n >> 6) & 63] as char);
                } else {
                    result.push('=');
                }
                if chunk.len() > 2 {
                    result.push(CHARSET[n & 63] as char);
                } else {
                    result.push('=');
                }
            }
            result
        };

        let mut headers = HeaderMap::new();
        let auth_val = format!("Basic {}", encoded);
        if let Ok(val) = HeaderValue::from_str(&auth_val) {
            headers.insert(AUTHORIZATION, val);
        }
        Some(headers)
    }

    async fn get_port(&self) -> Option<u16> {
        let guard = self.creds.read().await;
        guard.as_ref().map(|c| c.port)
    }

    pub async fn get_gameflow_phase(&self) -> String {
        if !self.ensure_credentials().await {
            return "None".to_string();
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return "None".to_string(),
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return "None".to_string(),
        };

        let url = format!("https://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase", port);
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(text) = res.text().await {
                let phase = text.trim().trim_matches('"').to_string();
                let mut p_guard = self.current_phase.write().await;
                *p_guard = phase.clone();
                return phase;
            }
        }
        "None".to_string()
    }

    pub async fn get_champ_select_session(&self) -> Value {
        if !self.ensure_credentials().await {
            return Value::Null;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return Value::Null,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return Value::Null,
        };

        let url = format!("https://127.0.0.1:{}/lol-champ-select/v1/session", port);
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(data) = res.json::<Value>().await {
                let mut s_guard = self.current_session.write().await;
                *s_guard = data.clone();
                return data;
            }
        }
        Value::Null
    }

    pub async fn get_gameflow_session(&self) -> Value {
        if !self.ensure_credentials().await {
            return Value::Null;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return Value::Null,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return Value::Null,
        };

        let url = format!("https://127.0.0.1:{}/lol-gameflow/v1/session", port);
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(data) = res.json::<Value>().await {
                return data;
            }
        }
        Value::Null
    }

    pub async fn get_rso_access_token(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-rso-auth/v1/authorization/access-token", port);
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn swap_bench_champion(&self, champion_id: &str) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-champ-select/v1/session/bench/swap/{}",
            port, champion_id
        );
        self.handle_response(self.client.post(&url).headers(headers).send().await).await.is_some()
    }

    pub async fn reroll_champion(&self) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-champ-select/v1/session/my-selection/reroll",
            port
        );
        self.handle_response(self.client.post(&url).headers(headers).send().await).await.is_some()
    }

    pub async fn apply_summoner_spells(&self, spell1_id: i64, spell2_id: i64) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-champ-select/v1/session/my-selection",
            port
        );
        let payload = json!({
            "spell1Id": spell1_id,
            "spell2Id": spell2_id
        });

        self.handle_response(self.client.patch(&url).headers(headers).json(&payload).send().await).await.is_some()
    }

    pub async fn get_perk_pages(&self) -> Vec<Value> {
        if !self.ensure_credentials().await {
            return Vec::new();
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return Vec::new(),
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return Vec::new(),
        };

        let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", port);
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(pages) = res.json::<Vec<Value>>().await {
                return pages;
            }
        }
        Vec::new()
    }

    pub async fn apply_perk_page(&self, payload: &Value) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let pages = self.get_perk_pages().await;
        let mut editable_page_id = None;

        for p in &pages {
            if p.get("isEditable").and_then(|v| v.as_bool()).unwrap_or(false) {
                let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.contains("HexBrain") {
                    editable_page_id = p.get("id").and_then(|v| v.as_i64());
                    break;
                }
                if editable_page_id.is_none() {
                    editable_page_id = p.get("id").and_then(|v| v.as_i64());
                }
            }
        }

        let mut page_payload = payload.clone();
        if let Some(obj) = page_payload.as_object_mut() {
            obj.insert("isCurrent".to_string(), Value::Bool(true));
            obj.insert("current".to_string(), Value::Bool(true));
        }

        if let Some(pid) = editable_page_id {
            let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages/{}", port, pid);
            if self.handle_response(self.client.put(&url).headers(headers.clone()).json(&page_payload).send().await).await.is_some() {
                return true;
            }
        }

        // Otherwise create new page
        let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", port);
        self.handle_response(self.client.post(&url).headers(headers).json(&page_payload).send().await).await.is_some()
    }

    pub async fn get_chat_conversation_participants(&self, chat_room_id: &str) -> Vec<Value> {
        if !self.ensure_credentials().await || chat_room_id.is_empty() {
            return Vec::new();
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return Vec::new(),
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return Vec::new(),
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-chat/v1/conversations/{}/participants",
            port, chat_room_id
        );
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(list) = res.json::<Vec<Value>>().await {
                return list;
            }
        }
        Vec::new()
    }

    pub async fn get_player_ranked_stats(&self, puuid: &str) -> Option<Value> {
        if !self.ensure_credentials().await || puuid.is_empty() {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-ranked/v1/ranked-stats/{}",
            port, puuid
        );
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_player_match_history(&self, puuid: &str, end_idx: usize) -> Option<Value> {
        if !self.ensure_credentials().await || puuid.is_empty() {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
            port, puuid, end_idx
        );
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_current_summoner(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-summoner/v1/current-summoner", port);
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_chat_conversations(&self) -> Vec<Value> {
        if !self.ensure_credentials().await {
            return Vec::new();
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return Vec::new(),
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return Vec::new(),
        };

        let url = format!("https://127.0.0.1:{}/lol-chat/v1/conversations", port);
        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(list) = res.json::<Vec<Value>>().await {
                return list;
            }
        }
        Vec::new()
    }

    pub async fn get_conversations(&self) -> Vec<Value> {
        self.get_chat_conversations().await
    }

    pub async fn get_match_detail(&self, game_id: &str) -> Option<Value> {
        if !self.ensure_credentials().await || game_id.is_empty() {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/games/{}",
            port, game_id
        );
        let res = match self.handle_response(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!(
                    "http://127.0.0.1:{}/lol-match-history/v1/games/{}",
                    port, game_id
                );
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        }?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_game_timeline(&self, game_id: u64) -> Option<Value> {
        if !self.ensure_credentials().await || game_id == 0 {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/game-timelines/{}",
            port, game_id
        );
        let res = match self.handle_response(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!(
                    "http://127.0.0.1:{}/lol-match-history/v1/game-timelines/{}",
                    port, game_id
                );
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        }?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_match_timeline(&self, game_id: u64) -> Option<Value> {
        self.get_game_timeline(game_id).await
    }

    pub async fn send_chat_message(&self, chat_room_id: &str, message: &str) -> bool {
        if !self.ensure_credentials().await || chat_room_id.is_empty() || message.is_empty() {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-chat/v1/conversations/{}/messages",
            port, chat_room_id
        );
        let payload = json!({
            "body": message,
            "type": "chat"
        });
        self.handle_response(self.client.post(&url).headers(headers).json(&payload).send().await).await.is_some()
    }

    pub async fn get_summoner_by_name(&self, name: &str) -> Option<Value> {
        if !self.ensure_credentials().await || name.is_empty() {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let query_name = if let Some(pure_name) = name.split('#').next() {
            pure_name.trim()
        } else {
            name.trim()
        };

        let url = format!(
            "https://127.0.0.1:{}/lol-summoner/v1/summoners?name={}",
            port, query_name
        );
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_summoner_by_puuid(&self, puuid: &str) -> Option<Value> {
        if !self.ensure_credentials().await || puuid.is_empty() {
            return None;
        }
        if puuid == "me" || puuid == "current" {
            return self.get_current_summoner().await;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-summoner/v2/summoners/puuid/{}",
            port, puuid
        );
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_ready_check(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check", port);
        let res = self.handle_response(self.client.get(&url).headers(headers).send().await).await?;
        res.json::<Value>().await.ok()
    }

    pub async fn accept_ready_check(&self) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept", port);
        self.handle_response(self.client.post(&url).headers(headers).send().await).await.is_some()
    }

    pub async fn decline_ready_check(&self) -> bool {
        if !self.ensure_credentials().await {
            return false;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return false,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return false,
        };

        let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check/decline", port);
        self.handle_response(self.client.post(&url).headers(headers).send().await).await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consecutive_failure_threshold_invalidates_credentials() {
        let client = LcuClient::new();
        client.set_credentials(12345, "dummy_token").await;
        assert!(client.is_connected().await);
        assert_eq!(client.get_consecutive_failures(), 0);

        // Record failure 1
        assert_eq!(client.record_failure().await, 1);
        assert!(client.is_connected().await);

        // Record failure 2
        assert_eq!(client.record_failure().await, 2);
        assert!(client.is_connected().await);

        // Success resets counter
        client.record_success();
        assert_eq!(client.get_consecutive_failures(), 0);

        // 3 consecutive failures clear credentials
        client.record_failure().await;
        client.record_failure().await;
        client.record_failure().await;
        assert!(!client.is_connected().await);
        assert_eq!(*client.current_phase.read().await, "None");
    }

    #[tokio::test]
    async fn test_explicit_credential_invalidation() {
        let client = LcuClient::new();
        client.set_credentials(54321, "tok").await;
        *client.current_phase.write().await = "ChampSelect".to_string();
        *client.current_session.write().await = json!({"id": 1});
        assert!(client.is_connected().await);

        client.invalidate_credentials().await;
        assert!(!client.is_connected().await);
        assert_eq!(*client.current_phase.read().await, "None");
        assert_eq!(*client.current_session.read().await, Value::Null);
    }
}

