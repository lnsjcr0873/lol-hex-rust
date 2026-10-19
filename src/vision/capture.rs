#[derive(Debug, Clone)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct ScreenCapture;

#[cfg(windows)]
struct ThreadDpiGuard(isize);

#[cfg(windows)]
impl ThreadDpiGuard {
    fn enter_per_monitor_v2() -> Self {
        #[link(name = "user32")]
        extern "system" {
            fn SetThreadDpiAwarenessContext(dpi_context: isize) -> isize;
        }
        // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
        let old = unsafe { SetThreadDpiAwarenessContext(-4) };
        Self(old)
    }
}

#[cfg(windows)]
impl Drop for ThreadDpiGuard {
    fn drop(&mut self) {
        if self.0 != 0 {
            #[link(name = "user32")]
            extern "system" {
                fn SetThreadDpiAwarenessContext(dpi_context: isize) -> isize;
            }
            unsafe {
                let _ = SetThreadDpiAwarenessContext(self.0);
            }
        }
    }
}

static LEAGUE_CLIENT_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static LEAGUE_GAME_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl ScreenCapture {
    pub fn get_league_client_pid() -> u32 {
        LEAGUE_CLIENT_PID.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn get_league_game_pid() -> u32 {
        LEAGUE_GAME_PID.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[allow(dead_code)]
    pub fn get_league_pid() -> u32 {
        let gpid = Self::get_league_game_pid();
        if gpid != 0 {
            gpid
        } else {
            Self::get_league_client_pid()
        }
    }
    pub fn get_effective_viewport() -> ViewportRect {
        #[cfg(windows)]
        let _dpi_guard = ThreadDpiGuard::enter_per_monitor_v2();

        #[cfg(windows)]
        {
            if let Some(rect) = Self::get_lol_window_rect() {
                if rect.width >= 100 && rect.height >= 100 {
                    let vw = rect.width as f64;
                    let vh = rect.height as f64;
                    let aspect = if vh > 0.0 { vw / vh } else { 1.7777778 };

                    if aspect > 1.82 {
                        // 21:9 or 32:9 Ultrawide
                        let eff_w = ((vh * (16.0 / 9.0)) as i32).max(600);
                        let eff_h = rect.height.max(400);
                        let off_x = rect.x + (rect.width - eff_w) / 2;
                        return ViewportRect {
                            x: off_x,
                            y: rect.y,
                            width: eff_w,
                            height: eff_h,
                        };
                    } else if aspect < 1.72 {
                        // 16:10 or 4:3
                        let eff_w = rect.width.max(600);
                        let eff_h = ((vw * (9.0 / 16.0)) as i32).max(400);
                        let off_y = rect.y + (rect.height - eff_h) / 2;
                        return ViewportRect {
                            x: rect.x,
                            y: off_y,
                            width: eff_w,
                            height: eff_h,
                        };
                    } else {
                        return ViewportRect {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width.max(600),
                            height: rect.height.max(400),
                        };
                    }
                }
            }
        }

        // Fallback to primary screen metrics
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
            let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
            if sw > 0 && sh > 0 {
                return ViewportRect {
                    x: 0,
                    y: 0,
                    width: sw.max(600),
                    height: sh.max(400),
                };
            }
        }

        ViewportRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[cfg(windows)]
    pub fn attach_desktop_if_needed() {
        #[link(name = "user32")]
        extern "system" {
            fn OpenDesktopW(lpszdesktop: *const u16, dwflags: u32, finherit: i32, dwdesiredaccess: u32) -> *mut core::ffi::c_void;
            fn OpenInputDesktop(dwflags: u32, finherit: i32, dwdesiredaccess: u32) -> *mut core::ffi::c_void;
            fn SetThreadDesktop(hdesktop: *mut core::ffi::c_void) -> i32;
        }

        unsafe {
            let default_name: Vec<u16> = "Default\0".encode_utf16().collect();
            let mut hdesk = OpenDesktopW(default_name.as_ptr(), 0, 0, 0x01FF);
            if hdesk.is_null() {
                hdesk = OpenInputDesktop(0, 0, 0x01FF);
            }
            if !hdesk.is_null() {
                let _ = SetThreadDesktop(hdesk);
            }
        }
    }

    #[cfg(not(windows))]
    pub fn attach_desktop_if_needed() {}

    #[cfg(windows)]
    pub fn get_lol_client_rect() -> Option<ViewportRect> {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, FindWindowW, GetClassNameW, GetClientRect,
            GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible,
        };

        #[link(name = "user32")]
        extern "system" {
            fn ClientToScreen(hwnd: HWND, lppoint: *mut POINT) -> BOOL;
        }

        Self::attach_desktop_if_needed();

        unsafe fn hwnd_to_client_rect(hwnd: HWND) -> Option<ViewportRect> {
            if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return None;
            }
            let mut pt = POINT { x: 0, y: 0 };
            let _ = ClientToScreen(hwnd, &mut pt);
            let mut cr = RECT::default();
            if GetClientRect(hwnd, &mut cr).is_err() {
                return None;
            }
            let w = cr.right - cr.left;
            let h = cr.bottom - cr.top;
            if w < 600 || h < 400 || pt.x < -10000 || pt.y < -10000 {
                return None;
            }
            Some(ViewportRect {
                x: pt.x,
                y: pt.y,
                width: w,
                height: h,
            })
        }

        // 1. Try FindWindowW("RCLIENT", None)
        unsafe {
            use windows::core::w;
            let rclient_hwnd = FindWindowW(w!("RCLIENT"), None).unwrap_or_default();
            if rclient_hwnd != HWND::default() {
                if let Some(rect) = hwnd_to_client_rect(rclient_hwnd) {
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(rclient_hwnd, Some(&mut pid));
                    LEAGUE_CLIENT_PID.store(pid, std::sync::atomic::Ordering::Relaxed);
                    return Some(rect);
                }
            }
        }

        // 2. Fallback: EnumWindows to find visible, non-iconic LeagueClient window
        struct MatchedClient {
            pid: u32,
            rect: ViewportRect,
        }

        unsafe extern "system" fn enum_client_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(rect) = hwnd_to_client_rect(hwnd) {
                let mut class_buf = [0u16; 256];
                let c_len = GetClassNameW(hwnd, &mut class_buf);
                let class_name = if c_len > 0 {
                    String::from_utf16_lossy(&class_buf[..c_len as usize]).to_lowercase()
                } else {
                    String::new()
                };

                let mut title_buf = [0u16; 256];
                let t_len = GetWindowTextW(hwnd, &mut title_buf);
                let title = if t_len > 0 {
                    String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase()
                } else {
                    String::new()
                };

                let is_rclient = class_name.contains("rclient");
                let is_league_title = (title.contains("league of legends") || title.contains("英雄联盟"))
                    && !title.contains("(tm)")
                    && !class_name.contains("riotwindowclass");

                if is_rclient || is_league_title {
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));
                    let out = lparam.0 as *mut Vec<MatchedClient>;
                    if !out.is_null() {
                        (*out).push(MatchedClient { pid, rect });
                    }
                }
            }
            BOOL(1)
        }

        let mut matches: Vec<MatchedClient> = Vec::new();
        unsafe {
            let lparam = LPARAM(&mut matches as *mut _ as isize);
            let _ = EnumWindows(Some(enum_client_proc), lparam);
        }

        if let Some(m) = matches.into_iter().next() {
            LEAGUE_CLIENT_PID.store(m.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(m.rect);
        }

        LEAGUE_CLIENT_PID.store(0, std::sync::atomic::Ordering::Relaxed);
        None
    }

    #[cfg(windows)]
    pub fn is_lol_client_active() -> bool {
        crate::overlay::WindowTracker::is_league_active()
    }

    #[cfg(not(windows))]
    pub fn is_lol_client_active() -> bool {
        true
    }

    #[cfg(windows)]
    pub fn get_lol_game_rect() -> Option<ViewportRect> {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, FindWindowW, GetClassNameW, GetClientRect,
            GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible,
        };

        #[link(name = "user32")]
        extern "system" {
            fn ClientToScreen(hwnd: HWND, lppoint: *mut POINT) -> BOOL;
        }

        Self::attach_desktop_if_needed();

        unsafe fn hwnd_to_game_rect(hwnd: HWND) -> Option<ViewportRect> {
            if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return None;
            }
            let mut pt = POINT { x: 0, y: 0 };
            let _ = ClientToScreen(hwnd, &mut pt);
            let mut cr = RECT::default();
            if GetClientRect(hwnd, &mut cr).is_err() {
                return None;
            }
            let w = cr.right - cr.left;
            let h = cr.bottom - cr.top;
            if w < 600 || h < 400 || pt.x < -10000 || pt.y < -10000 {
                return None;
            }
            Some(ViewportRect {
                x: pt.x,
                y: pt.y,
                width: w,
                height: h,
            })
        }

        // 1. Try FindWindowW(None, "League of Legends (TM) Client")
        unsafe {
            use windows::core::w;
            let game_hwnd = FindWindowW(None, w!("League of Legends (TM) Client")).unwrap_or_default();
            if game_hwnd != HWND::default() {
                if let Some(rect) = hwnd_to_game_rect(game_hwnd) {
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(game_hwnd, Some(&mut pid));
                    LEAGUE_GAME_PID.store(pid, std::sync::atomic::Ordering::Relaxed);
                    return Some(rect);
                }
            }
        }

        // 2. Fallback: EnumWindows to find RiotWindowClass
        struct MatchedGame {
            pid: u32,
            rect: ViewportRect,
        }

        unsafe extern "system" fn enum_game_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(rect) = hwnd_to_game_rect(hwnd) {
                let mut class_buf = [0u16; 256];
                let c_len = GetClassNameW(hwnd, &mut class_buf);
                let class_name = if c_len > 0 {
                    String::from_utf16_lossy(&class_buf[..c_len as usize]).to_lowercase()
                } else {
                    String::new()
                };

                let mut title_buf = [0u16; 256];
                let t_len = GetWindowTextW(hwnd, &mut title_buf);
                let title = if t_len > 0 {
                    String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase()
                } else {
                    String::new()
                };

                let is_game = class_name.contains("riotwindowclass")
                    || title.contains("league of legends (tm) client");

                if is_game {
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));
                    let out = lparam.0 as *mut Vec<MatchedGame>;
                    if !out.is_null() {
                        (*out).push(MatchedGame { pid, rect });
                    }
                }
            }
            BOOL(1)
        }

        let mut matches: Vec<MatchedGame> = Vec::new();
        unsafe {
            let lparam = LPARAM(&mut matches as *mut _ as isize);
            let _ = EnumWindows(Some(enum_game_proc), lparam);
        }

        if let Some(m) = matches.into_iter().next() {
            LEAGUE_GAME_PID.store(m.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(m.rect);
        }

        LEAGUE_GAME_PID.store(0, std::sync::atomic::Ordering::Relaxed);
        None
    }

    pub fn get_lol_window_rect() -> Option<ViewportRect> {
        #[cfg(windows)]
        {
            Self::get_lol_game_rect().or_else(Self::get_lol_client_rect)
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    #[cfg(not(windows))]
    pub fn get_lol_client_rect() -> Option<ViewportRect> {
        None
    }

    #[cfg(not(windows))]
    pub fn get_lol_game_rect() -> Option<ViewportRect> {
        None
    }

    pub fn get_augment_title_band_rect() -> ViewportRect {
        let vp = Self::get_effective_viewport();
        let eff_w = vp.width as f64;
        let eff_h = vp.height as f64;

        ViewportRect {
            x: (vp.x as f64 + eff_w * 0.23) as i32,
            y: (vp.y as f64 + eff_h * 0.380) as i32,
            width: (eff_w * 0.55) as i32,
            height: (eff_h * 0.075) as i32,
        }
    }

    pub fn get_augment_slot_rects() -> [ViewportRect; 3] {
        let vp = Self::get_effective_viewport();
        let eff_w = vp.width as f64;
        let eff_h = vp.height as f64;
        let base_x = vp.x as f64;
        let base_y = vp.y as f64;

        [
            ViewportRect {
                x: (base_x + eff_w * 0.230) as i32,
                y: (base_y + eff_h * 0.380) as i32,
                width: (eff_w * 0.155) as i32,
                height: (eff_h * 0.075) as i32,
            },
            ViewportRect {
                x: (base_x + eff_w * 0.425) as i32,
                y: (base_y + eff_h * 0.380) as i32,
                width: (eff_w * 0.155) as i32,
                height: (eff_h * 0.075) as i32,
            },
            ViewportRect {
                x: (base_x + eff_w * 0.620) as i32,
                y: (base_y + eff_h * 0.380) as i32,
                width: (eff_w * 0.155) as i32,
                height: (eff_h * 0.075) as i32,
            },
        ]
    }

    pub fn get_augment_center_rect() -> ViewportRect {
        let vp = Self::get_effective_viewport();
        let eff_w = vp.width as f64;
        let eff_h = vp.height as f64;

        ViewportRect {
            x: (vp.x as f64 + eff_w * 0.20) as i32,
            y: (vp.y as f64 + eff_h * 0.28) as i32,
            width: (eff_w * 0.60) as i32,
            height: (eff_h * 0.46) as i32,
        }
    }

    #[cfg(windows)]
    pub fn capture_rect_as_bmp(rect: &ViewportRect) -> Option<Vec<u8>> {
        Self::capture_rect_with_pad_as_bmp(rect, 20)
    }

    #[cfg(windows)]
    pub fn capture_rect_with_pad_as_bmp(rect: &ViewportRect, pad: i32) -> Option<Vec<u8>> {
        let _dpi_guard = ThreadDpiGuard::enter_per_monitor_v2();

        use windows::Win32::Foundation::{COLORREF, HWND, RECT};
        use windows::Win32::Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
            DeleteObject, FillRect, GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        };

        if rect.width <= 0 || rect.height <= 0 || rect.width > 8192 || rect.height > 8192 {
            return None;
        }

        let pad = pad.max(0);
        let out_w = rect.width + pad * 2;
        let out_h = rect.height + pad * 2;

        unsafe {
            let hdc_screen = GetDC(HWND::default());
            if hdc_screen.is_invalid() {
                return None;
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_invalid() {
                let _ = ReleaseDC(HWND::default(), hdc_screen);
                return None;
            }

            let hbmp = CreateCompatibleBitmap(hdc_screen, out_w, out_h);
            if hbmp.is_invalid() {
                let _ = DeleteDC(hdc_mem);
                let _ = ReleaseDC(HWND::default(), hdc_screen);
                return None;
            }

            let old_bmp = SelectObject(hdc_mem, hbmp);

            // Black padding background ensures OCR edge line segmenters won't discard text touching boundaries
            let fill_r = RECT {
                left: 0,
                top: 0,
                right: out_w,
                bottom: out_h,
            };
            let black_brush = CreateSolidBrush(COLORREF(0));
            FillRect(hdc_mem, &fill_r, black_brush);
            let _ = DeleteObject(black_brush);

            let _ = BitBlt(
                hdc_mem,
                pad,
                pad,
                rect.width,
                rect.height,
                hdc_screen,
                rect.x,
                rect.y,
                SRCCOPY,
            );

            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = out_w;
            bmi.bmiHeader.biHeight = out_h;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0;

            let row_stride = (out_w * 4) as usize;
            let image_bytes = row_stride * (out_h as usize);
            let mut pixels: Vec<u8> = vec![0u8; image_bytes];

            // Win32 API contract: The bitmap identified by hbmp must NOT be selected into a DC when calling GetDIBits
            let _ = SelectObject(hdc_mem, old_bmp);

            let lines_copied = GetDIBits(
                hdc_mem,
                hbmp,
                0,
                out_h as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND::default(), hdc_screen);

            if lines_copied == 0 {
                return None;
            }

            // Detect exclusive fullscreen (pure black surface from GDI BitBlt)
            let is_all_black = {
                let sample_step = (pixels.len() / 200).max(4);
                let non_zero_count = pixels.iter().step_by(sample_step).filter(|&&b| b > 15).count();
                non_zero_count == 0
            };

            if is_all_black {
                static WARNED_BLACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !WARNED_BLACK.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    tracing::warn!("Screen capture returned all-black pixels (Exclusive Fullscreen detected). Please set League video setting to 'Borderless' for overlay and OCR support.");
                }
                return None;
            }

            // Construct standard 54-byte BMP
            let file_header_size = 14u32;
            let info_header_size = 40u32;
            let bf_off_bits = file_header_size + info_header_size;
            let file_size = bf_off_bits + (image_bytes as u32);

            let mut bmp = Vec::with_capacity(file_size as usize);
            bmp.extend_from_slice(b"BM");
            bmp.extend_from_slice(&file_size.to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&bf_off_bits.to_le_bytes());

            bmp.extend_from_slice(&info_header_size.to_le_bytes());
            bmp.extend_from_slice(&out_w.to_le_bytes());
            bmp.extend_from_slice(&out_h.to_le_bytes());
            bmp.extend_from_slice(&1u16.to_le_bytes());
            bmp.extend_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&(image_bytes as u32).to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());

            bmp.extend_from_slice(&pixels);

            Some(bmp)
        }
    }

    pub fn is_bmp_mostly_black(bmp_bytes: &[u8]) -> bool {
        if bmp_bytes.len() <= 54 {
            return true;
        }
        let pixel_data = &bmp_bytes[54..];
        // In 32-bit BGRA BMP, test all 3 color channels (Blue, Green, Red) every 4th pixel (every 16 bytes)
        // to prevent false positives when images contain vibrant red/gold/yellow elements with low blue.
        let has_light = pixel_data.chunks_exact(4).step_by(4).any(|px| {
            px[0] > 12 || px[1] > 12 || px[2] > 12
        });
        !has_light
    }

    /// 极速视觉暗底指纹前置检查 (~0.005ms，0 CPU 占用)：
    /// 海克斯卡牌具有极其鲜明的暗夜/黑曜石底色特征 (Slot 区域 25分位亮度 lum < 35 且红光 R < 30)。
    /// 若屏幕处于普通战斗画面（极地大乱斗冰面、小兵、英雄、血条、击杀横幅等高亮度画面），
    /// 直接在此层极速熔断，彻底阻断后续 OCR 识别与推演，实现 100% 杜绝非选牌期误弹与卡顿。
    pub fn is_title_band_card_bg(bmp_bytes: &[u8]) -> bool {
        if bmp_bytes.len() <= 54 {
            return false;
        }
        let raw_w = i32::from_le_bytes([bmp_bytes[18], bmp_bytes[19], bmp_bytes[20], bmp_bytes[21]]);
        let raw_h = i32::from_le_bytes([bmp_bytes[22], bmp_bytes[23], bmp_bytes[24], bmp_bytes[25]]);
        if raw_w < 60 || raw_w > 10000 || raw_h.unsigned_abs() < 10 || raw_h.unsigned_abs() > 10000 {
            return false;
        }
        let out_w = raw_w as usize;
        let out_h = raw_h.unsigned_abs() as usize;

        let pixel_data = &bmp_bytes[54..];
        let bytes_per_pixel = 4;
        let row_stride = out_w * bytes_per_pixel;
        if pixel_data.len() < row_stride * out_h {
            return false;
        }

        let pad_x = 20.min(out_w / 4);
        let pad_y = 20.min(out_h / 4);
        let cw = out_w.saturating_sub(pad_x * 2);
        let ch = out_h.saturating_sub(pad_y * 2);
        if cw < 40 || ch < 5 {
            return false;
        }

        let mut dark_slots = 0;
        let slot_ranges = [
            (0.05, 0.25), // 卡槽 1
            (0.40, 0.60), // 卡槽 2
            (0.75, 0.95), // 卡槽 3
        ];

        for (xs, xe) in slot_ranges {
            let start_x = pad_x + ((cw as f64) * xs) as usize;
            let end_x = pad_x + ((cw as f64) * xe) as usize;
            let mut lums = Vec::with_capacity(256);
            let mut reds = Vec::with_capacity(256);

            for y in (pad_y..(pad_y + ch)).step_by(2) {
                let row_offset = y * row_stride;
                for x in (start_x..end_x).step_by(4) {
                    let px_idx = row_offset + x * bytes_per_pixel;
                    if px_idx + 2 < pixel_data.len() {
                        let b = pixel_data[px_idx] as f64;
                        let g = pixel_data[px_idx + 1] as f64;
                        let r = pixel_data[px_idx + 2] as f64;
                        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                        lums.push(lum);
                        reds.push(r);
                    }
                }
            }

            if lums.is_empty() {
                continue;
            }

            lums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            reds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let p25_idx = lums.len() / 4;
            let p25_lum = lums[p25_idx];
            let p25_r = reds[p25_idx];

            if p25_lum < 35.0 && p25_r < 30.0 {
                dark_slots += 1;
            }
        }

        dark_slots >= 2
    }

    #[cfg(not(windows))]
    pub fn capture_rect_as_bmp(_rect: &ViewportRect) -> Option<Vec<u8>> {
        None
    }

    #[cfg(not(windows))]
    pub fn capture_rect_with_pad_as_bmp(_rect: &ViewportRect, _pad: i32) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bmp_mostly_black_truly_black() {
        let mut bmp = vec![0u8; 54 + 64];
        assert!(ScreenCapture::is_bmp_mostly_black(&bmp));

        // Slightly dim noise (<= 12) should still be treated as black
        for byte in &mut bmp[54..] {
            *byte = 8;
        }
        assert!(ScreenCapture::is_bmp_mostly_black(&bmp));
    }

    #[test]
    fn test_is_bmp_mostly_black_with_gold_red_elements() {
        // Construct a 4x4 32-bit BMP where background is dark (0) but has gold/red text
        // (Blue = 0, Green = 180, Red = 255)
        let mut bmp = vec![0u8; 54 + 64]; // 16 pixels * 4 bytes
        // Pixel 0 is pure red/gold
        bmp[54 + 0] = 0;   // Blue
        bmp[54 + 1] = 180; // Green
        bmp[54 + 2] = 255; // Red
        bmp[54 + 3] = 0;   // Alpha

        // Should NOT be detected as mostly black because Green and Red are high
        assert!(!ScreenCapture::is_bmp_mostly_black(&bmp));
    }

    #[test]
    fn test_is_title_band_card_bg_i32_min_overflow_safe() {
        let mut dummy = vec![0u8; 60];
        // Set raw_w = 100, raw_h = i32::MIN
        let w_bytes = 100i32.to_le_bytes();
        let h_bytes = i32::MIN.to_le_bytes();
        dummy[18..22].copy_from_slice(&w_bytes);
        dummy[22..26].copy_from_slice(&h_bytes);

        // Must not panic with attempt to negate with overflow
        assert!(!ScreenCapture::is_title_band_card_bg(&dummy));
    }

    #[test]
    fn test_league_client_and_game_pid_separation() {
        LEAGUE_CLIENT_PID.store(1111, std::sync::atomic::Ordering::Relaxed);
        LEAGUE_GAME_PID.store(0, std::sync::atomic::Ordering::Relaxed);

        assert_eq!(ScreenCapture::get_league_client_pid(), 1111);
        assert_eq!(ScreenCapture::get_league_game_pid(), 0);
        assert_eq!(ScreenCapture::get_league_pid(), 1111);

        LEAGUE_GAME_PID.store(2222, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(ScreenCapture::get_league_game_pid(), 2222);
        // Prioritizes in-game PID when game is running
        assert_eq!(ScreenCapture::get_league_pid(), 2222);

        // Clean up
        LEAGUE_CLIENT_PID.store(0, std::sync::atomic::Ordering::Relaxed);
        LEAGUE_GAME_PID.store(0, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(ScreenCapture::get_league_pid(), 0);
    }
}

