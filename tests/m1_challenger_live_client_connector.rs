//! Adversarial Empirical Challenge & Verification Suite for Milestone 1
//! Modules under test: `src/lcu/live_client.rs` & `src/lcu/connector.rs`
//!
//! Objectives:
//! 1. Verify skinID behavior: skinID values 0, 1, 2, 266001 do NOT map hero_id to "1" or "2".
//! 2. Verify Riot ID disambiguation: two players sharing the same gameName ("TestPlayer#TAG1" vs "TestPlayer#TAG2")
//!    are strictly separated; local player is bound, second player never overwrites inventory or stats.
//! 3. Verify process sniffing safety: LcuConnector::sniff_credentials() when League is not running
//!    never launches powershell.exe or wmic.exe, and returns None cleanly within milliseconds.
//! 4. Comprehensive parser & cooldown stress tests.

use lol_hex_rust::lcu::connector::LcuConnector;
use lol_hex_rust::lcu::live_client::LiveClientDataReader;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

static LIVE_SERVER_MUTEX: Mutex<()> = Mutex::const_new(());

/// In-process RAII Mock HTTPS Server on 127.0.0.1:2999 backed by Python & OpenSSL TLS
struct MockLiveClientServer {
    child: Child,
    temp_dir: PathBuf,
}

