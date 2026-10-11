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
            return true;
        }
        false
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
        match self.client.get(&url).headers(headers).send().await {
            Ok(res) if res.status().is_success() => {
                if let Ok(text) = res.text().await {
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                    let phase = text.trim().trim_matches('"').to_string();
                    let mut p_guard = self.current_phase.write().await;
                    *p_guard = phase.clone();
                    return phase;
                }
            }
            _ => {
                let fails = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                if fails >= 3 {
                    let mut guard = self.creds.write().await;
                    *guard = None;
                }
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(data) = res.json::<Value>().await {
                    let mut s_guard = self.current_session.write().await;
                    *s_guard = data.clone();
                    return data;
                }
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(data) = res.json::<Value>().await {
                    return data;
                }
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
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
        match self.client.post(&url).headers(headers).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
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
        match self.client.post(&url).headers(headers).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
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

        match self.client.patch(&url).headers(headers).json(&payload).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(pages) = res.json::<Vec<Value>>().await {
                    return pages;
                }
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
            if let Ok(res) = self.client.put(&url).headers(headers.clone()).json(&page_payload).send().await {
                if res.status().is_success() {
                    return true;
                }
            }
        }

        // Otherwise create new page
        let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", port);
        match self.client.post(&url).headers(headers).json(&page_payload).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn get_chat_conversation_participants(&self, chat_room_id: &str) -> Vec<Value> {
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

        let url = format!(
            "https://127.0.0.1:{}/lol-chat/v1/conversations/{}/participants",
            port, chat_room_id
        );
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(list) = res.json::<Vec<Value>>().await {
                    return list;
                }
            }
        }
        Vec::new()
    }

    pub async fn get_player_ranked_stats(&self, puuid: &str) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-ranked/v1/ranked-stats/{}",
            port, puuid
        );
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
    }

    pub async fn get_player_match_history(&self, puuid: &str, end_idx: usize) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
            port, puuid, end_idx
        );
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
    }

    pub async fn get_current_summoner(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-summoner/v1/current-summoner", port);
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(list) = res.json::<Vec<Value>>().await {
                    return list;
                }
            }
        }
        Vec::new()
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
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
        match self.client.post(&url).headers(headers).json(&payload).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn get_conversations(&self) -> Vec<Value> {
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                if let Ok(convs) = res.json::<Vec<Value>>().await {
                    return convs;
                }
            }
        }
        Vec::new()
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
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
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
    }

    pub async fn get_ready_check(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check", port);
        if let Ok(res) = self.client.get(&url).headers(headers).send().await {
            if res.status().is_success() {
                return res.json::<Value>().await.ok();
            }
        }
        None
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
        match self.client.post(&url).headers(headers).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
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
        match self.client.post(&url).headers(headers).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }
}

