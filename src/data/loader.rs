use super::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone, Default)]
pub struct Database {
    pub meta: MetaInfo,
    pub heroes_summary: Vec<HeroSummary>,
    pub heroes_map: HashMap<String, HeroSummary>,
    pub heroes_detail: HashMap<String, HeroDetail>,
    pub augments_summary: Vec<AugmentSummary>,
    pub augments_map: HashMap<String, AugmentSummary>,
    pub postmatch_data: HashMap<String, PostMatchInfo>,

    // Search Indices
    pub name_to_hero_id: HashMap<String, String>,
    pub name_to_augment_id: HashMap<String, String>,
    pub norm_name_to_augment_id: HashMap<String, String>,
}

impl Database {
    pub fn load_from_dir(base_dir: &str) -> Self {
        let mut db = Database::default();

        let json_dir = Path::new(base_dir).join("json");
        let meta_file = json_dir.join("meta.json");
        let heroes_file = json_dir.join("heroes_summary.json");
        let augments_file = json_dir.join("augments_summary.json");
        let postmatch_file = json_dir.join("postmatch.json");
        let detail_dir = json_dir.join("heroes_detail");

        // 1. Meta
        if meta_file.exists() {
            if let Ok(content) = fs::read_to_string(&meta_file) {
                if let Ok(meta) = serde_json::from_str::<MetaInfo>(&content) {
                    db.meta = meta;
                }
            }
        }

        // 2. Heroes Summary
        if heroes_file.exists() {
            if let Ok(content) = fs::read_to_string(&heroes_file) {
                if let Ok(list) = serde_json::from_str::<Vec<HeroSummary>>(&content) {
                    for h in list {
                        let id = h.id.clone();
                        let name_lower = h.name.to_lowercase();
                        db.name_to_hero_id.insert(name_lower.clone(), id.clone());
                        db.name_to_hero_id.insert(h.name.clone(), id.clone());

                        for term in &h.search_terms {
                            db.name_to_hero_id.insert(term.to_lowercase(), id.clone());
                        }

                        db.heroes_map.insert(id, h.clone());
                        db.heroes_summary.push(h);
                    }
                }
            }
        }

        // 3. Augments Summary
        if augments_file.exists() {
            if let Ok(content) = fs::read_to_string(&augments_file) {
                if let Ok(list) = serde_json::from_str::<Vec<AugmentSummary>>(&content) {
                    for a in list {
                        let id = a.id.clone();
                        let name = a.name.clone();
                        db.name_to_augment_id.insert(name.clone(), id.clone());
                        db.name_to_augment_id.insert(name.to_lowercase(), id.clone());

                        // Normalize punctuation & spaces
                        let norm: String = name
                            .chars()
                            .filter(|c| !crate::vision::OcrScanner::is_punctuation(*c))
                            .collect();
                        if !norm.is_empty() {
                            db.norm_name_to_augment_id.insert(norm, id.clone());
                        }

                        if let Some(slug) = &a.seo_slug {
                            db.name_to_augment_id.insert(slug.to_lowercase(), id.clone());
                        }

                        db.augments_map.insert(id, a.clone());
                        db.augments_summary.push(a);
                    }
                }
            }
        }

        // 4. PostMatch
        if postmatch_file.exists() {
            if let Ok(content) = fs::read_to_string(&postmatch_file) {
                if let Ok(pm_map) = serde_json::from_str::<HashMap<String, PostMatchInfo>>(&content) {
                    db.postmatch_data = pm_map;
                }
            }
        }

        // 5. Heroes Detail
        if detail_dir.exists() {
            if let Ok(entries) = fs::read_dir(&detail_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(detail) = serde_json::from_str::<HeroDetail>(&content) {
                                let id_str = match &detail.hero_id {
                                    serde_json::Value::String(s) => s.clone(),
                                    serde_json::Value::Number(n) => n.to_string(),
                                    _ => path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                                };
                                if !id_str.is_empty() {
                                    db.heroes_detail.insert(id_str, detail);
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(
            "Loaded database with {} heroes, {} details, {} augments",
            db.heroes_summary.len(),
            db.heroes_detail.len(),
            db.augments_summary.len()
        );

        db
    }

    pub fn resolve_hero_id(&self, identifier: &str) -> Option<String> {
        let raw = identifier.trim();
        if raw.is_empty() {
            return None;
        }
        if self.heroes_map.contains_key(raw) {
            return Some(raw.to_string());
        }
        let lower = raw.to_lowercase();
        if let Some(id) = self.name_to_hero_id.get(&lower) {
            return Some(id.clone());
        }
        if let Some(id) = self.name_to_hero_id.get(raw) {
            return Some(id.clone());
        }
        None
    }

    pub fn resolve_augment_id(&self, identifier: &str) -> Option<String> {
        let raw = identifier.trim();
        if raw.is_empty() {
            return None;
        }
        if self.augments_map.contains_key(raw) {
            return Some(raw.to_string());
        }
        let lower = raw.to_lowercase();
        if let Some(id) = self.name_to_augment_id.get(&lower) {
            return Some(id.clone());
        }
        if let Some(id) = self.name_to_augment_id.get(raw) {
            return Some(id.clone());
        }
        let norm: String = raw
            .chars()
            .filter(|c| !crate::vision::OcrScanner::is_punctuation(*c))
            .collect();
        if let Some(id) = self.norm_name_to_augment_id.get(&norm) {
            return Some(id.clone());
        }
        None
    }
}
