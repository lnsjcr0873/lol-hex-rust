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
pub struct ItemOverlayData {
    pub hero_name: String,
    pub current_gold: f64,
    pub items: Vec<ItemCardView>,
}

#[derive(Debug, Clone)]
pub struct BenchChampView {
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
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            content: HudContent::None,
            expires_at: None,
            is_visible: false,
            is_test_mode: false,
        }
    }
}

pub struct NativeHud {
    pub running: Arc<AtomicBool>,
    pub state: Arc<std::sync::RwLock<HudState>>,
    #[cfg(windows)]
    pub hwnd: Arc<std::sync::Mutex<Option<isize>>>,
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_HUD: Arc<NativeHud> = Arc::new(NativeHud::new());
    pub static ref RANKED_INTERACTION: std::sync::RwLock<RankedInteractionState> = std::sync::RwLock::new(RankedInteractionState::default());
    pub static ref GLOBAL_HOTZONES: std::sync::RwLock<HudHotzones> = std::sync::RwLock::new(HudHotzones::default());
    pub static ref GLOBAL_LCU: std::sync::RwLock<Option<Arc<crate::lcu::LcuClient>>> = std::sync::RwLock::new(None);
    pub static ref GLOBAL_HISTORY: std::sync::RwLock<Option<Arc<crate::engine::PlayerHistoryManager>>> = std::sync::RwLock::new(None);
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

