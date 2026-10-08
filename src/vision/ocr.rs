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

            // 2. Substring match (e.g. OCR returned extra prefixes/suffixes)
            let mut found_substring = false;
            for (aug_norm, aid) in &db.norm_name_to_augment_id {
                if aug_norm.len() >= 2 && (norm.contains(aug_norm) || (aug_norm.len() <= norm.len() && norm.starts_with(aug_norm))) {
                    if !matched_ids.contains(aid) {
                        matched_ids.push(aid.clone());
                        found_substring = true;
                        break;
                    }
                }
            }
            if found_substring {
                continue;
            }

            // 3. Fuzzy match with jaro_winkler using pre-normalized names
            let mut best_id = None;
            let mut best_score = 0.0;

            for (aug_norm, aid) in &db.norm_name_to_augment_id {
                let score = jaro_winkler(&norm, aug_norm);
                if score > 0.82 && score > best_score {
                    best_score = score;
                    best_id = Some(aid.clone());
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

        let res = std::panic::catch_unwind(|| -> windows::core::Result<Vec<String>> {
            use windows::Graphics::Imaging::BitmapDecoder;
            use windows::Media::Ocr::OcrEngine;
            use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

            let stream = InMemoryRandomAccessStream::new()?;
            let writer = DataWriter::CreateDataWriter(&stream)?;
            writer.WriteBytes(bmp_bytes)?;
            writer.StoreAsync()?.get()?;
            writer.FlushAsync()?.get()?;

            let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
            let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

            let engine = OcrEngine::TryCreateFromUserProfileLanguages()
                .or_else(|_| {
                    use windows::Globalization::Language;
                    let lang = Language::CreateLanguage(&windows::core::HSTRING::from("zh-Hans-CN"))?;
                    OcrEngine::TryCreateFromLanguage(&lang)
                })?;

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
            _ => Vec::new(),
        }
    }

    #[cfg(not(windows))]
    pub async fn run_windows_ocr_bytes(_bmp_bytes: &[u8]) -> Vec<String> {
        Vec::new()
    }
}
