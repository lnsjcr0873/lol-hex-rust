use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        #[cfg(windows)]
        std::thread::spawn(move || {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT};
            use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

            unsafe {
                // HOTKEY_ID_1: '[' (VK_OEM_4 = 0xDB)
                // HOTKEY_ID_2: ']' (VK_OEM_6 = 0xDD)
                let _ = RegisterHotKey(HWND(0), 1, MOD_NOREPEAT, 0xDB);
                let _ = RegisterHotKey(HWND(0), 2, MOD_NOREPEAT, 0xDD);

                let mut msg = MSG::default();
                while running.load(Ordering::SeqCst) && GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    if msg.message == WM_HOTKEY {
                        if msg.wParam.0 == 1 {
                            on_hotkey_bracket_left();
                        } else if msg.wParam.0 == 2 {
                            on_hotkey_bracket_right();
                        }
                    }
                }

                let _ = UnregisterHotKey(HWND(0), 1);
                let _ = UnregisterHotKey(HWND(0), 2);
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
