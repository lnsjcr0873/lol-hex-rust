use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct OpggClient {
    client: reqwest::Client,
    region: String,
    tier_list_cache: RwLock<Option<(Instant, Value)>>,
    champion_details_cache: RwLock<HashMap<String, (Instant, Value)>>,
    cache_dir: PathBuf,
}

impl Default for OpggClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OpggClient {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            ),
        );
        headers.insert(
            reqwest::header::REFERER,
            reqwest::header::HeaderValue::from_static("https://www.op.gg/"),
        );
        headers.insert(
            reqwest::header::ORIGIN,
            reqwest::header::HeaderValue::from_static("https://www.op.gg"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json, text/plain, */*"),
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        let cache_dir = PathBuf::from("data/opgg_cache");
        let _ = fs::create_dir_all(&cache_dir);

        let s = Self {
            client,
            region: "kr".to_string(),
            tier_list_cache: RwLock::new(None),
            champion_details_cache: RwLock::new(HashMap::new()),
            cache_dir,
        };
        s.load_disk_tier_cache();
        s
    }

    fn load_disk_tier_cache(&self) {
        let path = self.cache_dir.join(format!("ranked_tier_list_{}.json", self.region));
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    if let Ok(mut lock) = self.tier_list_cache.write() {
                        *lock = Some((Instant::now(), val));
                    }
                }
            }
        }
    }

    pub fn get_cached_stats(&self, champion_id: &str, position: &str) -> Option<(f64, i32)> {
        if let Ok(lock) = self.tier_list_cache.read() {
            if let Some((_, ref val)) = *lock {
                if let Some(champ) = val.get("champions").and_then(|c| c.get(champion_id)) {
                    let pos_upper = position.to_uppercase();
                    if let Some(pos_data) = champ.get("positions").and_then(|p| p.get(&pos_upper)) {
                        let wr = pos_data.get("winRate").and_then(|v| v.as_f64())?;
                        let tier = pos_data.get("tier").and_then(|v| v.as_i64()).unwrap_or(3) as i32;
                        return Some((wr, tier));
                    }
                    let overall_wr = champ.get("overallWinRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                    let overall_tier = champ.get("overallTier").and_then(|v| v.as_i64()).unwrap_or(3) as i32;
                    return Some((overall_wr, overall_tier));
                }
            }
        }
        None
    }

    pub async fn get_ranked_tier_list(&self, force_refresh: bool) -> Value {
        // 1. Check in-memory cache (TTL: 12 hours)
        if !force_refresh {
            if let Ok(lock) = self.tier_list_cache.read() {
                if let Some((instant, ref val)) = *lock {
                    if instant.elapsed() < Duration::from_secs(12 * 3600) {
                        return val.clone();
                    }
                }
            }
        }

        let url = format!("https://lol-api-champion.op.gg/api/{}/champions/ranked", self.region);
        info!("Fetching OP.GG ranked tier list: {}", url);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(raw) = resp.json::<Value>().await {
                    let mut parsed_champs = HashMap::new();
                    if let Some(champ_list) = raw.get("data").and_then(|v| v.as_array()) {
                        for item in champ_list {
                            if let Some(cid) = item.get("id").and_then(|v| v.as_i64()) {
                                let cid_str = cid.to_string();
                                let avg = item.get("average_stats");
                                let overall_tier = avg.and_then(|a| a.get("tier")).and_then(|v| v.as_i64()).unwrap_or(3);
                                let overall_wr = avg.and_then(|a| a.get("win_rate")).and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let overall_pr = avg.and_then(|a| a.get("pick_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let overall_br = avg.and_then(|a| a.get("ban_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                                let mut positions = HashMap::new();
                                let mut primary_pos = "MID".to_string();
                                let mut max_role_rate = -1.0;

                                if let Some(pos_list) = item.get("positions").and_then(|v| v.as_array()) {
                                    for p in pos_list {
                                        let pname = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_uppercase();
                                        if pname.is_empty() { continue; }
                                        let pstats = p.get("stats");
                                        let tier_data = pstats.and_then(|s| s.get("tier_data"));
                                        let ptier = tier_data.and_then(|t| t.get("tier")).and_then(|v| v.as_i64())
                                            .or_else(|| pstats.and_then(|s| s.get("tier")).and_then(|v| v.as_i64()))
                                            .unwrap_or(overall_tier);
                                        let p_wr = pstats.and_then(|s| s.get("win_rate")).and_then(|v| v.as_f64()).unwrap_or(overall_wr);
                                        let p_pr = pstats.and_then(|s| s.get("pick_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let p_rr = pstats.and_then(|s| s.get("role_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                                        let pos_cn = match pname.as_str() {
                                            "TOP" => "上单",
                                            "JUNGLE" => "打野",
                                            "MID" => "中单",
                                            "ADC" => "下路",
                                            "SUPPORT" => "辅助",
                                            _ => pname.as_str(),
                                        };

                                        if p_rr > max_role_rate {
                                            max_role_rate = p_rr;
                                            primary_pos = pname.clone();
                                        }

                                        positions.insert(pname.clone(), json!({
                                            "position": pname,
                                            "positionCn": pos_cn,
                                            "tier": ptier,
                                            "winRate": p_wr,
                                            "pickRate": p_pr,
                                            "roleRate": p_rr
                                        }));
                                    }
                                }

                                parsed_champs.insert(cid_str.clone(), json!({
                                    "id": cid,
                                    "championId": cid_str,
                                    "overallTier": overall_tier,
                                    "overallWinRate": overall_wr,
                                    "overallPickRate": overall_pr,
                                    "overallBanRate": overall_br,
                                    "primaryPosition": primary_pos,
                                    "positions": positions
                                }));
                            }
                        }
                    }

                    let result = json!({
                        "region": self.region,
                        "champions": parsed_champs
                    });

                    // Save to in-memory cache
                    if let Ok(mut lock) = self.tier_list_cache.write() {
                        *lock = Some((Instant::now(), result.clone()));
                    }
                    // Save to disk cache
                    let path = self.cache_dir.join(format!("ranked_tier_list_{}.json", self.region));
                    let _ = fs::write(path, serde_json::to_string_pretty(&result).unwrap_or_default());

                    return result;
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG ranked tier list: {}", e);
            }
            _ => {}
        }

        // Return fallback cache if available
        if let Ok(lock) = self.tier_list_cache.read() {
            if let Some((_, ref val)) = *lock {
                return val.clone();
            }
        }

        json!({
            "region": self.region,
            "champions": {}
        })
    }

    pub async fn get_champion_detail(&self, champion_id: &str, position: &str) -> Option<Value> {
        let pos_upper = match position.to_lowercase().as_str() {
            "top" => "TOP",
            "jungle" => "JUNGLE",
            "middle" | "mid" => "MID",
            "bottom" | "bot" | "adc" => "ADC",
            "utility" | "support" => "SUPPORT",
            _ => "MID",
        };
        let pos_lower = pos_upper.to_lowercase();
        let cache_key = format!("{}_{}", champion_id, pos_upper);

        // 1. Check in-memory cache (TTL: 24h)
        if let Ok(lock) = self.champion_details_cache.read() {
            if let Some((instant, ref val)) = lock.get(&cache_key) {
                if instant.elapsed() < Duration::from_secs(24 * 3600) {
                    return Some(val.clone());
                }
            }
        }

        // 2. Check disk cache
        let disk_filename = format!("detail_{}_{}_{}.json", self.region, champion_id, pos_upper);
        let disk_path = self.cache_dir.join(&disk_filename);
        if disk_path.exists() {
            if let Ok(content) = fs::read_to_string(&disk_path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    if let Ok(mut lock) = self.champion_details_cache.write() {
                        lock.insert(cache_key.clone(), (Instant::now(), val.clone()));
                    }
                    return Some(val);
                }
            }
        }

        // 3. Fetch from OP.GG
        let url = format!(
            "https://lol-api-champion.op.gg/api/{}/champions/ranked/{}/{}",
            self.region, champion_id, pos_lower
        );
        info!("Fetching OP.GG champion detail: {}", url);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(raw) = resp.json::<Value>().await {
                    let data = raw.get("data")?;

                    // Parse counters
                    let mut counters = Vec::new();
                    let mut counter_map = HashMap::new();
                    if let Some(arr) = data.get("counters").and_then(|v| v.as_array()) {
                        for c in arr {
                            if let Some(opp_id) = c.get("champion_id").and_then(|v| v.as_i64()) {
                                let opp_str = opp_id.to_string();
                                let play = c.get("play").and_then(|v| v.as_i64()).unwrap_or(0);
                                let win = c.get("win").and_then(|v| v.as_i64()).unwrap_or(0);
                                let wr = if play > 0 { win as f64 / play as f64 } else { 0.5 };
                                let entry = json!({
                                    "opponentId": opp_id,
                                    "opponentIdStr": opp_str,
                                    "games": play,
                                    "wins": win,
                                    "winRate": wr,
                                    "advantageDelta": wr - 0.5,
                                    "isAdvantage": wr >= 0.5
                                });
                                counters.push(entry.clone());
                                counter_map.insert(opp_str, entry);
                            }
                        }
                    }

                    // Parse rune_pages
                    let mut recommended_runes = Value::Null;
                    if let Some(rune_pages) = data.get("rune_pages").and_then(|v| v.as_array()) {
                        if let Some(top_page) = rune_pages.first() {
                            let pri_style = top_page.get("primary_page_id").and_then(|v| v.as_i64()).unwrap_or(8000);
                            let sec_style = top_page.get("secondary_page_id").and_then(|v| v.as_i64()).unwrap_or(8200);
                            if let Some(build) = top_page.get("builds").and_then(|b| b.as_array()).and_then(|arr| arr.first()) {
                                let pri_runes = build.get("primary_rune_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                                let sec_runes = build.get("secondary_rune_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                                let stat_mods = build.get("stat_mod_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                                let mut all_perks = pri_runes.clone();
                                all_perks.extend(sec_runes.clone());
                                all_perks.extend(stat_mods.clone());

                                recommended_runes = json!({
                                    "primaryStyleId": pri_style,
                                    "subStyleId": sec_style,
                                    "selectedPerkIds": all_perks,
                                    "primaryRunes": pri_runes,
                                    "secondaryRunes": sec_runes,
                                    "statMods": stat_mods,
                                    "pickRate": build.get("pick_rate"),
                                });
                            }
                        }
                    }

                    // Parse summoner_spells
                    let mut recommended_spells = Vec::new();
                    if let Some(spells) = data.get("summoner_spells").and_then(|v| v.as_array()) {
                        for s in spells {
                            if let Some(ids) = s.get("ids").and_then(|v| v.as_array()) {
                                if ids.len() >= 2 {
                                    recommended_spells.push(json!({
                                        "spell1Id": ids[0],
                                        "spell2Id": ids[1],
                                        "pickRate": s.get("pick_rate"),
                                    }));
                                }
                            }
                        }
                    }

                    let detail_result = json!({
                        "championId": champion_id,
                        "position": pos_upper,
                        "counters": counters,
                        "counterMap": counter_map,
                        "recommendedRunes": recommended_runes,
                        "recommendedSpells": recommended_spells,
                    });

                    // Store in memory
                    if let Ok(mut lock) = self.champion_details_cache.write() {
                        lock.insert(cache_key, (Instant::now(), detail_result.clone()));
                    }
                    // Store to disk
                    let _ = fs::write(disk_path, serde_json::to_string_pretty(&detail_result).unwrap_or_default());

                    return Some(detail_result);
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG champion detail: {}", e);
            }
            _ => {}
        }

        None
    }

    pub async fn get_matchup_win_rate(&self, my_cid: &str, enemy_cid: &str, position: &str) -> Option<Value> {
        let detail = self.get_champion_detail(my_cid, position).await;
        if let Some(ref d) = detail {
            if let Some(matchup) = d.get("counterMap").and_then(|m| m.get(enemy_cid)) {
                let wr = matchup.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let diff = wr - 0.5;
                let label = if diff >= 0.03 {
                    format!("明显克制 (+{:.1}%)", diff * 100.0)
                } else if diff >= 0.005 {
                    format!("略占优势 (+{:.1}%)", diff * 100.0)
                } else if diff <= -0.03 {
                    format!("明显劣势 ({:.1}%)", diff * 100.0)
                } else if diff <= -0.005 {
                    format!("稍显劣势 ({:.1}%)", diff * 100.0)
                } else {
                    "均势对局 (50-50)".to_string()
                };

                return Some(json!({
                    "myChampId": my_cid,
                    "enemyChampId": enemy_cid,
                    "position": position,
                    "winRate": wr,
                    "advantageDelta": diff,
                    "isAdvantage": diff >= 0.0,
                    "games": matchup.get("games").and_then(|v| v.as_i64()).unwrap_or(0),
                    "label": label
                }));
            }
        }

        // Fallback default
        Some(json!({
            "myChampId": my_cid,
            "enemyChampId": enemy_cid,
            "position": position,
            "winRate": 0.505,
            "games": 100,
            "label": "均势对局 (50-50)"
        }))
    }

    pub async fn get_counter_picks_against(&self, enemy_cid: &str, position: &str, top_n: Option<usize>) -> Value {
        let enemy_detail = self.get_champion_detail(enemy_cid, position).await;
        let mut candidates = Vec::new();

        if let Some(ref d) = enemy_detail {
            if let Some(counters) = d.get("counters").and_then(|v| v.as_array()) {
                for c in counters {
                    let enemy_wr = c.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                    let counter_wr = 1.0 - enemy_wr;
                    candidates.push(json!({
                        "championId": c.get("opponentIdStr"),
                        "counterWinRate": counter_wr,
                        "games": c.get("games"),
                    }));
                }
            }
        }

        candidates.sort_by(|a, b| {
            let wr_a = a.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let wr_b = b.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            wr_b.partial_cmp(&wr_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = top_n.unwrap_or(50);
        if candidates.len() > n {
            candidates.truncate(n);
        }

        json!({
            "enemyChampId": enemy_cid,
            "position": position,
            "counters": candidates
        })
    }

    pub async fn get_champion_synergies(&self, champion_id: &str, position: &str) -> Value {
        let pos_upper = position.to_uppercase();
        let pos_lower = match pos_upper.as_str() {
            "TOP" => "top",
            "JUNGLE" => "jungle",
            "MID" => "mid",
            "ADC" => "adc",
            "SUPPORT" => "support",
            _ => "mid",
        };
        let disk_filename = format!("synergies_{}_{}_{}.json", self.region, champion_id, pos_upper);
        let disk_path = self.cache_dir.join(&disk_filename);

        if disk_path.exists() {
            if let Ok(content) = fs::read_to_string(&disk_path) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    return val;
                }
            }
        }

        let url = format!(
            "https://lol-api-champion.op.gg/api/{}/champions/ranked/{}/{}/synergies",
            self.region, champion_id, pos_lower
        );
        info!("Fetching OP.GG champion synergies: {}", url);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(raw) = resp.json::<Value>().await {
                    let mut list = Vec::new();
                    if let Some(arr) = raw.get("data").and_then(|v| v.as_array()) {
                        for item in arr {
                            if let Some(syn_cid) = item.get("synergy_champion_id").and_then(|v| v.as_i64()) {
                                let wr = item.get("win_rate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let games = item.get("play").and_then(|v| v.as_i64()).unwrap_or(0);
                                let tier = item.get("synergy_tier_data").and_then(|t| t.get("tier")).and_then(|v| v.as_i64()).unwrap_or(3) as i32;
                                let syn_pos = item.get("synergy_position").and_then(|v| v.as_str()).unwrap_or("").to_uppercase();
                                list.push(json!({
                                    "championId": syn_cid.to_string(),
                                    "position": syn_pos,
                                    "winRate": wr,
                                    "games": games,
                                    "tier": tier,
                                    "delta": wr - 0.5,
                                }));
                            }
                        }
                    }

                    list.sort_by(|a, b| {
                        let wr_a = a.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let wr_b = b.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        wr_b.partial_cmp(&wr_a).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let res = json!({
                        "championId": champion_id,
                        "position": pos_upper,
                        "synergies": list
                    });

                    let _ = fs::write(&disk_path, serde_json::to_string_pretty(&res).unwrap_or_default());
                    return res;
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG synergies: {}", e);
            }
            _ => {}
        }

        json!({
            "championId": champion_id,
            "position": pos_upper,
            "synergies": []
        })
    }
}
