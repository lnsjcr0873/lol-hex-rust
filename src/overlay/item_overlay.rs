use super::win32_hud::{
    AnalyticsItemRow, ItemCardView, ItemOverlayData, NextItemView, TrioItemView, ACTIVE_ALGORITHM,
    GLOBAL_HUD,
};
use crate::crawler::OpggClient;
use crate::data::loader::Database;
use crate::data::recipe_data::calculate_item_net_cost;
use crate::engine::counter::CounterItemEngine;
use crate::engine::decision::DecisionEngine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    pub static ref ITEM_MANUAL_OPEN: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOverlayConfig {
    pub enabled: bool,
    pub voice_enabled: bool,
    pub hotkey: String,
}

impl Default for ItemOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            voice_enabled: false,
            hotkey: "]".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LastLiveContext {
    pub is_aram: bool,
    pub hero_id: String,
    pub hero_name: String,
    pub assigned_position: String,
    pub current_gold: f64,
    pub owned_item_ids: Vec<String>,
    pub active_augments: Vec<String>,
    pub enemies: Vec<Value>,
    pub game_time: f64,
}

#[derive(Clone)]
pub struct ItemOverlay {
    pub config: Arc<RwLock<ItemOverlayConfig>>,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
    pub manual_opened: Arc<AtomicBool>,
    pub last_toggle_instant: Arc<std::sync::Mutex<Instant>>,
    pub last_item_signature: Arc<std::sync::RwLock<String>>,
    pub cached_overlay_data: Arc<std::sync::RwLock<Option<ItemOverlayData>>>,
    pub cached_db: Arc<std::sync::RwLock<Option<Arc<Database>>>>,
    pub cached_opgg: Arc<std::sync::RwLock<Option<Arc<OpggClient>>>>,
    pub last_live_context: Arc<std::sync::RwLock<Option<LastLiveContext>>>,
    pub current_main_tab: Arc<std::sync::RwLock<String>>,
    pub current_analytics_subtab: Arc<std::sync::RwLock<String>>,
}

