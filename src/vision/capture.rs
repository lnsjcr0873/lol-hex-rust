#[derive(Debug, Clone)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct ScreenCapture;

impl ScreenCapture {
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
        use windows::core::w;
        use windows::Win32::Foundation::{HWND, POINT, RECT};
        use windows::Win32::UI::WindowsAndMessaging::{
            ClientToScreen, FindWindowW, GetClientRect, GetWindowRect, IsWindow, IsWindowVisible,
        };

        unsafe {
            let mut hwnd = FindWindowW(None, w!("League of Legends (TM) Client")).unwrap_or_default();
            if hwnd == HWND::default() || !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
                hwnd = FindWindowW(w!("RiotWindowClass"), None).unwrap_or_default();
            }

            if hwnd != HWND::default() && IsWindow(hwnd).as_bool() && IsWindowVisible(hwnd).as_bool() {
                let mut client_rect = RECT::default();
                if GetClientRect(hwnd, &mut client_rect).is_ok() {
                    let mut pt = POINT { x: client_rect.left, y: client_rect.top };
                    let _ = ClientToScreen(hwnd, &mut pt);
                    let w = client_rect.right - client_rect.left;
                    let h = client_rect.bottom - client_rect.top;
                    if w >= 640 && h >= 480 {
                        return Some(ViewportRect {
                            x: pt.x,
                            y: pt.y,
                            width: w,
                            height: h,
                        });
                    }
                }

                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect).is_ok() {
                    let w = rect.right - rect.left;
                    let h = rect.bottom - rect.top;
                    if w >= 640 && h >= 480 {
                        return Some(ViewportRect {
                            x: rect.left,
                            y: rect.top,
                            width: w,
                            height: h,
                        });
                    }
                }
            }
        }
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
