use super::win32_hud::{ItemCardView, ItemOverlayData, GLOBAL_HUD};
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
        self.is_showing.store(true, Ordering::SeqCst);

        // Populate with quick recommendation status
        let placeholder = ItemOverlayData {
            hero_name: "实时出装".to_string(),
            current_gold: 0.0,
            items: vec![
                ItemCardView {
                    name: "灭世者的死亡之帽".to_string(),
                    id: "3089".to_string(),
                    net_cost: 2350,
                    total_cost: 3600,
                    is_affordable: false,
                    gold_diff: 850,
                    tactical_score: 92.0,
                    reason: "核心法强爆发 (已扣除无用大棒)".to_string(),
                },
                ItemCardView {
                    name: "虚空之杖".to_string(),
                    id: "3135".to_string(),
                    net_cost: 1750,
                    total_cost: 3000,
                    is_affordable: true,
                    gold_diff: 0,
                    tactical_score: 88.0,
                    reason: "对位高魔抗克制穿透".to_string(),
                },
            ],
        };
        GLOBAL_HUD.show_items(placeholder, Duration::from_secs(10));
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
