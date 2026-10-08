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

    pub fn trigger_scan_with_db(&self, db: &Arc<crate::data::loader::Database>) {
        self.is_showing.store(true, Ordering::SeqCst);

        let placeholder = AugmentOverlayData {
            hero_name: "实时推演中...".to_string(),
            cards: vec![],
        };
        GLOBAL_HUD.show_augments(placeholder, Duration::from_secs(12));

        let db_clone = db.clone();
        tokio::spawn(async move {
            let rect = crate::vision::ScreenCapture::get_augment_title_band_rect();
            if let Some(bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(&rect) {
                let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;
                if !lines.is_empty() {
                    let matched_ids = crate::vision::OcrScanner::match_augments_from_texts(&db_clone, &lines);
                    if !matched_ids.is_empty() {
                        let mut cards = Vec::new();
                        for aid in matched_ids.iter().take(3) {
                            if let Some(aug) = db_clone.augments_map.get(aid) {
                                let wr = aug.win_rate;
                                let tier = if wr >= 0.54 {
                                    "S"
                                } else if wr >= 0.51 {
                                    "A"
                                } else if wr >= 0.48 {
                                    "B"
                                } else {
                                    "TRAP"
                                };
                                cards.push(AugmentCardView {
                                    name: aug.name.clone(),
                                    tier: tier.to_string(),
                                    win_rate: wr,
                                    delta_win_rate: aug.delta_win_rate,
                                    wilson_lower: aug.wilson_lower,
                                    games: aug.games,
                                    recommendation: format!("胜率指数: {:.1}%", wr * 100.0),
                                });
                            }
                        }
                        if !cards.is_empty() {
                            let data = AugmentOverlayData {
                                hero_name: "海克斯强化推演".to_string(),
                                cards,
                            };
                            GLOBAL_HUD.show_augments(data, Duration::from_secs(14));
                        }
                    }
                }
            }
        });
    }

    pub fn trigger_scan(&self) {
        self.is_showing.store(true, Ordering::SeqCst);
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
