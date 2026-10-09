use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static HOOK_HANDLE: Mutex<Option<usize>> = Mutex::new(None);
static CALLBACK_LEFT: Mutex<Option<Box<dyn Fn() + Send + 'static>>> = Mutex::new(None);
static CALLBACK_RIGHT: Mutex<Option<Box<dyn Fn() + Send + 'static>>> = Mutex::new(None);

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

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        #[cfg(windows)]
        {
            let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if tid != 0 {
                #[link(name = "user32")]
                extern "system" {
                    fn PostThreadMessageW(idthread: u32, msg: u32, wparam: usize, lparam: isize) -> i32;
                }
                unsafe {
                    // WM_QUIT = 0x0012
                    let _ = PostThreadMessageW(tid, 0x0012, 0, 0);
                }
            }
        }
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

    static LAST_ASYNC_LEFT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    static LAST_ASYNC_RIGHT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    // VK_OEM_4 = 0xDB ('[' and '【'), VK_OEM_6 = 0xDD (']' and '】')
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(25)); // 40Hz polling, 0.00% CPU

        unsafe {
            let left_state = GetAsyncKeyState(0xDB);
            if (left_state as u16 & 0x8000) != 0 {
                let now = GetTickCount64();
                let last = LAST_ASYNC_LEFT.load(Ordering::Relaxed);
                if now.saturating_sub(last) > 400 {
                    LAST_ASYNC_LEFT.store(now, Ordering::Relaxed);
                    tracing::info!("GetAsyncKeyState detected '[' / '【' pressed");
                    crate::spawn_task(async move {
                        if let Ok(guard) = CALLBACK_LEFT.lock() {
                            if let Some(ref cb) = *guard {
                                cb();
                            }
                        }
                    });
                }
            }

            let right_state = GetAsyncKeyState(0xDD);
            if (right_state as u16 & 0x8000) != 0 {
                let now = GetTickCount64();
                let last = LAST_ASYNC_RIGHT.load(Ordering::Relaxed);
                if now.saturating_sub(last) > 400 {
                    LAST_ASYNC_RIGHT.store(now, Ordering::Relaxed);
                    tracing::info!("GetAsyncKeyState detected ']' / '】' pressed");
                    crate::spawn_task(async move {
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
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentThreadId() -> u32;
        fn GetModuleHandleW(lpmodulename: *const u16) -> *mut core::ffi::c_void;
    }

    unsafe {
        let tid = GetCurrentThreadId();
        HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

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
        while running.load(Ordering::SeqCst) && GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            // Processing loop; WM_QUIT causes GetMessageW to return 0
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

    static LAST_TRIGGER_LEFT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    static LAST_TRIGGER_RIGHT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    #[repr(C)]
    struct KbdLlHookStruct {
        vk_code: u32,
        scan_code: u32,
        flags: u32,
        time: u32,
        extra_info: usize,
    }

    // HC_ACTION = 0, WM_KEYDOWN = 0x0100, WM_SYSKEYDOWN = 0x0104
    if code == 0 && (wparam == 0x0100 || wparam == 0x0104) && lparam != 0 {
        let kb = &*(lparam as *const KbdLlHookStruct);
        let vk_code = kb.vk_code;
        let scan_code = kb.scan_code & 0xFF;
        let is_extended = (kb.flags & 1) != 0;

        // Support both Virtual Key (VK_OEM_4 / VK_OEM_6) and Physical Scan Code (0x1A for '[', 0x1B for ']')
        // to ensure 100% trigger reliability regardless of Chinese/English IME, filtering out extended keypads
        let is_left_bracket = !is_extended && (vk_code == 0xDB || scan_code == 0x1A);
        let is_right_bracket = !is_extended && (vk_code == 0xDD || scan_code == 0x1B);

        if is_left_bracket {
            let now = GetTickCount64();
            let last = LAST_TRIGGER_LEFT.load(Ordering::Relaxed);
            if now.saturating_sub(last) > 400 {
                LAST_TRIGGER_LEFT.store(now, Ordering::Relaxed);
                crate::spawn_task(async move {
                    if let Ok(guard) = CALLBACK_LEFT.lock() {
                        if let Some(ref cb) = *guard {
                            cb();
                        }
                    }
                });
            }
        } else if is_right_bracket {
            let now = GetTickCount64();
            let last = LAST_TRIGGER_RIGHT.load(Ordering::Relaxed);
            if now.saturating_sub(last) > 400 {
                LAST_TRIGGER_RIGHT.store(now, Ordering::Relaxed);
                crate::spawn_task(async move {
                    if let Ok(guard) = CALLBACK_RIGHT.lock() {
                        if let Some(ref cb) = *guard {
                            cb();
                        }
                    }
                });
            }
        }
    }

    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}
