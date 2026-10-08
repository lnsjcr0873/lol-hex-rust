use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct LcuCredentials {
    pub port: u16,
    pub auth_token: String,
}

pub struct LcuConnector;

const COMMON_LOL_PATHS: &[&str] = &[
    "C:/Riot Games/League of Legends",
    "D:/Riot Games/League of Legends",
    "E:/Riot Games/League of Legends",
    "F:/Riot Games/League of Legends",
    "D:/WeGameApps/英雄联盟",
    "E:/WeGameApps/英雄联盟",
    "F:/WeGameApps/英雄联盟",
    "C:/WeGameApps/英雄联盟",
    "D:/腾讯游戏/英雄联盟",
    "E:/腾讯游戏/英雄联盟",
];

impl LcuConnector {
    pub fn sniff_credentials() -> Option<LcuCredentials> {
        // Method 1: Check lockfile in known install paths
        for dir in COMMON_LOL_PATHS {
            let lockfile = Path::new(dir).join("lockfile");
            if lockfile.exists() {
                if let Ok(content) = fs::read_to_string(&lockfile) {
                    if let Some(creds) = Self::parse_lockfile(&content) {
                        return Some(creds);
                    }
                }
            }
        }

        // Method 2: On Windows, use process snapshot
        #[cfg(windows)]
        {
            if let Some(creds) = Self::sniff_from_process_win32() {
                return Some(creds);
            }
        }

        None
    }

    pub fn parse_lockfile(content: &str) -> Option<LcuCredentials> {
        let parts: Vec<&str> = content.trim().split(':').collect();
        if parts.len() >= 5 {
            if let Ok(port) = parts[2].parse::<u16>() {
                return Some(LcuCredentials {
                    port,
                    auth_token: parts[3].to_string(),
                });
            }
        }
        None
    }

    pub fn parse_commandline(cmd: &str) -> Option<LcuCredentials> {
        let port_key = "--app-port=";
        let token_key = "--remoting-auth-token=";

        let port: u16 = if let Some(idx) = cmd.find(port_key) {
            let rest = &cmd[idx + port_key.len()..];
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            num_str.parse().ok()?
        } else {
            return None;
        };

        let auth_token: String = if let Some(idx) = cmd.find(token_key) {
            let rest = &cmd[idx + token_key.len()..];
            let token_str: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
                .collect();
            if token_str.is_empty() {
                return None;
            }
            token_str
        } else {
            return None;
        };

        Some(LcuCredentials { port, auth_token })
    }

    #[cfg(windows)]
    fn sniff_from_process_win32() -> Option<LcuCredentials> {
        use std::path::PathBuf;

        #[repr(C)]
        struct ProcessEntry32W {
            dw_size: u32,
            cnt_usage: u32,
            th32_process_id: u32,
            th32_default_heap_id: usize,
            th32_module_id: u32,
            cnt_threads: u32,
            th32_parent_process_id: u32,
            pc_pri_class_base: i32,
            dw_flags: u32,
            sz_exe_file: [u16; 260],
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn CreateToolhelp32Snapshot(dwflags: u32, th32processid: u32) -> *mut core::ffi::c_void;
            fn Process32FirstW(hsnapshot: *mut core::ffi::c_void, lppe: *mut ProcessEntry32W) -> i32;
            fn Process32NextW(hsnapshot: *mut core::ffi::c_void, lppe: *mut ProcessEntry32W) -> i32;
            fn OpenProcess(dwdesiredaccess: u32, binherithandle: i32, dwprocessid: u32) -> *mut core::ffi::c_void;
            fn CloseHandle(hobject: *mut core::ffi::c_void) -> i32;
            fn QueryFullProcessImageNameW(hprocess: *mut core::ffi::c_void, dwflags: u32, lpexename: *mut u16, lpsize: *mut u32) -> i32;
        }

        unsafe {
            // TH32CS_SNAPPROCESS = 0x00000002
            let snapshot = CreateToolhelp32Snapshot(0x00000002, 0);
            if snapshot.is_null() || snapshot == (-1isize as *mut core::ffi::c_void) {
                return None;
            }

            let mut entry = ProcessEntry32W {
                dw_size: std::mem::size_of::<ProcessEntry32W>() as u32,
                cnt_usage: 0,
                th32_process_id: 0,
                th32_default_heap_id: 0,
                th32_module_id: 0,
                cnt_threads: 0,
                th32_parent_process_id: 0,
                pc_pri_class_base: 0,
                dw_flags: 0,
                sz_exe_file: [0u16; 260],
            };

            let mut target_pid = None;
            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(entry.sz_exe_file.len());
                    let exe_name = String::from_utf16_lossy(&entry.sz_exe_file[..len]).to_lowercase();
                    if exe_name == "leagueclientux.exe" || exe_name == "leagueclient.exe" {
                        target_pid = Some(entry.th32_process_id);
                        break;
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);

            let pid = target_pid?;
            // PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
            let proc_handle = OpenProcess(0x1000, 0, pid);
            if proc_handle.is_null() {
                return None;
            }

            let mut path_buf = [0u16; 1024];
            let mut size = path_buf.len() as u32;
            let q_res = QueryFullProcessImageNameW(proc_handle, 0, path_buf.as_mut_ptr(), &mut size);
            let _ = CloseHandle(proc_handle);

            if q_res != 0 && size > 0 {
                let exe_path_str = String::from_utf16_lossy(&path_buf[..size as usize]);
                let exe_path = PathBuf::from(exe_path_str);
                if let Some(dir) = exe_path.parent() {
                    let lockfile = dir.join("lockfile");
                    if lockfile.exists() {
                        if let Ok(content) = fs::read_to_string(&lockfile) {
                            if let Some(creds) = Self::parse_lockfile(&content) {
                                return Some(creds);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
