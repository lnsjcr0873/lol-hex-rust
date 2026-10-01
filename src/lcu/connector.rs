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
        use std::process::Command;
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_Process -Filter \"name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine",
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let cmdline = String::from_utf8_lossy(&output.stdout);
            return Self::parse_commandline(&cmdline);
        }
        None
    }
}
