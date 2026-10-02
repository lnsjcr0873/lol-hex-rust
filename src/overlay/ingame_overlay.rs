use super::win32_hud::{AugmentCardView, AugmentOverlayData, GLOBAL_HUD};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InGameOverlayConfig {
    pub auto_sensing_enabled: bool,
    pub voice_enabled: bool,
    pub hotkey: String,
}

impl Default for InGameOverlayConfig {
    fn default() -> Self {
        Self {
            auto_sensing_enabled: true,
            voice_enabled: true,
            hotkey: "[".to_string(),
        }
    }
}

pub struct InGameOverlay {
    pub config: Arc<RwLock<InGameOverlayConfig>>,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
}

impl Default for InGameOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl InGameOverlay {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(InGameOverlayConfig::default())),
            is_showing: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
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

    pub fn trigger_scan(&self) {
        self.is_showing.store(true, Ordering::SeqCst);

        // Show augment HUD with placeholder or existing data
        let placeholder = AugmentOverlayData {
            hero_name: "实时推演".to_string(),
            cards: vec![],
        };
        GLOBAL_HUD.show_augments(placeholder, Duration::from_secs(12));
    }

    pub fn show_cards(&self, hero_name: &str, cards: Vec<AugmentCardView>) {
        self.is_showing.store(true, Ordering::SeqCst);
        let data = AugmentOverlayData {
            hero_name: hero_name.to_string(),
            cards,
        };
        GLOBAL_HUD.show_augments(data, Duration::from_secs(14));
    }
}
