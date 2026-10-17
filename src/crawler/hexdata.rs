use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

pub struct HexdataCrawler {
    client: reqwest::Client,
    base_domain: String,
    cookie: Arc<RwLock<Option<String>>>,
    last_handshake: Arc<RwLock<Option<Instant>>>,
    handshake_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl Default for HexdataCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl HexdataCrawler {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));
        headers.insert(REFERER, HeaderValue::from_static("https://hexdata.com.cn/"));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
        headers.insert("x-hex-client", HeaderValue::from_static("true"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_domain: "https://hexdata.com.cn".to_string(),
            cookie: Arc::new(RwLock::new(None)),
            last_handshake: Arc::new(RwLock::new(None)),
            handshake_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Refresh token / session cookie every 30 minutes to satisfy HexData WAF
    pub async fn ensure_handshake(&self) {
        // 1. Fast path: check if cached handshake is still valid
        if let Ok(lock) = self.last_handshake.read() {
            if let Some(t) = *lock {
                if t.elapsed() < Duration::from_secs(1800) {
                    if let Ok(c_lock) = self.cookie.read() {
                        if c_lock.is_some() {
                            return;
                        }
                    }
                }
            }
        }

        // 2. Prevent TOCTOU race: acquire serialization lock
        let _guard = self.handshake_mutex.lock().await;

        // 3. Double-check under lock in case another request completed handshake
        if let Ok(lock) = self.last_handshake.read() {
            if let Some(t) = *lock {
                if t.elapsed() < Duration::from_secs(1800) {
                    if let Ok(c_lock) = self.cookie.read() {
                        if c_lock.is_some() {
                            return;
                        }
                    }
                }
            }
        }

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let check_url = format!("{}/?runtime-check={}", self.base_domain, now_ms);

        match self.client.get(&check_url).send().await {
            Ok(resp) => {
                if let Some(set_cookie) = resp.headers().get("set-cookie") {
                    if let Ok(cookie_str) = set_cookie.to_str() {
                        let token = cookie_str.split(';').next().unwrap_or("").trim().to_string();
                        if !token.is_empty() {
                            info!("HexData session cookie refreshed: {}", token);
                            if let Ok(mut c) = self.cookie.write() {
                                *c = Some(token);
                            }
                            if let Ok(mut t) = self.last_handshake.write() {
                                *t = Some(Instant::now());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("HexData handshake runtime-check failed: {}", e);
            }
        }
    }

    pub async fn handshake(&self) {
        self.ensure_handshake().await;
    }

    pub async fn fetch_meta(&self) -> Option<Value> {
        let url = format!("{}/data/meta.json", self.base_domain);
        self.fetch_json(&url).await
    }

    pub async fn fetch_heroes_summary(&self) -> Option<Value> {
        let url = format!("{}/api/hexdata/heroes_summary", self.base_domain);
        self.fetch_json(&url).await
    }

    pub async fn fetch_augments_summary(&self) -> Option<Value> {
        let url = format!("{}/api/hexdata/augments_summary", self.base_domain);
        self.fetch_json(&url).await
    }

    /// On-demand single hero detail fetch (ARAM/Cherry mode only, strictly on-demand)
    pub async fn fetch_hero_detail(&self, hero_id: &str) -> Option<Value> {
        let hid = hero_id.trim();
        if hid.is_empty() {
            return None;
        }

        // 1. Try static CDN data path: /data/heroes/{hero_id}.json
        let cdn_url = format!("{}/data/heroes/{}.json", self.base_domain, hid);
        if let Some(val) = self.fetch_json(&cdn_url).await {
            return Some(val);
        }

        // 2. Try API path: /api/hexdata/heroes/{hero_id}
        let api_url = format!("{}/api/hexdata/heroes/{}", self.base_domain, hid);
        if let Some(val) = self.fetch_json(&api_url).await {
            return Some(val);
        }

        None
    }

    async fn fetch_json(&self, url: &str) -> Option<Value> {
        self.ensure_handshake().await;
        debug!("Fetching Hexdata JSON: {}", url);

        let mut req = self.client.get(url);
        if let Ok(lock) = self.cookie.read() {
            if let Some(ref c) = *lock {
                if let Ok(header_val) = HeaderValue::from_str(c) {
                    req = req.header(COOKIE, header_val);
                }
            }
        }

        match req.send().await {
            Ok(res) if res.status().is_success() => res.json::<Value>().await.ok(),
            Ok(res) => {
                debug!("HTTP request to {} returned status {}", url, res.status());
                None
            }
            Err(e) => {
                debug!("HTTP request to {} failed: {}", url, e);
                None
            }
        }
    }
}
