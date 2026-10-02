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
    pub config: Arc<RwLock<RankedOverlayConfig>>,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
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
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.hide();
    }

    pub fn show_ranked_draft(&self, data: super::win32_hud::RankedOverlayData) {
        self.is_showing.store(true, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.show_ranked(data, std::time::Duration::from_secs(30));
    }
}
