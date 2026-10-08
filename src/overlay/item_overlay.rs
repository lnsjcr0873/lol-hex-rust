use super::win32_hud::{ItemCardView, ItemOverlayData, GLOBAL_HUD};
use crate::data::loader::Database;
use crate::data::recipe_data::calculate_item_net_cost;
use crate::engine::counter::CounterItemEngine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
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
    pub last_recommendation: Arc<RwLock<Option<Value>>>,
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
            last_recommendation: Arc::new(RwLock::new(None)),
            cached_overlay_data: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        GLOBAL_HUD.start();
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        GLOBAL_HUD.hide();
    }

    pub fn trigger_manual_scan(&self) {
        let is_showing = self.is_showing.load(Ordering::SeqCst);
        if is_showing {
            // Toggle off if already showing
            self.stop();
            return;
        }

        self.is_showing.store(true, Ordering::SeqCst);

        // 1. Try displaying cached real-time live recommendation data
        if let Ok(guard) = self.cached_overlay_data.read() {
            if let Some(ref data) = *guard {
                GLOBAL_HUD.show_items(data.clone(), Duration::from_secs(12));
                return;
            }
        }

        // 2. If no game is running or stats haven't arrived, display graceful status card
        let placeholder = ItemOverlayData {
            hero_name: "实时装备推演".to_string(),
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
                    reason: "进入游戏后按 ] 键自动推演当局敌方克制神装与散件抵扣".to_string(),
                },
            ],
        };
        GLOBAL_HUD.show_items(placeholder, Duration::from_secs(8));
    }

    /// Dynamically recalculate recommended items based on in-game 2999 stats
    pub fn update_recommendations_from_live(
        &self,
        db: &Database,
        hero_id: &str,
        hero_name: &str,
        current_gold: f64,
        owned_item_ids: &[String],
        enemies: &[Value],
    ) {
        // 1. Threat Analysis on Enemy Team
        let threats = CounterItemEngine::analyze_enemy_threats(enemies);

        // 2. Resolve hero archetype info
        let resolved_name = if hero_name.is_empty() {
            db.resolve_hero_name(hero_id).unwrap_or_else(|| "当前英雄".to_string())
        } else {
            hero_name.to_string()
        };

        let archetype = crate::engine::archetype::ArchetypeEngine::get_hero_archetype_info(hero_id);
        let urgent_ids = CounterItemEngine::get_urgent_counter_ids(&threats, &archetype);

        let mut item_views = Vec::new();

        // 3. Populate urgent counter items first
        for item_id in urgent_ids.iter().take(3) {
            let item_cost = calculate_item_net_cost(item_id, owned_item_ids, current_gold, "NET_DEDUCT");
            let iname = match item_id.as_str() {
                "3165" => "莫雷洛秘典",
                "3033" => "凡性的提醒",
                "3814" => "巨蛇之牙",
                "3036" => "多米尼克领主的致意",
                "3135" => "虚空之杖",
                "3157" => "中娅沙漏",
                _ => "针对克制核心",
            };
            let reason = if threats.has_high_heal && (item_id == "3165" || item_id == "3033") {
                "针对敌方高回复 (重伤抵扣散件)"
            } else if threats.has_heavy_shield && item_id == "3814" {
                "针对敌方厚重护盾 (破盾破阵)"
            } else if threats.has_super_tank {
                "针对敌方超高护甲魔抗 (百分比穿透)"
            } else {
                "实时克制出装推荐"
            };

            item_views.push(ItemCardView {
                name: iname.to_string(),
                id: item_id.clone(),
                net_cost: item_cost.net_cost,
                total_cost: item_cost.total_cost,
                is_affordable: item_cost.is_affordable,
                gold_diff: (item_cost.net_cost - current_gold as i32).max(0),
                tactical_score: 95.0,
                reason: reason.to_string(),
            });
        }

        // 4. Update memory cache
        let overlay_data = ItemOverlayData {
            hero_name: resolved_name,
            current_gold,
            items: item_views,
        };

        if let Ok(mut guard) = self.cached_overlay_data.write() {
            *guard = Some(overlay_data.clone());
        }

        // If HUD is currently popped open, immediately refresh the view
        if self.is_showing.load(Ordering::SeqCst) {
            GLOBAL_HUD.show_items(overlay_data, Duration::from_secs(12));
        }
    }

    pub fn show_item_recommendations(&self, hero_name: &str, current_gold: f64, items: Vec<ItemCardView>) {
        self.is_showing.store(true, Ordering::SeqCst);
        let data = ItemOverlayData {
            hero_name: hero_name.to_string(),
            current_gold,
            items,
        };
        GLOBAL_HUD.show_items(data, Duration::from_secs(12));
    }
}
