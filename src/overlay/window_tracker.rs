#[derive(Debug, Clone, Default)]
pub struct WindowTracker;

impl WindowTracker {
    #[cfg(windows)]
    pub fn is_league_active() -> bool {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetAncestor, GetClassNameW, GetForegroundWindow, GetWindowTextW,
            GetWindowThreadProcessId, GA_ROOT,
        };

        unsafe {
            let fg = GetForegroundWindow();
            if fg == HWND::default() {
                return false;
            }

            // 1. PID Matching (CEF Child Window & Input Field Penetration)
            let mut fg_pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            if fg_pid != 0 {
                if fg_pid == std::process::id() {
                    return true;
                }
                let lol_pid = crate::vision::capture::get_league_pid();
                if lol_pid != 0 && fg_pid == lol_pid {
                    return true;
                }
            }

            // 2. Direct Window Class & Title Matching
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

            // 3. Ancestor Root Window Check
            let root = GetAncestor(fg, GA_ROOT);
            if root != HWND::default() && root != fg {
                let r_len = GetClassNameW(root, &mut class_buf);
                if r_len > 0 {
                    let class_name = String::from_utf16_lossy(&class_buf[..r_len as usize]).to_lowercase();
                    if class_name.contains("rclient") || class_name.contains("riotwindowclass") {
                        return true;
                    }
                }
                let rt_len = GetWindowTextW(root, &mut title_buf);
                if rt_len > 0 {
                    let title = String::from_utf16_lossy(&title_buf[..rt_len as usize]).to_lowercase();
                    if title.contains("league of legends") || title.contains("英雄联盟") {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[cfg(windows)]
    pub fn is_game_active() -> bool {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetAncestor, GetClassNameW, GetForegroundWindow, GetWindowTextW,
            GetWindowThreadProcessId, GA_ROOT,
        };

        unsafe {
            let fg = GetForegroundWindow();
            if fg == HWND::default() {
                return false;
            }

            let mut fg_pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            if fg_pid != 0 {
                if fg_pid == std::process::id() {
                    return true;
                }
                let lol_pid = crate::vision::capture::get_league_pid();
                if lol_pid != 0 && fg_pid == lol_pid {
                    return true;
                }
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

            let root = GetAncestor(fg, GA_ROOT);
            if root != HWND::default() && root != fg {
                let r_len = GetClassNameW(root, &mut class_buf);
                if r_len > 0 {
                    let class_name = String::from_utf16_lossy(&class_buf[..r_len as usize]).to_lowercase();
                    if class_name.contains("riotwindowclass") {
                        return true;
                    }
                }
                let rt_len = GetWindowTextW(root, &mut title_buf);
                if rt_len > 0 {
                    let title = String::from_utf16_lossy(&title_buf[..rt_len as usize]).to_lowercase();
                    if title.contains("league of legends (tm)") {
                        return true;
                    }
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
