use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    pub config: InGameOverlayConfig,
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
            config: InGameOverlayConfig::default(),
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

    pub fn trigger_scan(&self) {
        self.is_showing.store(true, Ordering::SeqCst);
    }
}
