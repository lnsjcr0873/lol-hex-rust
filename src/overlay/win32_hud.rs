use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AugmentCardView {
    pub name: String,
    pub tier: String, // "S", "A", "B", "TRAP"
    pub win_rate: f64,
    pub delta_win_rate: f64,
    pub wilson_lower: f64,
    pub games: i64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Default)]
pub struct AugmentOverlayData {
    pub hero_name: String,
    pub cards: Vec<AugmentCardView>,
}

#[derive(Debug, Clone)]
pub struct ItemCardView {
    pub name: String,
    pub id: String,
    pub net_cost: i32,
    pub total_cost: i32,
    pub is_affordable: bool,
    pub gold_diff: i32,
    pub tactical_score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct TrioItemView {
    pub name: String,
    pub is_owned: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NextItemView {
    pub name: String,
    pub id: String,
    pub badge: String,
    pub reason: String,
    pub net_cost: i64,
    pub total_cost: i64,
    pub is_affordable: bool,
    pub gold_diff: i64,
    pub progress_pct: f64,
    pub gold_status_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct AnalyticsItemRow {
    pub name: String,
    pub id: String,
    pub win_rate: f64,
    pub delta_metric: f64, // coreDelta or deltaWinRate
    pub pick_rate: f64,
    pub games: i64,
    pub label: String,
    pub is_boot: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ItemOverlayData {
    pub hero_name: String,
    pub mode_title: String,
    pub archetype_title: String,
    pub current_gold: f64,
    pub next_item: Option<NextItemView>,
    pub alternatives: Vec<ItemCardView>,
    pub trio_items: Vec<TrioItemView>,
    pub trio_win_rate: f64,
    pub boots_name: String,
    pub active_algorithm: String,
    pub active_main_tab: String,    // "recommend" vs "analytics"
    pub analytics_subtab: String,   // "featured", "items", "boots", "trios"
    pub analytics_rows: Vec<AnalyticsItemRow>,
}

#[derive(Debug, Clone)]
pub struct BenchChampView {
    pub slot_index: usize,
    pub champion_id: i32,
    pub champion_name: String,
    pub tier: String,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Default)]
pub struct BenchOverlayData {
    pub champions: Vec<BenchChampView>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CounterHeroItem {
    pub champion_id: i32,
    pub name: String,
    pub tier: i32,
    pub counter_win_rate: f64,
    pub overall_win_rate: f64,
    pub games: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuneGuideView {
    pub champion_name: String,
    pub position_cn: String,
    pub skill_order: String,
    pub rune_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankedDecisionCard {
    pub mode: String,
    pub my_locked: bool,
    pub default_tab: String,
    pub rune_guide: Option<RuneGuideView>,
    pub lane_counters: Vec<CounterHeroItem>,
    pub multi_counters: Vec<CounterHeroItem>,
    pub synergy_picks: Vec<CounterHeroItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankedSlotView {
    pub slot_index: usize,
    pub position: String,
    pub position_cn: String,
    pub champion_name: String,
    pub champion_id: i32,
    pub win_rate: f64,
    pub tier: i32,
    pub is_local_player: bool,
    pub is_locked: bool,
    pub is_manual_pos: bool,
    pub matchup_win_rate: Option<f64>,
    pub profile: Option<crate::engine::player_history::PlayerTacticalProfile>,
    pub counter_heroes: Vec<CounterHeroItem>,
    pub synergy_heroes: Vec<CounterHeroItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RankedOverlayData {
    pub my_team: Vec<RankedSlotView>,
    pub their_team: Vec<RankedSlotView>,
    pub decision_card: RankedDecisionCard,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct RankedInteractionState {
    pub active_tab: String,
    pub counter_scroll_offset: usize,
    pub left_wing_collapsed: bool,
    pub right_wing_collapsed: bool,
    pub sample_match_count: i32,
    pub btn_status_text: Option<String>,
    pub btn_status_time: Option<Instant>,
    pub last_local_locked: bool,
    pub inspect_slot: Option<(bool, usize)>,
    pub inspect_tab: String,
    pub sort_ascending: bool,
    pub inspect_scroll_offset: usize,
    pub inspect_closed: bool,
}

impl Default for RankedInteractionState {
    fn default() -> Self {
        Self {
            active_tab: "lane_counter".to_string(),
            counter_scroll_offset: 0,
            left_wing_collapsed: false,
            right_wing_collapsed: false,
            sample_match_count: 20,
            btn_status_text: None,
            btn_status_time: None,
            last_local_locked: false,
            inspect_slot: None,
            inspect_tab: "counter".to_string(),
            sort_ascending: false,
            inspect_scroll_offset: 0,
            inspect_closed: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HudHotzones {
    pub tab_btns: Vec<((i32, i32, i32, i32), String)>,
    pub sample_btns: Vec<((i32, i32, i32, i32), i32)>,
    pub nav_btns: Vec<((i32, i32, i32, i32), i32)>,
    pub rune_btn: Option<(i32, i32, i32, i32)>,
    pub left_wing_toggle: Option<(i32, i32, i32, i32)>,
    pub right_wing_toggle: Option<(i32, i32, i32, i32)>,
    pub pos_btns: Vec<((i32, i32, i32, i32), bool, usize)>,
    pub inspect_slot_btns: Vec<((i32, i32, i32, i32), bool, usize)>,
    pub inspect_tab_btns: Vec<((i32, i32, i32, i32), String)>,
    pub inspect_sort_btn: Option<(i32, i32, i32, i32)>,
    pub inspect_close_btn: Option<(i32, i32, i32, i32)>,
    pub inspect_nav_btns: Vec<((i32, i32, i32, i32), i32)>,
    pub item_close_btn: Option<(i32, i32, i32, i32)>,
    pub item_algo_btns: Vec<((i32, i32, i32, i32), String)>,
    pub item_main_tabs: Vec<((i32, i32, i32, i32), String)>,
    pub item_subtabs: Vec<((i32, i32, i32, i32), String)>,
}

impl HudHotzones {
    pub fn is_point_interactive(&self, px: i32, py: i32) -> bool {
        if let Some((x1, y1, x2, y2)) = self.item_close_btn {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.item_algo_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.item_main_tabs {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.item_subtabs {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        if let Some((x1, y1, x2, y2)) = self.left_wing_toggle {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        if let Some((x1, y1, x2, y2)) = self.right_wing_toggle {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        if let Some((x1, y1, x2, y2)) = self.rune_btn {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        if let Some((x1, y1, x2, y2)) = self.inspect_sort_btn {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        if let Some((x1, y1, x2, y2)) = self.inspect_close_btn {
            if px >= x1 && px <= x2 && py >= y1 && py <= y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.tab_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.sample_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.nav_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _, _) in &self.pos_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _, _) in &self.inspect_slot_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.inspect_tab_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        for ((x1, y1, x2, y2), _) in &self.inspect_nav_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 { return true; }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub enum HudContent {
    None,
    Augment(AugmentOverlayData),
    Item(ItemOverlayData),
    Bench(BenchOverlayData),
    Ranked(RankedOverlayData),
}

#[derive(Debug, Clone)]
pub struct HudState {
    pub content: HudContent,
    pub expires_at: Option<Instant>,
    pub is_visible: bool,
    pub is_test_mode: bool,
    pub dirty: bool,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            content: HudContent::None,
            expires_at: None,
            is_visible: false,
            is_test_mode: false,
            dirty: false,
        }
    }
}

pub struct NativeHud {
    pub running: Arc<AtomicBool>,
    pub state: Arc<std::sync::RwLock<HudState>>,
    #[cfg(windows)]
    pub hwnd: Arc<std::sync::Mutex<Option<isize>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontKind {
    Text,  // "Microsoft YaHei UI"
    Emoji, // "Segoe UI Emoji"
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_HUD: Arc<NativeHud> = Arc::new(NativeHud::new());
    pub static ref RANKED_INTERACTION: std::sync::RwLock<RankedInteractionState> = std::sync::RwLock::new(RankedInteractionState::default());
    pub static ref GLOBAL_HOTZONES: std::sync::RwLock<HudHotzones> = std::sync::RwLock::new(HudHotzones::default());
    pub static ref GLOBAL_LCU: std::sync::RwLock<Option<Arc<crate::lcu::LcuClient>>> = std::sync::RwLock::new(None);
    pub static ref GLOBAL_HISTORY: std::sync::RwLock<Option<Arc<crate::engine::PlayerHistoryManager>>> = std::sync::RwLock::new(None);
    pub static ref ACTIVE_ALGORITHM: std::sync::RwLock<String> = std::sync::RwLock::new("SMART".to_string());
    pub static ref GLOBAL_ITEM_OVERLAY: std::sync::RwLock<Option<Arc<crate::overlay::ItemOverlay>>> = std::sync::RwLock::new(None);
    static ref FONT_CACHE: std::sync::Mutex<std::collections::HashMap<(i32, i32, FontKind), isize>> = std::sync::Mutex::new(std::collections::HashMap::new());
}

#[cfg(windows)]
unsafe fn get_cached_font_kind(size: i32, weight: i32, kind: FontKind) -> windows::Win32::Graphics::Gdi::HFONT {
    use windows::Win32::Graphics::Gdi::{
        CreateFontW, HFONT, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH, FF_DONTCARE,
    };
    if let Ok(mut cache) = FONT_CACHE.lock() {
        if let Some(&handle) = cache.get(&(size, weight, kind)) {
            return HFONT(handle as *mut core::ffi::c_void);
        }
        let font_face = match kind {
            FontKind::Text => "Microsoft YaHei UI\0",
            FontKind::Emoji => "Segoe UI Emoji\0",
        };
        let font_name: Vec<u16> = font_face.encode_utf16().collect();
        let hfont = CreateFontW(
            size,
            0, 0, 0,
            weight,
            0, 0, 0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
            windows::core::PCWSTR(font_name.as_ptr()),
        );
        cache.insert((size, weight, kind), hfont.0 as isize);
        hfont
    } else {
        HFONT::default()
    }
}

#[cfg(windows)]
#[allow(dead_code)]
unsafe fn get_cached_font(size: i32, weight: i32) -> windows::Win32::Graphics::Gdi::HFONT {
    get_cached_font_kind(size, weight, FontKind::Text)
}

/// Frees all cached Win32 HFONT handles using DeleteObject and clears the cache map.
pub fn cleanup_font_cache() {
    #[cfg(windows)]
    {
        use windows::Win32::Graphics::Gdi::{DeleteObject, HGDIOBJ};
        if let Ok(mut cache) = FONT_CACHE.lock() {
            for (_, handle) in cache.drain() {
                if handle != 0 {
                    unsafe {
                        let _ = DeleteObject(HGDIOBJ(handle as *mut core::ffi::c_void));
                    }
                }
            }
        }
    }
}

pub fn apply_current_runes_now() {
    let rune_data = if let Ok(st) = GLOBAL_HUD.state.read() {
        if let HudContent::Ranked(ref data) = st.content {
            data.decision_card.rune_guide.as_ref().map(|rg| rg.rune_data.clone())
        } else {
            None
        }
    } else {
        None
    };

    crate::spawn_task(async move {
        let success = if let Some(payload) = rune_data {
            let lcu = if let Ok(guard) = GLOBAL_LCU.read() {
                guard.clone()
            } else {
                None
            };
            if let Some(lcu) = lcu {
                lcu.apply_perk_page(&payload).await
            } else {
                false
            }
        } else {
            false
        };

        if let Ok(mut inter) = RANKED_INTERACTION.write() {
            if success {
                inter.btn_status_text = Some("✅ 符文已写入客户端!".to_string());
            } else {
                inter.btn_status_text = Some("⚠️ 写入未完成(离线)".to_string());
            }
            inter.btn_status_time = Some(Instant::now());
        }
        GLOBAL_HUD.request_repaint();
    });
}

fn truncate_text_to_half_width(s: &str, max_hw: usize) -> String {
    let mut current_hw = 0;
    let mut result = String::new();
    for c in s.chars() {
        let w = if c.is_ascii() { 1 } else { 2 };
        if current_hw + w > max_hw {
            result.push('…');
            return result;
        }
        result.push(c);
        current_hw += w;
    }
    result
}

fn cycle_slot_position(is_my_team: bool, slot_index: usize) {
    if let Ok(mut st) = GLOBAL_HUD.state.write() {
        if let HudContent::Ranked(ref mut data) = st.content {
            let team = if is_my_team { &mut data.my_team } else { &mut data.their_team };
            if let Some(slot) = team.get_mut(slot_index) {
                let roles = ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"];
                let roles_cn = ["上单", "打野", "中单", "下路", "辅助"];
                let curr_idx = roles.iter().position(|&r| r == slot.position).unwrap_or(0);
                let next_idx = (curr_idx + 1) % roles.len();
                slot.position = roles[next_idx].to_string();
                slot.position_cn = roles_cn[next_idx].to_string();
                slot.is_manual_pos = true;
            }
        }
    }
}

fn handle_lbutton_down(hwnd: windows::Win32::Foundation::HWND, px: i32, py: i32) {
    let hotzones = if let Ok(guard) = GLOBAL_HOTZONES.read() {
        guard.clone()
    } else {
        return;
    };

    let mut repaint_needed = false;

    // 0. Item Overlay close button
    if let Some((x1, y1, x2, y2)) = hotzones.item_close_btn {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            crate::overlay::item_overlay::ITEM_MANUAL_OPEN.store(false, Ordering::SeqCst);
            GLOBAL_HUD.hide();
            return;
        }
    }

    // 0.1 Item Overlay algorithm buttons
    for ((x1, y1, x2, y2), algo_key) in &hotzones.item_algo_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut algo) = ACTIVE_ALGORITHM.write() {
                *algo = algo_key.clone();
            }
            if let Ok(guard) = GLOBAL_ITEM_OVERLAY.read() {
                if let Some(ref item_mgr) = *guard {
                    item_mgr.trigger_recalculate();
                }
            }
            GLOBAL_HUD.request_repaint();
            return;
        }
    }

    // 0.2 Item Overlay main tabs ("recommend" vs "analytics")
    for ((x1, y1, x2, y2), tab_key) in &hotzones.item_main_tabs {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(guard) = GLOBAL_ITEM_OVERLAY.read() {
                if let Some(ref item_mgr) = *guard {
                    item_mgr.switch_main_tab(tab_key);
                }
            }
            GLOBAL_HUD.request_repaint();
            return;
        }
    }

    // 0.3 Item Overlay analytics subtabs ("featured", "items", "boots", "trios")
    for ((x1, y1, x2, y2), sub_key) in &hotzones.item_subtabs {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(guard) = GLOBAL_ITEM_OVERLAY.read() {
                if let Some(ref item_mgr) = *guard {
                    item_mgr.switch_analytics_subtab(sub_key);
                }
            }
            GLOBAL_HUD.request_repaint();
            return;
        }
    }

    // 1. Left wing toggle
    if let Some((x1, y1, x2, y2)) = hotzones.left_wing_toggle {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.left_wing_collapsed = !inter.left_wing_collapsed;
                repaint_needed = true;
            }
        }
    }

    // 2. Right wing toggle
    if let Some((x1, y1, x2, y2)) = hotzones.right_wing_toggle {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.right_wing_collapsed = !inter.right_wing_collapsed;
                repaint_needed = true;
            }
        }
    }

    // 3. Tab buttons
    for ((x1, y1, x2, y2), tab_key) in &hotzones.tab_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                if inter.active_tab != *tab_key {
                    inter.active_tab = tab_key.clone();
                    inter.counter_scroll_offset = 0;
                    repaint_needed = true;
                }
            }
            break;
        }
    }

    // 4. Sample buttons
    for ((x1, y1, x2, y2), count) in &hotzones.sample_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            let new_sample = *count;
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.sample_match_count = new_sample;
                let history_opt = GLOBAL_HISTORY.try_read().ok().and_then(|g| g.clone());
                if let Some(ref history_mgr) = history_opt {
                    history_mgr.sample_count.store(new_sample, Ordering::Relaxed);
                }

                // Immediately recompute tactical profiles for all players in current HUD state
                if let Ok(mut st) = GLOBAL_HUD.state.write() {
                    if let HudContent::Ranked(ref mut data) = st.content {
                        if let Some(ref history_mgr) = history_opt {
                            for slot in &mut data.my_team {
                                if let Some(ref p) = slot.profile {
                                    let puuid = p.puuid.clone();
                                    if let Some(new_prof) = history_mgr.recompute_profile_sync(&puuid, slot.champion_id, new_sample as usize) {
                                        slot.profile = Some(new_prof);
                                    }
                                }
                            }
                            for slot in &mut data.their_team {
                                if let Some(ref p) = slot.profile {
                                    let puuid = p.puuid.clone();
                                    if let Some(new_prof) = history_mgr.recompute_profile_sync(&puuid, slot.champion_id, new_sample as usize) {
                                        slot.profile = Some(new_prof);
                                    }
                                }
                            }
                        }
                    }
                    st.dirty = true;
                }
                repaint_needed = true;
            }
            break;
        }
    }

    // 5. Nav buttons
    for ((x1, y1, x2, y2), delta) in &hotzones.nav_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                let curr = inter.counter_scroll_offset as i32;
                let next = (curr + delta).max(0) as usize;
                inter.counter_scroll_offset = next;
                repaint_needed = true;
            }
            break;
        }
    }

