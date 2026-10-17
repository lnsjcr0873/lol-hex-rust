use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

static LAST_SNIFF_ATTEMPT: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub struct LcuCredentials {
    pub port: u16,
    pub auth_token: String,
    pub protocol: String,
}

impl Default for LcuCredentials {
    fn default() -> Self {
        Self {
            port: 0,
            auth_token: String::new(),
            protocol: "https".to_string(),
        }
    }
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
    pub fn reset_cooldown() {
        if let Ok(mut last) = LAST_SNIFF_ATTEMPT.lock() {
            *last = None;
        }
    }

    pub fn sniff_credentials() -> Option<LcuCredentials> {
        // Enforce a 3-second cooldown on failed sniff attempts to prevent high CPU / process storm when client is closed
        {
            if let Ok(mut last) = LAST_SNIFF_ATTEMPT.lock() {
                if let Some(t) = *last {
                    if t.elapsed() < std::time::Duration::from_secs(3) {
                        return None;
                    }
                }
                *last = Some(Instant::now());
            }
        }

        // Priority 1: Native Win32 process lockfile sniffing (< 1ms, 0 subprocess overhead, 100% Win11 24H2 compatible)
        #[cfg(windows)]
        {
            if let Some(creds) = Self::sniff_from_process_win32() {
                return Some(creds);
            }
            // If on Windows, sniff_from_process_win32 already checked whether League processes exist.
            // If neither LeagueClientUx.exe nor LeagueClient.exe exists, it returned None without invoking WMIC/PowerShell.
            None
        }

        // Priority 2: Check lockfile in known install paths (non-Windows fallback)
        #[cfg(not(windows))]
        {
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
            None
        }
    }

    /// Async helper that executes credential sniffing on a blocking thread pool
    pub async fn sniff_credentials_async() -> Option<LcuCredentials> {
        tokio::task::spawn_blocking(Self::sniff_credentials).await.ok().flatten()
    }

    pub fn parse_lockfile(content: &str) -> Option<LcuCredentials> {
        let parts: Vec<&str> = content.trim().split(':').collect();
        if parts.len() >= 5 {
            if let Ok(port) = parts[2].parse::<u16>() {
                let token = parts[3].trim();
                let proto = parts[4].trim().to_lowercase();
                if port > 0 && !token.is_empty() {
                    return Some(LcuCredentials {
                        port,
                        auth_token: token.to_string(),
                        protocol: if proto == "http" { "http".to_string() } else { "https".to_string() },
                    });
                }
            }
        }
        None
    }

