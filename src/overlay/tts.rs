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
                // Background dedicated queue thread: serializes voice without CPU spikes or overlaps
                while let Ok(msg) = rx.recv() {
                    // Ultra-lightweight SAPI COM invocation without heavy PowerShell process launch
                    let script = format!(
                        "CreateObject(\"SAPI.SpVoice\").Speak \"{}\"",
                        msg.replace('"', "\"\"")
                    );
                    use std::os::windows::process::CommandExt;
                    let _ = std::process::Command::new("mshta")
                        .args([format!("vbscript:Execute(\"{}\")(close)", script)])
                        .creation_flags(0x08000000)
                        .output();
                }
            });
            tx
        });

        let _ = sender.send(clean_msg);
    }
}