    // 6. Rune button
    if let Some((x1, y1, x2, y2)) = hotzones.rune_btn {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            apply_current_runes_now();
            repaint_needed = true;
        }
    }

    // 7. Hero Inspection close button
    if let Some((x1, y1, x2, y2)) = hotzones.inspect_close_btn {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.inspect_closed = true;
                inter.inspect_slot = None;
                repaint_needed = true;
            }
        }
    }

    // 8. Hero Inspection sort button (toggle asc/desc)
    if let Some((x1, y1, x2, y2)) = hotzones.inspect_sort_btn {
        if px >= x1 && px <= x2 && py >= y1 && py <= y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.sort_ascending = !inter.sort_ascending;
                repaint_needed = true;
            }
        }
    }

    // 9. Hero Inspection tab buttons (counter vs synergy)
    for ((x1, y1, x2, y2), tab_key) in &hotzones.inspect_tab_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                if inter.inspect_tab != *tab_key {
                    inter.inspect_tab = tab_key.clone();
                    inter.inspect_scroll_offset = 0;
                    repaint_needed = true;
                }
            }
            break;
        }
    }

    // 10. Pos buttons (left click cycles position pill - checked before slot card)
    let mut hit_pos_btn = false;
    for ((x1, y1, x2, y2), is_my_team, slot_index) in &hotzones.pos_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            cycle_slot_position(*is_my_team, *slot_index);
            repaint_needed = true;
            hit_pos_btn = true;
            break;
        }
    }

    // 11. Hero Inspection slot buttons (click slot card to inspect that hero)
    if !hit_pos_btn {
        for ((x1, y1, x2, y2), is_my_team, slot_index) in &hotzones.inspect_slot_btns {
            if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
                if let Ok(mut inter) = RANKED_INTERACTION.write() {
                    if inter.inspect_slot == Some((*is_my_team, *slot_index)) {
                        inter.inspect_slot = None;
                    } else {
                        inter.inspect_closed = false;
                        inter.inspect_slot = Some((*is_my_team, *slot_index));
                        inter.inspect_scroll_offset = 0;
                    }
                    repaint_needed = true;
                }
                break;
            }
        }
    }

    // 12. Hero Inspection nav buttons
    for ((x1, y1, x2, y2), delta) in &hotzones.inspect_nav_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                let curr = inter.inspect_scroll_offset as i32;
                let next = (curr + delta).max(0) as usize;
                inter.inspect_scroll_offset = next;
                repaint_needed = true;
            }
            break;
        }
    }

    if repaint_needed {
        unsafe {
            use windows::Win32::Foundation::BOOL;
            use windows::Win32::Graphics::Gdi::InvalidateRect;
            let _ = InvalidateRect(hwnd, None, BOOL(0));
        }
    }
}

impl Default for NativeHud {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeHud {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(std::sync::RwLock::new(HudState::default())),
            #[cfg(windows)]
            hwnd: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return; // Already running
        }

        #[cfg(windows)]
        {
            let running_flag = self.running.clone();
            let hwnd_holder = self.hwnd.clone();

            std::thread::spawn(move || {
                run_win32_hud_loop(running_flag, hwnd_holder);
            });
        }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        #[cfg(windows)]
        {
            if let Ok(guard) = self.hwnd.lock() {
                if let Some(val) = *guard {
                    unsafe {
                        use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
                        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
                        let hwnd = HWND(val as *mut core::ffi::c_void);
                        let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
            }
            cleanup_font_cache();
        }
    }
}

impl Drop for NativeHud {
    fn drop(&mut self) {
        self.stop();
        cleanup_font_cache();
    }
}

impl NativeHud {

    pub fn show_augments(&self, data: AugmentOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::Augment(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.dirty = true;
        }
        self.request_repaint();
    }

    pub fn show_items(&self, data: ItemOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::Item(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.dirty = true;
        }
        self.request_repaint();
    }

    pub fn show_bench(&self, data: BenchOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::Bench(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.dirty = true;
        }
        self.request_repaint();
    }

    pub fn show_ranked(&self, data: RankedOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::Ranked(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.is_test_mode = false;
            st.dirty = true;
        }
        self.request_repaint();
    }

    #[allow(dead_code)]
    pub fn show_ranked_test(&self, data: RankedOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::Ranked(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.is_test_mode = true;
            st.dirty = true;
        }
        self.request_repaint();
    }

