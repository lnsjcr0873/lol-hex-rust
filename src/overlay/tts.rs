use std::io::Write;
use std::sync::mpsc::{channel, Sender};
use std::sync::OnceLock;

static TTS_SENDER: OnceLock<Sender<String>> = OnceLock::new();

pub struct TextToSpeech;

impl TextToSpeech {
    pub fn speak(text: &str) {
        let clean_msg: String = text
            .chars()
            .filter(|c| !c.is_control() && *c != '"' && *c != '\'' && *c != '`')
            .collect();
        if clean_msg.trim().is_empty() {
            return;
        }

        let sender = TTS_SENDER.get_or_init(|| {
            let (tx, rx) = channel::<String>();
            #[cfg(windows)]
            std::thread::spawn(move || {
                use std::os::windows::process::CommandExt;
                use std::process::{Command, Stdio};

                let spawn_worker = || {
                    Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-NonInteractive",
                            "-ExecutionPolicy",
                            "Bypass",
                            "-Command",
                            "[Console]::InputEncoding = [System.Text.Encoding]::UTF8; \
                             $voice = New-Object -ComObject SAPI.SpVoice; \
                             while ($line = [Console]::In.ReadLine()) { \
                                 if (-not [string]::IsNullOrWhiteSpace($line)) { \
                                     try { $voice.Speak($line) } catch {} \
                                 } \
                             }",
                        ])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .creation_flags(0x08000000) // CREATE_NO_WINDOW
                        .spawn()
                };

                let mut child = spawn_worker();

                while let Ok(msg) = rx.recv() {
                    let mut restart = false;
                    if let Ok(ref mut proc) = child {
                        if let Some(ref mut stdin) = proc.stdin.as_mut() {
                            if writeln!(stdin, "{}", msg).is_err() || stdin.flush().is_err() {
                                restart = true;
                            }
                        } else {
                            restart = true;
                        }
                    } else {
                        restart = true;
                    }

                    if restart {
                        child = spawn_worker();
                        if let Ok(ref mut proc) = child {
                            if let Some(ref mut stdin) = proc.stdin.as_mut() {
                                let _ = writeln!(stdin, "{}", msg);
                                let _ = stdin.flush();
                            }
                        }
                    }
                }

                if let Ok(mut proc) = child {
                    let _ = proc.kill();
                }
            });
            tx
        });

        let _ = sender.send(clean_msg);
    }
}
