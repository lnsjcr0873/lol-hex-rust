use super::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct Database {
    pub meta: MetaInfo,
    pub heroes_summary: Vec<HeroSummary>,
    pub heroes_map: HashMap<String, HeroSummary>,
    pub heroes_detail: Arc<std::sync::RwLock<HashMap<String, Arc<HeroDetail>>>>,
    pub augments_summary: Vec<AugmentSummary>,
    pub augments_map: HashMap<String, AugmentSummary>,
    pub postmatch_data: HashMap<String, PostMatchInfo>,

    // Search Indices
    pub name_to_hero_id: HashMap<String, String>,
    pub name_to_augment_id: HashMap<String, String>,
    pub norm_name_to_augment_id: HashMap<String, String>,

    // Optional Local Disk Paths (Optional fallback only)
    pub base_dir: PathBuf,
    pub detail_dir: PathBuf,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            meta: MetaInfo::default(),
            heroes_summary: Vec::new(),
            heroes_map: HashMap::new(),
            heroes_detail: Arc::new(std::sync::RwLock::new(HashMap::new())),
            augments_summary: Vec::new(),
            augments_map: HashMap::new(),
            postmatch_data: HashMap::new(),
            name_to_hero_id: HashMap::new(),
            name_to_augment_id: HashMap::new(),
            norm_name_to_augment_id: HashMap::new(),
            base_dir: PathBuf::from("data"),
            detail_dir: PathBuf::from("data/json/heroes_detail"),
        }
    }
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load online from Hexdata Cloud API into in-memory hot cache (Zero local disk dependency)
    pub async fn load_online(crawler: &crate::crawler::HexdataCrawler) -> Self {
        let mut db = Self::default();
        db.sync_from_api(crawler).await;
        db
    }

    /// Fetch latest meta, heroes summary, and augments summary from Hexdata cloud API
    pub async fn sync_from_api(&mut self, crawler: &crate::crawler::HexdataCrawler) {
        info!("Connecting to Hexdata cloud API for latest esports big data...");

        // 1. Meta
        if let Some(val) = crawler.fetch_meta().await {
            if let Ok(meta) = serde_json::from_value::<MetaInfo>(val) {
                self.meta = meta;
            }
        }

        // 2. Heroes Summary
        if let Some(val) = crawler.fetch_heroes_summary().await {
            if let Ok(list) = serde_json::from_value::<Vec<HeroSummary>>(val) {
                self.populate_heroes(list);
            }
        }

        // 3. Augments Summary
        if let Some(val) = crawler.fetch_augments_summary().await {
            if let Ok(list) = serde_json::from_value::<Vec<AugmentSummary>>(val) {
                self.populate_augments(list);
            }
        }

        info!(
            "⚡ Cloud BigData Ingested: {} heroes, {} augments in memory (Hero details cached on demand)",
            self.heroes_summary.len(),
            self.augments_summary.len()
        );
    }

    fn populate_heroes(&mut self, list: Vec<HeroSummary>) {
        for h in list {
            let id = h.id.clone();
            let name_lower = h.name.to_lowercase();
            self.name_to_hero_id.insert(name_lower, id.clone());
            self.name_to_hero_id.insert(h.name.clone(), id.clone());

            for term in &h.search_terms {
                self.name_to_hero_id.insert(term.to_lowercase(), id.clone());
                self.name_to_hero_id.insert(term.clone(), id.clone());
            }

            self.heroes_map.insert(id, h.clone());
            self.heroes_summary.push(h);
        }
    }

    fn populate_augments(&mut self, list: Vec<AugmentSummary>) {
        for a in list {
            let id = a.id.clone();
            let name = a.name.clone();
            self.name_to_augment_id.insert(name.clone(), id.clone());
            self.name_to_augment_id.insert(name.to_lowercase(), id.clone());

            // Normalize punctuation & spaces for OCR fuzzy matching
            let norm: String = name
                .chars()
                .filter(|c| !crate::vision::OcrScanner::is_punctuation(*c))
                .collect();
            if !norm.is_empty() {
                self.norm_name_to_augment_id.insert(norm, id.clone());
            }

            if let Some(slug) = &a.seo_slug {
                self.name_to_augment_id.insert(slug.to_lowercase(), id.clone());
            }

            self.augments_map.insert(id, a.clone());
            self.augments_summary.push(a);
        }
    }

    /// Load from optional local directory if present, otherwise gracefully fallback to defaults
    pub fn load_from_dir(base_dir: &str) -> Self {
        let mut db = Database::default();
        db.base_dir = PathBuf::from(base_dir);

        let mut json_dir = Path::new(base_dir).join("json");
        if !json_dir.exists() {
            if Path::new(base_dir).is_dir() && Path::new(base_dir).join("heroes_summary.json").exists() {
                json_dir = Path::new(base_dir).to_path_buf();
            } else if Path::new("data/json").exists() {
                json_dir = Path::new("data/json").to_path_buf();
            } else if Path::new("../data/json").exists() {
                json_dir = Path::new("../data/json").to_path_buf();
            } else if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let candidate = parent.join("data").join("json");
                    if candidate.exists() {
                        json_dir = candidate;
                    }
                }
            }
        }

        let meta_file = json_dir.join("meta.json");
        let heroes_file = json_dir.join("heroes_summary.json");
        let augments_file = json_dir.join("augments_summary.json");
        let postmatch_file = json_dir.join("postmatch.json");
        let detail_dir = json_dir.join("heroes_detail");
        db.detail_dir = detail_dir.clone();

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
                    db.populate_heroes(list);
                }
            }
        }

        // 3. Augments Summary
        if augments_file.exists() {
            if let Ok(content) = fs::read_to_string(&augments_file) {
                if let Ok(list) = serde_json::from_str::<Vec<AugmentSummary>>(&content) {
                    db.populate_augments(list);
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

        // 5. Index filenames in detail_dir if directory exists
        if detail_dir.exists() {
            if let Ok(entries) = fs::read_dir(&detail_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        let stem_parts: Vec<&str> = stem.split('_').collect();
                        let stem_id = stem_parts.first().copied().unwrap_or("").to_string();
                        if !stem_id.is_empty() {
                            db.name_to_hero_id.insert(stem.clone(), stem_id.clone());
                            if stem_parts.len() > 1 {
                                let stem_name = stem_parts[1..].join("_");
                                db.name_to_hero_id.insert(stem_name.clone(), stem_id.clone());
                                db.name_to_hero_id.insert(stem_name.to_lowercase(), stem_id);
                            }
                        }
                    }
                }
            }
        }

        info!(
            "Loaded database with {} heroes, {} augments",
            db.heroes_summary.len(),
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

    pub fn resolve_hero_name(&self, id: &str) -> Option<String> {
        let raw = id.trim();
        if raw.is_empty() {
            return None;
        }
        if let Some(h) = self.heroes_map.get(raw) {
            return Some(h.name.clone());
        }
        if let Some(d) = self.get_hero_detail(raw) {
            if !d.hero_name.is_empty() {
                return Some(d.hero_name.clone());
            }
        }
        None
    }

    pub fn get_hero_detail(&self, query: &str) -> Option<Arc<HeroDetail>> {
        let raw = query.trim();
        let hid = self.resolve_hero_id(raw).unwrap_or_else(|| raw.to_string());

        // 1. Check L1 in-memory cache
        if let Ok(map) = self.heroes_detail.read() {
            if let Some(detail) = map.get(&hid) {
                return Some(Arc::clone(detail));
            }
            if let Some(detail) = map.get(raw) {
                return Some(Arc::clone(detail));
            }
        }

        // 2. Check local disk fallback if present
        if let Some(detail) = self.load_hero_detail_from_disk(&hid) {
            let detail_arc = Arc::new(detail);
            if let Ok(mut map) = self.heroes_detail.write() {
                map.insert(hid.clone(), Arc::clone(&detail_arc));
                if !detail_arc.hero_name.is_empty() {
                    map.insert(detail_arc.hero_name.clone(), Arc::clone(&detail_arc));
                }
            }
            return Some(detail_arc);
        }

        None
    }

    fn load_hero_detail_from_disk(&self, hero_id: &str) -> Option<HeroDetail> {
        if !self.detail_dir.exists() {
            return None;
        }

        let target_prefix = format!("{}_", hero_id);
        let target_exact = format!("{}.json", hero_id);

        if let Ok(entries) = fs::read_dir(&self.detail_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with(&target_prefix) || fname == target_exact {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(mut detail) = serde_json::from_str::<HeroDetail>(&content) {
                            if detail.hero_name.is_empty() {
                                if let Some(h) = self.heroes_map.get(hero_id) {
                                    detail.hero_name = h.name.clone();
                                }
                            }
                            if detail.hero_id.is_null() || detail.hero_id == serde_json::Value::String(String::new()) {
                                detail.hero_id = serde_json::Value::String(hero_id.to_string());
                            }
                            return Some(detail);
                        }
                    }
                }
            }
        }
        None
    }

    /// On-demand online fetching into in-memory hot cache (Zero local disk file generation)
    pub async fn fetch_and_cache_hero_detail(
        &self,
        hero_id: &str,
        crawler: &crate::crawler::HexdataCrawler,
    ) -> Option<Arc<HeroDetail>> {
        let hid = self.resolve_hero_id(hero_id).unwrap_or_else(|| hero_id.to_string());

        // 1. Check L1 in-memory cache first
        if let Ok(map) = self.heroes_detail.read() {
            if let Some(detail) = map.get(&hid) {
                return Some(Arc::clone(detail));
            }
        }

        // 2. Fetch directly from cloud API
        if let Some(val) = crawler.fetch_hero_detail(&hid).await {
            if let Ok(mut detail) = serde_json::from_value::<HeroDetail>(val) {
                if detail.hero_name.is_empty() {
                    if let Some(h) = self.heroes_map.get(&hid) {
                        detail.hero_name = h.name.clone();
                    }
                }
                if detail.hero_id.is_null() || detail.hero_id == serde_json::Value::String(String::new()) {
                    detail.hero_id = serde_json::Value::String(hid.clone());
                }

                let detail_arc = Arc::new(detail);
                // Update L1 in-memory cache ONLY (no local disk clutter)
                if let Ok(mut map) = self.heroes_detail.write() {
                    map.insert(hid.clone(), Arc::clone(&detail_arc));
                    if !detail_arc.hero_name.is_empty() {
                        map.insert(detail_arc.hero_name.clone(), Arc::clone(&detail_arc));
                    }
                }
                info!("⚡ Loaded hero detail from Hexdata Cloud API for {} ({})", hid, detail_arc.hero_name);
                return Some(detail_arc);
            }
        }

        // 3. Fallback to existing memory or disk if available
        self.get_hero_detail(&hid)
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
