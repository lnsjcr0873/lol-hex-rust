use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static HOOK_HANDLE: Mutex<Option<usize>> = Mutex::new(None);
static CALLBACK_LEFT: Mutex<Option<Box<dyn Fn() + Send + 'static>>> = Mutex::new(None);
static CALLBACK_RIGHT: Mutex<Option<Box<dyn Fn() + Send + 'static>>> = Mutex::new(None);
static SHARED_LAST_TRIGGER_LEFT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SHARED_LAST_TRIGGER_RIGHT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static IS_KEY_DOWN_LEFT: AtomicBool = AtomicBool::new(false);
static IS_KEY_DOWN_RIGHT: AtomicBool = AtomicBool::new(false);

fn check_and_mark_trigger(target: &std::sync::atomic::AtomicU64, now: u64, debounce_ms: u64) -> bool {
    let mut current = target.load(Ordering::Acquire);
    loop {
        if now.saturating_sub(current) <= debounce_ms {
            return false;
        }
        match target.compare_exchange_weak(current, now, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => return true,
            Err(actual) => current = actual,
        }
    }
}


#[cfg(windows)]
fn is_text_input_active() -> bool {
    #[repr(C)]
    struct GUITHREADINFO {
        cb_size: u32,
        flags: u32,
        hwnd_active: *mut core::ffi::c_void,
        hwnd_focus: *mut core::ffi::c_void,
        hwnd_capture: *mut core::ffi::c_void,
        hwnd_menu_owner: *mut core::ffi::c_void,
        hwnd_move_size: *mut core::ffi::c_void,
        hwnd_caret: *mut core::ffi::c_void,
        rc_caret: [i32; 4],
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut core::ffi::c_void;
        fn GetWindowThreadProcessId(hwnd: *mut core::ffi::c_void, lpprocessid: *mut u32) -> u32;
        fn GetGUIThreadInfo(idthread: u32, pgui: *mut GUITHREADINFO) -> i32;
    }

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }
        let tid = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        if tid == 0 {
            return false;
        }
        let mut info: GUITHREADINFO = std::mem::zeroed();
        info.cb_size = std::mem::size_of::<GUITHREADINFO>() as u32;
        if GetGUIThreadInfo(tid, &mut info) != 0 {
            // GUI_CARETBLINKING = 0x00000001: The caret's blink state is visible (active typing)
            let caret_blinking = (info.flags & 1) != 0;
            let has_caret = !info.hwnd_caret.is_null();
            return caret_blinking || has_caret;
        }
        false
    }
}

#[cfg(windows)]
fn is_allowed_foreground_for_hotkey() -> bool {
    // If the user is currently typing in an in-game chatbox or text field (active caret), bypass hotkey overlay
    if is_text_input_active() {
        return false;
    }
    super::window_tracker::WindowTracker::is_game_active()
        || super::window_tracker::WindowTracker::is_league_active()
}

#[cfg(not(windows))]
fn is_allowed_foreground_for_hotkey() -> bool {
    false
}