    pub fn hide(&self) {
        {
            let mut st = self.state.write().unwrap_or_else(|e| e.into_inner());
            st.content = HudContent::None;
            st.expires_at = None;
            st.is_visible = false;
            st.is_test_mode = false;
            st.dirty = true;
        }
        if let Ok(mut inter) = RANKED_INTERACTION.write() {
            inter.last_local_locked = false;
        }
        #[cfg(windows)]
        {
            if let Ok(guard) = self.hwnd.lock() {
                if let Some(val) = *guard {
                    unsafe {
                        use windows::Win32::Foundation::HWND;
                        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                        let hwnd = HWND(val as *mut core::ffi::c_void);
                        let _ = ShowWindow(hwnd, SW_HIDE);
                    }
                }
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state.read().map(|st| st.is_visible).unwrap_or(false)
    }

    pub fn request_repaint(&self) {
        #[cfg(windows)]
        {
            if let Ok(guard) = self.hwnd.lock() {
                if let Some(val) = *guard {
                    unsafe {
                        use windows::Win32::Foundation::{BOOL, HWND};
                        use windows::Win32::Graphics::Gdi::InvalidateRect;
                        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};
                        let hwnd = HWND(val as *mut core::ffi::c_void);
                        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                        let _ = InvalidateRect(hwnd, None, BOOL(0));
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------
// Win32 Layered Window Lifecycle and Message Loop (方案 A)
// -------------------------------------------------------------

#[cfg(windows)]
unsafe fn attach_to_interactive_desktop() {
    #[link(name = "user32")]
    extern "system" {
        fn OpenDesktopW(lpszdesktop: *const u16, dwflags: u32, finherit: i32, dwdesiredaccess: u32) -> *mut core::ffi::c_void;
        fn OpenInputDesktop(dwflags: u32, finherit: i32, dwdesiredaccess: u32) -> *mut core::ffi::c_void;
        fn SetThreadDesktop(hdesktop: *mut core::ffi::c_void) -> i32;
    }

    let default_name: Vec<u16> = "Default\0".encode_utf16().collect();
    let mut hdesk = OpenDesktopW(default_name.as_ptr(), 0, 0, 0x01FF);
    if hdesk.is_null() {
        hdesk = OpenInputDesktop(0, 0, 0x01FF);
    }
    if !hdesk.is_null() {
        let _ = SetThreadDesktop(hdesk);
    }
}

#[cfg(windows)]
fn run_win32_hud_loop(
    running: Arc<AtomicBool>,
    hwnd_holder: Arc<std::sync::Mutex<Option<isize>>>,
) {
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, RegisterClassExW,
        SetLayeredWindowAttributes, SetTimer, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
        LWA_ALPHA, LWA_COLORKEY, MSG, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lpmodulename: *const u16) -> *mut core::ffi::c_void;
    }

    unsafe {
        attach_to_interactive_desktop();

        let hmod = GetModuleHandleW(std::ptr::null());
        let hinstance = HINSTANCE(hmod);

        let class_name = w!("HexBrainOverlayClass");
        let mut wndclass = WNDCLASSEXW::default();
        wndclass.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wndclass.style = CS_HREDRAW | CS_VREDRAW;
        wndclass.lpfnWndProc = Some(overlay_wndproc);
        wndclass.hInstance = hinstance;
        wndclass.lpszClassName = class_name;

        let _ = RegisterClassExW(&wndclass);

        let vp = crate::vision::ScreenCapture::get_effective_viewport();

        let ex_style = WS_EX_LAYERED
            | WS_EX_TOOLWINDOW
            | WS_EX_TOPMOST
            | WS_EX_NOACTIVATE;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("HexBrain Native HUD"),
            WS_POPUP,
            vp.x,
            vp.y,
            vp.width,
            vp.height,
            HWND::default(),
            windows::Win32::UI::WindowsAndMessaging::HMENU::default(),
            hinstance,
            None,
        ).unwrap_or_default();

        if hwnd == HWND::default() {
            tracing::error!("Failed to create Win32 HUD Overlay Window");
            return;
        }

        // Chroma key: RGB(1, 1, 1) is 100% transparent. Alpha 240 for drawn UI elements.
        let transparent_color = COLORREF(0x00010101);
        let _ = SetLayeredWindowAttributes(hwnd, transparent_color, 240, LWA_COLORKEY | LWA_ALPHA);

        // Store HWND as isize (Send + Sync)
        if let Ok(mut guard) = hwnd_holder.lock() {
            *guard = Some(hwnd.0 as isize);
        }

        // 100ms timer for tracking active game window & auto-hiding (balanced 10Hz)
        let _ = SetTimer(hwnd, 1, 100, None);

        let mut msg = MSG::default();
        while running.load(Ordering::SeqCst) && GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if let Ok(mut guard) = hwnd_holder.lock() {
            *guard = None;
        }
        let _ = DestroyWindow(hwnd);
        cleanup_font_cache();
    }
}

#[cfg(windows)]
unsafe extern "system" fn overlay_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{COLORREF, LRESULT, RECT};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush,
        DeleteDC, DeleteObject, EndPaint, FillRect, SelectObject, PAINTSTRUCT, SRCCOPY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, GetClientRect, KillTimer, PostQuitMessage, WM_CLOSE, WM_DESTROY,
        WM_ERASEBKGND, WM_PAINT, WM_TIMER, WM_LBUTTONDOWN, WM_RBUTTONDOWN,
    };

    match msg {
        windows::Win32::UI::WindowsAndMessaging::WM_NCHITTEST => {
            use windows::Win32::Foundation::POINT;
            use windows::Win32::Graphics::Gdi::ScreenToClient;
            let screen_x = (lparam.0 & 0xffff) as i16 as i32;
            let screen_y = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
            let mut pt = POINT { x: screen_x, y: screen_y };
            let _ = ScreenToClient(hwnd, &mut pt);

            let is_interactive = if let Ok(hz) = GLOBAL_HOTZONES.read() {
                hz.is_point_interactive(pt.x, pt.y)
            } else {
                false
            };

            if is_interactive {
                LRESULT(1) // HTCLIENT: dispatch mouse click to HUD button
            } else {
                LRESULT(-1) // HTTRANSPARENT: 100% pass-through directly to League of Legends.exe!
            }
        }
        WM_LBUTTONDOWN => {
            let px = (lparam.0 & 0xffff) as i16 as i32;
            let py = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
            handle_lbutton_down(hwnd, px, py);
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            // Right-clicks are never consumed by HUD: zero interference with character movement/spells
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == 1 {
                handle_timer_tick(hwnd);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut client_rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut client_rect);
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;

            if width > 0 && height > 0 {
                // Double buffering: Create compatible in-memory DC & bitmap
                let mem_dc = CreateCompatibleDC(hdc);
                if !mem_dc.is_invalid() {
                    let mem_bmp = CreateCompatibleBitmap(hdc, width, height);
                    if !mem_bmp.is_invalid() {
                        let old_bmp = SelectObject(mem_dc, mem_bmp);

                        // Clear memory buffer with Chroma Key (RGB 1, 1, 1)
                        let bg_brush = CreateSolidBrush(COLORREF(0x00010101));
                        FillRect(mem_dc, &client_rect, bg_brush);
                        let _ = DeleteObject(bg_brush);

                        // Read snapshot and render HUD content
                        if let Ok(st) = GLOBAL_HUD.state.read() {
                            if st.is_visible {
                                render_hud(mem_dc, width, height, &st.content);
                            }
                        }

                        // BitBlt memory buffer to screen
                        let _ = BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

                        // Clean up GDI memory objects
                        let _ = SelectObject(mem_dc, old_bmp);
                        let _ = DeleteObject(mem_bmp);
                    }
                    let _ = DeleteDC(mem_dc);
                }
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1), // Prevent flickering
        WM_CLOSE => {
            let _ = KillTimer(hwnd, 1);
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            cleanup_font_cache();
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
unsafe fn handle_timer_tick(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE,
        SW_SHOWNOACTIVATE,
    };
    use std::sync::Mutex;
    static LAST_WIN_GEOM: Mutex<(i32, i32, i32, i32)> = Mutex::new((-1, -1, -1, -1));
    static LAST_CLIENT_GEOM: Mutex<(i32, i32, i32, i32)> = Mutex::new((-1, -1, -1, -1));

    let reset_geom_caches = || {
        if let Ok(mut g) = LAST_WIN_GEOM.lock() {
            *g = (-1, -1, -1, -1);
        }
        if let Ok(mut g) = LAST_CLIENT_GEOM.lock() {
            *g = (-1, -1, -1, -1);
        }
    };

    let (content_type, is_test, is_expired) = if let Ok(mut st) = GLOBAL_HUD.state.write() {
        let mut expired = false;
        if let Some(exp) = st.expires_at {
            if Instant::now() >= exp {
                st.is_visible = false;
                st.content = HudContent::None;
                st.expires_at = None;
                st.is_test_mode = false;
                expired = true;
            }
        }
        (st.content.clone(), st.is_test_mode, expired)
    } else {
        (HudContent::None, false, true)
    };

    if is_expired {
        reset_geom_caches();
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }

    let is_draft_bench = matches!(&content_type, HudContent::Bench(_));
    let is_draft_ranked = matches!(&content_type, HudContent::Ranked(_));

    if is_draft_bench {
        let game_running = crate::vision::ScreenCapture::get_lol_game_rect().is_some();
        if game_running {
            reset_geom_caches();
            let _ = ShowWindow(hwnd, SW_HIDE);
            return;
        }
    }

    if is_draft_ranked && !is_test {
        let game_running = crate::vision::ScreenCapture::get_lol_game_rect().is_some();
        if game_running {
            reset_geom_caches();
            let _ = ShowWindow(hwnd, SW_HIDE);
            return;
        }
    }

    let mut target_rect = match &content_type {
        HudContent::Ranked(_) | HudContent::Bench(_) => {
            if crate::vision::ScreenCapture::get_lol_game_rect().is_some() && !is_test {
                None
            } else {
                crate::vision::ScreenCapture::get_lol_client_rect()
            }
        }
        HudContent::Augment(_) | HudContent::Item(_) => {
            crate::vision::ScreenCapture::get_lol_game_rect()
                .or_else(crate::vision::ScreenCapture::get_lol_client_rect)
        }
        HudContent::None => None,
    };

    if target_rect.is_none() {
        let vp = crate::vision::ScreenCapture::get_effective_viewport();
        if is_test {
            // Test mode: Center 1280x720 window on desktop
            let cw = 1280.min(vp.width);
            let ch = 720.min(vp.height);
            target_rect = Some(crate::vision::ViewportRect {
                x: vp.x + (vp.width - cw) / 2,
                y: vp.y + (vp.height - ch) / 2,
                width: cw,
                height: ch,
            });
        } else if matches!(&content_type, HudContent::Augment(_) | HudContent::Item(_)) {
            // In-game overlays can use effective viewport fallback
            target_rect = Some(vp);
        }
    }

    let should_hide = match &content_type {
        HudContent::None => true,
        HudContent::Augment(_) | HudContent::Item(_) => {
            target_rect.is_none() || (!is_test && !crate::overlay::WindowTracker::is_game_active())
        }
        HudContent::Ranked(_) | HudContent::Bench(_) => {
            target_rect.is_none() || (!is_test && !crate::overlay::WindowTracker::is_league_active())
        }
    };

    if should_hide {
        reset_geom_caches();
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }

    // Align with active League window position (with outer docking for Ranked)
    let mut any_geom_changed = false;
    if let Some(rect) = target_rect {
        if matches!(&content_type, HudContent::Ranked(_)) {
            let (left_collapsed, right_collapsed) = if let Ok(inter) = RANKED_INTERACTION.read() {
                (inter.left_wing_collapsed, inter.right_wing_collapsed)
            } else {
                (false, false)
            };

            let sx = rect.width as f64 / 1280.0;
            let sy = rect.height as f64 / 720.0;
            let top_dock_height = (44.0 * sy) as i32;
            let wing_w = (168.0 * sx) as i32;
            let mini_w = (24.0 * sx) as i32;

            let left_w = if left_collapsed { mini_w } else { wing_w };
            let right_w = if right_collapsed { mini_w } else { wing_w };

            use windows::Win32::Graphics::Gdi::{
                GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            };
            let client_r = windows::Win32::Foundation::RECT {
                left: rect.x,
                top: rect.y,
                right: rect.x + rect.width,
                bottom: rect.y + rect.height,
            };
            let hmon = MonitorFromRect(&client_r, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let (mon_left, mon_top, mon_right) = if GetMonitorInfoW(hmon, &mut mi).as_bool() {
                (mi.rcWork.left, mi.rcWork.top, mi.rcWork.right)
            } else {
                let sw = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                    windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
                );
                (0, 0, sw)
            };

            let left_avail = rect.x - mon_left;
            let top_avail = rect.y - mon_top;
            let right_avail = mon_right - (rect.x + rect.width);

            let actual_offset_y = if top_avail >= top_dock_height { top_dock_height } else { top_avail.max(0) };
            let actual_left_margin = if left_avail >= left_w { left_w } else { left_avail.max(0) };
            let actual_right_margin = if right_avail >= right_w { right_w } else { right_avail.max(0) };

            let win_x = rect.x - actual_left_margin;
            let win_y = rect.y - actual_offset_y;
            let win_w = rect.width + actual_left_margin + actual_right_margin;
            let win_h = rect.height + actual_offset_y;

            let target_geom = (win_x, win_y, win_w, win_h);
            let mut geom_changed = false;
            if let Ok(mut g) = LAST_WIN_GEOM.lock() {
                if *g != target_geom {
                    *g = target_geom;
                    geom_changed = true;
                    any_geom_changed = true;
                }
            }
            if geom_changed {
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    win_x,
                    win_y,
                    win_w,
                    win_h,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        } else {
            let target_geom = (rect.x, rect.y, rect.width, rect.height);
            let mut geom_changed = false;
            if let Ok(mut g) = LAST_CLIENT_GEOM.lock() {
                if *g != target_geom {
                    *g = target_geom;
                    geom_changed = true;
                    any_geom_changed = true;
                }
            }
            if geom_changed {
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        }
    }

    // Dynamic Click-Through: In-Game (Augment/Item) overlays must be 100% click-through (WS_EX_TRANSPARENT)
    // to prevent blocking character movement/skills during combat. Only Ranked draft HUD is clickable.
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
        };
        let is_interactive = matches!(&content_type, HudContent::Ranked(_) | HudContent::Item(_));
        let current_ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let desired_ex = if is_interactive {
            current_ex & !(WS_EX_TRANSPARENT.0)
        } else {
            current_ex | WS_EX_TRANSPARENT.0
        };
        if desired_ex != current_ex {
            let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, desired_ex as isize);
            use windows::Win32::UI::WindowsAndMessaging::{
                SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            };
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
            );
        }
    }

    // Only repaint when dirty flag is set or window geometry actually changed.
    // This reduces stationary DWM compose overhead to 0 FPS / 0 GPU spikes, entirely fixing stuttering.
    let is_dirty = if let Ok(mut st) = GLOBAL_HUD.state.write() {
        let dirty = st.dirty;
        st.dirty = false;
        dirty
    } else {
        false
    };

    if is_dirty || any_geom_changed {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = InvalidateRect(hwnd, None, BOOL(0));
    }
}

// -------------------------------------------------------------
// GDI Hextech Rendering Engine
// -------------------------------------------------------------

#[cfg(windows)]
unsafe fn render_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    content: &HudContent,
) {
    use windows::Win32::Graphics::Gdi::{GetStockObject, SelectObject, SetBkMode, DEFAULT_GUI_FONT, TRANSPARENT};
    let _ = SetBkMode(hdc, TRANSPARENT);
    let stock_font = GetStockObject(DEFAULT_GUI_FONT);
    SelectObject(hdc, stock_font);

    match content {
        HudContent::Augment(data) => render_augment_hud(hdc, width, height, data),
        HudContent::Item(data) => render_item_hud(hdc, width, height, data),
        HudContent::Bench(data) => render_bench_hud(hdc, width, height, data),
        HudContent::Ranked(data) => render_ranked_hud(hdc, width, height, data),
        HudContent::None => {}
    }
}

#[cfg(windows)]
unsafe fn render_augment_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    data: &AugmentOverlayData,
) {
    let sx = width as f64 / 1920.0;
    let sy = height as f64 / 1080.0;
    let min_s = sx.min(sy);

    let total_cards = data.cards.len();

    // 1. 查找最优卡牌与全陷阱判定
    let mut best_slot_idx = 1;
    let mut best_name = "首选强化";
    let mut all_traps = false;

    if total_cards > 0 {
        if let Some((idx, best)) = data.cards.iter().enumerate().max_by(|(_, a), (_, b)| {
            a.win_rate.partial_cmp(&b.win_rate).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            best_slot_idx = idx + 1;
            best_name = &best.name;
        }
        all_traps = data.cards.iter().all(|c| c.tier == "TRAP" || c.delta_win_rate <= -0.005);
    }

    // 2. 屏幕正顶部主决策横幅 HUD (Header Overlay)
    // 尺寸: 760x112, 坐标: pos_x 居中, pos_y = 35 * sy (屏幕极顶端 35px，远离卡片 150px 以上，0 遮挡卡牌立绘与正文)
    let banner_w = (760.0 * sx) as i32;
    let banner_h = (112.0 * sy) as i32;
    let banner_x = (width - banner_w) / 2;
    let banner_y = (35.0 * sy) as i32;

    let banner_bg = rgb(14, 22, 38); // 深邃科技蓝黑 (#0e1626)
    let banner_border = if all_traps {
        rgb(244, 63, 94) // 警报红
    } else {
        rgb(10, 200, 185) // 海克斯青金 (#0ac8b9)
    };

    draw_card_rect(hdc, banner_x, banner_y, banner_w, banner_h, banner_bg, banner_border);

    // 顶部横幅字体大小
    let title_font_size = (16.0 * min_s).max(12.0) as i32;
    let body_font_size = (15.0 * min_s).max(11.0) as i32;
    let sub_font_size = (13.0 * min_s).max(10.0) as i32;

    if total_cards > 0 {
        let (title_text, title_col) = if all_traps {
            (
                "🚨 【全陷阱高危报警】建议消耗海克斯骰子重掷！ (自动推演)".to_string(),
                rgb(244, 63, 94),
            )
        } else {
            (
                format!("👑 智脑首选推荐：【第 {} 张卡片】-> {} (局内推演)", best_slot_idx, best_name),
                rgb(10, 200, 185),
            )
        };
        draw_text_custom(hdc, &title_text, banner_x + 16, banner_y + 10, title_font_size, 700, title_col, false);

        // 顶部横幅行 2: 正文推演理由 (Wilson 95% 置信下限)
        let body_text = if let Some(best) = data.cards.get(best_slot_idx - 1) {
            format!(
                "推演依据: 胜率 {:.1}% (提升 {:+.1}%)，Wilson 置信下限 {:.1}% ({}局样本)，契合当前英雄阵容",
                best.win_rate * 100.0,
                best.delta_win_rate * 100.0,
                best.wilson_lower * 100.0,
                best.games
            )
        } else {
            "推演完成：契合当前英雄战术羁绊".to_string()
        };
        draw_text_custom(hdc, &body_text, banner_x + 16, banner_y + 40, body_font_size, 600, rgb(255, 255, 255), false);

        // 顶部横幅行 3: 三张卡并排对比速览
        let mut summaries = Vec::new();
        for (i, c) in data.cards.iter().take(3).enumerate() {
            let slot_num = i + 1;
            let tag = if slot_num == best_slot_idx {
                "👑首选"
            } else if c.tier == "TRAP" {
                "🚫避雷"
            } else {
                "备选"
            };
            summaries.push(format!("[{}号卡] {} ({} {:+.1}%)", slot_num, c.name, tag, c.delta_win_rate * 100.0));
        }
        let summary_line = summaries.join("   |   ");
        draw_text_custom(hdc, &summary_line, banner_x + 16, banner_y + 76, sub_font_size, 400, rgb(200, 170, 110), false);
    } else {
        let title_text = format!("⚡ HexBrain 海克斯实时推演 - {}", if data.hero_name.is_empty() { "局内待命" } else { &data.hero_name });
        draw_text_custom(hdc, &title_text, banner_x + 16, banner_y + 12, title_font_size, 700, rgb(10, 200, 185), false);
        draw_text_custom(hdc, "正在扫描识别当前画面三选一卡牌...", banner_x + 16, banner_y + 44, body_font_size, 600, rgb(255, 255, 255), false);
        draw_text_custom(hdc, "三选一界面出现后将全自动推演播报，也可按 [ 或 【 键手动重扫", banner_x + 16, banner_y + 76, sub_font_size, 400, rgb(148, 163, 184), false);
    }

    if total_cards == 0 {
        return;
    }

    // 3. 卡牌内下方空白处透明大字胜率投影 (Card Badges) + 最优卡流光外框 (Best Card Border)
    // 1920x1080 基准绝对坐标:
    // 卡槽中心 cx: 599, 967, 1335
    // 胶囊 cy: 618 (卡牌立绘与长正文介绍下方空白处，绝不遮挡原生技能介绍正文，不遮挡重随骰子)
    let slot_centers_x = [
        (599.0 * sx) as i32,
        (967.0 * sx) as i32,
        (1335.0 * sx) as i32,
    ];
    let badge_cy = ((145.0 + 473.0) * sy) as i32; // 618px 基准
    let badge_w = (216.0 * sx) as i32;
    let badge_h = (76.0 * sy) as i32;

    let wr_font_size = (24.0 * min_s).max(15.0) as i32;
    let badge_sub_font_size = (11.0 * min_s).max(8.0) as i32;

    // 3.1 绘制最优卡牌外圈推荐青金光圈与顶部徽章 (Card Border)
    if best_slot_idx >= 1 && best_slot_idx <= 3 {
        let best_cx = slot_centers_x[best_slot_idx - 1];
        let card_w = (290.0 * sx) as i32;
        let card_h = (530.0 * sy) as i32;
        let card_bx = best_cx - card_w / 2;
        let card_by = ((145.0 + 37.0) * sy) as i32; // 182px 基准

        // 顶部居中勋章
        let top_badge_w = (184.0 * sx) as i32;
        let top_badge_h = (26.0 * sy) as i32;
        let top_bx = best_cx - top_badge_w / 2;
        let top_by = card_by - top_badge_h - 4;
        draw_card_rect(hdc, top_bx, top_by, top_badge_w, top_badge_h, rgb(7, 21, 36), rgb(245, 158, 11));
        draw_text_centered(hdc, "★ 智脑首选推荐 ★", best_cx, top_by + (4.0 * sy) as i32, (10.5 * min_s).max(8.0) as i32, 700, rgb(251, 191, 36), true);

        // 卡牌外围高亮推荐边框 (圆角矩形，厚度 2px)
        let border_pen_col = if all_traps { rgb(244, 63, 94) } else { rgb(10, 200, 185) };
        use windows::Win32::Graphics::Gdi::{CreatePen, DeleteObject, SelectObject, RoundRect, PS_SOLID, NULL_BRUSH, GetStockObject};
        let pen = CreatePen(PS_SOLID, 2, border_pen_col);
        let old_pen = SelectObject(hdc, pen);
        let null_brush = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(hdc, null_brush);
        let _ = RoundRect(hdc, card_bx, card_by, card_bx + card_w, card_by + card_h, 12, 12);
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    // 3.2 绘制 3 个槽位的透明大字胶囊 (Card Badges)
    for (i, card) in data.cards.iter().take(3).enumerate() {
        let cx = slot_centers_x[i];
        let bx1 = cx - badge_w / 2;
        let by1 = badge_cy - badge_h / 2;

        let is_best = i + 1 == best_slot_idx;
        let is_trap = card.tier == "TRAP" || card.delta_win_rate <= -0.005;

        let (fill_col, outline_col, wr_col, sub_col, sub_text) = if is_best {
            (
                rgb(7, 21, 36),     // 暗黑青金底
                rgb(245, 158, 11),  // 亮金外边框
                rgb(251, 191, 36),  // 金黄大字胜率
                rgb(10, 200, 185),  // 青色标签
                format!("👑 首选推荐 ({:+.1}%)", card.delta_win_rate * 100.0),
            )
        } else if is_trap {
            (
                rgb(31, 11, 15),    // 暗黑深红底
                rgb(89, 26, 34),    // 猩红边框
                rgb(248, 113, 113), // 警戒红胜率
                rgb(251, 113, 133), // 粉红标签
                format!("🚫 避雷建议 ({:+.1}%)", card.delta_win_rate * 100.0),
            )
        } else {
            let tier_label = match card.tier.as_str() {
                "S" => "S 核心",
                "A" => "A 强力",
                "B" => "B 优选",
                _ => "常规",
            };
            (
                rgb(6, 16, 26),     // 黑曜石底
                rgb(34, 51, 68),    // 科技暗蓝边框
                rgb(241, 245, 249), // 纯白天体胜率
                rgb(148, 163, 184), // 银灰标签
                format!("{} ({:+.1}%)", tier_label, card.delta_win_rate * 100.0),
            )
        };

        // 绘制胶囊外框背景
        draw_card_rect(hdc, bx1, by1, badge_w, badge_h, fill_col, outline_col);

        // 居中大字胜率 (24pt YaHei bold，带黑色阴影提升高反差可读性)
        let wr_str = format!("{:.1}%", card.win_rate * 100.0);
        let wr_y = badge_cy - (20.0 * sy) as i32;
        draw_text_centered(hdc, &wr_str, cx, wr_y, wr_font_size, 700, wr_col, true);

        // 居中副标签 (11pt YaHei bold)
        let sub_y = badge_cy + (10.0 * sy) as i32;
        draw_text_centered(hdc, &sub_text, cx, sub_y, badge_sub_font_size, 700, sub_col, true);
    }
}

#[cfg(windows)]
unsafe fn render_item_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    data: &ItemOverlayData,
) {
    let sx = width as f64 / 1920.0;
    let sy = height as f64 / 1080.0;
    let min_s = sx.min(sy);

    let is_analytics = data.active_main_tab == "analytics";
    let box_w = ((440.0 * sx) as i32).clamp(380, width - 40);
    let box_h = if is_analytics {
        ((356.0 * sy) as i32).clamp(320, height - 60)
    } else {
        ((252.0 * sy) as i32).clamp(240, height - 60)
    };
    let box_x = (18.0 * sx) as i32;
    // 避开游戏原生英雄属性面板 (左下角 0~260px) 与技能加点热区 (+ 号按钮，底部向上约 260px)
    let bottom_clearance = (270.0 * sy) as i32;
    let box_y = (height - box_h - bottom_clearance).max(10);

    // 1. Dark Hextech container: #0a101d, Hextech Gold border: #785a28
    draw_card_rect(hdc, box_x, box_y, box_w, box_h, rgb(10, 16, 29), rgb(120, 90, 40));

    // Hotzones tracking
    let mut new_item_algo_btns = Vec::new();
    let mut new_item_main_tabs = Vec::new();
    let mut new_item_subtabs = Vec::new();

    // 2. Header Bar:
    // Left: 🛡️ 装备推演
    draw_text_custom(hdc, "🛡️ 装备推演", box_x + 10, box_y + 8, (12.0 * min_s).max(10.0) as i32, 700, rgb(200, 170, 110), false);

    // Two Main Tabs: [💡 推荐出装] [📊 全维表现]
    let tab1_x = box_x + 88;
    let tab1_w = 72;
    let tab1_h = 20;
    let (t1_bg, t1_border, t1_fg) = if !is_analytics {
        (rgb(30, 41, 59), rgb(200, 170, 110), rgb(200, 170, 110))
    } else {
        (rgb(10, 16, 29), rgb(30, 41, 59), rgb(100, 116, 139))
    };
    draw_pill_badge(hdc, tab1_x, box_y + 6, tab1_w, tab1_h, t1_bg, t1_border, t1_fg, "💡 推荐出装");
    new_item_main_tabs.push(((tab1_x, box_y + 6, tab1_x + tab1_w, box_y + 6 + tab1_h), "recommend".to_string()));

    let tab2_x = tab1_x + tab1_w + 4;
    let tab2_w = 72;
    let tab2_h = 20;
    let (t2_bg, t2_border, t2_fg) = if is_analytics {
        (rgb(30, 41, 59), rgb(200, 170, 110), rgb(200, 170, 110))
    } else {
        (rgb(10, 16, 29), rgb(30, 41, 59), rgb(100, 116, 139))
    };
    draw_pill_badge(hdc, tab2_x, box_y + 6, tab2_w, tab2_h, t2_bg, t2_border, t2_fg, "📊 全维表现");
    new_item_main_tabs.push(((tab2_x, box_y + 6, tab2_x + tab2_w, box_y + 6 + tab2_h), "analytics".to_string()));

    // Archetype badge
    let arch_text = if !data.archetype_title.is_empty() {
        data.archetype_title.clone()
    } else {
        "🎯【待锁定】".to_string()
    };
    let arch_w = ((arch_text.chars().count() as i32 * 11) + 12).clamp(60, 120);
    draw_pill_badge(hdc, box_x + box_w - arch_w - 32, box_y + 6, arch_w, 20, rgb(17, 28, 46), rgb(2, 132, 199), rgb(56, 189, 248), &arch_text);

    // Close Button [✕]
    let close_x = box_x + box_w - 24;
    let close_y = box_y + 6;
    draw_text_custom(hdc, "✕", close_x + 4, close_y + 2, (12.0 * min_s).max(10.0) as i32, 700, rgb(148, 163, 184), false);
    let new_item_close = Some((close_x, close_y, close_x + 20, close_y + 20));

    if is_analytics {
        // Analytics Subtabs Bar: [⭐ 精选] [⚔️ 成装] [🥾 鞋子] [🏆 三神装]
        let sub_y = box_y + 32;
        let subtabs = [
            ("featured", "⭐ 精选装备"),
            ("items", "⚔️ 成装表现"),
            ("boots", "🥾 鞋子表现"),
            ("trios", "🏆 终局三神装"),
        ];
        let mut sub_x = box_x + 10;
        let cur_sub = if !data.analytics_subtab.is_empty() { data.analytics_subtab.as_str() } else { "featured" };
        for (skey, slabel) in &subtabs {
            let is_cur = cur_sub == *skey;
            let (bg, border, fg) = if is_cur {
                (rgb(17, 28, 46), rgb(2, 132, 199), rgb(56, 189, 248))
            } else {
                (rgb(10, 16, 29), rgb(30, 41, 59), rgb(100, 116, 139))
            };
            let sw = 94;
            let sh = 20;
            draw_pill_badge(hdc, sub_x, sub_y, sw, sh, bg, border, fg, slabel);
            new_item_subtabs.push(((sub_x, sub_y, sub_x + sw, sub_y + sh), skey.to_string()));
            sub_x += sw + 4;
        }

        // Render Analytics Rows
        let row_start_y = box_y + 58;
        let row_h = 44;
        let row_w = box_w - 16;
        let row_x = box_x + 8;

        if data.analytics_rows.is_empty() {
            draw_card_rect(hdc, row_x, row_start_y, row_w, 220, rgb(15, 23, 42), rgb(30, 41, 59));
            draw_text_custom(hdc, "⏳ 正在同步全维实战与收益画像...", row_x + 20, row_start_y + 90, (11.0 * min_s).max(9.0) as i32, 400, rgb(148, 163, 184), false);
        } else {
            for (idx, row) in data.analytics_rows.iter().take(5).enumerate() {
                let ry = row_start_y + idx as i32 * (row_h + 5);
                draw_card_rect(hdc, row_x, ry, row_w, row_h, rgb(15, 23, 42), rgb(30, 41, 59));

                // Left Icon Pill (28x28)
                let icon_x = row_x + 6;
                let icon_y = ry + 8;
                let icon_emoji = if row.is_boot { "🥾" } else { "🛡️" };
                let border_col = if row.delta_metric > 0.0 { rgb(200, 170, 110) } else { rgb(51, 65, 85) };
                draw_card_rect(hdc, icon_x, icon_y, 28, 28, rgb(22, 32, 50), border_col);
                draw_text_centered(hdc, icon_emoji, icon_x + 14, icon_y + 6, 10, 700, rgb(200, 170, 110), true);

                // Row Details
                let text_x = icon_x + 36;
                let name_disp = if row.name.chars().count() > 14 { format!("{}…", row.name.chars().take(13).collect::<String>()) } else { row.name.clone() };
                draw_text_custom(hdc, &name_disp, text_x, ry + 6, (10.5 * min_s).max(9.0) as i32, 700, rgb(248, 250, 252), false);

                if !row.label.is_empty() {
                    let tag_x = text_x + (name_disp.chars().count() as i32 * 11) + 6;
                    draw_text_custom(hdc, &format!("[{}]", row.label), tag_x, ry + 7, (9.0 * min_s).max(8.0) as i32, 700, rgb(200, 170, 110), false);
                }

                // Line 2: Stats
                let stats_text = if row.games > 0 {
                    format!("场次: {} | 选取: {:.1}%", row.games, row.pick_rate * 100.0)
                } else {
                    format!("选取率: {:.1}%", row.pick_rate * 100.0)
                };
                draw_text_custom(hdc, &stats_text, text_x, ry + 24, (9.0 * min_s).max(8.0) as i32, 400, rgb(100, 116, 139), false);

                // Right: Win Rate & Delta
                let right_x = row_x + row_w - 78;
                let wr_text = format!("{:.1}%", row.win_rate * 100.0);
                let wr_col = if row.win_rate >= 0.50 { rgb(16, 185, 129) } else { rgb(244, 63, 94) };
                draw_text_custom(hdc, &wr_text, right_x, ry + 6, (11.0 * min_s).max(9.5) as i32, 700, wr_col, false);

                let metric_label = if cur_sub == "featured" {
                    format!("评分 {:.1}", row.delta_metric)
                } else if cur_sub == "items" {
                    format!("适配 {:+.1}%", row.delta_metric * 100.0)
                } else {
                    format!("收益 {:+.1}%", row.delta_metric * 100.0)
                };
                let m_col = if row.delta_metric > 0.0 { rgb(16, 185, 129) } else { rgb(148, 163, 184) };
                draw_text_custom(hdc, &metric_label, right_x - 12, ry + 24, (9.0 * min_s).max(8.0) as i32, 600, m_col, false);
            }
        }

        // Footer
        let footer_y = box_y + box_h - 22;
        let boot_text = if !data.boots_name.is_empty() {
            format!("👟 最优鞋: {}", data.boots_name)
        } else {
            "👟 鞋子: 推荐中...".to_string()
        };
        draw_text_custom(hdc, &boot_text, box_x + 10, footer_y, (9.5 * min_s).max(8.5) as i32, 600, rgb(56, 189, 248), false);

        let hint = "[按 ] 或 ✕ 关闭]";
        draw_text_custom(hdc, hint, box_x + box_w - 92, footer_y, (9.0 * min_s).max(8.0) as i32, 400, rgb(100, 116, 139), false);
    } else {
        // Recommend Tab:
        // 3. Algorithm Selection Bar:
        let algo_y = box_y + 30;
        draw_text_custom(hdc, "算法:", box_x + 10, algo_y + 2, (10.0 * min_s).max(9.0) as i32, 400, rgb(148, 163, 184), false);

        let cur_algo = if let Ok(guard) = ACTIVE_ALGORITHM.read() {
            guard.clone()
        } else {
            "SMART".to_string()
        };

        let presets = [
            ("SMART", "👑 智能"),
            ("HEX_FUN", "⚡ 海克斯"),
            ("HYBRID_FREE", "🧪 黑科技"),
            ("IRON_COUNTER", "🛡️ 铁血"),
        ];

        let mut algo_btn_x = box_x + 46;
        for (key, label) in &presets {
            let is_act = key == &cur_algo;
            let btn_w = 68;
            let btn_h = 18;
            let (bg_col, border_col, fg_col) = if is_act {
                (rgb(30, 41, 59), rgb(234, 179, 8), rgb(250, 204, 21))
            } else {
                (rgb(15, 23, 42), rgb(30, 41, 59), rgb(100, 116, 139))
            };
            draw_pill_badge(hdc, algo_btn_x, algo_y, btn_w, btn_h, bg_col, border_col, fg_col, label);
            new_item_algo_btns.push(((algo_btn_x, algo_y, algo_btn_x + btn_w, algo_y + btn_h), key.to_string()));
            algo_btn_x += btn_w + 4;
        }

        // 4. Core Hero Card (Next Item Card)
        let card_x = box_x + 8;
        let card_y = box_y + 54;
        let card_w = box_w - 16;
        let card_h = 84;
        draw_card_rect(hdc, card_x, card_y, card_w, card_h, rgb(15, 23, 42), rgb(30, 41, 59));

        if let Some(ref next) = data.next_item {
            // Left Icon Pill (44x44)
            let icon_box_x = card_x + 6;
            let icon_box_y = card_y + 6;
            let border_c = if next.badge.contains("克制") || next.badge.contains("针对") {
                rgb(244, 63, 94)
            } else if next.badge.contains("核心") || next.badge.contains("T1") {
                rgb(200, 170, 110)
            } else {
                rgb(56, 189, 248)
            };
            draw_card_rect(hdc, icon_box_x, icon_box_y, 44, 44, rgb(22, 32, 50), border_c);
            draw_text_centered(hdc, "🛡️", icon_box_x + 22, icon_box_y + 12, 14, 700, rgb(200, 170, 110), true);

            // Line 1: Item Name + Badge
            let detail_x = icon_box_x + 50;
            let name_text = format!("【{}】", next.name);
            draw_text_custom(hdc, &name_text, detail_x, card_y + 6, (12.0 * min_s).max(10.0) as i32, 700, rgb(248, 250, 252), false);

            if !next.badge.is_empty() {
                let badge_x = detail_x + (name_text.chars().count() as i32 * 12) + 6;
                let badge_col = if next.badge.contains("克制") { rgb(244, 63, 94) } else { rgb(250, 204, 21) };
                draw_text_custom(hdc, &next.badge, badge_x, card_y + 7, (10.0 * min_s).max(9.0) as i32, 700, badge_col, false);
            }

            // Line 2: Dynamic Gold Progress Bar + Gold Text
            let bar_x = detail_x;
            let bar_y = card_y + 28;
            let bar_total_w = 160;
            let bar_h = 7;

            draw_card_rect(hdc, bar_x, bar_y, bar_total_w, bar_h, rgb(9, 14, 23), rgb(30, 41, 59));
            let fill_w = ((next.progress_pct * bar_total_w as f64) as i32).clamp(0, bar_total_w - 2);
            if fill_w > 0 {
                let fill_col = if next.is_affordable {
                    rgb(16, 185, 129) // Emerald green
                } else if next.badge.contains("克制") {
                    rgb(244, 63, 94) // Crimson
                } else {
                    rgb(245, 158, 11) // Amber gold
                };
                draw_card_rect(hdc, bar_x + 1, bar_y + 1, fill_w, bar_h - 2, fill_col, fill_col);
                draw_h_line(hdc, bar_x + 1, bar_y + 1, bar_x + fill_w, rgb(255, 255, 255));
            }

            let g_text = if !next.gold_status_text.is_empty() {
                next.gold_status_text.clone()
            } else if next.is_affordable {
                format!("💰 {}/{}g  [🎉可直接全款]", data.current_gold as i64, next.net_cost)
            } else {
                format!("💰 {}/{}g (差{}g)", data.current_gold as i64, next.net_cost, next.gold_diff)
            };
            let g_col = if next.is_affordable { rgb(16, 185, 129) } else { rgb(148, 163, 184) };
            draw_text_custom(hdc, &g_text, bar_x + bar_total_w + 8, bar_y - 2, (10.0 * min_s).max(8.5) as i32, if next.is_affordable { 700 } else { 400 }, g_col, false);

            // Line 3: Tactical Reason
            let r_text = if !next.reason.is_empty() {
                if next.reason.chars().count() > 32 {
                    format!("{}…", next.reason.chars().take(31).collect::<String>())
                } else {
                    next.reason.clone()
                }
            } else {
                "基于实时流派与敌方态势推演最佳出装".to_string()
            };
            let r_col = if next.badge.contains("克制") {
                rgb(244, 63, 94)
            } else if next.badge.contains("核心") {
                rgb(250, 204, 21)
            } else {
                rgb(56, 189, 248)
            };
            draw_text_custom(hdc, &r_text, detail_x, card_y + 44, (10.0 * min_s).max(8.5) as i32, 400, r_col, false);
        } else {
            draw_text_custom(hdc, "等待局内推演中...", card_x + 12, card_y + 32, (11.0 * min_s).max(9.0) as i32, 400, rgb(148, 163, 184), false);
        }

        // 5. Terminal Trio Miniature Chain (终局神装三件套)
        let trio_y = card_y + card_h + 8;
        draw_text_custom(hdc, "🏆 终局神装:", box_x + 10, trio_y + 2, (10.0 * min_s).max(9.0) as i32, 700, rgb(200, 170, 110), false);

        let mut trio_x = box_x + 82;
        if !data.trio_items.is_empty() {
            for (idx, item) in data.trio_items.iter().take(3).enumerate() {
                let (label, bg, border, fg) = if item.is_owned {
                    (format!("✓ {}", item.name), rgb(6, 78, 59), rgb(16, 185, 129), rgb(52, 211, 153))
                } else {
                    (item.name.clone(), rgb(15, 23, 42), rgb(51, 65, 85), rgb(148, 163, 184))
                };
                let pill_w = ((label.chars().count() as i32 * 10) + 12).clamp(48, 88);
                draw_pill_badge(hdc, trio_x, trio_y, pill_w, 20, bg, border, fg, &label);
                trio_x += pill_w + 3;

                if idx < 2 && idx < data.trio_items.len() - 1 {
                    draw_text_custom(hdc, "+", trio_x, trio_y + 2, (9.0 * min_s).max(8.0) as i32, 700, rgb(71, 85, 105), false);
                    trio_x += 10;
                }
            }

            if data.trio_win_rate > 0.0 {
                let wr_str = format!("终局 {:.1}%", data.trio_win_rate * 100.0);
                draw_text_custom(hdc, &wr_str, trio_x + 6, trio_y + 2, (10.0 * min_s).max(9.0) as i32, 700, rgb(16, 185, 129), false);
            }
        } else {
            draw_text_custom(hdc, "推演组合生成中...", trio_x, trio_y + 2, (10.0 * min_s).max(9.0) as i32, 400, rgb(100, 116, 139), false);
        }

        // 6. Alternatives / Situational Counter Row
        let alt_y = trio_y + 26;
        if let Some(first_alt) = data.alternatives.first() {
            let prefix = if first_alt.reason.contains("针对") || first_alt.reason.contains("重伤") || first_alt.reason.contains("破盾") {
                "⚠️ 针对备选:"
            } else {
                "⚔️ 备选神装:"
            };
            draw_text_custom(hdc, prefix, box_x + 10, alt_y + 2, (9.5 * min_s).max(8.5) as i32, 700, if prefix.contains("⚠️") { rgb(244, 63, 94) } else { rgb(200, 170, 110) }, false);

            let alt_str = format!("{} (抵扣后 {}G) - {}", first_alt.name, first_alt.net_cost, first_alt.reason);
            let disp_alt = if alt_str.chars().count() > 34 { format!("{}…", alt_str.chars().take(33).collect::<String>()) } else { alt_str };
            draw_text_custom(hdc, &disp_alt, box_x + 82, alt_y + 2, (9.0 * min_s).max(8.0) as i32, 400, rgb(148, 163, 184), false);
        }

        // 7. Footer Row: Boots + Hotkey Hint
        let footer_y = alt_y + 24;
        let boot_text = if !data.boots_name.is_empty() {
            format!("👟 鞋子: {}", data.boots_name)
        } else {
            "👟 鞋子: 推荐中...".to_string()
        };
        draw_text_custom(hdc, &boot_text, box_x + 10, footer_y, (10.0 * min_s).max(9.0) as i32, 600, rgb(56, 189, 248), false);

        let hint = "[按 ] 或 ✕ 关闭]";
        draw_text_custom(hdc, hint, box_x + box_w - 92, footer_y, (9.0 * min_s).max(8.0) as i32, 400, rgb(100, 116, 139), false);
    }

    // Save hotzones
    if let Ok(mut gz) = GLOBAL_HOTZONES.write() {
        gz.item_close_btn = new_item_close;
        gz.item_algo_btns = new_item_algo_btns;
        gz.item_main_tabs = new_item_main_tabs;
        gz.item_subtabs = new_item_subtabs;
    }
}

#[cfg(windows)]
unsafe fn render_bench_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    data: &BenchOverlayData,
) {
    if data.champions.is_empty() {
        return;
    }

    let sx = width as f64 / 1280.0;
    let sy = height as f64 / 720.0;

    // 1280x720 baseline geometry constants from Python bench_overlay.py:
    // BENCH_START_X = 355.0, BENCH_PITCH_X = 58.0, BENCH_ICON_W = 46.0, BENCH_BADGE_Y = 59.0, BENCH_BADGE_H = 16.0
    let badge_w = (46.0 * sx).round() as i32;
    let badge_h = (16.0 * sy).round().max(14.0) as i32;
    let badge_y = (59.0 * sy).round() as i32;

    for champ in &data.champions {
        let slot_x = ((355.0 + champ.slot_index as f64 * 58.0) * sx).round() as i32;
        let wr_pct = champ.win_rate * 100.0;

        let (bg_col, border_col, text_col, prefix) = if wr_pct >= 52.0 {
            (
                rgb(11, 43, 26),    // Emerald dark green
                rgb(0, 230, 150),   // Bright aurora green
                rgb(230, 255, 242), // Fluorescent white-green
                "★",
            )
        } else if wr_pct >= 50.0 {
            (
                rgb(13, 32, 56),    // Hextech dark blue
                rgb(56, 189, 248),  // Sky tech blue
                rgb(224, 242, 254), // Light blue white
                "",
            )
        } else if wr_pct >= 48.0 {
            (
                rgb(35, 25, 15),    // Classic dark gold
                rgb(200, 155, 60),  // Hextech gold border
                rgb(255, 235, 180), // Bright gold
                "",
            )
        } else {
            (
                rgb(45, 15, 20),    // Warning dark crimson
                rgb(239, 68, 68),   // Alert crimson border
                rgb(254, 202, 202), // Light pink white
                "▼",
            )
        };

        let badge_text = format!("{}{:.1}%", prefix, wr_pct);
        draw_pill_badge(
            hdc,
            slot_x,
            badge_y,
            badge_w,
            badge_h,
            bg_col,
            border_col,
            text_col,
            &badge_text,
        );
    }
}

#[cfg(windows)]
const fn rgb(r: u8, g: u8, b: u8) -> windows::Win32::Foundation::COLORREF {
    windows::Win32::Foundation::COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

#[cfg(windows)]
unsafe fn render_ranked_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    _width: i32,
    _height: i32,
    data: &RankedOverlayData,
) {
    let inter = if let Ok(guard) = RANKED_INTERACTION.read() {
        guard.clone()
    } else {
        RankedInteractionState::default()
    };

    let left_collapsed = inter.left_wing_collapsed;
    let right_collapsed = inter.right_wing_collapsed;
    let active_tab = inter.active_tab.clone();
    let counter_scroll_offset = inter.counter_scroll_offset;
    let sample_count = inter.sample_match_count;
    let btn_status = inter.btn_status_text.as_ref().and_then(|txt| {
        if let Some(t) = inter.btn_status_time {
            if t.elapsed() < Duration::from_secs(3) {
                return Some(txt.clone());
            }
        }
        None
    });

    // Auto-switch to runes when locked in
    if data.decision_card.my_locked && data.decision_card.rune_guide.is_some() && active_tab != "runes" {
        if let Ok(mut inter) = RANKED_INTERACTION.write() {
            if !inter.last_local_locked {
                inter.last_local_locked = true;
                inter.active_tab = "runes".to_string();
            }
        }
    } else if !data.decision_card.my_locked {
        if let Ok(mut inter) = RANKED_INTERACTION.write() {
            if inter.last_local_locked {
                inter.last_local_locked = false;
            }
        }
    }

    let (actual_left_margin, actual_offset_y, cw, ch) = if let Some(r) = crate::vision::ScreenCapture::get_lol_client_rect() {
        let sx = r.width as f64 / 1280.0;
        let sy = r.height as f64 / 720.0;
        let top_h = (44.0 * sy) as i32;
        let lw = if left_collapsed { (24.0 * sx) as i32 } else { (168.0 * sx) as i32 };
        let off_y = if r.y >= top_h { top_h } else { r.y.max(0) };
        let off_x = if r.x >= lw { lw } else { 0 };
        (off_x, off_y, r.width, r.height)
    } else {
        let lw = if left_collapsed { 24 } else { 168 };
        (lw, 44, 1280, 720)
    };

    let sx = cw as f64 / 1280.0;
    let sy = sx; // Keep uniform aspect ratio

    let mut new_hotzones = HudHotzones::default();

    // -------------------------------------------------------------
    // 1. Top Decision Navigation Bar (Tabs, Pills/Runes, Samples)
    // -------------------------------------------------------------
    let card_w = (860.0 * sx).min(cw as f64 - 20.0) as i32;
    let card_h = (32.0 * sy).max(28.0) as i32;
    let card_x = actual_left_margin + (cw - card_w) / 2;
    let card_y = if actual_offset_y >= card_h + 4 {
        (actual_offset_y - card_h) / 2
    } else {
        2
    };

    let active_border_col = match active_tab.as_str() {
        "lane_counter" => rgb(0, 230, 150),
        "multi_counter" => rgb(56, 189, 248),
        "synergy" => rgb(167, 139, 250),
        "runes" => rgb(245, 158, 11),
        _ => rgb(56, 189, 248),
    };

    // Draw base container card
    draw_card_rect(hdc, card_x, card_y, card_w, card_h, rgb(8, 14, 24), active_border_col);

    // 1.1 Left Tabs
    let mut available_tabs = vec![
        ("lane_counter", "🎯 对位"),
        ("multi_counter", "⚔️ 全阵容"),
        ("synergy", "🤝 协同"),
    ];
    if data.decision_card.rune_guide.is_some() {
        available_tabs.push(("runes", "⚡ 符文"));
    }

    let tab_btn_w = (54.0 * sx) as i32;
    let tab_btn_h = card_h - 8;
    let tab_gap = (3.0 * sx) as i32;
    let tab_start_x = card_x + (6.0 * sx) as i32;
    let tab_y1 = card_y + 4;
    let tab_y2 = tab_y1 + tab_btn_h;

    for (idx, (t_key, t_label)) in available_tabs.iter().enumerate() {
        let tx1 = tab_start_x + idx as i32 * (tab_btn_w + tab_gap);
        let tx2 = tx1 + tab_btn_w;
        let is_active = active_tab == *t_key;
        let (t_bg, t_bd, t_fg) = if is_active {
            match *t_key {
                "lane_counter" => (rgb(6, 78, 59), active_border_col, rgb(255, 255, 255)),
                "multi_counter" => (rgb(30, 58, 138), active_border_col, rgb(255, 255, 255)),
                "synergy" => (rgb(76, 29, 149), active_border_col, rgb(255, 255, 255)),
                _ => (rgb(120, 53, 15), active_border_col, rgb(255, 255, 255)),
            }
        } else {
            (rgb(17, 24, 39), rgb(55, 65, 81), rgb(156, 163, 175))
        };
        draw_card_rect(hdc, tx1, tab_y1, tab_btn_w, tab_btn_h, t_bg, t_bd);
        draw_text_line(hdc, t_label, tx1 + 6, tab_y1 + 4, t_fg);
        new_hotzones.tab_btns.push(((tx1, tab_y1, tx2, tab_y2), t_key.to_string()));
    }

    let tabs_end_x = tab_start_x + available_tabs.len() as i32 * (tab_btn_w + tab_gap) + 4;

    // 1.2 Right Sample Switcher
    let sample_counts = [10, 20, 30, 50];
    let s_btn_w = (28.0 * sx) as i32;
    let s_btn_h = tab_btn_h;
    let s_gap = (3.0 * sx) as i32;
    let total_s_w = sample_counts.len() as i32 * s_btn_w + (sample_counts.len() as i32 - 1) * s_gap;
    let s_group_right = card_x + card_w - (8.0 * sx) as i32;
    let s_group_left = s_group_right - total_s_w;

    let s_label_x = s_group_left - (45.0 * sx) as i32;
    draw_text_line(hdc, "📊样本:", s_label_x, tab_y1 + 4, rgb(148, 163, 184));

    for (idx, &sc) in sample_counts.iter().enumerate() {
        let sbx1 = s_group_left + idx as i32 * (s_btn_w + s_gap);
        let sby1 = tab_y1;
        let sbx2 = sbx1 + s_btn_w;
        let sby2 = sby1 + s_btn_h;
        let is_active_sample = sample_count == sc;

        let (s_bg, s_border, s_fg) = if is_active_sample {
            (rgb(6, 78, 59), rgb(52, 211, 153), rgb(255, 255, 255))
        } else {
            (rgb(11, 18, 30), rgb(31, 41, 61), rgb(148, 163, 184))
        };

        draw_card_rect(hdc, sbx1, sby1, s_btn_w, s_btn_h, s_bg, s_border);
        let s_text = format!("{}场", sc);
        draw_text_line(hdc, &s_text, sbx1 + 3, sby1 + 3, s_fg);
        new_hotzones.sample_btns.push(((sbx1, sby1, sbx2, sby2), sc));
    }

    let middle_end_x = s_label_x - (8.0 * sx) as i32;

    // 1.3 Middle Area Content
    let mid_start_x = tabs_end_x + 6;
    let maybe_rg = if active_tab == "runes" { data.decision_card.rune_guide.as_ref() } else { None };
    if let Some(rg) = maybe_rg {
        let info_text = if rg.skill_order.is_empty() {
            format!("⚡ [{} - {}] 对线符文已匹配", rg.champion_name, rg.position_cn)
        } else {
            format!("⚡ [{} - {}] 对线符文已匹配  加点: {}", rg.champion_name, rg.position_cn, rg.skill_order)
        };
        draw_text_line(hdc, &info_text, mid_start_x, tab_y1 + 4, rgb(224, 242, 254));

        let btn_w = (150.0 * sx) as i32;
        let bx2 = middle_end_x;
        let bx1 = bx2 - btn_w;
        let (btn_bg, btn_border, btn_label) = if let Some(ref status) = btn_status {
            (rgb(6, 78, 59), rgb(52, 211, 153), status.as_str())
        } else {
            (rgb(2, 132, 199), rgb(56, 189, 248), "⚡ 一键导入 OP.GG 符文")
        };
        draw_card_rect(hdc, bx1, tab_y1, btn_w, tab_btn_h, btn_bg, btn_border);
        draw_text_line(hdc, btn_label, bx1 + 8, tab_y1 + 4, rgb(255, 255, 255));
        new_hotzones.rune_btn = Some((bx1, tab_y1, bx2, tab_y2));
    } else {
        let items = if active_tab == "lane_counter" {
            &data.decision_card.lane_counters
        } else if active_tab == "multi_counter" {
            &data.decision_card.multi_counters
        } else {
            &data.decision_card.synergy_picks
        };

        if items.is_empty() {
            draw_text_line(hdc, "⏳ 正在实时分析对位克制与推荐英雄...", mid_start_x + 10, tab_y1 + 4, rgb(148, 163, 184));
        } else {
            let visible_count = 4;
            let offset = counter_scroll_offset.min(items.len().saturating_sub(visible_count)).min(items.len());

            // Left scroll arrow ◀
            let arr_w = (16.0 * sx) as i32;
            let arr_lx1 = mid_start_x;
            let arr_lx2 = arr_lx1 + arr_w;
            let can_prev = offset > 0;
            let (arr_l_bg, arr_l_fg) = if can_prev {
                (rgb(30, 41, 59), rgb(56, 189, 248))
            } else {
                (rgb(11, 19, 32), rgb(51, 65, 85))
            };
            draw_card_rect(hdc, arr_lx1, tab_y1, arr_w, tab_btn_h, arr_l_bg, rgb(51, 65, 85));
            draw_text_line(hdc, "◀", arr_lx1 + 4, tab_y1 + 4, arr_l_fg);
            if can_prev {
                new_hotzones.nav_btns.push(((arr_lx1, tab_y1, arr_lx2, tab_y2), -2));
            }

            // Hero pills
            let pill_start_x = arr_lx2 + (4.0 * sx) as i32;
            let pill_w = (86.0 * sx) as i32;
            let pill_gap = (4.0 * sx) as i32;
            let slice_end = items.len().min(offset + visible_count);
            let visible_items = if offset <= slice_end { &items[offset..slice_end] } else { &[] };

            for (p_idx, item) in visible_items.iter().enumerate() {
                let px1 = pill_start_x + p_idx as i32 * (pill_w + pill_gap);
                let (t_fg, t_bg) = match item.tier {
                    1 => (rgb(0, 230, 150), rgb(4, 47, 26)),
                    2 => (rgb(59, 130, 246), rgb(12, 43, 78)),
                    3 => (rgb(200, 170, 110), rgb(58, 43, 22)),
                    4 => (rgb(249, 115, 22), rgb(63, 27, 10)),
                    _ => (rgb(239, 68, 68), rgb(69, 10, 10)),
                };
                draw_card_rect(hdc, px1, tab_y1, pill_w, tab_btn_h, t_bg, t_fg);
                let p_label = if item.counter_win_rate > 0.0 {
                    format!("{} {:.0}%", item.name, item.counter_win_rate * 100.0)
                } else {
                    item.name.clone()
                };
                draw_text_line(hdc, &p_label, px1 + 4, tab_y1 + 4, rgb(255, 255, 255));
            }

            // Right scroll arrow ▶
            let pills_end_x = pill_start_x + visible_items.len() as i32 * (pill_w + pill_gap);
            let arr_rx1 = pills_end_x;
            let arr_rx2 = arr_rx1 + arr_w;
            let can_next = offset + visible_count < items.len();
            let (arr_r_bg, arr_r_fg) = if can_next {
                (rgb(30, 41, 59), rgb(56, 189, 248))
            } else {
                (rgb(11, 19, 32), rgb(51, 65, 85))
            };
            draw_card_rect(hdc, arr_rx1, tab_y1, arr_w, tab_btn_h, arr_r_bg, rgb(51, 65, 85));
            draw_text_line(hdc, "▶", arr_rx1 + 4, tab_y1 + 4, arr_r_fg);
            if can_next {
                new_hotzones.nav_btns.push(((arr_rx1, tab_y1, arr_rx2, tab_y2), 2));
            }
        }
    }

    // -------------------------------------------------------------
    // 2. Left Wing (我方战绩情报)
    // -------------------------------------------------------------
    let mini_w = (24.0 * sx) as i32;
    let wing_w = (168.0 * sx) as i32;

    if left_collapsed {
        let tx1 = 2;
        let tx2 = tx1 + mini_w;
        let ty1 = actual_offset_y + (210.0 * sy) as i32;
        let ty2 = ty1 + (80.0 * sy) as i32;
        draw_card_rect(hdc, tx1, ty1, mini_w, ty2 - ty1, rgb(8, 20, 38), rgb(2, 132, 199));
        draw_text_line(hdc, "▶", tx1 + 6, ty1 + 8, rgb(56, 189, 248));
        draw_text_line(hdc, "我", tx1 + 6, ty1 + 22, rgb(56, 189, 248));
        draw_text_line(hdc, "方", tx1 + 6, ty1 + 36, rgb(56, 189, 248));
        draw_text_line(hdc, "战", tx1 + 6, ty1 + 50, rgb(56, 189, 248));
        draw_text_line(hdc, "绩", tx1 + 6, ty1 + 64, rgb(56, 189, 248));
        new_hotzones.left_wing_toggle = Some((tx1, ty1, tx2, ty2));
    } else {
        let wx1 = 2;
        let wx2 = actual_left_margin - 4;
        let hy1 = actual_offset_y + (64.0 * sy) as i32;
        let hy2 = hy1 + (24.0 * sy) as i32;

        draw_card_rect(hdc, wx1, hy1, wx2 - wx1, hy2 - hy1, rgb(8, 19, 36), rgb(30, 58, 95));
        draw_text_line(hdc, "🛡️ 我方战绩情报", wx1 + 6, hy1 + 4, rgb(56, 189, 248));

        let btn_w = (48.0 * sx) as i32;
        let bx1 = wx2 - btn_w - 2;
        let bx2 = wx2 - 2;
        draw_card_rect(hdc, bx1, hy1 + 2, btn_w, hy2 - hy1 - 4, rgb(12, 29, 54), rgb(2, 132, 199));
        draw_text_line(hdc, "◀ 收起", bx1 + 4, hy1 + 4, rgb(125, 211, 252));
        new_hotzones.left_wing_toggle = Some((bx1, hy1 + 2, bx2, hy2 - 2));

        let card_h = (74.0 * sy) as i32;
        for (idx, slot) in data.my_team.iter().take(5).enumerate() {
            let cy1 = actual_offset_y + ((100.0 + idx as f64 * 92.0 - 8.0) * sy) as i32;
            let is_inspected = inter.inspect_slot == Some((true, idx));
            let card_border = if is_inspected { rgb(251, 191, 36) } else { rgb(30, 58, 95) };

            draw_card_rect(hdc, wx1, cy1, wx2 - wx1, card_h, rgb(9, 20, 36), card_border);
            draw_h_line(hdc, wx1 + 4, cy1 + 1, wx2 - 4, if is_inspected { rgb(251, 191, 36) } else { rgb(2, 132, 199) });
            new_hotzones.inspect_slot_btns.push(((wx1, cy1, wx2, cy1 + card_h), true, idx));

            let slot_name = if let Some(ref p) = slot.profile {
                if !p.summoner_name.is_empty() {
                    let raw_n = p.summoner_name.clone();
                    if raw_n.chars().count() > 6 { format!("{}…", raw_n.chars().take(5).collect::<String>()) } else { raw_n }
                } else if slot.champion_name.is_empty() {
                    format!("我方{}楼", idx + 1)
                } else {
                    slot.champion_name.clone()
                }
            } else if slot.champion_name.is_empty() {
                format!("我方{}楼", idx + 1)
            } else {
                slot.champion_name.clone()
            };

            let is_real_champ = !slot.champion_name.is_empty()
                && !slot.champion_name.starts_with("队友")
                && !slot.champion_name.starts_with("我方");

            let row1_left = if is_real_champ && slot.champion_name != slot_name {
                format!("[{}] {} · {}", slot.position_cn, slot_name, slot.champion_name)
            } else {
                format!("[{}] {}", slot.position_cn, slot_name)
            };
            let tier_str = if let Some(ref p) = slot.profile {
                format!("{} {}P", p.tier_str, p.lp)
            } else {
                format!("T{}级", slot.tier)
            };
            let (tier_w, _) = measure_text_line(hdc, &tier_str);
            let tier_x = (wx2 - 6 - tier_w).max(wx1 + 50);
            draw_text_line(hdc, &tier_str, tier_x, cy1 + 5, rgb(200, 170, 110));

            let max_avail_w = (tier_x - wx1 - 10).max(40);
            let max_hw = (max_avail_w / 7) as usize;
            let disp_row1 = truncate_text_to_half_width(&row1_left, max_hw);
            draw_text_line(hdc, &disp_row1, wx1 + 6, cy1 + 5, rgb(255, 255, 255));

            if let Some(ref p) = slot.profile {
                if (p.recent_wins + p.recent_losses) > 0 {
                    let (row2_text, wr_fg) = if p.cur_champ_id > 0 && p.cur_champ_games > 0 {
                        let col = if p.cur_champ_win_rate_pct >= 60 { rgb(52, 211, 153) } else if p.cur_champ_win_rate_pct >= 50 { rgb(56, 189, 248) } else { rgb(248, 113, 113) };
                        (format!("⭐该英雄近{}场 {}% ({}胜{}负)", p.cur_champ_games, p.cur_champ_win_rate_pct, p.cur_champ_wins, p.cur_champ_losses), col)
                    } else if p.cur_champ_id > 0 && p.cur_champ_games == 0 {
                        (format!("近{}场 {}% · ⚠️试英雄(0场)", p.sample_count, p.win_rate_pct), rgb(251, 191, 36))
                    } else {
                        let col = if p.win_rate_pct >= 60 { rgb(52, 211, 153) } else if p.win_rate_pct >= 50 { rgb(56, 189, 248) } else { rgb(248, 113, 113) };
                        (format!("近{}场 {}% ({}胜{}负)", p.sample_count, p.win_rate_pct, p.recent_wins, p.recent_losses), col)
                    };

                    let dot_start_x = wx2 - 48;
                    let dot_cy = cy1 + 29;
                    let max_r2_w = (dot_start_x - wx1 - 10).max(40);
                    let max_r2_hw = (max_r2_w / 7) as usize;
                    let disp_row2 = truncate_text_to_half_width(&row2_text, max_r2_hw);
                    draw_text_line(hdc, &disp_row2, wx1 + 6, cy1 + 24, wr_fg);

                    for (d_idx, &is_win) in p.recent_dots.iter().take(5).enumerate() {
                        let dx = dot_start_x + d_idx as i32 * 9;
                        let (fill_c, border_c) = if is_win {
                            (rgb(16, 185, 129), rgb(52, 211, 153))
                        } else {
                            (rgb(239, 68, 68), rgb(248, 113, 113))
                        };
                        draw_dot(hdc, dx, dot_cy, 3, fill_c, border_c);
                    }
                } else if p.wins > 0 || p.losses > 0 {
                    let s_text = format!("排位 {}胜{}负 (总胜率{}%)", p.wins, p.losses, p.overall_win_rate_pct);
                    draw_text_line(hdc, &s_text, wx1 + 6, cy1 + 24, rgb(56, 189, 248));
                } else if p.total_games > 0 {
                    let s_text = format!("排位已获 {} 胜 (暂无近期明细)", p.total_games);
                    draw_text_line(hdc, &s_text, wx1 + 6, cy1 + 24, rgb(148, 163, 184));
                } else {
                    draw_text_line(hdc, "排位未定级 · 暂无实战记录", wx1 + 6, cy1 + 24, rgb(148, 163, 184));
                }
            } else {
                draw_text_line(hdc, "⏳ 正在同步实战画像...", wx1 + 6, cy1 + 24, rgb(148, 163, 184));
            }

            if let Some(ref p) = slot.profile {
                if (p.recent_wins + p.recent_losses) > 0 && !p.badge_text.is_empty() {
                    let badge_w = 84.min(wx2 - wx1 - 12);
                    let (b_bg, b_border, b_fg) = if p.streak_type == "win" || p.badge_text.contains("连胜") {
                        (rgb(6, 78, 59), rgb(52, 211, 153), rgb(52, 211, 153))
                    } else if p.streak_type == "loss" || p.badge_text.contains("连败") {
                        (rgb(69, 10, 10), rgb(248, 113, 113), rgb(248, 113, 113))
                    } else if p.badge_text.contains("通天代") || p.badge_text.contains("绝活") {
                        (rgb(30, 58, 138), rgb(96, 165, 250), rgb(96, 165, 250))
                    } else {
                        (rgb(17, 24, 39), rgb(148, 163, 184), rgb(148, 163, 184))
                    };

                    draw_card_rect(hdc, wx1 + 6, cy1 + 44, badge_w, 18, b_bg, b_border);
                    draw_text_line(hdc, &p.badge_text, wx1 + 10, cy1 + 46, b_fg);
                } else if p.wins > 0 || p.losses > 0 {
                    draw_text_line(hdc, "📊 实战数据已就绪", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
                } else {
                    draw_text_line(hdc, "⏳ 等待首局实战样本", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
                }
            } else {
                draw_text_line(hdc, "🔄 实时连接 LCU 对局实录...", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
            }
        }
    }

    // -------------------------------------------------------------
    // 3. Right Wing (敌方战绩情报)
    // -------------------------------------------------------------
    if right_collapsed {
        let tx1 = actual_left_margin + cw + 2;
        let tx2 = tx1 + mini_w;
        let ty1 = actual_offset_y + (210.0 * sy) as i32;
        let ty2 = ty1 + (80.0 * sy) as i32;
        draw_card_rect(hdc, tx1, ty1, mini_w, ty2 - ty1, rgb(28, 9, 18), rgb(225, 29, 72));
        draw_text_line(hdc, "◀", tx1 + 6, ty1 + 8, rgb(244, 63, 94));
        draw_text_line(hdc, "敌", tx1 + 6, ty1 + 22, rgb(244, 63, 94));
        draw_text_line(hdc, "方", tx1 + 6, ty1 + 36, rgb(244, 63, 94));
        draw_text_line(hdc, "战", tx1 + 6, ty1 + 50, rgb(244, 63, 94));
        draw_text_line(hdc, "绩", tx1 + 6, ty1 + 64, rgb(244, 63, 94));
        new_hotzones.right_wing_toggle = Some((tx1, ty1, tx2, ty2));
    } else {
        let wx1 = actual_left_margin + cw + 4;
        let wx2 = actual_left_margin + cw + wing_w - 2;
        let hy1 = actual_offset_y + (64.0 * sy) as i32;
        let hy2 = hy1 + (24.0 * sy) as i32;

        draw_card_rect(hdc, wx1, hy1, wx2 - wx1, hy2 - hy1, rgb(26, 11, 20), rgb(74, 24, 32));
        draw_text_line(hdc, "⚔️ 敌方战绩情报", wx1 + 6, hy1 + 4, rgb(244, 63, 94));

        let btn_w = (48.0 * sx) as i32;
        let bx1 = wx2 - btn_w - 2;
        let bx2 = wx2 - 2;
        draw_card_rect(hdc, bx1, hy1 + 2, btn_w, hy2 - hy1 - 4, rgb(43, 14, 27), rgb(225, 29, 72));
        draw_text_line(hdc, "收起 ▶", bx1 + 4, hy1 + 4, rgb(253, 164, 175));
        new_hotzones.right_wing_toggle = Some((bx1, hy1 + 2, bx2, hy2 - 2));

        let card_h = (74.0 * sy) as i32;
        for (idx, slot) in data.their_team.iter().take(5).enumerate() {
            let cy1 = actual_offset_y + ((100.0 + idx as f64 * 92.0 - 8.0) * sy) as i32;
            let is_inspected = inter.inspect_slot == Some((false, idx));
            let card_border = if is_inspected { rgb(251, 191, 36) } else { rgb(74, 24, 32) };

            draw_card_rect(hdc, wx1, cy1, wx2 - wx1, card_h, rgb(26, 12, 22), card_border);
            draw_h_line(hdc, wx1 + 4, cy1 + 1, wx2 - 4, if is_inspected { rgb(251, 191, 36) } else { rgb(225, 29, 72) });
            new_hotzones.inspect_slot_btns.push(((wx1, cy1, wx2, cy1 + card_h), false, idx));

            let enemy_cname = if slot.champion_name.is_empty() {
                format!("敌方{}楼", idx + 1)
            } else {
                slot.champion_name.clone()
            };

            let slot_name = if let Some(ref p) = slot.profile {
                if !p.summoner_name.is_empty() {
                    let raw_n = p.summoner_name.clone();
                    if raw_n.chars().count() > 6 { format!("{}…", raw_n.chars().take(5).collect::<String>()) } else { raw_n }
                } else {
                    enemy_cname.clone()
                }
            } else {
                enemy_cname.clone()
            };

            let is_real_champ = !slot.champion_name.is_empty()
                && !slot.champion_name.starts_with("对手")
                && !slot.champion_name.starts_with("敌方");

            let row1_left = if is_real_champ && slot.champion_name != slot_name {
                format!("[{}] {} · {}", slot.position_cn, slot_name, slot.champion_name)
            } else {
                format!("[{}] {}", slot.position_cn, slot_name)
            };
            let tier_str = if let Some(ref p) = slot.profile {
                format!("{} {}P", p.tier_str, p.lp)
            } else {
                format!("T{}级", slot.tier)
            };
            let (tier_w, _) = measure_text_line(hdc, &tier_str);
            let tier_x = (wx2 - 6 - tier_w).max(wx1 + 50);
            draw_text_line(hdc, &tier_str, tier_x, cy1 + 5, rgb(200, 170, 110));

            let max_avail_w = (tier_x - wx1 - 10).max(40);
            let max_hw = (max_avail_w / 7) as usize;
            let disp_row1 = truncate_text_to_half_width(&row1_left, max_hw);
            draw_text_line(hdc, &disp_row1, wx1 + 6, cy1 + 5, rgb(255, 255, 255));

            if let Some(ref p) = slot.profile {
                if (p.recent_wins + p.recent_losses) > 0 {
                    let (row2_text, wr_fg) = if p.cur_champ_id > 0 && p.cur_champ_games > 0 {
                        let col = if p.cur_champ_win_rate_pct >= 60 { rgb(52, 211, 153) } else if p.cur_champ_win_rate_pct >= 50 { rgb(56, 189, 248) } else { rgb(248, 113, 113) };
                        (format!("⭐该英雄近{}场 {}% ({}胜{}负)", p.cur_champ_games, p.cur_champ_win_rate_pct, p.cur_champ_wins, p.cur_champ_losses), col)
                    } else if p.cur_champ_id > 0 && p.cur_champ_games == 0 {
                        (format!("近{}场 {}% · ⚠️试英雄(0场)", p.sample_count, p.win_rate_pct), rgb(251, 191, 36))
                    } else {
                        let col = if p.win_rate_pct >= 60 { rgb(52, 211, 153) } else if p.win_rate_pct >= 50 { rgb(56, 189, 248) } else { rgb(248, 113, 113) };
                        (format!("近{}场 {}% ({}胜{}负)", p.sample_count, p.win_rate_pct, p.recent_wins, p.recent_losses), col)
                    };

                    let dot_start_x = wx2 - 48;
                    let dot_cy = cy1 + 29;
                    let max_r2_w = (dot_start_x - wx1 - 10).max(40);
                    let max_r2_hw = (max_r2_w / 7) as usize;
                    let disp_row2 = truncate_text_to_half_width(&row2_text, max_r2_hw);
                    draw_text_line(hdc, &disp_row2, wx1 + 6, cy1 + 24, wr_fg);

                    for (d_idx, &is_win) in p.recent_dots.iter().take(5).enumerate() {
                        let dx = dot_start_x + d_idx as i32 * 9;
                        let (fill_c, border_c) = if is_win {
                            (rgb(16, 185, 129), rgb(52, 211, 153))
                        } else {
                            (rgb(239, 68, 68), rgb(248, 113, 113))
                        };
                        draw_dot(hdc, dx, dot_cy, 3, fill_c, border_c);
                    }
                } else if p.wins > 0 || p.losses > 0 {
                    let s_text = format!("排位 {}胜{}负 (总胜率{}%)", p.wins, p.losses, p.overall_win_rate_pct);
                    draw_text_line(hdc, &s_text, wx1 + 6, cy1 + 24, rgb(56, 189, 248));
                } else if p.total_games > 0 {
                    let s_text = format!("排位已获 {} 胜 (暂无近期明细)", p.total_games);
                    draw_text_line(hdc, &s_text, wx1 + 6, cy1 + 24, rgb(148, 163, 184));
                } else {
                    draw_text_line(hdc, "排位未定级 · 暂无实战记录", wx1 + 6, cy1 + 24, rgb(148, 163, 184));
                }

                if (p.recent_wins + p.recent_losses) > 0 && !p.badge_text.is_empty() {
                    let badge_w = 84.min(wx2 - wx1 - 12);
                    let (b_bg, b_border, b_fg) = if p.streak_type == "win" || p.badge_text.contains("连胜") {
                        (rgb(6, 78, 59), rgb(52, 211, 153), rgb(52, 211, 153))
                    } else if p.streak_type == "loss" || p.badge_text.contains("连败") {
                        (rgb(69, 10, 10), rgb(248, 113, 113), rgb(248, 113, 113))
                    } else if p.badge_text.contains("通天代") || p.badge_text.contains("绝活") {
                        (rgb(30, 58, 138), rgb(96, 165, 250), rgb(96, 165, 250))
                    } else {
                        (rgb(17, 24, 39), rgb(148, 163, 184), rgb(148, 163, 184))
                    };
                    draw_card_rect(hdc, wx1 + 6, cy1 + 44, badge_w, 18, b_bg, b_border);
                    draw_text_line(hdc, &p.badge_text, wx1 + 10, cy1 + 46, b_fg);
                } else if p.wins > 0 || p.losses > 0 {
                    draw_text_line(hdc, "📊 实战数据已就绪", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
                } else {
                    draw_text_line(hdc, "⏳ 等待首局实战样本", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
                }
            } else {
                draw_text_line(hdc, "🔒 选人期匿名保护", wx1 + 6, cy1 + 25, rgb(148, 163, 184));
                draw_text_line(hdc, "⚔️ 进入对局后自动解锁", wx1 + 6, cy1 + 46, rgb(100, 116, 139));
            }
        }
    }

    // -------------------------------------------------------------
    // 4. 5v5 Slots (My Team Left X=205, Their Team Right X=950)
    // -------------------------------------------------------------
    let slot_pitch_y = (92.0 * sy) as i32;
    let slot_card_w = ((130.0 * sx) as i32).max(124);
    let slot_card_h = (44.0 * sy) as i32;
    let slot_start_y = actual_offset_y + (100.0 * sy) as i32;

    // 4.1 My Team Slots
    let left_slot_x = actual_left_margin + (205.0 * sx) as i32;
    for (idx, slot) in data.my_team.iter().take(5).enumerate() {
        let sy1 = slot_start_y + idx as i32 * slot_pitch_y;
        let border_col = if slot.is_local_player { rgb(200, 170, 110) } else { rgb(42, 67, 101) };

        draw_card_rect(hdc, left_slot_x, sy1, slot_card_w, slot_card_h, rgb(9, 20, 40), border_col);

        let (t_fg, t_bg) = match slot.tier {
            1 => (rgb(0, 230, 150), rgb(4, 47, 26)),
            2 => (rgb(59, 130, 246), rgb(12, 43, 78)),
            3 => (rgb(200, 170, 110), rgb(58, 43, 22)),
            4 => (rgb(249, 115, 22), rgb(63, 27, 10)),
            _ => (rgb(239, 68, 68), rgb(69, 10, 10)),
        };
        draw_card_rect(hdc, left_slot_x + 4, sy1 + 4, 22, 14, t_bg, t_fg);
        let t_str = format!("T{}", slot.tier);
        draw_text_line(hdc, &t_str, left_slot_x + 6, sy1 + 5, t_fg);

        let pos_bx1 = left_slot_x + 30;
        let pos_bx2 = pos_bx1 + 42;
        let (p_bg, p_border, p_fg) = if slot.is_manual_pos {
            (rgb(39, 23, 4), rgb(245, 158, 11), rgb(251, 191, 36))
        } else {
            (rgb(13, 27, 42), rgb(96, 165, 250), rgb(224, 242, 254))
        };
        draw_card_rect(hdc, pos_bx1, sy1 + 4, 42, 14, p_bg, p_border);
        let pos_label = format!("{}▾", slot.position_cn);
        draw_text_line(hdc, &pos_label, pos_bx1 + 3, sy1 + 5, p_fg);
        new_hotzones.pos_btns.push(((pos_bx1, sy1 + 4, pos_bx2, sy1 + 18), true, idx));

        let wr_str = if let Some(ref p) = slot.profile {
            if p.cur_champ_id > 0 && p.cur_champ_games > 0 {
                format!("实战: {}%", p.cur_champ_win_rate_pct)
            } else if p.cur_champ_id > 0 && p.cur_champ_games == 0 {
                "试英雄(0场)".to_string()
            } else if p.sample_count > 0 {
                format!("近期: {}%", p.win_rate_pct)
            } else {
                format!("梯队: {:.1}%", slot.win_rate * 100.0)
            }
        } else {
            format!("梯队: {:.1}%", slot.win_rate * 100.0)
        };
        draw_text_line(hdc, &wr_str, left_slot_x + slot_card_w - 56, sy1 + 5, rgb(203, 213, 225));
        new_hotzones.inspect_slot_btns.push(((left_slot_x, sy1, left_slot_x + slot_card_w, sy1 + slot_card_h), true, idx));

        if let Some(m_wr) = slot.matchup_win_rate {
            let diff = m_wr - 0.5;
            let (m_col, m_text) = if diff >= 0.005 {
                (rgb(0, 230, 150), format!("⚔️ 对位胜率: {:.1}% (+{:.1}%)", m_wr * 100.0, diff * 100.0))
            } else if diff <= -0.005 {
                (rgb(239, 68, 68), format!("⚠️ 对位胜率: {:.1}% ({:.1}%)", m_wr * 100.0, diff * 100.0))
            } else {
                (rgb(200, 170, 110), format!("⚖️ 对位胜率: {:.1}% (50-50)", m_wr * 100.0))
            };
            draw_text_line(hdc, &m_text, left_slot_x + 5, sy1 + 24, m_col);
        } else {
            draw_text_line(hdc, "⏳ 等待对位锁定...", left_slot_x + 5, sy1 + 24, rgb(100, 116, 139));
        }
    }

    // 4.2 Their Team Slots
    let right_slot_x = actual_left_margin + (950.0 * sx) as i32;
    for (idx, slot) in data.their_team.iter().take(5).enumerate() {
        let sy1 = slot_start_y + idx as i32 * slot_pitch_y;
        let border_col = rgb(99, 23, 27);

        draw_card_rect(hdc, right_slot_x, sy1, slot_card_w, slot_card_h, rgb(26, 15, 20), border_col);

        let (t_fg, t_bg) = match slot.tier {
            1 => (rgb(0, 230, 150), rgb(4, 47, 26)),
            2 => (rgb(59, 130, 246), rgb(12, 43, 78)),
            3 => (rgb(200, 170, 110), rgb(58, 43, 22)),
            4 => (rgb(249, 115, 22), rgb(63, 27, 10)),
            _ => (rgb(239, 68, 68), rgb(69, 10, 10)),
        };
        draw_card_rect(hdc, right_slot_x + 4, sy1 + 4, 22, 14, t_bg, t_fg);
        let t_str = format!("T{}", slot.tier);
        draw_text_line(hdc, &t_str, right_slot_x + 6, sy1 + 5, t_fg);

        let pos_bx1 = right_slot_x + 30;
        let pos_bx2 = pos_bx1 + 42;
        let (p_bg, p_border, p_fg) = if slot.is_manual_pos {
            (rgb(39, 23, 4), rgb(245, 158, 11), rgb(251, 191, 36))
        } else {
            (rgb(26, 15, 20), rgb(244, 63, 94), rgb(254, 226, 226))
        };
        draw_card_rect(hdc, pos_bx1, sy1 + 4, 42, 14, p_bg, p_border);
        let pos_label = format!("{}▾", slot.position_cn);
        draw_text_line(hdc, &pos_label, pos_bx1 + 3, sy1 + 5, p_fg);
        new_hotzones.pos_btns.push(((pos_bx1, sy1 + 4, pos_bx2, sy1 + 18), false, idx));

        let wr_str = if let Some(ref p) = slot.profile {
            if p.cur_champ_id > 0 && p.cur_champ_games > 0 {
                format!("实战: {}%", p.cur_champ_win_rate_pct)
            } else if p.cur_champ_id > 0 && p.cur_champ_games == 0 {
                "试英雄(0场)".to_string()
            } else if p.sample_count > 0 {
                format!("近期: {}%", p.win_rate_pct)
            } else {
                format!("梯队: {:.1}%", slot.win_rate * 100.0)
            }
        } else {
            format!("梯队: {:.1}%", slot.win_rate * 100.0)
        };
        draw_text_line(hdc, &wr_str, right_slot_x + slot_card_w - 56, sy1 + 5, rgb(203, 213, 225));
        new_hotzones.inspect_slot_btns.push(((right_slot_x, sy1, right_slot_x + slot_card_w, sy1 + slot_card_h), false, idx));

        if let Some(m_wr) = slot.matchup_win_rate {
            let enemy_wr = 1.0 - m_wr;
            let diff = enemy_wr - 0.5;
            let (m_col, m_text) = if diff >= 0.005 {
                (rgb(239, 68, 68), format!("⚠️ 敌方胜率: {:.1}% (+{:.1}%)", enemy_wr * 100.0, diff * 100.0))
            } else if diff <= -0.005 {
                (rgb(0, 230, 150), format!("⚔️ 敌方胜率: {:.1}% ({:.1}%)", enemy_wr * 100.0, diff * 100.0))
            } else {
                (rgb(200, 170, 110), format!("⚖️ 敌方胜率: {:.1}% (50-50)", enemy_wr * 100.0))
            };
            draw_text_line(hdc, &m_text, right_slot_x + 5, sy1 + 24, m_col);
        } else {
            draw_text_line(hdc, "⏳ 等待对位锁定...", right_slot_x + 5, sy1 + 24, rgb(100, 116, 139));
        }
    }

    // -------------------------------------------------------------
    // 5. Hero Floating Inspector Overlay (Counter & Synergy)
    // STRICTLY user-triggered: NEVER auto-popup unprompted
    // -------------------------------------------------------------
    let inspect_target = inter.inspect_slot;

    if let Some((is_my_team, slot_index)) = inspect_target {
        let slot_opt = if is_my_team {
            data.my_team.get(slot_index)
        } else {
            data.their_team.get(slot_index)
        };

        if let Some(slot) = slot_opt {
            let f_w = (275.0 * sx) as i32;
            let f_h = (295.0 * sy) as i32;
            let f_x = if is_my_team {
                actual_left_margin + (85.0 * sx) as i32
            } else {
                actual_left_margin + cw - (85.0 * sx) as i32 - f_w
            };

            let base_y = actual_offset_y + ((100.0 + slot_index as f64 * 92.0 - 15.0) * sy) as i32;
            let f_y = base_y.clamp(actual_offset_y + (42.0 * sy) as i32, (actual_offset_y + ch) - f_h - 10);

            // Card container
            let border_col = if is_my_team { rgb(14, 165, 233) } else { rgb(244, 63, 94) };
            draw_card_rect(hdc, f_x, f_y, f_w, f_h, rgb(10, 16, 28), border_col);
            draw_h_line(hdc, f_x + 4, f_y + 1, f_x + f_w - 4, if is_my_team { rgb(56, 189, 248) } else { rgb(251, 113, 133) });

            // Header Title
            let hero_name = if slot.champion_name.is_empty() {
                if is_my_team { format!("我方{}楼", slot_index + 1) } else { format!("敌方{}楼", slot_index + 1) }
            } else {
                slot.champion_name.clone()
            };
            let title = format!("[{}] {} · 战术情报", slot.position_cn, hero_name);
            draw_text_line(hdc, &title, f_x + 10, f_y + 6, rgb(255, 255, 255));

            // Close button
            let close_w = 20;
            let close_h = 16;
            let close_x = f_x + f_w - close_w - 8;
            let close_y = f_y + 6;
            draw_card_rect(hdc, close_x, close_y, close_w, close_h, rgb(30, 41, 59), rgb(100, 116, 139));
            draw_text_line(hdc, "✕", close_x + 5, close_y + 1, rgb(226, 232, 240));
            new_hotzones.inspect_close_btn = Some((close_x, close_y, close_x + close_w, close_y + close_h));

            // Toolbar: Tabs & Sort
            let ty = f_y + 28;
            let is_cnt = inter.inspect_tab == "counter";
            let tab1_w = (72.0 * sx) as i32;
            let tab1_x = f_x + 8;
            draw_card_rect(hdc, tab1_x, ty, tab1_w, 20, if is_cnt { rgb(15, 76, 129) } else { rgb(15, 23, 42) }, if is_cnt { rgb(56, 189, 248) } else { rgb(51, 65, 85) });
            draw_text_line(hdc, "🎯 克制", tab1_x + 10, ty + 3, if is_cnt { rgb(255, 255, 255) } else { rgb(148, 163, 184) });
            new_hotzones.inspect_tab_btns.push(((tab1_x, ty, tab1_x + tab1_w, ty + 20), "counter".to_string()));

            let is_syn = inter.inspect_tab == "synergy";
            let tab2_w = (72.0 * sx) as i32;
            let tab2_x = tab1_x + tab1_w + 6;
            draw_card_rect(hdc, tab2_x, ty, tab2_w, 20, if is_syn { rgb(15, 76, 129) } else { rgb(15, 23, 42) }, if is_syn { rgb(56, 189, 248) } else { rgb(51, 65, 85) });
            draw_text_line(hdc, "🤝 协同", tab2_x + 10, ty + 3, if is_syn { rgb(255, 255, 255) } else { rgb(148, 163, 184) });
            new_hotzones.inspect_tab_btns.push(((tab2_x, ty, tab2_x + tab2_w, ty + 20), "synergy".to_string()));

            let sort_w = (82.0 * sx) as i32;
            let sort_x = f_x + f_w - sort_w - 8;
            let sort_label = if inter.sort_ascending { "胜率 ▲升序" } else { "胜率 ▼降序" };
            draw_card_rect(hdc, sort_x, ty, sort_w, 20, rgb(39, 23, 4), rgb(245, 158, 11));
            draw_text_line(hdc, sort_label, sort_x + 6, ty + 3, rgb(251, 191, 36));
            new_hotzones.inspect_sort_btn = Some((sort_x, ty, sort_x + sort_w, ty + 20));

            // Column Header
            let hy = f_y + 52;
            draw_h_line(hdc, f_x + 6, hy - 1, f_x + f_w - 6, rgb(30, 41, 59));
            draw_text_line(hdc, "英雄 / 评级", f_x + 10, hy + 2, rgb(100, 116, 139));
            let wr_col_name = if inter.inspect_tab == "synergy" { "协同胜率" } else { "克制胜率" };
            draw_text_line(hdc, wr_col_name, f_x + 120, hy + 2, rgb(100, 116, 139));
            draw_text_line(hdc, "场次", f_x + 215, hy + 2, rgb(100, 116, 139));

            // Data Items
            let mut items = if inter.inspect_tab == "synergy" {
                slot.synergy_heroes.clone()
            } else {
                slot.counter_heroes.clone()
            };

            if inter.sort_ascending {
                items.sort_by(|a, b| a.counter_win_rate.partial_cmp(&b.counter_win_rate).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                items.sort_by(|a, b| b.counter_win_rate.partial_cmp(&a.counter_win_rate).unwrap_or(std::cmp::Ordering::Equal));
            }

            let page_size = 6;
            let total_pages = if items.is_empty() { 1 } else { (items.len() + page_size - 1) / page_size };
            let page_idx = (inter.inspect_scroll_offset / page_size).min(total_pages - 1);
            let start_idx = page_idx * page_size;
            let visible_items: Vec<_> = items.iter().skip(start_idx).take(page_size).collect();

            if visible_items.is_empty() {
                draw_text_line(hdc, "⏳ 暂无 OP.GG 战术数据", f_x + 20, f_y + 110, rgb(148, 163, 184));
                draw_text_line(hdc, "(等待选定英雄或数据拉取中)", f_x + 20, f_y + 130, rgb(100, 116, 139));
            } else {
                for (r_i, item) in visible_items.iter().enumerate() {
                    let ry = f_y + 70 + r_i as i32 * 32;
                    let row_bg = if r_i % 2 == 0 { rgb(15, 23, 42) } else { rgb(20, 30, 52) };
                    draw_card_rect(hdc, f_x + 6, ry, f_w - 12, 28, row_bg, rgb(30, 41, 59));

                    let (t_fg, t_bg) = match item.tier {
                        1 => (rgb(0, 230, 150), rgb(4, 47, 26)),
                        2 => (rgb(59, 130, 246), rgb(12, 43, 78)),
                        3 => (rgb(200, 170, 110), rgb(58, 43, 22)),
                        4 => (rgb(249, 115, 22), rgb(63, 27, 10)),
                        _ => (rgb(239, 68, 68), rgb(69, 10, 10)),
                    };
                    if item.tier > 0 {
                        draw_card_rect(hdc, f_x + 10, ry + 5, 20, 16, t_bg, t_fg);
                        draw_text_line(hdc, &format!("T{}", item.tier), f_x + 12, ry + 6, t_fg);
                    }

                    draw_text_line(hdc, &item.name, f_x + 36, ry + 6, rgb(241, 245, 249));

                    let (wr_str, wr_fg) = if item.counter_win_rate > 0.0 {
                        let wr_pct = item.counter_win_rate * 100.0;
                        let wr_diff = wr_pct - 50.0;
                        let wr_fg = if wr_diff >= 2.0 {
                            rgb(0, 230, 150)
                        } else if wr_diff <= -2.0 {
                            rgb(239, 68, 68)
                        } else {
                            rgb(251, 191, 36)
                        };
                        (format!("{:.1}%", wr_pct), wr_fg)
                    } else {
                        ("- -".to_string(), rgb(148, 163, 184))
                    };
                    draw_text_line(hdc, &wr_str, f_x + 120, ry + 6, wr_fg);

                    let g_str = if item.games > 0 {
                        if item.games >= 1000 {
                            format!("{:.1}k", item.games as f64 / 1000.0)
                        } else {
                            format!("{}", item.games)
                        }
                    } else {
                        "-".to_string()
                    };
                    draw_text_line(hdc, &g_str, f_x + 215, ry + 6, rgb(148, 163, 184));
                }
            }

            // Footer
            let py = f_y + f_h - 26;
            draw_h_line(hdc, f_x + 6, py - 2, f_x + f_w - 6, rgb(30, 41, 59));
            if items.len() > page_size {
                let can_prev = page_idx > 0;
                let prev_w = 42;
                let prev_x = f_x + 10;
                let (prev_bg, prev_fg) = if can_prev { (rgb(20, 30, 50), rgb(203, 213, 225)) } else { (rgb(11, 19, 32), rgb(51, 65, 85)) };
                draw_card_rect(hdc, prev_x, py, prev_w, 20, prev_bg, rgb(51, 65, 85));
                draw_text_line(hdc, "◀ 上页", prev_x + 4, py + 3, prev_fg);
                if can_prev {
                    new_hotzones.inspect_nav_btns.push(((prev_x, py, prev_x + prev_w, py + 20), -(page_size as i32)));
                }

                let page_info = format!("{}/{}页 (共{}位)", page_idx + 1, total_pages, items.len());
                draw_text_line(hdc, &page_info, f_x + 68, py + 3, rgb(203, 213, 225));

                let can_next = page_idx + 1 < total_pages;
                let next_w = 42;
                let next_x = f_x + f_w - next_w - 10;
                let (next_bg, next_fg) = if can_next { (rgb(20, 30, 50), rgb(203, 213, 225)) } else { (rgb(11, 19, 32), rgb(51, 65, 85)) };
                draw_card_rect(hdc, next_x, py, next_w, 20, next_bg, rgb(51, 65, 85));
                draw_text_line(hdc, "下页 ▶", next_x + 4, py + 3, next_fg);
                if can_next {
                    new_hotzones.inspect_nav_btns.push(((next_x, py, next_x + next_w, py + 20), page_size as i32));
                }
            } else {
                draw_text_line(hdc, "💡 点击头像/卡片可切换查看英雄", f_x + 20, py + 3, rgb(100, 116, 139));
            }
        }
    }

    if let Ok(mut hz) = GLOBAL_HOTZONES.write() {
        *hz = new_hotzones;
    }
}

// -------------------------------------------------------------
// GDI Helper Primitives
// -------------------------------------------------------------

#[cfg(windows)]
unsafe fn draw_dot(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    cx: i32,
    cy: i32,
    r: i32,
    fill_color: windows::Win32::Foundation::COLORREF,
    border_color: windows::Win32::Foundation::COLORREF,
) {
    use windows::Win32::Graphics::Gdi::{CreatePen, CreateSolidBrush, DeleteObject, Ellipse, SelectObject, PS_SOLID};
    let brush = CreateSolidBrush(fill_color);
    let pen = CreatePen(PS_SOLID, 1, border_color);
    let old_brush = SelectObject(hdc, brush);
    let old_pen = SelectObject(hdc, pen);

    let _ = Ellipse(hdc, cx - r, cy - r, cx + r, cy + r);

    let _ = SelectObject(hdc, old_brush);
    let _ = SelectObject(hdc, old_pen);
    let _ = DeleteObject(brush);
    let _ = DeleteObject(pen);
}

#[cfg(windows)]
unsafe fn draw_h_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x1: i32,
    y: i32,
    x2: i32,
    color: windows::Win32::Foundation::COLORREF,
) {
    use windows::Win32::Graphics::Gdi::{CreatePen, DeleteObject, LineTo, MoveToEx, SelectObject, PS_SOLID};
    let pen = CreatePen(PS_SOLID, 1, color);
    let old_pen = SelectObject(hdc, pen);
    let _ = MoveToEx(hdc, x1, y, None);
    let _ = LineTo(hdc, x2, y);
    let _ = SelectObject(hdc, old_pen);
    let _ = DeleteObject(pen);
}

#[cfg(windows)]
unsafe fn draw_card_rect(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bg_color: windows::Win32::Foundation::COLORREF,
    border_color: windows::Win32::Foundation::COLORREF,
) {
    use windows::Win32::Graphics::Gdi::{CreatePen, CreateSolidBrush, DeleteObject, RoundRect, SelectObject, PS_SOLID};

    let pen = CreatePen(PS_SOLID, 1, border_color);
    let brush = CreateSolidBrush(bg_color);
    let old_pen = SelectObject(hdc, pen);
    let old_brush = SelectObject(hdc, brush);

    let _ = RoundRect(hdc, x, y, x + w, y + h, 8, 8);

    let _ = SelectObject(hdc, old_brush);
    let _ = SelectObject(hdc, old_pen);
    let _ = DeleteObject(brush);
    let _ = DeleteObject(pen);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    pub kind: FontKind,
}

pub fn is_emoji_char(c: char) -> bool {
    let u = c as u32;
    // Plane 1 emojis and pictographs: U+1F000..=U+1FAFF
    (0x1F000..=0x1FAFF).contains(&u)
    // Miscellaneous symbols and arrows: U+2600..=U+27BF (e.g. ⚔, ⚙, ⚡, ⚠, ⏳, etc.)
    || (0x2600..=0x27BF).contains(&u)
    // Dingbats, Miscellaneous Technical: U+2300..=U+23FF
    || (0x2300..=0x23FF).contains(&u)
    // Supplemental arrows and symbols: U+2B00..=U+2BFF
    || (0x2B00..=0x2BFF).contains(&u)
}

pub fn sanitize_for_gdi(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            let u = c as u32;
            // Remove Variation Selectors (U+FE00..=U+FE0F) and Zero Width Joiners (U+200B..=U+200D)
            // which produce black box .notdef glyphs in GDI TextOutW
            !(0xFE00..=0xFE0F).contains(&u) && !(0x200B..=0x200D).contains(&u)
        })
        .collect()
}

pub fn split_text_runs(raw_text: &str) -> Vec<TextRun> {
    let sanitized = sanitize_for_gdi(raw_text);
    if sanitized.is_empty() {
        return Vec::new();
    }

    let mut runs: Vec<TextRun> = Vec::new();
    let mut current_text = String::new();
    let mut current_kind = None;

    for c in sanitized.chars() {
        let kind = if is_emoji_char(c) {
            FontKind::Emoji
        } else {
            FontKind::Text
        };

        match current_kind {
            Some(k) if k == kind => {
                current_text.push(c);
            }
            Some(k) => {
                runs.push(TextRun {
                    text: current_text,
                    kind: k,
                });
                current_text = c.to_string();
                current_kind = Some(kind);
            }
            None => {
                current_text.push(c);
                current_kind = Some(kind);
            }
        }
    }

    if let Some(k) = current_kind {
        if !current_text.is_empty() {
            runs.push(TextRun {
                text: current_text,
                kind: k,
            });
        }
    }

    runs
}

#[cfg(windows)]
unsafe fn measure_text_custom(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    font_size: i32,
    weight: i32,
) -> (i32, i32) {
    use windows::Win32::Foundation::SIZE;
    use windows::Win32::Graphics::Gdi::{GetTextExtentPoint32W, SelectObject};

    let runs = split_text_runs(text);
    let mut total_w = 0;
    let mut max_h = 0;

    for run in &runs {
        let hfont = get_cached_font_kind(font_size, weight, run.kind);
        let old_font = SelectObject(hdc, hfont);
        let wide: Vec<u16> = run.text.encode_utf16().collect();
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &wide, &mut sz);
        total_w += sz.cx;
        if sz.cy > max_h {
            max_h = sz.cy;
        }
        let _ = SelectObject(hdc, old_font);
    }

    (total_w, max_h)
}

#[cfg(windows)]
unsafe fn measure_text_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
) -> (i32, i32) {
    measure_text_custom(hdc, text, 12, 400)
}

#[cfg(windows)]
unsafe fn draw_text_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    x: i32,
    y: i32,
    color: windows::Win32::Foundation::COLORREF,
) {
    draw_text_custom(hdc, text, x, y, 12, 400, color, false);
}

#[cfg(windows)]
unsafe fn draw_pill_badge(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    bg_color: windows::Win32::Foundation::COLORREF,
    border_color: windows::Win32::Foundation::COLORREF,
    text_color: windows::Win32::Foundation::COLORREF,
    text: &str,
) {
    use windows::Win32::Graphics::Gdi::{
        CreatePen, CreateSolidBrush, DeleteObject, RoundRect, SelectObject, PS_SOLID,
    };

    let pen = CreatePen(PS_SOLID, 1, border_color);
    let brush = CreateSolidBrush(bg_color);
    let old_pen = SelectObject(hdc, pen);
    let old_brush = SelectObject(hdc, brush);

    let _ = RoundRect(hdc, x, y, x + w, y + h, 6, 6);

    let _ = SelectObject(hdc, old_brush);
    let _ = SelectObject(hdc, old_pen);
    let _ = DeleteObject(brush);
    let _ = DeleteObject(pen);

    let (total_w, text_h) = measure_text_custom(hdc, text, 11, 600);
    let text_x = x + (w - total_w) / 2;
    let text_y = y + (h - text_h) / 2;
    draw_text_custom(hdc, text, text_x, text_y, 11, 600, text_color, false);
}

#[cfg(windows)]
unsafe fn draw_text_custom(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    x: i32,
    y: i32,
    font_size: i32,
    weight: i32,
    color: windows::Win32::Foundation::COLORREF,
    shadow: bool,
) {
    use windows::Win32::Graphics::Gdi::{
        SelectObject, SetBkMode, SetTextColor, TextOutW, GetTextExtentPoint32W, TRANSPARENT,
    };
    use windows::Win32::Foundation::SIZE;

    let runs = split_text_runs(text);
    if runs.is_empty() {
        return;
    }

    let _ = SetBkMode(hdc, TRANSPARENT);

    // Draw shadow if requested
    if shadow {
        SetTextColor(hdc, windows::Win32::Foundation::COLORREF(0x00000000));
        let mut cur_x = x + 1;
        for run in &runs {
            let hfont = get_cached_font_kind(font_size, weight, run.kind);
            let old_font = SelectObject(hdc, hfont);
            let wide: Vec<u16> = run.text.encode_utf16().collect();
            let _ = TextOutW(hdc, cur_x, y + 1, &wide);
            let mut sz = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &wide, &mut sz);
            cur_x += sz.cx;
            let _ = SelectObject(hdc, old_font);
        }
    }

