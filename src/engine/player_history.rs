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
    #[serde(default)]
    pub team_id: i64,
    pub spell1_id: i32,
    pub spell2_id: i32,
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
    pub division: String,
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
pub struct ParticipantPreviewItem {
    pub participant_id: i64,
    pub puuid: String,
    pub summoner_name: String,
    pub game_name: String,
    pub tag_line: String,
    pub team_id: i32,
    pub is_blue: bool,
    pub champion_id: i32,
    pub champion_name: String,
    pub champion_icon: String,
    pub is_win: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub spell1_id: i32,
    pub spell2_id: i32,
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
    pub team_id: i32,
    pub is_blue: bool,
    // Backward compat:
    pub is_win: bool,
    pub spell1_id: i32,
    pub spell2_id: i32,
    pub physical_damage: i64,
    pub magic_damage: i64,
    pub true_damage: i64,
    pub primary_rune: i32,
    pub secondary_rune: i32,
    pub participants_preview: Vec<ParticipantPreviewItem>,
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
    #[serde(default)]
    pub queue_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OngoingMatchResponse {
    pub phase: String,
    pub phase_cn: String,
    pub queue_id: i32,
    pub queue_name: String,
    pub is_my_team_blue: bool,
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
    pub raw_cache: Arc<RwLock<HashMap<String, CachedPlayerRecord>>>,
    pub sample_count: Arc<std::sync::atomic::AtomicI32>,
    cached_local_puuid: Arc<RwLock<Option<String>>>,
    cached_local_name: Arc<RwLock<Option<String>>>,
    database: Arc<RwLock<Option<Arc<Database>>>>,
    sgp_client: Arc<RwLock<Option<Arc<crate::sgp::SgpClient>>>>,
    pub unlinked_candidates: Arc<RwLock<Vec<(String, String)>>>,
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

#[derive(Debug, Clone)]
pub struct MatchParticipantInput {
    pub participant_id: i64,
    pub puuid: String,
    pub team_id: i64,
    pub is_win: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub damage: i64,
    pub damage_taken: i64,
    pub damage_mitigated: i64,
    pub gold: i64,
    pub cs: i32,
    pub vision_score: i32,
}

#[derive(Debug, Clone)]
pub struct MatchParticipantEvaluation {
    pub participant_id: i64,
    pub puuid: String,
    pub team_id: i64,
    pub score: f64,
    pub damage_share_pct: f64,
    pub damage_taken_share_pct: f64,
    pub kp_pct: f64,
    pub is_highest_damage: bool,
    pub is_mvp: bool,
    pub is_svp: bool,
}

pub fn evaluate_match_participants(
    duration_sec: i64,
    participants: &[MatchParticipantInput],
) -> Vec<MatchParticipantEvaluation> {
    if participants.is_empty() {
        return Vec::new();
    }

    let duration_min = (duration_sec as f64 / 60.0).max(1.0);
    let game_max_dmg = participants.iter().map(|p| p.damage).max().unwrap_or(0);

    let mut team_dmg: HashMap<i64, i64> = HashMap::new();
    let mut team_taken: HashMap<i64, i64> = HashMap::new();
    let mut team_kills: HashMap<i64, i32> = HashMap::new();
    let mut team_tank_eff: HashMap<i64, f64> = HashMap::new();

    for p in participants {
        *team_dmg.entry(p.team_id).or_insert(0) += p.damage;
        *team_taken.entry(p.team_id).or_insert(0) += p.damage_taken;
        *team_kills.entry(p.team_id).or_insert(0) += p.kills;
        let tank_eff = p.damage_taken as f64 + 0.5 * (p.damage_mitigated as f64);
        *team_tank_eff.entry(p.team_id).or_insert(0.0) += tank_eff;
    }

    struct Candidate {
        index: usize,
        score: f64,
        dmg_share: f64,
        dmg_share_pct: f64,
        taken_share_pct: f64,
        kda_ratio: f64,
        kp: f64,
        kp_pct: f64,
        deaths: i32,
        pid: i64,
        is_win: bool,
    }

    let mut candidates = Vec::with_capacity(participants.len());

    for (idx, p) in participants.iter().enumerate() {
        let t_dmg = team_dmg.get(&p.team_id).copied().unwrap_or(0);
        let t_taken = team_taken.get(&p.team_id).copied().unwrap_or(0);
        let t_kills = team_kills.get(&p.team_id).copied().unwrap_or(0);
        let t_eff = team_tank_eff.get(&p.team_id).copied().unwrap_or(0.0);

        let dmg_share = if t_dmg > 0 { p.damage as f64 / t_dmg as f64 } else { 0.2 };
        let dmg_share_pct = ((dmg_share * 1000.0).round() / 10.0).clamp(0.0, 100.0);

        let taken_share = if t_taken > 0 { p.damage_taken as f64 / t_taken as f64 } else { 0.2 };
        let taken_share_pct = ((taken_share * 1000.0).round() / 10.0).clamp(0.0, 100.0);

        let kda_ratio = (p.kills.max(0) as f64 + 0.8 * (p.assists.max(0) as f64)) / (p.deaths.max(1) as f64);
        let kp = if t_kills > 0 { (p.kills + p.assists) as f64 / t_kills as f64 } else { 0.2 };
        let kp_pct = ((kp * 1000.0).round() / 10.0).clamp(0.0, 100.0);

        let cs_per_min = p.cs.max(0) as f64 / duration_min;
        let my_tank_eff = p.damage_taken as f64 + 0.5 * (p.damage_mitigated as f64);
        let tank_share = if t_eff > 0.0 { my_tank_eff / t_eff } else { 0.2 };

        let s_dmg = 30.0 * (dmg_share / 0.20).max(0.0).powf(0.85);
        let s_kda = 25.0 * (kda_ratio / 4.0).clamp(0.0, 2.0);
        let s_kp = 20.0 * (kp / 0.55).clamp(0.0, 1.5);
        let s_econ = 15.0 * (cs_per_min / 7.5).clamp(0.0, 1.4);
        let s_tank = 10.0 * (tank_share / 0.20).clamp(0.0, 2.0);
        let s_vis = 5.0 * (p.vision_score as f64 / (duration_min * 1.2).max(1.0)).clamp(0.0, 2.0);

        let mut total_score = s_dmg + s_kda + s_kp + s_econ + s_tank + s_vis;
        if !total_score.is_finite() {
            total_score = 50.0;
        }

        let safe_dmg_share = if dmg_share.is_finite() && dmg_share >= 0.0 { dmg_share } else { 0.2 };
        let safe_dmg_share_pct = if dmg_share_pct.is_finite() && dmg_share_pct >= 0.0 { dmg_share_pct } else { 20.0 };
        let safe_taken_share_pct = if taken_share_pct.is_finite() && taken_share_pct >= 0.0 { taken_share_pct } else { 20.0 };
        let safe_kda_ratio = if kda_ratio.is_finite() && kda_ratio >= 0.0 { kda_ratio } else { 0.0 };
        let safe_kp = if kp.is_finite() && kp >= 0.0 { kp } else { 0.2 };
        let safe_kp_pct = if kp_pct.is_finite() && kp_pct >= 0.0 { kp_pct } else { 20.0 };

        candidates.push(Candidate {
            index: idx,
            score: total_score,
            dmg_share: safe_dmg_share,
            dmg_share_pct: safe_dmg_share_pct,
            taken_share_pct: safe_taken_share_pct,
            kda_ratio: safe_kda_ratio,
            kp: safe_kp,
            kp_pct: safe_kp_pct,
            deaths: p.deaths,
            pid: p.participant_id,
            is_win: p.is_win,
        });
    }

    let compare_candidates = |a: &Candidate, b: &Candidate| -> std::cmp::Ordering {
        if (a.score - b.score).abs() >= 0.01 {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        } else if (a.dmg_share - b.dmg_share).abs() >= 0.001 {
            b.dmg_share.partial_cmp(&a.dmg_share).unwrap_or(std::cmp::Ordering::Equal)
        } else if (a.kda_ratio - b.kda_ratio).abs() >= 0.01 {
            b.kda_ratio.partial_cmp(&a.kda_ratio).unwrap_or(std::cmp::Ordering::Equal)
        } else if (a.kp - b.kp).abs() >= 0.001 {
            b.kp.partial_cmp(&a.kp).unwrap_or(std::cmp::Ordering::Equal)
        } else if a.deaths != b.deaths {
            a.deaths.cmp(&b.deaths)
        } else {
            a.pid.cmp(&b.pid)
        }
    };

    let mut win_candidates: Vec<&Candidate> = candidates.iter().filter(|c| c.is_win).collect();
    win_candidates.sort_by(|a, b| compare_candidates(a, b));
    let best_win_idx = win_candidates.first().map(|c| c.index);

    let mut loss_candidates: Vec<&Candidate> = candidates.iter().filter(|c| !c.is_win).collect();
    loss_candidates.sort_by(|a, b| compare_candidates(a, b));
    let best_loss_idx = loss_candidates.first().map(|c| c.index);

    let mut evaluations = Vec::with_capacity(participants.len());
    for (idx, p) in participants.iter().enumerate() {
        let cand = &candidates[idx];
        let is_highest_dmg = p.damage == game_max_dmg && p.damage > 0;
        let is_mvp = Some(idx) == best_win_idx;
        let is_svp = Some(idx) == best_loss_idx;

        evaluations.push(MatchParticipantEvaluation {
            participant_id: p.participant_id,
            puuid: p.puuid.clone(),
            team_id: p.team_id,
            score: (cand.score * 10.0).round() / 10.0,
            damage_share_pct: cand.dmg_share_pct,
            damage_taken_share_pct: cand.taken_share_pct,
            kp_pct: cand.kp_pct,
            is_highest_damage: is_highest_dmg,
            is_mvp,
            is_svp,
        });
    }

    evaluations
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

    let safe_clamp = |v: f64| -> i32 {
        if v.is_nan() || v.is_infinite() {
            50
        } else {
            (v as i32).clamp(25, 99)
        }
    };

    let dmg_score = safe_clamp(avg_dmg / 350.0 + avg_dmg_share * 1.5);
    let kda_score = safe_clamp(avg_kda * 18.0);
    let kp_score = safe_clamp(avg_kp * 1.35);
    let surv_score = safe_clamp(100.0 - avg_deaths * 9.5);
    let econ_score = safe_clamp(avg_cs_min * 10.0 + avg_gold / 260.0);
    let vis_score = safe_clamp(avg_vis * 2.8);

    map.insert("damage".to_string(), dmg_score);
    map.insert("kda".to_string(), kda_score);
    map.insert("kp".to_string(), kp_score);
    map.insert("survival".to_string(), surv_score);
    map.insert("economy".to_string(), econ_score);
    map.insert("vision".to_string(), vis_score);
    map
}

fn get_combinations(indices: &[usize], k: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    if k == 0 || k > indices.len() {
        return result;
    }
    fn backtrack(indices: &[usize], start: usize, k: usize, current: &mut Vec<usize>, result: &mut Vec<Vec<usize>>) {
        if current.len() == k {
            result.push(current.clone());
            return;
        }
        for i in start..indices.len() {
            current.push(indices[i]);
            backtrack(indices, i + 1, k, current, result);
            current.pop();
        }
    }
    let mut current = Vec::new();
    backtrack(indices, 0, k, &mut current, &mut result);
    result
}

pub fn detect_premade_teams_with_queue(players: &mut [OngoingPlayerCard], queue_id: i32) {
    if players.len() < 2 {
        return;
    }
    let n = players.len();
    let mut weight: Vec<Vec<usize>> = vec![vec![0; n]; n];

    for i in 0..n {
        let matches_i = &players[i].matches;
        let mut teammate_counts_i: HashMap<String, usize> = HashMap::new();
        let has_teammates_i = matches_i.iter().any(|m| !m.teammate_puuids.is_empty());

        for m in matches_i {
            let mut seen_in_match = std::collections::HashSet::new();
            for t in &m.teammate_puuids {
                if seen_in_match.insert(t) {
                    *teammate_counts_i.entry(t.clone()).or_insert(0) += 1;
                }
            }
        }

        for j in (i + 1)..n {
            // Verify players are not opponents in current match (if team_id is known)
            if players[i].team_id > 0 && players[j].team_id > 0 && players[i].team_id != players[j].team_id {
                continue;
            }

            let puuid_i = &players[i].puuid;
            let puuid_j = &players[j].puuid;
            let matches_j = &players[j].matches;
            let has_teammates_j = matches_j.iter().any(|m| !m.teammate_puuids.is_empty());

            let shared_by_puuid_i = if !puuid_j.is_empty() {
                teammate_counts_i.get(puuid_j).copied().unwrap_or(0)
            } else {
                0
            };

            let shared_by_puuid_j = if !puuid_i.is_empty() {
                matches_j.iter().filter(|m| m.teammate_puuids.contains(puuid_i)).count()
            } else {
                0
            };

            let shared_as_teammates = shared_by_puuid_i.max(shared_by_puuid_j);

            let total_shared = if has_teammates_i || has_teammates_j {
                // Strictly verify players were on the same team
                shared_as_teammates
            } else {
                // Fallback for synthetic/canned test fixtures where teammate_puuids is omitted
                // Strictly verify players were on the SAME team in past matches
                let mut map_j: HashMap<i64, i32> = HashMap::new();
                for m in matches_j {
                    if m.game_id > 0 {
                        map_j.insert(m.game_id, m.team_id);
                    }
                }
                let mut count = 0;
                for m in matches_i {
                    if m.game_id > 0 {
                        if let Some(&tid_j) = map_j.get(&m.game_id) {
                            if (m.team_id == 0 && tid_j == 0) || (m.team_id > 0 && m.team_id == tid_j) {
                                count += 1;
                            }
                        }
                    }
                }
                count
            };

            // STRICT REQUIREMENT: >= 2 games together to eliminate 1-game accidental matchmaking false positive!
            if total_shared >= 2 {
                weight[i][j] = total_shared;
                weight[j][i] = total_shared;
            }
        }
    }

    // Party size limit: Solo/Duo queue (420) strictly caps premade size at 2 (maximum duo)
    let max_party_size = if queue_id == 420 { 2 } else { 5 };

    // Group player indices by team_id
    let mut team_groups: HashMap<i32, Vec<usize>> = HashMap::new();
    for i in 0..n {
        team_groups.entry(players[i].team_id).or_default().push(i);
    }

    let mut sorted_teams: Vec<i32> = team_groups.keys().copied().collect();
    sorted_teams.sort_unstable();

    struct CandidateClique {
        size: usize,
        total_weight: usize,
        members: Vec<usize>,
    }

    let mut components = Vec::new();

    for tid in sorted_teams {
        let indices = team_groups.remove(&tid).unwrap_or_default();
        if indices.len() < 2 {
            continue;
        }

        let max_k = indices.len().min(max_party_size);
        let mut candidate_cliques = Vec::new();

        for s in (2..=max_k).rev() {
            let combs = get_combinations(&indices, s);
            for comb in combs {
                let mut is_clique = true;
                let mut total_w = 0;
                for a in 0..comb.len() {
                    for b in (a + 1)..comb.len() {
                        let w = weight[comb[a]][comb[b]];
                        if w < 2 {
                            is_clique = false;
                            break;
                        }
                        total_w += w;
                    }
                    if !is_clique {
                        break;
                    }
                }
                if is_clique {
                    candidate_cliques.push(CandidateClique {
                        size: s,
                        total_weight: total_w,
                        members: comb,
                    });
                }
            }
        }

        // Sort candidates: larger size first, then higher co-play weight, then deterministic member ordering
        candidate_cliques.sort_by(|a, b| {
            b.size.cmp(&a.size)
                .then_with(|| b.total_weight.cmp(&a.total_weight))
                .then_with(|| a.members.cmp(&b.members))
        });

        let mut used = std::collections::HashSet::new();
        for cand in candidate_cliques {
            if cand.members.iter().all(|m| !used.contains(m)) {
                for &m in &cand.members {
                    used.insert(m);
                }
                components.push(cand.members);
            }
        }
    }

    components.sort_by_key(|c| c.first().copied().unwrap_or(0));

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

pub fn detect_premade_teams(players: &mut [OngoingPlayerCard]) {
    let queue_id = players.iter().map(|p| p.queue_id).find(|&q| q > 0).unwrap_or(0);
    detect_premade_teams_with_queue(players, queue_id);
}

pub fn queue_id_to_mode_cn(queue_id: i32) -> &'static str {
    match queue_id {
        420 => "排位赛(单双)",
        440 => "排位赛(灵活)",
        450 | 860 => "极地大乱斗",
        400 => "匹配模式",
        430 => "匹配模式(自选)",
        490 => "快速游戏",
        1700 | 1710 => "斗魂竞技场",
        1090..=1160 => "云顶之弈",
        700 => "克隆大作战",
        900 | 1010 | 1020 | 1900 => "无限火力",
        1300 => "极限闪击",
        1800..=1840 => "无尽狂潮",
        830..=850 | 870..=890 => "人机对战",
        0 => "自定义对局",
        2000..=2020 => "训练模式",
        _ => "其他模式",
    }
}

pub fn infer_typical_position(champion_id: i32) -> &'static str {
    match champion_id {
        // TOP
        14 | 266 | 24 | 122 | 86 | 114 | 164 | 92 | 58 | 39 | 98 | 54 | 516 | 82 | 875 | 150 | 897 | 17 | 6 | 31 | 75 | 420 | 41 | 126 | 887 | 78 | 133 | 240 => "TOP",
        // JUNGLE
        64 | 141 | 5 | 59 | 11 | 254 | 56 | 120 | 234 | 104 | 245 | 60 | 28 | 876 | 203 | 19 | 35 | 154 | 32 | 113 | 107 | 121 | 76 | 200 | 233 | 77 | 421 => "JUNGLE",
        // MID
        8 | 103 | 517 | 157 | 777 | 238 | 55 | 134 | 112 | 61 | 7 | 711 | 84 | 4 | 45 | 1 | 38 | 91 | 910 | 800 | 127 | 136 | 268 | 3 | 69 => "MID",
        // ADC
        51 | 145 | 81 | 222 | 67 | 202 | 22 | 360 | 236 | 18 | 21 | 498 | 29 | 119 | 523 | 221 | 15 | 110 | 96 | 429 | 901 => "ADC",
        // SUPPORT
        412 | 111 | 53 | 89 | 117 | 267 | 350 | 40 | 555 | 497 | 16 | 25 | 99 | 235 | 43 | 201 | 12 | 432 | 37 | 526 | 44 | 902 | 888 | 143 => "SUPPORT",
        _ => "",
    }
}

pub fn assign_team_positions(slots: &mut [GameflowSlotInfo]) {
    let standard_order = ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"];
    let mut assigned: Vec<Option<&'static str>> = vec![None; slots.len()];
    let mut used_positions: std::collections::HashSet<&'static str> = std::collections::HashSet::new();

    // Pass 1: Explicit selectedPosition from LCU (HIGHEST PRIORITY)
    for (i, s) in slots.iter().enumerate() {
        let pos_candidate = match s.position.to_uppercase().as_str() {
            "TOP" => Some("TOP"),
            "JUNGLE" => Some("JUNGLE"),
            "MID" | "MIDDLE" => Some("MID"),
            "ADC" | "BOT" | "BOTTOM" => Some("ADC"),
            "SUPPORT" | "SUP" | "UTILITY" => Some("SUPPORT"),
            _ => None,
        };
        if let Some(pos) = pos_candidate {
            if !used_positions.contains(pos) {
                assigned[i] = Some(pos);
                used_positions.insert(pos);
            }
        }
    }

    // Pass 2: Fallback for unassigned / blind pick modes: player carrying Smite (spell 11) gets JUNGLE if not yet taken
    if !used_positions.contains("JUNGLE") {
        for (i, s) in slots.iter().enumerate() {
            if assigned[i].is_none() && (s.spell1_id == 11 || s.spell2_id == 11) {
                assigned[i] = Some("JUNGLE");
                used_positions.insert("JUNGLE");
                break;
            }
        }
    }

    // Pass 3: Champion typical position
    for (i, s) in slots.iter().enumerate() {
        if assigned[i].is_some() { continue; }
        let typ = infer_typical_position(s.champion_id);
        if !typ.is_empty() && !used_positions.contains(typ) {
            assigned[i] = Some(typ);
            used_positions.insert(typ);
        }
    }

    // Pass 4: Fill remaining from unused standard positions
    for i in 0..slots.len() {
        if assigned[i].is_none() {
            if let Some(&rem) = standard_order.iter().find(|p| !used_positions.contains(*p)) {
                assigned[i] = Some(rem);
                used_positions.insert(rem);
            } else {
                assigned[i] = Some(standard_order[i % 5]);
            }
        }
    }

    for (i, s) in slots.iter_mut().enumerate() {
        let final_pos = assigned[i].unwrap_or("MID");
        s.position = final_pos.to_string();
        s.position_cn = match final_pos {
            "TOP" => "上单",
            "JUNGLE" => "打野",
            "MID" => "中单",
            "ADC" => "射手",
            "SUPPORT" => "辅助",
            _ => "中单",
        }.to_string();
    }
}

pub fn is_valid_match_id(id: i64) -> bool {
    // League game IDs must be within valid numeric boundaries (typically 7 to 11 digits)
    if id < 1_000_000 || id > 99_999_999_999 {
        return false;
    }
    // Filter out Unix second-level timestamps:
    // e.g., 1_600_000_000 to 1_850_000_000 (covers 1_700_000_000..1_800_000_000)
    if (1_600_000_000..=1_850_000_000).contains(&id) {
        return false;
    }
    true
}

impl PlayerHistoryManager {
    pub fn new() -> Self {
        Self {
            raw_cache: Arc::new(RwLock::new(HashMap::new())),
            sample_count: Arc::new(std::sync::atomic::AtomicI32::new(20)),
            cached_local_puuid: Arc::new(RwLock::new(None)),
            cached_local_name: Arc::new(RwLock::new(None)),
            database: Arc::new(RwLock::new(None)),
            sgp_client: Arc::new(RwLock::new(None)),
            unlinked_candidates: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_unlinked_candidates(&self) -> Vec<(String, String)> {
        let guard = self.unlinked_candidates.read().await;
        guard.clone()
    }

    pub async fn set_database(&self, db: Arc<Database>) {
        let mut guard = self.database.write().await;
        *guard = Some(db);
    }

    pub async fn get_database(&self) -> Option<Arc<Database>> {
        let guard = self.database.read().await;
        guard.clone()
    }

    pub async fn set_sgp_client(&self, sgp: Arc<crate::sgp::SgpClient>) {
        let mut guard = self.sgp_client.write().await;
        *guard = Some(sgp);
    }

    pub async fn get_sgp_client(&self) -> Option<Arc<crate::sgp::SgpClient>> {
        let guard = self.sgp_client.read().await;
        guard.clone()
    }

    pub fn set_sample_match_count(&self, count: i32) {
        let valid = [10, 20, 30, 40, 50, 100, 200];
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
        let effective_max_age = if rec.matches.is_empty() {
            max_age.min(Duration::from_secs(10))
        } else {
            max_age
        };
        if rec.fetched_at.elapsed() <= effective_max_age {
            Some(rec.clone())
        } else {
            None
        }
    }

    pub async fn get_cached_local_puuid(&self) -> Option<String> {
        self.cached_local_puuid.read().await.clone()
    }

    pub async fn get_cached_local_name(&self) -> Option<String> {
        self.cached_local_name.read().await.clone()
    }

    pub async fn clear_cache(&self) {
        self.raw_cache.write().await.clear();
    }

    pub async fn clear_local_summoner(&self) {
        let mut p = self.cached_local_puuid.write().await;
        *p = None;
        let mut n = self.cached_local_name.write().await;
        *n = None;
        self.clear_cache().await;
    }

    pub async fn set_cached_local_summoner(&self, puuid: &str, name: &str) {
        let prev = {
            let guard = self.cached_local_puuid.read().await;
            guard.clone()
        };
        if let Some(ref old_p) = prev {
            if old_p != puuid {
                self.clear_cache().await;
            }
        }
        {
            let mut p_guard = self.cached_local_puuid.write().await;
            *p_guard = Some(puuid.to_string());
        }
        {
            let mut n_guard = self.cached_local_name.write().await;
            *n_guard = Some(name.to_string());
        }
    }

    pub async fn sync_local_summoner(
        &self,
        lcu: &crate::lcu::LcuClient,
    ) -> (Option<String>, Option<String>) {
        if let Some(cur) = lcu.get_current_summoner().await {
            if let Some(p) = cur.get("puuid").and_then(|v| v.as_str()) {
                if !p.is_empty() {
                    let n = cur
                        .get("gameName")
                        .or_else(|| cur.get("displayName"))
                        .or_else(|| cur.get("summonerName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let prev_puuid = {
                        let guard = self.cached_local_puuid.read().await;
                        guard.clone()
                    };

                    if let Some(ref old_p) = prev_puuid {
                        if old_p != p {
                            // Account switch detected! Clear cache immediately so new account
                            // is not treated as a teammate or ghost duplicate.
                            self.clear_cache().await;
                        }
                    }

                    {
                        let mut p_guard = self.cached_local_puuid.write().await;
                        *p_guard = Some(p.to_string());
                    }
                    {
                        let mut n_guard = self.cached_local_name.write().await;
                        *n_guard = Some(n.to_string());
                    }
                    return (Some(p.to_string()), Some(n.to_string()));
                }
            }
        }

        let p = self.cached_local_puuid.read().await.clone();
        let n = self.cached_local_name.read().await.clone();
        (p, n)
    }

    pub async fn fetch_chat_participants_with_retry(
        &self,
        lcu: &crate::lcu::LcuClient,
        room_id: Option<&str>,
        local_puuid: Option<&str>,
        needed_count: usize,
    ) -> Vec<Value> {
        let backoff_delays = [200, 400, 800];
        let mut target_room = room_id.filter(|s| !s.is_empty()).map(|s| s.to_string());
        let mut participants = Vec::new();

        for attempt in 0..=3 {
            // If room_id is unknown, attempt to locate champ-select room from active conversations
            if target_room.is_none() {
                let convs = lcu.get_chat_conversations().await;
                for c in convs {
                    let c_type = c.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    let c_id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    if c_type == "championSelect" || c_id.contains("champ-select") {
                        target_room = Some(c_id.to_string());
                        break;
                    }
                }
            }

            if let Some(ref r_id) = target_room {
                let parts = lcu.get_chat_conversation_participants(r_id).await;
                if !parts.is_empty() {
                    participants = parts;
                    // Check if enough non-local teammates have arrived
                    let valid_candidates_count = participants
                        .iter()
                        .filter(|p| {
                            let pid = p
                                .get("id")
                                .or_else(|| p.get("puuid"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if pid.is_empty() {
                                return false;
                            }
                            if let Some(lp) = local_puuid {
                                if pid == lp {
                                    return false;
                                }
                            }
                            true
                        })
                        .count();

                    if valid_candidates_count >= needed_count && needed_count > 0 {
                        // All expected teammates have joined!
                        break;
                    }
                }
            }

            // Retry with exponential backoff if teammates haven't fully joined yet
            if attempt < 3 {
                tokio::time::sleep(Duration::from_millis(backoff_delays[attempt])).await;
            }
        }

        participants
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

        let mut champ_counts: HashMap<i32, (i32, i32, f64, f64, f64, String)> = HashMap::new(); // cid -> (games, wins, kills, deaths, assists, name)

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
                let entry = champ_counts.entry(m.champion_id).or_insert((0, 0, 0.0, 0.0, 0.0, m.champion_name.clone()));
                entry.0 += 1;
                if m.win {
                    entry.1 += 1;
                }
                entry.2 += m.kills as f64;
                entry.3 += m.deaths as f64;
                entry.4 += m.assists as f64;
            }
        }

        let m_losses = m_count - m_wins;
        let recent_dots: Vec<bool> = all_matches.iter().take(5).map(|m| m.win).collect();

        let recent_wr = if m_count > 0 {
            m_wins as f64 / m_count as f64
        } else {
            0.0
        };
        let recent_win_rate_pct = if m_count > 0 { (recent_wr * 100.0).round() as i32 } else { 0 };

        let total_ranked = record.ranked_wins + record.ranked_losses;
        // Fix 100% win rate / zero losses bug:
        let overall_win_rate = if total_ranked > 0 {
            Some(record.ranked_wins as f64 / total_ranked as f64)
        } else if m_count > 0 {
            Some(recent_wr)
        } else {
            None
        };
        let overall_win_rate_pct = overall_win_rate.map(|w| (w * 100.0).round() as i32).unwrap_or(0);

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
            if cur_g >= 3 && cur_wr_pct >= 60 {
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

        let is_testing_champ = current_champion_id > 0 && cur_g == 0 && m_count >= 4;

        if current_champion_id > 0 && cur_g >= 3 && cur_wr_pct >= 60 {
            badges.push(BadgeItem {
                text: format!("👑 绝活({}%)", cur_wr_pct),
                bg: "#312e81".to_string(),
                fg: "#a5b4fc".to_string(),
                desc: format!("绝活认证！该英雄近 {} 场实战胜率高达 {}%！", cur_g, cur_wr_pct),
            });
        } else if is_testing_champ {
            badges.push(BadgeItem {
                text: "⚠️ 试英雄".to_string(),
                bg: "#451a03".to_string(),
                fg: "#fbbf24".to_string(),
                desc: "近期无该英雄胜场记录，疑似排位试英雄或补位练习。".to_string(),
            });
        }

        // Smurf requires high recent win rate, solid KDA, sufficient games, and no loss streak!
        // A player with low recent win rate or testing champion CANNOT be a smurf.
        let is_smurf = !is_testing_champ
            && streak_type != "loss"
            && recent_wr >= 0.65
            && m_count >= 5
            && m_wins > m_losses
            && (avg_kda_ratio >= 3.8 || (overall_win_rate.unwrap_or(0.0) >= 0.65 && total_ranked >= 15));

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
        sorted_champs.sort_by(|a, b| b.1.0.cmp(&a.1.0).then_with(|| b.1.1.cmp(&a.1.1)).then_with(|| a.0.cmp(&b.0)));

        let top_champs: Vec<TopChampItem> = sorted_champs.iter().take(3).map(|(cid, info)| {
            let g = info.0;
            let w = info.1;
            let kda = if info.3 > 0.0 { (info.2 + info.4) / info.3 } else { info.2 + info.4 };
            TopChampItem {
                champion_id: *cid,
                champion_name: info.5.clone(),
                games: g,
                win_rate: if g > 0 { w as f64 / g as f64 } else { 0.5 },
                win_rate_pct: if g > 0 { ((w as f64 / g as f64) * 100.0).round() as i32 } else { 50 },
                kda: (kda * 10.0).round() / 10.0,
            }
        }).collect();

        let top_champions = top_champs.iter().map(|tc| tc.champion_id.to_string()).collect();

        let division = record.raw_ranked
            .get("solo")
            .and_then(|s| s.get("division"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        PlayerTacticalProfile {
            tier: record.tier.clone(),
            division,
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

    pub fn is_valid_match_id(id: i64) -> bool {
        is_valid_match_id(id)
    }

    pub fn record_game_id(&self, game_id: i64) {
        if !is_valid_match_id(game_id) {
            return;
        }
        static RECORD_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = RECORD_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

        let cache_path = std::path::Path::new("data/cache/encountered_games.json");
        let mut list: Vec<i64> = if let Ok(c) = std::fs::read_to_string(cache_path) {
            serde_json::from_str(&c).unwrap_or_default()
        } else {
            Vec::new()
        };
        if !list.contains(&game_id) {
            list.push(game_id);
            if list.len() > 1000 {
                list.remove(0);
            }
            if let Some(parent) = cache_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(cache_path, serde_json::to_string(&list).unwrap_or_default());
        }
    }

    pub async fn discover_game_ids(&self) -> Vec<i64> {
        tokio::task::spawn_blocking(|| {
            let mut gids = std::collections::BTreeSet::new();

            // 1. Read from local cache file
            let cache_paths = ["data/cache/encountered_games.json", "cache/encountered_games.json"];
            for cp in cache_paths {
                if let Ok(content) = std::fs::read_to_string(cp) {
                    if let Ok(arr) = serde_json::from_str::<Vec<i64>>(&content) {
                        for id in arr {
                            if is_valid_match_id(id) {
                                gids.insert(id);
                            }
                        }
                    }
                }
            }

            // 2. Scan %APPDATA%/league-akari/LeagueAkari.db with strict prefix pattern
            static PREFIX_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
            let re = PREFIX_RE.get_or_init(|| {
                regex::Regex::new(r#"(?i)(?:gameId|GameId|EndOfGameStats_GameId|/games/)[\"':\s=]*(\d{7,11})"#).unwrap()
            });

            if let Ok(appdata) = std::env::var("APPDATA") {
                let akari_db = std::path::Path::new(&appdata).join("league-akari").join("LeagueAkari.db");
                if akari_db.exists() {
                    if let Ok(bytes) = std::fs::read(&akari_db) {
                        let s = String::from_utf8_lossy(&bytes);
                        for cap in re.captures_iter(&s) {
                            if let Some(m) = cap.get(1) {
                                if let Ok(num) = m.as_str().parse::<i64>() {
                                    if is_valid_match_id(num) {
                                        gids.insert(num);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Scan recent LeagueClient log files across common drives with lossy decoding (handles GBK/CP936 logs)
            let potential_log_dirs = [
                "F:\\wegameapps\\英雄联盟（含经典模式）\\Game\\Logs\\LeagueClient Logs",
                "D:\\wegameapps\\英雄联盟（含经典模式）\\Game\\Logs\\LeagueClient Logs",
                "C:\\wegameapps\\英雄联盟（含经典模式）\\Game\\Logs\\LeagueClient Logs",
                "E:\\wegameapps\\英雄联盟（含经典模式）\\Game\\Logs\\LeagueClient Logs",
                "G:\\wegameapps\\英雄联盟（含经典模式）\\Game\\Logs\\LeagueClient Logs",
            ];
            for dir in potential_log_dirs {
                let p = std::path::Path::new(dir);
                if p.exists() {
                    if let Ok(entries) = std::fs::read_dir(p) {
                        let mut log_files: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
                            .collect();
                        log_files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
                        for file in log_files.iter().rev().take(40) {
                            if let Ok(bytes) = std::fs::read(file.path()) {
                                let content = String::from_utf8_lossy(&bytes);
                                for cap in re.captures_iter(&content) {
                                    if let Some(m) = cap.get(1) {
                                        if let Ok(num) = m.as_str().parse::<i64>() {
                                            if is_valid_match_id(num) {
                                                gids.insert(num);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut sorted: Vec<i64> = gids.into_iter().collect();
            sorted.sort_unstable_by(|a, b| b.cmp(a)); // Descending: newest IDs first

            // Persist discovered valid game IDs to data/cache/encountered_games.json
            if !sorted.is_empty() {
                let _ = std::fs::create_dir_all("data/cache");
                let _ = std::fs::write("data/cache/encountered_games.json", serde_json::to_string(&sorted).unwrap_or_default());
            }

            sorted
        })
        .await
        .unwrap_or_default()
    }

    pub async fn fetch_and_cache_player(
        &self,
        puuid: &str,
        candidate_name: &str,
        lcu: &crate::lcu::LcuClient,
        beg_index: usize,
        end_index: usize,
    ) -> Option<CachedPlayerRecord> {
        if puuid.is_empty() {
            return None;
        }

        let (beg_index, end_index) = (beg_index, end_index.max(beg_index));
        let count = (end_index + 1).saturating_sub(beg_index).clamp(1, 200);
        let end_index = beg_index + count - 1;

        // 1. Fetch ranked stats
        let ranked = lcu.get_player_ranked_stats(puuid).await;
        let mut tier_str = "未定级".to_string();
        let mut raw_tier = "UNRANKED".to_string();
        let mut raw_division = String::new();
        let mut lp = 0i32;
        let mut ranked_wins = 0i32;
        let mut ranked_losses = 0i32;

        if let Some(r) = &ranked {
            let queues = r.get("queueMap").and_then(|q| q.as_object());
            let solo = queues.and_then(|qm| qm.get("RANKED_SOLO_5x5"));
            if let Some(s) = solo {
                let t = s.get("tier").and_then(|v| v.as_str()).unwrap_or("");
                let div = s.get("division").and_then(|v| v.as_str()).unwrap_or("");
                raw_division = if div != "NA" && div != "NONE" { div.to_string() } else { String::new() };
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

        // Fetch summoner basic profile to ensure non-empty name, tagLine, icon and level
        let mut real_name = candidate_name.to_string();
        let mut tag_line = String::new();
        let mut level = 0i32;
        let mut icon_id = 0i32;

        if let Some(s_info) = lcu.get_summoner_by_puuid(puuid).await {
            if let Some(gn) = s_info.get("gameName").or_else(|| s_info.get("displayName")).or_else(|| s_info.get("summonerName")).and_then(|v| v.as_str()) {
                if !gn.is_empty() {
                    real_name = gn.to_string();
                }
            }
            if let Some(tl) = s_info.get("tagLine").and_then(|v| v.as_str()) {
                if !tl.is_empty() {
                    tag_line = tl.to_string();
                }
            }
            if let Some(lvl) = s_info.get("summonerLevel").and_then(|v| v.as_i64()) {
                level = lvl as i32;
            }
            if let Some(ico) = s_info.get("profileIconId").and_then(|v| v.as_i64()) {
                icon_id = ico as i32;
            }
        }

        // 2. Fetch match history with dynamic bounds (up to 200 matches)
        let history = lcu.get_player_match_history(puuid, beg_index, end_index).await;
        let mut parsed_matches = Vec::new();

        let mut raw_games: Vec<Value> = Vec::new();

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
                raw_games.push(g.clone());
            }
        }

        // Fallback for CN server where /products/lol/{puuid}/matches returns 500:
        // Tier 1 Fallback: Query remote SGP Match History Summary directly
        if raw_games.is_empty() {
            if let Some(sgp) = self.get_sgp_client().await {
                if let Ok(sgp_val) = sgp.get_match_history_summary(puuid, beg_index, count).await {
                    let empty_vec = Vec::new();
                    let games = sgp_val.get("games")
                        .and_then(|g| {
                            if let Some(arr) = g.as_array() {
                                Some(arr)
                            } else {
                                g.get("games").and_then(|v| v.as_array())
                            }
                        })
                        .or_else(|| sgp_val.get("matches").and_then(|v| v.as_array()))
                        .or_else(|| sgp_val.get("data").and_then(|d| d.get("games").and_then(|g| g.as_array()).or_else(|| d.as_array())))
                        .or_else(|| sgp_val.as_array())
                        .unwrap_or(&empty_vec);

                    for g in games {
                        raw_games.push(g.clone());
                    }
                }
            }
        }

        // Tier 2 Fallback: Scan locally discovered match IDs from logs & DB
        if raw_games.is_empty() {
            let discovered_gids = self.discover_game_ids().await;
            if !discovered_gids.is_empty() {
                // Filter candidate IDs with is_valid_match_id and cap candidates to at most 16 IDs to prevent 404 storms
                let candidate_ids: Vec<i64> = discovered_gids
                    .into_iter()
                    .filter(|&id| is_valid_match_id(id))
                    .take(16)
                    .collect();

                let target_needed = (beg_index + count).min(200);
                // Bound the fallback scanning by a 3-second timeout
                let _ = tokio::time::timeout(Duration::from_secs(3), async {
                    let mut candidate_iter = candidate_ids.into_iter();
                    while raw_games.len() < target_needed {
                        let batch: Vec<i64> = candidate_iter.by_ref().take(8).collect();
                        if batch.is_empty() {
                            break;
                        }
                        let mut handles = Vec::new();
                        for gid in batch {
                            let lcu_c = lcu.clone();
                            handles.push(tokio::spawn(async move {
                                lcu_c.get_match_detail(&gid.to_string()).await
                            }));
                        }
                        for h in handles {
                            if let Ok(Some(g_val)) = h.await {
                                if g_val.get("participants").is_some() {
                                    let in_match = g_val.get("participantIdentities")
                                        .and_then(|pi| pi.as_array())
                                        .map(|idents| {
                                            idents.iter().any(|item| {
                                                let p = item.get("player");
                                                let p_puuid = p.and_then(|x| x.get("puuid")).or_else(|| item.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
                                                p_puuid.eq_ignore_ascii_case(puuid)
                                            })
                                        })
                                        .unwrap_or(false);
                                    if in_match {
                                        raw_games.push(g_val);
                                    }
                                }
                            }
                        }
                    }
                }).await;
            }
        }

        if !raw_games.is_empty() {
            for g in &raw_games {
                let game_id = g.get("gameId").and_then(|v| v.as_i64()).unwrap_or(0);
                let game_creation = g.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
                let duration_sec = g.get("gameDuration").and_then(|v| v.as_i64()).unwrap_or(0).max(0);
                let duration_str = format!("{}:{:02}", duration_sec / 60, duration_sec % 60);
                let queue_id = g.get("queueId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let mode_cn = queue_id_to_mode_cn(queue_id).to_string();

                let empty_parts = Vec::new();
                let participants = g.get("participants").and_then(|p| p.as_array()).unwrap_or(&empty_parts);
                let empty_idents = Vec::new();
                let identities = g.get("participantIdentities").and_then(|p| p.as_array()).unwrap_or(&empty_idents);

                let mut ident_map: HashMap<i64, (String, String, String)> = HashMap::new(); // pid -> (puuid, name, tag)
                let mut target_pid = None;

                for pi in identities {
                    let pid = pi.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pl = pi.get("player");
                    let p_puuid = pl.and_then(|p| p.get("puuid")).or_else(|| pi.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
                    let p_name = pl.and_then(|p| p.get("gameName").or_else(|| p.get("summonerName"))).and_then(|v| v.as_str()).unwrap_or("");
                    let p_tag = pl.and_then(|p| p.get("tagLine")).and_then(|v| v.as_str()).unwrap_or("");
                    ident_map.insert(pid, (p_puuid.to_string(), p_name.to_string(), p_tag.to_string()));

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
                    let p_puuid = p.get("puuid").and_then(|v| v.as_str());
                    let is_match = if let Some(t_pid) = target_pid {
                        pid == Some(t_pid)
                    } else if let Some(p_id) = p_puuid {
                        p_id.eq_ignore_ascii_case(puuid)
                    } else {
                        false
                    };
                    if is_match {
                        if target_pid.is_none() {
                            target_pid = pid;
                        }
                        target_part = Some(p);
                        break;
                    }
                }

                let part = match target_part {
                    Some(p) => p,
                    None => continue,
                };

                let stats_obj = part.get("stats").unwrap_or(part);
                let stats = Some(stats_obj);
                let my_tid = part.get("teamId")
                    .or_else(|| stats.and_then(|s| s.get("teamId")))
                    .and_then(|v| v.as_i64())
                    .unwrap_or_else(|| {
                        if let Some(pid) = target_pid {
                            if pid <= 5 { 100 } else { 200 }
                        } else {
                            100
                        }
                    });
                let is_blue_team = my_tid == 100;
                let is_win = stats.and_then(|s| s.get("win"))
                    .or_else(|| part.get("win"))
                    .map(|w| w.as_bool().unwrap_or_else(|| w.as_i64() == Some(1) || w.as_str() == Some("true") || w.as_str() == Some("Win")))
                    .unwrap_or_else(|| {
                        if queue_id == 1700 || queue_id == 1710 {
                            let placement = stats.and_then(|s| s.get("subteamPlacement")).and_then(|v| v.as_i64()).unwrap_or(0);
                            placement == 1
                        } else {
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
                        }
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

                let s1 = part.get("spell1Id").or_else(|| stats.and_then(|s| s.get("spell1Id"))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let s2 = part.get("spell2Id").or_else(|| stats.and_then(|s| s.get("spell2Id"))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let phys_dmg = stats.and_then(|s| s.get("physicalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
                let magic_dmg = stats.and_then(|s| s.get("magicDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
                let true_dmg = stats.and_then(|s| s.get("trueDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
                let rune_primary = stats.and_then(|s| s.get("perkPrimaryStyle")).or_else(|| stats.and_then(|s| s.get("perk0"))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let rune_sub = stats.and_then(|s| s.get("perkSubStyle")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

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

                let mut teammate_puuids = Vec::new();
                let mut eval_inputs = Vec::new();
                let mut participants_preview = Vec::new();

                for pt in participants {
                    let pt_tid = pt.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_pid = pt.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_stats_obj = pt.get("stats").unwrap_or(pt);
                    let pt_stats = Some(pt_stats_obj);
                    let pt_win = pt_stats.and_then(|s| s.get("win")).map(|w| {
                        w.as_bool().unwrap_or_else(|| w.as_str() == Some("Win") || w.as_i64() == Some(1))
                    }).unwrap_or(false);
                    let pt_k = pt_stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let pt_d = pt_stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let pt_a = pt_stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let pt_dmg = pt_stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_taken = pt_stats.and_then(|s| s.get("totalDamageTaken")).and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_mitig = pt_stats.and_then(|s| s.get("damageSelfMitigated")).and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_gold = pt_stats.and_then(|s| s.get("goldEarned")).and_then(|v| v.as_i64()).unwrap_or(0);
                    let pt_cs = pt_stats.and_then(|s| s.get("totalMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32
                        + pt_stats.and_then(|s| s.get("neutralMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let pt_vis = pt_stats.and_then(|s| s.get("visionScore")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                    let (pt_puuid, pt_name, pt_tag) = ident_map.get(&pt_pid).cloned().unwrap_or_else(|| {
                        let pl = pt.get("player");
                        let p_puuid = pl.and_then(|p| p.get("puuid")).or_else(|| pt.get("puuid")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let p_name = pl.and_then(|p| p.get("gameName").or_else(|| p.get("summonerName"))).or_else(|| pt.get("gameName")).or_else(|| pt.get("summonerName")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let p_tag = pl.and_then(|p| p.get("tagLine")).or_else(|| pt.get("tagLine")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (p_puuid, p_name, p_tag)
                    });

                    let pt_cid = pt.get("championId").or_else(|| pt_stats.and_then(|s| s.get("championId"))).and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                    let pt_cname = if let Some(db) = self.get_database().await {
                        db.resolve_hero_name(&pt_cid.to_string()).unwrap_or_else(|| format!("英雄{}", pt_cid))
                    } else {
                        format!("英雄{}", pt_cid)
                    };
                    let pt_s1 = pt.get("spell1Id").or_else(|| pt_stats.and_then(|s| s.get("spell1Id"))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let pt_s2 = pt.get("spell2Id").or_else(|| pt_stats.and_then(|s| s.get("spell2Id"))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                    participants_preview.push(ParticipantPreviewItem {
                        participant_id: pt_pid,
                        puuid: pt_puuid.clone(),
                        summoner_name: pt_name.clone(),
                        game_name: pt_name,
                        tag_line: pt_tag,
                        team_id: pt_tid as i32,
                        is_blue: pt_tid == 100,
                        champion_id: pt_cid,
                        champion_name: pt_cname,
                        champion_icon: format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", pt_cid),
                        is_win: pt_win,
                        kills: pt_k,
                        deaths: pt_d,
                        assists: pt_a,
                        spell1_id: pt_s1,
                        spell2_id: pt_s2,
                    });

                    if pt_tid == my_tid && pt_pid != target_pid.unwrap_or(-1) {
                        if !pt_puuid.is_empty() {
                            teammate_puuids.push(pt_puuid.clone());
                        }
                    }

                    eval_inputs.push(MatchParticipantInput {
                        participant_id: pt_pid,
                        puuid: pt_puuid,
                        team_id: pt_tid,
                        is_win: pt_win,
                        kills: pt_k,
                        deaths: pt_d,
                        assists: pt_a,
                        damage: pt_dmg,
                        damage_taken: pt_taken,
                        damage_mitigated: pt_mitig,
                        gold: pt_gold,
                        cs: pt_cs,
                        vision_score: pt_vis,
                    });
                }

                let evals = evaluate_match_participants(duration_sec, &eval_inputs);
                let my_eval = target_pid.and_then(|tpid| evals.iter().find(|e| e.participant_id == tpid));

                let dmg_share_pct = my_eval.map(|e| e.damage_share_pct).unwrap_or(0.0);
                let kp_pct = my_eval.map(|e| e.kp_pct).unwrap_or(0.0);
                let is_mvp = my_eval.map(|e| e.is_mvp).unwrap_or(false);
                let is_svp = my_eval.map(|e| e.is_svp).unwrap_or(false);

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
                    team_id: my_tid as i32,
                    is_blue: is_blue_team,
                    is_win,
                    spell1_id: s1,
                    spell2_id: s2,
                    physical_damage: phys_dmg,
                    magic_damage: magic_dmg,
                    true_damage: true_dmg,
                    primary_rune: rune_primary,
                    secondary_rune: rune_sub,
                    participants_preview,
                });
            }
        }

        parsed_matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation).then_with(|| b.game_id.cmp(&a.game_id)));

        if parsed_matches.is_empty() && ranked.is_none() {
            return None;
        }

        let record = CachedPlayerRecord {
            puuid: puuid.to_string(),
            summoner_name: if !real_name.is_empty() { real_name.clone() } else { candidate_name.to_string() },
            tier_str,
            tier_color: get_tier_color(&raw_tier).to_string(),
            tier: raw_tier,
            lp,
            ranked_wins,
            ranked_losses,
            matches: parsed_matches.clone(),
            fetched_at: Instant::now(),
            raw_summoner: json!({
                "puuid": puuid,
                "name": real_name,
                "gameName": real_name,
                "displayName": real_name,
                "tagLine": tag_line,
                "full_name": format!("{}#{}", real_name, tag_line),
                "level": level,
                "profile_icon_id": icon_id
            }),
            raw_ranked: ranked.unwrap_or(json!({
                "solo": {
                    "tier": "UNRANKED",
                    "division": raw_division,
                    "tier_str": "未定级",
                    "lp": 0,
                    "wins": 0,
                    "losses": 0,
                    "win_rate": null
                }
            })),
        };

        let mut guard = self.raw_cache.write().await;
        if guard.len() > 200 {
            guard.retain(|_, r| r.fetched_at.elapsed() < Duration::from_secs(3600));
            if guard.len() > 200 {
                let mut entries: Vec<(String, Instant)> = guard.iter().map(|(k, v)| (k.clone(), v.fetched_at)).collect();
                entries.sort_by_key(|(_, t)| *t);
                let to_remove = guard.len().saturating_sub(150);
                for (k, _) in entries.into_iter().take(to_remove) {
                    guard.remove(&k);
                }
            }
        }
        let final_record = if let Some(existing) = guard.get_mut(puuid) {
            existing.tier_str = record.tier_str.clone();
            existing.tier = record.tier.clone();
            existing.tier_color = record.tier_color.clone();
            existing.lp = record.lp;
            existing.ranked_wins = record.ranked_wins;
            existing.ranked_losses = record.ranked_losses;
            existing.fetched_at = Instant::now();
            if !record.summoner_name.is_empty() {
                existing.summoner_name = record.summoner_name.clone();
            }
            if record.raw_summoner.is_object() {
                existing.raw_summoner = record.raw_summoner.clone();
            }
            if record.raw_ranked.is_object() {
                existing.raw_ranked = record.raw_ranked.clone();
            }

            for m in &parsed_matches {
                if !existing.matches.iter().any(|em| em.game_id == m.game_id) {
                    existing.matches.push(m.clone());
                }
            }
            existing.matches.sort_by(|a, b| b.game_creation.cmp(&a.game_creation).then_with(|| b.game_id.cmp(&a.game_id)));
            if existing.matches.len() > 500 {
                existing.matches.truncate(500);
            }
            existing.clone()
        } else {
            guard.insert(puuid.to_string(), record.clone());
            record
        };

        Some(final_record)
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

        for mi in 0..200 {
            let is_win = cfg.4[mi % cfg.4.len()];
            let k = if is_win { 11 } else { 3 };
            let d = if is_win { 2 } else { 6 };
            let a = if is_win { 8 } else { 4 };
            let items = if is_win { vec![3031, 3006, 3072, 3036, 3026, 3139] } else { vec![3031, 3006, 1055, 0, 0, 0] };

            matches.push(ParsedMatchRecord {
                game_id: 99990000 + (slot_index as i64) * 1000 + (mi as i64),
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
                team_id: if mi % 2 == 0 { 100 } else { 200 },
                is_blue: mi % 2 == 0,
                is_win,
                spell1_id: 4,
                spell2_id: 14,
                physical_damage: if is_win { 18000 } else { 8000 },
                magic_damage: if is_win { 6000 } else { 4000 },
                true_damage: if is_win { 2500 } else { 2200 },
                primary_rune: 8000,
                secondary_rune: 8100,
                participants_preview: Vec::new(),
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
                    queue_id: 420,
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
            queue_id: 420,
            queue_name: "排位赛(单双)".to_string(),
            is_my_team_blue: true,
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
        let _ = self.sync_local_summoner(lcu).await;
        let phase = lcu.get_gameflow_phase().await;

        if phase == "ChampSelect" {
            let session = lcu.get_champ_select_session().await;
            if session.is_null() {
                return OngoingMatchResponse {
                    phase: phase.clone(),
                    phase_cn: "英雄选择中".to_string(),
                    queue_id: 0,
                    queue_name: "英雄选择中".to_string(),
                    is_my_team_blue: true,
                    is_active: true,
                    is_mock: false,
                    chat_room_id: String::new(),
                    blue_team: Vec::new(),
                    red_team: Vec::new(),
                    my_team: Vec::new(),
                    their_team: Vec::new(),
                    game_id: None,
                    message: Some("正在从客户端同步选人会话...".to_string()),
                };
            }

            let queue_id = session.get("gameData").and_then(|g| g.get("queueId"))
                .or_else(|| session.get("queueId"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

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
                    let matches = cached.as_ref().map(|c| c.matches.clone()).unwrap_or_default();
                    let (real_level, real_icon, real_tag) = if let Some(ref c) = cached {
                        let lvl = c.raw_summoner.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let ico = c.raw_summoner.get("profile_icon_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let tag = c.raw_summoner.get("tagLine").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (lvl, ico, tag)
                    } else {
                        (0, 0, String::new())
                    };

                    let pos = if queue_id == 1700 || queue_id == 1710 {
                        "ARENA".to_string()
                    } else if queue_id == 450 || queue_id == 860 || session.get("benchChampions").is_some() {
                        "ARAM".to_string()
                    } else {
                        p.get("assignedPosition").and_then(|v| v.as_str()).unwrap_or("").to_uppercase()
                    };
                    let tid = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(100) as i32;

                    my_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: tid,
                        is_local_player: is_locals.get(idx).copied().unwrap_or(false),
                        puuid: prof.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: if real_tag.is_empty() { name.clone() } else { format!("{}#{}", name, real_tag) },
                        level: real_level,
                        profile_icon_id: real_icon,
                        position: pos,
                        champion_id: cid,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        spell2_id: p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        ranked: json!({ "solo": { "tier": prof.tier, "division": prof.division, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                        queue_id,
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
                    let matches = cached.as_ref().map(|c| c.matches.clone()).unwrap_or_default();
                    let (real_level, real_icon, real_tag) = if let Some(ref c) = cached {
                        let lvl = c.raw_summoner.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let ico = c.raw_summoner.get("profile_icon_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let tag = c.raw_summoner.get("tagLine").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (lvl, ico, tag)
                    } else {
                        (0, 0, String::new())
                    };

                    let pos = if queue_id == 1700 || queue_id == 1710 {
                        "ARENA".to_string()
                    } else if queue_id == 450 || queue_id == 860 {
                        "ARAM".to_string()
                    } else {
                        p.get("assignedPosition").and_then(|v| v.as_str()).unwrap_or("").to_uppercase()
                    };
                    let tid = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(200) as i32;

                    their_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: tid,
                        is_local_player: false,
                        puuid: prof.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: if real_tag.is_empty() { name.clone() } else { format!("{}#{}", name, real_tag) },
                        level: real_level,
                        profile_icon_id: real_icon,
                        position: pos,
                        champion_id: cid,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        spell2_id: p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        ranked: json!({ "solo": { "tier": prof.tier, "division": prof.division, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                        queue_id,
                    });
                }
            }

            detect_premade_teams_with_queue(&mut my_cards, queue_id);
            detect_premade_teams_with_queue(&mut their_cards, queue_id);

            let is_blue = my_cards.first().map(|c| c.team_id == 100).unwrap_or(true);
            let (blue_team, red_team) = if is_blue {
                (my_cards.clone(), their_cards.clone())
            } else {
                (their_cards.clone(), my_cards.clone())
            };

            let q_name = match queue_id {
                0 => "排位选人阶段",
                _ => queue_id_to_mode_cn(queue_id),
            };

            OngoingMatchResponse {
                phase,
                phase_cn: "英雄选择阶段".to_string(),
                queue_id,
                queue_name: q_name.to_string(),
                is_my_team_blue: is_blue,
                is_active: true,
                is_mock: false,
                chat_room_id,
                blue_team,
                red_team,
                my_team: my_cards,
                their_team: their_cards,
                game_id: None,
                message: None,
            }
        } else if phase == "GameStart" || phase == "InProgress" {
            let gf_session = lcu.get_gameflow_session().await;
            let queue_id = gf_session.get("gameData").and_then(|g| g.get("queue").and_then(|q| q.get("id")))
                .or_else(|| gf_session.get("gameData").and_then(|g| g.get("queueId")))
                .or_else(|| gf_session.get("queueId"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            if let Some((my_slots, their_slots)) = self.resolve_gameflow_10_players(&gf_session, lcu).await {
                let mut my_cards = Vec::with_capacity(my_slots.len());
                for (idx, s) in my_slots.into_iter().enumerate() {
                    let prof = s.profile.unwrap_or_default();
                    let cname = if s.champion_id > 0 { db.resolve_hero_name(&s.champion_id.to_string()).unwrap_or_else(|| s.summoner_name.clone()) } else { s.summoner_name.clone() };
                    let cavatar = if s.champion_id > 0 { format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", s.champion_id) } else { String::new() };
                    let name = if !s.summoner_name.is_empty() {
                        s.summoner_name.clone()
                    } else if !prof.summoner_name.is_empty() {
                        prof.summoner_name.clone()
                    } else if s.champion_id > 0 {
                        cname.clone()
                    } else {
                        format!("蓝方{}号位", idx + 1)
                    };
                    let cached = self.get_player_data(&s.puuid).await;
                    let matches = cached.as_ref().map(|c| c.matches.clone()).unwrap_or_default();
                    let (real_level, real_icon, real_tag) = if let Some(ref c) = cached {
                        let lvl = c.raw_summoner.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let ico = c.raw_summoner.get("profile_icon_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let tag = c.raw_summoner.get("tagLine").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (lvl, ico, tag)
                    } else {
                        (0, 0, String::new())
                    };

                    my_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: if s.team_id > 0 { s.team_id as i32 } else { 100 },
                        is_local_player: s.is_local_player,
                        puuid: s.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: if real_tag.is_empty() { name.clone() } else { format!("{}#{}", name, real_tag) },
                        level: real_level,
                        profile_icon_id: real_icon,
                        position: s.position,
                        champion_id: s.champion_id,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: s.spell1_id,
                        spell2_id: s.spell2_id,
                        ranked: json!({ "solo": { "tier": prof.tier, "division": prof.division, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                        queue_id,
                    });
                }

                let mut their_cards = Vec::with_capacity(their_slots.len());
                for (idx, s) in their_slots.into_iter().enumerate() {
                    let prof = s.profile.unwrap_or_default();
                    let cname = if s.champion_id > 0 { db.resolve_hero_name(&s.champion_id.to_string()).unwrap_or_else(|| s.summoner_name.clone()) } else { s.summoner_name.clone() };
                    let cavatar = if s.champion_id > 0 { format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", s.champion_id) } else { String::new() };
                    let name = if !s.summoner_name.is_empty() {
                        s.summoner_name.clone()
                    } else if !prof.summoner_name.is_empty() {
                        prof.summoner_name.clone()
                    } else if s.champion_id > 0 {
                        cname.clone()
                    } else {
                        format!("红方{}号位", idx + 1)
                    };
                    let cached = self.get_player_data(&s.puuid).await;
                    let matches = cached.as_ref().map(|c| c.matches.clone()).unwrap_or_default();
                    let (real_level, real_icon, real_tag) = if let Some(ref c) = cached {
                        let lvl = c.raw_summoner.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let ico = c.raw_summoner.get("profile_icon_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let tag = c.raw_summoner.get("tagLine").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (lvl, ico, tag)
                    } else {
                        (0, 0, String::new())
                    };

                    their_cards.push(OngoingPlayerCard {
                        slot_index: idx,
                        team_id: if s.team_id > 0 { s.team_id as i32 } else { 200 },
                        is_local_player: s.is_local_player,
                        puuid: s.puuid.clone(),
                        summoner_name: name.clone(),
                        full_name: if real_tag.is_empty() { name.clone() } else { format!("{}#{}", name, real_tag) },
                        level: real_level,
                        profile_icon_id: real_icon,
                        position: s.position,
                        champion_id: s.champion_id,
                        champion_name: cname,
                        champion_avatar: cavatar,
                        spell1_id: s.spell1_id,
                        spell2_id: s.spell2_id,
                        ranked: json!({ "solo": { "tier": prof.tier, "division": prof.division, "tier_str": prof.tier_str, "lp": prof.lp } }),
                        tactical_analysis: prof,
                        matches,
                        premade_group: None,
                        premade_tag: None,
                        premade_color: None,
                        premade_bg: None,
                        premade_text: None,
                        queue_id,
                    });
                }

                detect_premade_teams_with_queue(&mut my_cards, queue_id);
                detect_premade_teams_with_queue(&mut their_cards, queue_id);

                let is_blue = my_cards.first().map(|c| c.team_id == 100).unwrap_or(true);
                let (blue_team, red_team) = if is_blue {
                    (my_cards.clone(), their_cards.clone())
                } else {
                    (their_cards.clone(), my_cards.clone())
                };

                let queue_name = match queue_id {
                    0 => if phase == "InProgress" { "游戏进行中" } else { "对局加载中" },
                    _ => queue_id_to_mode_cn(queue_id),
                }.to_string();

                OngoingMatchResponse {
                    phase: phase.clone(),
                    phase_cn: if phase == "InProgress" { "正在游戏中".to_string() } else { "对局加载中".to_string() },
                    queue_id,
                    queue_name,
                    is_my_team_blue: is_blue,
                    is_active: true,
                    is_mock: false,
                    chat_room_id: String::new(),
                    blue_team,
                    red_team,
                    my_team: my_cards,
                    their_team: their_cards,
                    game_id: gf_session.get("gameData").and_then(|g| g.get("gameId")).and_then(|v| v.as_i64()),
                    message: None,
                }
            } else {
                OngoingMatchResponse {
                    phase: phase.clone(),
                    phase_cn: if phase == "InProgress" { "正在游戏中".to_string() } else { "对局加载中".to_string() },
                    queue_id: 0,
                    queue_name: if phase == "InProgress" { "正在游戏中".to_string() } else { "对局加载中".to_string() },
                    is_my_team_blue: true,
                    is_active: true,
                    is_mock: false,
                    chat_room_id: String::new(),
                    blue_team: Vec::new(),
                    red_team: Vec::new(),
                    my_team: Vec::new(),
                    their_team: Vec::new(),
                    game_id: None,
                    message: Some("正在从客户端同步实时对局状态...".to_string()),
                }
            }
        } else {
            OngoingMatchResponse {
                phase,
                phase_cn: "未在对局中".to_string(),
                queue_id: 0,
                queue_name: "未在对局中".to_string(),
                is_my_team_blue: true,
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
        let slot_count = if my_team_cids.is_empty() { 5 } else { my_team_cids.len() };
        let mut assigned_puuids: Vec<Option<(String, String)>> = vec![None; slot_count]; // (puuid, name)

        // 1. Ensure local player PUUID & name (sync with account switch detection)
        let (local_puuid_opt, local_name_opt) = self.sync_local_summoner(lcu).await;

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

        // 2. Collect candidate teammates from chat conversation participants with retry
        let chat_details = session.get("chatDetails");
        let chat_room_id = chat_details
            .and_then(|c| c.get("multiUserChatId").or_else(|| c.get("chatRoomId")).or_else(|| c.get("chatRoomName")))
            .and_then(|v| v.as_str())
            .or_else(|| session.get("chatRoomId").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let needed_count = my_team_is_locals
            .iter()
            .take(slot_count)
            .filter(|&&is_local| !is_local)
            .count()
            .max(1);

        let participants = self
            .fetch_chat_participants_with_retry(
                lcu,
                chat_room_id.as_deref(),
                local_puuid_opt.as_deref(),
                needed_count,
            )
            .await;

        // Build candidate list (STRICTLY EXCLUDING local player to prevent duplicate identity bug!)
        let mut candidate_teammates = Vec::new();
        for p in &participants {
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

        // 3. Match candidate teammates to slots with explicit matching (summonerId, puuid, or exact name)
        // NEVER sequentially assign chat candidates into slots 0..4!
        let mut used_candidates = std::collections::HashSet::new();

        if let Some(my_arr) = session.get("myTeam").and_then(|v| v.as_array()) {
            for (idx, m) in my_arr.iter().enumerate() {
                if idx >= assigned_puuids.len() || assigned_puuids[idx].is_some() {
                    continue;
                }
                let is_local = my_team_is_locals.get(idx).copied().unwrap_or(false);
                if is_local {
                    continue;
                }

                let slot_puuid = m.get("puuid").and_then(|v| v.as_str()).unwrap_or("");
                let slot_sum_id = m.get("summonerId")
                    .and_then(|v| v.as_i64())
                    .or_else(|| m.get("summonerId").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok()))
                    .unwrap_or(0);
                let slot_name = m.get("summonerName")
                    .or_else(|| m.get("gameName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();

                let is_generic_placeholder = slot_name.is_empty()
                    || slot_name.starts_with("Summoner ")
                    || slot_name.starts_with("我方")
                    || slot_name.starts_with("召唤师");

                // Priority A: Slot has explicit non-zero puuid
                if !slot_puuid.is_empty() && slot_puuid != "0" {
                    if let Some(ref lp) = local_puuid_opt {
                        if slot_puuid == lp {
                            continue;
                        }
                    }
                    // Find if any chat participant matches this puuid to get name
                    let matched_name = candidate_teammates.iter().enumerate().find_map(|(c_i, (cid, cname))| {
                        if cid == slot_puuid {
                            used_candidates.insert(c_i);
                            Some(cname.clone())
                        } else {
                            None
                        }
                    }).unwrap_or_else(|| {
                        if !is_generic_placeholder { slot_name.to_string() } else { String::new() }
                    });
                    assigned_puuids[idx] = Some((slot_puuid.to_string(), matched_name));
                    continue;
                }

                // Priority B: Match by summonerId against chat participants
                if slot_sum_id > 0 {
                    let mut matched_cand = None;
                    for (c_i, (cid, cname)) in candidate_teammates.iter().enumerate() {
                        if used_candidates.contains(&c_i) {
                            continue;
                        }
                        let part_sum_id = participants.get(c_i)
                            .and_then(|p| p.get("summonerId").and_then(|v| v.as_i64()))
                            .or_else(|| cid.parse::<i64>().ok());
                        if part_sum_id == Some(slot_sum_id) {
                            matched_cand = Some((c_i, cid.clone(), cname.clone()));
                            break;
                        }
                    }
                    if let Some((c_i, cid, cname)) = matched_cand {
                        used_candidates.insert(c_i);
                        assigned_puuids[idx] = Some((cid, cname));
                        continue;
                    }
                }

                // Priority C: Match by exact summonerName / gameName
                if !is_generic_placeholder {
                    let mut matched_cand = None;
                    for (c_i, (cid, cname)) in candidate_teammates.iter().enumerate() {
                        if used_candidates.contains(&c_i) {
                            continue;
                        }
                        if cname.eq_ignore_ascii_case(slot_name) {
                            matched_cand = Some((c_i, cid.clone(), cname.clone()));
                            break;
                        }
                    }
                    if let Some((c_i, cid, cname)) = matched_cand {
                        used_candidates.insert(c_i);
                        assigned_puuids[idx] = Some((cid, cname));
                        continue;
                    }
                }
            }
        }

        // Collect unlinked participants into unlinked pool and cache their tactical data
        let mut unlinked = Vec::new();
        for (c_i, (cid, cname)) in candidate_teammates.iter().enumerate() {
            if !used_candidates.contains(&c_i) {
                unlinked.push((cid.clone(), cname.clone()));
            }
        }
        {
            let mut guard = self.unlinked_candidates.write().await;
            *guard = unlinked.clone();
        }

        // Pre-fetch unlinked candidates in background with cur_cid = 0 to avoid false trial champion warning
        for (cid, cname) in unlinked {
            let mgr = self.clone();
            let lcu_c = lcu.clone();
            tokio::spawn(async move {
                let sample_limit = mgr.sample_count.load(std::sync::atomic::Ordering::Relaxed) as usize;
                let end_index = sample_limit.saturating_sub(1);
                mgr.fetch_and_cache_player(&cid, &cname, &lcu_c, 0, end_index).await;
            });
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
                    let ttl = if rec.matches.is_empty() {
                        Duration::from_secs(10)
                    } else {
                        Duration::from_secs(600)
                    };
                    if rec.fetched_at.elapsed() < ttl {
                        Some(rec)
                    } else {
                        let _permit = sem.acquire().await.ok()?;
                        let end_index = sample_limit.saturating_sub(1);
                        mgr.fetch_and_cache_player(&puuid, &name, &lcu_client, 0, end_index).await
                    }
                } else {
                    let _permit = sem.acquire().await.ok()?;
                    let end_index = sample_limit.saturating_sub(1);
                    mgr.fetch_and_cache_player(&puuid, &name, &lcu_client, 0, end_index).await
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
        let mut spell_map: HashMap<String, (i32, i32)> = HashMap::new();
        let mut cell_spell_map: HashMap<i64, (i32, i32)> = HashMap::new();

        if let Some(sels) = selections_raw {
            for sel in sels {
                let cid = sel.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let s1 = sel.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let s2 = sel.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                if let Some(p) = sel.get("puuid").and_then(|v| v.as_str()) {
                    if cid > 0 { selection_map.insert(p.to_string(), cid); }
                    spell_map.insert(p.to_string(), (s1, s2));
                }
                if let Some(n) = sel.get("summonerInternalName").and_then(|v| v.as_str()) {
                    if cid > 0 { selection_map.insert(n.to_string(), cid); }
                    spell_map.insert(n.to_string(), (s1, s2));
                }
                if let Some(cell) = sel.get("cellId").and_then(|v| v.as_i64()) {
                    if cid > 0 { cell_selection_map.insert(cell, cid); }
                    cell_spell_map.insert(cell, (s1, s2));
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

                let (s1, s2) = if !puuid.is_empty() {
                    spell_map.get(&puuid).copied()
                } else {
                    None
                }.or_else(|| {
                    if !internal_name.is_empty() {
                        spell_map.get(internal_name).copied()
                    } else {
                        None
                    }
                }).or_else(|| cell_spell_map.get(&cell_id).copied()).unwrap_or((0, 0));

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
                let team_id = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);

                slots.push(GameflowSlotInfo {
                    cell_id,
                    puuid,
                    summoner_name,
                    champion_id: cid,
                    position: pos.to_string(),
                    position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                    is_local_player: is_local,
                    profile: None,
                    team_id,
                    spell1_id: s1,
                    spell2_id: s2,
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

        let queue_id = game_data
            .get("queue")
            .and_then(|q| q.get("id"))
            .and_then(|v| v.as_i64())
            .or_else(|| game_data.get("queueId").and_then(|v| v.as_i64()))
            .unwrap_or(0) as i32;
        let game_mode = game_data
            .get("gameMode")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let is_aram = queue_id == 450 || queue_id == 860 || game_mode.eq_ignore_ascii_case("ARAM");
        let is_arena = queue_id == 1700 || queue_id == 1710 || game_mode.eq_ignore_ascii_case("CHERRY");

        if is_aram {
            for s in my_team.iter_mut().chain(their_team.iter_mut()) {
                s.position = "ARAM".to_string();
                s.position_cn = "极地大乱斗".to_string();
            }
        } else if is_arena {
            for s in my_team.iter_mut().chain(their_team.iter_mut()) {
                s.position = "ARENA".to_string();
                s.position_cn = "斗魂竞技场".to_string();
            }
        } else {
            // Infer 5v5 distinct lane positions based on selectedPosition, Smite, and champion
            assign_team_positions(&mut my_team);
            assign_team_positions(&mut their_team);
        }

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
                if let Some(ref prof) = p {
                    if my_team[i].summoner_name.is_empty() && !prof.summoner_name.is_empty() {
                        my_team[i].summoner_name = prof.summoner_name.clone();
                    }
                }
                my_team[i].profile = p;
            } else if (i - my_len) < their_team.len() {
                let idx = i - my_len;
                if let Some(ref prof) = p {
                    if their_team[idx].summoner_name.is_empty() && !prof.summoner_name.is_empty() {
                        their_team[idx].summoner_name = prof.summoner_name.clone();
                    }
                }
                their_team[idx].profile = p;
            }
        }

        Some((my_team, their_team))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_id_to_mode_cn_mappings() {
        assert_eq!(queue_id_to_mode_cn(420), "排位赛(单双)");
        assert_eq!(queue_id_to_mode_cn(440), "排位赛(灵活)");
        assert_eq!(queue_id_to_mode_cn(450), "极地大乱斗");
        assert_eq!(queue_id_to_mode_cn(860), "极地大乱斗");
        assert_eq!(queue_id_to_mode_cn(490), "快速游戏");
        assert_eq!(queue_id_to_mode_cn(1700), "斗魂竞技场");
        assert_eq!(queue_id_to_mode_cn(1810), "无尽狂潮");
        assert_eq!(queue_id_to_mode_cn(1300), "极限闪击");
        assert_eq!(queue_id_to_mode_cn(0), "自定义对局");
        assert_eq!(queue_id_to_mode_cn(9999), "其他模式");
    }

    #[test]
    fn test_detect_premade_teams_duplicate_teammates_in_single_match() {
        let mut player_a = OngoingPlayerCard::default();
        player_a.puuid = "puuid_a".to_string();
        player_a.team_id = 100;

        let mut player_b = OngoingPlayerCard::default();
        player_b.puuid = "puuid_b".to_string();
        player_b.team_id = 100;

        // Player A only has 1 match with Player B, but teammate_puuids has duplicate entries for puuid_b
        let mut match1 = ParsedMatchRecord::default();
        match1.game_id = 1001;
        match1.teammate_puuids = vec!["puuid_b".to_string(), "puuid_b".to_string(), "puuid_c".to_string()];
        player_a.matches = vec![match1];

        let mut cards = vec![player_a, player_b];
        detect_premade_teams(&mut cards);

        // Since they only played 1 match together, they should NOT be marked as premade (threshold is >= 2)
        assert!(cards[0].premade_group.is_none());
        assert!(cards[1].premade_group.is_none());
    }

    #[test]
    fn test_record_game_id_boundary_filtering() {
        let ph = PlayerHistoryManager::new();
        ph.record_game_id(0); // Should be ignored (<= 0)
        ph.record_game_id(-100); // Should be ignored
        ph.record_game_id(100_000_000_000); // Should be ignored (> 99B)
    }

    #[test]
    fn test_evaluate_match_participants_adversarial_nan_safeguard() {
        // Adversarial zero damage, zero duration, negative values
        let p1 = MatchParticipantInput {
            participant_id: 1,
            puuid: "p1".to_string(),
            team_id: 100,
            is_win: true,
            kills: 0,
            deaths: 0,
            assists: 0,
            damage: 0,
            damage_taken: 0,
            damage_mitigated: 0,
            gold: 0,
            cs: 0,
            vision_score: 0,
        };
        let p2 = MatchParticipantInput {
            participant_id: 2,
            puuid: "p2".to_string(),
            team_id: 200,
            is_win: false,
            kills: -5,
            deaths: -1,
            assists: -2,
            damage: -100,
            damage_taken: -50,
            damage_mitigated: -20,
            gold: -10,
            cs: -10,
            vision_score: -5,
        };

        let evals = evaluate_match_participants(0, &[p1, p2]);
        assert_eq!(evals.len(), 2);
        for e in &evals {
            assert!(e.score.is_finite(), "Score must be finite");
            assert!(e.damage_share_pct.is_finite(), "Dmg share must be finite");
            assert!(e.damage_taken_share_pct.is_finite(), "Taken share must be finite");
            assert!(e.kp_pct.is_finite(), "KP must be finite");
            assert!(e.score >= 0.0);
            assert!(e.damage_share_pct >= 0.0);
            assert!(e.damage_taken_share_pct >= 0.0);
            assert!(e.kp_pct >= 0.0);
        }
    }

    #[test]
    fn test_detect_premade_teams_deterministic_ordering() {
        let mut cards = Vec::new();
        // Create 4 players: A & B are premade on team 100, C & D are premade on team 200
        for i in 0..4 {
            let mut c = OngoingPlayerCard::default();
            c.puuid = format!("p_{}", i);
            c.team_id = if i < 2 { 100 } else { 200 };
            cards.push(c);
        }

        // Match 1 & Match 2 for A & B
        let m1 = ParsedMatchRecord { game_id: 101, teammate_puuids: vec!["p_1".to_string()], ..Default::default() };
        let m2 = ParsedMatchRecord { game_id: 102, teammate_puuids: vec!["p_1".to_string()], ..Default::default() };
        cards[0].matches = vec![m1, m2];

        // Match 3 & Match 4 for C & D
        let m3 = ParsedMatchRecord { game_id: 201, teammate_puuids: vec!["p_3".to_string()], ..Default::default() };
        let m4 = ParsedMatchRecord { game_id: 202, teammate_puuids: vec!["p_3".to_string()], ..Default::default() };
        cards[2].matches = vec![m3, m4];

        detect_premade_teams(&mut cards);

        // Group 1 should be p_0 and p_1 (team 100)
        assert_eq!(cards[0].premade_group, Some(1));
        assert_eq!(cards[1].premade_group, Some(1));
        // Group 2 should be p_2 and p_3 (team 200)
        assert_eq!(cards[2].premade_group, Some(2));
        assert_eq!(cards[3].premade_group, Some(2));
    }
}
