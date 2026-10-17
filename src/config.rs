use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationConfig {
    pub resolve_conflicts: bool,
    pub hex_synergy_weight: f64,
    pub archetype_strictness: f64,
    pub counter_intensity: f64,
    pub gold_calc_mode: String,
    pub trio_mode: String,
    pub current_preset: String,
    #[serde(default)]
    pub auto_accept_match: bool,
    #[serde(default = "default_auto_accept_delay")]
    pub auto_accept_delay_seconds: u64,
    #[serde(default)]
    pub auto_swap_bench: bool,
    #[serde(default)]
    pub auto_apply_runes: bool,
    #[serde(default)]
    pub bench_preferred_champions: Vec<String>,
}

pub const MAX_AUTO_ACCEPT_DELAY_SECONDS: u64 = 10;

fn default_auto_accept_delay() -> u64 {
    2
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            resolve_conflicts: true,
            hex_synergy_weight: 1.0,
            archetype_strictness: 0.7,
            counter_intensity: 0.8,
            gold_calc_mode: "NET_DEDUCT".to_string(),
            trio_mode: "DYNAMIC_COUNTER".to_string(),
            current_preset: "SMART".to_string(),
            auto_accept_match: false,
            auto_accept_delay_seconds: 2,
            auto_swap_bench: false,
            auto_apply_runes: false,
            bench_preferred_champions: Vec::new(),
        }
    }
}

impl RecommendationConfig {
    pub fn for_algorithm(algo: &str) -> Self {
        let mut cfg = Self::default();
        let key = match algo {
            "HEXTECH" | "HEX_FUN" => "HEX_FUN",
            "BLACK_TECH" | "HYBRID_FREE" => "HYBRID_FREE",
            "IRON_BLOOD" | "IRON_COUNTER" => "IRON_COUNTER",
            _ => "SMART",
        };
        let presets = get_preset_definitions();
        if let Some(def) = presets.get(key) {
            cfg.current_preset = algo.to_string();
            cfg.hex_synergy_weight = def.hex_synergy_weight;
            cfg.archetype_strictness = def.archetype_strictness;
            cfg.counter_intensity = def.counter_intensity;
            cfg.trio_mode = def.trio_mode.clone();
        }
        cfg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetDefinition {
    pub name: String,
    pub desc: String,
    pub hex_synergy_weight: f64,
    pub archetype_strictness: f64,
    pub counter_intensity: f64,
    pub trio_mode: String,
}

pub fn get_preset_definitions() -> HashMap<String, PresetDefinition> {
    let mut map = HashMap::new();
    map.insert(
        "SMART".to_string(),
        PresetDefinition {
            name: "👑 智能竞技 (平衡推荐)".to_string(),
            desc: "综合兼顾海克斯特效、英雄本职流派契合度与敌方针对装，容错率最高".to_string(),
            hex_synergy_weight: 1.0,
            archetype_strictness: 0.7,
            counter_intensity: 0.8,
            trio_mode: "DYNAMIC_COUNTER".to_string(),
        },
    );
    map.insert(
        "HEX_FUN".to_string(),
        PresetDefinition {
            name: "⚡ 海克斯纯爽 (特效联动)".to_string(),
            desc: "极限拉满当局海克斯专属强化收益，只要拿到质变强化立即围绕其组建终极套路".to_string(),
            hex_synergy_weight: 2.0,
            archetype_strictness: 0.4,
            counter_intensity: 0.5,
            trio_mode: "STRICT_TRIO".to_string(),
        },
    );
    map.insert(
        "HYBRID_FREE".to_string(),
        PresetDefinition {
            name: "🧪 自由黑科技 (双修突破)".to_string(),
            desc: "弱化传统法师/物理流派枷锁，允许 AP/AD 跨界出装，挖掘非常规冷门套路".to_string(),
            hex_synergy_weight: 1.3,
            archetype_strictness: 0.2,
            counter_intensity: 0.6,
            trio_mode: "OFFENSIVE_ONLY".to_string(),
        },
    );
    map.insert(
        "IRON_COUNTER".to_string(),
        PresetDefinition {
            name: "🛡️ 铁血克制 (分奴保胜)".to_string(),
            desc: "针对敌方 5 人阵容输出与续航机制顶格推演针对装，重伤与穿透拉满，防被翻盘".to_string(),
            hex_synergy_weight: 0.6,
            archetype_strictness: 0.8,
            counter_intensity: 1.8,
            trio_mode: "DYNAMIC_COUNTER".to_string(),
        },
    );
    map
}

#[derive(Clone)]
pub struct ConfigManager {
    pub config: Arc<RwLock<RecommendationConfig>>,
    pub config_file_path: String,
}

pub fn save_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    static TMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let pid = std::process::id();
    let cnt = TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let tmp_path = match path.file_name() {
        Some(fname) => path.with_file_name(format!("{}.tmp_{}_{}", fname.to_string_lossy(), pid, cnt)),
        None => path.with_extension(format!("tmp_{}_{}", pid, cnt)),
    };

    let bak_path = match path.file_name() {
        Some(fname) => path.with_file_name(format!("{}.bak", fname.to_string_lossy())),
        None => path.with_extension("bak"),
    };

    // 1. Write content to temporary file on the same volume and sync buffers
    {
        use std::io::Write;
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }

    // 2. Perform atomic replace with retries for Windows AV / indexer lock contention
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        let to_wide_null = |p: &Path| -> Vec<u16> {
            p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
        };

        extern "system" {
            fn MoveFileExW(lpexistingfilename: *const u16, lpnewfilename: *const u16, dwflags: u32) -> i32;
            fn ReplaceFileW(
                lpreplacedfilename: *const u16,
                lpreplacementfilename: *const u16,
                lpbackupfilename: *const u16,
                dwreplaceflags: u32,
                lpexclude: *mut core::ffi::c_void,
                lpreserved: *mut core::ffi::c_void,
            ) -> i32;
        }

        const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
        const MOVEFILE_WRITE_THROUGH: u32 = 0x00000008;

        let wide_tmp = to_wide_null(&tmp_path);
        let wide_dest = to_wide_null(path);
        let wide_bak = to_wide_null(&bak_path);

        let mut success = false;
        let mut last_error = std::io::Error::last_os_error();

        for attempt in 0..5 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(15 * attempt));
            }

            if path.exists() {
                // If destination exists, try ReplaceFileW (creates backup & atomically replaces)
                let res = unsafe {
                    ReplaceFileW(
                        wide_dest.as_ptr(),
                        wide_tmp.as_ptr(),
                        wide_bak.as_ptr(),
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                };
                if res != 0 {
                    success = true;
                    break;
                }

                // If ReplaceFileW failed, fallback to MoveFileExW
                let move_res = unsafe {
                    MoveFileExW(
                        wide_tmp.as_ptr(),
                        wide_dest.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                    )
                };
                if move_res != 0 {
                    success = true;
                    break;
                }
            } else {
                let move_res = unsafe {
                    MoveFileExW(
                        wide_tmp.as_ptr(),
                        wide_dest.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                    )
                };
                if move_res != 0 {
                    success = true;
                    break;
                }
            }
            last_error = std::io::Error::last_os_error();
        }

        if !success {
            // Clean up temporary file, leaving original path completely untouched!
            let _ = fs::remove_file(&tmp_path);
            return Err(last_error);
        }

        return Ok(());
    }

    #[cfg(not(windows))]
    {
        let mut last_err = None;
        for attempt in 0..5 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(15 * attempt));
            }
            if path.exists() {
                let _ = fs::copy(path, &bak_path);
            }
            match fs::rename(&tmp_path, path) {
                Ok(()) => return Ok(()),
                Err(e) => last_err = Some(e),
            }
        }
        let _ = fs::remove_file(&tmp_path);
        Err(last_err.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Atomic save failed")))
    }
}

