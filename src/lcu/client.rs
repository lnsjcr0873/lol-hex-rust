use super::connector::{LcuConnector, LcuCredentials};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone)]
pub struct LruStore<K: std::hash::Hash + Eq + Clone, V: Clone> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> LruStore<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(val) = self.map.get(key) {
            let val = val.clone();
            if let Some(pos) = self.order.iter().position(|x| x == key) {
                self.order.remove(pos);
            }
            self.order.push_back(key.clone());
            Some(val)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            self.map.insert(key.clone(), value);
            if let Some(pos) = self.order.iter().position(|x| x == &key) {
                self.order.remove(pos);
            }
            self.order.push_back(key);
            return;
        }

        while self.map.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.map.remove(&oldest);
            } else {
                break;
            }
        }

        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}

#[derive(Clone)]
pub struct LcuClient {
    client: reqwest::Client,
    creds: Arc<RwLock<Option<LcuCredentials>>>,
    pub current_phase: Arc<RwLock<String>>,
    pub current_session: Arc<RwLock<Value>>,
    consecutive_failures: Arc<AtomicU32>,
    pub match_detail_cache: Arc<std::sync::Mutex<LruStore<u64, Value>>>,
    pub match_timeline_cache: Arc<std::sync::Mutex<LruStore<u64, Value>>>,
    pub concurrency_limiter: Arc<tokio::sync::Semaphore>,
    in_flight_details: Arc<std::sync::Mutex<HashSet<u64>>>,
    in_flight_timelines: Arc<std::sync::Mutex<HashSet<u64>>>,
    cache_notify: Arc<tokio::sync::Notify>,
    negative_match_details: Arc<std::sync::Mutex<HashMap<u64, std::time::Instant>>>,
    negative_match_timelines: Arc<std::sync::Mutex<HashMap<u64, std::time::Instant>>>,
    opgg: Arc<RwLock<Option<Arc<crate::crawler::OpggClient>>>>,
}

struct InFlightGuard {
    set: Arc<std::sync::Mutex<HashSet<u64>>>,
    key: u64,
    notify: Arc<tokio::sync::Notify>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = self.set.lock() {
            set.remove(&self.key);
        }
        self.notify.notify_waiters();
    }
}

