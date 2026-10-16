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
    pub game_mode: String,
    pub map_number: i32,
    pub is_aram: bool,
    pub game_time: f64,
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

        if let Some(game_data) = data.get("gameData") {
            let gm = game_data.get("gameMode").and_then(|v| v.as_str()).unwrap_or("").to_uppercase();
            let mn = game_data.get("mapNumber").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            stats.is_aram = mn == 12 || mn == 30 || gm == "ARAM" || gm == "CHERRY" || gm == "KIWI";
            stats.game_mode = gm;
            stats.map_number = mn;
            stats.game_time = game_data.get("gameTime").and_then(|v| v.as_f64()).unwrap_or(0.0);
        }

        let active_opt = data.get("activePlayer");
        if let Some(active) = active_opt {
            stats.level = active.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            stats.gold = active.get("currentGold").and_then(|v| v.as_f64()).unwrap_or(0.0);
        }

        if let Some(players) = data.get("allPlayers").and_then(|v| v.as_array()) {
            let mut local_team = String::new();
            for p in players {
                let is_local = if let Some(active) = active_opt {
                    Self::check_is_local_player(active, p)
                } else {
                    false
                };

                if is_local {
                    local_team = Self::extract_player_info(p, &mut stats);
                    break;
                }
            }

            // Fallback for single-player games (Practice tool, solo custom match)
            if stats.hero_name.is_empty() && players.len() == 1 {
                local_team = Self::extract_player_info(&players[0], &mut stats);
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

    fn extract_player_info(p: &Value, stats: &mut LiveGameStats) -> String {
        let team = p.get("team").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if let Some(cname) = p.get("championName").and_then(|v| v.as_str()) {
            stats.hero_name = cname.to_string();
        } else if let Some(raw_cname) = p.get("rawChampionName").and_then(|v| v.as_str()) {
            let clean = raw_cname.split('_').last().unwrap_or(raw_cname);
            stats.hero_name = clean.to_string();
        }

        if let Some(skin_id) = p.get("skinID").and_then(|v| v.as_i64()) {
            if skin_id > 0 {
                let cid = if skin_id >= 1000 { skin_id / 1000 } else { skin_id };
                stats.hero_id = cid.to_string();
            }
        }
        if stats.hero_id.is_empty() {
            if let Some(cid) = p.get("championId").and_then(|v| v.as_i64()) {
                if cid > 0 {
                    stats.hero_id = cid.to_string();
                }
            }
        }

        if let Some(items) = p.get("items").and_then(|v| v.as_array()) {
            stats.current_items.clear();
            for it in items {
                if let Some(iid) = it.get("itemID").and_then(|v| v.as_i64()) {
                    if iid > 0 {
                        stats.current_items.push(iid.to_string());
                    }
                }
            }
        }

        team
    }

    fn check_is_local_player(active: &Value, player: &Value) -> bool {
        let a_puuid = active.get("puuid").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_puuid = player.get("puuid").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !a_puuid.is_empty() && !p_puuid.is_empty() && a_puuid.eq_ignore_ascii_case(p_puuid) {
            return true;
        }

        let a_sum = active.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_sum = player.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !a_sum.is_empty() && !p_sum.is_empty() && a_sum.eq_ignore_ascii_case(p_sum) {
            return true;
        }

        let a_riot = active.get("riotId").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_riot = player.get("riotId").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !a_riot.is_empty() && !p_riot.is_empty() && a_riot.eq_ignore_ascii_case(p_riot) {
            return true;
        }

        let a_game_name = active.get("riotIdGameName").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_game_name = player.get("riotIdGameName").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !a_game_name.is_empty() && !p_game_name.is_empty() && a_game_name.eq_ignore_ascii_case(p_game_name) {
            return true;
        }

        // Cross-compare summonerName, riotIdGameName, and tag-stripped riotId
        let a_base = if !a_game_name.is_empty() {
            a_game_name
        } else if !a_riot.is_empty() {
            a_riot.split('#').next().unwrap_or(a_riot).trim()
        } else {
            a_sum
        };

        let p_base = if !p_game_name.is_empty() {
            p_game_name
        } else if !p_riot.is_empty() {
            p_riot.split('#').next().unwrap_or(p_riot).trim()
        } else {
            p_sum
        };

        if !a_base.is_empty() && !p_base.is_empty() && a_base.eq_ignore_ascii_case(p_base) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_is_local_player_puuid() {
        let active = json!({ "puuid": "puuid-1234", "summonerName": "ActivePlayer" });
        let player = json!({ "puuid": "puuid-1234", "summonerName": "DifferentName" });
        assert!(LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_check_is_local_player_cross_riot_id() {
        let active = json!({ "riotId": "Faker#KR1", "summonerName": "" });
        let player = json!({ "riotIdGameName": "Faker", "riotIdTagLine": "KR1", "summonerName": "" });
        assert!(LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_check_is_local_player_case_insensitive() {
        let active = json!({ "summonerName": "theshy" });
        let player = json!({ "summonerName": "TheShy" });
        assert!(LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_extract_player_info_skin_and_items() {
        let player = json!({
            "championName": "Aatrox",
            "skinID": 266001,
            "team": "ORDER",
            "items": [
                { "itemID": 1055, "count": 1 },
                { "itemID": 2003, "count": 2 }
            ]
        });
        let mut stats = LiveGameStats::default();
        let team = LiveClientDataReader::extract_player_info(&player, &mut stats);
        assert_eq!(team, "ORDER");
        assert_eq!(stats.hero_name, "Aatrox");
        assert_eq!(stats.hero_id, "266");
        assert_eq!(stats.current_items, vec!["1055", "2003"]);
    }
}
