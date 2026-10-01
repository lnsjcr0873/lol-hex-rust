use crate::data::loader::Database;
use regex::Regex;
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
            "锻造器", "属性卡", "属性增强", "提供属性", "金币属性",
            "stat anvil", "anvil", "stat shard",
        ];
        for kw in keywords {
            if full_text.contains(kw) {
                return true;
            }
        }
        false
    }

    pub fn match_augments_from_texts(db: &Database, lines: &[String]) -> Vec<String> {
        let mut matched_ids = Vec::new();
        let norm_re = Regex::new(r"[\s:：\-!！,，。.()\[\]【】\"'`“”、·]+").unwrap();

        for line in lines {
            let cleaned = Self::clean_chinese_text(line);
            let norm = norm_re.replace_all(&cleaned, "").to_string();
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

            // 2. Fuzzy match with jaro_winkler
            let mut best_id = None;
            let mut best_score = 0.0;

            for (aid, aug) in &db.augments_map {
                let aug_norm = norm_re.replace_all(&aug.name, "").to_string();
                let score = jaro_winkler(&norm, &aug_norm);
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
    pub async fn run_windows_ocr_bytes(_bmp_bytes: &[u8]) -> Vec<String> {
        // In full Windows runtime, loads BitmapDecoder and calls OcrEngine::TryCreateFromLanguage
        Vec::new()
    }
}
