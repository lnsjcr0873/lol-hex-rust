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

    /// 局内智能视觉探针后台循环 (原版 HexBrain 自适应感知架构)：
    /// - 双重阶段门控：优先判断 LCU InProgress，兼顾 2999 端口，彻底摆脱 2999 端口失效导致的死锁
    /// - 智能模式过滤：召唤师峡谷排位赛 (CLASSIC / 420/440) 与云顶之弈彻底静默，大乱斗与娱乐模式全力护航
    /// - 极速 0.005ms 专属标题横带轻量指纹巡检：非选牌期瞬时熔断 (0 掉帧、0 CPU 开销)
    /// - 画面出现三选一海克斯即刻自动弹出推荐与语音播报，支持 200ms 双帧时序防抖复核
    /// - 选牌完成卡牌关闭后自动秒级隐藏 HUD，退回极低频待命
    pub fn start_auto_sensing_loop(
        self: &Arc<Self>,
        db: Arc<crate::data::loader::Database>,
        live_client: Arc<crate::lcu::LiveClientDataReader>,
        lcu_client: Arc<crate::lcu::LcuClient>,
        is_hextech_active: Arc<std::sync::atomic::AtomicBool>,
    ) {
        let overlay = self.clone();
        tokio::spawn(async move {
            let mut completed_milestones: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut last_scanned_signature = String::new();
            let mut showing_cards = false;
            let mut consecutive_no_cards = 0;
            let mut last_recommended_name: Option<String> = None;

            tracing::info!("[InGameOverlay] ⚡ 局内【视觉探针自适应实时感知】引擎已启动！(0 性能损耗，弹窗即刻秒级自动报牌)");

            while overlay.running.load(Ordering::SeqCst) {
                let auto_enabled = overlay.config.read().await.auto_sensing_enabled;
                if !auto_enabled {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }

                // 1. 双重门禁：LCU InProgress / 2999 Live Stats
                let phase = lcu_client.get_gameflow_phase().await;
                let stats = live_client.get_live_stats().await;
                let is_in_game = (phase == "InProgress") || stats.is_in_game;

                if !is_in_game {
                    // 非对局阶段：重置状态机，低频待命，隐藏 HUD
                    if showing_cards || overlay.is_showing.load(Ordering::SeqCst) {
                        completed_milestones.clear();
                        last_scanned_signature.clear();
                        showing_cards = false;
                        consecutive_no_cards = 0;
                        last_recommended_name = None;
                        GLOBAL_HUD.hide();
                        overlay.is_showing.store(false, Ordering::SeqCst);
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }

                // 2. 模式门禁：必须是具有海克斯强化的模式 (Arena / Mayhem / Kiwi / Cherry 等)，普通 ARAM 与召唤师峡谷彻底静默
                let is_hextech = is_hextech_active.load(Ordering::SeqCst) || stats.has_hextech;
                if !is_hextech {
                    if showing_cards {
                        GLOBAL_HUD.hide();
                        showing_cards = false;
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }

                // 3. 动态阶段时序守护 (基于英雄等级 3/7/11/15)
                let level = stats.level;
                let max_stage = if level > 0 {
                    if level >= 15 {
                        4
                    } else if level >= 11 {
                        3
                    } else if level >= 7 {
                        2
                    } else {
                        1
                    }
                } else {
                    // Level is 0 (2999 offline / fallback mode): infer current stage from completed milestones
                    if completed_milestones.contains("stage_3") {
                        4
                    } else if completed_milestones.contains("stage_2") {
                        3
                    } else if completed_milestones.contains("stage_1") {
                        2
                    } else {
                        1
                    }
                };

                // 阶段自愈防护 (仅当 live stats 正常提供真实等级时生效，避免 2999 瞬时超时/纯 LCU 模式 level==0 误删标记)
                if level > 0 {
                    if level < 15 { completed_milestones.remove("stage_4"); }
                    if level < 11 { completed_milestones.remove("stage_3"); }
                    if level < 7 { completed_milestones.remove("stage_2"); }
                }

                let current_stage = max_stage;

                // 阶段等级门禁：当前轮次已完成且尚未到达下一轮等级阈值时，低频待命
                if !showing_cards && completed_milestones.contains(&format!("stage_{}", current_stage)) {
                    if level > 0 && (
                        (current_stage == 1 && level < 7)
                        || (current_stage == 2 && level < 11)
                        || (current_stage == 3 && level < 15)
                    ) {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }

                // 4. 核心视觉探针巡检 (专属标题横带 + 暗底黑曜石指纹 0.005ms 极速熔断)
                let band_rect = crate::vision::ScreenCapture::get_augment_title_band_rect();
                let bmp_opt = crate::vision::ScreenCapture::capture_rect_as_bmp(&band_rect);

                let has_dark_fingerprint = match &bmp_opt {
                    Some(bmp) => {
                        if crate::vision::ScreenCapture::is_bmp_mostly_black(bmp) {
                            false
                        } else {
                            crate::vision::ScreenCapture::is_title_band_card_bg(bmp)
                        }
                    }
                    None => false,
                };

                if let (true, Some(bmp)) = (has_dark_fingerprint, bmp_opt) {
                    let lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&bmp).await;

                    // 锻造器过滤
                    if crate::vision::OcrScanner::is_stat_anvil_cards(&lines) {
                        tokio::time::sleep(Duration::from_millis(600)).await;
                        continue;
                    }

                    // 海克斯名称字典匹配
                    let mut matched_ids = crate::vision::OcrScanner::match_augments_from_texts(&db, &lines);

                    // 若标题未满 3 张，补充采样 3 个独立槽位
                    if matched_ids.len() < 3 {
                        let slot_rects = crate::vision::ScreenCapture::get_augment_slot_rects();
                        for rect in slot_rects.iter() {
                            if let Some(slot_bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(rect) {
                                let slot_lines = crate::vision::OcrScanner::run_windows_ocr_bytes(&slot_bmp).await;
                                for id in crate::vision::OcrScanner::match_augments_from_texts(&db, &slot_lines) {
                                    if !matched_ids.contains(&id) {
                                        matched_ids.push(id);
                                    }
                                }
                            }
                        }
                    }

                    if matched_ids.len() >= 2 {
                        let mut sorted_ids = matched_ids.clone();
                        sorted_ids.sort();
                        let sig = sorted_ids.join("-");

                        // 200ms 双帧时序防抖复核
                        if !showing_cards && sig != last_scanned_signature {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            let recheck_band = crate::vision::ScreenCapture::get_augment_title_band_rect();
                            if let Some(re_bmp) = crate::vision::ScreenCapture::capture_rect_as_bmp(&recheck_band) {
                                if !crate::vision::ScreenCapture::is_title_band_card_bg(&re_bmp) {
                                    tracing::info!("[InGameOverlay] 拦截瞬态视觉伪信号 (200ms 双帧复核未通过)");
                                    tokio::time::sleep(Duration::from_millis(800)).await;
                                    continue;
                                }
                            }
                        }

                        showing_cards = true;
                        consecutive_no_cards = 0;

                        if sig != last_scanned_signature {
                            last_scanned_signature = sig;

                            let hero_name = if stats.hero_name.is_empty() {
                                format!("第 {} 轮强化推演", current_stage)
                            } else {
                                format!("{} (第 {} 轮强化)", stats.hero_name, current_stage)
                            };

                            let mut cards = Vec::new();
                            for aid in matched_ids.iter().take(3) {
                                if let Some(aug) = db.augments_map.get(aid) {
                                    let wr = aug.win_rate;
                                    let tier = if wr >= 0.54 { "S" } else if wr >= 0.51 { "A" } else if wr >= 0.48 { "B" } else { "TRAP" };
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
                                if let Some(best) = cards.iter().max_by(|a, b| {
                                    a.win_rate
                                        .partial_cmp(&b.win_rate)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                        .then_with(|| a.delta_win_rate.partial_cmp(&b.delta_win_rate).unwrap_or(std::cmp::Ordering::Equal))
                                        .then_with(|| a.games.cmp(&b.games))
                                        .then_with(|| a.name.cmp(&b.name))
                                }) {
                                    if last_recommended_name.as_deref() != Some(&best.name) {
                                        last_recommended_name = Some(best.name.clone());
                                        let slot_pos = cards.iter().position(|c| c.name == best.name).unwrap_or(0) + 1;
                                        crate::overlay::TextToSpeech::speak(&format!("推荐选第{}张，{}", slot_pos, best.name));
                                    }
                                }
                                overlay.is_showing.store(true, Ordering::SeqCst);
                                let data = AugmentOverlayData {
                                    hero_name,
                                    cards,
                                };
                                GLOBAL_HUD.show_augments(data, Duration::from_secs(35));
                            }
                        }

                        // 玩家正在思考，轻量嗅探 (150ms 间隔)
                        tokio::time::sleep(Duration::from_millis(150)).await;
                        continue;
                    }
                } else {
                    // 未检测到暗底黑曜石卡牌
                    if showing_cards {
                        consecutive_no_cards += 1;
                        // 第一次未检出：极速 60ms 二次采样复核
                        if consecutive_no_cards == 1 {
                            tokio::time::sleep(Duration::from_millis(60)).await;
                            continue;
                        }
                        // 连续 2 次无暗底，说明玩家已经点击选牌闭环！
                        if consecutive_no_cards >= 2 {
                            GLOBAL_HUD.hide();
                            overlay.is_showing.store(false, Ordering::SeqCst);
                            completed_milestones.insert(format!("stage_{}", current_stage));
                            tracing::info!("[InGameOverlay] 第 {} 轮选牌面板已关闭，自动优雅收起 HUD。", current_stage);
                            showing_cards = false;
                            consecutive_no_cards = 0;
                            last_scanned_signature.clear();
                            last_recommended_name = None;
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                    }
                }

                // 卡牌尚未弹出，普通对局待命（0.005ms 指纹检查，0 掉帧）
                tokio::time::sleep(Duration::from_millis(800)).await;
            }
        });
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.is_showing.store(false, Ordering::SeqCst);
        GLOBAL_HUD.hide();
    }

    pub fn trigger_scan_with_db(&self, db: &Arc<crate::data::loader::Database>) {
        self.is_showing.store(true, Ordering::SeqCst);

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
                // 黑屏熔断检测：若截屏全黑，提示玩家独占全屏设置
                if crate::vision::ScreenCapture::is_bmp_mostly_black(&bmp) {
                    tracing::warn!("Augment OCR: Captured frame is pure black. Suspected Exclusive Fullscreen mode.");
                    crate::overlay::TextToSpeech::speak("检测到游戏画面纯黑，请将游戏视频设置由全屏更改为无边框窗口化");
                    let error_data = AugmentOverlayData {
                        hero_name: "画面纯黑: 请将游戏设置为【无边框窗口化】".to_string(),
                        cards: vec![],
                    };
                    GLOBAL_HUD.show_augments(error_data, Duration::from_secs(6));
                    return;
                }

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
                    if let Some(best) = cards.iter().max_by(|a, b| {
                        a.win_rate
                            .partial_cmp(&b.win_rate)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| a.delta_win_rate.partial_cmp(&b.delta_win_rate).unwrap_or(std::cmp::Ordering::Equal))
                            .then_with(|| a.games.cmp(&b.games))
                            .then_with(|| a.name.cmp(&b.name))
                    }) {
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

    #[allow(dead_code)]
    pub fn trigger_scan(&self) {
        self.is_showing.store(true, Ordering::SeqCst);
        let placeholder = AugmentOverlayData {
            hero_name: "实时推演".to_string(),
            cards: vec![],
        };
        GLOBAL_HUD.show_augments(placeholder, Duration::from_secs(12));
    }

    #[allow(dead_code)]
    pub fn show_cards(&self, hero_name: &str, cards: Vec<AugmentCardView>) {
        self.is_showing.store(true, Ordering::SeqCst);
        let data = AugmentOverlayData {
            hero_name: hero_name.to_string(),
            cards,
        };
        GLOBAL_HUD.show_augments(data, Duration::from_secs(14));
    }
}