impl Default for LcuClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Percent-encodes a path segment (such as chat room ID) in compliance with RFC 3986
fn percent_encode_path_segment(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{:02X}", b);
            }
        }
    }
    encoded
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
            match_detail_cache: Arc::new(std::sync::Mutex::new(LruStore::new(256))),
            match_timeline_cache: Arc::new(std::sync::Mutex::new(LruStore::new(256))),
            concurrency_limiter: Arc::new(tokio::sync::Semaphore::new(4)),
            in_flight_details: Arc::new(std::sync::Mutex::new(HashSet::new())),
            in_flight_timelines: Arc::new(std::sync::Mutex::new(HashSet::new())),
            cache_notify: Arc::new(tokio::sync::Notify::new()),
            negative_match_details: Arc::new(std::sync::Mutex::new(HashMap::new())),
            negative_match_timelines: Arc::new(std::sync::Mutex::new(HashMap::new())),
            opgg: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_opgg(&self, opgg: Arc<crate::crawler::OpggClient>) {
        let mut g = self.opgg.write().await;
        *g = Some(opgg);
    }

    pub async fn get_opgg(&self) -> Option<Arc<crate::crawler::OpggClient>> {
        let g = self.opgg.read().await;
        g.clone()
    }

    pub fn get_cached_match_detail(&self, game_id: u64) -> Option<Value> {
        self.match_detail_cache.lock().ok()?.get(&game_id)
    }

    pub fn insert_cached_match_detail(&self, game_id: u64, val: Value) {
        if let Ok(mut cache) = self.match_detail_cache.lock() {
            cache.insert(game_id, val);
        }
        self.cache_notify.notify_waiters();
    }

    pub fn is_negative_cached_detail(&self, game_id: u64) -> bool {
        if let Ok(mut map) = self.negative_match_details.lock() {
            if let Some(timestamp) = map.get(&game_id) {
                if timestamp.elapsed() < std::time::Duration::from_secs(30) {
                    return true;
                } else {
                    map.remove(&game_id);
                }
            }
        }
        false
    }

    pub fn insert_negative_cached_detail(&self, game_id: u64) {
        if let Ok(mut map) = self.negative_match_details.lock() {
            map.insert(game_id, std::time::Instant::now());
        }
        self.cache_notify.notify_waiters();
    }

    pub fn get_cached_match_timeline(&self, game_id: u64) -> Option<Value> {
        self.match_timeline_cache.lock().ok()?.get(&game_id)
    }

    pub fn insert_cached_match_timeline(&self, game_id: u64, val: Value) {
        if let Ok(mut cache) = self.match_timeline_cache.lock() {
            cache.insert(game_id, val);
        }
        self.cache_notify.notify_waiters();
    }

    pub fn is_negative_cached_timeline(&self, game_id: u64) -> bool {
        if let Ok(mut map) = self.negative_match_timelines.lock() {
            if let Some(timestamp) = map.get(&game_id) {
                if timestamp.elapsed() < std::time::Duration::from_secs(30) {
                    return true;
                } else {
                    map.remove(&game_id);
                }
            }
        }
        false
    }

    pub fn insert_negative_cached_timeline(&self, game_id: u64) {
        if let Ok(mut map) = self.negative_match_timelines.lock() {
            map.insert(game_id, std::time::Instant::now());
        }
        self.cache_notify.notify_waiters();
    }

    pub fn clear_lru_caches(&self) {
        if let Ok(mut c) = self.match_detail_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.match_timeline_cache.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.negative_match_details.lock() {
            c.clear();
        }
        if let Ok(mut c) = self.negative_match_timelines.lock() {
            c.clear();
        }
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let details = self.match_detail_cache.lock().map(|c| c.len()).unwrap_or(0);
        let timelines = self.match_timeline_cache.lock().map(|c| c.len()).unwrap_or(0);
        (details, timelines)
    }

    pub async fn ensure_credentials(&self) -> bool {
        {
            let guard = self.creds.read().await;
            if guard.is_some() {
                return true;
            }
        }
        if let Some(creds) = LcuConnector::sniff_credentials_async().await {
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
                } else if status.is_success() {
                    self.record_success();
                    Some(res)
                } else {
                    None
                }
            }
            Err(e) => {
                if e.is_connect() {
                    self.record_failure().await;
                }
                None
            }
        }
    }

    /// Handles an HTTP/HTTPS response candidate when an HTTP fallback will immediately follow.
    /// Invalidates credentials on 401 Unauthorized so that restarted/expired sessions trigger re-auth.
    /// Does not increment consecutive_failures on connection error, ensuring failures are only recorded
    /// if all fallback candidates fail.
    async fn handle_response_fallback_candidate(&self, res_result: Result<reqwest::Response, reqwest::Error>) -> Option<reqwest::Response> {
        match res_result {
            Ok(res) => {
                let status = res.status();
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    self.invalidate_credentials().await;
                    None
                } else if status.is_success() {
                    self.record_success();
                    Some(res)
                } else {
                    None
                }
            }
            Err(_) => None,
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

    pub async fn get_champion_mastery(&self, puuid: &str) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return None,
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return None,
        };

        let url = if puuid.is_empty() || puuid.starts_with("mock_") {
            format!("https://127.0.0.1:{}/lol-champion-mastery/v1/local-player/champion-mastery", port)
        } else {
            format!("https://127.0.0.1:{}/lol-champion-mastery/v1/{}/champion-mastery", port, puuid)
        };

        if let Some(res) = self.handle_response(self.client.get(&url).headers(headers).send().await).await {
            if let Ok(val) = res.json::<Value>().await {
                return Some(val);
            }
        }
        None
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

    pub async fn do_apply_runes_and_spells(
        &self,
        db: &crate::data::loader::Database,
        champion_id: i64,
        position: &str,
    ) -> (bool, String) {
        if !self.is_connected().await {
            return (false, "英雄联盟客户端未连接，无法应用符文与技能".to_string());
        }

        let cid_str = champion_id.to_string();
        let target_pos = match position.to_uppercase().as_str() {
            "TOP" => "top",
            "JUNGLE" => "jungle",
            "MID" | "MIDDLE" => "mid",
            "ADC" | "BOT" | "BOTTOM" => "adc",
            "SUPPORT" | "SUP" | "UTILITY" => "support",
            _ => "all",
        };

        let cached_opgg = self.get_opgg().await;
        let detail = if let Some(ref client) = cached_opgg {
            let res = client.get_champion_detail(&cid_str, target_pos).await;
            if res.is_none() && target_pos != "all" {
                client.get_champion_detail(&cid_str, "all").await
            } else {
                res
            }
        } else {
            let local_client = crate::crawler::OpggClient::new();
            let res = local_client.get_champion_detail(&cid_str, target_pos).await;
            if res.is_none() && target_pos != "all" {
                local_client.get_champion_detail(&cid_str, "all").await
            } else {
                res
            }
        };
        let mut perk_applied = false;
        let mut spells_applied = false;

        if let Some(detail) = detail {
            // 1. Apply recommended runes only when style IDs and non-empty perks are valid
            if let Some(runes) = detail.get("recommendedRunes").filter(|r| !r.is_null()) {
                let primary_style_id = runes.get("primaryStyleId").and_then(|v| v.as_i64()).unwrap_or(0);
                let sub_style_id = runes.get("subStyleId").and_then(|v| v.as_i64()).unwrap_or(0);
                let perk_ids_arr = runes.get("selectedPerkIds").and_then(|v| v.as_array());

                if primary_style_id > 0 && sub_style_id > 0 {
                    if let Some(arr) = perk_ids_arr {
                        if !arr.is_empty() {
                            let hero_name = db.resolve_hero_name(&cid_str).unwrap_or_else(|| format!("英雄{}", champion_id));
                            let payload = json!({
                                "name": format!("HexBrain: {}", hero_name),
                                "primaryStyleId": primary_style_id,
                                "subStyleId": sub_style_id,
                                "selectedPerkIds": arr,
                                "current": true
                            });
                            perk_applied = self.apply_perk_page(&payload).await;
                        }
                    }
                }
            }

            // 2. Apply summoner spells only when real spell IDs are provided
            if let Some(spells) = detail.get("summonerSpells").and_then(|s| s.as_array()) {
                if let Some(first_pair) = spells.first().and_then(|p| p.as_array()) {
                    if first_pair.len() >= 2 {
                        if let (Some(s1), Some(mut s2)) = (first_pair[0].as_i64(), first_pair[1].as_i64()) {
                            let is_jungle = position.eq_ignore_ascii_case("JUNGLE") || target_pos == "jungle";
                            // Protect Smite (spell 11) for junglers:
                            if is_jungle && s1 != 11 && s2 != 11 {
                                s2 = 11;
                            }
                            if s1 > 0 && s2 > 0 {
                                spells_applied = self.apply_summoner_spells(s1, s2).await;
                            }
                        }
                    }
                }
            }
        }

        if perk_applied || spells_applied {
            (true, "推荐符文与召唤师技能已自动应用至客户端".to_string())
        } else {
            (false, "未获取到该英雄权威推荐或未能写入客户端，请检查是否处于英雄选择阶段".to_string())
        }
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

        let encoded_room_id = percent_encode_path_segment(chat_room_id);
        let url = format!(
            "https://127.0.0.1:{}/lol-chat/v1/conversations/{}/participants",
            port, encoded_room_id
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

    async fn fetch_match_history_batch(
        &self,
        port: u16,
        headers: &HeaderMap,
        puuid: &str,
        beg_index: usize,
        end_index: usize,
    ) -> Option<Value> {
        let https_url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/products/lol/{}/matches?begIndex={}&endIndex={}",
            port, puuid, beg_index, end_index
        );
        let mut any_connect_err = false;
        let https_res = self.client.get(&https_url).headers(headers.clone()).send().await;
        if let Err(ref e) = https_res {
            if e.is_connect() {
                any_connect_err = true;
            }
        }
        if let Some(res) = self.handle_response_fallback_candidate(https_res).await {
            if let Ok(val) = res.json::<Value>().await {
                if val.get("games").is_some() || val.get("matches").is_some() {
                    return Some(val);
                }
            }
        }
        // HTTP fallback for test harness / local mock
        let http_url = format!(
            "http://127.0.0.1:{}/lol-match-history/v1/products/lol/{}/matches?begIndex={}&endIndex={}",
            port, puuid, beg_index, end_index
        );
        let mut http_connect_err = false;
        let http_res = self.client.get(&http_url).headers(headers.clone()).send().await;
        if let Err(ref e) = http_res {
            if e.is_connect() {
                http_connect_err = true;
            }
        }
        if let Some(res) = self.handle_response_fallback_candidate(http_res).await {
            if let Ok(val) = res.json::<Value>().await {
                if val.get("games").is_some() || val.get("matches").is_some() {
                    return Some(val);
                }
            }
        }
        // Fallback for current summoner endpoint if puuid is "current" / "me" / empty OR matches current summoner's PUUID:
        let is_current = if puuid == "current" || puuid == "me" || puuid.is_empty() {
            true
        } else if let Some(cur) = self.get_current_summoner().await {
            cur.get("puuid").and_then(|v| v.as_str()).map(|p| p.eq_ignore_ascii_case(puuid)).unwrap_or(false)
        } else {
            false
        };

        if is_current {
            let cur_https = format!(
                "https://127.0.0.1:{}/lol-match-history/v1/products/lol/current-summoner/matches?begIndex={}&endIndex={}",
                port, beg_index, end_index
            );
            if let Some(res) = self.handle_response_fallback_candidate(self.client.get(&cur_https).headers(headers.clone()).send().await).await {
                if let Ok(val) = res.json::<Value>().await {
                    if val.get("games").is_some() || val.get("matches").is_some() {
                        return Some(val);
                    }
                }
            }
            let cur_http = format!(
                "http://127.0.0.1:{}/lol-match-history/v1/products/lol/current-summoner/matches?begIndex={}&endIndex={}",
                port, beg_index, end_index
            );
            if let Some(res) = self.handle_response(self.client.get(&cur_http).headers(headers.clone()).send().await).await {
                if let Ok(val) = res.json::<Value>().await {
                    if val.get("games").is_some() || val.get("matches").is_some() {
                        return Some(val);
                    }
                }
            }
        } else if any_connect_err && http_connect_err {
            self.record_failure().await;
        }
        None
    }

    pub async fn get_raw_match_history(
        &self,
        puuid: &str,
        beg_index: usize,
        end_index: usize,
    ) -> Option<Value> {
        if !self.ensure_credentials().await || puuid.is_empty() {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let (beg_index, end_index) = (beg_index, end_index.max(beg_index));
        let total_requested = end_index.saturating_add(1).saturating_sub(beg_index).clamp(1, 200);
        let effective_end = beg_index.saturating_add(total_requested).saturating_sub(1).min(end_index);

        // If requested 20 or fewer matches, single request is sufficient
        if total_requested <= 20 {
            let _permit = self.concurrency_limiter.acquire().await.ok()?;
            return self.fetch_match_history_batch(port, &headers, puuid, beg_index, effective_end).await;
        }

        // For > 20 matches (e.g. 30, 50, 100, 200), chunk in batches of 20 (LCU native limit per page)
        let mut all_games = Vec::new();
        let mut cur_beg = beg_index;
        let mut base_obj: Option<Value> = None;

        while cur_beg <= effective_end && all_games.len() < total_requested {
            let cur_end = cur_beg.saturating_add(19).min(effective_end);
            let _permit = self.concurrency_limiter.acquire().await.ok()?;

            let chunk_val = self.fetch_match_history_batch(port, &headers, puuid, cur_beg, cur_end).await;

            if let Some(val) = chunk_val {
                if base_obj.is_none() {
                    base_obj = Some(val.clone());
                }
                let mut found_count = 0;
                if let Some(games) = val.get("games")
                    .and_then(|g| g.get("games").and_then(|arr| arr.as_array()).or_else(|| g.as_array()))
                    .or_else(|| val.get("matches").and_then(|m| m.as_array()))
                {
                    found_count = games.len();
                    for g in games {
                        all_games.push(g.clone());
                        if all_games.len() >= total_requested {
                            break;
                        }
                    }
                }

                let batch_expected = cur_end.saturating_add(1).saturating_sub(cur_beg);
                if found_count == 0 || found_count < batch_expected {
                    // Reached end of available match history on client
                    break;
                }
            } else {
                break;
            }

            match cur_beg.checked_add(20) {
                Some(next) => cur_beg = next,
                None => break,
            }
        }

        if let Some(mut base) = base_obj {
            let total_games_len = all_games.len();
            if let Some(g_map) = base.get_mut("games") {
                if let Some(inner_games) = g_map.get_mut("games") {
                    *inner_games = Value::Array(all_games);
                } else {
                    *g_map = json!({ "games": all_games });
                }
                if let Some(obj) = g_map.as_object_mut() {
                    obj.insert("gameCount".to_string(), json!(total_games_len));
                    if total_games_len > 0 {
                        obj.insert(
                            "gameIndexEnd".to_string(),
                            json!(beg_index.saturating_add(total_games_len).saturating_sub(1)),
                        );
                    }
                }
            } else {
                base["games"] = json!({
                    "games": all_games,
                    "gameCount": total_games_len,
                    "gameIndexBegin": beg_index,
                    "gameIndexEnd": if total_games_len > 0 { beg_index.saturating_add(total_games_len).saturating_sub(1) } else { beg_index }
                });
            }
            Some(base)
        } else {
            None
        }
    }

    pub async fn get_player_match_history(
        &self,
        puuid: &str,
        beg_index: usize,
        end_index: usize,
    ) -> Option<Value> {
        self.get_raw_match_history(puuid, beg_index, end_index).await
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

        let parsed_id = game_id.parse::<u64>().ok();
        if let Some(gid) = parsed_id {
            if self.is_negative_cached_detail(gid) {
                return None;
            }
            if let Some(cached) = self.get_cached_match_detail(gid) {
                return Some(cached);
            }
        }

        let _in_flight_guard = if let Some(gid) = parsed_id {
            loop {
                if self.is_negative_cached_detail(gid) {
                    return None;
                }
                if let Some(cached) = self.get_cached_match_detail(gid) {
                    return Some(cached);
                }
                let is_in_flight = {
                    let mut set = self.in_flight_details.lock().ok()?;
                    if set.contains(&gid) {
                        true
                    } else {
                        set.insert(gid);
                        false
                    }
                };
                if is_in_flight {
                    let notified = self.cache_notify.notified();
                    if self.is_negative_cached_detail(gid) {
                        return None;
                    }
                    if let Some(cached) = self.get_cached_match_detail(gid) {
                        return Some(cached);
                    }
                    let _ = tokio::time::timeout(std::time::Duration::from_millis(3500), notified).await;
                } else {
                    break Some(InFlightGuard {
                        set: self.in_flight_details.clone(),
                        key: gid,
                        notify: self.cache_notify.clone(),
                    });
                }
            }
        } else {
            None
        };

        let port = match self.get_port().await {
            Some(p) => p,
            None => {
                if let Some(gid) = parsed_id {
                    self.insert_negative_cached_detail(gid);
                }
                return None;
            }
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => {
                if let Some(gid) = parsed_id {
                    self.insert_negative_cached_detail(gid);
                }
                return None;
            }
        };

        let _permit = match self.concurrency_limiter.acquire().await {
            Ok(p) => p,
            Err(_) => {
                if let Some(gid) = parsed_id {
                    self.insert_negative_cached_detail(gid);
                }
                return None;
            }
        };

        if let Some(gid) = parsed_id {
            if self.is_negative_cached_detail(gid) {
                return None;
            }
            if let Some(cached) = self.get_cached_match_detail(gid) {
                return Some(cached);
            }
        }

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/games/{}",
            port, game_id
        );
        let res_opt = match self.handle_response_fallback_candidate(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!(
                    "http://127.0.0.1:{}/lol-match-history/v1/games/{}",
                    port, game_id
                );
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        };

        let res = match res_opt {
            Some(r) => r,
            None => {
                if let Some(gid) = parsed_id {
                    self.insert_negative_cached_detail(gid);
                }
                return None;
            }
        };
        let val = match res.json::<Value>().await {
            Ok(v) => v,
            Err(_) => {
                if let Some(gid) = parsed_id {
                    self.insert_negative_cached_detail(gid);
                }
                return None;
            }
        };
        if let Some(gid) = parsed_id {
            self.insert_cached_match_detail(gid, val.clone());
        }
        Some(val)
    }

    pub async fn get_raw_match_detail(&self, game_id: u64) -> Option<Value> {
        self.get_match_detail(&game_id.to_string()).await
    }

    pub async fn get_raw_match_timeline(&self, game_id: u64) -> Option<Value> {
        if !self.ensure_credentials().await || game_id == 0 {
            return None;
        }

        if self.is_negative_cached_timeline(game_id) {
            return None;
        }

        if let Some(cached) = self.get_cached_match_timeline(game_id) {
            return Some(cached);
        }

        let _in_flight_guard = loop {
            if self.is_negative_cached_timeline(game_id) {
                return None;
            }
            if let Some(cached) = self.get_cached_match_timeline(game_id) {
                return Some(cached);
            }
            let is_in_flight = {
                let mut set = self.in_flight_timelines.lock().ok()?;
                if set.contains(&game_id) {
                    true
                } else {
                    set.insert(game_id);
                    false
                }
            };
            if is_in_flight {
                let notified = self.cache_notify.notified();
                if self.is_negative_cached_timeline(game_id) {
                    return None;
                }
                if let Some(cached) = self.get_cached_match_timeline(game_id) {
                    return Some(cached);
                }
                let _ = tokio::time::timeout(std::time::Duration::from_millis(3500), notified).await;
            } else {
                break InFlightGuard {
                    set: self.in_flight_timelines.clone(),
                    key: game_id,
                    notify: self.cache_notify.clone(),
                };
            }
        };

        let port = match self.get_port().await {
            Some(p) => p,
            None => {
                self.insert_negative_cached_timeline(game_id);
                return None;
            }
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => {
                self.insert_negative_cached_timeline(game_id);
                return None;
            }
        };

        let _permit = match self.concurrency_limiter.acquire().await {
            Ok(p) => p,
            Err(_) => {
                self.insert_negative_cached_timeline(game_id);
                return None;
            }
        };

        if self.is_negative_cached_timeline(game_id) {
            return None;
        }
        if let Some(cached) = self.get_cached_match_timeline(game_id) {
            return Some(cached);
        }

        let url = format!(
            "https://127.0.0.1:{}/lol-match-history/v1/game-timelines/{}",
            port, game_id
        );
        let res_opt = match self.handle_response_fallback_candidate(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!(
                    "http://127.0.0.1:{}/lol-match-history/v1/game-timelines/{}",
                    port, game_id
                );
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        };
        let res = match res_opt {
            Some(r) => r,
            None => {
                self.insert_negative_cached_timeline(game_id);
                return None;
            }
        };
        let val = match res.json::<Value>().await {
            Ok(v) => v,
            Err(_) => {
                self.insert_negative_cached_timeline(game_id);
                return None;
            }
        };
        self.insert_cached_match_timeline(game_id, val.clone());
        Some(val)
    }

    pub async fn get_game_timeline(&self, game_id: u64) -> Option<Value> {
        self.get_raw_match_timeline(game_id).await
    }

    pub async fn get_match_timeline(&self, game_id: u64) -> Option<Value> {
        self.get_raw_match_timeline(game_id).await
    }

    pub async fn get_entitlements_token(&self) -> Option<Value> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/entitlements/v1/token", port);
        let res = match self.handle_response_fallback_candidate(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!("http://127.0.0.1:{}/entitlements/v1/token", port);
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        }?;
        res.json::<Value>().await.ok()
    }

    pub async fn get_league_session_token(&self) -> Option<String> {
        if !self.ensure_credentials().await {
            return None;
        }
        let port = self.get_port().await?;
        let headers = self.get_auth_header().await?;

        let url = format!("https://127.0.0.1:{}/lol-league-session/v1/league-session-token", port);
        let res = match self.handle_response_fallback_candidate(self.client.get(&url).headers(headers.clone()).send().await).await {
            Some(r) => Some(r),
            None => {
                let http_url = format!("http://127.0.0.1:{}/lol-league-session/v1/league-session-token", port);
                self.handle_response(self.client.get(&http_url).headers(headers).send().await).await
            }
        }?;
        let text = res.text().await.ok()?;
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed == "null" {
            return None;
        }

        // Handle JSON object response (e.g. {"token":"..."} or {"leagueSessionToken":"..."})
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                if let Some(tok) = val.get("token")
                    .or_else(|| val.get("leagueSessionToken"))
                    .or_else(|| val.get("sessionToken"))
                    .or_else(|| val.get("jwt"))
                    .or_else(|| val.get("accessToken"))
                    .and_then(|v| v.as_str())
                {
                    let t = tok.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }

        let token = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };
        if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        }
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

        let encoded_room_id = percent_encode_path_segment(chat_room_id);
        let url = format!(
            "https://127.0.0.1:{}/lol-chat/v1/conversations/{}/messages",
            port, encoded_room_id
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
            "https://127.0.0.1:{}/lol-summoner/v1/summoners",
            port
        );
        let req = self.client.get(&url).headers(headers.clone()).query(&[("name", query_name)]);
        if let Some(res) = self.handle_response(req.send().await).await {
            if let Ok(val) = res.json::<Value>().await {
                if !val.is_null() && (val.get("puuid").is_some() || val.get("displayName").is_some() || val.get("gameName").is_some()) {
                    return Some(val);
                }
            }
        }

        // Fallback to HTTP
        let url_http = format!(
            "http://127.0.0.1:{}/lol-summoner/v1/summoners",
            port
        );
        let req_http = self.client.get(&url_http).headers(headers).query(&[("name", query_name)]);
        if let Some(res) = self.handle_response(req_http.send().await).await {
            if let Ok(val) = res.json::<Value>().await {
                if !val.is_null() && (val.get("puuid").is_some() || val.get("displayName").is_some() || val.get("gameName").is_some()) {
                    return Some(val);
                }
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

        let url_v2 = format!(
            "https://127.0.0.1:{}/lol-summoner/v2/summoners/puuid/{}",
            port, puuid
        );
        if let Some(res) = self.handle_response(self.client.get(&url_v2).headers(headers.clone()).send().await).await {
            if let Ok(val) = res.json::<Value>().await {
                if !val.is_null() && (val.get("gameName").is_some() || val.get("displayName").is_some()) {
                    return Some(val);
                }
            }
        }

        let url_v1 = format!(
            "https://127.0.0.1:{}/lol-summoner/v1/summoners/by-puuid/{}",
            port, puuid
        );
        let res = self.handle_response(self.client.get(&url_v1).headers(headers).send().await).await?;
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

    /// Retrieves region code and platform ID from LCU endpoints (e.g. "TENCENT", Some("HN1")).
    pub async fn get_region_and_platform(&self) -> (String, Option<String>) {
        if !self.ensure_credentials().await {
            return (String::new(), None);
        }
        let port = match self.get_port().await {
            Some(p) => p,
            None => return (String::new(), None),
        };
        let headers = match self.get_auth_header().await {
            Some(h) => h,
            None => return (String::new(), None),
        };

        let mut region = String::new();
        let mut platform_id = None;

        // 1. /riotclient/region-locale
        let url_rl = format!("https://127.0.0.1:{}/riotclient/region-locale", port);
        if let Some(res) = self.handle_response_fallback_candidate(self.client.get(&url_rl).headers(headers.clone()).send().await).await {
            if let Ok(v) = res.json::<Value>().await {
                if let Some(r) = v.get("region").and_then(|s| s.as_str()) {
                    region = r.to_string();
                }
            }
        }

        // 2. /lol-platform-config/v1/namespaces/LoginDataPacket/platformId
        let url_pid = format!("https://127.0.0.1:{}/lol-platform-config/v1/namespaces/LoginDataPacket/platformId", port);
        if let Some(res) = self.handle_response_fallback_candidate(self.client.get(&url_pid).headers(headers.clone()).send().await).await {
            if let Ok(t) = res.text().await {
                let trimmed = t.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    platform_id = Some(trimmed.to_string());
                }
            }
        }

        // 3. Fallback: inspect entitlements token subject (e.g. "user@HN1")
        if platform_id.is_none() {
            if let Some(ent) = self.get_entitlements_token().await {
                if let Some(sub) = ent.get("subject").and_then(|s| s.as_str()) {
                    if let Some(pos) = sub.find('@') {
                        let potential_pid = &sub[pos + 1..];
                        if !potential_pid.is_empty() {
                            platform_id = Some(potential_pid.to_uppercase());
                        }
                    }
                }
            }
        }

        (region, platform_id)
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

    #[test]
    fn test_percent_encode_path_segment() {
        assert_eq!(percent_encode_path_segment("simple"), "simple");
        assert_eq!(percent_encode_path_segment("room#123"), "room%23123");
        assert_eq!(percent_encode_path_segment("room?key=val"), "room%3Fkey%3Dval");
        assert_eq!(percent_encode_path_segment("hello world"), "hello%20world");
        assert_eq!(
            percent_encode_path_segment("champ-select-123@champ-select.lol.riotgames.com"),
            "champ-select-123%40champ-select.lol.riotgames.com"
        );
    }

    #[tokio::test]
    async fn test_negative_caching_detail_and_timeline() {
        let client = LcuClient::new();
        assert!(!client.is_negative_cached_detail(12345));
        assert!(!client.is_negative_cached_timeline(12345));

        client.insert_negative_cached_detail(12345);
        client.insert_negative_cached_timeline(12345);

        assert!(client.is_negative_cached_detail(12345));
        assert!(client.is_negative_cached_timeline(12345));

        // When cleared, negative caches are emptied
        client.clear_lru_caches();
        assert!(!client.is_negative_cached_detail(12345));
        assert!(!client.is_negative_cached_timeline(12345));
    }

    #[tokio::test]
    async fn test_fallback_candidate_unauthorized_invalidates() {
        let app = axum::Router::new().route("/test-401", axum::routing::get(|| async {
            axum::http::StatusCode::UNAUTHORIZED
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = LcuClient::new();
        client.set_credentials(port, "token").await;
        assert!(client.is_connected().await);

        let url = format!("http://127.0.0.1:{}/test-401", port);
        let res = client.client.get(&url).send().await;
        let candidate_res = client.handle_response_fallback_candidate(res).await;
        assert!(candidate_res.is_none());
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_fetch_match_history_batch_connection_failure_records_failure() {
        let client = LcuClient::new();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let dead_port = listener.local_addr().unwrap().port();
        drop(listener);

        client.set_credentials(dead_port, "dummy_token").await;
        assert_eq!(client.get_consecutive_failures(), 0);

        let headers = client.get_auth_header().await.unwrap();
        let result = client.fetch_match_history_batch(dead_port, &headers, "some-other-puuid", 0, 10).await;
        assert!(result.is_none());
        assert!(client.get_consecutive_failures() > 0);
    }
}

