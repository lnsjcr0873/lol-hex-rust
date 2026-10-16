use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SgpServerConfig {
    #[serde(rename = "matchHistory")]
    pub match_history: String,
    pub common: String,
    #[serde(rename = "isTencent", default)]
    pub is_tencent: bool,
    #[serde(rename = "regionPathParam", default)]
    pub region_path_param: Option<String>,
}

/// Normalizes region and platform ID into an SGP Server ID, matching LeagueAkari's `getSgpServerId`.
pub fn get_sgp_server_id(region: &str, rso_platform_id: Option<&str>) -> String {
    let r = region.trim().to_uppercase();
    let p = rso_platform_id.map(|s| s.trim().to_uppercase()).unwrap_or_default();

    if r.is_empty() {
        if !p.is_empty() {
            if p.starts_with("HN")
                || p.starts_with("WT")
                || p.starts_with("BGP")
                || p.starts_with("EDU")
                || p.starts_with("TJ")
                || p.starts_with("NJ")
                || p.starts_with("GZ")
                || p.starts_with("CQ")
            {
                return format!("TENCENT_{}", p);
            }
            return p;
        }
        return String::new();
    }

    if r == "TENCENT" {
        return format!("TENCENT_{}", p);
    }

    // Normalize known LCU region codes into SGP config IDs.
    match r.as_str() {
        "NA" => "NA1".to_string(),
        "BR" => "BR1".to_string(),
        "TR" => "TR1".to_string(),
        "LAN" => "LA1".to_string(),
        "LAS" => "LA2".to_string(),
        "OCE" => "OC1".to_string(),
        "EUW1" => "EUW".to_string(),
        "JP1" => "JP".to_string(),
        other => other.to_string(),
    }
}

/// Resolves the sub-ID placeholder (e.g. for URL replacement), matching LeagueAkari.
pub fn resolve_sub_id(
    server_id: &str,
    server_config: &SgpServerConfig,
    rso_platform_id: Option<&str>,
) -> String {
    if let Some(ref path_param) = server_config.region_path_param {
        return path_param.clone();
    }

    if server_config.is_tencent {
        if let Some(pid) = rso_platform_id {
            if !pid.trim().is_empty() {
                return pid.trim().to_uppercase();
            }
        }
        if let Some(stripped) = server_id.strip_prefix("TENCENT_") {
            return stripped.to_string();
        }
    }

    server_id.to_string()
}

