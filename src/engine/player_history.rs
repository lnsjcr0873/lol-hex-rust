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
    pub streak_type: String, // "win", "loss", "none"
    pub streak_count: i32,
    pub badge_text: String, // e.g. "🔥 连胜大爹", "👑 绝活哥", "⚡ 通天代", "⚠️ 练英雄", "📈 稳定上分", "⚖️ 均势玩家"
    pub top_champions: Vec<String>,
}

#[derive(Clone)]
pub struct PlayerHistoryManager {
    cache: Arc<RwLock<HashMap<String, (PlayerTacticalProfile, Instant)>>>,
    pub sample_count: Arc<std::sync::atomic::AtomicI32>,
}

impl Default for PlayerHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerHistoryManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            sample_count: Arc::new(std::sync::atomic::AtomicI32::new(20)),
        }
    }

    pub async fn get_profile(&self, puuid: &str) -> Option<PlayerTacticalProfile> {
        let guard = self.cache.read().await;
        if let Some((profile, exp)) = guard.get(puuid) {
            if Instant::now() < *exp {
                return Some(profile.clone());
            }
        }
        None
    }

    pub async fn resolve_teammates(
        &self,
        session: &Value,
        lcu: &crate::lcu::LcuClient,
    ) -> Vec<PlayerTacticalProfile> {
        let chat_details = session.get("chatDetails");
        let chat_room_id = chat_details
            .and_then(|c| c.get("multiUserChatId").or_else(|| c.get("chatRoomId")).or_else(|| c.get("chatRoomName")))
            .and_then(|v| v.as_str())
            .or_else(|| session.get("chatRoomId").and_then(|v| v.as_str()));

        let mut participants = Vec::new();
        if let Some(room_id) = chat_room_id {
            participants = lcu.get_chat_conversation_participants(room_id).await;
        }

        let mut profiles = Vec::new();
        for p in participants {
            let puuid = p.get("id").or_else(|| p.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
            if puuid.is_empty() {
                continue;
            }

            let name = p.get("name").or_else(|| p.get("game_name")).and_then(|v| v.as_str()).unwrap_or("");

            if let Some(cached) = self.get_profile(puuid).await {
                profiles.push(cached);
                continue;
            }

            // Fetch ranked stats
            let ranked = lcu.get_player_ranked_stats(puuid).await;
            let mut tier_str = "未定级".to_string();
            let mut lp = 0i32;
            let mut wins = 0i32;
            let mut losses = 0i32;

            if let Some(r) = &ranked {
                let queues = r.get("queueMap").and_then(|q| q.as_object());
                let solo = queues.and_then(|qm| qm.get("RANKED_SOLO_5x5"));
                if let Some(s) = solo {
                    let t = s.get("tier").and_then(|v| v.as_str()).unwrap_or("");
                    let div = s.get("division").and_then(|v| v.as_str()).unwrap_or("");
                    lp = s.get("leaguePoints").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    wins = s.get("wins").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    losses = s.get("losses").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

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

            // Fetch match history
            let limit = self.sample_count.load(std::sync::atomic::Ordering::Relaxed) as usize;
            let history = lcu.get_player_match_history(puuid, limit).await;
            let mut m_wins = 0i32;
            let mut m_count = 0i32;
            let mut streak_type = "none".to_string();
            let mut streak_count = 0i32;
            let mut champ_counts: HashMap<String, (i32, i32)> = HashMap::new(); // champ_name -> (games, wins)

            if let Some(h) = &history {
                if let Some(games) = h.get("games").and_then(|g| g.get("games")).and_then(|v| v.as_array()) {
                    m_count = games.len() as i32;
                    let mut first_win = None;

                    for g in games {
                        let is_win = g.get("participants")
                            .and_then(|p| p.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|p| p.get("stats"))
                            .and_then(|s| s.get("win"))
                            .and_then(|w| w.as_bool())
                            .unwrap_or(false);

                        if is_win {
                            m_wins += 1;
                        }

                        if first_win.is_none() {
                            first_win = Some(is_win);
                            streak_type = if is_win { "win".to_string() } else { "loss".to_string() };
                            streak_count = 1;
                        } else if first_win == Some(is_win) {
                            streak_count += 1;
                        }

                        let cid = g.get("participants")
                            .and_then(|p| p.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|p| p.get("championId"))
                            .and_then(|c| c.as_i64())
                            .unwrap_or(0);
                        if cid > 0 {
                            let entry = champ_counts.entry(cid.to_string()).or_insert((0, 0));
                            entry.0 += 1;
                            if is_win {
                                entry.1 += 1;
                            }
                        }
                    }
                }
            }

            let win_rate_pct = if m_count > 0 {
                ((m_wins as f64 / m_count as f64) * 100.0).round() as i32
            } else if (wins + losses) > 0 {
                ((wins as f64 / (wins + losses) as f64) * 100.0).round() as i32
            } else {
                50
            };

            let badge_text = if streak_type == "win" && streak_count >= 3 {
                format!("🔥 {}连胜大爹", streak_count)
            } else if streak_type == "loss" && streak_count >= 3 {
                format!("❄️ {}连败预警", streak_count)
            } else if win_rate_pct >= 70 && m_count >= 8 {
                "⚡ 通天代".to_string()
            } else if win_rate_pct >= 55 {
                "📈 稳定上分".to_string()
            } else if win_rate_pct <= 42 && m_count >= 8 {
                "⚠️ 状态低迷".to_string()
            } else {
                "⚖️ 均势玩家".to_string()
            };

            let mut sorted_champs: Vec<_> = champ_counts.into_iter().collect();
            sorted_champs.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
            let top_champions = sorted_champs.into_iter().take(3).map(|(cid, _)| cid).collect();

            let profile = PlayerTacticalProfile {
                puuid: puuid.to_string(),
                summoner_name: name.to_string(),
                tier_str,
                lp,
                win_rate_pct,
                total_games: if (wins + losses) > 0 { wins + losses } else { m_count },
                streak_type,
                streak_count,
                badge_text,
                top_champions,
            };

            // Cache for 10 minutes
            let mut cache_guard = self.cache.write().await;
            cache_guard.insert(puuid.to_string(), (profile.clone(), Instant::now() + Duration::from_secs(600)));
            profiles.push(profile);
        }

        profiles
    }
}
