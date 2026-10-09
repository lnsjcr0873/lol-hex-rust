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
            let mut left_guard = CALLBACK_LEFT.lock().unwrap();
            *left_guard = Some(Box::new(on_hotkey_bracket_left));
            let mut right_guard = CALLBACK_RIGHT.lock().unwrap();
            *right_guard = Some(Box::new(on_hotkey_bracket_right));
        }

        #[cfg(windows)]
        {
            let running = self.running.clone();
            std::thread::spawn(move || {
                run_keyboard_hook_loop(running);
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
    }

    unsafe {
        let tid = GetCurrentThreadId();
        HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

        // WH_KEYBOARD_LL = 13
        let hook = SetWindowsHookExW(13, Some(low_level_keyboard_proc), std::ptr::null_mut(), 0);
        if !hook.is_null() {
            if let Ok(mut guard) = HOOK_HANDLE.lock() {
                *guard = Some(hook as usize);
            }
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

    // HC_ACTION = 0, WM_KEYDOWN = 0x0100, WM_SYSKEYDOWN = 0x0104
    if code == 0 && (wparam == 0x0100 || wparam == 0x0104) && lparam != 0 {
        let vk_code = *(lparam as *const u32);
        // VK_OEM_4 = 0xDB ('['), VK_OEM_6 = 0xDD (']')
        if vk_code == 0xDB {
            crate::spawn_task(async move {
                if crate::overlay::WindowTracker::is_league_active() {
                    if let Ok(guard) = CALLBACK_LEFT.lock() {
                        if let Some(ref cb) = *guard {
                            cb();
                        }
                    }
                }
            });
        } else if vk_code == 0xDD {
            crate::spawn_task(async move {
                if crate::overlay::WindowTracker::is_league_active() {
                    if let Ok(guard) = CALLBACK_RIGHT.lock() {
                        if let Some(ref cb) = *guard {
                            cb();
                        }
                    }
                }
            });
        }
    }

    let hook_val = HOOK_HANDLE.lock().ok().and_then(|g| *g).unwrap_or(0);
    CallNextHookEx(hook_val as *mut core::ffi::c_void, code, wparam, lparam)
}
