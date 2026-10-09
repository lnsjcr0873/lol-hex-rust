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
            hero_name: "正在感知强化卡片...".to_string(),
            cards: vec![],
        };
        GLOBAL_HUD.show_augments(placeholder, Duration::from_secs(12));

        let db_clone = db.clone();
        crate::spawn_task(async move {
            let mut matched_ids = Vec::new();
            let mut recognized_any_text = false;

            let show_anvil_card = || {
                crate::overlay::TextToSpeech::speak("检测到属性锻造卡片，建议根据当前英雄主要收益属性进行选择");
                let anvil_card = AugmentCardView {
                    name: "属性锻造器".to_string(),
                    tier: "A".to_string(),
                    win_rate: 0.50,
                    delta_win_rate: 0.0,
                    wilson_lower: 0.49,
                    games: 100000,
                    recommendation: "按英雄属性优先级选择 (攻击/法强/双抗/极速)".to_string(),
                };
                let data = AugmentOverlayData {
                    hero_name: "属性卡推演".to_string(),
                    cards: vec![anvil_card],
                };
                GLOBAL_HUD.show_augments(data, Duration::from_secs(10));
            };

            // Level 1: 标题横带物理感知（带20px安全黑边padding）
            let band_rect = crate::vision::ScreenCapture::get_augment_title_band_rect();
            if let Some(bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(&band_rect) {
                tracing::info!("Augment OCR L1: Captured title band ({}x{}), scanning...", band_rect.width, band_rect.height);
                let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;
                if !lines.is_empty() {
                    recognized_any_text = true;
                    tracing::info!("Augment OCR L1: Recognized lines: {:?}", lines);

                    if crate::vision::OcrScanner::is_stat_anvil_cards(&lines) {
                        show_anvil_card();
                        return;
                    }

                    let ids = crate::vision::OcrScanner::match_augments_from_texts(&db_clone, &lines);
                    for id in ids {
                        if !matched_ids.contains(&id) {
                            matched_ids.push(id);
                        }
                    }
                }
            }

            // Level 2: 独立 3 槽位物理嗅探（如果未能完全识别出 3 张）
            if matched_ids.len() < 3 {
                let slot_rects = crate::vision::ScreenCapture::get_augment_slot_rects();
                for (idx, rect) in slot_rects.iter().enumerate() {
                    if let Some(bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(rect) {
                        let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;
                        if !lines.is_empty() {
                            recognized_any_text = true;
                            tracing::info!("Augment OCR L2: Slot {} recognized lines: {:?}", idx + 1, lines);
                            if crate::vision::OcrScanner::is_stat_anvil_cards(&lines) {
                                show_anvil_card();
                                return;
                            }
                            let ids = crate::vision::OcrScanner::match_augments_from_texts(&db_clone, &lines);
                            for id in ids {
                                if !matched_ids.contains(&id) {
                                    matched_ids.push(id);
                                }
                            }
                        }
                    }
                }
            }

            // Level 3: 动效避让与延迟重试（若仍未命中任何卡片）
            if matched_ids.is_empty() {
                tokio::time::sleep(Duration::from_millis(350)).await;
                let band_rect = crate::vision::ScreenCapture::get_augment_title_band_rect();
                if let Some(bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(&band_rect) {
                    let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;
                    if !lines.is_empty() {
                        recognized_any_text = true;
                        tracing::info!("Augment OCR L3 (retry): Recognized lines: {:?}", lines);
                        if crate::vision::OcrScanner::is_stat_anvil_cards(&lines) {
                            show_anvil_card();
                            return;
                        }
                        let ids = crate::vision::OcrScanner::match_augments_from_texts(&db_clone, &lines);
                        for id in ids {
                            if !matched_ids.contains(&id) {
                                matched_ids.push(id);
                            }
                        }
                    }
                }
            }

            // Level 4: 中央大区域全局语义回退（若卡片依然不足 2 张）
            if matched_ids.len() < 2 {
                let center_rect = crate::vision::ScreenCapture::get_augment_center_rect();
                if let Some(bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(&center_rect) {
                    let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;
                    if !lines.is_empty() {
                        recognized_any_text = true;
                        tracing::info!("Augment OCR L4 (center): Recognized lines: {:?}", lines);
                        if crate::vision::OcrScanner::is_stat_anvil_cards(&lines) {
                            show_anvil_card();
                            return;
                        }
                        let ids = crate::vision::OcrScanner::match_augments_from_texts(&db_clone, &lines);
                        for id in ids {
                            if !matched_ids.contains(&id) {
                                matched_ids.push(id);
                            }
                        }
                    }
                }
            }

            // 结果匹配与渲染
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
                            delta_win_rate: aug.avg_delta_win_rate.unwrap_or(0.0),
                            wilson_lower: crate::engine::decision::calc_wilson_lower_bound(wr, aug.games, 1.96),
                            games: aug.games,
                            recommendation: format!("胜率指数: {:.1}%", wr * 100.0),
                        });
                    }
                }

                if !cards.is_empty() {
                    if let Some(best) = cards.iter().max_by(|a, b| a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal)) {
                        crate::overlay::TextToSpeech::speak(&format!("建议优选{}，胜率{:.0}%", best.name, best.win_rate * 100.0));
                    }
                    let data = AugmentOverlayData {
                        hero_name: "海克斯强化推演".to_string(),
                        cards,
                    };
                    GLOBAL_HUD.show_augments(data, Duration::from_secs(14));
                    return;
                }
            }

            // 识别失败兜底提示（绝对不静默无反应）
            let error_msg = if recognized_any_text {
                "未能匹配到强化卡，请确保处于三选一界面"
            } else {
                "未检测到强化卡片，请在三选一界面按【"
            };
            crate::overlay::TextToSpeech::speak(error_msg);
            let error_data = AugmentOverlayData {
                hero_name: error_msg.to_string(),
                cards: vec![],
            };
            GLOBAL_HUD.show_augments(error_data, Duration::from_secs(4));
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
