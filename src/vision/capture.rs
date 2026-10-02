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

        // Fallback default 1080p
        ViewportRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[cfg(windows)]
    pub fn get_lol_window_rect() -> Option<ViewportRect> {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
            IsIconic, IsWindow, IsWindowVisible,
        };

        // Attach to user interactive desktop if launched from background
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

        struct Candidate {
            pid: u32,
            is_game: bool,
            rect: ViewportRect,
        }

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return BOOL(1);
            }

            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return BOOL(1);
            }

            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w < 600 || h < 400 || rect.left < -10000 {
                return BOOL(1);
            }

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

            let is_client = class_name.contains("rclient")
                || title.contains("league of legends")
                || title.contains("英雄联盟");

            if is_game || is_client {
                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));

                let out = lparam.0 as *mut Vec<Candidate>;
                if !out.is_null() {
                    (*out).push(Candidate {
                        pid,
                        is_game,
                        rect: ViewportRect {
                            x: rect.left,
                            y: rect.top,
                            width: w,
                            height: h,
                        },
                    });
                }
            }

            BOOL(1)
        }

        let mut candidates: Vec<Candidate> = Vec::new();
        unsafe {
            let lparam = LPARAM(&mut candidates as *mut _ as isize);
            let _ = EnumWindows(Some(enum_proc), lparam);
        }

        if let Some(game) = candidates.iter().find(|c| c.is_game) {
            LEAGUE_PID.store(game.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(game.rect.clone());
        }

        if let Some(client) = candidates.iter().find(|c| !c.is_game) {
            LEAGUE_PID.store(client.pid, std::sync::atomic::Ordering::Relaxed);
            return Some(client.rect.clone());
        }

        None
    }

    #[cfg(not(windows))]
    pub fn get_lol_window_rect() -> Option<ViewportRect> {
        None
    }

    pub fn get_augment_title_band_rect() -> ViewportRect {
        let vp = Self::get_effective_viewport();
        let eff_w = vp.width as f64;
        let eff_h = vp.height as f64;

        ViewportRect {
            x: (vp.x as f64 + eff_w * 0.23) as i32,
            y: (vp.y as f64 + eff_h * 0.385) as i32,
            width: (eff_w * 0.55) as i32,
            height: (eff_h * 0.06) as i32,
        }
    }
}
