use super::win32_hud::{BenchChampView, BenchOverlayData, GLOBAL_HUD};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

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
    pub config: Arc<RwLock<BenchOverlayConfig>>,
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
            config: Arc::new(RwLock::new(BenchOverlayConfig::default())),
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
        if let Ok(mut gz) = super::win32_hud::GLOBAL_HOTZONES.write() {
            gz.bench_champs.clear();
        }
        GLOBAL_HUD.hide();
    }

    pub fn show_bench_heroes(&self, champions: Vec<BenchChampView>) {
        if let Ok(cfg) = self.config.try_read() {
            if !cfg.enabled {
                self.stop();
                return;
            }
        }
        if crate::vision::ScreenCapture::get_lol_game_rect().is_some() || !crate::vision::ScreenCapture::is_lol_client_active() {
            self.stop();
            return;
        }
        self.is_showing.store(true, Ordering::SeqCst);
        let data = BenchOverlayData { champions };
        // Short 2.5s auto-expiring lease continuously renewed by active ChampSelect polling loop
        GLOBAL_HUD.show_bench(data, Duration::from_millis(2500));
    }
}