    pub fn parse_commandline(cmd: &str) -> Option<LcuCredentials> {
        let port_key = "--app-port=";
        let token_key = "--remoting-auth-token=";

        let port: u16 = if let Some(idx) = cmd.find(port_key) {
            let rest = &cmd[idx + port_key.len()..];
            let rest = rest.trim_start_matches(|c| c == '"' || c == '\'');
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            let p: u16 = num_str.parse().ok()?;
            if p == 0 {
                return None;
            }
            p
        } else {
            return None;
        };

        let auth_token: String = if let Some(idx) = cmd.find(token_key) {
            let rest = &cmd[idx + token_key.len()..];
            let rest = rest.trim_start_matches(|c| c == '"' || c == '\'');
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

        Some(LcuCredentials {
            port,
            auth_token,
            protocol: "https".to_string(),
        })
    }

    /// Extracts region and platform ID metadata from command line arguments.
    pub fn parse_commandline_metadata(cmd: &str) -> (Option<String>, Option<String>) {
        let extract = |key: &str| -> Option<String> {
            if let Some(idx) = cmd.find(key) {
                let rest = &cmd[idx + key.len()..];
                let rest = rest.trim_start_matches(|c| c == '"' || c == '\'');
                let val: String = rest
                    .chars()
                    .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
                    .collect();
                if !val.is_empty() {
                    return Some(val);
                }
            }
            None
        };

        let region = extract("--region=");
        let rso_platform_id = extract("--rso-platform-id=");
        (region, rso_platform_id)
    }

    #[cfg(windows)]
    pub fn sniff_from_commandline_win32() -> Option<LcuCredentials> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        // 1. Primary: PowerShell CIM (standard on Windows 10 & Windows 11 24H2+, where wmic is removed)
        let ps_cmd = "Get-CimInstance Win32_Process -Filter \"Name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine";
        if let Ok(output) = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-NonInteractive", "-Command", ps_cmd])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Some(creds) = Self::parse_commandline(line) {
                        tracing::info!("Successfully sniffed LCU credentials via PowerShell CIM: port {}", creds.port);
                        return Some(creds);
                    }
                }
                if let Some(creds) = Self::parse_commandline(&text) {
                    tracing::info!("Successfully sniffed LCU credentials via PowerShell CIM (fallback): port {}", creds.port);
                    return Some(creds);
                }
            }
        }

        // 2. Fallback: wmic (legacy fallback for older Windows systems if wmic.exe exists)
        if let Ok(output) = std::process::Command::new("wmic")
            .args(&["process", "where", "name='LeagueClientUx.exe'", "get", "commandline"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Some(creds) = Self::parse_commandline(line) {
                        tracing::info!("Successfully sniffed LCU credentials via WMIC: port {}", creds.port);
                        return Some(creds);
                    }
                }
                if let Some(creds) = Self::parse_commandline(&text) {
                    tracing::info!("Successfully sniffed LCU credentials via WMIC (fallback): port {}", creds.port);
                    return Some(creds);
                }
            }
        }

        None
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

            let mut ux_pids = Vec::new();
            let mut client_pids = Vec::new();

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(entry.sz_exe_file.len());
                    let exe_name = String::from_utf16_lossy(&entry.sz_exe_file[..len]).to_lowercase();
                    if exe_name == "leagueclientux.exe" {
                        ux_pids.push(entry.th32_process_id);
                    } else if exe_name == "leagueclient.exe" {
                        client_pids.push(entry.th32_process_id);
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);

            // Check LeagueClientUx.exe instances first, then LeagueClient.exe
            let mut candidate_pids = ux_pids;
            candidate_pids.extend(client_pids);

            // If neither LeagueClientUx.exe nor LeagueClient.exe PID is found in the snapshot,
            // return None immediately. Do NOT fall back to sniff_from_commandline_win32()!
            if candidate_pids.is_empty() {
                return None;
            }

            for pid in candidate_pids {
                // PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
                let proc_handle = OpenProcess(0x1000, 0, pid);
                if proc_handle.is_null() {
                    continue;
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

            // Only if League processes were confirmed to exist in the snapshot,
            // but the lockfile could not be read directly from their directory,
            // fall back to command line sniffing:
            if let Some(creds) = Self::sniff_from_commandline_win32() {
                return Some(creds);
            }

            // Also check known install paths if process is confirmed running
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
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_credentials_cooldown() {
        LcuConnector::reset_cooldown();
        let _ = LcuConnector::sniff_credentials();
        let second = LcuConnector::sniff_credentials();
        assert!(second.is_none());
        LcuConnector::reset_cooldown();
    }

    #[test]
    fn test_parse_commandline_unquoted() {
        let cmd = r#""LeagueClientUx.exe" "--app-port=51842" "--remoting-auth-token=test_token_123" "--region=TENCENT""#;
        let creds = LcuConnector::parse_commandline(cmd).expect("Should parse unquoted/quoted commandline");
        assert_eq!(creds.port, 51842);
        assert_eq!(creds.auth_token, "test_token_123");
    }

    #[test]
    fn test_parse_commandline_quoted_values() {
        let cmd = r#"LeagueClientUx.exe --app-port="52000" --remoting-auth-token="super-secret-auth" --rso-platform-id="HN10""#;
        let creds = LcuConnector::parse_commandline(cmd).expect("Should parse quoted values");
        assert_eq!(creds.port, 52000);
        assert_eq!(creds.auth_token, "super-secret-auth");
    }

    #[test]
    fn test_parse_commandline_single_quotes() {
        let cmd = "LeagueClientUx.exe --app-port='49152' --remoting-auth-token='single_quoted_tok'";
        let creds = LcuConnector::parse_commandline(cmd).expect("Should parse single quoted values");
        assert_eq!(creds.port, 49152);
        assert_eq!(creds.auth_token, "single_quoted_tok");
    }

    #[test]
    fn test_parse_commandline_metadata() {
        let cmd = r#"LeagueClientUx.exe --region="TENCENT" --rso-platform-id="HN10" --app-port=1234"#;
        let (region, rso_platform_id) = LcuConnector::parse_commandline_metadata(cmd);
        assert_eq!(region.as_deref(), Some("TENCENT"));
        assert_eq!(rso_platform_id.as_deref(), Some("HN10"));
    }

    #[test]
    fn test_parse_lockfile() {
        let content = "LeagueClient:12345:51842:secret_token:https\n";
        let creds = LcuConnector::parse_lockfile(content).expect("Should parse lockfile");
        assert_eq!(creds.port, 51842);
        assert_eq!(creds.auth_token, "secret_token");
    }
}