    // Draw foreground text
    SetTextColor(hdc, color);
    let mut cur_x = x;
    for run in &runs {
        let hfont = get_cached_font_kind(font_size, weight, run.kind);
        let old_font = SelectObject(hdc, hfont);
        let wide: Vec<u16> = run.text.encode_utf16().collect();
        let _ = TextOutW(hdc, cur_x, y, &wide);
        let mut sz = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &wide, &mut sz);
        cur_x += sz.cx;
        let _ = SelectObject(hdc, old_font);
    }
}

#[cfg(windows)]
unsafe fn draw_text_centered(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    cx: i32,
    y: i32,
    font_size: i32,
    weight: i32,
    color: windows::Win32::Foundation::COLORREF,
    shadow: bool,
) {
    let (total_w, _) = measure_text_custom(hdc, text, font_size, weight);
    let start_x = cx - total_w / 2;
    draw_text_custom(hdc, text, start_x, y, font_size, weight, color, shadow);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_font_cache() {
        #[cfg(windows)]
        {
            unsafe {
                let _f1 = get_cached_font(12, 400);
                let _f2 = get_cached_font(14, 700);
                let _f3 = get_cached_font_kind(12, 400, FontKind::Emoji);
            }
            assert!(!FONT_CACHE.lock().unwrap().is_empty());
            cleanup_font_cache();
            assert!(FONT_CACHE.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn test_split_text_runs() {
        let runs = split_text_runs("🛡️ 我方战绩");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].kind, FontKind::Emoji);
        assert_eq!(runs[0].text, "🛡");
        assert_eq!(runs[1].kind, FontKind::Text);
        assert_eq!(runs[1].text, " 我方战绩");
    }
}

