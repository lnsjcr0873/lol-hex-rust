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
}

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

    let tmp_path = match path.file_name() {
        Some(fname) => path.with_file_name(format!("{}.tmp", fname.to_string_lossy())),
        None => path.with_extension("tmp"),
    };

    {
        use std::io::Write;
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }

    // Atomically rename tmp file to target path
    if let Err(e) = fs::rename(&tmp_path, path) {
        // Fallback for Windows if destination already exists or rename fails
        if path.exists() {
            let _ = fs::remove_file(path);
            if let Err(e2) = fs::rename(&tmp_path, path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(e2);
            }
        } else {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }
    }

    Ok(())
}

impl ConfigManager {
    pub fn new(path: &str) -> Self {
        let mut initial = RecommendationConfig::default();
        if Path::new(path).exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if !content.trim().is_empty() {
                    if let Ok(cfg) = serde_json::from_str::<RecommendationConfig>(&content) {
                        initial = cfg;
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
            cfg.auto_accept_delay_seconds = aads.clamp(0, 10);
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

        // Ensure temp file was cleaned up / renamed
        let tmp_path = temp_dir.join("config.json.tmp");
        assert!(!tmp_path.exists());

        // Test corrupted 0-byte file recovery
        let corrupt_path = temp_dir.join("corrupt.json");
        fs::write(&corrupt_path, "").unwrap();
        let corrupt_mgr = ConfigManager::new(corrupt_path.to_str().unwrap());
        let def_cfg = corrupt_mgr.get().await;
        assert_eq!(def_cfg.current_preset, "SMART");

        let _ = fs::remove_dir_all(temp_dir);
    }
}
