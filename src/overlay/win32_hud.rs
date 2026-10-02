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

#[derive(Debug, Clone)]
pub struct RankedSlotView {
    pub position_cn: String,
    pub champion_name: String,
    pub win_rate: f64,
    pub tier: i32,
    pub is_local_player: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RankedOverlayData {
    pub my_team: Vec<RankedSlotView>,
    pub their_team: Vec<RankedSlotView>,
    pub recommendation: String,
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
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            content: HudContent::None,
            expires_at: None,
            is_visible: false,
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
        }
        self.request_repaint();
    }

    pub fn hide(&self) {
        {
            let mut st = self.state.write().unwrap();
            st.content = HudContent::None;
            st.expires_at = None;
            st.is_visible = false;
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
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
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
            | WS_EX_TRANSPARENT
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

        // 80ms timer for tracking active game window & auto-hiding
        let _ = SetTimer(hwnd, 1, 80, None);

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
        WM_ERASEBKGND, WM_PAINT, WM_TIMER,
    };

    match msg {
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
                let mem_bmp = CreateCompatibleBitmap(hdc, width, height);
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
                let _ = DeleteDC(mem_dc);
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

    let lol_rect = crate::vision::ScreenCapture::get_lol_window_rect();
    let is_league_active = crate::overlay::WindowTracker::is_league_active();

    let mut should_hide = lol_rect.is_none();
    let mut is_expired = false;

    if let Ok(mut st) = GLOBAL_HUD.state.write() {
        if let Some(exp) = st.expires_at {
            if Instant::now() >= exp {
                st.is_visible = false;
                st.content = HudContent::None;
                st.expires_at = None;
                is_expired = true;
            }
        }
        if !st.is_visible {
            should_hide = true;
        }

        match &st.content {
            HudContent::Augment(_) | HudContent::Item(_) => {
                if !is_league_active {
                    should_hide = true;
                }
            }
            HudContent::Ranked(_) | HudContent::Bench(_) => {
                // In ChampSelect: stay docked onto League Client as long as client is visible on screen!
            }
            HudContent::None => {
                should_hide = true;
            }
        }
    }

    if should_hide || is_expired {
        let _ = ShowWindow(hwnd, SW_HIDE);
        return;
    }

    // Align with active League window position
    if let Some(rect) = lol_rect {
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

    SetBkMode(hdc, TRANSPARENT);
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
unsafe fn render_ranked_hud(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    width: i32,
    height: i32,
    data: &RankedOverlayData,
) {
    use windows::Win32::Foundation::COLORREF;

    let scale_x = width as f64 / 1280.0;
    let scale_y = height as f64 / 720.0;

    // 1. Top Banner (Centered at top)
    let banner_w = ((640.0 * scale_x) as i32).min(width - 40);
    let banner_h = (36.0 * scale_y) as i32;
    let banner_x = (width - banner_w) / 2;
    let banner_y = (15.0 * scale_y) as i32;

    draw_card_rect(
        hdc,
        banner_x,
        banner_y,
        banner_w,
        banner_h,
        COLORREF(0x00281e0f), // Dark navy bg
        COLORREF(0x003c9bc8), // Gold border
    );
    draw_text_line(
        hdc,
        "👑 HexBrain 排位选人辅助 · 5v5 对位胜率全景分析",
        banner_x + (16.0 * scale_x) as i32,
        banner_y + (10.0 * scale_y) as i32,
        COLORREF(0x0045f0f0), // Cyan/gold text
    );

    // 2. Left side: My Team (我方 5 个槽位, 吸附于我方头像右侧 X=205)
    let left_x = (205.0 * scale_x) as i32;
    let card_w = (140.0 * scale_x) as i32;
    let card_h = (44.0 * scale_y) as i32;
    let start_y = (100.0 * scale_y) as i32;
    let pitch_y = (92.0 * scale_y) as i32;

    for (i, slot) in data.my_team.iter().take(5).enumerate() {
        let cy = start_y + i as i32 * pitch_y;
        let border_col = if slot.is_local_player {
            COLORREF(0x0000d7ff) // Gold for local player
        } else {
            COLORREF(0x0038bdf8) // Sky blue for teammates
        };

        draw_card_rect(hdc, left_x, cy, card_w, card_h, COLORREF(0x001e140a), border_col);

        let title_line = format!("[{}] {}", slot.position_cn, slot.champion_name);
        draw_text_line(hdc, &title_line, left_x + 8, cy + 5, COLORREF(0x00ffffff));

        let stat_line = format!("胜率: {:.1}% | T{}级", slot.win_rate * 100.0, slot.tier);
        let stat_col = if slot.win_rate >= 0.52 {
            COLORREF(0x004ade80) // Green
        } else {
            COLORREF(0x00c0c0c0) // Silver
        };
        draw_text_line(hdc, &stat_line, left_x + 8, cy + 24, stat_col);
    }

    // 3. Right side: Their Team (敌方 5 个槽位, 吸附于敌方头像左侧 X=935)
    let right_x = (935.0 * scale_x) as i32;
    for (i, slot) in data.their_team.iter().take(5).enumerate() {
        let cy = start_y + i as i32 * pitch_y;
        let border_col = COLORREF(0x005546ff); // Red/rose border

        draw_card_rect(hdc, right_x, cy, card_w, card_h, COLORREF(0x001e140a), border_col);

        let title_line = format!("[{}] {}", slot.position_cn, slot.champion_name);
        draw_text_line(hdc, &title_line, right_x + 8, cy + 5, COLORREF(0x00ffffff));

        let stat_line = format!("胜率: {:.1}% | T{}级", slot.win_rate * 100.0, slot.tier);
        let stat_col = if slot.win_rate >= 0.52 {
            COLORREF(0x005546ff) // Red warning
        } else {
            COLORREF(0x00c0c0c0)
        };
        draw_text_line(hdc, &stat_line, right_x + 8, cy + 24, stat_col);
    }

    // 4. Bottom Recommendation Bar
    if !data.recommendation.is_empty() {
        let bot_w = ((680.0 * scale_x) as i32).min(width - 40);
        let bot_h = (36.0 * scale_y) as i32;
        let bot_x = (width - bot_w) / 2;
        let bot_y = (height - bot_h - (16.0 * scale_y) as i32).max(start_y + 5 * pitch_y + 10);

        draw_card_rect(
            hdc,
            bot_x,
            bot_y,
            bot_w,
            bot_h,
            COLORREF(0x001e1b0f),
            COLORREF(0x0034d399), // Emerald border
        );
        draw_text_line(hdc, &data.recommendation, bot_x + 16, bot_y + 10, COLORREF(0x006ee7b7));
    }
}

// -------------------------------------------------------------
// GDI Helper Primitives
// -------------------------------------------------------------

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

    // 1. Draw outer rounded rect with border color
    let border_brush = CreateSolidBrush(border_color);
    let old_brush = SelectObject(hdc, border_brush);
    let _ = RoundRect(hdc, x, y, x + w, y + h, 10, 10);

    // 2. Draw inner rounded rect inset by 2px with bg color
    let bg_brush = CreateSolidBrush(bg_color);
    let _ = SelectObject(hdc, bg_brush);
    let _ = RoundRect(hdc, x + 2, y + 2, x + w - 2, y + h - 2, 8, 8);

    // Restore and delete brushes
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
    use windows::Win32::Graphics::Gdi::{SetTextColor, TextOutW};

    SetTextColor(hdc, color);
    let wide: Vec<u16> = text.encode_utf16().collect();
    let _ = TextOutW(hdc, x, y, &wide);
}