pub struct HotkeyManager {
    running: Arc<AtomicBool>,
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start<F1, F2>(&self, on_hotkey_bracket_left: F1, on_hotkey_bracket_right: F2)
    where
        F1: Fn() + Send + 'static,
        F2: Fn() + Send + 'static,
    {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        {
            let mut left_guard = CALLBACK_LEFT.lock().unwrap_or_else(|e| e.into_inner());
            *left_guard = Some(Box::new(on_hotkey_bracket_left));
            let mut right_guard = CALLBACK_RIGHT.lock().unwrap_or_else(|e| e.into_inner());
            *right_guard = Some(Box::new(on_hotkey_bracket_right));
        }

        #[cfg(windows)]
        {
            let running = self.running.clone();
            std::thread::spawn(move || {
                run_keyboard_hook_loop(running);
            });

            let running_async = self.running.clone();
            std::thread::spawn(move || {
                run_async_keystate_polling_loop(running_async);
            });
        }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        #[cfg(windows)]
        {
            let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if tid != 0 {
                #[link(name = "user32")]
                extern "system" {
                    #[allow(dead_code)]
                    fn PostThreadMessageW(idthread: u32, msg: u32, wparam: usize, lparam: isize) -> i32;
                }
                unsafe {
                    // Post WM_QUIT (0x0012), retrying if the message queue is initializing
                    for _ in 0..5 {
                        if PostThreadMessageW(tid, 0x0012, 0, 0) != 0 {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
            }
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
fn run_async_keystate_polling_loop(running: Arc<AtomicBool>) {
    #[link(name = "user32")]
    extern "system" {
        fn GetAsyncKeyState(vkey: i32) -> i16;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetTickCount64() -> u64;
    }

    let mut was_down_left = false;
    let mut was_down_right = false;

    // VK_OEM_4 = 0xDB ('[' and '【'), VK_OEM_6 = 0xDD (']' and '】')
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(30));


        if !is_allowed_foreground_for_hotkey() {
            unsafe {
                if (GetAsyncKeyState(0xDB) as u16 & 0x8000) == 0 {
                    was_down_left = false;
                    IS_KEY_DOWN_LEFT.store(false, Ordering::SeqCst);
                }
                if (GetAsyncKeyState(0xDD) as u16 & 0x8000) == 0 {
                    was_down_right = false;
                    IS_KEY_DOWN_RIGHT.store(false, Ordering::SeqCst);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
            continue;
        }

        unsafe {
            let left_state = GetAsyncKeyState(0xDB);
            let is_down_left = (left_state as u16 & 0x8000) != 0;
            if is_down_left && !was_down_left {
                was_down_left = true;
                let now = GetTickCount64();
                if check_and_mark_trigger(&SHARED_LAST_TRIGGER_LEFT, now, 300) {
                    tracing::info!("GetAsyncKeyState fallback detected '[' / '【' pressed");
                    crate::spawn_task(async move {
                        if let Ok(guard) = CALLBACK_LEFT.lock() {
                            if let Some(ref cb) = *guard {
                                cb();
                            }
                        }
                    });
                }
            } else if !is_down_left {
                was_down_left = false;
                IS_KEY_DOWN_LEFT.store(false, Ordering::SeqCst);
            }

            let right_state = GetAsyncKeyState(0xDD);
            let is_down_right = (right_state as u16 & 0x8000) != 0;
            if is_down_right && !was_down_right {
                was_down_right = true;
                let now = GetTickCount64();
                if check_and_mark_trigger(&SHARED_LAST_TRIGGER_RIGHT, now, 300) {
                    tracing::info!("GetAsyncKeyState fallback detected ']' / '】' pressed");
                    crate::spawn_task(async move {
                        if let Ok(guard) = CALLBACK_RIGHT.lock() {
                            if let Some(ref cb) = *guard {
                                cb();
                            }
                        }
                    });
                }
            } else if !is_down_right {
                was_down_right = false;
                IS_KEY_DOWN_RIGHT.store(false, Ordering::SeqCst);
            }
        }
    }
}

#[cfg(windows)]
fn run_keyboard_hook_loop(running: Arc<AtomicBool>) {
    #[repr(C)]
    struct Msg {
        hwnd: *mut core::ffi::c_void,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowsHookExW(
            idhook: i32,
            lpfn: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
            hmod: *mut core::ffi::c_void,
            dwthreadid: u32,
        ) -> *mut core::ffi::c_void;
        fn UnhookWindowsHookEx(hhk: *mut core::ffi::c_void) -> i32;
        fn GetMessageW(
            lpmsg: *mut Msg,
            hwnd: *mut core::ffi::c_void,
            wmsgfiltermin: u32,
            wmsgfiltermax: u32,
        ) -> i32;
        fn PeekMessageW(
            lpmsg: *mut Msg,
            hwnd: *mut core::ffi::c_void,
            wmsgfiltermin: u32,
            wmsgfiltermax: u32,
            wremovemsg: u32,
        ) -> i32;
        fn SetTimer(
            hwnd: *mut core::ffi::c_void,
            nidevent: usize,
            uelapse: u32,
            lptimerfunc: Option<unsafe extern "system" fn(*mut core::ffi::c_void, u32, usize, u32)>,
        ) -> usize;
        fn KillTimer(hwnd: *mut core::ffi::c_void, uidevent: usize) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentThreadId() -> u32;
        fn GetModuleHandleW(lpmodulename: *const u16) -> *mut core::ffi::c_void;
    }

    unsafe {
        let tid = GetCurrentThreadId();

        // 1. Force message queue creation before storing tid so PostThreadMessage never fails with ERROR_INVALID_THREAD_ID
        let mut dummy = std::mem::zeroed::<Msg>();
        let _ = PeekMessageW(&mut dummy, std::ptr::null_mut(), 0, 0, 0);

        HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

        // 2. Install a 200ms thread timer so GetMessageW wakes up periodically and never hangs if WM_QUIT is missed
        let timer_id = SetTimer(std::ptr::null_mut(), 0, 200, None);

        let hmod = GetModuleHandleW(std::ptr::null());
        // WH_KEYBOARD_LL = 13
        let hook = SetWindowsHookExW(13, Some(low_level_keyboard_proc), hmod, 0);
        if !hook.is_null() {
            tracing::info!("Low-level keyboard hook registered successfully");
            if let Ok(mut guard) = HOOK_HANDLE.lock() {
                *guard = Some(hook as usize);
            }
        } else {
            tracing::warn!("SetWindowsHookExW returned NULL, relying on GetAsyncKeyState hardware polling fallback");
        }

        let mut msg = std::mem::zeroed::<Msg>();
        while running.load(Ordering::SeqCst) {
            let res = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if res <= 0 {
                // WM_QUIT (0) or error (-1)
                break;
            }
        }

        if timer_id != 0 {
            let _ = KillTimer(std::ptr::null_mut(), timer_id);
        }

        if let Ok(mut guard) = HOOK_HANDLE.lock() {
            if let Some(h) = guard.take() {
                let _ = UnhookWindowsHookEx(h as *mut core::ffi::c_void);
            }
        }
        HOOK_THREAD_ID.store(0, Ordering::SeqCst);
    }
}

#[cfg(windows)]
unsafe extern "system" fn low_level_keyboard_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    #[link(name = "user32")]
    extern "system" {
        fn CallNextHookEx(hhk: *mut core::ffi::c_void, ncode: i32, wparam: usize, lparam: isize) -> isize;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetTickCount64() -> u64;
    }

    #[repr(C)]
    struct KbdLlHookStruct {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        extra_info: usize,
    }

    // HC_ACTION = 0
    if code == 0 && lparam != 0 {
        let kb = &*(lparam as *const KbdLlHookStruct);
        let vk_code = kb.vk_code;
        let scan_code = kb.scan_code & 0xFF;
        let is_extended = (kb.flags & 1) != 0;

        // Support both Virtual Key (VK_OEM_4 / VK_OEM_6) and Physical Scan Code (0x1A for '[', 0x1B for ']')
        // to ensure 100% trigger reliability regardless of Chinese/English IME, filtering out extended keypads
        let is_left_bracket = !is_extended && (vk_code == 0xDB || scan_code == 0x1A);
        let is_right_bracket = !is_extended && (vk_code == 0xDD || scan_code == 0x1B);

        // WM_KEYUP = 0x0101, WM_SYSKEYUP = 0x0105
        if wparam == 0x0101 || wparam == 0x0105 {
            if is_left_bracket {
                IS_KEY_DOWN_LEFT.store(false, Ordering::SeqCst);
            } else if is_right_bracket {
                IS_KEY_DOWN_RIGHT.store(false, Ordering::SeqCst);
            }
        } else if wparam == 0x0100 || wparam == 0x0104 {
            // WM_KEYDOWN = 0x0100, WM_SYSKEYDOWN = 0x0104
            // ONLY trigger on the rising edge (false -> true). Ignore all Windows auto-repeat keydown events!
            // Defer foreground window inspection to async task to keep the low-level hook callback under 1 microsecond,
            // preventing Windows from silently unhooking on system load or anti-virus OpenProcess hooks.
            if is_left_bracket {
                if !IS_KEY_DOWN_LEFT.swap(true, Ordering::SeqCst) {
                    let now = GetTickCount64();
                    if check_and_mark_trigger(&SHARED_LAST_TRIGGER_LEFT, now, 300) {
                        crate::spawn_task(async move {
                            if !is_allowed_foreground_for_hotkey() {
                                return;
                            }
                            if let Ok(guard) = CALLBACK_LEFT.lock() {
                                if let Some(ref cb) = *guard {
                                    cb();
                                }
                            }
                        });
                    }
                }
            } else if is_right_bracket {
                if !IS_KEY_DOWN_RIGHT.swap(true, Ordering::SeqCst) {
                    let now = GetTickCount64();
                    if check_and_mark_trigger(&SHARED_LAST_TRIGGER_RIGHT, now, 300) {
                        crate::spawn_task(async move {
                            if !is_allowed_foreground_for_hotkey() {
                                return;
                            }
                            if let Ok(guard) = CALLBACK_RIGHT.lock() {
                                if let Some(ref cb) = *guard {
                                    cb();
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}
