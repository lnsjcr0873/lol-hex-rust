pub struct TextToSpeech;

impl TextToSpeech {
    pub fn speak(text: &str) {
        let clean_msg: String = text
            .chars()
            .filter(|c| !c.is_control() && *c != '$' && *c != '`')
            .collect();
        if clean_msg.trim().is_empty() {
            return;
        }

        #[cfg(windows)]
        std::thread::spawn(move || {
            let script = format!(
                "Add-Type -AssemblyName System.speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
                clean_msg.replace('\'', "''")
            );
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                .spawn();
        });
        #[cfg(not(windows))]
        {
            let _ = clean_msg;
        }
    }
}
