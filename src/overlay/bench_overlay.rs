use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchOverlayConfig {
    pub enabled: bool,
    pub click_to_swap: bool,
    pub badge_position: String,
    pub vertical_offset: i32,
    pub horizontal_offset: i32,
}

impl Default for BenchOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            click_to_swap: true,
            badge_position: "bottom".to_string(),
            vertical_offset: 0,
            horizontal_offset: 0,
        }
    }
}

pub struct BenchOverlay {
    pub config: BenchOverlayConfig,
    pub is_showing: Arc<AtomicBool>,
    pub running: Arc<AtomicBool>,
}

impl Default for BenchOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchOverlay {
    pub fn new() -> Self {
        Self {
            config: BenchOverlayConfig::default(),
            is_showing: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        // Background window loop manages WS_EX_LAYERED | WS_EX_NOACTIVATE window
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
    }
}