impl ConfigManager {
    pub fn new(path: &str) -> Self {
        let mut initial = RecommendationConfig::default();
        let p = Path::new(path);
        let bak = p.with_extension("bak");
        let mut loaded = false;

        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    if let Ok(cfg) = serde_json::from_str::<RecommendationConfig>(trimmed) {
                        initial = cfg;
                        loaded = true;
                    }
                }
            }
        }

        // Fallback to backup file if primary file was corrupted or missing
        if !loaded && bak.exists() {
            if let Ok(content) = fs::read_to_string(&bak) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    if let Ok(cfg) = serde_json::from_str::<RecommendationConfig>(trimmed) {
                        initial = cfg;
                        let _ = fs::copy(&bak, p);
                    }
                }
            }
        }

        Self {
            config: Arc::new(RwLock::new(initial)),
            config_file_path: path.to_string(),
        }
    }

    pub async fn get(&self) -> RecommendationConfig {
        self.config.read().await.clone()
    }

    pub async fn save_to_disk(&self) -> bool {
        let cfg = self.config.read().await;
        if let Ok(json_str) = serde_json::to_string_pretty(&*cfg) {
            let path = Path::new(&self.config_file_path);
            return save_atomic(path, &json_str).is_ok();
        }
        false
    }

    pub async fn update(
        &self,
        resolve_conflicts: Option<bool>,
        hex_synergy_weight: Option<f64>,
        archetype_strictness: Option<f64>,
        counter_intensity: Option<f64>,
        gold_calc_mode: Option<String>,
        trio_mode: Option<String>,
        preset: Option<String>,
        auto_accept_match: Option<bool>,
        auto_accept_delay_seconds: Option<u64>,
    ) -> RecommendationConfig {
        let mut cfg = self.config.write().await;
        if let Some(p) = preset {
            let presets = get_preset_definitions();
            if let Some(def) = presets.get(&p) {
                cfg.current_preset = p;
                cfg.hex_synergy_weight = def.hex_synergy_weight;
                cfg.archetype_strictness = def.archetype_strictness;
                cfg.counter_intensity = def.counter_intensity;
                cfg.trio_mode = def.trio_mode.clone();
            }
        }
        if let Some(rc) = resolve_conflicts {
            cfg.resolve_conflicts = rc;
        }
        if let Some(hw) = hex_synergy_weight {
            if !hw.is_nan() && !hw.is_infinite() {
                cfg.hex_synergy_weight = hw.clamp(0.0, 3.0);
            }
        }
        if let Some(as_) = archetype_strictness {
            if !as_.is_nan() && !as_.is_infinite() {
                cfg.archetype_strictness = as_.clamp(0.0, 1.0);
            }
        }
        if let Some(ci) = counter_intensity {
            if !ci.is_nan() && !ci.is_infinite() {
                cfg.counter_intensity = ci.clamp(0.0, 3.0);
            }
        }
        if let Some(gcm) = gold_calc_mode {
            cfg.gold_calc_mode = gcm;
        }
        if let Some(tm) = trio_mode {
            cfg.trio_mode = tm;
        }
        if let Some(aam) = auto_accept_match {
            cfg.auto_accept_match = aam;
        }
        if let Some(aads) = auto_accept_delay_seconds {
            cfg.auto_accept_delay_seconds = aads.clamp(0, MAX_AUTO_ACCEPT_DELAY_SECONDS);
        }

        // Persist to disk safely via atomic rename
        if let Ok(json_str) = serde_json::to_string_pretty(&*cfg) {
            let path = Path::new(&self.config_file_path);
            let _ = save_atomic(path, &json_str);
        }

        cfg.clone()
    }

    pub async fn set_auto_accept(&self, enabled: bool, delay_secs: Option<u64>) -> RecommendationConfig {
        self.update(None, None, None, None, None, None, None, Some(enabled), delay_secs).await
    }

    pub async fn update_automation(
        &self,
        auto_accept_match: Option<bool>,
        auto_accept_delay_seconds: Option<u64>,
        auto_swap_bench: Option<bool>,
        auto_apply_runes: Option<bool>,
        bench_preferred_champions: Option<Vec<String>>,
    ) -> RecommendationConfig {
        let mut cfg = self.config.write().await;
        if let Some(aam) = auto_accept_match {
            cfg.auto_accept_match = aam;
        }
        if let Some(aads) = auto_accept_delay_seconds {
            cfg.auto_accept_delay_seconds = aads.clamp(0, MAX_AUTO_ACCEPT_DELAY_SECONDS);
        }
        if let Some(asb) = auto_swap_bench {
            cfg.auto_swap_bench = asb;
        }
        if let Some(aar) = auto_apply_runes {
            cfg.auto_apply_runes = aar;
        }
        if let Some(bpc) = bench_preferred_champions {
            cfg.bench_preferred_champions = bpc;
        }

        // Persist to disk safely via atomic rename
        if let Ok(json_str) = serde_json::to_string_pretty(&*cfg) {
            let path = Path::new(&self.config_file_path);
            let _ = save_atomic(path, &json_str);
        }

        cfg.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_atomic_persistence_and_no_zero_byte_corruption() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_atomic_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let cfg_path = temp_dir.join("config.json");

        let mgr = ConfigManager::new(cfg_path.to_str().unwrap());
        mgr.update(
            Some(false),
            Some(2.5),
            Some(0.9),
            Some(1.5),
            Some("RAW".to_string()),
            Some("STATIC".to_string()),
            None,
            Some(true),
            Some(5),
        ).await;

        assert!(cfg_path.exists());
        let content = fs::read_to_string(&cfg_path).unwrap();
        assert!(!content.is_empty());
        let parsed: RecommendationConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.hex_synergy_weight, 2.5);
        assert_eq!(parsed.auto_accept_match, true);
        assert_eq!(parsed.auto_accept_delay_seconds, 5);

        // Ensure temp files were cleaned up / renamed
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                assert!(!fname.contains(".tmp"), "Temp file was not cleaned up: {}", fname);
            }
        }

        // Test corrupted 0-byte file recovery
        let corrupt_path = temp_dir.join("corrupt.json");
        fs::write(&corrupt_path, "").unwrap();
        let corrupt_mgr = ConfigManager::new(corrupt_path.to_str().unwrap());
        let def_cfg = corrupt_mgr.get().await;
        assert_eq!(def_cfg.current_preset, "SMART");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
