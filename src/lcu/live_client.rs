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
    pub active_augments: Vec<String>,
    pub enemy_team: Vec<Value>,
    pub game_mode: String,
    pub map_number: i32,
    pub is_aram: bool,
    pub has_hextech: bool,
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
            stats.has_hextech = mn == 30 || gm == "CHERRY" || gm == "KIWI" || gm == "MAYHEM" || gm == "SWIFTPLAY";
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

        if stats.active_augments.is_empty() {
            if let Some(active) = active_opt {
                if let Some(arr) = active.get("augments").or_else(|| active.get("playerAugments")).and_then(|v| v.as_array()) {
                    stats.active_augments = Self::parse_augments_array(arr);
                }
            }
        }

        if !stats.has_hextech && !stats.active_augments.is_empty() {
            stats.has_hextech = true;
        }

        stats
    }

    pub fn parse_augments_array(arr: &[Value]) -> Vec<String> {
        let mut augments = Vec::new();
        for aug in arr {
            if let Some(s) = aug.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    augments.push(trimmed.to_string());
                }
            } else if let Some(aug_id) = aug.get("id").and_then(|v| {
                v.as_str().map(|s| s.trim().to_string()).or_else(|| v.as_i64().map(|n| n.to_string()))
            }) {
                if !aug_id.is_empty() {
                    augments.push(aug_id);
                }
            } else if let Some(s) = aug.get("name").and_then(|v| v.as_str()) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    augments.push(trimmed.to_string());
                }
            }
        }
        augments
    }

    fn extract_player_info(p: &Value, stats: &mut LiveGameStats) -> String {
        let team = p.get("team").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut hero_name = String::new();
        if let Some(raw_cname) = p.get("rawChampionName").and_then(|v| v.as_str()) {
            let trimmed = raw_cname.trim();
            if !trimmed.is_empty() {
                let clean = trimmed.strip_prefix("game_character_displayname_").unwrap_or(trimmed);
                hero_name = clean.to_string();
            }
        }
        if hero_name.is_empty() {
            if let Some(cname) = p.get("championName").and_then(|v| v.as_str()) {
                let trimmed = cname.trim();
                if !trimmed.is_empty() {
                    hero_name = trimmed.to_string();
                }
            }
        }
        stats.hero_name = hero_name;

        // Note: Do NOT use skinID to calculate hero_id. skinID in Riot Live Client Data API
        // represents the player's skin slot index (0, 1, 2...), NOT a global skin ID.
        // Doing so incorrectly mapped players with skins to Annie (1) or Olaf (2).
        // Only accept explicit championId if provided in payload; otherwise leave stats.hero_id
        // empty so that main.rs can resolve it via Database::resolve_hero_id(&stats.hero_name).
        if let Some(cid) = p.get("championId").and_then(|v| v.as_i64()) {
            if cid > 0 {
                stats.hero_id = cid.to_string();
            }
        }

        if let Some(items) = p.get("items").and_then(|v| v.as_array()) {
            stats.current_items.clear();
            for it in items {
                if let Some(iid) = it.get("itemID").and_then(|v| v.as_i64()) {
                    if iid > 0 {
                        let count = it
                            .get("count")
                            .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                            .unwrap_or(1)
                            .clamp(1, 100);
                        for _ in 0..count {
                            stats.current_items.push(iid.to_string());
                        }
                    }
                }
            }
        }

        stats.active_augments.clear();
        if let Some(arr) = p.get("augments").or_else(|| p.get("playerAugments")).and_then(|v| v.as_array()) {
            stats.active_augments = Self::parse_augments_array(arr);
        }

        team
    }

    fn check_is_local_player(active: &Value, player: &Value) -> bool {
        let a_puuid = active.get("puuid").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_puuid = player.get("puuid").and_then(|v| v.as_str()).unwrap_or("").trim();

        // 1. Strict PUUID check: if both have PUUID, they must match; if they differ, reject immediately
        if !a_puuid.is_empty() && !p_puuid.is_empty() {
            return a_puuid.eq_ignore_ascii_case(p_puuid);
        }

        // Helper to extract (game_name, tag_line)
        let parse_riot_identity = |val: &Value| -> (String, String) {
            let game_name_field = val.get("riotIdGameName").and_then(|v| v.as_str()).unwrap_or("").trim();
            let tag_line_field = val.get("riotIdTagLine").and_then(|v| v.as_str()).unwrap_or("").trim();
            let full_riot_id = val.get("riotId").and_then(|v| v.as_str()).unwrap_or("").trim();

            let mut game_name = game_name_field.to_string();
            let mut tag_line = tag_line_field.to_string();

            if !full_riot_id.is_empty() {
                if let Some((g, t)) = full_riot_id.split_once('#') {
                    if game_name.is_empty() {
                        game_name = g.trim().to_string();
                    }
                    if tag_line.is_empty() {
                        tag_line = t.trim().to_string();
                    }
                } else if game_name.is_empty() {
                    game_name = full_riot_id.to_string();
                }
            }

            if game_name.is_empty() {
                let summoner_name = val.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").trim();
                if !summoner_name.is_empty() {
                    if let Some((g, t)) = summoner_name.split_once('#') {
                        game_name = g.trim().to_string();
                        if tag_line.is_empty() {
                            tag_line = t.trim().to_string();
                        }
                    } else {
                        game_name = summoner_name.to_string();
                    }
                }
            }

            (game_name, tag_line)
        };

        let (a_name, a_tag) = parse_riot_identity(active);
        let (p_name, p_tag) = parse_riot_identity(player);

        // 2. Strict Riot ID Tag comparison: If BOTH sides have tag lines, they MUST match
        if !a_tag.is_empty() && !p_tag.is_empty() {
            if !a_tag.eq_ignore_ascii_case(&p_tag) {
                return false;
            }
            return !a_name.is_empty() && !p_name.is_empty() && a_name.eq_ignore_ascii_case(&p_name);
        }

        // 3. If tag line is missing on one or both sides (e.g. bots or legacy client), match base game name
        if !a_name.is_empty() && !p_name.is_empty() && a_name.eq_ignore_ascii_case(&p_name) {
            return true;
        }

        // 4. Fallback summonerName exact match if not covered above
        let a_sum = active.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").trim();
        let p_sum = player.get("summonerName").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !a_sum.is_empty() && !p_sum.is_empty() && a_sum.eq_ignore_ascii_case(p_sum) {
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
    fn test_check_is_local_player_different_puuid_rejects_even_if_name_matches() {
        let active = json!({ "puuid": "puuid-1111", "summonerName": "SameName" });
        let player = json!({ "puuid": "puuid-2222", "summonerName": "SameName" });
        assert!(!LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_check_is_local_player_cross_riot_id() {
        let active = json!({ "riotId": "Faker#KR1", "summonerName": "" });
        let player = json!({ "riotIdGameName": "Faker", "riotIdTagLine": "KR1", "summonerName": "" });
        assert!(LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_check_is_local_player_same_name_different_tag_no_match() {
        let active = json!({ "riotId": "Faker#KR1", "summonerName": "Faker" });
        let player1 = json!({ "riotId": "Faker#KR2", "summonerName": "Faker" });
        assert!(!LiveClientDataReader::check_is_local_player(&active, &player1));

        let player2 = json!({ "riotIdGameName": "Faker", "riotIdTagLine": "KR2", "summonerName": "Faker" });
        assert!(!LiveClientDataReader::check_is_local_player(&active, &player2));
    }

    #[test]
    fn test_check_is_local_player_case_insensitive() {
        let active = json!({ "summonerName": "theshy" });
        let player = json!({ "summonerName": "TheShy" });
        assert!(LiveClientDataReader::check_is_local_player(&active, &player));
    }

    #[test]
    fn test_extract_player_info_skin_and_items() {
        // Player wearing skin 1 with championId 266 (must NOT map hero_id to 1 / Annie!)
        let player = json!({
            "championName": "Aatrox",
            "skinID": 1,
            "championId": 266,
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
        assert_eq!(stats.current_items, vec!["1055", "2003", "2003"]);
    }

    #[test]
    fn test_extract_player_info_skin_slot_does_not_override_hero_id() {
        // When championId is absent, skinID 2 (Olaf) must NOT map to hero_id "2"
        let player = json!({
            "rawChampionName": "game_character_displayname_Aatrox",
            "skinID": 2,
            "team": "CHAOS",
            "items": []
        });
        let mut stats = LiveGameStats::default();
        let team = LiveClientDataReader::extract_player_info(&player, &mut stats);
        assert_eq!(team, "CHAOS");
        assert_eq!(stats.hero_name, "Aatrox");
        assert_eq!(stats.hero_id, ""); // Left empty for database resolution
    }

    #[test]
    fn test_extract_player_info_augments_string_and_numeric() {
        let player = json!({
            "championName": "Ahri",
            "team": "ORDER",
            "items": [],
            "augments": [
                "1001",
                { "id": 1068 },
                { "id": "2001" },
                { "name": "循环往复" }
            ]
        });
        let mut stats = LiveGameStats::default();
        LiveClientDataReader::extract_player_info(&player, &mut stats);
        assert_eq!(stats.active_augments, vec!["1001", "1068", "2001", "循环往复"]);
    }

    #[test]
    fn test_extract_player_info_player_augments_fallback() {
        let player = json!({
            "championName": "Ahri",
            "team": "ORDER",
            "items": [],
            "playerAugments": [
                { "id": 3001 }
            ]
        });
        let mut stats = LiveGameStats::default();
        LiveClientDataReader::extract_player_info(&player, &mut stats);
        assert_eq!(stats.active_augments, vec!["3001"]);
    }

    #[test]
    fn test_check_is_local_player_mirror_match_isolation() {
        // In mirror matches (two Ahris), check_is_local_player uses PUUID/RiotID to isolate
        let active = json!({
            "puuid": "puuid-local-1234",
            "riotId": "AhriKing#KR1",
            "summonerName": "AhriKing"
        });
        let enemy_mirror_player = json!({
            "championName": "Ahri",
            "puuid": "puuid-enemy-5678",
            "riotId": "AhriGod#KR2",
            "summonerName": "AhriGod"
        });
        let local_mirror_player = json!({
            "championName": "Ahri",
            "puuid": "puuid-local-1234",
            "riotId": "AhriKing#KR1",
            "summonerName": "AhriKing"
        });

        assert!(!LiveClientDataReader::check_is_local_player(&active, &enemy_mirror_player));
        assert!(LiveClientDataReader::check_is_local_player(&active, &local_mirror_player));
    }
}