/// Returns the built-in SGP server routing table mirroring LeagueAkari's BUILTIN_SGP_LEAGUE_SERVERS_CONFIG.
pub fn get_builtin_sgp_servers() -> HashMap<String, SgpServerConfig> {
    let mut map = HashMap::new();

    // Tencent Shards
    let tencent_servers = [
        ("TENCENT_HN1", "https://hn1-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_HN10", "https://hn10-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_TJ100", "https://tj100-sgp.lol.qq.com:21019"),
        ("TENCENT_TJ101", "https://tj101-sgp.lol.qq.com:21019"),
        ("TENCENT_NJ100", "https://nj100-sgp.lol.qq.com:21019"),
        ("TENCENT_GZ100", "https://gz100-sgp.lol.qq.com:21019"),
        ("TENCENT_CQ100", "https://cq100-sgp.lol.qq.com:21019"),
        ("TENCENT_BGP2", "https://bgp2-k8s-sgp.lol.qq.com:21019"),
        ("TENCENT_PBE", "https://pbe-sgp.lol.qq.com:21019"),
        ("TENCENT_PREPBE", "https://prepbe-sgp.lol.qq.com:21019"),
    ];
    for (id, url) in tencent_servers {
        map.insert(
            id.to_string(),
            SgpServerConfig {
                match_history: url.to_string(),
                common: url.to_string(),
                is_tencent: true,
                region_path_param: None,
            },
        );
    }

    // Riot Global Shards
    map.insert(
        "TW2".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://tw2-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "SG2".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://sg2-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "PH2".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://ph2-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "VN2".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://vn2-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "PBE".to_string(),
        SgpServerConfig {
            match_history: "https://usw2-red.pp.sgp.pvp.net".to_string(),
            common: "https://pbe-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: Some("PBE1".to_string()),
        },
    );
    map.insert(
        "EUW".to_string(),
        SgpServerConfig {
            match_history: "https://euc1-red.pp.sgp.pvp.net".to_string(),
            common: "https://euw-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: Some("EUW1".to_string()),
        },
    );
    map.insert(
        "JP".to_string(),
        SgpServerConfig {
            match_history: "https://apne1-red.pp.sgp.pvp.net".to_string(),
            common: "https://jp-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: Some("JP1".to_string()),
        },
    );
    map.insert(
        "RU".to_string(),
        SgpServerConfig {
            match_history: "https://euc1-red.pp.sgp.pvp.net".to_string(),
            common: "https://ru-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "BR1".to_string(),
        SgpServerConfig {
            match_history: "https://usw2-red.pp.sgp.pvp.net".to_string(),
            common: "https://br-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "OC1".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://oce-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "TR1".to_string(),
        SgpServerConfig {
            match_history: "https://euc1-red.pp.sgp.pvp.net".to_string(),
            common: "https://tr-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "LA1".to_string(),
        SgpServerConfig {
            match_history: "https://usw2-red.pp.sgp.pvp.net".to_string(),
            common: "https://lan-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "LA2".to_string(),
        SgpServerConfig {
            match_history: "https://usw2-red.pp.sgp.pvp.net".to_string(),
            common: "https://las-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "NA1".to_string(),
        SgpServerConfig {
            match_history: "https://usw2-red.pp.sgp.pvp.net".to_string(),
            common: "https://na-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "TH2".to_string(),
        SgpServerConfig {
            match_history: "https://apse1-red.pp.sgp.pvp.net".to_string(),
            common: "https://th2-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );
    map.insert(
        "KR".to_string(),
        SgpServerConfig {
            match_history: "https://apne1-red.pp.sgp.pvp.net".to_string(),
            common: "https://kr-red.lol.sgp.pvp.net".to_string(),
            is_tencent: false,
            region_path_param: None,
        },
    );

    map
}

/// Resolves the SGP server configuration for a given server ID.
/// Checks the built-in table first, with dynamic fallback for unlisted Tencent shards.
pub fn resolve_server_config(server_id: &str) -> Option<SgpServerConfig> {
    let sid = server_id.trim().to_uppercase();
    if sid.is_empty() {
        return None;
    }

    let builtin = get_builtin_sgp_servers();
    if let Some(cfg) = builtin.get(&sid) {
        return Some(cfg.clone());
    }

    // Try TENCENT_ prefix if not present
    let tencent_sid = format!("TENCENT_{}", sid);
    if let Some(cfg) = builtin.get(&tencent_sid) {
        return Some(cfg.clone());
    }

    // Dynamic resolution for other Tencent shards (e.g. HN2..HN19, WT1..WT7, BGP1, EDU1)
    let is_tencent_shard = sid.starts_with("TENCENT_")
        || sid.starts_with("HN")
        || sid.starts_with("WT")
        || sid.starts_with("BGP")
        || sid.starts_with("EDU")
        || sid.starts_with("TJ")
        || sid.starts_with("NJ")
        || sid.starts_with("GZ")
        || sid.starts_with("CQ");

    if is_tencent_shard {
        let platform = sid.strip_prefix("TENCENT_").unwrap_or(&sid).to_lowercase();
        let url = format!("https://{}-k8s-sgp.lol.qq.com:21019", platform);
        return Some(SgpServerConfig {
            match_history: url.clone(),
            common: url,
            is_tencent: true,
            region_path_param: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sgp_server_id_normalization() {
        assert_eq!(get_sgp_server_id("TENCENT", Some("HN1")), "TENCENT_HN1");
        assert_eq!(get_sgp_server_id("tencent", Some("hn10")), "TENCENT_HN10");
        assert_eq!(get_sgp_server_id("NA", None), "NA1");
        assert_eq!(get_sgp_server_id("EUW1", None), "EUW");
        assert_eq!(get_sgp_server_id("KR", None), "KR");
        assert_eq!(get_sgp_server_id("", Some("HN1")), "TENCENT_HN1");
    }

    #[test]
    fn test_resolve_server_config_builtin_and_dynamic() {
        let hn1 = resolve_server_config("TENCENT_HN1").expect("Should resolve TENCENT_HN1");
        assert!(hn1.is_tencent);
        assert_eq!(hn1.match_history, "https://hn1-k8s-sgp.lol.qq.com:21019");

        // Bare HN1 should resolve to Tencent HN1 config
        let bare_hn1 = resolve_server_config("HN1").expect("Should resolve bare HN1");
        assert!(bare_hn1.is_tencent);
        assert_eq!(bare_hn1.match_history, "https://hn1-k8s-sgp.lol.qq.com:21019");

        // Dynamic unlisted shard e.g. WT3 (弗雷尔卓德)
        let wt3 = resolve_server_config("WT3").expect("Should dynamically resolve WT3");
        assert!(wt3.is_tencent);
        assert_eq!(wt3.match_history, "https://wt3-k8s-sgp.lol.qq.com:21019");

        // Global shard
        let na1 = resolve_server_config("NA1").expect("Should resolve NA1");
        assert!(!na1.is_tencent);
        assert_eq!(na1.match_history, "https://usw2-red.pp.sgp.pvp.net");
    }

    #[test]
    fn test_resolve_sub_id() {
        let hn1_cfg = resolve_server_config("TENCENT_HN1").unwrap();
        assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, None), "HN1");
        assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, Some("HN1")), "HN1");

        let na1_cfg = resolve_server_config("NA1").unwrap();
        assert_eq!(resolve_sub_id("NA1", &na1_cfg, None), "NA1");
    }
}