    tokio::spawn(async move {
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
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.sample_match_count = *count;
                if let Ok(gh) = GLOBAL_HISTORY.try_read() {
                    if let Some(history_mgr) = gh.as_ref() {
                        history_mgr.sample_count.store(*count, Ordering::Relaxed);
                    }
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

    // 10. Hero Inspection slot buttons (click slot to inspect that hero)
    for ((x1, y1, x2, y2), is_my_team, slot_index) in &hotzones.inspect_slot_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            if let Ok(mut inter) = RANKED_INTERACTION.write() {
                inter.inspect_closed = false;
                inter.inspect_slot = Some((*is_my_team, *slot_index));
                inter.inspect_scroll_offset = 0;
                repaint_needed = true;
            }
            break;
        }
    }

    // 11. Hero Inspection nav buttons
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

    // 12. Pos buttons (left click cycles as well)
    for ((x1, y1, x2, y2), is_my_team, slot_index) in &hotzones.pos_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            cycle_slot_position(*is_my_team, *slot_index);
            repaint_needed = true;
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

fn handle_rbutton_down(hwnd: windows::Win32::Foundation::HWND, px: i32, py: i32) {
    let hotzones = if let Ok(guard) = GLOBAL_HOTZONES.read() {
        guard.clone()
    } else {
        return;
    };

    let mut repaint_needed = false;
    for ((x1, y1, x2, y2), is_my_team, slot_index) in &hotzones.pos_btns {
        if px >= *x1 && px <= *x2 && py >= *y1 && py <= *y2 {
            cycle_slot_position(*is_my_team, *slot_index);
            repaint_needed = true;
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
        }
    }

    pub fn show_augments(&self, data: AugmentOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::Augment(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
        }
        self.request_repaint();
    }

    pub fn show_items(&self, data: ItemOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::Item(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
        }
        self.request_repaint();
    }

    pub fn show_bench(&self, data: BenchOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::Bench(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
        }
        self.request_repaint();
    }

    pub fn show_ranked(&self, data: RankedOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::Ranked(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.is_test_mode = false;
        }
        self.request_repaint();
    }

    pub fn show_ranked_test(&self, data: RankedOverlayData, duration: Duration) {
        self.start();
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::Ranked(data);
            st.expires_at = Some(Instant::now() + duration);
            st.is_visible = true;
            st.is_test_mode = true;
        }
        self.request_repaint();
    }

    pub fn hide(&self) {
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::None;
            st.expires_at = None;
            st.is_visible = false;
            st.is_test_mode = false;
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
        WM_LBUTTONDOWN => {
            let px = (lparam.0 & 0xffff) as i16 as i32;
            let py = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
            handle_lbutton_down(hwnd, px, py);
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let px = (lparam.0 & 0xffff) as i16 as i32;
            let py = ((lparam.0 >> 16) & 0xffff) as i16 as i32;
            handle_rbutton_down(hwnd, px, py);
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
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }

    let mut target_rect = match &content_type {
        HudContent::Ranked(_) | HudContent::Bench(_) => {
            crate::vision::ScreenCapture::get_lol_client_rect()
        }
        HudContent::Augment(_) | HudContent::Item(_) => {
            crate::vision::ScreenCapture::get_lol_game_rect()
                .or_else(crate::vision::ScreenCapture::get_lol_client_rect)
        }
        HudContent::None => None,
    };

    if target_rect.is_none() && is_test {
        // Fallback for test mode: Center 1280x720 on screen (identical to Python test behavior)
        let vp = crate::vision::ScreenCapture::get_effective_viewport();
        let cw = 1280.min(vp.width);
        let ch = 720.min(vp.height);
        target_rect = Some(crate::vision::ViewportRect {
            x: (vp.width - cw) / 2,
            y: (vp.height - ch) / 2,
            width: cw,
            height: ch,
        });
    }

    let is_league_active = crate::overlay::WindowTracker::is_league_active();
    let should_hide = match &content_type {
        HudContent::None => true,
        HudContent::Augment(_) | HudContent::Item(_) => {
            target_rect.is_none() || !is_league_active
        }
        HudContent::Ranked(_) | HudContent::Bench(_) => {
            target_rect.is_none()
        }
    };

    if should_hide {
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }

    // Align with active League window position (with outer docking for Ranked)
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

            let screen_w = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
            );

            let actual_offset_y = if rect.y >= top_dock_height { top_dock_height } else { rect.y.max(0) };
            let actual_left_margin = if rect.x >= left_w { left_w } else { 0 };
            let right_avail = screen_w - (rect.x + rect.width);
            let actual_right_margin = if right_avail >= right_w { right_w } else { 0 };

            let win_x = rect.x - actual_left_margin;
            let win_y = rect.y - actual_offset_y;
            let win_w = rect.width + actual_left_margin + actual_right_margin;
            let win_h = rect.height + actual_offset_y;

            use std::sync::Mutex;
            static LAST_WIN_GEOM: Mutex<(i32, i32, i32, i32)> = Mutex::new((-1, -1, -1, -1));
            let target_geom = (win_x, win_y, win_w, win_h);
            let mut geom_changed = false;
            if let Ok(mut g) = LAST_WIN_GEOM.lock() {
                if *g != target_geom {
                    *g = target_geom;
                    geom_changed = true;
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
            use std::sync::Mutex;
            static LAST_CLIENT_GEOM: Mutex<(i32, i32, i32, i32)> = Mutex::new((-1, -1, -1, -1));
            let target_geom = (rect.x, rect.y, rect.width, rect.height);
            let mut geom_changed = false;
            if let Ok(mut g) = LAST_CLIENT_GEOM.lock() {
                if *g != target_geom {
                    *g = target_geom;
                    geom_changed = true;
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
        let is_interactive = matches!(&content_type, HudContent::Ranked(_));
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

    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    let _ = InvalidateRect(hwnd, None, BOOL(0));
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
    use windows::Win32::Foundation::COLORREF;

    // 1. Header Banner
    let banner_w = 460.min(width - 40);
    let banner_h = 36;
    let banner_x = (width - banner_w) / 2;
    let banner_y = (height as f64 * 0.32) as i32;

    draw_card_rect(
        hdc,
        banner_x,
        banner_y,
        banner_w,
        banner_h,
        COLORREF(0x00281e0f), // Dark navy bg (0x0f, 0x1e, 0x28)
        COLORREF(0x003c9bc8), // Gold border (0xc8, 0x9b, 0x3c)
    );

    let title_text = format!("⚡ HexBrain 海克斯实时推演 - {}", if data.hero_name.is_empty() { "实时建议" } else { &data.hero_name });
    draw_text_line(hdc, &title_text, banner_x + 16, banner_y + 10, COLORREF(0x0045f0f0)); // Light gold/cyan text

    // 2. Augment Cards (Up to 3 cards corresponding to in-game slots)
    let card_w = 260.min((width - 60) / 3);
    let card_h = 135;
    let start_y = (height as f64 * 0.44) as i32;

    let total_cards = data.cards.len();
    for (i, card) in data.cards.iter().take(3).enumerate() {
        let slot_x = match i {
            0 => (width as f64 * 0.22) as i32,
            1 => (width as f64 * 0.43) as i32,
            _ => (width as f64 * 0.64) as i32,
        };

        // Determine border and badge colors based on tier
        let (border_col, badge_col, badge_text) = match card.tier.as_str() {
            "S" => (COLORREF(0x0000d7ff), COLORREF(0x0000d7ff), "[ S 级核心 ]"), // Gold (0xff, 0xd7, 0x00)
            "A" => (COLORREF(0x00ffe500), COLORREF(0x00ffe500), "[ A 级优选 ]"), // Cyan (0x00, 0xe5, 0xff)
            "B" => (COLORREF(0x00c0c0c0), COLORREF(0x00c0c0c0), "[ B 级过渡 ]"), // Silver (0xc0, 0xc0, 0xc0)
            _ => (COLORREF(0x005546ff), COLORREF(0x005546ff), "[ ⚠️ 陷阱慎选 ]"), // Red (0xff, 0x46, 0x55)
        };

        // Background box
        draw_card_rect(hdc, slot_x, start_y, card_w, card_h, COLORREF(0x0023190f), border_col);

        // Header line: [Tier Badge] + Augment Name
        draw_text_line(hdc, badge_text, slot_x + 12, start_y + 12, badge_col);
        draw_text_line(hdc, &card.name, slot_x + 105, start_y + 12, COLORREF(0x00ffffff));

        // Line 2: WinRate + Delta
        let wr_text = format!("胜率: {:.1}% (Δ {:+.1}%)", card.win_rate * 100.0, card.delta_win_rate * 100.0);
        draw_text_line(hdc, &wr_text, slot_x + 12, start_y + 40, COLORREF(0x00d0d0d0));

        // Line 3: Wilson bound + Games count
        let wilson_text = format!("Wilson下限: {:.1}% ({}局)", card.wilson_lower * 100.0, card.games);
        draw_text_line(hdc, &wilson_text, slot_x + 12, start_y + 68, COLORREF(0x00a0a0a0));

        // Line 4: Tactical tip
        let tip_text = format!("建议: {}", card.recommendation);
        draw_text_line(hdc, &tip_text, slot_x + 12, start_y + 96, COLORREF(0x0050ff50));
    }

    if total_cards == 0 {
        let msg_w = 340;
        let msg_x = (width - msg_w) / 2;
        draw_card_rect(hdc, msg_x, start_y, msg_w, 70, COLORREF(0x0023190f), COLORREF(0x00a0a0a0));
        draw_text_line(hdc, "正在检测海克斯强化卡片...", msg_x + 20, start_y + 16, COLORREF(0x00ffffff));
        draw_text_line(hdc, "按 [ 键立即触发全局推演", msg_x + 20, start_y + 40, COLORREF(0x0000e5ff));
    }
}

#[cfg(windows)]
unsafe fn render_item_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    data: &ItemOverlayData,
) {
    use windows::Win32::Foundation::COLORREF;

    let box_w = 390.min(width - 40);
    let box_h = 240;
    let box_x = width - box_w - 30;
    let box_y = height - box_h - 70;

    // Dark slate border with cyan accent
    draw_card_rect(hdc, box_x, box_y, box_w, box_h, COLORREF(0x0020140a), COLORREF(0x00ab9703));

    // Title
    let title = format!("⚡ HexBrain 出装推荐 (持有金币: {}G)", data.current_gold as i64);
    draw_text_line(hdc, &title, box_x + 14, box_y + 12, COLORREF(0x0000e5ff));

    for (i, item) in data.items.iter().take(4).enumerate() {
        let item_y = box_y + 42 + (i as i32 * 46);

        // Item name & tactical score
        let header = format!("{}. {} (战术评分: {:.0})", i + 1, item.name, item.tactical_score);
        draw_text_line(hdc, &header, box_x + 14, item_y, COLORREF(0x00ffffff));

        // Affordability badge & net cost
        let (cost_text, cost_col) = if item.is_affordable {
            (
                format!("  [可直接合成] 合成价: {}G (总价: {}G)", item.net_cost, item.total_cost),
                COLORREF(0x0050ff50), // Green
            )
        } else {
            (
                format!("  [差 {}G] 合成价: {}G (总价: {}G)", item.gold_diff, item.net_cost, item.total_cost),
                COLORREF(0x0050c8ff), // Amber / Yellow
            )
        };

        draw_text_line(hdc, &cost_text, box_x + 14, item_y + 20, cost_col);
    }
}

#[cfg(windows)]
unsafe fn render_bench_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    _height: i32,
    data: &BenchOverlayData,
) {
    use windows::Win32::Foundation::COLORREF;

    let bar_w = 640.min(width - 40);
    let bar_h = 75;
    let bar_x = (width - bar_w) / 2;
    let bar_y = 25;

    draw_card_rect(hdc, bar_x, bar_y, bar_w, bar_h, COLORREF(0x0023190f), COLORREF(0x003c9bc8));
    draw_text_line(hdc, "⚡ 备战席英雄实时胜率推演 (按需点击交换)", bar_x + 16, bar_y + 10, COLORREF(0x0000d7ff));

    let mut offset_x = bar_x + 16;
    for hero in data.champions.iter().take(5) {
        let badge = format!("{}: {:.1}% ({})", hero.champion_name, hero.win_rate * 100.0, hero.tier);
        let col = if hero.tier == "S" {
            COLORREF(0x0000d7ff)
        } else {
            COLORREF(0x00ffffff)
        };
        draw_text_line(hdc, &badge, offset_x, bar_y + 38, col);
        offset_x += 120;
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
    let mut active_tab = inter.active_tab.clone();
    let _offset = inter.counter_scroll_offset;
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
                active_tab = "runes".to_string();
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
    // 1. Top Minimal Bar (Compact Rune Button & Sample Switcher)
    // -------------------------------------------------------------
    let top_y = if actual_offset_y >= 30 { (actual_offset_y - 28) / 2 } else { 2 };
    
    // Sample switcher on top right
    let sample_counts = [10, 20, 30, 50];
    let s_btn_w = (28.0 * sx) as i32;
    let s_btn_h = (18.0 * sy) as i32;
    let s_gap = (3.0 * sx) as i32;
    let total_s_w = sample_counts.len() as i32 * s_btn_w + (sample_counts.len() as i32 - 1) * s_gap;
    let s_group_right = actual_left_margin + cw - 12;
    let s_group_left = s_group_right - total_s_w;

    let label_x = s_group_left - (45.0 * sx) as i32;
    draw_text_line(hdc, "📊样本:", label_x, top_y + 3, rgb(148, 163, 184));

    for (idx, &sc) in sample_counts.iter().enumerate() {
        let sbx1 = s_group_left + idx as i32 * (s_btn_w + s_gap);
        let sby1 = top_y;
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
        draw_text_line(hdc, &s_text, sbx1 + 3, sby1 + 2, s_fg);
        new_hotzones.sample_btns.push(((sbx1, sby1, sbx2, sby2), sc));
    }

    // Rune import pill button at top center (if available)
    if let Some(ref rg) = data.decision_card.rune_guide {
        let btn_w = (230.0 * sx) as i32;
        let btn_h = (24.0 * sy) as i32;
        let bx1 = actual_left_margin + (cw - btn_w) / 2;
        let bx2 = bx1 + btn_w;
        let by1 = top_y;
        let by2 = by1 + btn_h;

        let (btn_bg, btn_border, btn_label) = if let Some(ref status) = btn_status {
            (rgb(6, 78, 59), rgb(52, 211, 153), status.as_str())
        } else {
            (rgb(2, 132, 199), rgb(56, 189, 248), "⚡ 一键导入 OP.GG 符文")
        };

        draw_card_rect(hdc, bx1, by1, btn_w, btn_h, btn_bg, btn_border);
        draw_text_line(hdc, btn_label, bx1 + 10, by1 + 4, rgb(255, 255, 255));
        new_hotzones.rune_btn = Some((bx1, by1, bx2, by2));
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

            let slot_name = if slot.champion_name.is_empty() { format!("我方{}楼", idx + 1) } else { slot.champion_name.clone() };
            let row1_left = format!("[{}] {}", slot.position_cn, slot_name);
            draw_text_line(hdc, &row1_left, wx1 + 6, cy1 + 5, rgb(255, 255, 255));

            let tier_str = if let Some(ref p) = slot.profile {
                format!("{} {}P", p.tier_str, p.lp)
            } else {
                format!("T{}级", slot.tier)
            };
            draw_text_line(hdc, &tier_str, wx2 - 58, cy1 + 5, rgb(200, 170, 110));

            if let Some(ref p) = slot.profile {
                let row2_text = format!("近{}场 {}%", sample_count, p.win_rate_pct);
                let wr_fg = if p.win_rate_pct >= 60 { rgb(52, 211, 153) } else if p.win_rate_pct >= 50 { rgb(56, 189, 248) } else { rgb(248, 113, 113) };
                draw_text_line(hdc, &row2_text, wx1 + 6, cy1 + 24, wr_fg);

                let dot_start_x = wx2 - 48;
                let dot_cy = cy1 + 29;
                for d_idx in 0..5 {
                    let dx = dot_start_x + d_idx * 9;
                    let (fill_c, border_c) = if p.streak_type == "win" && d_idx < p.streak_count.min(5) {
                        (rgb(16, 185, 129), rgb(52, 211, 153))
                    } else if p.streak_type == "loss" && d_idx < p.streak_count.min(5) {
                        (rgb(239, 68, 68), rgb(248, 113, 113))
                    } else if (d_idx % 2 == 0 && p.win_rate_pct >= 50) || p.win_rate_pct >= 65 {
                        (rgb(16, 185, 129), rgb(52, 211, 153))
                    } else {
                        (rgb(30, 41, 59), rgb(51, 65, 85))
                    };
                    draw_dot(hdc, dx, dot_cy, 3, fill_c, border_c);
                }
            } else {
                let row2_text = format!("排位胜率 {:.1}%", slot.win_rate * 100.0);
                draw_text_line(hdc, &row2_text, wx1 + 6, cy1 + 24, rgb(148, 163, 184));
            }

            if let Some(ref p) = slot.profile {
                if !p.badge_text.is_empty() {
                    let badge_w = 82.min(wx2 - wx1 - 12);
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
                }
            } else {
                draw_text_line(hdc, "🎯 点击查看对位/协同 ▶", wx1 + 6, cy1 + 46, rgb(56, 189, 248));
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

            let enemy_cname = if slot.champion_name.is_empty() { format!("敌方{}楼", idx + 1) } else { slot.champion_name.clone() };
            let row1_text = format!("[{}] {}", slot.position_cn, enemy_cname);
            draw_text_line(hdc, &row1_text, wx1 + 6, cy1 + 5, rgb(255, 255, 255));
            draw_text_line(hdc, "🔒 选人期匿名保护", wx1 + 6, cy1 + 25, rgb(148, 163, 184));
            draw_text_line(hdc, "🎯 点击查看克制/协同 ▶", wx1 + 6, cy1 + 46, rgb(244, 63, 94));
        }
    }

    // -------------------------------------------------------------
    // 4. 5v5 Slots (My Team Left X=205, Their Team Right X=950)
    // -------------------------------------------------------------
    let slot_pitch_y = (92.0 * sy) as i32;
    let slot_card_w = (130.0 * sx) as i32;
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

        let wr_str = format!("{:.1}%", slot.win_rate * 100.0);
        draw_text_line(hdc, &wr_str, left_slot_x + slot_card_w - 40, sy1 + 5, rgb(203, 213, 225));
        new_hotzones.inspect_slot_btns.push(((actual_left_margin, sy1, left_slot_x + slot_card_w, sy1 + slot_card_h), true, idx));

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

        let wr_str = format!("{:.1}%", slot.win_rate * 100.0);
        draw_text_line(hdc, &wr_str, right_slot_x + slot_card_w - 40, sy1 + 5, rgb(203, 213, 225));
        new_hotzones.inspect_slot_btns.push(((right_slot_x, sy1, actual_left_margin + cw, sy1 + slot_card_h), false, idx));

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
    // 5. Hero-Adjacent Floating Inspector Overlay (Counter & Synergy)
    // -------------------------------------------------------------
    let inspect_target = if let Some((is_my, slot_i)) = inter.inspect_slot {
        Some((is_my, slot_i))
    } else if !inter.inspect_closed {
        // Auto-inspect: prioritize enemy laner facing local player, then local player, then any picked champ
        let local_slot = data.my_team.iter().find(|s| s.is_local_player);
        let enemy_laner = local_slot.and_then(|ls| {
            data.their_team.iter().enumerate().find(|(_, ts)| ts.position == ls.position && ts.champion_id > 0)
        });
        if let Some((enemy_i, _)) = enemy_laner {
            Some((false, enemy_i))
        } else if let Some(local_i) = data.my_team.iter().position(|s| s.is_local_player && s.champion_id > 0) {
            Some((true, local_i))
        } else if let Some(enemy_i) = data.their_team.iter().position(|s| s.champion_id > 0) {
            Some((false, enemy_i))
        } else if let Some(ally_i) = data.my_team.iter().position(|s| s.champion_id > 0) {
            Some((true, ally_i))
        } else {
            data.my_team.iter().position(|s| s.is_local_player).map(|i| (true, i))
        }
    } else {
        None
    };

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
                    draw_card_rect(hdc, f_x + 10, ry + 5, 20, 16, t_bg, t_fg);
                    draw_text_line(hdc, &format!("T{}", item.tier), f_x + 12, ry + 6, t_fg);

                    draw_text_line(hdc, &item.name, f_x + 36, ry + 6, rgb(241, 245, 249));

                    let wr_pct = item.counter_win_rate * 100.0;
                    let wr_diff = wr_pct - 50.0;
                    let wr_fg = if wr_diff >= 2.0 {
                        rgb(0, 230, 150)
                    } else if wr_diff <= -2.0 {
                        rgb(239, 68, 68)
                    } else {
                        rgb(251, 191, 36)
                    };
                    let wr_str = format!("{:.1}%", wr_pct);
                    draw_text_line(hdc, &wr_str, f_x + 120, ry + 6, wr_fg);

                    let g_str = if item.games >= 1000 {
                        format!("{:.1}k", item.games as f64 / 1000.0)
                    } else {
                        format!("{}", item.games)
                    };
                    draw_text_line(hdc, &g_str, f_x + 215, ry + 6, rgb(148, 163, 184));
                }
            }

            // Footer
            let py = f_y + f_h - 26;
            draw_h_line(hdc, f_x + 6, py - 2, f_x + f_w - 6, rgb(30, 41, 59));
            if items.len() > page_size {
                let prev_w = 42;
                let prev_x = f_x + 10;
                draw_card_rect(hdc, prev_x, py, prev_w, 20, rgb(20, 30, 50), rgb(51, 65, 85));
                draw_text_line(hdc, "◀ 上页", prev_x + 4, py + 3, rgb(148, 163, 184));
                new_hotzones.inspect_nav_btns.push(((prev_x, py, prev_x + prev_w, py + 20), -(page_size as i32)));

                let page_info = format!("{}/{}页 (共{}位)", page_idx + 1, total_pages, items.len());
                draw_text_line(hdc, &page_info, f_x + 68, py + 3, rgb(203, 213, 225));

                let next_w = 42;
                let next_x = f_x + f_w - next_w - 10;
                draw_card_rect(hdc, next_x, py, next_w, 20, rgb(20, 30, 50), rgb(51, 65, 85));
                draw_text_line(hdc, "下页 ▶", next_x + 4, py + 3, rgb(148, 163, 184));
                new_hotzones.inspect_nav_btns.push(((next_x, py, next_x + next_w, py + 20), page_size as i32));
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
    use windows::Win32::Graphics::Gdi::{CreateSolidBrush, DeleteObject, RoundRect, SelectObject};

    let border_brush = CreateSolidBrush(border_color);
    let old_brush = SelectObject(hdc, border_brush);
    let _ = RoundRect(hdc, x, y, x + w, y + h, 8, 8);

    let bg_brush = CreateSolidBrush(bg_color);
    let _ = SelectObject(hdc, bg_brush);
    let _ = RoundRect(hdc, x + 1, y + 1, x + w - 1, y + h - 1, 6, 6);

    let _ = SelectObject(hdc, old_brush);
    let _ = DeleteObject(border_brush);
    let _ = DeleteObject(bg_brush);
}

#[cfg(windows)]
unsafe fn draw_text_line(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    x: i32,
    y: i32,
    color: windows::Win32::Foundation::COLORREF,
) {
    use windows::Win32::Graphics::Gdi::{SetBkMode, SetTextColor, TextOutW, BKMODE};
    let _ = SetBkMode(hdc, BKMODE(1));
    SetTextColor(hdc, color);
    let wide: Vec<u16> = text.encode_utf16().collect();
    let _ = TextOutW(hdc, x, y, &wide);
}
