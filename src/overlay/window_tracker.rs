#[derive(Debug, Clone, Default)]
pub struct WindowTracker;

impl WindowTracker {
    #[cfg(windows)]
    pub fn is_league_active() -> bool {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowTextW};

        unsafe {
            let fg = GetForegroundWindow();
            if fg == HWND::default() {
                return false;
            }

            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(fg, &mut class_buf);
            if len > 0 {
                let class_name = String::from_utf16_lossy(&class_buf[..len as usize]).to_lowercase();
                if class_name.contains("rclient") || class_name.contains("riotwindowclass") {
                    return true;
                }
            }

            let mut title_buf = [0u16; 256];
            let t_len = GetWindowTextW(fg, &mut title_buf);
            if t_len > 0 {
                let title = String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase();
                if title.contains("league of legends") || title.contains("英雄联盟") {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(windows)]
    pub fn is_game_active() -> bool {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowTextW};

        unsafe {
            let fg = GetForegroundWindow();
            if fg == HWND::default() {
                return false;
            }

            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(fg, &mut class_buf);
            if len > 0 {
                let class_name = String::from_utf16_lossy(&class_buf[..len as usize]).to_lowercase();
                if class_name.contains("riotwindowclass") {
                    return true;
                }
            }

            let mut title_buf = [0u16; 256];
            let t_len = GetWindowTextW(fg, &mut title_buf);
            if t_len > 0 {
                let title = String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase();
                if title.contains("league of legends (tm)") {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(windows))]
    pub fn is_league_active() -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn is_game_active() -> bool {
        false
    }
}
