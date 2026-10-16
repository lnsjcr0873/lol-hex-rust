use crate::data::loader::Database;
use strsim::jaro_winkler;

pub struct OcrScanner;

impl OcrScanner {
    pub fn clean_chinese_text(text: &str) -> String {
        // Strip spaces between Chinese characters
        let mut cleaned = String::new();
        let chars: Vec<char> = text.chars().collect();
        for i in 0..chars.len() {
            if chars[i].is_whitespace() {
                let prev_is_cjk = i > 0 && Self::is_cjk(chars[i - 1]);
                let next_is_cjk = i + 1 < chars.len() && Self::is_cjk(chars[i + 1]);
                if prev_is_cjk && next_is_cjk {
                    continue; // Skip whitespace between CJK
                }
            }
            cleaned.push(chars[i]);
        }
        cleaned.trim().to_string()
    }

    fn is_cjk(c: char) -> bool {
        matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF)
    }

    pub fn is_stat_anvil_cards(lines: &[String]) -> bool {
        let full_text = lines.join(" ").to_lowercase();
        let keywords = [
            "锻造器", "属性卡", "属性增强", "提供属性", "金币属性", "属性铁砧", "铁砧",
            "stat anvil", "anvil", "stat shard", "shard anvil",
        ];
        for kw in keywords {
            if full_text.contains(kw) {
                return true;
            }
        }
        false
    }

    pub fn is_punctuation(c: char) -> bool {
        c.is_whitespace()
            || matches!(
                c,
                ':' | '：'
                    | '-'
                    | '!'
                    | '！'
                    | ','
                    | '，'
                    | '。'
                    | '.'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '【'
                    | '】'
                    | '"'
                    | '\''
                    | '`'
                    | '“'
                    | '”'
                    | '、'
                    | '·'
                    | '《'
                    | '》'
                    | '<'
                    | '>'
            )
    }

    pub fn match_augments_from_texts(db: &Database, lines: &[String]) -> Vec<String> {
        let mut matched_ids = Vec::new();

        for line in lines {
            let cleaned = Self::clean_chinese_text(line);
            let norm: String = cleaned.chars().filter(|c| !Self::is_punctuation(*c)).collect();
            if norm.is_empty() {
                continue;
            }

            // 1. Exact match in normalized dict
            if let Some(id) = db.norm_name_to_augment_id.get(&norm) {
                if !matched_ids.contains(id) {
                    matched_ids.push(id.clone());
                    continue;
                }
            }

            // 2. Substring match with Longest Match Principle to prevent random HashMap iteration from picking short names
            let mut best_sub_match: Option<(&String, usize)> = None;
            for (aug_norm, aid) in &db.norm_name_to_augment_id {
                let match_len = aug_norm.chars().count();
                if match_len >= 2 && norm.contains(aug_norm) {
                    if !matched_ids.contains(aid) {
                        if let Some((best_aid, best_len)) = best_sub_match {
                            if match_len > best_len || (match_len == best_len && aid < best_aid) {
                                best_sub_match = Some((aid, match_len));
                            }
                        } else {
                            best_sub_match = Some((aid, match_len));
                        }
                    }
                }
            }
            if let Some((aid, _)) = best_sub_match {
                matched_ids.push(aid.clone());
                continue;
            }

            // 3. Fuzzy match with jaro_winkler (bounded by character length delta <= 3)
            let mut best_id: Option<String> = None;
            let mut best_score = 0.0;
            let norm_char_count = norm.chars().count() as i32;

            for (aug_norm, aid) in &db.norm_name_to_augment_id {
                let aug_char_count = aug_norm.chars().count() as i32;
                let len_diff = (norm_char_count - aug_char_count).abs();
                if len_diff <= 3 {
                    let score = jaro_winkler(&norm, aug_norm);
                    if score > 0.82 {
                        let is_better = score > best_score + 1e-6
                            || ((score - best_score).abs() <= 1e-6 && best_id.as_ref().map_or(true, |cur| aid < cur));
                        if is_better {
                            best_score = score;
                            best_id = Some(aid.clone());
                        }
                    }
                }
            }

            if let Some(id) = best_id {
                if !matched_ids.contains(&id) {
                    matched_ids.push(id);
                }
            }
        }

        matched_ids
    }

    #[cfg(windows)]
    pub async fn run_windows_ocr_bytes(bmp_bytes: &[u8]) -> Vec<String> {
        if bmp_bytes.is_empty() {
            return Vec::new();
        }

        #[derive(Clone)]
        struct SafeOcrEngine(windows::Media::Ocr::OcrEngine);
        unsafe impl Send for SafeOcrEngine {}
        unsafe impl Sync for SafeOcrEngine {}

        static GLOBAL_OCR_ENGINE: std::sync::Mutex<Option<SafeOcrEngine>> = std::sync::Mutex::new(None);

        let res = std::panic::catch_unwind(|| -> windows::core::Result<Vec<String>> {
            #[link(name = "ole32")]
            extern "system" {
                fn CoInitializeEx(pvreserved: *mut core::ffi::c_void, dwcoinit: u32) -> i32;
            }
            // COINIT_MULTITHREADED = 0x0
            unsafe {
                let _ = CoInitializeEx(std::ptr::null_mut(), 0);
            }

            use windows::Graphics::Imaging::BitmapDecoder;
            use windows::Media::Ocr::OcrEngine;
            use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

            let stream = InMemoryRandomAccessStream::new()?;
            let writer = DataWriter::CreateDataWriter(&stream)?;
            writer.WriteBytes(bmp_bytes)?;
            writer.StoreAsync()?.get()?;
            writer.FlushAsync()?.get()?;
            let _ = writer.DetachStream();
            stream.Seek(0)?;

            let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
            let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

            let engine = {
                let mut guard = GLOBAL_OCR_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                if guard.is_none() {
                    let created = OcrEngine::TryCreateFromUserProfileLanguages().or_else(|e1| {
                        tracing::warn!("OcrEngine TryCreateFromUserProfileLanguages failed: {:?}, falling back to zh-Hans-CN...", e1);
                        use windows::Globalization::Language;
                        let lang = Language::CreateLanguage(&windows::core::HSTRING::from("zh-Hans-CN"))?;
                        OcrEngine::TryCreateFromLanguage(&lang)
                    });
                    match created {
                        Ok(eng) => {
                            tracing::info!("WinRT OcrEngine successfully initialized and cached.");
                            *guard = Some(SafeOcrEngine(eng));
                        }
                        Err(err) => {
                            tracing::error!("Failed to create WinRT OcrEngine: {:?}. Please ensure Chinese (zh-Hans-CN) OCR package is installed in Windows.", err);
                            return Err(err);
                        }
                    }
                }
                match guard.as_ref() {
                    Some(eng) => eng.0.clone(),
                    None => {
                        return Err(windows::core::Error::new(
                            windows::core::HRESULT(0x80004005_u32 as i32),
                            "WinRT OcrEngine initialization returned empty guard",
                        ));
                    }
                }
            };

            let ocr_result = engine.RecognizeAsync(&software_bitmap)?.get()?;
            let mut lines = Vec::new();
            for line in ocr_result.Lines()? {
                let text = line.Text()?.to_string();
                if !text.trim().is_empty() {
                    lines.push(text);
                }
            }
            Ok(lines)
        });

        match res {
            Ok(Ok(lines)) => lines,
            Ok(Err(e)) => {
                tracing::warn!("Windows OCR recognition error: {:?}", e);
                Vec::new()
            }
            Err(_) => {
                tracing::error!("Windows OCR panic caught");
                Vec::new()
            }
        }
    }

    #[cfg(not(windows))]
    pub async fn run_windows_ocr_bytes(_bmp_bytes: &[u8]) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_chinese_text() {
        assert_eq!(OcrScanner::clean_chinese_text("珠 光 护 手"), "珠光护手");
        assert_eq!(OcrScanner::clean_chinese_text("AD Carry 100"), "AD Carry 100");
    }

    #[test]
    fn test_match_augments_from_texts_deterministic() {
        let mut db = Database::default();
        // Insert two augments with the same length: "珠光护手" (101) and "日炎斗篷" (102)
        db.norm_name_to_augment_id.insert("珠光护手".to_string(), "101".to_string());
        db.norm_name_to_augment_id.insert("日炎斗篷".to_string(), "102".to_string());
        // Insert a 1-character name that must not match via substring
        db.norm_name_to_augment_id.insert("珠".to_string(), "999".to_string());

        let lines = vec!["这是关于珠光护手的卡牌".to_string()];
        let matched = OcrScanner::match_augments_from_texts(&db, &lines);
        assert_eq!(matched, vec!["101"]);
    }
}

