use super::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

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
    pub hero_id_to_file: HashMap<String, PathBuf>,
    pub missed_hero_details: Arc<std::sync::RwLock<HashMap<String, std::time::Instant>>>,

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
            hero_id_to_file: HashMap::new(),
            missed_hero_details: Arc::new(std::sync::RwLock::new(HashMap::new())),
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

    pub fn clear_heroes(&mut self) {
        self.heroes_summary.clear();
        self.heroes_map.clear();
        self.name_to_hero_id.clear();
    }

    pub fn clear_augments(&mut self) {
        self.augments_summary.clear();
        self.augments_map.clear();
        self.name_to_augment_id.clear();
        self.norm_name_to_augment_id.clear();
    }

    pub fn clear_cached_details(&self) {
        if let Ok(mut map) = self.heroes_detail.write() {
            map.clear();
        }
        if let Ok(mut map) = self.missed_hero_details.write() {
            map.clear();
        }
    }

    /// Fetch latest meta, heroes summary, and augments summary from Hexdata cloud API
    pub async fn sync_from_api(&mut self, crawler: &crate::crawler::HexdataCrawler) {
        info!("Connecting to Hexdata cloud API for latest esports big data...");

        // 1. Meta
        let mut patch_changed = false;
        if let Some(val) = crawler.fetch_meta().await {
            if let Ok(meta) = serde_json::from_value::<MetaInfo>(val) {
                if !self.meta.report_patch.is_empty() && self.meta.report_patch != meta.report_patch {
                    patch_changed = true;
                }
                self.meta = meta;
            }
        }

        if patch_changed {
            self.clear_cached_details();
        }

        // 2. Heroes Summary
        if let Some(val) = crawler.fetch_heroes_summary().await {
            if let Ok(list) = serde_json::from_value::<Vec<HeroSummary>>(val) {
                self.clear_heroes();
                self.populate_heroes(list);
            }
        }

        // 3. Augments Summary
        if let Some(val) = crawler.fetch_augments_summary().await {
            if let Ok(list) = serde_json::from_value::<Vec<AugmentSummary>>(val) {
                self.clear_augments();
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
            } else if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let candidate = Path::new(&manifest_dir).join("data").join("json");
                if candidate.exists() {
                    json_dir = candidate;
                }
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
        if db.meta.report_patch.is_empty() {
            const EMBEDDED_META: &str = include_str!("../../data/json/meta.json");
            if let Ok(meta) = serde_json::from_str::<MetaInfo>(EMBEDDED_META) {
                db.meta = meta;
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
        if db.heroes_summary.is_empty() {
            const EMBEDDED_HEROES: &str = include_str!("../../data/json/heroes_summary.json");
            if let Ok(list) = serde_json::from_str::<Vec<HeroSummary>>(EMBEDDED_HEROES) {
                db.populate_heroes(list);
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
        if db.augments_summary.is_empty() {
            const EMBEDDED_AUGMENTS: &str = include_str!("../../data/json/augments_summary.json");
            if let Ok(list) = serde_json::from_str::<Vec<AugmentSummary>>(EMBEDDED_AUGMENTS) {
                db.populate_augments(list);
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
                            db.hero_id_to_file.insert(stem_id.clone(), path.clone());
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
        // Substring / alias search (e.g. "剑魔" -> "暗裔剑魔", "卡莎" -> "虚空之女")
        for (name, id) in &self.name_to_hero_id {
            if name.contains(&lower) || (!lower.is_empty() && lower.contains(name)) {
                return Some(id.clone());
            }
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

    pub fn resolve_item_name(&self, item_id: &str) -> String {
        crate::data::recipe_data::resolve_item_name(item_id)
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

        // 2. Negative cache check to prevent repeated disk scanning for missing heroes
        if let Ok(missed) = self.missed_hero_details.read() {
            if let Some(instant) = missed.get(&hid) {
                if instant.elapsed() < std::time::Duration::from_secs(10) {
                    return None;
                }
            }
        }

        // 3. Check local disk fallback if present
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

        // Record miss in negative cache
        if let Ok(mut missed) = self.missed_hero_details.write() {
            missed.insert(hid, std::time::Instant::now());
        }

        None
    }

    fn load_hero_detail_from_disk(&self, hero_id: &str) -> Option<HeroDetail> {
        let target_prefix = format!("{}_", hero_id);
        let target_exact = format!("{}.json", hero_id);

        // 0. Indexed file fast path (O(1) direct lookup from load_from_dir indexing)
        if let Some(indexed_path) = self.hero_id_to_file.get(hero_id) {
            if let Ok(content) = fs::read_to_string(indexed_path) {
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

        // 1. Direct file fast path in detail_dir
        let direct_file = self.detail_dir.join(&target_exact);
        if direct_file.is_file() {
            if let Ok(content) = fs::read_to_string(&direct_file) {
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

        // 2. Check self.detail_dir with prefix matching (e.g. data/json/heroes_detail)
        if self.detail_dir.exists() {
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
        }

        // 3. Check data/cache/*/heroes_detail/{hero_id}.json
        let cache_root = Path::new("data/cache");
        if cache_root.exists() {
            if let Ok(patch_dirs) = fs::read_dir(cache_root) {
                for patch_entry in patch_dirs.flatten() {
                    let hero_file = patch_entry.path().join("heroes_detail").join(&target_exact);
                    if hero_file.exists() {
                        if let Ok(content) = fs::read_to_string(&hero_file) {
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
        }

        None
    }

    /// On-demand online fetching for active champion into in-memory hot cache and local disk cache
    pub async fn fetch_and_cache_hero_detail(
        &self,
        hero_id: &str,
        crawler: &crate::crawler::HexdataCrawler,
    ) -> Option<Arc<HeroDetail>> {
        let hid = self.resolve_hero_id(hero_id).unwrap_or_else(|| hero_id.to_string());

        // 1. Check local in-memory or disk cache first (Instant 0ms, zero network requests)
        if let Some(detail) = self.get_hero_detail(&hid) {
            return Some(detail);
        }

        // 2. Fetch directly from cloud API ONLY if local files are missing
        if let Some(val) = crawler.fetch_hero_detail(&hid).await {
            if let Ok(mut detail) = serde_json::from_value::<HeroDetail>(val.clone()) {
                if detail.hero_name.is_empty() {
                    if let Some(name) = self.resolve_hero_name(&hid) {
                        detail.hero_name = name;
                    } else if let Some(h) = self.heroes_map.get(&hid) {
                        detail.hero_name = h.name.clone();
                    }
                }
                if detail.hero_id.is_null() || detail.hero_id == serde_json::Value::String(String::new()) {
                    detail.hero_id = serde_json::Value::String(hid.clone());
                }

                let detail_arc = Arc::new(detail);
                // Update L1 in-memory cache
                if let Ok(mut map) = self.heroes_detail.write() {
                    map.insert(hid.clone(), Arc::clone(&detail_arc));
                    if !detail_arc.hero_name.is_empty() {
                        map.insert(detail_arc.hero_name.clone(), Arc::clone(&detail_arc));
                    }
                }

                // Persist to local disk cache asynchronously without blocking Tokio worker threads
                let patch = if !self.meta.report_patch.is_empty() {
                    self.meta.report_patch.clone()
                } else {
                    "default".to_string()
                };
                let clean_patch = patch.replace(['/', '\\'], "_");
                let cache_dir = Path::new("data/cache").join(&clean_patch).join("heroes_detail");
                let json_val = val.clone();
                let hid_for_disk = hid.clone();
                tokio::task::spawn_blocking(move || {
                    if let Ok(()) = fs::create_dir_all(&cache_dir) {
                        let cache_file = cache_dir.join(format!("{}.json", hid_for_disk));
                        let _ = fs::write(cache_file, serde_json::to_string(&json_val).unwrap_or_default());
                    }
                });

                info!("⚡ Loaded hero detail from Hexdata Cloud API for {} ({})", hid, detail_arc.hero_name);
                return Some(detail_arc);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_anti_duplication_on_repeated_ingestion() {
        let mut db = Database::load_from_dir("data");
        let initial_heroes_len = db.heroes_summary.len();
        let initial_augments_len = db.augments_summary.len();
        assert!(initial_heroes_len > 0, "Initial heroes summary must not be empty");
        assert!(initial_augments_len > 0, "Initial augments summary must not be empty");

        let heroes_clone = db.heroes_summary.clone();
        let augments_clone = db.augments_summary.clone();

        // Simulate 1st sync (clearing before populating)
        db.clear_heroes();
        db.populate_heroes(heroes_clone.clone());
        assert_eq!(db.heroes_summary.len(), initial_heroes_len);
        assert_eq!(db.heroes_map.len(), initial_heroes_len);
        let initial_name_to_hero = db.name_to_hero_id.len();

        db.clear_augments();
        db.populate_augments(augments_clone.clone());
        assert_eq!(db.augments_summary.len(), initial_augments_len);
        assert_eq!(db.augments_map.len(), initial_augments_len);
        let initial_name_to_aug = db.name_to_augment_id.len();
        let initial_norm_aug = db.norm_name_to_augment_id.len();

        // Simulate 2nd sync (repeated sync must NOT double count or leak reverse indices)
        db.clear_heroes();
        db.populate_heroes(heroes_clone.clone());
        assert_eq!(db.heroes_summary.len(), initial_heroes_len);
        assert_eq!(db.heroes_map.len(), initial_heroes_len);
        assert_eq!(db.name_to_hero_id.len(), initial_name_to_hero);

        db.clear_augments();
        db.populate_augments(augments_clone.clone());
        assert_eq!(db.augments_summary.len(), initial_augments_len);
        assert_eq!(db.augments_map.len(), initial_augments_len);
        assert_eq!(db.name_to_augment_id.len(), initial_name_to_aug);
        assert_eq!(db.norm_name_to_augment_id.len(), initial_norm_aug);

        // Simulate 3rd sync
        db.clear_heroes();
        db.populate_heroes(heroes_clone.clone());
        assert_eq!(db.heroes_summary.len(), initial_heroes_len);
        assert_eq!(db.heroes_map.len(), initial_heroes_len);
        assert_eq!(db.name_to_hero_id.len(), initial_name_to_hero);

        db.clear_augments();
        db.populate_augments(augments_clone.clone());
        assert_eq!(db.augments_summary.len(), initial_augments_len);
        assert_eq!(db.augments_map.len(), initial_augments_len);
        assert_eq!(db.name_to_augment_id.len(), initial_name_to_aug);
        assert_eq!(db.norm_name_to_augment_id.len(), initial_norm_aug);
    }

    #[tokio::test]
    async fn test_sync_from_api_graceful_offline_no_multiplier() {
        let mut db = Database::load_from_dir("data");
        let initial_heroes_len = db.heroes_summary.len();
        let initial_augments_len = db.augments_summary.len();

        let crawler = crate::crawler::HexdataCrawler::new();

        // Multiple successive sync_from_api calls
        db.sync_from_api(&crawler).await;
        db.sync_from_api(&crawler).await;

        // Count must not multiply or leak
        assert_eq!(db.heroes_summary.len(), initial_heroes_len);
        assert_eq!(db.augments_summary.len(), initial_augments_len);
    }

    #[test]
    fn test_load_from_dir_fallback_when_path_not_exists() {
        let db = Database::load_from_dir("__non_existent_dir_for_test__");
        assert!(!db.heroes_summary.is_empty(), "Embedded fallback should populate heroes_summary");
        assert!(!db.augments_summary.is_empty(), "Embedded fallback should populate augments_summary");
        assert!(!db.meta.report_patch.is_empty(), "Embedded fallback should populate meta");
    }
}
