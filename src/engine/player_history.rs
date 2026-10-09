use crate::engine::ranked_draft::{ORDERED_POSITIONS, SHORT_POS_NAMES_CN};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameflowSlotInfo {
    pub cell_id: i64,
    pub puuid: String,
    pub summoner_name: String,
    pub champion_id: i32,
    pub position: String,
    pub position_cn: String,
    pub is_local_player: bool,
    pub profile: Option<PlayerTacticalProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerTacticalProfile {
    pub puuid: String,
    pub summoner_name: String,
    pub tier_str: String,
    pub lp: i32,
    pub win_rate_pct: i32,
    pub total_games: i32,
    pub recent_wins: i32,
    pub recent_losses: i32,
    pub sample_count: i32,
    pub recent_dots: Vec<bool>, // true = win, false = loss (latest 5 games)
    pub streak_type: String,     // "win", "loss", "none"
    pub streak_count: i32,
    pub badge_text: String,      // e.g. "🔥 3连胜", "👑 绝活哥", "⚡ 通天代", "⚠️ 试英雄", "📈 稳定上分", "⚖️ 均势玩家"
    pub top_champions: Vec<String>,
    // Specific champion stats for currently selected champion
    pub cur_champ_id: i32,
    pub cur_champ_games: i32,
    pub cur_champ_wins: i32,
    pub cur_champ_losses: i32,
    pub cur_champ_win_rate_pct: i32,
    pub cur_champ_tag: String,
}

#[derive(Debug, Clone)]
pub struct ParsedMatchRecord {
    pub is_win: bool,
    pub champion_id: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
}

#[derive(Debug, Clone)]
pub struct CachedPlayerRecord {
    pub puuid: String,
    pub summoner_name: String,
    pub tier_str: String,
    pub lp: i32,
    pub ranked_wins: i32,
    pub ranked_losses: i32,
    pub matches: Vec<ParsedMatchRecord>,
    pub fetched_at: Instant,
}

#[derive(Clone)]
pub struct PlayerHistoryManager {
    raw_cache: Arc<RwLock<HashMap<String, CachedPlayerRecord>>>,
    pub sample_count: Arc<std::sync::atomic::AtomicI32>,
    cached_local_puuid: Arc<RwLock<Option<String>>>,
    cached_local_name: Arc<RwLock<Option<String>>>,
}

impl Default for PlayerHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerHistoryManager {
    pub fn new() -> Self {
        Self {
            raw_cache: Arc::new(RwLock::new(HashMap::new())),
            sample_count: Arc::new(std::sync::atomic::AtomicI32::new(20)),
            cached_local_puuid: Arc::new(RwLock::new(None)),
            cached_local_name: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_sample_match_count(&self, count: i32) {
        let valid = [10, 20, 30, 50];
        let c = if valid.contains(&count) { count } else { 20 };
        self.sample_count.store(c, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn compute_tactical_profile(
        &self,
        record: &CachedPlayerRecord,
        current_champion_id: i32,
        sample_limit: usize,
    ) -> PlayerTacticalProfile {
        let all_matches = &record.matches;
        let eff_limit = sample_limit.min(all_matches.len());
        let slice = if eff_limit > 0 { &all_matches[..eff_limit] } else { &[][..] };

        let m_count = slice.len() as i32;
        let mut m_wins = 0i32;
        let mut streak_type = "none".to_string();
        let mut streak_count = 0i32;
        let mut streak_ended = false;
        let mut first_win = None;

        let mut champ_counts: HashMap<i32, (i32, i32)> = HashMap::new(); // cid -> (games, wins)
        let mut total_k = 0i32;
        let mut total_d = 0i32;
        let mut total_a = 0i32;

        for m in slice {
            if m.is_win {
                m_wins += 1;
            }
            if !streak_ended {
                if first_win.is_none() {
                    first_win = Some(m.is_win);
                    streak_type = if m.is_win { "win".to_string() } else { "loss".to_string() };
                    streak_count = 1;
                } else if first_win == Some(m.is_win) {
                    streak_count += 1;
                } else {
                    streak_ended = true;
                }
            }

            if m.champion_id > 0 {
                let entry = champ_counts.entry(m.champion_id).or_insert((0, 0));
                entry.0 += 1;
                if m.is_win {
                    entry.1 += 1;
                }
            }

            total_k += m.kills;
            total_d += m.deaths;
            total_a += m.assists;
        }

        let m_losses = m_count - m_wins;
        let recent_dots: Vec<bool> = all_matches.iter().take(5).map(|m| m.is_win).collect();

        let win_rate_pct = if m_count > 0 {
            ((m_wins as f64 / m_count as f64) * 100.0).round() as i32
        } else if record.ranked_losses > 0 && (record.ranked_wins + record.ranked_losses) > 0 {
            let total = record.ranked_wins + record.ranked_losses;
            ((record.ranked_wins as f64 / total as f64) * 100.0).round() as i32
        } else {
            50
        };

        // Specific champion profile
        let (cur_g, cur_w) = if current_champion_id > 0 {
            champ_counts.get(&current_champion_id).cloned().unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        let cur_l = cur_g - cur_w;
        let cur_wr_pct = if cur_g > 0 {
            ((cur_w as f64 / cur_g as f64) * 100.0).round() as i32
        } else {
            0
        };

        let cur_champ_tag = if current_champion_id > 0 {
            if cur_g >= 4 && cur_wr_pct >= 60 {
                format!("👑 绝活({}%)", cur_wr_pct)
            } else if cur_g >= 1 {
                format!("近{}场 {}%", cur_g, cur_wr_pct)
            } else {
                "⚠️ 试英雄(0场)".to_string()
            }
        } else {
            String::new()
        };

        // Smart badge logic aligned with HexBrain
        let badge_text = if streak_type == "win" && streak_count >= 3 {
            format!("🔥 {}连胜大爹", streak_count)
        } else if streak_type == "loss" && streak_count >= 3 {
            format!("❄️ {}连败预警", streak_count)
        } else if current_champion_id > 0 && cur_g >= 4 && cur_wr_pct >= 60 {
            format!("👑 绝活({}%)", cur_wr_pct)
        } else if current_champion_id > 0 && cur_g == 0 && m_count >= 2 {
            "⚠️ 试英雄".to_string()
        } else if win_rate_pct >= 70 && m_count >= 5 {
            "⚡ 通天代".to_string()
        } else if win_rate_pct >= 55 {
            "📈 稳定上分".to_string()
        } else if win_rate_pct <= 42 && m_count >= 5 {
            "⚠️ 状态低迷".to_string()
        } else {
            "⚖️ 均势玩家".to_string()
        };

        let mut sorted_champs: Vec<_> = champ_counts.into_iter().collect();
        sorted_champs.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
        let top_champions = sorted_champs.into_iter().take(3).map(|(cid, _)| cid.to_string()).collect();

        PlayerTacticalProfile {
            puuid: record.puuid.clone(),
            summoner_name: record.summoner_name.clone(),
            tier_str: record.tier_str.clone(),
            lp: record.lp,
            win_rate_pct,
            total_games: if (record.ranked_wins + record.ranked_losses) > 0 {
                record.ranked_wins + record.ranked_losses
            } else {
                m_count
            },
            recent_wins: m_wins,
            recent_losses: m_losses,
            sample_count: m_count,
            recent_dots,
            streak_type,
            streak_count,
            badge_text,
            top_champions,
            cur_champ_id: current_champion_id,
            cur_champ_games: cur_g,
            cur_champ_wins: cur_w,
            cur_champ_losses: cur_l,
            cur_champ_win_rate_pct: cur_wr_pct,
            cur_champ_tag,
        }
    }

    pub async fn fetch_and_cache_player(
        &self,
        puuid: &str,
        candidate_name: &str,
        lcu: &crate::lcu::LcuClient,
    ) -> Option<CachedPlayerRecord> {
        if puuid.is_empty() {
            return None;
        }

        // 1. Fetch ranked stats
        let ranked = lcu.get_player_ranked_stats(puuid).await;
        let mut tier_str = "未定级".to_string();
        let mut lp = 0i32;
        let mut ranked_wins = 0i32;
        let mut ranked_losses = 0i32;

        if let Some(r) = &ranked {
            let queues = r.get("queueMap").and_then(|q| q.as_object());
            let solo = queues.and_then(|qm| qm.get("RANKED_SOLO_5x5"));
            if let Some(s) = solo {
                let t = s.get("tier").and_then(|v| v.as_str()).unwrap_or("");
                let div = s.get("division").and_then(|v| v.as_str()).unwrap_or("");
                lp = s.get("leaguePoints").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                ranked_wins = s.get("wins").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                ranked_losses = s.get("losses").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                tier_str = match t.to_uppercase().as_str() {
                    "CHALLENGER" => "王者".to_string(),
                    "GRANDMASTER" => "宗师".to_string(),
                    "MASTER" => "大师".to_string(),
                    "DIAMOND" => format!("钻石 {}", div),
                    "EMERALD" => format!("翡翠 {}", div),
                    "PLATINUM" => format!("铂金 {}", div),
                    "GOLD" => format!("黄金 {}", div),
                    "SILVER" => format!("白银 {}", div),
                    "BRONZE" => format!("黄铜 {}", div),
                    "IRON" => format!("黑铁 {}", div),
                    _ => "未定级".to_string(),
                };
            }
        }

        // 2. Fetch match history (up to 50 games for instant slice switching)
        let history = lcu.get_player_match_history(puuid, 49).await;
        let mut parsed_matches = Vec::new();
        let mut real_name = candidate_name.to_string();

        if let Some(h) = &history {
            let empty_vec = Vec::new();
            let games = h.get("games")
                .and_then(|g| {
                    if let Some(arr) = g.as_array() {
                        Some(arr)
                    } else {
                        g.get("games").and_then(|v| v.as_array())
                    }
                })
                .or_else(|| h.get("matches").and_then(|v| v.as_array()))
                .or_else(|| h.as_array())
                .unwrap_or(&empty_vec);

            for g in games {
                let empty_parts = Vec::new();
                let participants = g.get("participants")
                    .and_then(|p| p.as_array())
                    .unwrap_or(&empty_parts);

                let empty_idents = Vec::new();
                let identities = g.get("participantIdentities")
                    .and_then(|p| p.as_array())
                    .unwrap_or(&empty_idents);

                let mut target_p = None;

                // 1. Match directly by puuid in participant
                for p in participants {
                    if let Some(p_puuid) = p.get("puuid").and_then(|v| v.as_str()) {
                        if p_puuid.eq_ignore_ascii_case(puuid) {
                            target_p = Some(p);
                            break;
                        }
                    }
                }

                // 2. Match by participantIdentities
                if target_p.is_none() && !identities.is_empty() {
                    let mut target_pid = None;
                    for pi in identities {
                        let pl = pi.get("player");
                        let pl_puuid = pl.and_then(|p| p.get("puuid")).and_then(|v| v.as_str());
                        let inline_puuid = pi.get("puuid").and_then(|v| v.as_str());
                        if pl_puuid.map(|s| s.eq_ignore_ascii_case(puuid)).unwrap_or(false)
                            || inline_puuid.map(|s| s.eq_ignore_ascii_case(puuid)).unwrap_or(false)
                        {
                            target_pid = pi.get("participantId").and_then(|v| v.as_i64());
                            if real_name.is_empty() {
                                if let Some(n) = pl.and_then(|p| p.get("gameName").or_else(|| p.get("summonerName"))).and_then(|v| v.as_str()) {
                                    real_name = n.to_string();
                                }
                            }
                            break;
                        }
                    }
                    if let Some(pid) = target_pid {
                        for p in participants {
                            if p.get("participantId").and_then(|v| v.as_i64()) == Some(pid) {
                                target_p = Some(p);
                                break;
                            }
                        }
                    }
                }

                let part = match target_p {
                    Some(p) => p,
                    None => continue,
                };

                let stats = part.get("stats");
                let is_win = stats.and_then(|s| s.get("win"))
                    .or_else(|| part.get("win"))
                    .map(|w| {
                        w.as_bool().unwrap_or_else(|| w.as_i64() == Some(1) || w.as_str() == Some("true") || w.as_str() == Some("Win"))
                    })
                    .unwrap_or(false);

                let cid = part.get("championId")
                    .or_else(|| stats.and_then(|s| s.get("championId")))
                    .and_then(|c| c.as_i64())
                    .unwrap_or(0) as i32;

                let k = stats.and_then(|s| s.get("kills"))
                    .or_else(|| part.get("kills"))
                    .and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                let d = stats.and_then(|s| s.get("deaths"))
                    .or_else(|| part.get("deaths"))
                    .and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                let a = stats.and_then(|s| s.get("assists"))
                    .or_else(|| part.get("assists"))
                    .and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                parsed_matches.push(ParsedMatchRecord {
                    is_win,
                    champion_id: cid,
                    kills: k,
                    deaths: d,
                    assists: a,
                });
            }
        }

        if parsed_matches.is_empty() && ranked.is_none() {
            // Network failure or timeout: do not cache empty failure long-term
            return None;
        }

        let record = CachedPlayerRecord {
            puuid: puuid.to_string(),
            summoner_name: if real_name.is_empty() { candidate_name.to_string() } else { real_name },
            tier_str,
            lp,
            ranked_wins,
            ranked_losses,
            matches: parsed_matches,
            fetched_at: Instant::now(),
        };

        // Insert into cache (TTL 10 min)
        let mut guard = self.raw_cache.write().await;
        guard.insert(puuid.to_string(), record.clone());

        Some(record)
    }

    pub async fn resolve_teammates(
        &self,
        session: &Value,
        my_team_cids: &[i32],
        my_team_is_locals: &[bool],
        lcu: &crate::lcu::LcuClient,
    ) -> Vec<Option<PlayerTacticalProfile>> {
        let slot_count = my_team_cids.len().max(5);
        let mut assigned_puuids: Vec<Option<(String, String)>> = vec![None; slot_count]; // (puuid, name)

        // 1. Ensure local player PUUID & name
        let mut local_puuid_opt = {
            let guard = self.cached_local_puuid.read().await;
            guard.clone()
        };
        let mut local_name_opt = {
            let guard = self.cached_local_name.read().await;
            guard.clone()
        };

        if local_puuid_opt.is_none() {
            if let Some(cur) = lcu.get_current_summoner().await {
                if let Some(p) = cur.get("puuid").and_then(|v| v.as_str()) {
                    let n = cur.get("gameName")
                        .or_else(|| cur.get("displayName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let mut p_guard = self.cached_local_puuid.write().await;
                    *p_guard = Some(p.to_string());
                    local_puuid_opt = Some(p.to_string());

                    let mut n_guard = self.cached_local_name.write().await;
                    *n_guard = Some(n.to_string());
                    local_name_opt = Some(n.to_string());
                }
            }
        }

        // Assign local player to slot where is_local == true
        if let Some(ref lp) = local_puuid_opt {
            let ln = local_name_opt.clone().unwrap_or_default();
            for (idx, &is_local) in my_team_is_locals.iter().enumerate() {
                if is_local && idx < assigned_puuids.len() {
                    assigned_puuids[idx] = Some((lp.clone(), ln.clone()));
                    break;
                }
            }
        }

        // 2. Collect candidate teammates from chat conversation participants
        let chat_details = session.get("chatDetails");
        let mut chat_room_id = chat_details
            .and_then(|c| c.get("multiUserChatId").or_else(|| c.get("chatRoomId")).or_else(|| c.get("chatRoomName")))
            .and_then(|v| v.as_str())
            .or_else(|| session.get("chatRoomId").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let mut participants = Vec::new();
        if let Some(ref room_id) = chat_room_id {
            participants = lcu.get_chat_conversation_participants(room_id).await;
        }

        // Fallback: If participants empty, query active chat conversations to locate champ-select room
        if participants.is_empty() {
            let convs = lcu.get_chat_conversations().await;
            for c in convs {
                let c_type = c.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let c_id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if c_type == "championSelect" || c_id.contains("champ-select") {
                    let parts = lcu.get_chat_conversation_participants(c_id).await;
                    if !parts.is_empty() {
                        participants = parts;
                        chat_room_id = Some(c_id.to_string());
                        break;
                    }
                }
            }
        }

        // Build candidate list (excluding local player)
        let mut candidate_teammates = Vec::new();
        for p in participants {
            let p_id = p.get("id").or_else(|| p.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
            if p_id.is_empty() {
                continue;
            }
            if let Some(ref lp) = local_puuid_opt {
                if p_id == lp {
                    continue; // Skip local player
                }
            }
            let name = p.get("name")
                .or_else(|| p.get("gameName"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            candidate_teammates.push((p_id.to_string(), name.to_string()));
        }

        // 3. Fallback: inspect session myTeam array for explicit puuids
        if let Some(my_arr) = session.get("myTeam").and_then(|v| v.as_array()) {
            for (idx, m) in my_arr.iter().enumerate() {
                if idx < assigned_puuids.len() && assigned_puuids[idx].is_none() {
                    if let Some(p) = m.get("puuid").and_then(|v| v.as_str()) {
                        if !p.is_empty() && p != "0" {
                            let is_local = my_team_is_locals.get(idx).copied().unwrap_or(false);
                            if !is_local {
                                assigned_puuids[idx] = Some((p.to_string(), String::new()));
                            }
                        }
                    }
                }
            }
        }

        // Distribute chat candidates into remaining empty non-local slots
        let mut c_idx = 0;
        for (idx, item) in assigned_puuids.iter_mut().enumerate() {
            let is_local = my_team_is_locals.get(idx).copied().unwrap_or(false);
            if !is_local && item.is_none() && c_idx < candidate_teammates.len() {
                *item = Some(candidate_teammates[c_idx].clone());
                c_idx += 1;
            }
        }

        // 4. Concurrently fetch or compute tactical profiles
        let mut player_inputs = Vec::with_capacity(slot_count);
        for (idx, slot) in assigned_puuids.into_iter().enumerate() {
            let cur_cid = my_team_cids.get(idx).copied().unwrap_or(0);
            if let Some((p, n)) = slot {
                player_inputs.push((Some(p), Some(n), cur_cid));
            } else {
                player_inputs.push((None, None, cur_cid));
            }
        }

        self.batch_resolve_players(&player_inputs, lcu).await
    }

    pub async fn resolve_opponents_champ_select(
        &self,
        session: &Value,
        their_team_cids: &[i32],
        lcu: &crate::lcu::LcuClient,
    ) -> Vec<Option<PlayerTacticalProfile>> {
        let slot_count = their_team_cids.len().max(5);
        let mut player_inputs = Vec::with_capacity(slot_count);

        let their_arr = session.get("theirTeam").and_then(|v| v.as_array());
        for idx in 0..slot_count {
            let cur_cid = their_team_cids.get(idx).copied().unwrap_or(0);
            let mut puuid_opt = None;
            let mut name_opt = None;

            if let Some(arr) = their_arr {
                if let Some(m) = arr.get(idx) {
                    if let Some(p) = m.get("puuid").and_then(|v| v.as_str()) {
                        if !p.is_empty() && p != "0" {
                            puuid_opt = Some(p.to_string());
                            let n = m.get("summonerName")
                                .or_else(|| m.get("gameName"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            name_opt = Some(n.to_string());
                        }
                    }
                }
            }

            player_inputs.push((puuid_opt, name_opt, cur_cid));
        }

        self.batch_resolve_players(&player_inputs, lcu).await
    }

    pub async fn batch_resolve_players(
        &self,
        players: &[(Option<String>, Option<String>, i32)],
        lcu: &crate::lcu::LcuClient,
    ) -> Vec<Option<PlayerTacticalProfile>> {
        let sample_limit = self.sample_count.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(4));

        let mut handles = Vec::with_capacity(players.len());
        for (puuid_opt, name_opt, cur_cid) in players {
            let p_opt = puuid_opt.clone();
            let n_opt = name_opt.clone();
            let cid = *cur_cid;
            let mgr = self.clone();
            let lcu_client = lcu.clone();
            let sem = semaphore.clone();

            handles.push(tokio::spawn(async move {
                let puuid = match p_opt {
                    Some(ref p) if !p.is_empty() && p != "0" => p.clone(),
                    _ => return None,
                };
                let name = n_opt.unwrap_or_default();

                let cached_record = {
                    let guard = mgr.raw_cache.read().await;
                    guard.get(&puuid).cloned()
                };

                let record = if let Some(rec) = cached_record {
                    if rec.fetched_at.elapsed() < Duration::from_secs(600) {
                        Some(rec)
                    } else {
                        let _permit = sem.acquire().await.ok()?;
                        mgr.fetch_and_cache_player(&puuid, &name, &lcu_client).await
                    }
                } else {
                    let _permit = sem.acquire().await.ok()?;
                    mgr.fetch_and_cache_player(&puuid, &name, &lcu_client).await
                };

                record.map(|r| mgr.compute_tactical_profile(&r, cid, sample_limit))
            }));
        }

        let mut profiles = Vec::with_capacity(players.len());
        for h in handles {
            profiles.push(h.await.unwrap_or(None));
        }
        profiles
    }

    pub async fn resolve_gameflow_10_players(
        &self,
        session: &Value,
        lcu: &crate::lcu::LcuClient,
    ) -> Option<(Vec<GameflowSlotInfo>, Vec<GameflowSlotInfo>)> {
        let game_data = session.get("gameData")?;
        let team_one_raw = game_data.get("teamOne").and_then(|v| v.as_array())?;
        let team_two_raw = game_data.get("teamTwo").and_then(|v| v.as_array())?;

        if team_one_raw.is_empty() && team_two_raw.is_empty() {
            return None;
        }

        let selections_raw = game_data.get("playerChampionSelections").and_then(|v| v.as_array());
        let mut selection_map: HashMap<String, i32> = HashMap::new();
        let mut cell_selection_map: HashMap<i64, i32> = HashMap::new();
        if let Some(sels) = selections_raw {
            for sel in sels {
                let cid = sel.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if cid > 0 {
                    if let Some(p) = sel.get("puuid").and_then(|v| v.as_str()) {
                        selection_map.insert(p.to_string(), cid);
                    }
                    if let Some(n) = sel.get("summonerInternalName").and_then(|v| v.as_str()) {
                        selection_map.insert(n.to_string(), cid);
                    }
                    if let Some(cell) = sel.get("cellId").and_then(|v| v.as_i64()) {
                        cell_selection_map.insert(cell, cid);
                    }
                }
            }
        }

        let mut local_puuid = {
            let guard = self.cached_local_puuid.read().await;
            guard.clone()
        };
        if local_puuid.is_none() {
            if let Some(cur) = lcu.get_current_summoner().await {
                if let Some(p) = cur.get("puuid").and_then(|v| v.as_str()) {
                    let mut p_guard = self.cached_local_puuid.write().await;
                    *p_guard = Some(p.to_string());
                    local_puuid = Some(p.to_string());
                }
            }
        }

        let parse_team = |raw_team: &[Value]| -> Vec<GameflowSlotInfo> {
            let mut slots = Vec::new();
            for (idx, p) in raw_team.iter().enumerate() {
                let puuid = p.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let summoner_name = p.get("gameName")
                    .or_else(|| p.get("summonerName"))
                    .or_else(|| p.get("displayName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let cell_id = p.get("cellId").and_then(|v| v.as_i64()).unwrap_or(idx as i64);
                let internal_name = p.get("summonerInternalName").and_then(|v| v.as_str()).unwrap_or("");

                let mut cid = p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                if cid == 0 && !puuid.is_empty() {
                    cid = selection_map.get(&puuid).copied().unwrap_or(0);
                }
                if cid == 0 && !internal_name.is_empty() {
                    cid = selection_map.get(internal_name).copied().unwrap_or(0);
                }
                if cid == 0 {
                    cid = cell_selection_map.get(&cell_id).copied().unwrap_or(0);
                }

                let auto_pos = p.get("selectedPosition").and_then(|v| v.as_str()).and_then(|s| {
                    match s.to_lowercase().as_str() {
                        "top" => Some("TOP"),
                        "jungle" => Some("JUNGLE"),
                        "middle" | "mid" => Some("MID"),
                        "bottom" | "bot" => Some("ADC"),
                        "utility" | "support" => Some("SUPPORT"),
                        _ => None,
                    }
                });
                let pos = auto_pos.unwrap_or_else(|| ORDERED_POSITIONS[idx % 5]);
                let pos_idx = ORDERED_POSITIONS.iter().position(|&op| op == pos).unwrap_or(idx % 5);

                let is_local = if let Some(ref lp) = local_puuid {
                    !puuid.is_empty() && puuid == *lp
                } else {
                    false
                };

                slots.push(GameflowSlotInfo {
                    cell_id,
                    puuid,
                    summoner_name,
                    champion_id: cid,
                    position: pos.to_string(),
                    position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                    is_local_player: is_local,
                    profile: None,
                });
            }
            slots
        };

        let team_one = parse_team(team_one_raw);
        let team_two = parse_team(team_two_raw);

        let team_one_has_local = team_one.iter().any(|s| s.is_local_player);
        let team_two_has_local = team_two.iter().any(|s| s.is_local_player);

        let (mut my_team, mut their_team) = if team_two_has_local && !team_one_has_local {
            (team_two, team_one)
        } else {
            (team_one, team_two)
        };

        let mut inputs = Vec::new();
        for s in &my_team {
            inputs.push((Some(s.puuid.clone()), Some(s.summoner_name.clone()), s.champion_id));
        }
        for s in &their_team {
            inputs.push((Some(s.puuid.clone()), Some(s.summoner_name.clone()), s.champion_id));
        }

        let profiles = self.batch_resolve_players(&inputs, lcu).await;
        let my_len = my_team.len();
        for (i, p) in profiles.into_iter().enumerate() {
            if i < my_len {
                my_team[i].profile = p;
            } else if (i - my_len) < their_team.len() {
                their_team[i - my_len].profile = p;
            }
        }

        Some((my_team, their_team))
    }
}

