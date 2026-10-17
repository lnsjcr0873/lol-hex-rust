use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedOverlayConfig {
    pub enabled: bool,
    pub only_ranked: bool,
}

impl Default for RankedOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            only_ranked: false,
        }
    }
}

pub struct RankedOverlay {
    #[allow(dead_code)]
    pub config: Arc<RwLock<RankedOverlayConfig>>,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
    pub test_mode_until: Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

impl Default for RankedOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl RankedOverlay {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(RankedOverlayConfig::default())),
            is_showing: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            test_mode_until: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.start();
    }

    pub fn is_in_test_mode(&self) -> bool {
        if let Ok(guard) = self.test_mode_until.lock() {
            if let Some(exp) = *guard {
                return std::time::Instant::now() < exp;
            }
        }
        false
    }

    pub fn stop(&self) {
        if self.is_in_test_mode() {
            return;
        }
        self.running.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.hide();
    }

    pub fn show_ranked_draft(&self, data: super::win32_hud::RankedOverlayData) {
        self.is_showing.store(true, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.show_ranked(data, std::time::Duration::from_secs(30));
    }

    #[allow(dead_code)]
    pub fn trigger_test(&self, data: super::win32_hud::RankedOverlayData, duration: std::time::Duration) {
        if let Ok(mut guard) = self.test_mode_until.lock() {
            *guard = Some(std::time::Instant::now() + duration);
        }
        self.is_showing.store(true, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.show_ranked_test(data, duration);
    }
}

pub async fn fetch_slot_counters_and_synergies(
    opgg: &crate::crawler::OpggClient,
    db: &crate::data::loader::Database,
    cid_str: &str,
    pos: &str,
) -> (Vec<super::win32_hud::CounterHeroItem>, Vec<super::win32_hud::CounterHeroItem>) {
    if cid_str.is_empty() || cid_str == "0" {
        return (Vec::new(), Vec::new());
    }

    let (c_data, syn_data) = tokio::join!(
        opgg.get_counter_picks_against(cid_str, pos, Some(30)),
        opgg.get_champion_synergies(cid_str, pos)
    );

    let mut counters = Vec::new();
    if let Some(arr) = c_data.get("counters").and_then(|v| v.as_array()) {
        for c in arr {
            let opp_cid = c.get("championId").and_then(|v| v.as_str()).unwrap_or("");
            let opp_id_num: i32 = opp_cid.parse().unwrap_or(0);
            if opp_id_num <= 0 {
                continue;
            }
            let opp_name = db.resolve_hero_name(opp_cid).unwrap_or_else(|| format!("Hero-{}", opp_cid));
            let c_wr = c.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let games = c.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
            let tier = if c_wr >= 0.53 { 1 } else if c_wr >= 0.51 { 2 } else if c_wr > 0.0 { 3 } else { 0 };
            counters.push(super::win32_hud::CounterHeroItem {
                champion_id: opp_id_num,
                name: opp_name,
                tier,
                counter_win_rate: c_wr,
                overall_win_rate: c_wr,
                games,
            });
        }
    }

    let mut synergies = Vec::new();
    if let Some(arr) = syn_data.get("synergies").and_then(|v| v.as_array()) {
        for s in arr {
            let syn_cid = s.get("championId").and_then(|v| v.as_str()).unwrap_or("");
            let syn_id_num: i32 = syn_cid.parse().unwrap_or(0);
            if syn_id_num <= 0 {
                continue;
            }
            let syn_name = db.resolve_hero_name(syn_cid).unwrap_or_else(|| format!("Hero-{}", syn_cid));
            let wr = s.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let games = s.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
            let tier = s.get("tier").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            synergies.push(super::win32_hud::CounterHeroItem {
                champion_id: syn_id_num,
                name: syn_name,
                tier,
                counter_win_rate: wr,
                overall_win_rate: wr,
                games,
            });
        }
    }

    (counters, synergies)
}

pub async fn update_ranked_draft_overlay(
    session: &serde_json::Value,
    draft_mgr: &tokio::sync::RwLock<crate::engine::RankedDraftManager>,
    ranked_overlay: &RankedOverlay,
    lcu: &crate::lcu::LcuClient,
    history: &crate::engine::PlayerHistoryManager,
    opgg: &crate::crawler::OpggClient,
    db: &crate::data::loader::Database,
    last_draft_sig: &mut String,
    cached_ranked_data: &mut Option<super::win32_hud::RankedOverlayData>,
) {
    let mut d = draft_mgr.write().await;
    d.parse_session_with_db_and_opgg(session, Some(db), Some(opgg));

    if d.state.my_team.is_empty() {
        return;
    }

    let sample_limit = history.sample_count.load(std::sync::atomic::Ordering::Relaxed);
    let mut current_sig = format!("S:{};", sample_limit);
    for s in &d.state.my_team {
        current_sig.push_str(&format!("M:{}:{}:{};", s.cell_id, s.champion_id, s.position));
    }
    for s in &d.state.their_team {
        current_sig.push_str(&format!("T:{}:{}:{};", s.cell_id, s.champion_id, s.position));
    }

    let my_cids: Vec<i32> = d.state.my_team.iter().map(|s| s.champion_id.parse::<i32>().unwrap_or(0)).collect();
    let my_locals: Vec<bool> = d.state.my_team.iter().map(|s| s.is_local_player).collect();

    if current_sig == *last_draft_sig {
        if let Some(ref cached) = cached_ranked_data {
            let my_ready = cached.my_team.iter().filter(|s| s.profile.is_some()).count();
            if my_ready >= 4 || my_ready == d.state.my_team.len() {
                ranked_overlay.show_ranked_draft(cached.clone());
                return;
            }
        }
    }

    let profiles = history.resolve_teammates(session, &my_cids, &my_locals, lcu).await;
    let their_cids: Vec<i32> = d.state.their_team.iter().map(|s| s.champion_id.parse::<i32>().unwrap_or(0)).collect();
    let their_profiles = history.resolve_opponents_champ_select(session, &their_cids, lcu).await;

    let local_slot = d.state.my_team.iter().find(|s| s.is_local_player).cloned();
    let local_pos = local_slot.as_ref().map(|s| s.position.clone()).unwrap_or_default();

    // Concurrently fetch counter & synergy data for all slots to eliminate serial network latency
    async fn fetch_slot(
        opgg: &crate::crawler::OpggClient,
        db: &crate::data::loader::Database,
        slot_opt: Option<&crate::engine::ranked_draft::DraftSlot>,
    ) -> (Vec<super::win32_hud::CounterHeroItem>, Vec<super::win32_hud::CounterHeroItem>) {
        if let Some(s) = slot_opt {
            if !s.champion_id.is_empty() {
                return fetch_slot_counters_and_synergies(opgg, db, &s.champion_id, &s.position).await;
            }
        }
        (Vec::new(), Vec::new())
    }

    let (my_c0, my_c1, my_c2, my_c3, my_c4) = tokio::join!(
        fetch_slot(opgg, db, d.state.my_team.get(0)),
        fetch_slot(opgg, db, d.state.my_team.get(1)),
        fetch_slot(opgg, db, d.state.my_team.get(2)),
        fetch_slot(opgg, db, d.state.my_team.get(3)),
        fetch_slot(opgg, db, d.state.my_team.get(4)),
    );
    let my_counters = vec![my_c0, my_c1, my_c2, my_c3, my_c4];

    let (th_c0, th_c1, th_c2, th_c3, th_c4) = tokio::join!(
        fetch_slot(opgg, db, d.state.their_team.get(0)),
        fetch_slot(opgg, db, d.state.their_team.get(1)),
        fetch_slot(opgg, db, d.state.their_team.get(2)),
        fetch_slot(opgg, db, d.state.their_team.get(3)),
        fetch_slot(opgg, db, d.state.their_team.get(4)),
    );
    let their_counters = vec![th_c0, th_c1, th_c2, th_c3, th_c4];

    let mut my_slots = Vec::new();
    for (idx, s) in d.state.my_team.iter().enumerate() {
        let prof = profiles.get(idx).cloned().flatten();
        let cid_num = s.champion_id.parse::<i32>().unwrap_or(0);
        let opp_slot = d.state.their_team.iter().find(|t| t.position == s.position);
        let m_wr = if s.is_local_player && !s.champion_id.is_empty() {
            if let Some(opp) = opp_slot {
                if !opp.champion_id.is_empty() {
                    opgg.get_matchup_win_rate(&s.champion_id, &opp.champion_id, &s.position).await
                        .and_then(|v| v.get("winRate").and_then(|w| w.as_f64()))
                } else { None }
            } else { None }
        } else { None };

        let (c_heroes, s_heroes) = my_counters.get(idx).cloned().unwrap_or_default();

        my_slots.push(super::win32_hud::RankedSlotView {
            slot_index: idx,
            position: s.position.clone(),
            position_cn: s.position_cn.clone(),
            champion_name: s.champion_name.clone(),
            champion_id: cid_num,
            win_rate: s.win_rate,
            tier: s.tier,
            is_local_player: s.is_local_player,
            is_locked: s.is_locked,
            is_manual_pos: false,
            matchup_win_rate: m_wr,
            profile: prof,
            counter_heroes: c_heroes,
            synergy_heroes: s_heroes,
        });
    }

    let mut their_slots = Vec::new();
    for (idx, s) in d.state.their_team.iter().enumerate() {
        let prof = their_profiles.get(idx).cloned().flatten();
        let cid_num = s.champion_id.parse::<i32>().unwrap_or(0);
        let is_opp = !local_pos.is_empty() && s.position == local_pos;
        let my_opp = d.state.my_team.iter().find(|m| m.position == s.position);
        let m_wr = if is_opp && !s.champion_id.is_empty() {
            if let Some(opp) = my_opp {
                if !opp.champion_id.is_empty() {
                    opgg.get_matchup_win_rate(&opp.champion_id, &s.champion_id, &s.position).await
                        .and_then(|v| v.get("winRate").and_then(|w| w.as_f64()))
                } else { None }
            } else { None }
        } else { None };

        let (c_heroes, s_heroes) = their_counters.get(idx).cloned().unwrap_or_default();

        their_slots.push(super::win32_hud::RankedSlotView {
            slot_index: idx,
            position: s.position.clone(),
            position_cn: s.position_cn.clone(),
            champion_name: s.champion_name.clone(),
            champion_id: cid_num,
            win_rate: s.win_rate,
            tier: s.tier,
            is_local_player: false,
            is_locked: s.is_locked,
            is_manual_pos: false,
            matchup_win_rate: m_wr,
            profile: prof,
            counter_heroes: c_heroes,
            synergy_heroes: s_heroes,
        });
    }

    let mut rune_guide = None;
    let mut lane_counters = Vec::new();

    if let Some(ref local) = local_slot {
        if !local.champion_id.is_empty() {
            if let Some(detail) = opgg.get_champion_detail(&local.champion_id, &local.position).await {
                if let Some(runes) = detail.get("recommendedRunes") {
                    if !runes.is_null() {
                        let skill_order = detail.get("skillOrder").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        rune_guide = Some(super::win32_hud::RuneGuideView {
                            champion_name: local.champion_name.clone(),
                            position_cn: local.position_cn.clone(),
                            skill_order,
                            rune_data: runes.clone(),
                        });
                    }
                }
            }

            let enemy_slot = d.state.their_team.iter().find(|s| s.position == local.position);
            if let Some(enemy) = enemy_slot {
                if !enemy.champion_id.is_empty() {
                    let c_data = opgg.get_counter_picks_against(&enemy.champion_id, &enemy.position, Some(8)).await;
                    if let Some(c_arr) = c_data.get("counters").and_then(|v| v.as_array()) {
                        for c in c_arr {
                            let opp_cid = c.get("championId").and_then(|v| v.as_str()).unwrap_or("");
                            let opp_id_num: i32 = opp_cid.parse().unwrap_or(0);
                            if opp_id_num <= 0 {
                                continue;
                            }
                            let opp_name = db.resolve_hero_name(opp_cid).unwrap_or_else(|| format!("Hero-{}", opp_cid));
                            let c_wr = c.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                            let games = c.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                            let tier = if c_wr >= 0.53 { 1 } else if c_wr >= 0.51 { 2 } else { 3 };
                            lane_counters.push(super::win32_hud::CounterHeroItem {
                                champion_id: opp_id_num,
                                name: opp_name,
                                tier,
                                counter_win_rate: c_wr,
                                overall_win_rate: c_wr,
                                games,
                            });
                        }
                    }
                }
            }
        }
    }

    if lane_counters.is_empty() {
        let enemy_counterpart = d.state.their_team.iter().find(|s| s.position == local_pos && !s.champion_id.is_empty())
            .or_else(|| d.state.their_team.iter().find(|s| !s.champion_id.is_empty()));
        if let Some(enemy) = enemy_counterpart {
            let c_data = opgg.get_counter_picks_against(&enemy.champion_id, &local_pos, Some(8)).await;
            if let Some(c_arr) = c_data.get("counters").and_then(|v| v.as_array()) {
                for c in c_arr {
                    let opp_cid = c.get("championId").and_then(|v| v.as_str()).unwrap_or("");
                    let opp_id_num: i32 = opp_cid.parse().unwrap_or(0);
                    if opp_id_num <= 0 {
                        continue;
                    }
                    let opp_name = db.resolve_hero_name(opp_cid).unwrap_or_else(|| format!("Hero-{}", opp_cid));
                    let c_wr = c.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let games = c.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                    let tier = if c_wr >= 0.53 { 1 } else if c_wr >= 0.51 { 2 } else if c_wr > 0.0 { 3 } else { 0 };
                    lane_counters.push(super::win32_hud::CounterHeroItem {
                        champion_id: opp_id_num,
                        name: opp_name,
                        tier,
                        counter_win_rate: c_wr,
                        overall_win_rate: c_wr,
                        games,
                    });
                }
            }
        }

        if lane_counters.is_empty() && !local_pos.is_empty() {
            let tier_list = opgg.get_ranked_tier_list(false).await;
            if let Some(champs) = tier_list.get("champions").and_then(|v| v.as_object()) {
                let mut meta_picks: Vec<(i32, String, i32, f64, i64)> = Vec::new();
                for (cid_str, cval) in champs {
                    let pos_lookup = cval.get("positions").and_then(|p| {
                        p.get(&local_pos)
                            .or_else(|| if local_pos == "ADC" { p.get("BOT").or_else(|| p.get("BOTTOM")) } else { None })
                            .or_else(|| if local_pos == "SUPPORT" { p.get("UTILITY") } else { None })
                            .or_else(|| if local_pos == "MID" { p.get("MIDDLE") } else { None })
                    });
                    if let Some(pos_obj) = pos_lookup {
                        let tier = pos_obj.get("tier").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        let wr = pos_obj.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let pr = pos_obj.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let games = (pr * 1000.0).round() as i64;
                        let name = db.resolve_hero_name(cid_str).unwrap_or_else(|| format!("Hero-{}", cid_str));
                        let cid_num = cid_str.parse::<i32>().unwrap_or(0);
                        meta_picks.push((cid_num, name, tier, wr, games));
                    }
                }
                meta_picks.sort_by(|a, b| {
                    let effective_tier_a = if a.2 <= 0 { 99 } else { a.2 };
                    let effective_tier_b = if b.2 <= 0 { 99 } else { b.2 };
                    effective_tier_a.cmp(&effective_tier_b)
                        .then_with(|| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal))
                        .then_with(|| b.4.cmp(&a.4))
                        .then_with(|| a.0.cmp(&b.0))
                });
                for (cid_num, name, tier, wr, games) in meta_picks.into_iter().take(8) {
                    lane_counters.push(super::win32_hud::CounterHeroItem {
                        champion_id: cid_num,
                        name,
                        tier,
                        counter_win_rate: wr,
                        overall_win_rate: wr,
                        games,
                    });
                }
            }
        }
    }

    let rec_text = if let Some(ref local) = local_slot {
        if !local.champion_id.is_empty() {
            let enemy_slot = d.state.their_team.iter().find(|s| s.position == local.position);
            if let Some(enemy) = enemy_slot {
                if !enemy.champion_id.is_empty() {
                    format!(
                        "对位[{}]: 胜率 {:.1}% · 推荐符文已就绪 · 技能: 闪现 + 点燃",
                        enemy.champion_name,
                        local.win_rate * 100.0
                    )
                } else {
                    format!("已选[{}]: 等待敌方对位锁定 · 符文已就绪", local.champion_name)
                }
            } else {
                format!("已选[{}]: 推荐符文已就绪 · 召唤师技能: 闪现 + 点燃", local.champion_name)
            }
        } else {
            let enemy_counterpart = d.state.their_team.iter().find(|s| s.position == local_pos && !s.champion_id.is_empty());
            if let Some(enemy) = enemy_counterpart {
                format!("敌方已锁定 [{}]: 克制英雄推荐已呈现于顶部决策栏", enemy.champion_name)
            } else {
                format!("[{}] 选人思考期: 当前分路版本强势英雄已推荐于顶部", local.position_cn)
            }
        }
    } else {
        "排位选人辅助已激活 · 实时分析对位胜率".to_string()
    };

    let mut counter_map: std::collections::HashMap<i32, super::win32_hud::CounterHeroItem> = std::collections::HashMap::new();
    for s in &their_slots {
        if s.is_locked && !s.counter_heroes.is_empty() {
            for c in &s.counter_heroes {
                if c.champion_id <= 0 { continue; }
                counter_map.entry(c.champion_id)
                    .and_modify(|existing| {
                        if c.counter_win_rate > existing.counter_win_rate {
                            existing.counter_win_rate = c.counter_win_rate;
                        }
                        existing.games += c.games;
                    })
                    .or_insert_with(|| c.clone());
            }
        }
    }
    let mut multi_counters: Vec<super::win32_hud::CounterHeroItem> = counter_map.into_values().collect();
    multi_counters.sort_by(|a, b| {
        b.counter_win_rate.partial_cmp(&a.counter_win_rate).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.games.cmp(&a.games))
            .then_with(|| a.champion_id.cmp(&b.champion_id))
    });
    multi_counters.truncate(15);

    let local_synergies = my_slots.iter().find(|s| s.is_local_player).map(|s| s.synergy_heroes.clone()).unwrap_or_default();
    let synergy_picks = if !local_synergies.is_empty() {
        let mut syn_map: std::collections::HashMap<i32, super::win32_hud::CounterHeroItem> = std::collections::HashMap::new();
        for syn in local_synergies {
            if syn.champion_id <= 0 { continue; }
            syn_map.entry(syn.champion_id).or_insert(syn);
        }
        let mut syn_list: Vec<super::win32_hud::CounterHeroItem> = syn_map.into_values().collect();
        syn_list.sort_by(|a, b| {
            b.counter_win_rate.partial_cmp(&a.counter_win_rate).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.games.cmp(&a.games))
                .then_with(|| a.champion_id.cmp(&b.champion_id))
        });
        syn_list.truncate(15);
        syn_list
    } else {
        let mut syn_map: std::collections::HashMap<i32, super::win32_hud::CounterHeroItem> = std::collections::HashMap::new();
        for s in &my_slots {
            if !s.is_local_player && s.is_locked && !s.synergy_heroes.is_empty() {
                for syn in &s.synergy_heroes {
                    if syn.champion_id <= 0 { continue; }
                    syn_map.entry(syn.champion_id)
                        .and_modify(|existing| {
                            if syn.counter_win_rate > existing.counter_win_rate {
                                existing.counter_win_rate = syn.counter_win_rate;
                            }
                            existing.games += syn.games;
                        })
                        .or_insert_with(|| syn.clone());
                }
            }
        }
        let mut syn_from_team: Vec<super::win32_hud::CounterHeroItem> = syn_map.into_values().collect();
        syn_from_team.sort_by(|a, b| {
            b.counter_win_rate.partial_cmp(&a.counter_win_rate).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.games.cmp(&a.games))
                .then_with(|| a.champion_id.cmp(&b.champion_id))
        });
        syn_from_team.truncate(15);
        syn_from_team
    };

    let decision_card = super::win32_hud::RankedDecisionCard {
        mode: "counter".to_string(),
        my_locked: local_slot.as_ref().map_or(false, |l| l.is_locked),
        default_tab: if local_slot.as_ref().map_or(false, |l| l.is_locked) && rune_guide.is_some() { "runes".to_string() } else { "lane_counter".to_string() },
        rune_guide,
        lane_counters,
        multi_counters,
        synergy_picks,
    };

    let overlay_data = super::win32_hud::RankedOverlayData {
        my_team: my_slots,
        their_team: their_slots,
        decision_card,
        recommendation: rec_text,
    };

    ranked_overlay.show_ranked_draft(overlay_data.clone());
    *cached_ranked_data = Some(overlay_data);
    *last_draft_sig = current_sig;
}

