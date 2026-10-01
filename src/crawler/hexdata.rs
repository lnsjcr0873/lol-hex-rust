use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER, USER_AGENT};
use serde_json::Value;
use std::time::Duration;
use tracing::info;

pub struct HexdataCrawler {
    client: reqwest::Client,
    base_url: String,
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
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));
        headers.insert(REFERER, HeaderValue::from_static("https://hexdata.com.cn/"));
        headers.insert("sec-ch-ua", HeaderValue::from_static("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
        headers.insert("x-hex-client", HeaderValue::from_static("true"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: "https://hexdata.com.cn/api/hexdata".to_string(),
        }
    }

    pub async fn fetch_meta(&self) -> Option<Value> {
        let url = format!("{}/meta", self.base_url);
        self.fetch_json(&url).await
    }

    pub async fn fetch_heroes_summary(&self) -> Option<Value> {
        let url = format!("{}/heroes_summary", self.base_url);
        self.fetch_json(&url).await
    }

    pub async fn fetch_augments_summary(&self) -> Option<Value> {
        let url = format!("{}/augments_summary", self.base_url);
        self.fetch_json(&url).await
    }

    pub async fn fetch_hero_detail(&self, hero_id: &str) -> Option<Value> {
        let url = format!("{}/heroes_detail/{}", self.base_url, hero_id);
        self.fetch_json(&url).await
    }

    async fn fetch_json(&self, url: &str) -> Option<Value> {
        info!("Fetching {}", url);
        match self.client.get(url).send().await {
            Ok(res) if res.status().is_success() => res.json::<Value>().await.ok(),
            _ => None,
        }
    }
}