impl MockLiveClientServer {
    fn start(response_payload: Value) -> Option<Self> {
        let mut bind_ok = false;
        let start_bind = Instant::now();
        while start_bind.elapsed() < Duration::from_millis(600) {
            if std::net::TcpListener::bind("127.0.0.1:2999").is_ok() {
                bind_ok = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        if !bind_ok {
            eprintln!("[INFO] Port 2999 already in use (e.g. active game running). Skipping mock test.");
            return None;
        }

        let temp_dir = std::env::temp_dir().join(format!("lcu_mock_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let payload_file = temp_dir.join("payload.json");
        std::fs::write(&payload_file, response_payload.to_string())
            .expect("Failed to write mock payload.json");

        let cert_pem = r#"-----BEGIN CERTIFICATE-----
MIICtzCCAZ+gAwIBAgICA+gwDQYJKoZIhvcNAQELBQAwFDESMBAGA1UEAwwJMTI3
LjAuMC4xMB4XDTI1MTAxNzA2MTcxOFoXDTM2MTAxNDA2MTcxOFowFDESMBAGA1UE
AwwJMTI3LjAuMC4xMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtB8j
imFh1RON5OUN4bC73+JIhRMPi1TE97yqk/m1gFYUNM3LKhN+DbXG1xWeXxjOebVj
17Ndh5VSbBSyDOjvHioXeThHSIj2aiYCHEHdLaF5S/oEBZoq5yXvLbmQbk27A+Uk
btwno2pRg3Gvm704x2UmwP/MFudhAZdXSxF8C3WbwJ3q/jvOHO5QTUdrCTyPcPdF
jrzv08cKCpykI2g+Ye5ev0PrTAfIMJ46unQmzAyJ4MVp+1+NxD9mEusPwQbfx18Y
4waYgKDRUNuX3NpzwGrO8I0jozdpwEd+FVg137vuwauQpAtBFewHOs+jP9mgMrwn
/+lGymbMmf7Pn+hj9QIDAQABoxMwETAPBgNVHREECDAGhwR/AAABMA0GCSqGSIb3
DQEBCwUAA4IBAQB8rq5z+ZA+MIEuy8N1MErn0iflgCAYGKF2wABkm5VOyoyJ7998
KmAlFGiplpQ6Ck80v6HCJ6w/6APXHwTQ9LWTqPrB9LBqEDHsHN86c09pbt3seI4j
DSV+aIEh/nRf6VCpVRbNimL4gkYSQnOBKc5Y3e0mrBaW8sw1+SKlKlfDEz1FONLX
SjBM/xhqzSyVGlff70VldlI0DiDw1JD7LcF0LHtEnI2dE4XJQd16gjd2QlS265L0
Y8zf/rdzm5mwH3Az5fEcLiCU661o88120wmcQ7kDFWlEGjD8QlooPIYWF8I85E7v
GbsuMhVszC73kDQ7ltne9kAIVlB534xNX4Vo
-----END CERTIFICATE-----"#;

        let key_pem = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAtB8jimFh1RON5OUN4bC73+JIhRMPi1TE97yqk/m1gFYUNM3L
KhN+DbXG1xWeXxjOebVj17Ndh5VSbBSyDOjvHioXeThHSIj2aiYCHEHdLaF5S/oE
BZoq5yXvLbmQbk27A+Ukbtwno2pRg3Gvm704x2UmwP/MFudhAZdXSxF8C3WbwJ3q
/jvOHO5QTUdrCTyPcPdFjrzv08cKCpykI2g+Ye5ev0PrTAfIMJ46unQmzAyJ4MVp
+1+NxD9mEusPwQbfx18Y4waYgKDRUNuX3NpzwGrO8I0jozdpwEd+FVg137vuwauQ
pAtBFewHOs+jP9mgMrwn/+lGymbMmf7Pn+hj9QIDAQABAoIBAAroi914klU2LvS6
zoFpk6c2bA/rHJjilF8whOAbUluoWubrw68TqSGpbZZfD81QkkiarKZGjZaFe41i
H3parQiVogBMYTzKVqoNDTmq33tA+TiZtp6kPI7tJDQPteiIOpg8y10PZjMmYDNm
sizrY8HnpVYw56pHbv+Kq9W3BF0pMvTWRbNGbNTvLXW6EL6WEhCOrsxQSZOUh2p1
owFjIqMxp0lofZMtc1CIVu6LoPpPssHI5G1K+k6yTawHmKI2A5APOUEwJiYKq+Pw
BR6h6EgUq9A3v5zy9tu/nwhFTQV1MmAM98bFA5M4CcbVSXEFyNECLI1eKEF9RXdS
CUi9raECgYEA94F1pgVPb04nIO20KaCo2Mi9iGucekhRKFAv4yCQBeaap2uBnc3N
ibqr33RqGVo3guYRh7F4r3HDdegKJoFqzXlswKqDU1RhiTQ13/r/34hITmwplzUG
kxnrJkndpPW1dfMEhJnVgl6w2oEY6PuLX+fjvDxR45PilCZI1M2cKhkCgYEAuk2n
r7+0Lx5/0qDxt6jELOjOY1qG09okvbzv2iodr3Z4g7kqyykFhA6j5Q2AT8Efla+2
QbyOdwOwvWJ1zG3xe+LTflBJhWcEp9rc/bGKq0+do5y7y5hmpM9aCPujgTEXDyP2
tyzLntZ5WlXMo80H5GS14JpUhoRl+RMvx/IIvD0CgYEA7W4Gg1Yyxo7NYHxHSWuy
rgucLkM1H+VB3Y9W8OZq4w4+K/8on04vL0Gg4UKXCW0PNYYpKOep43wZyoDnQ9sA
rs4d8pFgW87lpi4sxInPbgsZegmuuDwI65KAjwFrG3UON3ECU4YTWPYnZ+VSFSZB
pKYaRJgE4nEaeY5kbag0d/kCgYADSUNf8j23Zw5SJMyetLdhI2ogjoQ6UGq9fgut
CXkkky5lBGsg0kRqo+d4LtjIj1uDRGoulqjcqJABhy8UwAu2Z7XbUIc7dym53D5x
qyQLCYh6TlVxreqBlba82dN0f+0sMzIy4PNmhdn2IfVOHlu5R519rLs49DToQ9Jf
8ID+8QKBgQCSAhjXo2cUkxeTe2Z1qX4/ze3/a3d9m53e1Rcdzx0q1vM/ug9jerfe
mD/VvMubfTOmZCQLrDiJ3Gb+s2x98N2if3GSmzM0RTgaspVhM40prjiLr545Lkjt
GgC7UINjtUuBi+uB4PTZiT2I7WWYjhE2VQn4UZPKlAb1FuL0/FYSGg==
-----END RSA PRIVATE KEY-----"#;

        let cert_path = temp_dir.join("cert.pem");
        let key_path = temp_dir.join("key.pem");
        std::fs::write(&cert_path, cert_pem).expect("Failed to write mock cert.pem");
        std::fs::write(&key_path, key_pem).expect("Failed to write mock key.pem");

        let script = r#"
import sys, os, ssl, json, http.server, socketserver

temp_dir = sys.argv[1]
payload_file = sys.argv[2]

with open(payload_file, 'r', encoding='utf-8') as f:
    resp_body = f.read().encode('utf-8')

cert_path = os.path.join(temp_dir, 'cert.pem')
key_path = os.path.join(temp_dir, 'key.pem')

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/liveclientdata/allgamedata':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(resp_body)))
            self.end_headers()
            self.wfile.write(resp_body)
        else:
            self.send_response(404)
            self.end_headers()
    def log_message(self, format, *args):
        pass

ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(certfile=cert_path, keyfile=key_path)

socketserver.TCPServer.allow_reuse_address = True
httpd = socketserver.TCPServer(('127.0.0.1', 2999), Handler)
httpd.socket = ctx.wrap_socket(httpd.socket, server_side=True)

sys.stdout.write("READY\n")
sys.stdout.flush()

httpd.serve_forever()
"#;

        let script_file = temp_dir.join("server.py");
        std::fs::write(&script_file, script).expect("Failed to write mock server.py");

        let mut child = Command::new("python")
            .arg(&script_file)
            .arg(&temp_dir)
            .arg(&payload_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn Python mock server");

        let stdout = child.stdout.take().expect("Failed to take stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let start = Instant::now();
        loop {
            line.clear();
            if let Ok(n) = reader.read_line(&mut line) {
                if n > 0 && line.trim() == "READY" {
                    break;
                }
            }
            if start.elapsed() > Duration::from_secs(5) {
                let _ = child.kill();
                panic!("MockLiveClientServer timed out waiting for READY signal");
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        Some(Self { child, temp_dir })
    }
}

impl Drop for MockLiveClientServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.temp_dir);
        // Brief pause to ensure socket port 2999 is released by OS
        std::thread::sleep(Duration::from_millis(150));
    }
}

// ==============================================================================================
// 1. Skin ID Behavior Tests: skinID 0, 1, 2, 266001
// ==============================================================================================

#[tokio::test]
async fn test_skin_id_0_base_skin_does_not_corrupt_hero_id() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    let payload = json!({
        "gameData": {
            "gameMode": "CLASSIC",
            "mapNumber": 11,
            "gameTime": 120.5
        },
        "activePlayer": {
            "riotId": "PlayerOne#NA1",
            "level": 3,
            "currentGold": 500.0
        },
        "allPlayers": [
            {
                "riotId": "PlayerOne#NA1",
                "rawChampionName": "game_character_displayname_Aatrox",
                "skinID": 0, // Base skin
                "team": "ORDER",
                "items": [{ "itemID": 1055, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    assert_eq!(stats.hero_name, "Aatrox");
    // Under new architecture, skinID 0 is ignored; hero_id remains empty for Database::resolve_hero_id
    assert_eq!(stats.hero_id, "", "hero_id must not be populated from skinID 0");
    assert_ne!(stats.hero_id, "0", "hero_id must never be '0'");
    assert_eq!(stats.current_items, vec!["1055"]);
}

#[tokio::test]
async fn test_skin_id_1_does_not_become_annie() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // Regressing the bug described in debug.md: skinID = 1 previously mapped hero_id = "1" (Annie)
    let payload = json!({
        "gameData": {
            "gameMode": "ARAM",
            "mapNumber": 12,
            "gameTime": 60.0
        },
        "activePlayer": {
            "riotId": "Warrior#EUW",
            "level": 6,
            "currentGold": 1400.0
        },
        "allPlayers": [
            {
                "riotId": "Warrior#EUW",
                "championName": "Aatrox",
                "rawChampionName": "game_character_displayname_Aatrox",
                "skinID": 1, // Skin slot index 1 (e.g. Justicar Aatrox)
                "team": "ORDER",
                "items": [{ "itemID": 3071, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    assert_eq!(stats.hero_name, "Aatrox");
    // CRITICAL ADVERSARIAL CHECK: hero_id must NOT be Annie ("1")
    assert_ne!(stats.hero_id, "1", "Wearing skin 1 MUST NOT set hero_id to '1' (Annie)!");
    assert_eq!(stats.hero_id, "", "hero_id must remain empty when championId is omitted");
    assert_eq!(stats.current_items, vec!["3071"]);
}

#[tokio::test]
async fn test_skin_id_2_does_not_become_olaf() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // Regressing the bug described in debug.md: skinID = 2 previously mapped hero_id = "2" (Olaf)
    let payload = json!({
        "gameData": {
            "gameMode": "CHERRY",
            "mapNumber": 30,
            "gameTime": 45.0
        },
        "activePlayer": {
            "riotId": "ArenaMaster#KR1",
            "level": 4,
            "currentGold": 2000.0
        },
        "allPlayers": [
            {
                "riotId": "ArenaMaster#KR1",
                "championName": "Aatrox",
                "rawChampionName": "game_character_displayname_Aatrox",
                "skinID": 2, // Skin slot index 2 (e.g. Mecha Aatrox)
                "team": "ORDER",
                "items": [{ "itemID": 3153, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    assert_eq!(stats.hero_name, "Aatrox");
    // CRITICAL ADVERSARIAL CHECK: hero_id must NOT be Olaf ("2")
    assert_ne!(stats.hero_id, "2", "Wearing skin 2 MUST NOT set hero_id to '2' (Olaf)!");
    assert_eq!(stats.hero_id, "", "hero_id must remain empty when championId is omitted");
    assert_eq!(stats.current_items, vec!["3153"]);
}

#[tokio::test]
async fn test_skin_id_266001_legacy_format_does_not_break_parser() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    let payload = json!({
        "gameData": {
            "gameMode": "CLASSIC",
            "mapNumber": 11,
            "gameTime": 240.0
        },
        "activePlayer": {
            "riotId": "LegacyPlayer#KR1",
            "level": 5,
            "currentGold": 850.0
        },
        "allPlayers": [
            {
                "riotId": "LegacyPlayer#KR1",
                "championName": "Aatrox",
                "rawChampionName": "game_character_displayname_Aatrox",
                "skinID": 266001, // 6-digit legacy skin ID
                "team": "ORDER",
                "items": [{ "itemID": 1055, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    assert_eq!(stats.hero_name, "Aatrox");
    // skinID is ignored; hero_id is not set from skinID
    assert_ne!(stats.hero_id, "1");
    assert_ne!(stats.hero_id, "2");
    assert_eq!(stats.hero_id, "");
}

#[tokio::test]
async fn test_skin_id_with_explicit_champion_id_preserves_champion() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // When both skinID (1 or 2) AND championId (266) are present:
    // championId must be preserved as "266", and skinID 1 / 2 must NEVER override it to "1" or "2"
    let payload = json!({
        "gameData": { "gameMode": "CLASSIC", "mapNumber": 11, "gameTime": 300.0 },
        "activePlayer": { "riotId": "AatroxGod#CN1", "level": 9, "currentGold": 3200.0 },
        "allPlayers": [
            {
                "riotId": "AatroxGod#CN1",
                "championName": "Aatrox",
                "skinID": 1,
                "championId": 266,
                "team": "ORDER",
                "items": [{ "itemID": 3071, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert_eq!(stats.hero_name, "Aatrox");
    assert_eq!(stats.hero_id, "266", "Explicit championId 266 must be preserved!");
    assert_ne!(stats.hero_id, "1", "Must not override to '1' despite skinID = 1");
}

// ==============================================================================================
// 2. Riot ID Disambiguation: Duplicate gameNames ("TestPlayer#TAG1" vs "TestPlayer#TAG2")
// ==============================================================================================

#[tokio::test]
async fn test_riot_id_disambiguation_same_gamename_different_tags() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // Both players share the identical gameName "TestPlayer"
    // Player 0 has TAG1, Order team, Aatrox, Items [1055, 3006]
    // Player 1 has TAG2, Chaos team, Camille, Items [3078, 3053]
    // Active player is "TestPlayer#TAG1"
    let payload = json!({
        "gameData": {
            "gameMode": "CLASSIC",
            "mapNumber": 11,
            "gameTime": 500.0
        },
        "activePlayer": {
            "riotId": "TestPlayer#TAG1",
            "level": 6,
            "currentGold": 1200.0
        },
        "allPlayers": [
            {
                "riotId": "TestPlayer#TAG1",
                "championName": "Aatrox",
                "team": "ORDER",
                "items": [
                    { "itemID": 1055, "count": 1 },
                    { "itemID": 3006, "count": 1 }
                ]
            },
            {
                "riotId": "TestPlayer#TAG2",
                "championName": "Camille",
                "team": "CHAOS",
                "items": [
                    { "itemID": 3078, "count": 1 },
                    { "itemID": 3053, "count": 1 }
                ]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    // Local player must match Player 0 (Aatrox)
    assert_eq!(stats.hero_name, "Aatrox");
    // Local player inventory must ONLY contain Player 0 items; Player 1 items must NOT be present
    assert_eq!(stats.current_items, vec!["1055", "3006"]);
    // Enemy team must contain Camille (Player 1)
    assert_eq!(stats.enemy_team.len(), 1);
    assert_eq!(
        stats.enemy_team[0].get("championName").and_then(|v| v.as_str()),
        Some("Camille")
    );
}

#[tokio::test]
async fn test_riot_id_disambiguation_reverse_player_order() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // Adversarial order: Conflicting player (TAG2) appears FIRST in allPlayers array.
    // If the matcher is naive and matches on "TestPlayer", it would falsely grab Player 0.
    // Strict matcher must reject Player 0 and correctly locate Player 1 (TAG1).
    let payload = json!({
        "gameData": {
            "gameMode": "CLASSIC",
            "mapNumber": 11,
            "gameTime": 500.0
        },
        "activePlayer": {
            "riotId": "TestPlayer#TAG1",
            "level": 6,
            "currentGold": 1200.0
        },
        "allPlayers": [
            {
                "riotId": "TestPlayer#TAG2", // APPEARS FIRST
                "championName": "Camille",
                "team": "CHAOS",
                "items": [
                    { "itemID": 3078, "count": 1 },
                    { "itemID": 3053, "count": 1 }
                ]
            },
            {
                "riotId": "TestPlayer#TAG1", // LOCAL PLAYER SECOND
                "championName": "Aatrox",
                "team": "ORDER",
                "items": [
                    { "itemID": 1055, "count": 1 },
                    { "itemID": 3006, "count": 1 }
                ]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert!(stats.is_in_game);
    // Local player must still strictly resolve to Aatrox (TAG1)
    assert_eq!(stats.hero_name, "Aatrox");
    assert_eq!(stats.current_items, vec!["1055", "3006"]);
    assert_eq!(stats.enemy_team.len(), 1);
    assert_eq!(
        stats.enemy_team[0].get("championName").and_then(|v| v.as_str()),
        Some("Camille")
    );
}

#[tokio::test]
async fn test_riot_id_disambiguation_via_separate_fields() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // Test with separated riotIdGameName / riotIdTagLine fields
    let payload = json!({
        "gameData": { "gameMode": "CLASSIC", "mapNumber": 11, "gameTime": 100.0 },
        "activePlayer": {
            "riotIdGameName": "Faker",
            "riotIdTagLine": "T1",
            "summonerName": "Hide on bush"
        },
        "allPlayers": [
            {
                "riotIdGameName": "Faker",
                "riotIdTagLine": "IMPOSTOR",
                "championName": "Zed",
                "team": "CHAOS",
                "items": [{ "itemID": 3142, "count": 1 }]
            },
            {
                "riotIdGameName": "Faker",
                "riotIdTagLine": "T1",
                "championName": "Ahri",
                "team": "ORDER",
                "items": [{ "itemID": 3089, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert_eq!(stats.hero_name, "Ahri");
    assert_eq!(stats.current_items, vec!["3089"]);
    assert_eq!(stats.enemy_team.len(), 1);
    assert_eq!(
        stats.enemy_team[0].get("championName").and_then(|v| v.as_str()),
        Some("Zed")
    );
}

#[tokio::test]
async fn test_puuid_strict_precedence_over_names() {
    let _lock = LIVE_SERVER_MUTEX.lock().await;

    // If both activePlayer and allPlayers have PUUIDs:
    // Differing PUUIDs must reject immediately even if Riot ID or summonerName are identical!
    let payload = json!({
        "gameData": { "gameMode": "CLASSIC", "mapNumber": 11, "gameTime": 100.0 },
        "activePlayer": {
            "puuid": "puuid-local-real-1234",
            "riotId": "CloneMaster#NA1"
        },
        "allPlayers": [
            {
                "puuid": "puuid-remote-fake-9999", // Different PUUID
                "riotId": "CloneMaster#NA1",      // Same Riot ID!
                "championName": "LeBlanc",
                "team": "CHAOS",
                "items": [{ "itemID": 1056, "count": 1 }]
            },
            {
                "puuid": "puuid-local-real-1234", // Matching PUUID
                "riotId": "CloneMaster#NA1",
                "championName": "Shaco",
                "team": "ORDER",
                "items": [{ "itemID": 1055, "count": 1 }]
            }
        ]
    });

    let _server = match MockLiveClientServer::start(payload) {
        Some(s) => s,
        None => return,
    };
    let reader = LiveClientDataReader::new();
    let stats = reader.get_live_stats().await;

    assert_eq!(stats.hero_name, "Shaco");
    assert_eq!(stats.current_items, vec!["1055"]);
    assert_eq!(stats.enemy_team.len(), 1);
    assert_eq!(
        stats.enemy_team[0].get("championName").and_then(|v| v.as_str()),
        Some("LeBlanc")
    );
}

// ==============================================================================================
// 3. Process Sniffing Safety: Zero Subprocess Overhead & Rapid Termination
// ==============================================================================================

#[test]
fn test_process_sniffing_no_league_returns_none_in_milliseconds() {
    LcuConnector::reset_cooldown();

    let start = Instant::now();
    let creds = LcuConnector::sniff_credentials();
    let elapsed = start.elapsed();

    if creds.is_none() {
        // In an environment where League is not running:
        // Must return None immediately via Win32 snapshot check without invoking WMIC/PowerShell subprocesses.
        assert!(
            elapsed < Duration::from_millis(50),
            "sniff_credentials took {:?}, exceeding 50ms! PowerShell/WMIC subprocess detected when League not running!",
            elapsed
        );
        println!("Verified: League not running, sniff_credentials returned None in {:?}", elapsed);
    } else {
        println!("Verified: League is actively running on this host, sniffed credentials in {:?}", elapsed);
    }
}

#[test]
fn test_process_sniffing_cooldown_mechanism() {
    LcuConnector::reset_cooldown();

    // Call 1: probes snapshot (may find League if running, or None if not)
    let _first = LcuConnector::sniff_credentials();

    // Call 2: within 3-second cooldown window -> MUST return None due to cooldown!
    let start2 = Instant::now();
    let second = LcuConnector::sniff_credentials();
    let elapsed2 = start2.elapsed();

    assert!(second.is_none(), "Second call within cooldown must return None");
    // Under cooldown, function returns at the very top mutex guard (< 100 microseconds)
    assert!(
        elapsed2 < Duration::from_millis(10),
        "Cooldown check took {:?}, must return immediately",
        elapsed2
    );

    // Reset cooldown and confirm it allows re-probing
    LcuConnector::reset_cooldown();
    let _third = LcuConnector::sniff_credentials();
}

#[tokio::test]
async fn test_process_sniffing_async_wrapper() {
    LcuConnector::reset_cooldown();

    let start = Instant::now();
    let _creds = LcuConnector::sniff_credentials_async().await;
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(2000),
        "sniff_credentials_async took {:?}, exceeding threshold",
        elapsed
    );
}

// ==============================================================================================
// 4. Connector Command Line & Lockfile Parsing Stress Tests
// ==============================================================================================

#[test]
fn test_parse_commandline_adversarial_variations() {
    // 1. Standard quoted
    let cmd1 = r#""LeagueClientUx.exe" "--app-port=51842" "--remoting-auth-token=test_token_123""#;
    let c1 = LcuConnector::parse_commandline(cmd1).unwrap();
    assert_eq!(c1.port, 51842);
    assert_eq!(c1.auth_token, "test_token_123");

    // 2. Unquoted values with surrounding extra flags
    let cmd2 = "LeagueClientUx.exe --install-directory=C:\\Riot --app-port=49999 --remoting-auth-token=secretTokenABC --launch-product=league_of_legends";
    let c2 = LcuConnector::parse_commandline(cmd2).unwrap();
    assert_eq!(c2.port, 49999);
    assert_eq!(c2.auth_token, "secretTokenABC");

    // 3. Single quotes with metadata
    let cmd3 = "LeagueClientUx.exe --region='TENCENT' --app-port='55555' --remoting-auth-token='tok_xyz' --rso-platform-id='HN10'";
    let c3 = LcuConnector::parse_commandline(cmd3).unwrap();
    assert_eq!(c3.port, 55555);
    assert_eq!(c3.auth_token, "tok_xyz");

    let (region, rso) = LcuConnector::parse_commandline_metadata(cmd3);
    assert_eq!(region.as_deref(), Some("TENCENT"));
    assert_eq!(rso.as_deref(), Some("HN10"));

    // 4. Missing port
    let cmd_no_port = "LeagueClientUx.exe --remoting-auth-token=foo";
    assert!(LcuConnector::parse_commandline(cmd_no_port).is_none());

    // 5. Missing token
    let cmd_no_token = "LeagueClientUx.exe --app-port=12345";
    assert!(LcuConnector::parse_commandline(cmd_no_token).is_none());

    // 6. Zero port
    let cmd_zero_port = "LeagueClientUx.exe --app-port=0 --remoting-auth-token=bar";
    assert!(LcuConnector::parse_commandline(cmd_zero_port).is_none());

    // 7. Non-numeric port
    let cmd_invalid_port = "LeagueClientUx.exe --app-port=invalid --remoting-auth-token=bar";
    assert!(LcuConnector::parse_commandline(cmd_invalid_port).is_none());

    // 8. Empty string
    assert!(LcuConnector::parse_commandline("").is_none());
}

#[test]
fn test_parse_lockfile_adversarial_variations() {
    // 1. Standard 5-field lockfile
    let valid = "LeagueClient:12345:51842:secret_token_123:https\n";
    let c1 = LcuConnector::parse_lockfile(valid).unwrap();
    assert_eq!(c1.port, 51842);
    assert_eq!(c1.auth_token, "secret_token_123");

    // 2. Extra whitespace and trailing CRLF
    let valid_crlf = "  LeagueClient:999:50000:my_token:https  \r\n";
    let c2 = LcuConnector::parse_lockfile(valid_crlf).unwrap();
    assert_eq!(c2.port, 50000);
    assert_eq!(c2.auth_token, "my_token");

    // 3. Incomplete fields (less than 5 parts)
    assert!(LcuConnector::parse_lockfile("LeagueClient:12345").is_none());
    assert!(LcuConnector::parse_lockfile("LeagueClient:12345:50000").is_none());

    // 4. Zero port
    assert!(LcuConnector::parse_lockfile("LeagueClient:12345:0:tok:https").is_none());

    // 5. Empty token
    assert!(LcuConnector::parse_lockfile("LeagueClient:12345:50000::https").is_none());

    // 6. Non-numeric port
    assert!(LcuConnector::parse_lockfile("LeagueClient:12345:notaport:tok:https").is_none());

    // 7. Empty string
    assert!(LcuConnector::parse_lockfile("").is_none());
}
