use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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
        } else if (record.ranked_wins + record.ranked_losses) > 0 {
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
            50
        };

        let cur_champ_tag = if current_champion_id > 0 {
            if cur_g >= 3 && cur_wr_pct >= 60 {
                format!("👑 绝活({}%)", cur_wr_pct)
            } else if cur_g >= 1 {
                format!("近{}场 {}%", cur_g, cur_wr_pct)
            } else if m_count >= 4 {
                "⚠️ 试英雄(0场)".to_string()
            } else {
                "近期未选".to_string()
            }
        } else {
            String::new()
        };

        // Smart badge logic aligned with HexBrain
        let badge_text = if streak_type == "win" && streak_count >= 3 {
            format!("🔥 {}连胜大爹", streak_count)
        } else if streak_type == "loss" && streak_count >= 3 {
            format!("❄️ {}连败预警", streak_count)
        } else if current_champion_id > 0 && cur_g >= 3 && cur_wr_pct >= 60 {
            format!("👑 绝活({}%)", cur_wr_pct)
        } else if current_champion_id > 0 && m_count >= 4 && cur_g == 0 {
            "⚠️ 试英雄".to_string()
        } else if win_rate_pct >= 70 && m_count >= 6 {
            "⚡ 通天代".to_string()
        } else if win_rate_pct >= 55 {
            "📈 稳定上分".to_string()
        } else if win_rate_pct <= 42 && m_count >= 6 {
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
            if let Some(games) = h.get("games").and_then(|g| g.get("games")).and_then(|v| v.as_array()) {
                for g in games {
                    let target_participant_id = g.get("participantIdentities")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| {
                            arr.iter().find(|pi| {
                                let pl = pi.get("player");
                                let pl_puuid = pl.and_then(|p| p.get("puuid")).and_then(|v| v.as_str());
                                let inline_puuid = pi.get("puuid").and_then(|v| v.as_str());
                                if pl_puuid == Some(puuid) || inline_puuid == Some(puuid) {
                                    if real_name.is_empty() {
                                        if let Some(n) = pl.and_then(|p| p.get("gameName").or_else(|| p.get("summonerName"))).and_then(|v| v.as_str()) {
                                            real_name = n.to_string();
                                        }
                                    }
                                    true
                                } else {
                                    false
                                }
                            })
                        })
                        .and_then(|pi| pi.get("participantId"))
                        .and_then(|v| v.as_i64());

                    let participant = g.get("participants")
                        .and_then(|p| p.as_array())
                        .and_then(|arr| {
                            if let Some(pid) = target_participant_id {
                                arr.iter().find(|p| p.get("participantId").and_then(|v| v.as_i64()) == Some(pid))
                            } else {
                                arr.iter().find(|p| p.get("puuid").and_then(|v| v.as_str()) == Some(puuid))
                            }
                            .or_else(|| arr.first())
                        });

                    let stats = participant.and_then(|p| p.get("stats"));
                    let is_win = stats.and_then(|s| s.get("win")).and_then(|w| w.as_bool()).unwrap_or(false);
                    let cid = participant.and_then(|p| p.get("championId")).and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                    let k = stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let d = stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let a = stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                    parsed_matches.push(ParsedMatchRecord {
                        is_win,
                        champion_id: cid,
                        kills: k,
                        deaths: d,
                        assists: a,
                    });
                }
            }
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
        let sample_limit = self.sample_count.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let mut profiles = Vec::with_capacity(slot_count);

        for (idx, slot) in assigned_puuids.iter().enumerate() {
            let cur_cid = my_team_cids.get(idx).copied().unwrap_or(0);
            if let Some((ref puuid, ref name)) = slot {
                let cached_record = {
                    let guard = self.raw_cache.read().await;
                    guard.get(puuid).cloned()
                };

                let record = if let Some(rec) = cached_record {
                    if rec.fetched_at.elapsed() < Duration::from_secs(600) {
                        Some(rec)
                    } else {
                        self.fetch_and_cache_player(puuid, name, lcu).await
                    }
                } else {
                    self.fetch_and_cache_player(puuid, name, lcu).await
                };

                if let Some(rec) = record {
                    profiles.push(Some(self.compute_tactical_profile(&rec, cur_cid, sample_limit)));
                } else {
                    profiles.push(None);
                }
            } else {
                profiles.push(None);
            }
        }

        profiles
    }
}
