#[derive(Debug, Clone)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct ScreenCapture;

static LEAGUE_PID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn get_league_pid() -> u32 {
    LEAGUE_PID.load(std::sync::atomic::Ordering::Relaxed)
}

impl ScreenCapture {
    pub fn get_league_pid() -> u32 {
        get_league_pid()
    }
    pub fn get_effective_viewport() -> ViewportRect {
        #[cfg(windows)]
        {
            if let Some(rect) = Self::get_lol_window_rect() {
                let vw = rect.width as f64;
                let vh = rect.height as f64;
                let aspect = if vh > 0.0 { vw / vh } else { 1.7777778 };

                if aspect > 1.82 {
                    // 21:9 or 32:9 Ultrawide
                    let eff_w = (vh * (16.0 / 9.0)) as i32;
                    let eff_h = rect.height;
                    let off_x = rect.x + (rect.width - eff_w) / 2;
                    return ViewportRect {
                        x: off_x,
                        y: rect.y,
                        width: eff_w,
                        height: eff_h,
                    };
                } else if aspect < 1.72 {
                    // 16:10 or 4:3
                    let eff_w = rect.width;
                    let eff_h = (vw * (9.0 / 16.0)) as i32;
                    let off_y = rect.y + (rect.height - eff_h) / 2;
                    return ViewportRect {
                        x: rect.x,
                        y: off_y,
                        width: eff_w,
                        height: eff_h,
                    };
                } else {
                    return rect;
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
                    width: sw,
                    height: sh,
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
            if w < 600 || h < 400 || pt.x < -10000 {
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
                    LEAGUE_PID.store(pid, std::sync::atomic::Ordering::Relaxed);
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
            LEAGUE_PID.store(m.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(m.rect);
        }

        None
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
            if w < 600 || h < 400 || pt.x < -10000 {
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
                    LEAGUE_PID.store(pid, std::sync::atomic::Ordering::Relaxed);
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
            LEAGUE_PID.store(m.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(m.rect);
        }

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
        use windows::Win32::Foundation::{COLORREF, HWND, RECT};
        use windows::Win32::Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
            DeleteObject, FillRect, GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        };

        if rect.width <= 0 || rect.height <= 0 {
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

            let lines_copied = GetDIBits(
                hdc_mem,
                hbmp,
                0,
                out_h as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            let _ = SelectObject(hdc_mem, old_bmp);
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND::default(), hdc_screen);

            if lines_copied == 0 {
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

    #[cfg(not(windows))]
    pub fn capture_rect_as_bmp(_rect: &ViewportRect) -> Option<Vec<u8>> {
        None
    }

    #[cfg(not(windows))]
    pub fn capture_rect_with_pad_as_bmp(_rect: &ViewportRect, _pad: i32) -> Option<Vec<u8>> {
        None
    }
}
