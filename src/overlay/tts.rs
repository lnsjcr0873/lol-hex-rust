pub struct TextToSpeech;

impl TextToSpeech {
    pub fn speak(text: &str) {
        let message = text.to_string();
        #[cfg(windows)]
        std::thread::spawn(move || {
            // Using PowerShell SAPI voice for lightweight, instant speech without heavy COM dependencies
            let script = format!(
                "Add-Type -AssemblyName System.speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
                message.replace('\'', "''")
            );
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                .spawn();
        });
        #[cfg(not(windows))]
        {
            let _ = message;
        }
    }
}
