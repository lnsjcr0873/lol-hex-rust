use super::win32_hud::{ItemCardView, ItemOverlayData, GLOBAL_HUD};
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

pub struct ItemOverlay {
    pub config: Arc<RwLock<ItemOverlayConfig>>,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
    pub manual_opened: Arc<AtomicBool>,
    pub last_toggle_instant: Arc<std::sync::Mutex<Instant>>,
    pub last_item_signature: Arc<std::sync::RwLock<String>>,
    pub cached_overlay_data: Arc<std::sync::RwLock<Option<ItemOverlayData>>>,
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
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        GLOBAL_HUD.start();
    }

    pub fn stop(&self) {
        self.manual_opened.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        GLOBAL_HUD.hide();
    }

    pub fn trigger_manual_scan(&self) {
        // 1. 350ms 严格硬件按键防抖，防止键盘弹跳连发
        let now = Instant::now();
        if let Ok(mut last) = self.last_toggle_instant.lock() {
            if now.duration_since(*last) < Duration::from_millis(350) {
                return;
            }
            *last = now;
        }

        // 2. 精确 Toggle 控制
        let is_open = self.manual_opened.load(Ordering::SeqCst) || self.is_showing.load(Ordering::SeqCst);
        if is_open {
            self.stop();
            return;
        }

        // 3. 打开 HUD 并显示当前缓存数据
        self.manual_opened.store(true, Ordering::SeqCst);
        self.is_showing.store(true, Ordering::SeqCst);

        if let Ok(guard) = self.cached_overlay_data.read() {
            if let Some(ref data) = *guard {
                GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(60));
                return;
            }
        }

        let placeholder = ItemOverlayData {
            hero_name: "实时装备推演".to_string(),
            mode_title: "等待对局".to_string(),
            current_gold: 0.0,
            items: vec![
                ItemCardView {
                    name: "等待进入局内".to_string(),
                    id: "0".to_string(),
                    net_cost: 0,
                    total_cost: 0,
                    is_affordable: false,
                    gold_diff: 0,
                    tactical_score: 50.0,
                    reason: "进入游戏后按 ] 键随时开启/关闭神装推演与散件抵扣".to_string(),
                },
            ],
            trio_names: Vec::new(),
            trio_completed: 0,
            boots_name: String::new(),
        };
        GLOBAL_HUD.show_items(placeholder, Duration::from_secs(60));
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

        // 1. 计算复合签名 (英雄 + 分路 + 拥有装备 + 敌方阵容)
        let mut sorted_items = owned_item_ids.to_vec();
        sorted_items.sort();
        let mut sorted_augments = active_augments.to_vec();
        sorted_augments.sort();
        let enemy_keys: Vec<String> = enemies.iter()
            .filter_map(|e| e.get("championName").or_else(|| e.get("rawChampionName")).and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .collect();

        let pos_key = if is_aram { "ARAM" } else { assigned_position };
        let current_sig = format!(
            "{}@{}:{:?}@{:?}@{:?}",
            resolved_id,
            pos_key,
            sorted_items,
            sorted_augments,
            enemy_keys
        );

        let sig_changed = {
            let guard = self.last_item_signature.read().unwrap();
            *guard != current_sig
        };

        // 2. 若装备签名未变，仅金币发生变动：仅平滑更新差额与可合成标记，绝不打乱或重排装备！
        if !sig_changed {
            if let Ok(mut guard) = self.cached_overlay_data.write() {
                if let Some(ref mut data) = *guard {
                    data.current_gold = current_gold;
                    for item in &mut data.items {
                        item.is_affordable = (current_gold as i64) >= (item.net_cost as i64);
                        item.gold_diff = ((item.net_cost as i64) - (current_gold as i64)).max(0) as i32;
                    }
                    if self.manual_opened.load(Ordering::SeqCst) {
                        GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(60));
                    }
                }
            }
            return;
        }

        // 3. 签名变动：执行完整分流推演
        let mut item_views = Vec::new();
        let mut trio_names = Vec::new();
        let mut trio_completed = 0usize;
        let mut boots_name = String::new();
        let mode_title: String;

        let owned_set: std::collections::HashSet<String> = owned_item_ids.iter().cloned().collect();

        if is_aram {
            // === 狂暴大乱斗 (Kiwi/ARAM) 模式：HexBrain 1157万大数据海克斯联动推演 ===
            mode_title = "极地大乱斗·海克斯联动".to_string();

            let default_cfg = crate::engine::decision::RecommendationConfig::default();
            let rec = DecisionEngine::get_item_recommendation(
                db,
                &resolved_id,
                owned_item_ids,
                current_gold,
                1,
                active_augments,
                enemies,
                &default_cfg,
            );

            // 提取推荐神装路径
            if let Some(path) = rec.get("buildPath").and_then(|v| v.as_array()) {
                for item_val in path.iter().take(4) {
                    let iid = item_val.get("itemId")
                        .or_else(|| item_val.get("id"))
                        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    if iid.is_empty() { continue; }

                    let iname = item_val.get("itemName")
                        .or_else(|| item_val.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| db.resolve_item_name(&iid));

                    let score = item_val.get("tacticalScore").and_then(|v| v.as_f64()).unwrap_or(85.0);
                    let reason = item_val.get("reason").and_then(|v| v.as_str()).unwrap_or("海克斯协同与流派推荐神装").to_string();

                    let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");

                    item_views.push(ItemCardView {
                        name: iname,
                        id: iid,
                        net_cost: cost_info.net_cost as i32,
                        total_cost: cost_info.total_cost as i32,
                        is_affordable: cost_info.is_affordable,
                        gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                        tactical_score: score,
                        reason,
                    });
                }
            }

            // 提取终局神装三件套
            if let Some(trio) = rec.get("recommendedTrio") {
                if let Some(names) = trio.get("itemNames").and_then(|v| v.as_array()) {
                    for n in names.iter().filter_map(|v| v.as_str()) {
                        trio_names.push(n.to_string());
                    }
                } else if let Some(ids) = trio.get("itemIds").and_then(|v| v.as_array()) {
                    for id_str in ids.iter().filter_map(|v| v.as_str()) {
                        trio_names.push(db.resolve_item_name(id_str));
                    }
                }
                if let Some(ids) = trio.get("itemIds").and_then(|v| v.as_array()) {
                    for id_str in ids.iter().filter_map(|v| v.as_str()) {
                        if owned_set.contains(id_str) {
                            trio_completed += 1;
                        }
                    }
                }
            }

            // 提取 ARAM 适配鞋
            let opgg_aram = opgg.get_champion_detail(&resolved_id, "ARAM").await;
            if let Some(bts) = opgg_aram.as_ref().and_then(|d| d.get("boots")).and_then(|v| v.as_array()) {
                if let Some(top_b) = bts.first().and_then(|b| b.get("itemIds")).and_then(|v| v.as_array()) {
                    if let Some(bid) = top_b.first().and_then(|v| v.as_str()) {
                        boots_name = db.resolve_item_name(bid);
                    }
                }
            }
            if boots_name.is_empty() {
                boots_name = "水银之靴 / 铁板靴".to_string();
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

            let opgg_detail = opgg.get_champion_detail(&resolved_id, pos_upper).await;

            let is_starter_phase = (owned_item_ids.is_empty()
                || (owned_item_ids.len() <= 2 && owned_item_ids.iter().all(|x| matches!(x.as_str(), "2003" | "2031" | "2033" | "2055" | "3340" | "3363" | "3364"))))
                && game_time < 150.0;

            if is_starter_phase {
                // 开局泉水：推荐出门装套件
                if let Some(starters) = opgg_detail.as_ref().and_then(|d| d.get("starterItems")).and_then(|v| v.as_array()) {
                    if let Some(first_set) = starters.first().and_then(|s| s.get("itemIds")).and_then(|v| v.as_array()) {
                        for iid_val in first_set {
                            let iid = iid_val.as_str().unwrap_or("").to_string();
                            if iid.is_empty() || matches!(iid.as_str(), "2003" | "2031" | "3340") { continue; }
                            let iname = db.resolve_item_name(&iid);
                            let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                            item_views.push(ItemCardView {
                                name: iname,
                                id: iid,
                                net_cost: cost_info.net_cost as i32,
                                total_cost: cost_info.total_cost as i32,
                                is_affordable: cost_info.is_affordable,
                                gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                                tactical_score: 99.0,
                                reason: "OP.GG 官方天梯首选出门装套件".to_string(),
                            });
                        }
                    }
                }
            }

            // 中后期核心装备推导
            if let Some(cores) = opgg_detail.as_ref().and_then(|d| d.get("coreItems")).and_then(|v| v.as_array()) {
                if let Some(top_build) = cores.first() {
                    let wr = top_build.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.52);
                    if let Some(ids) = top_build.get("itemIds").and_then(|v| v.as_array()) {
                        for (idx, id_val) in ids.iter().enumerate() {
                            let iid = id_val.as_str().unwrap_or("").to_string();
                            if iid.is_empty() { continue; }
                            let iname = db.resolve_item_name(&iid);
                            trio_names.push(iname.clone());
                            if owned_set.contains(&iid) {
                                trio_completed += 1;
                            } else if item_views.len() < 3 {
                                let cost_info = calculate_item_net_cost(&iid, owned_item_ids, current_gold, "NET_DEDUCT");
                                item_views.push(ItemCardView {
                                    name: iname,
                                    id: iid,
                                    net_cost: cost_info.net_cost as i32,
                                    total_cost: cost_info.total_cost as i32,
                                    is_affordable: cost_info.is_affordable,
                                    gold_diff: ((cost_info.net_cost - current_gold as i64).max(0)) as i32,
                                    tactical_score: 90.0 - (idx as f64 * 3.0),
                                    reason: format!("OP.GG 第 {} 件核心神装 (天梯胜率 {:.1}%)", idx + 1, wr * 100.0),
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
                if !owned_set.contains(&cid) && !item_views.iter().any(|it| it.id == cid) && item_views.len() < 4 {
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
                    item_views.push(ItemCardView {
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
            if let Some(bts) = opgg_detail.as_ref().and_then(|d| d.get("boots")).and_then(|v| v.as_array()) {
                if let Some(top_b) = bts.first().and_then(|b| b.get("itemIds")).and_then(|v| v.as_array()) {
                    if let Some(bid) = top_b.first().and_then(|v| v.as_str()) {
                        boots_name = db.resolve_item_name(bid);
                    }
                }
            }
        }

        // 4. 构建并保存最新缓存
        let overlay_data = ItemOverlayData {
            hero_name: resolved_name,
            mode_title,
            current_gold,
            items: item_views,
            trio_names,
            trio_completed,
            boots_name,
        };

        {
            let mut sig_guard = self.last_item_signature.write().unwrap();
            *sig_guard = current_sig;
        }
        {
            let mut cache_guard = self.cached_overlay_data.write().unwrap();
            *cache_guard = Some(overlay_data.clone());
        }

        // 5. 严格依据 manual_opened 决定是否将数据推送到 HUD 界面！
        // 若玩家手动处于关闭状态，绝对不主动弹窗！
        if self.manual_opened.load(Ordering::SeqCst) {
            GLOBAL_HUD.show_items(overlay_data, Duration::from_secs(60));
        }
    }
}
