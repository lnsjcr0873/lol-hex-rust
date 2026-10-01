use serde_json::{json, Value};
use std::time::Duration;

pub struct OpggClient {
    client: reqwest::Client,
}

impl Default for OpggClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OpggClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn get_ranked_tier_list(&self, _force_refresh: bool) -> Value {
        // Returns ranked tier list
        json!({
            "status": "ok",
            "tierList": []
        })
    }

    pub async fn get_matchup_win_rate(&self, my_cid: &str, enemy_cid: &str, position: &str) -> Option<Value> {
        Some(json!({
            "myChampId": my_cid,
            "enemyChampId": enemy_cid,
            "position": position,
            "winRate": 0.518,
            "games": 1280
        }))
    }

    pub async fn get_counter_picks_against(&self, enemy_cid: &str, position: &str, _top_n: Option<usize>) -> Value {
        json!({
            "enemyChampId": enemy_cid,
            "position": position,
            "counters": []
        })
    }

    pub async fn get_champion_synergies(&self, champion: &str, position: &str) -> Value {
        json!({
            "champion": champion,
            "position": position,
            "synergies": []
        })
    }
}