impl Default for ItemOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemOverlay {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ItemOverlayConfig::default())),
            is_showing: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            manual_opened: Arc::new(AtomicBool::new(false)),
            last_toggle_instant: Arc::new(std::sync::Mutex::new(Instant::now() - Duration::from_secs(10))),
            last_item_signature: Arc::new(std::sync::RwLock::new(String::new())),
            cached_overlay_data: Arc::new(std::sync::RwLock::new(None)),
            cached_db: Arc::new(std::sync::RwLock::new(None)),
            cached_opgg: Arc::new(std::sync::RwLock::new(None)),
            last_live_context: Arc::new(std::sync::RwLock::new(None)),
            current_main_tab: Arc::new(std::sync::RwLock::new("recommend".to_string())),
            current_analytics_subtab: Arc::new(std::sync::RwLock::new("featured".to_string())),
        }
    }

    pub fn set_database(&self, db: Arc<Database>) {
        if let Ok(mut g) = self.cached_db.write() {
            *g = Some(db);
        }
    }

    pub fn set_opgg(&self, opgg: Arc<OpggClient>) {
        if let Ok(mut g) = self.cached_opgg.write() {
            *g = Some(opgg);
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        GLOBAL_HUD.start();
    }

    pub fn stop(&self) {
        ITEM_MANUAL_OPEN.store(false, Ordering::SeqCst);
        self.manual_opened.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        GLOBAL_HUD.hide();
    }

    pub fn trigger_manual_scan(&self) {
        // 1. 250ms 严格硬件按键防抖，防止键盘弹跳连发
        let now = Instant::now();
        if let Ok(mut last) = self.last_toggle_instant.lock() {
            if now.duration_since(*last) < Duration::from_millis(250) {
                return;
            }
            *last = now;
        }

        // 2. 精确 Toggle 控制：以 GLOBAL_HUD.is_visible() && ITEM_MANUAL_OPEN 为基准
        let is_hud_visible = GLOBAL_HUD.is_visible();
        let is_open = ITEM_MANUAL_OPEN.load(Ordering::SeqCst);
        if is_hud_visible && is_open {
            self.stop();
            return;
        }

        // 3. 打开 HUD 并显示当前缓存数据
        ITEM_MANUAL_OPEN.store(true, Ordering::SeqCst);
        self.manual_opened.store(true, Ordering::SeqCst);
        self.is_showing.store(true, Ordering::SeqCst);

        if let Ok(guard) = self.cached_overlay_data.read() {
            if let Some(ref data) = *guard {
                GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(3600));
                return;
            }
        }

        let cur_algo = if let Ok(guard) = ACTIVE_ALGORITHM.read() {
            guard.clone()
        } else {
            "SMART".to_string()
        };

        let placeholder = ItemOverlayData {
            hero_name: "实时装备推演".to_string(),
            mode_title: "等待对局".to_string(),
            archetype_title: "🎯【推演就绪】".to_string(),
            current_gold: 0.0,
            next_item: Some(NextItemView {
                name: "等待进入局内".to_string(),
                id: "0".to_string(),
                badge: "✨ 推演就绪".to_string(),
                reason: "进入游戏后按 ] 键随时开启/关闭神装推演与散件抵扣".to_string(),
                net_cost: 0,
                total_cost: 0,
                is_affordable: false,
                gold_diff: 0,
                progress_pct: 0.0,
                gold_status_text: "💰 0/0g".to_string(),
            }),
            alternatives: Vec::new(),
            trio_items: Vec::new(),
            trio_win_rate: 0.0,
            boots_name: String::new(),
            active_algorithm: cur_algo,
            active_main_tab: "recommend".to_string(),
            analytics_subtab: "featured".to_string(),
            analytics_rows: Vec::new(),
        };
        GLOBAL_HUD.show_items(placeholder, Duration::from_secs(3600));
    }

    pub fn trigger_recalculate(&self) {
        if let Ok(mut sig) = self.last_item_signature.write() {
            *sig = String::new();
        }

        let db_opt = self.cached_db.read().ok().and_then(|g| g.clone());
        let opgg_opt = self.cached_opgg.read().ok().and_then(|g| g.clone());
        let ctx_opt = self.last_live_context.read().ok().and_then(|g| g.clone());

        if let (Some(db), Some(opgg), Some(ctx)) = (db_opt, opgg_opt, ctx_opt) {
            let self_clone = self.clone();
            tokio::spawn(async move {
                self_clone.update_recommendations_from_live(
                    &db,
                    &opgg,
                    ctx.is_aram,
                    &ctx.hero_id,
                    &ctx.hero_name,
                    &ctx.assigned_position,
                    ctx.current_gold,
                    &ctx.owned_item_ids,
                    &ctx.active_augments,
                    &ctx.enemies,
                    ctx.game_time,
                ).await;
            });
        }
    }

    pub fn switch_main_tab(&self, tab_key: &str) {
        if let Ok(mut mt) = self.current_main_tab.write() {
            *mt = tab_key.to_string();
        }
        if let Ok(mut guard) = self.cached_overlay_data.write() {
            if let Some(ref mut data) = *guard {
                data.active_main_tab = tab_key.to_string();
                if tab_key == "analytics" {
                    let subtab = if data.analytics_subtab.is_empty() {
                        "featured".to_string()
                    } else {
                        data.analytics_subtab.clone()
                    };
                    if let Ok(db_guard) = self.cached_db.read() {
                        if let Some(ref db) = *db_guard {
                            if let Ok(ctx_guard) = self.last_live_context.read() {
                                if let Some(ref ctx) = *ctx_guard {
                                    data.analytics_rows = Self::generate_analytics_rows(
                                        db,
                                        &ctx.hero_id,
                                        &subtab,
                                        ctx.is_aram,
                                    );
                                }
                            }
                        }
                    }
                }
                if ITEM_MANUAL_OPEN.load(Ordering::SeqCst) {
                    GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(3600));
                }
            }
        }
    }

    pub fn switch_analytics_subtab(&self, sub_key: &str) {
        if let Ok(mut st) = self.current_analytics_subtab.write() {
            *st = sub_key.to_string();
        }
        if let Ok(mut guard) = self.cached_overlay_data.write() {
            if let Some(ref mut data) = *guard {
                data.analytics_subtab = sub_key.to_string();
                if let Ok(db_guard) = self.cached_db.read() {
                    if let Some(ref db) = *db_guard {
                        if let Ok(ctx_guard) = self.last_live_context.read() {
                            if let Some(ref ctx) = *ctx_guard {
                                data.analytics_rows = Self::generate_analytics_rows(
                                    db,
                                    &ctx.hero_id,
                                    sub_key,
                                    ctx.is_aram,
                                );
                            }
                        }
                    }
                }
                if ITEM_MANUAL_OPEN.load(Ordering::SeqCst) {
                    GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(3600));
                }
            }
        }
    }

    pub fn generate_analytics_rows(
        db: &Database,
        hero_id: &str,
        subtab: &str,
        _is_aram: bool,
    ) -> Vec<AnalyticsItemRow> {
        let mut rows = Vec::new();
        let detail_opt = db.get_hero_detail(hero_id);
        const BOOTS_IDS: &[&str] = &["1001", "3006", "3009", "3020", "3047", "3111", "3117", "3158", "3010"];

        let get_iid = |v: &Value| -> String {
            v.get("itemId")
                .and_then(|x| x.as_str().map(|s| s.to_string()).or_else(|| x.as_i64().map(|n| n.to_string())))
                .unwrap_or_default()
        };

        if let Some(detail) = detail_opt {
            match subtab {
                "featured" => {
                    if let Some(items_arr) = detail.items.as_array() {
                        let mut candidates: Vec<&Value> = items_arr.iter()
                            .filter(|it| {
                                let iid = get_iid(it);
                                !iid.is_empty() && !BOOTS_IDS.contains(&iid.as_str())
                            })
                            .collect();
                        candidates.sort_by(|a, b| {
                            let sa = a.get("hexScore").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            let sb = b.get("hexScore").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| get_iid(a).cmp(&get_iid(b)))
                        });
                        for it in candidates.into_iter().take(5) {
                            let iid = get_iid(it);
                            let iname = it.get("itemName").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| db.resolve_item_name(&iid));
                            let wr = it.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                            let score = it.get("hexScore").and_then(|v| v.as_f64()).unwrap_or(80.0);
                            let pr = it.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.1);
                            let games = it.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                            let label = it.get("hexLabel").and_then(|v| v.as_str()).unwrap_or("T1核心").to_string();

                            rows.push(AnalyticsItemRow {
                                name: iname,
                                id: iid,
                                win_rate: wr,
                                delta_metric: score,
                                pick_rate: pr,
                                games,
                                label,
                                is_boot: false,
                            });
                        }
                    }
                }
                "items" => {
                    if let Some(items_arr) = detail.items.as_array() {
                        let mut candidates: Vec<&Value> = items_arr.iter()
                            .filter(|it| {
                                let iid = get_iid(it);
                                !iid.is_empty() && !BOOTS_IDS.contains(&iid.as_str())
                            })
                            .collect();
                        candidates.sort_by(|a, b| {
                            let pa = a.get("pickRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            let pb = b.get("pickRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| get_iid(a).cmp(&get_iid(b)))
                        });
                        for it in candidates.into_iter().take(5) {
                            let iid = get_iid(it);
                            let iname = it.get("itemName").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| db.resolve_item_name(&iid));
                            let wr = it.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                            let core_delta = it.get("coreDelta").or_else(|| it.get("deltaWinRate")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let pr = it.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.1);
                            let games = it.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                            let label = if core_delta >= 0.05 { "质变".to_string() } else { "常规".to_string() };

                            rows.push(AnalyticsItemRow {
                                name: iname,
                                id: iid,
                                win_rate: wr,
                                delta_metric: core_delta,
                                pick_rate: pr,
                                games,
                                label,
                                is_boot: false,
                            });
                        }
                    }
                }
                "boots" => {
                    if let Some(items_arr) = detail.items.as_array() {
                        let mut candidates: Vec<&Value> = items_arr.iter()
                            .filter(|it| {
                                let iid = get_iid(it);
                                BOOTS_IDS.contains(&iid.as_str())
                            })
                            .collect();
                        candidates.sort_by(|a, b| {
                            let wa = a.get("winRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            let wb = b.get("winRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                                .then_with(|| get_iid(a).cmp(&get_iid(b)))
                        });
                        for it in candidates.into_iter().take(5) {
                            let iid = it.get("itemId")
                                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                                .unwrap_or_default();
                            let iname = it.get("itemName").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| db.resolve_item_name(&iid));
                            let wr = it.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                            let dwr = it.get("deltaWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let pr = it.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.1);
                            let games = it.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                            let label = if wr >= 0.52 { "极佳".to_string() } else { "备选".to_string() };

                            rows.push(AnalyticsItemRow {
                                name: iname,
                                id: iid,
                                win_rate: wr,
                                delta_metric: dwr,
                                pick_rate: pr,
                                games,
                                label,
                                is_boot: true,
                            });
                        }
                    }
                }
                "trios" => {
                    let mut trios_list = detail.terminal_item_trios.clone();
                    trios_list.sort_by(|a, b| {
                        let wa = a.get("winRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                        let wb = b.get("winRate").and_then(|v| v.as_f64()).filter(|v| v.is_finite()).unwrap_or(0.0);
                        wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| {
                                let ka = a.get("trioKey").and_then(|v| v.as_str()).unwrap_or("");
                                let kb = b.get("trioKey").and_then(|v| v.as_str()).unwrap_or("");
                                ka.cmp(kb)
                            })
                    });
                    for (idx, trio) in trios_list.into_iter().take(5).enumerate() {
                        let names: Vec<String> = trio.get("itemNames")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
                            .unwrap_or_default();
                        let display_name = if !names.is_empty() {
                            names.join(" + ")
                        } else {
                            format!("3神装组合 #{}", idx + 1)
                        };
                        let tkey = trio.get("trioKey").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let wr = trio.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.55);
                        let dwr = trio.get("deltaWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let pr = trio.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.1);
                        let games = trio.get("games").and_then(|v| v.as_i64()).unwrap_or(0);

                        rows.push(AnalyticsItemRow {
                            name: display_name,
                            id: tkey,
                            win_rate: wr,
                            delta_metric: dwr,
                            pick_rate: pr,
                            games,
                            label: format!("神装#{}", idx + 1),
                            is_boot: false,
                        });
                    }
                }
                _ => {}
            }
        }
        rows
    }

    /// 局内后台静默推演与金币平滑刷新 (对齐 HexBrain 架构)：
    /// - 复合签名防重算：仅在英雄/已有装备/敌情变动时才重构装备推荐
    /// - 仅金币变动时：静默更新当前目标装备的散件抵扣差额与可购买状态，绝不重构排序
    /// - 手动门禁控制：当玩家手动关闭面板时，仅静默更新内存缓存，绝不擅自重新弹窗
    pub async fn update_recommendations_from_live(
        &self,
        db: &Database,
        opgg: &OpggClient,
        is_aram: bool,
        hero_id: &str,
        hero_name: &str,
        assigned_position: &str,
        current_gold: f64,
        owned_item_ids: &[String],
        active_augments: &[String],
        enemies: &[Value],
        game_time: f64,
    ) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        let current_gold = if current_gold.is_finite() && current_gold > 0.0 { current_gold } else { 0.0 };

        let resolved_id = if !hero_id.is_empty() {
            hero_id.to_string()
        } else if !hero_name.is_empty() {
            db.resolve_hero_id(hero_name).unwrap_or_default()
        } else {
            String::new()
        };

        if resolved_id.is_empty() && hero_name.is_empty() {
            return;
        }

        let resolved_name = if !hero_name.is_empty() {
            hero_name.to_string()
        } else if !resolved_id.is_empty() {
            db.resolve_hero_name(&resolved_id).unwrap_or_else(|| "当前英雄".to_string())
        } else {
            "当前英雄".to_string()
        };

        // Cache db and context for instant recalculation upon UI interaction
        if let Ok(mut g) = self.cached_db.write() {
            if g.is_none() {
                *g = Some(Arc::new(db.clone()));
            }
        }
        if let Ok(mut g) = self.last_live_context.write() {
            *g = Some(LastLiveContext {
                is_aram,
                hero_id: resolved_id.clone(),
                hero_name: resolved_name.clone(),
                assigned_position: assigned_position.to_string(),
                current_gold,
                owned_item_ids: owned_item_ids.to_vec(),
                active_augments: active_augments.to_vec(),
                enemies: enemies.to_vec(),
                game_time,
            });
        }

        let cur_algo = if let Ok(guard) = ACTIVE_ALGORITHM.read() {
            guard.clone()
        } else {
            "SMART".to_string()
        };

        // 1. 计算复合签名 (算法 + 英雄 + 分路 + 拥有装备 + 敌方阵容)
        let mut sorted_items = owned_item_ids.to_vec();
        sorted_items.sort();
        let mut sorted_augments = active_augments.to_vec();
        sorted_augments.sort();
        let enemy_keys: Vec<String> = enemies
            .iter()
            .filter_map(|e| {
                e.get("championName")
                    .or_else(|| e.get("rawChampionName"))
                    .and_then(|v| v.as_str())
            })
            .map(|s| s.to_string())
            .collect();

        let pos_key = if is_aram { "ARAM" } else { assigned_position };
        let current_sig = format!(
            "{}:{}@{}:{:?}@{:?}@{:?}",
            cur_algo, resolved_id, pos_key, sorted_items, sorted_augments, enemy_keys
        );

        let sig_changed = {
            let guard = self.last_item_signature.read().unwrap_or_else(|e| e.into_inner());
            *guard != current_sig
        };

        // 2. 若装备签名未变，仅金币发生变动：仅平滑更新差额与进度条，绝不打乱或重排装备！
        if !sig_changed {
            let safe_gold = if current_gold.is_finite() && current_gold >= 0.0 {
                current_gold
            } else {
                0.0
            };

            if let Ok(mut g) = self.last_live_context.write() {
                if let Some(ref mut ctx) = *g {
                    ctx.current_gold = safe_gold;
                }
            }
            if let Ok(mut guard) = self.cached_overlay_data.write() {
                if let Some(ref mut data) = *guard {
                    if (data.current_gold - safe_gold).abs() < 1.0 {
                        // 金币无实际变动，直接返回，0 重绘 0 刷新！
                        return;
                    }

                    data.current_gold = safe_gold;
                    if let Some(ref mut next) = data.next_item {
                        next.is_affordable = (safe_gold as i64) >= next.net_cost;
                        next.gold_diff = (next.net_cost - safe_gold as i64).max(0);
                        next.progress_pct = (safe_gold / next.net_cost.max(1) as f64).clamp(0.0, 1.0);
                        next.gold_status_text = if next.is_affordable {
                            format!("💰 {}/{}g  [🎉可直接全款]", safe_gold as i64, next.net_cost)
                        } else {
                            format!("💰 {}/{}g (差{}g)", safe_gold as i64, next.net_cost, next.gold_diff)
                        };
                    }
                    for item in &mut data.alternatives {
                        item.is_affordable = (safe_gold as i64) >= (item.net_cost as i64);
                        item.gold_diff = ((item.net_cost as i64) - (safe_gold as i64)).max(0) as i32;
                    }

                    // 仅当玩家当前手动打开时，才通知 HUD 局部刷新数值
                    if ITEM_MANUAL_OPEN.load(Ordering::SeqCst) {
                        GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(3600));
                    }
                }
            }
            return;
        }

        // 3. 签名变动：执行完整分流推演
        let mut owned_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for it in owned_item_ids {
            let clean = it.trim();
            if !clean.is_empty() {
                owned_set.insert(clean.to_string());
                let norm = crate::data::recipe_data::normalize_item_id(clean);
                if !norm.is_empty() {
                    owned_set.insert(norm);
                }
            }
        }
        let mut next_item_view: Option<NextItemView> = None;
        let mut alternative_views = Vec::new();
        let mut trio_item_views = Vec::new();
        let mut trio_win_rate = 0.0;
        let mut boots_name = String::new();
        let mode_title: String;
        let archetype_title: String;

        if is_aram {
            // === 狂暴大乱斗 (Kiwi/ARAM) 模式：HexBrain 1157万大数据海克斯联动推演 ===
            mode_title = "极地大乱斗·海克斯联动".to_string();

            let cfg = crate::config::RecommendationConfig::for_algorithm(&cur_algo);
            let rec = DecisionEngine::get_item_recommendation(
                db,
                &resolved_id,
                owned_item_ids,
                current_gold,
                1,
                active_augments,
                enemies,
                &cfg,
            );

            // 流派解析
            let arch_name = rec
                .get("archetype")
                .and_then(|a| a.get("archetype_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("专属流派");
            archetype_title = format!("🎯【{}】", arch_name);

            // 提取 NextItem
            let mut candidate_next = rec.get("nextItem").cloned();
            if candidate_next.is_none() {
                if let Some(path) = rec.get("buildPath").and_then(|v| v.as_array()) {
                    candidate_next = path.iter().find(|it| {
                        let iid = it.get("itemId")
                            .or_else(|| it.get("id"))
                            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                            .unwrap_or_default();
                        let norm = crate::data::recipe_data::normalize_item_id(&iid);
                        !iid.is_empty() && !owned_set.contains(&iid) && !owned_set.contains(&norm)
                    }).cloned();
                }
            }

            if let Some(ref it) = candidate_next {
                let iid = it.get("itemId")
                    .or_else(|| it.get("id"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                    .unwrap_or_default();
                let iname = it.get("itemName")
                    .or_else(|| it.get("name"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| db.resolve_item_name(&iid));

                let badge = it.get("counterBadge")
                    .or_else(|| it.get("synergyBadge"))
                    .or_else(|| it.get("hexLabel"))
                    .and_then(|v| v.as_str())
                    .map(|s| if s.contains("克制") { format!("⚠️ {}", s) } else { format!("✨ {}", s) })
                    .unwrap_or_else(|| "✨ T1核心".to_string());

                let reason = it.get("counterReason")
                    .or_else(|| it.get("synergyReason"))
                    .or_else(|| it.get("reason"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("海克斯协同与流派核心神装")
                    .to_string();

                let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                let gold_diff = (cost_info.net_cost - current_gold as i64).max(0);
                let progress_pct = (current_gold / cost_info.net_cost.max(1) as f64).clamp(0.0, 1.0);
                let gold_status_text = if cost_info.is_affordable {
                    format!("💰 {}/{}g  [🎉可直接全款]", current_gold as i64, cost_info.net_cost)
                } else {
                    format!("💰 {}/{}g (差{}g)", current_gold as i64, cost_info.net_cost, gold_diff)
                };

                next_item_view = Some(NextItemView {
                    name: iname,
                    id: iid,
                    badge,
                    reason,
                    net_cost: cost_info.net_cost,
                    total_cost: cost_info.total_cost,
                    is_affordable: cost_info.is_affordable,
                    gold_diff,
                    progress_pct,
                    gold_status_text,
                });
            }

            // 提取备选装备 (Alternatives)
            if let Some(path) = rec.get("buildPath").and_then(|v| v.as_array()) {
                for item_val in path.iter() {
                    let iid = item_val.get("itemId")
                        .or_else(|| item_val.get("id"))
                        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    let norm = crate::data::recipe_data::normalize_item_id(&iid);
                    if iid.is_empty() || owned_set.contains(&iid) || owned_set.contains(&norm) { continue; }
                    if let Some(ref next) = next_item_view {
                        if next.id == iid { continue; }
                    }

                    let iname = item_val.get("itemName")
                        .or_else(|| item_val.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| db.resolve_item_name(&iid));

                    let score = item_val.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(85.0);
                    let reason = item_val.get("reason").and_then(|v| v.as_str()).unwrap_or("流派备选神装").to_string();

                    let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");

                    alternative_views.push(ItemCardView {
                        name: iname,
                        id: iid,
                        net_cost: cost_info.net_cost as i32,
                        total_cost: cost_info.total_cost as i32,
                        is_affordable: cost_info.is_affordable,
                        gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                        tactical_score: score,
                        reason,
                    });
                    if alternative_views.len() >= 3 {
                        break;
                    }
                }
            }

            // 提取终局神装三件套 (Trio)
            if let Some(trio) = rec.get("recommendedTrio") {
                trio_win_rate = trio.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.55);
                if let Some(ids) = trio.get("itemIds").and_then(|v| v.as_array()) {
                    for id_val in ids.iter() {
                        let iid = id_val.as_str().unwrap_or("").to_string();
                        let iname = db.resolve_item_name(&iid);
                        let is_owned = owned_set.contains(&iid);
                        trio_item_views.push(TrioItemView {
                            name: iname,
                            is_owned,
                        });
                    }
                }
            }

            // 提取 ARAM 适配鞋
            if resolved_id == "69" {
                boots_name = "蛇女无需购鞋 (被动移速)".to_string();
            } else {
                let opgg_aram = opgg.get_champion_detail(&resolved_id, "ARAM").await;
                if let Some(bts) = opgg_aram.as_ref().and_then(|d| d.get("boots")).and_then(|v| v.as_array()) {
                    if let Some(top_b) = bts.first().and_then(|b| b.get("itemIds")).and_then(|v| v.as_array()) {
                        if let Some(bid) = top_b.first().and_then(|v| v.as_str()) {
                            let bname = db.resolve_item_name(bid);
                            let owned = owned_set.contains(bid);
                            boots_name = if owned { format!("✔{}", bname) } else { bname };
                        }
                    }
                }
                if boots_name.is_empty() {
                    boots_name = "水银之靴 / 铁板靴".to_string();
                }
            }
        } else {
            // === 召唤师峡谷 (排位/匹配) 模式：OP.GG 官方天梯数据核心出装 ===
            let pos_upper = match assigned_position.to_uppercase().as_str() {
                "TOP" => "TOP",
                "JUNGLE" => "JUNGLE",
                "MID" | "MIDDLE" => "MID",
                "ADC" | "BOTTOM" | "BOT" => "ADC",
                "SUPPORT" | "UTILITY" => "SUPPORT",
                _ => "MID",
            };
            let pos_cn = match pos_upper {
                "TOP" => "上单",
                "JUNGLE" => "打野",
                "MID" => "中单",
                "ADC" => "下路",
                "SUPPORT" => "辅助",
                _ => "峡谷",
            };
            mode_title = format!("{}·OP.GG天梯核心", pos_cn);
            archetype_title = format!("⚡【{}·OP.GG】", pos_cn);

            let opgg_detail = opgg.get_champion_detail(&resolved_id, pos_upper).await;

            let is_starter_phase = (owned_item_ids.is_empty()
                || (owned_item_ids.len() <= 2
                    && owned_item_ids.iter().all(|x| {
                        matches!(
                            x.as_str(),
                            "2003" | "2031" | "2033" | "2055" | "3340" | "3363" | "3364"
                        )
                    })))
                && game_time < 150.0;

            if is_starter_phase {
                if let Some(starters) = opgg_detail
                    .as_ref()
                    .and_then(|d| d.get("starterItems"))
                    .and_then(|v| v.as_array())
                {
                    if let Some(first_set) = starters.first().and_then(|s| s.get("itemIds")).and_then(|v| v.as_array()) {
                        for iid_val in first_set {
                            let iid = iid_val.as_str().unwrap_or("").to_string();
                            if iid.is_empty() || matches!(iid.as_str(), "2003" | "2031" | "3340") {
                                continue;
                            }
                            let iname = db.resolve_item_name(&iid);
                            let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                            let gold_diff = (cost_info.net_cost - current_gold as i64).max(0);
                            let progress_pct = (current_gold / cost_info.net_cost.max(1) as f64).clamp(0.0, 1.0);
                            let gold_status_text = if cost_info.is_affordable {
                                format!("💰 {}/{}g  [🎉可直接全款]", current_gold as i64, cost_info.net_cost)
                            } else {
                                format!("💰 {}/{}g (差{}g)", current_gold as i64, cost_info.net_cost, gold_diff)
                            };

                            next_item_view = Some(NextItemView {
                                name: iname,
                                id: iid,
                                badge: "✨ 权威出门装".to_string(),
                                reason: "OP.GG 官方天梯首选出门装套件".to_string(),
                                net_cost: cost_info.net_cost,
                                total_cost: cost_info.total_cost,
                                is_affordable: cost_info.is_affordable,
                                gold_diff,
                                progress_pct,
                                gold_status_text,
                            });
                            break;
                        }
                    }
                }
            }

            // 中后期核心装备推导
            if let Some(cores) = opgg_detail
                .as_ref()
                .and_then(|d| d.get("coreItems"))
                .and_then(|v| v.as_array())
            {
                if let Some(top_build) = cores.first() {
                    let wr = top_build.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.52);
                    trio_win_rate = wr;
                    if let Some(ids) = top_build.get("itemIds").and_then(|v| v.as_array()) {
                        for (idx, id_val) in ids.iter().enumerate() {
                            let iid = id_val.as_str().unwrap_or("").to_string();
                            if iid.is_empty() {
                                continue;
                            }
                            let iname = db.resolve_item_name(&iid);
                            let is_owned = owned_set.contains(&iid);
                            trio_item_views.push(TrioItemView {
                                name: iname.clone(),
                                is_owned,
                            });

                            if next_item_view.is_none() && !is_owned {
                                let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                                let gold_diff = (cost_info.net_cost - current_gold as i64).max(0);
                                let progress_pct = (current_gold / cost_info.net_cost.max(1) as f64).clamp(0.0, 1.0);
                                let gold_status_text = if cost_info.is_affordable {
                                    format!("💰 {}/{}g  [🎉可直接全款]", current_gold as i64, cost_info.net_cost)
                                } else {
                                    format!("💰 {}/{}g (差{}g)", current_gold as i64, cost_info.net_cost, gold_diff)
                                };

                                next_item_view = Some(NextItemView {
                                    name: iname,
                                    id: iid,
                                    badge: format!("✨ 第{}件核心", idx + 1),
                                    reason: format!("OP.GG 第 {} 件核心神装 (天梯胜率 {:.1}%)", idx + 1, wr * 100.0),
                                    net_cost: cost_info.net_cost,
                                    total_cost: cost_info.total_cost,
                                    is_affordable: cost_info.is_affordable,
                                    gold_diff,
                                    progress_pct,
                                    gold_status_text,
                                });
                            } else if !is_owned && alternative_views.len() < 3 {
                                let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                                alternative_views.push(ItemCardView {
                                    name: iname,
                                    id: iid,
                                    net_cost: cost_info.net_cost as i32,
                                    total_cost: cost_info.total_cost as i32,
                                    is_affordable: cost_info.is_affordable,
                                    gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                                    tactical_score: 90.0 - (idx as f64 * 3.0),
                                    reason: format!("天梯第 {} 件核心神装", idx + 1),
                                });
                            }
                        }
                    }
                }
            }

            // 针对性克制装补充
            let threats = CounterItemEngine::analyze_enemy_threats(enemies);
            let archetype = crate::engine::archetype::ArchetypeEngine::detect_archetype(
                &resolved_id,
                owned_item_ids,
                &[],
                &[],
                false,
            );
            let urgent_ids = CounterItemEngine::get_urgent_counter_ids(&threats, &archetype);
            for cid in urgent_ids.into_iter().take(2) {
                if !owned_set.contains(&cid)
                    && !alternative_views.iter().any(|it| it.id == cid)
                    && alternative_views.len() < 3
                {
                    let cname = match cid.as_str() {
                        "3165" => "莫雷洛秘典",
                        "3033" => "凡性的提醒",
                        "6609" => "炼金朋克链锯",
                        "3075" => "荆棘之甲",
                        "6695" => "巨蛇之牙",
                        "3036" => "多米尼克领主的致意",
                        "3135" => "虚空之杖",
                        "3139" => "水银弯刀",
                        "3157" => "中娅沙漏",
                        _ => "局势克制装",
                    };
                    let reason = if threats.has_high_heal {
                        "针对敌方高回复英雄 (重伤备选)"
                    } else if threats.has_shield_heavy {
                        "针对敌方厚重护盾 (破盾备选)"
                    } else if threats.has_super_tank {
                        "针对敌方超高双抗 (百分比穿透备选)"
                    } else {
                        "敌方态势针对备选"
                    };
                    let cost_info = calculate_item_net_cost(&cid, owned_item_ids, current_gold, "NET_DEDUCT");
                    alternative_views.push(ItemCardView {
                        name: cname.to_string(),
                        id: cid,
                        net_cost: cost_info.net_cost as i32,
                        total_cost: cost_info.total_cost as i32,
                        is_affordable: cost_info.is_affordable,
                        gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                        tactical_score: 86.0,
                        reason: reason.to_string(),
                    });
                }
            }

            // 提取 OP.GG 推荐鞋
            if resolved_id == "69" {
                boots_name = "蛇女无需购鞋 (被动移速)".to_string();
            } else {
                if let Some(bts) = opgg_detail.as_ref().and_then(|d| d.get("boots")).and_then(|v| v.as_array()) {
                    if let Some(top_b) = bts.first().and_then(|b| b.get("itemIds")).and_then(|v| v.as_array()) {
                        if let Some(bid) = top_b.first().and_then(|v| v.as_str()) {
                            let bname = db.resolve_item_name(bid);
                            let owned = owned_set.contains(bid);
                            boots_name = if owned { format!("✔{}", bname) } else { bname };
                        }
                    }
                }
                if boots_name.is_empty() {
                    boots_name = "法师之靴 / 水银之靴".to_string();
                }
            }
        }

        // 4. 构建并保存最新缓存
        let main_tab = if let Ok(guard) = self.current_main_tab.read() {
            guard.clone()
        } else {
            "recommend".to_string()
        };
        let sub_tab = if let Ok(guard) = self.current_analytics_subtab.read() {
            guard.clone()
        } else {
            "featured".to_string()
        };
        let analytics_rows = Self::generate_analytics_rows(db, &resolved_id, &sub_tab, is_aram);

        let overlay_data = ItemOverlayData {
            hero_name: resolved_name,
            mode_title,
            archetype_title,
            current_gold,
            next_item: next_item_view,
            alternatives: alternative_views,
            trio_items: trio_item_views,
            trio_win_rate,
            boots_name,
            active_algorithm: cur_algo,
            active_main_tab: main_tab,
            analytics_subtab: sub_tab,
            analytics_rows,
        };

        {
            let mut sig_guard = self.last_item_signature.write().unwrap_or_else(|e| e.into_inner());
            *sig_guard = current_sig;
        }
        {
            let mut cache_guard = self.cached_overlay_data.write().unwrap_or_else(|e| e.into_inner());
            *cache_guard = Some(overlay_data.clone());
        }

        // 5. 严格依据 ITEM_MANUAL_OPEN 决定是否将数据推送到 HUD 界面！
        // 若玩家处于关闭状态，绝对不主动弹窗！
        if ITEM_MANUAL_OPEN.load(Ordering::SeqCst) {
            GLOBAL_HUD.show_items(overlay_data, Duration::from_secs(3600));
        }
    }
}
