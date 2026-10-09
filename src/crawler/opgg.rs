use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct OpggClient {
    client: reqwest::Client,
    region: String,
    tier_list_cache: RwLock<Option<(Instant, Value)>>,
    champion_details_cache: RwLock<HashMap<String, (Instant, Value)>>,
    synergies_cache: RwLock<HashMap<String, (Instant, Value)>>,
    cooldown_until: RwLock<Option<Instant>>,
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

        Self {
            client,
            region: "kr".to_string(),
            tier_list_cache: RwLock::new(None),
            champion_details_cache: RwLock::new(HashMap::new()),
            synergies_cache: RwLock::new(HashMap::new()),
            cooldown_until: RwLock::new(None),
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

    pub fn get_primary_position(&self, champion_id: &str) -> Option<String> {
        if let Ok(lock) = self.tier_list_cache.read() {
            if let Some((_, ref val)) = *lock {
                if let Some(champ) = val.get("champions").and_then(|c| c.get(champion_id)) {
                    if let Some(p) = champ.get("primaryPosition").and_then(|p| p.as_str()) {
                        return Some(p.to_string());
                    }
                }
            }
        }
        // Known champion fallback before tier list finishes initial download
        match champion_id {
            "24" | "86" | "266" | "85" | "58" | "98" | "14" | "516" | "83" | "114" | "164" | "75" | "36" | "80" | "31" => Some("TOP".to_string()),
            "64" | "76" | "104" | "121" | "154" | "254" | "19" | "5" | "20" | "35" | "77" | "56" | "60" | "79" | "113" => Some("JUNGLE".to_string()),
            "157" | "238" | "103" | "7" | "61" | "134" | "268" | "3" | "84" | "105" | "4" | "91" | "127" | "245" | "777" => Some("MID".to_string()),
            "222" | "51" | "81" | "145" | "236" | "21" | "119" | "22" | "498" | "18" | "15" | "29" | "67" | "96" | "110" => Some("ADC".to_string()),
            "412" | "89" | "53" | "12" | "43" | "111" | "201" | "40" | "267" | "350" | "497" | "235" | "223" | "432" | "888" => Some("SUPPORT".to_string()),
            _ => None,
        }
    }

    pub async fn get_ranked_tier_list(&self, force_refresh: bool) -> Value {
        // 1. Check in-memory cache with 6 hours TTL
        if !force_refresh {
            if let Ok(lock) = self.tier_list_cache.read() {
                if let Some((instant, ref val)) = *lock {
                    if instant.elapsed() < Duration::from_secs(6 * 3600) {
                        return val.clone();
                    }
                }
            }
        }

        // Check if currently under rate-limit cooldown
        if let Ok(lock) = self.cooldown_until.read() {
            if let Some(until) = *lock {
                if Instant::now() < until {
                    if let Ok(c_lock) = self.tier_list_cache.read() {
                        if let Some((_, ref val)) = *c_lock {
                            return val.clone();
                        }
                    }
                    return json!({ "region": self.region, "champions": {} });
                }
            }
        }

        let url = format!(
            "https://lol-api-champion.op.gg/api/{}/champions/ranked?tier=all",
            self.region
        );
        info!("Fetching OP.GG ranked tier list: {}", url);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(raw) = resp.json::<Value>().await {
                    let mut parsed_champs = json!({});

                    if let Some(arr) = raw.get("data").and_then(|v| v.as_array()) {
                        for item in arr {
                            if let Some(cid) = item.get("id").and_then(|v| v.as_i64()) {
                                let cid_str = cid.to_string();
                                let mut positions = json!({});

                                if let Some(pos_arr) = item.get("positions").and_then(|v| v.as_array()) {
                                    for p in pos_arr {
                                        if let Some(pos_name) = p.get("name").and_then(|v| v.as_str()) {
                                            let stats = p.get("stats");
                                            let wr = stats.and_then(|s| s.get("win_rate")).and_then(|v| v.as_f64()).unwrap_or(0.5);
                                            let pr = stats.and_then(|s| s.get("pick_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                            let br = stats.and_then(|s| s.get("ban_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                            let tier = stats.and_then(|s| s.get("tier_data")).and_then(|t| t.get("tier")).and_then(|v| v.as_i64()).unwrap_or(3) as i32;

                                            positions[pos_name.to_uppercase()] = json!({
                                                "position": pos_name.to_uppercase(),
                                                "tier": tier,
                                                "winRate": wr,
                                                "pickRate": pr,
                                                "banRate": br
                                            });
                                        }
                                    }
                                }

                                let overall_tier = item.get("average_stats").and_then(|s| s.get("tier")).and_then(|v| v.as_i64()).unwrap_or(3) as i32;
                                let overall_wr = item.get("average_stats").and_then(|s| s.get("win_rate")).and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let overall_pr = item.get("average_stats").and_then(|s| s.get("pick_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let overall_br = item.get("average_stats").and_then(|s| s.get("ban_rate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let primary_pos = item.get("position").and_then(|v| v.as_str()).unwrap_or("MID").to_uppercase();

                                parsed_champs[cid_str.clone()] = json!({
                                    "championId": cid_str,
                                    "overallTier": overall_tier,
                                    "overallWinRate": overall_wr,
                                    "overallPickRate": overall_pr,
                                    "overallBanRate": overall_br,
                                    "primaryPosition": primary_pos,
                                    "positions": positions
                                });
                            }
                        }
                    }

                    let result = json!({
                        "region": self.region,
                        "champions": parsed_champs
                    });

                    // Store in memory cache ONLY (zero disk clutter)
                    if let Ok(mut lock) = self.tier_list_cache.write() {
                        *lock = Some((Instant::now(), result.clone()));
                    }

                    return result;
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() == 403 || status.as_u16() == 429 {
                    warn!("OP.GG rate-limited ({}), entering 60s cooldown to protect connection", status);
                    if let Ok(mut lock) = self.cooldown_until.write() {
                        *lock = Some(Instant::now() + Duration::from_secs(60));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG ranked tier list: {}", e);
            }
        }

        // Return fallback cache if available
        if let Ok(lock) = self.tier_list_cache.read() {
            if let Some((_, ref val)) = *lock {
                return val.clone();
            }
        }

        json!({ "region": self.region, "champions": {} })
    }

    pub async fn get_champion_detail_exact(&self, champion_id: &str, position: &str) -> Option<Value> {
        if champion_id.is_empty() || champion_id == "0" {
            return None;
        }

        // Check if currently under rate-limit cooldown
        if let Ok(lock) = self.cooldown_until.read() {
            if let Some(until) = *lock {
                if Instant::now() < until {
                    return None;
                }
            }
        }

        let pos_upper = position.to_uppercase();
        let pos_lower = match pos_upper.as_str() {
            "TOP" => "top",
            "JUNGLE" => "jungle",
            "MID" => "mid",
            "ADC" => "adc",
            "SUPPORT" => "support",
            _ => "mid",
        };
        let cache_key = format!("{}_{}_{}", self.region, champion_id, pos_upper);

        // 1. Check in-memory cache with 12 hours TTL
        if let Ok(lock) = self.champion_details_cache.read() {
            if let Some((instant, ref val)) = lock.get(&cache_key) {
                if instant.elapsed() < Duration::from_secs(12 * 3600) {
                    return Some(val.clone());
                }
            }
        }

        // 2. Fetch directly from OP.GG API
        let url = format!(
            "https://lol-api-champion.op.gg/api/{}/champions/ranked/{}/{}",
            self.region, champion_id, pos_lower
        );
        info!("Fetching OP.GG champion detail: {}", url);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(raw) = resp.json::<Value>().await {
                    let mut counter_map = json!({});
                    let mut counters_list = Vec::new();

                    if let Some(counters_data) = raw.get("data").and_then(|d| d.get("counters")).and_then(|c| c.as_array()) {
                        for c in counters_data {
                            if let Some(opp_id) = c.get("opponent_champion_id").and_then(|v| v.as_i64()) {
                                let opp_id_str = opp_id.to_string();
                                let wr = c.get("win").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let games = c.get("play").and_then(|v| v.as_i64()).unwrap_or(0);
                                counter_map[opp_id_str.clone()] = json!({
                                    "opponentId": opp_id,
                                    "opponentIdStr": opp_id_str.clone(),
                                    "winRate": wr,
                                    "games": games
                                });
                                counters_list.push(json!({
                                    "opponentId": opp_id,
                                    "opponentIdStr": opp_id_str,
                                    "winRate": wr,
                                    "games": games
                                }));
                            }
                        }
                    }

                    // Parse Recommended Runes
                    let mut rune_build = json!(null);
                    if let Some(runes) = raw.get("data").and_then(|d| d.get("runes")).and_then(|r| r.as_array()) {
                        if let Some(first_rune) = runes.first() {
                            let primary_page_id = first_rune.get("primary_page_id").and_then(|v| v.as_i64()).unwrap_or(8000);
                            let secondary_page_id = first_rune.get("secondary_page_id").and_then(|v| v.as_i64()).unwrap_or(8100);
                            let primary_rune_ids: Vec<i64> = first_rune.get("primary_rune_ids")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|x| x.as_i64()).collect())
                                .unwrap_or_default();
                            let secondary_rune_ids: Vec<i64> = first_rune.get("secondary_rune_ids")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|x| x.as_i64()).collect())
                                .unwrap_or_default();
                            let stat_mod_ids: Vec<i64> = first_rune.get("stat_mod_ids")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|x| x.as_i64()).collect())
                                .unwrap_or_default();

                            let mut all_selected_perks = Vec::new();
                            all_selected_perks.extend(primary_rune_ids);
                            all_selected_perks.extend(secondary_rune_ids);
                            all_selected_perks.extend(stat_mod_ids);

                            let wr = first_rune.get("win").and_then(|v| v.as_f64()).unwrap_or(0.5);
                            let pick_rate = first_rune.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);

                            rune_build = json!({
                                "primaryStyleId": primary_page_id,
                                "subStyleId": secondary_page_id,
                                "selectedPerkIds": all_selected_perks,
                                "winRate": wr,
                                "pickRate": pick_rate
                            });
                        }
                    }

                    // Parse Recommended Core Item Builds
                    let mut core_items = Vec::new();
                    if let Some(items) = raw.get("data").and_then(|d| d.get("core_items")).and_then(|i| i.as_array()) {
                        for build in items.iter().take(3) {
                            if let Some(ids) = build.get("ids").and_then(|v| v.as_array()) {
                                let item_ids: Vec<String> = ids.iter().filter_map(|x| x.as_i64().map(|n| n.to_string())).collect();
                                let wr = build.get("win").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let games = build.get("play").and_then(|v| v.as_i64()).unwrap_or(0);
                                core_items.push(json!({
                                    "itemIds": item_ids,
                                    "winRate": wr,
                                    "games": games
                                }));
                            }
                        }
                    }

                    let detail_result = json!({
                        "championId": champion_id,
                        "position": pos_upper,
                        "counterMap": counter_map,
                        "counters": counters_list,
                        "recommendedRunes": rune_build,
                        "coreItems": core_items
                    });

                    // Store in memory cache ONLY (zero disk clutter)
                    if let Ok(mut lock) = self.champion_details_cache.write() {
                        lock.insert(cache_key, (Instant::now(), detail_result.clone()));
                    }

                    return Some(detail_result);
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() == 403 || status.as_u16() == 429 {
                    warn!("OP.GG rate-limited ({}), entering 60s cooldown to protect connection", status);
                    if let Ok(mut lock) = self.cooldown_until.write() {
                        *lock = Some(Instant::now() + Duration::from_secs(60));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG champion detail: {}", e);
            }
        }

        None
    }

    pub async fn get_champion_detail(&self, champion_id: &str, position: &str) -> Option<Value> {
        let exact = self.get_champion_detail_exact(champion_id, position).await;
        let is_empty = exact.as_ref()
            .and_then(|d| d.get("counters"))
            .and_then(|c| c.as_array())
            .map_or(true, |a| a.is_empty());
        if is_empty {
            if let Some(primary) = self.get_primary_position(champion_id) {
                if primary.to_uppercase() != position.to_uppercase() {
                    info!("OP.GG detail for {} on {} empty, falling back to primary position {}", champion_id, position, primary);
                    if let Some(fb) = self.get_champion_detail_exact(champion_id, &primary).await {
                        return Some(fb);
                    }
                }
            }
        }
        exact
    }

    pub async fn get_matchup_win_rate(&self, my_cid: &str, enemy_cid: &str, position: &str) -> Option<Value> {
        if my_cid.is_empty() || my_cid == "0" || enemy_cid.is_empty() || enemy_cid == "0" {
            return None;
        }

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
        if enemy_cid.is_empty() || enemy_cid == "0" {
            return json!({
                "enemyChampId": enemy_cid,
                "position": position,
                "counters": []
            });
        }

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

    pub async fn get_champion_synergies_exact(&self, champion_id: &str, position: &str) -> Value {
        if champion_id.is_empty() || champion_id == "0" {
            return json!({
                "championId": champion_id,
                "position": position,
                "synergies": []
            });
        }

        // Check if currently under rate-limit cooldown
        if let Ok(lock) = self.cooldown_until.read() {
            if let Some(until) = *lock {
                if Instant::now() < until {
                    return json!({
                        "championId": champion_id,
                        "position": position,
                        "synergies": []
                    });
                }
            }
        }

        let pos_upper = position.to_uppercase();
        let pos_lower = match pos_upper.as_str() {
            "TOP" => "top",
            "JUNGLE" => "jungle",
            "MID" => "mid",
            "ADC" => "adc",
            "SUPPORT" => "support",
            _ => "mid",
        };
        let cache_key = format!("{}_{}_{}", self.region, champion_id, pos_upper);

        // 1. Check in-memory cache with 12 hours TTL
        if let Ok(lock) = self.synergies_cache.read() {
            if let Some((instant, ref val)) = lock.get(&cache_key) {
                if instant.elapsed() < Duration::from_secs(12 * 3600) {
                    return val.clone();
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

                    // Store in memory cache ONLY (zero disk clutter)
                    if let Ok(mut lock) = self.synergies_cache.write() {
                        lock.insert(cache_key, (Instant::now(), res.clone()));
                    }
                    return res;
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.as_u16() == 403 || status.as_u16() == 429 {
                    warn!("OP.GG rate-limited ({}), entering 60s cooldown to protect connection", status);
                    if let Ok(mut lock) = self.cooldown_until.write() {
                        *lock = Some(Instant::now() + Duration::from_secs(60));
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch OP.GG synergies: {}", e);
            }
        }

        json!({
            "championId": champion_id,
            "position": pos_upper,
            "synergies": []
        })
    }

    pub async fn get_champion_synergies(&self, champion_id: &str, position: &str) -> Value {
        let exact = self.get_champion_synergies_exact(champion_id, position).await;
        let is_empty = exact.get("synergies").and_then(|v| v.as_array()).map_or(true, |a| a.is_empty());
        if is_empty {
            if let Some(primary) = self.get_primary_position(champion_id) {
                if primary.to_uppercase() != position.to_uppercase() {
                    let fb = self.get_champion_synergies_exact(champion_id, &primary).await;
                    if !fb.get("synergies").and_then(|v| v.as_array()).map_or(true, |a| a.is_empty()) {
                        return fb;
                    }
                }
            }
        }
        exact
    }
}
