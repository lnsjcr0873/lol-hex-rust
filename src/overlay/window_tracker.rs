#[derive(Debug, Clone, Default)]
pub struct WindowTracker;

#[cfg(windows)]
fn get_process_image_path(pid: u32) -> Option<String> {
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
            return None;
        }

        let mut buf = [0u16; 512];
        let mut size = buf.len() as u32;
        let res = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        let _ = CloseHandle(handle);

        if res != 0 && size > 0 {
            Some(String::from_utf16_lossy(&buf[..size as usize]).to_lowercase())
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn is_process_game(pid: u32) -> bool {
    if let Some(path) = get_process_image_path(pid) {
        let p = path.replace('/', "\\");
        (p.ends_with("league of legends.exe") || p.contains(r"\game\league of legends.exe"))
            && !p.contains("leagueclient")
    } else {
        false
    }
}

#[cfg(windows)]
fn is_process_league_client(pid: u32) -> bool {
    if let Some(path) = get_process_image_path(pid) {
        let p = path.replace('/', "\\");
        p.contains("leagueclient") || (p.contains("riotclient") && !p.contains(r"\game\"))
    } else {
        false
    }
}

impl WindowTracker {
    #[cfg(windows)]
    pub fn is_league_active() -> bool {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetAncestor, GetClassNameW, GetForegroundWindow, GetWindowTextW,
            GetWindowThreadProcessId, GA_ROOT,
        };

        // If the 3D match game client is active in the foreground, LCU client is NOT active
        if Self::is_game_active() {
            return false;
        }

        unsafe {
            let fg = GetForegroundWindow();
            if fg == HWND::default() {
                return false;
            }

            // 1. Direct Window Class Matching
            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(fg, &mut class_buf);
            let class_name = if len > 0 {
                String::from_utf16_lossy(&class_buf[..len as usize]).to_lowercase()
            } else {
                String::new()
            };

            // 0. HexBrain Companion or Overlay Window itself
            if class_name.contains("hexbrain") || class_name.contains("tauri") {
                return true;
            }

            if class_name.contains("rclient") {
                return true;
            }

            // 2. Ancestor Root Window Check (Penetrates CEF Child Windows e.g. Chrome_RenderWidgetHostHWND)
            let root = GetAncestor(fg, GA_ROOT);
            if root != HWND::default() && root != fg {
                let r_len = GetClassNameW(root, &mut class_buf);
                let root_class = if r_len > 0 {
                    String::from_utf16_lossy(&class_buf[..r_len as usize]).to_lowercase()
                } else {
                    String::new()
                };
                if root_class.contains("rclient") || root_class.contains("hexbrain") {
                    return true;
                }
            }

            // 3. PID & Process Name Matching
            let mut fg_pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            if fg_pid != 0 {
                let my_pid = std::process::id();
                if fg_pid == my_pid {
                    return true;
                }
                let client_pid = crate::vision::capture::ScreenCapture::get_league_client_pid();
                if client_pid != 0 && fg_pid == client_pid {
                    return true;
                }
                if is_process_league_client(fg_pid) {
                    return true;
                }
            }

            // 4. Safe Title matching guarded against third-party browsers/editors
            let is_browser_or_editor = class_name.contains("chrome")
                || class_name.contains("mozilla")
                || class_name.contains("edge")
                || class_name.contains("notepad")
                || class_name.contains("cabinetwclass");

            if !is_browser_or_editor && !class_name.contains("riotwindowclass") {
                let mut title_buf = [0u16; 256];
                let t_len = GetWindowTextW(fg, &mut title_buf);
                let title = if t_len > 0 {
                    String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase()
                } else {
                    String::new()
                };
                if (title == "league of legends" || title == "英雄联盟") && !title.contains("(tm)") {
                    return true;
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

            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(fg, &mut class_buf);
            let class_name = if len > 0 {
                String::from_utf16_lossy(&class_buf[..len as usize]).to_lowercase()
            } else {
                String::new()
            };

            // If it's explicitly the LCU client, it's not the 3D match game
            if class_name.contains("rclient") {
                return false;
            }

            // 1. Direct class check: RiotWindowClass is the unique class of League 3D game client
            if class_name.contains("riotwindowclass") {
                return true;
            }

            // 2. Ancestor root check
            let root = GetAncestor(fg, GA_ROOT);
            if root != HWND::default() && root != fg {
                let r_len = GetClassNameW(root, &mut class_buf);
                let root_class = if r_len > 0 {
                    String::from_utf16_lossy(&class_buf[..r_len as usize]).to_lowercase()
                } else {
                    String::new()
                };
                if root_class.contains("rclient") {
                    return false;
                }
                if root_class.contains("riotwindowclass") {
                    return true;
                }
            }

            // 3. Process image check & game PID check
            let mut fg_pid = 0u32;
            GetWindowThreadProcessId(fg, Some(&mut fg_pid));
            if fg_pid != 0 {
                let game_pid = crate::vision::capture::ScreenCapture::get_league_game_pid();
                if game_pid != 0 && fg_pid == game_pid {
                    return true;
                }
                if is_process_game(fg_pid) {
                    return true;
                }
            }

            // 4. Safe title fallback for full game title (rejecting browser/editor classes)
            let is_browser_or_editor = class_name.contains("chrome")
                || class_name.contains("mozilla")
                || class_name.contains("edge")
                || class_name.contains("notepad")
                || class_name.contains("cabinetwclass");

            if !is_browser_or_editor {
                let mut title_buf = [0u16; 256];
                let t_len = GetWindowTextW(fg, &mut title_buf);
                let title = if t_len > 0 {
                    String::from_utf16_lossy(&title_buf[..t_len as usize]).to_lowercase()
                } else {
                    String::new()
                };
                if title.contains("league of legends (tm) client") {
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
