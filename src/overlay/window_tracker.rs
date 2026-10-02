#[derive(Debug, Clone, Default)]
pub struct WindowTracker;

#[cfg(windows)]
fn is_process_league_or_riot(pid: u32) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwdesiredaccess: u32, binherithandle: i32, dwprocessid: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(hobject: *mut core::ffi::c_void) -> i32;
        fn QueryFullProcessImageNameW(hprocess: *mut core::ffi::c_void, dwflags: u32, lpexename: *mut u16, lpsize: *mut u32) -> i32;
    }

    unsafe {
        // PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
        let handle = OpenProcess(0x1000, 0, pid);
        if handle.is_null() {
            return false;
        }

        let mut buf = [0u16; 512];
        let mut size = buf.len() as u32;
        let res = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        let _ = CloseHandle(handle);

        if res != 0 && size > 0 {
            let path = String::from_utf16_lossy(&buf[..size as usize]).to_lowercase();
            return path.contains("league") || path.contains("riot") || path.contains("hex");
        }
    }
    false
}

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

            // 1. PID & Process Name Matching (Penetrates CEF Child Windows & Renderers)
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
                if is_process_league_or_riot(fg_pid) {
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
                if is_process_league_or_riot(fg_pid) {
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
