use crate::data::loader::Database;
use crate::engine::ranked_draft::{ORDERED_POSITIONS, SHORT_POS_NAMES_CN};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
pub struct BadgeItem {
    pub text: String,
    pub bg: String,
    pub fg: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopChampItem {
    pub champion_id: i32,
    pub champion_name: String,
    pub games: i32,
    pub win_rate: f64,
    pub win_rate_pct: i32,
    pub kda: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerTacticalProfile {
    pub tier: String,
    pub tier_str: String,
    pub tier_color: String,
    pub lp: i32,
    pub overall_win_rate: Option<f64>,
    pub overall_win_rate_pct: i32,
    pub wins: i32,
    pub losses: i32,
    pub total_games: i32,
    pub sample_limit: usize,
    pub recent_count: i32,
    pub recent_wins: i32,
    pub recent_losses: i32,
    pub recent_win_rate: f64,
    pub recent_win_rate_pct: i32,
    pub recent_dots: Vec<bool>, // true = win, false = loss (latest 5 games)
    pub streak_type: String,     // "win", "loss", "none"
    pub streak_count: i32,
    pub avg_kda_str: String,
    pub avg_kda_ratio: f64,
    pub badges: Vec<BadgeItem>,
    pub top_champs: Vec<TopChampItem>,
    pub radar: HashMap<String, i32>,

    // Selected champion stats
    pub cur_champ_id: i32,
    pub cur_champ_games: i32,
    pub cur_champ_wins: i32,
    pub cur_champ_losses: i32,
    pub cur_champ_win_rate_pct: i32,
    pub cur_champ_tag: String,

    // Backward-compatibility fields for Win32 HUD:
    pub puuid: String,
    pub summoner_name: String,
    pub sample_count: i32,
    pub win_rate_pct: i32,
    pub badge_text: String,
    pub top_champions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedMatchRecord {
    pub game_id: i64,
    pub game_creation: i64,
    pub game_duration: i64,
    pub game_duration_str: String,
    pub queue_id: i32,
    pub mode_cn: String,
    pub champion_id: i32,
    pub champion_name: String,
    pub champion_avatar: String,
    pub win: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda_str: String,
    pub kda_ratio: f64,
    pub kda: f64,
    pub items: Vec<i32>,
    pub all_items: Vec<i32>,
    pub cs: i32,
    pub cs_per_min: f64,
    pub damage: i64,
    pub damage_share_pct: f64,
    pub damage_taken: i64,
    pub damage_mitigated: i64,
    pub gold: i64,
    pub vision_score: i32,
    pub kp_pct: f64,
    pub teammate_puuids: Vec<String>,
    pub is_mvp: bool,
    pub is_svp: bool,
    // Backward compat:
    pub is_win: bool,
}

#[derive(Debug, Clone)]
pub struct CachedPlayerRecord {
    pub puuid: String,
    pub summoner_name: String,
    pub tier_str: String,
    pub tier: String,
    pub tier_color: String,
    pub lp: i32,
    pub ranked_wins: i32,
    pub ranked_losses: i32,
    pub matches: Vec<ParsedMatchRecord>,
    pub fetched_at: Instant,
    pub raw_summoner: Value,
    pub raw_ranked: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OngoingPlayerCard {
    pub slot_index: usize,
    pub team_id: i32,
    pub is_local_player: bool,
    pub puuid: String,
    pub summoner_name: String,
    pub full_name: String,
    pub level: i32,
    pub profile_icon_id: i32,
    pub position: String,
    pub champion_id: i32,
    pub champion_name: String,
    pub champion_avatar: String,
    pub spell1_id: i32,
    pub spell2_id: i32,
    pub ranked: Value,
    pub tactical_analysis: PlayerTacticalProfile,
    pub matches: Vec<ParsedMatchRecord>,
    pub premade_group: Option<i32>,
    pub premade_tag: Option<String>,
    pub premade_color: Option<String>,
    pub premade_bg: Option<String>,
    pub premade_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OngoingMatchResponse {
    pub phase: String,
    pub phase_cn: String,
    pub is_active: bool,
    pub is_mock: bool,
    pub chat_room_id: String,
    pub blue_team: Vec<OngoingPlayerCard>,
    pub red_team: Vec<OngoingPlayerCard>,
    pub my_team: Vec<OngoingPlayerCard>,
    pub their_team: Vec<OngoingPlayerCard>,
    pub game_id: Option<i64>,
    pub message: Option<String>,
}

#[derive(Clone)]
pub struct PlayerHistoryManager {
    raw_cache: Arc<RwLock<HashMap<String, CachedPlayerRecord>>>,
    pub sample_count: Arc<std::sync::atomic::AtomicI32>,
    cached_local_puuid: Arc<RwLock<Option<String>>>,
    cached_local_name: Arc<RwLock<Option<String>>>,
    database: Arc<RwLock<Option<Arc<Database>>>>,
}

impl Default for PlayerHistoryManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_tier_color(tier: &str) -> &'static str {
    match tier.to_uppercase().as_str() {
        "CHALLENGER" => "#f59e0b",
        "GRANDMASTER" => "#ef4444",
        "MASTER" => "#a855f7",
        "DIAMOND" => "#38bdf8",
        "EMERALD" => "#00e696",
        "PLATINUM" => "#2dd4bf",
        "GOLD" => "#c8aa6e",
        "SILVER" => "#94a3b8",
        "BRONZE" => "#b45309",
        "IRON" => "#71717a",
        _ => "#64748b",
    }
}

pub fn compute_radar(matches: &[ParsedMatchRecord]) -> HashMap<String, i32> {
    let mut map = HashMap::new();
    if matches.is_empty() {
        map.insert("damage".to_string(), 50);
        map.insert("kda".to_string(), 50);
        map.insert("kp".to_string(), 50);
        map.insert("survival".to_string(), 50);
        map.insert("economy".to_string(), 50);
        map.insert("vision".to_string(), 50);
        return map;
    }
    let m_count = matches.len() as f64;
    let avg_dmg = matches.iter().map(|m| m.damage as f64).sum::<f64>() / m_count;
    let avg_dmg_share = matches.iter().map(|m| m.damage_share_pct).sum::<f64>() / m_count;
    let total_k = matches.iter().map(|m| m.kills as f64).sum::<f64>();
    let total_d = matches.iter().map(|m| m.deaths as f64).sum::<f64>();
    let total_a = matches.iter().map(|m| m.assists as f64).sum::<f64>();
    let avg_kda = (total_k + total_a) / total_d.max(1.0);
    let avg_deaths = total_d / m_count;
    let avg_kp = matches.iter().map(|m| m.kp_pct).sum::<f64>() / m_count;
    let avg_cs_min = matches.iter().map(|m| m.cs_per_min).sum::<f64>() / m_count;
    let avg_gold = matches.iter().map(|m| m.gold as f64).sum::<f64>() / m_count;
    let avg_vis = matches.iter().map(|m| m.vision_score as f64).sum::<f64>() / m_count;

    let dmg_score = ((avg_dmg / 350.0 + avg_dmg_share * 1.5) as i32).clamp(25, 99);
    let kda_score = ((avg_kda * 18.0) as i32).clamp(25, 99);
    let kp_score = ((avg_kp * 1.35) as i32).clamp(25, 99);
    let surv_score = ((100.0 - avg_deaths * 9.5) as i32).clamp(25, 99);
    let econ_score = ((avg_cs_min * 10.0 + avg_gold / 260.0) as i32).clamp(25, 99);
    let vis_score = ((avg_vis * 2.8) as i32).clamp(25, 99);

    map.insert("damage".to_string(), dmg_score);
    map.insert("kda".to_string(), kda_score);
    map.insert("kp".to_string(), kp_score);
    map.insert("survival".to_string(), surv_score);
    map.insert("economy".to_string(), econ_score);
    map.insert("vision".to_string(), vis_score);
    map
}

pub fn detect_premade_teams(players: &mut [OngoingPlayerCard]) {
    if players.len() < 2 {
        return;
    }
    let n = players.len();
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];

    for i in 0..n {
        let matches_i = &players[i].matches;
        let mut game_ids_i = std::collections::HashSet::new();
        let mut teammate_counts: HashMap<String, usize> = HashMap::new();

        for m in matches_i {
            if m.game_id > 0 {
                game_ids_i.insert(m.game_id);
            }
            for t in &m.teammate_puuids {
                *teammate_counts.entry(t.clone()).or_insert(0) += 1;
            }
        }

        for j in (i + 1)..n {
            let puuid_j = &players[j].puuid;
            let matches_j = &players[j].matches;
            let game_ids_j: std::collections::HashSet<i64> = matches_j.iter().filter(|m| m.game_id > 0).map(|m| m.game_id).collect();

            let shared_games_by_id = if !game_ids_i.is_empty() && !game_ids_j.is_empty() {
                game_ids_i.intersection(&game_ids_j).count()
            } else {
                0
            };
            let shared_by_puuid = if !puuid_j.is_empty() {
                teammate_counts.get(puuid_j).copied().unwrap_or(0)
            } else {
                0
            };

            // STRICT REQUIREMENT: >= 2 games together to eliminate 1-game accidental matchmaking false positive!
            let total_shared = shared_games_by_id.max(shared_by_puuid);
            if total_shared >= 2 {
                adj[i].insert(j);
                adj[j].insert(i);
            }
        }
    }

    let mut visited = std::collections::HashSet::new();
    let mut components = Vec::new();
    for i in 0..n {
        if !visited.contains(&i) {
            let mut comp = Vec::new();
            let mut queue = vec![i];
            visited.insert(i);
            while let Some(curr) = queue.pop() {
                comp.push(curr);
                for &nbr in &adj[curr] {
                    if !visited.contains(&nbr) {
                        visited.insert(nbr);
                        queue.push(nbr);
                    }
                }
            }
            if comp.len() >= 2 {
                components.push(comp);
            }
        }
    }

    let premade_colors = [
        ("#ec4899", "rgba(236,72,153,0.18)", "#f472b6"), // Duo: Peach Pink
        ("#f59e0b", "rgba(245,158,11,0.18)", "#fbbf24"), // Trio: Amber Gold
        ("#8b5cf6", "rgba(139,92,246,0.18)", "#a78bfa"), // Quad: Aurora Purple
        ("#06b6d4", "rgba(6,182,212,0.18)", "#22d3ee"),  // Pent: Sky Blue
    ];

    for (group_idx, comp) in components.iter().enumerate() {
        let (border, bg, text) = premade_colors[group_idx % premade_colors.len()];
        let tag_name = match comp.len() {
            2 => "双排车队",
            3 => "三排车队",
            4 => "四排车队",
            5 => "五排车队",
            _ => "车队",
        };

        for &p_idx in comp {
            let p = &mut players[p_idx];
            p.premade_group = Some((group_idx + 1) as i32);
            p.premade_tag = Some(tag_name.to_string());
            p.premade_color = Some(border.to_string());
            p.premade_bg = Some(bg.to_string());
            p.premade_text = Some(text.to_string());

            if !p.tactical_analysis.badges.iter().any(|b| b.text.contains("车队")) {
                p.tactical_analysis.badges.insert(0, BadgeItem {
                    text: format!("🚗 {}", tag_name),
                    bg: bg.to_string(),
                    fg: text.to_string(),
                    desc: format!("近期共同出战多场对局，研判为同队{}。", tag_name),
                });
            }
        }
    }
}

impl PlayerHistoryManager {
    pub fn new() -> Self {
        Self {
            raw_cache: Arc::new(RwLock::new(HashMap::new())),
            sample_count: Arc::new(std::sync::atomic::AtomicI32::new(20)),
            cached_local_puuid: Arc::new(RwLock::new(None)),
            cached_local_name: Arc::new(RwLock::new(None)),
            database: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_database(&self, db: Arc<Database>) {
        let mut guard = self.database.write().await;
        *guard = Some(db);
    }

    pub async fn get_database(&self) -> Option<Arc<Database>> {
        let guard = self.database.read().await;
        guard.clone()
    }

    pub fn set_sample_match_count(&self, count: i32) {
        let valid = [10, 20, 30, 50];
        let c = if valid.contains(&count) { count } else { 20 };
        self.sample_count.store(c, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn get_player_data(&self, puuid: &str) -> Option<CachedPlayerRecord> {
        let guard = self.raw_cache.read().await;
        guard.get(puuid).cloned()
    }

    pub async fn get_player_data_fresh(&self, puuid: &str, max_age: Duration) -> Option<CachedPlayerRecord> {
        let guard = self.raw_cache.read().await;
        let rec = guard.get(puuid)?;
        if rec.fetched_at.elapsed() <= max_age {
            Some(rec.clone())
        } else {
            None
        }
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

        let mut champ_counts: HashMap<i32, (i32, i32, f64, f64, String)> = HashMap::new(); // cid -> (games, wins, kills, deaths, name)

        let mut total_k = 0;
        let mut total_d = 0;
        let mut total_a = 0;

        for m in slice {
            if m.win {
                m_wins += 1;
            }
            total_k += m.kills;
            total_d += m.deaths;
            total_a += m.assists;

            if !streak_ended {
                if first_win.is_none() {
                    first_win = Some(m.win);
                    streak_type = if m.win { "win".to_string() } else { "loss".to_string() };
                    streak_count = 1;
                } else if first_win == Some(m.win) {
                    streak_count += 1;
                } else {
                    streak_ended = true;
                }
            }

            if m.champion_id > 0 {
                let entry = champ_counts.entry(m.champion_id).or_insert((0, 0, 0.0, 0.0, m.champion_name.clone()));
                entry.0 += 1;
                if m.win {
                    entry.1 += 1;
                }
                entry.2 += m.kills as f64;
                entry.3 += m.deaths as f64;
            }
        }

        let m_losses = m_count - m_wins;
        let recent_dots: Vec<bool> = all_matches.iter().take(5).map(|m| m.win).collect();

        let recent_wr = if m_count > 0 {
            m_wins as f64 / m_count as f64
        } else {
            0.5
        };
        let recent_win_rate_pct = (recent_wr * 100.0).round() as i32;

        let total_ranked = record.ranked_wins + record.ranked_losses;
        // Fix 100% win rate / zero losses bug:
        let overall_win_rate = if total_ranked > 0 {
            Some(record.ranked_wins as f64 / total_ranked as f64)
        } else if m_count > 0 {
            Some(recent_wr)
        } else {
            None
        };
        let overall_win_rate_pct = overall_win_rate.map(|w| (w * 100.0).round() as i32).unwrap_or(recent_win_rate_pct);

        let avg_k = if m_count > 0 { (total_k as f64 / m_count as f64 * 10.0).round() / 10.0 } else { 0.0 };
        let avg_d = if m_count > 0 { (total_d as f64 / m_count as f64 * 10.0).round() / 10.0 } else { 0.0 };
        let avg_a = if m_count > 0 { (total_a as f64 / m_count as f64 * 10.0).round() / 10.0 } else { 0.0 };
        let avg_kda_str = format!("{}/{}/{}", avg_k, avg_d, avg_a);
        let avg_kda_ratio = if total_d > 0 {
            ((total_k + total_a) as f64 / total_d as f64 * 100.0).round() / 100.0
        } else {
            (total_k + total_a) as f64
        };

        // Specific champion profile
        let (cur_g, cur_w) = if current_champion_id > 0 {
            champ_counts.get(&current_champion_id).map(|c| (c.0, c.1)).unwrap_or((0, 0))
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
        let mut badges = Vec::new();
        if streak_type == "win" && streak_count >= 3 {
            badges.push(BadgeItem {
                text: format!("🔥 {}连胜", streak_count),
                bg: "#064e3b".to_string(),
                fg: "#34d399".to_string(),
                desc: format!("状态火热，近期已豪取 {} 连胜！", streak_count),
            });
        } else if streak_type == "loss" && streak_count >= 3 {
            badges.push(BadgeItem {
                text: format!("❄️ {}连败", streak_count),
                bg: "#450a0a".to_string(),
                fg: "#f87171".to_string(),
                desc: format!("警惕心态！近期遭遇 {} 连败，建议稳扎稳打。", streak_count),
            });
        }

        if current_champion_id > 0 && cur_g >= 3 && cur_wr_pct >= 60 {
            badges.push(BadgeItem {
                text: format!("👑 绝活({}%)", cur_wr_pct),
                bg: "#312e81".to_string(),
                fg: "#a5b4fc".to_string(),
                desc: format!("绝活认证！该英雄近 {} 场实战胜率高达 {}%！", cur_g, cur_wr_pct),
            });
        } else if current_champion_id > 0 && cur_g == 0 && m_count >= 4 {
            badges.push(BadgeItem {
                text: "⚠️ 试英雄".to_string(),
                bg: "#451a03".to_string(),
                fg: "#fbbf24".to_string(),
                desc: "近期无该英雄胜场记录，疑似排位试英雄或补位练习。".to_string(),
            });
        }

        let is_smurf = (overall_win_rate.unwrap_or(0.0) >= 0.65 && total_ranked >= 15)
            || (recent_wr >= 0.75 && m_count >= 6 && avg_kda_ratio >= 3.8);
        if is_smurf {
            badges.push(BadgeItem {
                text: "⚡ 通天代".to_string(),
                bg: "#1e3a8a".to_string(),
                fg: "#38bdf8".to_string(),
                desc: "天梯通天代指标！高胜率与高 KDA 统治战局。".to_string(),
            });
        }

        if !is_smurf && m_losses >= 3 {
            let loss_matches: Vec<_> = slice.iter().filter(|m| !m.win).collect();
            let l_k: i32 = loss_matches.iter().map(|m| m.kills).sum();
            let l_d: i32 = loss_matches.iter().map(|m| m.deaths).sum();
            let l_a: i32 = loss_matches.iter().map(|m| m.assists).sum();
            let loss_kda = (l_k + l_a) as f64 / l_d.max(1) as f64;
            if loss_kda >= 3.2 {
                badges.push(BadgeItem {
                    text: "🛡️ 尽力局院长".to_string(),
                    bg: "#14532d".to_string(),
                    fg: "#86efac".to_string(),
                    desc: "尽力局院长！即便输局也保持优异的战损比与高输出占比。".to_string(),
                });
            }
        }

        if avg_kda_ratio >= 4.5 && m_count >= 5 {
            badges.push(BadgeItem {
                text: format!("🎯 KDA {:.1}", avg_kda_ratio),
                bg: "#3f1b0a".to_string(),
                fg: "#fdba74".to_string(),
                desc: format!("场均战损比极高 ({})，团战极度稳健。", avg_kda_str),
            });
        }

        if badges.is_empty() {
            if recent_wr >= 0.55 {
                badges.push(BadgeItem {
                    text: "📈 稳定上分".to_string(),
                    bg: "#064e3b".to_string(),
                    fg: "#a7f3d0".to_string(),
                    desc: "近期打法成熟稳健，胜率稳步上升。".to_string(),
                });
            } else {
                badges.push(BadgeItem {
                    text: "⚖️ 均势玩家".to_string(),
                    bg: "#1f2937".to_string(),
                    fg: "#9ca3af".to_string(),
                    desc: "近期表现平稳，胜负对半开。".to_string(),
                });
            }
        }

        let badge_text = badges.first().map(|b| b.text.clone()).unwrap_or_else(|| "⚖️ 均势玩家".to_string());

        let mut sorted_champs: Vec<_> = champ_counts.into_iter().collect();
        sorted_champs.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        let top_champs: Vec<TopChampItem> = sorted_champs.iter().take(3).map(|(cid, info)| {
            let g = info.0;
            let w = info.1;
            let kda = if info.3 > 0.0 { (info.2 + 0.0) / info.3 } else { info.2 };
            TopChampItem {
                champion_id: *cid,
                champion_name: info.4.clone(),
                games: g,
                win_rate: if g > 0 { w as f64 / g as f64 } else { 0.5 },
                win_rate_pct: if g > 0 { ((w as f64 / g as f64) * 100.0).round() as i32 } else { 50 },
                kda: (kda * 10.0).round() / 10.0,
            }
        }).collect();

        let top_champions = top_champs.iter().map(|tc| tc.champion_id.to_string()).collect();

        PlayerTacticalProfile {
            tier: record.tier.clone(),
            tier_str: record.tier_str.clone(),
            tier_color: record.tier_color.clone(),
            lp: record.lp,
            overall_win_rate,
            overall_win_rate_pct,
            wins: record.ranked_wins,
            losses: record.ranked_losses,
            total_games: if total_ranked > 0 { total_ranked } else { m_count },
            sample_limit,
            recent_count: m_count,
            recent_wins: m_wins,
            recent_losses: m_losses,
            recent_win_rate: recent_wr,
            recent_win_rate_pct,
            recent_dots,
            streak_type,
            streak_count,
            avg_kda_str,
            avg_kda_ratio,
            badges,
            top_champs,
            radar: compute_radar(slice),
            cur_champ_id: current_champion_id,
            cur_champ_games: cur_g,
            cur_champ_wins: cur_w,
            cur_champ_losses: cur_l,
            cur_champ_win_rate_pct: cur_wr_pct,
            cur_champ_tag,
            puuid: record.puuid.clone(),
            summoner_name: record.summoner_name.clone(),
            sample_count: m_count,
            win_rate_pct: recent_win_rate_pct,
            badge_text,
            top_champions,
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
        let mut raw_tier = "UNRANKED".to_string();
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
                raw_tier = t.to_uppercase();

                tier_str = match raw_tier.as_str() {
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

        // 2. Fetch match history (20 matches for fast, responsive parsing)
        let history = lcu.get_player_match_history(puuid, 19).await;
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
                let game_id = g.get("gameId").and_then(|v| v.as_i64()).unwrap_or(0);
                let game_creation = g.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
                let duration_sec = g.get("gameDuration").and_then(|v| v.as_i64()).unwrap_or(0);
                let duration_str = format!("{}:{:02}", duration_sec / 60, duration_sec % 60);
                let queue_id = g.get("queueId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let mode_cn = match queue_id {
                    420 => "排位赛(单双)",
                    440 => "排位赛(灵活)",
                    450 => "极地大乱斗",
                    400 => "匹配模式",
                    1700 | 1710 => "斗魂竞技场",
                    _ => "其他模式",
                }.to_string();

                let empty_parts = Vec::new();
                let participants = g.get("participants").and_then(|p| p.as_array()).unwrap_or(&empty_parts);
                let empty_idents = Vec::new();
                let identities = g.get("participantIdentities").and_then(|p| p.as_array()).unwrap_or(&empty_idents);

                let mut ident_map: HashMap<i64, (String, String)> = HashMap::new(); // pid -> (puuid, name)
                let mut target_pid = None;

                for pi in identities {
                    let pid = pi.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pl = pi.get("player");
                    let p_puuid = pl.and_then(|p| p.get("puuid")).or_else(|| pi.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
                    let p_name = pl.and_then(|p| p.get("gameName").or_else(|| p.get("summonerName"))).and_then(|v| v.as_str()).unwrap_or("");
                    ident_map.insert(pid, (p_puuid.to_string(), p_name.to_string()));

                    if p_puuid.eq_ignore_ascii_case(puuid) {
                        target_pid = Some(pid);
                        if real_name.is_empty() && !p_name.is_empty() {
                            real_name = p_name.to_string();
                        }
                    }
                }

                let mut target_part = None;
                for p in participants {
                    let pid = p.get("participantId").and_then(|v| v.as_i64());
                    if pid == target_pid || (target_pid.is_none() && p.get("puuid").and_then(|v| v.as_str()) == Some(puuid)) {
                        target_part = Some(p);
                        break;
                    }
                }

                let part = match target_part {
                    Some(p) => p,
                    None => continue,
                };

                let my_tid = part.get("teamId").and_then(|v| v.as_i64()).unwrap_or(100);
                let stats = part.get("stats");
                let is_win = stats.and_then(|s| s.get("win"))
                    .or_else(|| part.get("win"))
                    .map(|w| w.as_bool().unwrap_or_else(|| w.as_i64() == Some(1) || w.as_str() == Some("true") || w.as_str() == Some("Win")))
                    .unwrap_or_else(|| {
                        g.get("teams")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter().any(|t| {
                                    t.get("teamId").and_then(|v| v.as_i64()) == Some(my_tid)
                                        && (t.get("win").and_then(|v| v.as_str()) == Some("Win")
                                            || t.get("win").and_then(|v| v.as_bool()) == Some(true))
                                })
                            })
                            .unwrap_or(false)
                    });

                let cid = part.get("championId").or_else(|| stats.and_then(|s| s.get("championId"))).and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                let cname = if let Some(db) = self.get_database().await {
                    db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid))
                } else {
                    format!("英雄{}", cid)
                };
                let cavatar = format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid);

                let k = stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let d = stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let a = stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let kda_ratio = if d > 0 { ((k + a) as f64 / d as f64 * 100.0).round() / 100.0 } else { (k + a) as f64 };
                let kda_str = format!("{}/{}/{}", k, d, a);

                let all_items: Vec<i32> = (0..7).map(|i| {
                    stats.and_then(|s| s.get(&format!("item{}", i))).and_then(|v| v.as_i64()).unwrap_or(0) as i32
                }).collect();
                let non_empty_items: Vec<i32> = all_items.iter().copied().filter(|&it| it > 0).collect();

                let dmg = stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
                let gold = stats.and_then(|s| s.get("goldEarned")).and_then(|v| v.as_i64()).unwrap_or(0);
                let total_cs = stats.and_then(|s| s.get("totalMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32
                    + stats.and_then(|s| s.get("neutralMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let duration_min = (duration_sec as f64 / 60.0).max(1.0);
                let cs_min = (total_cs as f64 / duration_min * 10.0).round() / 10.0;
                let vision = stats.and_then(|s| s.get("visionScore")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let taken = stats.and_then(|s| s.get("totalDamageTaken")).and_then(|v| v.as_i64()).unwrap_or(0);
                let mitig = stats.and_then(|s| s.get("damageSelfMitigated")).and_then(|v| v.as_i64()).unwrap_or(0);

                let mut team_dmg = 0i64;
                let mut team_kills = 0i32;
                let mut team_max_dmg = 0i64;
                let mut team_max_pid = 0i64;
                let mut teammate_puuids = Vec::new();

                for pt in participants {
                    let pt_tid = pt.get("teamId").and_then(|v| v.as_i64()).unwrap_or(100);
                    let pt_pid = pt.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_stats = pt.get("stats");
                    let pt_dmg = pt_stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);

                    if pt_tid == my_tid {
                        team_dmg += pt_dmg;
                        team_kills += pt_stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        if pt_dmg > team_max_dmg {
                            team_max_dmg = pt_dmg;
                            team_max_pid = pt_pid;
                        }
                        if pt_pid != target_pid.unwrap_or(-1) {
                            if let Some((t_puuid, _)) = ident_map.get(&pt_pid) {
                                if !t_puuid.is_empty() {
                                    teammate_puuids.push(t_puuid.clone());
                                }
                            }
                        }
                    }
                }

                let dmg_share_pct = if team_dmg > 0 { ((dmg as f64 / team_dmg as f64) * 1000.0).round() / 10.0 } else { 20.0 };
                let kp_pct = if team_kills > 0 { (((k + a) as f64 / team_kills as f64) * 1000.0).round() / 10.0 } else { 50.0 };
                let is_team_highest_dmg = target_pid == Some(team_max_pid);
                let is_mvp = is_win && is_team_highest_dmg;
                let is_svp = !is_win && is_team_highest_dmg;

                parsed_matches.push(ParsedMatchRecord {
                    game_id,
                    game_creation,
                    game_duration: duration_sec,
                    game_duration_str: duration_str,
                    queue_id,
                    mode_cn,
                    champion_id: cid,
                    champion_name: cname,
                    champion_avatar: cavatar,
                    win: is_win,
                    kills: k,
                    deaths: d,
                    assists: a,
                    kda_str,
                    kda_ratio,
                    kda: kda_ratio,
                    items: non_empty_items,
                    all_items,
                    cs: total_cs,
                    cs_per_min: cs_min,
                    damage: dmg,
                    damage_share_pct: dmg_share_pct,
                    damage_taken: taken,
                    damage_mitigated: mitig,
                    gold,
                    vision_score: vision,
                    kp_pct,
                    teammate_puuids,
                    is_mvp,
                    is_svp,
                    is_win,
                });
            }
        }

        if parsed_matches.is_empty() && ranked.is_none() {
            return None;
        }

        let record = CachedPlayerRecord {
            puuid: puuid.to_string(),
            summoner_name: if real_name.is_empty() { candidate_name.to_string() } else { real_name },
            tier_str,
            tier_color: get_tier_color(&raw_tier).to_string(),
            tier: raw_tier,
            lp,
            ranked_wins,
            ranked_losses,
            matches: parsed_matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({
                "puuid": puuid,
                "name": candidate_name,
                "level": 128,
                "profile_icon_id": 548
            }),
            raw_ranked: ranked.unwrap_or(json!({
                "solo": {
                    "tier": "UNRANKED",
                    "division": "",
                    "tier_str": "未定级",
                    "lp": 0,
                    "wins": 0,
                    "losses": 0,
                    "win_rate": null
                }
            })),
        };

        let mut guard = self.raw_cache.write().await;
        guard.insert(puuid.to_string(), record.clone());

        Some(record)
    }

    pub fn build_mock_player_profile(&self, slot_index: usize, champion_id: i32) -> CachedPlayerRecord {
        let my_names = ["狂傲天王", "野区萧炎", "中单老顽童", "暴风输出手", "金牌辅助包"];
        let name = my_names[slot_index % my_names.len()];

        let mock_cfgs: [(&str, &str, i32, f64, [bool; 5]); 5] = [
            ("EMERALD", "翡翠 III", 58, 0.58, [true, true, true, true, true]),
            ("DIAMOND", "钻石 IV", 15, 0.62, [true, true, false, true, true]),
            ("GOLD", "黄金 IV", 42, 0.52, [true, false, true, false, false]),
            ("SILVER", "白银 I", 88, 0.48, [false, false, false, true, false]),
            ("SILVER", "白银 II", 20, 0.44, [false, false, false, false, false]),
        ];
        let cfg = mock_cfgs[slot_index % mock_cfgs.len()];
        let cid = if champion_id > 0 { champion_id } else { 266 };

        let mut matches = Vec::new();
        let pool = ["剑魔", "盲僧", "塞拉斯", "卡莎", "阿狸"];
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);

        for mi in 0..20 {
            let is_win = cfg.4[mi % cfg.4.len()];
            let k = if is_win { 11 } else { 3 };
            let d = if is_win { 2 } else { 6 };
            let a = if is_win { 8 } else { 4 };
            let items = if is_win { vec![3031, 3006, 3072, 3036, 3026, 3139] } else { vec![3031, 3006, 1055, 0, 0, 0] };

            matches.push(ParsedMatchRecord {
                game_id: 99990000 + (slot_index as i64) * 100 + (mi as i64),
                game_creation: now_ms - (mi as i64) * 3600000,
                game_duration: 1820,
                game_duration_str: "30:20".to_string(),
                queue_id: 420,
                mode_cn: "排位赛(单双)".to_string(),
                champion_id: cid,
                champion_name: pool[mi % pool.len()].to_string(),
                champion_avatar: format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid),
                win: is_win,
                kills: k,
                deaths: d,
                assists: a,
                kda_str: format!("{}/{}/{}", k, d, a),
                kda_ratio: (k + a) as f64 / d.max(1) as f64,
                kda: (k + a) as f64 / d.max(1) as f64,
                items: items.clone(),
                all_items: items,
                cs: if is_win { 210 } else { 160 },
                cs_per_min: if is_win { 6.9 } else { 5.2 },
                damage: if is_win { 26500 } else { 14200 },
                damage_share_pct: if is_win { 28.5 } else { 18.2 },
                damage_taken: 18500,
                damage_mitigated: 12000,
                gold: if is_win { 14200 } else { 9500 },
                vision_score: 26,
                kp_pct: if is_win { 68.5 } else { 42.0 },
                teammate_puuids: (0..4).map(|x| format!("mock_mate_{}_{}", slot_index, x)).collect(),
                is_mvp: is_win && mi == 0,
                is_svp: !is_win && mi == 1,
                is_win,
            });
        }

        CachedPlayerRecord {
            puuid: format!("mock_puuid_{}", slot_index),
            summoner_name: name.to_string(),
            tier_str: cfg.1.to_string(),
            tier: cfg.0.to_string(),
            tier_color: get_tier_color(cfg.0).to_string(),
            lp: cfg.2,
            ranked_wins: (cfg.3 * 100.0).round() as i32,
            ranked_losses: ((1.0 - cfg.3) * 100.0).round() as i32,
            matches,
            fetched_at: Instant::now(),
            raw_summoner: json!({
                "puuid": format!("mock_puuid_{}", slot_index),
                "name": name,
                "gameName": name,
                "tagLine": "MOCK",
                "full_name": format!("{}#MOCK", name),
                "level": 128 + slot_index as i32 * 10,
                "profile_icon_id": 500 + slot_index as i32
            }),
            raw_ranked: json!({
                "solo": {
                    "tier": cfg.0,
                    "division": "III",
                    "tier_str": cfg.1,
                    "lp": cfg.2,
                    "wins": (cfg.3 * 100.0).round() as i32,
                    "losses": ((1.0 - cfg.3) * 100.0).round() as i32,
                    "has_losses": true,
                    "total_games": 100,
                    "win_rate": cfg.3
                }
            }),
        }
    }

    pub fn get_mock_ongoing_match(&self, db: &Database) -> OngoingMatchResponse {
        let blue_configs = [
            (0, "TOP", 266, "剑魔", "狂傲天王", "EMERALD", "翡翠 III", 58, 0.58, 4, 12),
            (1, "JUNGLE", 64, "盲僧", "野区萧炎", "DIAMOND", "钻石 IV", 15, 0.62, 4, 11),
            (2, "MID", 517, "塞拉斯", "中单老顽童", "GOLD", "黄金 IV", 42, 0.52, 4, 14),
            (3, "ADC", 145, "卡莎", "暴风输出手", "DIAMOND", "钻石 IV", 80, 0.65, 4, 7),
            (4, "SUPPORT", 111, "深海泰坦", "金牌辅助包", "SILVER", "白银 I", 88, 0.44, 4, 3),
        ];

        let red_configs = [
            (0, "TOP", 24, "贾克斯", "武器大师傅", "EMERALD", "翡翠 II", 36, 0.55, 4, 12),
            (1, "JUNGLE", 5, "赵信", "赵云在此", "GOLD", "黄金 I", 70, 0.51, 4, 11),
            (2, "MID", 103, "阿狸", "九尾妖仙", "MASTER", "大师", 82, 0.68, 4, 14),
            (3, "ADC", 81, "伊泽瑞尔", "探险家黄毛", "DIAMOND", "钻石 III", 24, 0.56, 4, 7),
            (4, "SUPPORT", 412, "锤石", "夺命钩魂手", "PLATINUM", "铂金 II", 65, 0.50, 4, 14),
        ];

        let make_cards = |cfgs: &[(usize, &str, i32, &str, &str, &str, &str, i32, f64, i32, i32)], tid: i32| -> Vec<OngoingPlayerCard> {
            cfgs.iter().map(|cfg| {
                let rec = self.build_mock_player_profile(cfg.0, cfg.2);
                let tactical = self.compute_tactical_profile(&rec, cfg.2, 20);
                let cname = db.resolve_hero_name(&cfg.2.to_string()).unwrap_or_else(|| cfg.3.to_string());
                let cavatar = format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cfg.2);

                OngoingPlayerCard {
                    slot_index: cfg.0,
                    team_id: tid,
                    is_local_player: tid == 100 && cfg.0 == 0,
                    puuid: format!("mock_puuid_{}_{}", tid, cfg.0),
                    summoner_name: cfg.4.to_string(),
                    full_name: format!("{}#MOCK", cfg.4),
                    level: 120 + cfg.0 as i32 * 12,
                    profile_icon_id: 500 + cfg.0 as i32,
                    position: cfg.1.to_string(),
                    champion_id: cfg.2,
                    champion_name: cname,
                    champion_avatar: cavatar,
                    spell1_id: cfg.9,
                    spell2_id: cfg.10,
                    ranked: rec.raw_ranked.clone(),
                    tactical_analysis: tactical,
                    matches: rec.matches,
                    premade_group: None,
                    premade_tag: None,
                    premade_color: None,
                    premade_bg: None,
                    premade_text: None,
                }
            }).collect()
        };

        let mut blue_team = make_cards(&blue_configs, 100);
        let mut red_team = make_cards(&red_configs, 200);

        // Simulate Duo Premade on Blue (Jungle + ADC)
        blue_team[1].premade_group = Some(1);
        blue_team[1].premade_tag = Some("双排车队".to_string());
        blue_team[1].premade_color = Some("#ec4899".to_string());
        blue_team[1].premade_bg = Some("rgba(236,72,153,0.18)".to_string());
        blue_team[1].premade_text = Some("#f472b6".to_string());
        blue_team[1].tactical_analysis.badges.insert(0, BadgeItem {
            text: "🚗 双排车队".to_string(),
            bg: "rgba(236,72,153,0.18)".to_string(),
            fg: "#f472b6".to_string(),
            desc: "与同队 ADC 为双排车队".to_string(),
        });

        blue_team[3].premade_group = Some(1);
        blue_team[3].premade_tag = Some("双排车队".to_string());
        blue_team[3].premade_color = Some("#ec4899".to_string());
        blue_team[3].premade_bg = Some("rgba(236,72,153,0.18)".to_string());
        blue_team[3].premade_text = Some("#f472b6".to_string());
        blue_team[3].tactical_analysis.badges.insert(0, BadgeItem {
            text: "🚗 双排车队".to_string(),
            bg: "rgba(236,72,153,0.18)".to_string(),
            fg: "#f472b6".to_string(),
            desc: "与同队打野为双排车队".to_string(),
        });

        // Simulate Trio Premade on Red (Top, Jungle, Mid)
        for r_idx in 0..3 {
            red_team[r_idx].premade_group = Some(2);
            red_team[r_idx].premade_tag = Some("三排车队".to_string());
            red_team[r_idx].premade_color = Some("#f59e0b".to_string());
            red_team[r_idx].premade_bg = Some("rgba(245,158,11,0.18)".to_string());
            red_team[r_idx].premade_text = Some("#fbbf24".to_string());
            red_team[r_idx].tactical_analysis.badges.insert(0, BadgeItem {
                text: "🚗 三排车队".to_string(),
                bg: "rgba(245,158,11,0.18)".to_string(),
                fg: "#fbbf24".to_string(),
                desc: "上中野三排连体车队".to_string(),
            });
        }

        OngoingMatchResponse {
            phase: "ChampSelect".to_string(),
            phase_cn: "选人模拟实战".to_string(),
            is_active: true,
            is_mock: true,
            chat_room_id: "mock_champ_select_chat_room".to_string(),
            blue_team: blue_team.clone(),
            red_team: red_team.clone(),
            my_team: blue_team,
            their_team: red_team,
            game_id: Some(99991234),
            message: None,
        }
    }

    pub async fn get_ongoing_match(&self, lcu: &crate::lcu::LcuClient, db: &Database) -> OngoingMatchResponse {
        let phase = lcu.get_gameflow_phase().await;

        if phase == "ChampSelect" {
            let session = lcu.get_champ_select_session().await;
            if session.is_null() {
                return self.get_mock_ongoing_match(db);
            }

            let chat_details = session.get("chatDetails");
            let chat_room_id = chat_details
                .and_then(|c| c.get("multiUserChatId").or_else(|| c.get("chatRoomId")).or_else(|| c.get("chatRoomName")))
                .and_then(|v| v.as_str())
                .or_else(|| session.get("chatRoomId").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            let my_raw = session.get("myTeam").and_then(|v| v.as_array());
            let their_raw = session.get("theirTeam").and_then(|v| v.as_array());

            let mut my_cards = Vec::new();
            if let Some(arr) = my_raw {
                let cids: Vec<i32> = arr.iter().map(|p| p.get("championId").or_else(|| p.get("championPickIntent")).and_then(|v| v.as_i64()).unwrap_or(0) as i32).collect();
                let is_locals: Vec<bool> = arr.iter().map(|p| p.get("is_local_player").and_then(|v| v.as_bool()).unwrap_or(false)).collect();
                let profiles = self.resolve_teammates(&session, &cids, &is_locals, lcu).await;

                for (idx, p) in arr.iter().enumerate() {
                    let cid = cids.get(idx).copied().unwrap_or(0);
                    let cname = if cid > 0 { db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid)) } else { "选择中...".to_string() };
                    let cavatar = if cid > 0 { format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid) } else { String::new() };
                    let prof = profiles.get(idx).cloned().flatten().unwrap_or_default();
                    let sname = p.get("summonerName").or_else(|| p.get("gameName")).and_then(|v| v.as_str()).unwrap_or("");
                    let name = if !prof.summoner_name.is_empty() { prof.summoner_name.clone() } else if !sname.is_empty() { sname.to_string() } else { format!("我方{}楼", idx + 1) };

                    let cached = self.get_player_data(&prof.puuid).await;
                    let matches = cached.map(|c| c.matches).unwrap_or_default();

                    my_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: 100,
                        is_local_player: is_locals.get(idx).copied().unwrap_or(false),
                        puuid: prof.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: format!("{}#CN", name),
                        level: 120,
                        profile_icon_id: 548,
                        position: p.get("assignedPosition").and_then(|v| v.as_str()).unwrap_or("").to_uppercase(),
                        champion_id: cid,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4) as i32,
                        spell2_id: p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(12) as i32,
                        ranked: json!({ "solo": { "tier": prof.tier, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                    });
                }
            }

            let mut their_cards = Vec::new();
            if let Some(arr) = their_raw {
                let cids: Vec<i32> = arr.iter().map(|p| p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32).collect();
                let profiles = self.resolve_opponents_champ_select(&session, &cids, lcu).await;

                for (idx, p) in arr.iter().enumerate() {
                    let cid = cids.get(idx).copied().unwrap_or(0);
                    let cname = if cid > 0 { db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid)) } else { "敌方英雄".to_string() };
                    let cavatar = if cid > 0 { format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid) } else { String::new() };
                    let prof = profiles.get(idx).cloned().flatten().unwrap_or_default();
                    let sname = p.get("summonerName").or_else(|| p.get("gameName")).and_then(|v| v.as_str()).unwrap_or("");
                    let name = if !prof.summoner_name.is_empty() { prof.summoner_name.clone() } else if !sname.is_empty() { sname.to_string() } else { format!("敌方{}楼", idx + 1) };

                    let cached = self.get_player_data(&prof.puuid).await;
                    let matches = cached.map(|c| c.matches).unwrap_or_default();

                    their_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: 200,
                        is_local_player: false,
                        puuid: prof.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: format!("{}#CN", name),
                        level: 120,
                        profile_icon_id: 500,
                        position: p.get("assignedPosition").and_then(|v| v.as_str()).unwrap_or("").to_uppercase(),
                        champion_id: cid,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4) as i32,
                        spell2_id: p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(14) as i32,
                        ranked: json!({ "solo": { "tier": prof.tier, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                    });
                }
            }

            detect_premade_teams(&mut my_cards);
            detect_premade_teams(&mut their_cards);

            OngoingMatchResponse {
                phase,
                phase_cn: "英雄选择阶段".to_string(),
                is_active: true,
                is_mock: false,
                chat_room_id,
                blue_team: my_cards.clone(),
                red_team: their_cards.clone(),
                my_team: my_cards,
                their_team: their_cards,
                game_id: None,
                message: None,
            }
        } else if phase == "GameStart" || phase == "InProgress" {
            let gf_session = lcu.get_gameflow_session().await;
            if let Some((my_slots, their_slots)) = self.resolve_gameflow_10_players(&gf_session, lcu).await {
                let parse_team = |slots: Vec<GameflowSlotInfo>, tid: i32| -> Vec<OngoingPlayerCard> {
                    slots.into_iter().enumerate().map(|(idx, s)| {
                        let cname = if s.champion_id > 0 { db.resolve_hero_name(&s.champion_id.to_string()).unwrap_or_else(|| s.summoner_name.clone()) } else { s.summoner_name.clone() };
                        let cavatar = if s.champion_id > 0 { format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", s.champion_id) } else { String::new() };
                        let prof = s.profile.unwrap_or_default();

                        OngoingPlayerCard {
                            slot_index: idx,
                            team_id: tid,
                            is_local_player: s.is_local_player,
                            puuid: s.puuid.clone(),
                            summoner_name: s.summoner_name.clone(),
                            full_name: format!("{}#CN", s.summoner_name),
                            level: 120,
                            profile_icon_id: 548,
                            position: s.position,
                            champion_id: s.champion_id,
                            champion_name: cname,
                            champion_avatar: cavatar,
                            spell1_id: 4,
                            spell2_id: 14,
                            ranked: json!({ "solo": { "tier": prof.tier, "tier_str": prof.tier_str, "lp": prof.lp } }),
                            tactical_analysis: prof,
                            matches: Vec::new(),
                            premade_group: None,
                            premade_tag: None,
                            premade_color: None,
                            premade_bg: None,
                            premade_text: None,
                        }
                    }).collect()
                };

                let mut my_cards = parse_team(my_slots, 100);
                let mut their_cards = parse_team(their_slots, 200);

                detect_premade_teams(&mut my_cards);
                detect_premade_teams(&mut their_cards);

                OngoingMatchResponse {
                    phase: phase.clone(),
                    phase_cn: if phase == "InProgress" { "正在游戏中".to_string() } else { "对局加载中".to_string() },
                    is_active: true,
                    is_mock: false,
                    chat_room_id: String::new(),
                    blue_team: my_cards.clone(),
                    red_team: their_cards.clone(),
                    my_team: my_cards,
                    their_team: their_cards,
                    game_id: gf_session.get("gameData").and_then(|g| g.get("gameId")).and_then(|v| v.as_i64()),
                    message: None,
                }
            } else {
                self.get_mock_ongoing_match(db)
            }
        } else {
            OngoingMatchResponse {
                phase,
                phase_cn: "未在对局中".to_string(),
                is_active: false,
                is_mock: false,
                chat_room_id: String::new(),
                blue_team: Vec::new(),
                red_team: Vec::new(),
                my_team: Vec::new(),
                their_team: Vec::new(),
                game_id: None,
                message: Some("当前未在英雄联盟对局或选人阶段，可点击【模拟实战】预览 10 人全景透视！".to_string()),
            }
        }
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
        let chat_room_id = chat_details
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
                        break;
                    }
                }
            }
        }

        // Build candidate list (STRICTLY EXCLUDING local player to prevent duplicate identity bug!)
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
