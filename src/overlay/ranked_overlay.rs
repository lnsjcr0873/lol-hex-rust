use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    pub config: RankedOverlayConfig,
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
            config: RankedOverlayConfig::default(),
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
    }
}
