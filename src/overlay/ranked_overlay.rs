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

    pub fn trigger_test(&self, data: super::win32_hud::RankedOverlayData, duration: std::time::Duration) {
        if let Ok(mut guard) = self.test_mode_until.lock() {
            *guard = Some(std::time::Instant::now() + duration);
        }
        self.is_showing.store(true, Ordering::SeqCst);
        super::win32_hud::GLOBAL_HUD.show_ranked_test(data, duration);
    }
}
