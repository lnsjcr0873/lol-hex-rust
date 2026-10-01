use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LiveGameStats {
    pub is_in_game: bool,
    pub hero_name: String,
    pub hero_id: String,
    pub gold: f64,
    pub level: i32,
    pub current_items: Vec<String>,
    pub enemy_team: Vec<Value>,
}

pub struct LiveClientDataReader {
    client: reqwest::Client,
}

impl Default for LiveClientDataReader {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveClientDataReader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_millis(600))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn get_all_game_data(&self) -> Option<Value> {
        let url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
        if let Ok(res) = self.client.get(url).send().await {
            if res.status().is_success() {
                if let Ok(data) = res.json::<Value>().await {
                    return Some(data);
                }
            }
        }
        None
    }

    pub async fn get_live_stats(&self) -> LiveGameStats {
        let mut stats = LiveGameStats::default();
        let data = match self.get_all_game_data().await {
            Some(d) => d,
            None => return stats,
        };

        stats.is_in_game = true;

        let mut active_summoner_name = String::new();
        let mut active_riot_id = String::new();

        if let Some(active) = data.get("activePlayer") {
            stats.level = active.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            stats.gold = active.get("currentGold").and_then(|v| v.as_f64()).unwrap_or(0.0);
            active_summoner_name = active.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            active_riot_id = active.get("riotId").and_then(|v| v.as_str()).unwrap_or("").to_string();
        }

        if let Some(players) = data.get("allPlayers").and_then(|v| v.as_array()) {
            let mut local_team = "";
            for p in players {
                let p_summoner = p.get("summonerName").and_then(|v| v.as_str()).unwrap_or("");
                let p_riot_id = p.get("riotId").and_then(|v| v.as_str()).unwrap_or("");
                let is_local = (!active_summoner_name.is_empty() && p_summoner == active_summoner_name)
                    || (!active_riot_id.is_empty() && p_riot_id == active_riot_id);

                if is_local {
                    local_team = p.get("team").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(cname) = p.get("championName").and_then(|v| v.as_str()) {
                        stats.hero_name = cname.to_string();
                    } else if let Some(raw_cname) = p.get("rawChampionName").and_then(|v| v.as_str()) {
                        let clean = raw_cname.split('_').last().unwrap_or(raw_cname);
                        stats.hero_name = clean.to_string();
                    }

                    if let Some(items) = p.get("items").and_then(|v| v.as_array()) {
                        for it in items {
                            if let Some(iid) = it.get("itemID").and_then(|v| v.as_i64()) {
                                if iid > 0 {
                                    stats.current_items.push(iid.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if !local_team.is_empty() {
                for p in players {
                    let team = p.get("team").and_then(|v| v.as_str()).unwrap_or("");
                    if team != local_team {
                        stats.enemy_team.push(p.clone());
                    }
                }
            }
        }

        stats
    }
}
