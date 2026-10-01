use super::connector::{LcuConnector, LcuCredentials};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct LcuClient {
    client: reqwest::Client,
    creds: Arc<RwLock<Option<LcuCredentials>>>,
    pub current_phase: Arc<RwLock<String>>,
    pub current_session: Arc<RwLock<Value>>,
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
            .build()
            .unwrap_or_default();

        Self {
            client,
            creds: Arc::new(RwLock::new(None)),
            current_phase: Arc::new(RwLock::new("None".to_string())),
            current_session: Arc::new(RwLock::new(Value::Null)),
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
                    let phase = text.trim().trim_matches('"').to_string();
                    let mut p_guard = self.current_phase.write().await;
                    *p_guard = phase.clone();
                    return phase;
                }
            }
            _ => {
                // Invalidate credentials on failure
                let mut guard = self.creds.write().await;
                *guard = None;
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

        if let Some(pid) = editable_page_id {
            let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages/{}", port, pid);
            if let Ok(res) = self.client.put(&url).headers(headers.clone()).json(payload).send().await {
                if res.status().is_success() {
                    return true;
                }
            }
        }

        // Otherwise create new page
        let url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", port);
        match self.client.post(&url).headers(headers).json(payload).send().await {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }
}
