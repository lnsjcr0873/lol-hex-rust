#![recursion_limit = "512"]

mod lcu_mock_harness;

use lcu_mock_harness::MockLcuServer;
use lol_hex_rust::lcu::client::{LcuClient, LruStore};
use lol_hex_rust::sgp::{
    extract_entitlements_token, extract_league_session_token, extract_sgp_tokens,
    get_builtin_sgp_servers, get_sgp_server_id, resolve_server_config, resolve_sub_id,
    GsmLedgeRegionResponse, SgpClient, SgpTokens,
};
use serde_json::json;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_raw_match_history_lossless_preservation() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "lossless-player-puuid-12345";

    // Build raw Riot LCU match history payload with deep nested fields
    let raw_payload = json!({
        "accountId": 987654321,
        "platformId": "HN1",
        "games": {
            "games": [
                {
                    "gameId": 555666777,
                    "gameCreation": 1713000000000i64,
                    "gameDuration": 1820,
                    "queueId": 420,
                    "mapId": 11,
                    "gameMode": "CLASSIC",
                    "participantIdentities": [
                        {
                            "participantId": 1,
                            "player": {
                                "puuid": target_puuid,
                                "summonerName": "LosslessMaster",
                                "gameName": "LosslessMaster",
                                "tagLine": "0001"
                            }
                        }
                    ],
                    "participants": [
                        {
                            "participantId": 1,
                            "teamId": 100,
                            "championId": 103, // Ahri
                            "spell1Id": 4,     // Flash
                            "spell2Id": 14,    // Ignite
                            "stats": {
                                "win": true,
                                "kills": 12,
                                "deaths": 2,
                                "assists": 9,
                                "totalDamageDealtToChampions": 34500,
                                "physicalDamageDealtToChampions": 1200,
                                "magicDamageDealtToChampions": 31000,
                                "trueDamageDealtToChampions": 2300,
                                "goldEarned": 16500,
                                "totalMinionsKilled": 210,
                                "neutralMinionsKilled": 12,
                                "visionScore": 42,
                                "timeCCingOthers": 35,
                                // Keystone runes & variable contributions
                                "perk0": 8112,      // Electrocute
                                "perk0Var1": 1845,  // Actual Electrocute damage!
                                "perk0Var2": 0,
                                "perk0Var3": 0,
                                "perk1": 8143,      // Sudden Impact
                                "perk1Var1": 820,
                                "perkPrimaryStyle": 8100, // Domination
                                "perkSubStyle": 8200      // Sorcery
                            },
                            "timeline": {
                                "creepsPerMinDeltas": { "0-10": 8.5, "10-20": 9.2 },
                                "goldPerMinDeltas": { "0-10": 340.0, "10-20": 510.5 }
                            }
                        }
                    ]
                }
            ]
        }
    });

    server.state.set_match_history(target_puuid, raw_payload.clone()).await;

    // Fetch through native lossless LCU endpoint
    let res = client
        .get_raw_match_history(target_puuid, 0, 19)
        .await
        .expect("get_raw_match_history returned None");

    // 1. Verify root container preservation
    assert_eq!(res["accountId"], 987654321);
    assert_eq!(res["platformId"], "HN1");

    // 2. Verify game and participant preservation
    let game = &res["games"]["games"][0];
    assert_eq!(game["gameId"], 555666777);
    assert_eq!(game["participants"][0]["championId"], 103);

    // 3. Verify 100% genuine keystone rune values without zeroing out
    let stats = &game["participants"][0]["stats"];
    assert_eq!(stats["perk0"], 8112);
    assert_eq!(stats["perk0Var1"], 1845, "perk0Var1 must retain exact damage stat");
    assert_eq!(stats["perk1Var1"], 820);
    assert_eq!(stats["perkPrimaryStyle"], 8100);
    assert_eq!(stats["perkSubStyle"], 8200);

    // 4. Verify minute deltas in timeline are preserved
    let deltas = &game["participants"][0]["timeline"]["creepsPerMinDeltas"];
    assert_eq!(deltas["0-10"], 8.5);
    assert_eq!(deltas["10-20"], 9.2);

    server.stop();
}

#[tokio::test]
async fn test_raw_match_history_chunking_large_range() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let target_puuid = "chunking-tester-puuid";

    // Build a mock dataset of 50 matches (game IDs 1001..=1050)
    let mut all_games = Vec::new();
    for i in 0..50 {
        all_games.push(json!({
            "gameId": 1000 + i,
            "gameCreation": 1713000000000i64 + (i * 3600000),
            "gameDuration": 1500,
            "queueId": 420,
            "gameMode": "CLASSIC",
            "participants": [{
                "participantId": 1,
                "championId": 10 + (i % 20),
                "stats": {
                    "win": i % 2 == 0,
                    "kills": 5,
                    "deaths": 2,
                    "assists": 8,
                    "perk0Var1": 1000 + i
                }
            }]
        }));
    }

    let full_history = json!({
        "accountId": 777888999,
        "games": {
            "games": all_games
        }
    });

    server.state.set_match_history(target_puuid, full_history).await;

    let start_requests = server.state.get_request_count();

    // Request 50 games (beg_index: 0, end_index: 49)
    // The native client must chunk in batches of 20: 0..19, 20..39, 40..49 (3 requests)
    let res = client
        .get_raw_match_history(target_puuid, 0, 49)
        .await
        .expect("get_raw_match_history returned None");

    let merged_games = res["games"]["games"]
        .as_array()
        .expect("games.games must be array");

    assert_eq!(merged_games.len(), 50, "Merged result must contain all 50 games");
    assert_eq!(merged_games[0]["gameId"], 1000);
    assert_eq!(merged_games[49]["gameId"], 1049);
    assert_eq!(merged_games[49]["participants"][0]["stats"]["perk0Var1"], 1049);

    let end_requests = server.state.get_request_count();
    let requests_made = end_requests - start_requests;
    assert_eq!(requests_made, 3, "Range 0..49 must be requested in exactly 3 chunks of 20");

    server.stop();
}

#[tokio::test]
async fn test_raw_match_timeline_preservation() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let game_id = 9988776655u64;

    let raw_timeline = json!({
        "gameId": game_id,
        "frameInterval": 60000,
        "frames": [
            {
                "timestamp": 0,
                "events": [],
                "participantFrames": {
                    "1": {
                        "participantId": 1,
                        "position": { "x": 560, "y": 580 },
                        "currentGold": 500,
                        "totalGold": 500,
                        "level": 1,
                        "xp": 0,
                        "minionsKilled": 0,
                        "jungleMinionsKilled": 0
                    }
                }
            },
            {
                "timestamp": 60000,
                "events": [
                    {
                        "type": "ITEM_PURCHASED",
                        "timestamp": 12500,
                        "participantId": 1,
                        "itemId": 1055 // Doran's Blade
                    },
                    {
                        "type": "SKILL_LEVEL_UP",
                        "timestamp": 45000,
                        "participantId": 1,
                        "skillSlot": 1,
                        "levelUpType": "NORMAL"
                    },
                    {
                        "type": "CHAMPION_KILL",
                        "timestamp": 58200,
                        "killerId": 1,
                        "victimId": 6,
                        "assistingParticipantIds": [2, 3],
                        "position": { "x": 4820, "y": 7890 }
                    }
                ],
                "participantFrames": {
                    "1": {
                        "participantId": 1,
                        "position": { "x": 4820, "y": 7890 },
                        "currentGold": 450,
                        "totalGold": 950,
                        "level": 2,
                        "xp": 380,
                        "minionsKilled": 6,
                        "jungleMinionsKilled": 0
                    }
                }
            }
        ]
    });

    server.state.set_match_timeline(game_id, raw_timeline.clone()).await;

    let res = client
        .get_raw_match_timeline(game_id)
        .await
        .expect("get_raw_match_timeline returned None");

    assert_eq!(res["gameId"], game_id);
    assert_eq!(res["frameInterval"], 60000);

    let frames = res["frames"].as_array().expect("frames array");
    assert_eq!(frames.len(), 2);

    let events = frames[1]["events"].as_array().expect("frame 1 events");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["type"], "ITEM_PURCHASED");
    assert_eq!(events[0]["itemId"], 1055);
    assert_eq!(events[1]["type"], "SKILL_LEVEL_UP");
    assert_eq!(events[1]["skillSlot"], 1);
    assert_eq!(events[2]["type"], "CHAMPION_KILL");
    assert_eq!(events[2]["killerId"], 1);
    assert_eq!(events[2]["victimId"], 6);
    assert_eq!(events[2]["position"]["x"], 4820);

    server.stop();
}

#[tokio::test]
async fn test_in_memory_lru_cache_hits_and_eviction() {
    let mut store = LruStore::<u64, String>::new(256);

    // Insert 256 entries
    for i in 1..=256 {
        store.insert(i, format!("match_{}", i));
    }
    assert_eq!(store.len(), 256);

    // Access key 1 to promote it to most-recently-used
    assert_eq!(store.get(&1), Some("match_1".to_string()));

    // Insert 257th entry - should evict key 2 (since key 1 was promoted)
    store.insert(257, "match_257".to_string());
    assert_eq!(store.len(), 256);

    // Key 1 should still exist because it was accessed recently
    assert!(store.contains_key(&1));
    // Key 2 should have been evicted as the oldest
    assert!(!store.contains_key(&2));
    // Key 257 must be present
    assert!(store.contains_key(&257));

    // Integration test with LcuClient
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let test_game_id = 888999111u64;
    let timeline_data = json!({
        "gameId": test_game_id,
        "frames": []
    });
    server.state.set_match_timeline(test_game_id, timeline_data).await;

    let reqs_before = server.state.get_request_count();

    // First lookup: misses cache, queries server
    let t1 = client.get_raw_match_timeline(test_game_id).await;
    assert!(t1.is_some());
    let reqs_after_first = server.state.get_request_count();
    assert_eq!(reqs_after_first, reqs_before + 1);

    // Second lookup: hits cache instantly without issuing any network request!
    let t2 = client.get_raw_match_timeline(test_game_id).await;
    assert!(t2.is_some());
    let reqs_after_second = server.state.get_request_count();
    assert_eq!(reqs_after_second, reqs_after_first, "Repeat lookup must hit LRU cache with 0 network calls");

    server.stop();
}

#[tokio::test]
async fn test_concurrency_limiter_semaphore() {
    let client = LcuClient::new();

    // Initial permits should be 4
    assert_eq!(client.concurrency_limiter.available_permits(), 4);

    // Acquire 4 permits
    let p1 = client.concurrency_limiter.acquire().await.unwrap();
    let p2 = client.concurrency_limiter.acquire().await.unwrap();
    let p3 = client.concurrency_limiter.acquire().await.unwrap();
    let p4 = client.concurrency_limiter.acquire().await.unwrap();

    assert_eq!(client.concurrency_limiter.available_permits(), 0);

    // 5th acquisition must block and timeout
    let acquire_attempt = timeout(
        Duration::from_millis(50),
        client.concurrency_limiter.acquire(),
    )
    .await;
    assert!(acquire_attempt.is_err(), "Must block when 4 permits are in flight");

    // Releasing one permit allows acquisition
    drop(p1);
    assert_eq!(client.concurrency_limiter.available_permits(), 1);

    let p5 = timeout(
        Duration::from_millis(50),
        client.concurrency_limiter.acquire(),
    )
    .await;
    assert!(p5.is_ok(), "Must acquire successfully once permit is released");

    drop(p2);
    drop(p3);
    drop(p4);
    drop(p5);
    assert_eq!(client.concurrency_limiter.available_permits(), 4);
}

#[test]
fn test_sgp_server_id_normalization_and_routing() {
    // 1. Tencent cluster server ID resolution
    assert_eq!(get_sgp_server_id("TENCENT", Some("HN1")), "TENCENT_HN1");
    assert_eq!(get_sgp_server_id("tencent", Some("hn10")), "TENCENT_HN10");
    assert_eq!(get_sgp_server_id("TENCENT", Some("bgp2")), "TENCENT_BGP2");
    assert_eq!(get_sgp_server_id("TENCENT", Some("tj100")), "TENCENT_TJ100");

    // 2. Riot global region alias resolution
    assert_eq!(get_sgp_server_id("NA", None), "NA1");
    assert_eq!(get_sgp_server_id("EUW1", None), "EUW");
    assert_eq!(get_sgp_server_id("OCE", None), "OC1");
    assert_eq!(get_sgp_server_id("LAN", None), "LA1");
    assert_eq!(get_sgp_server_id("LAS", None), "LA2");
    assert_eq!(get_sgp_server_id("JP1", None), "JP");
    assert_eq!(get_sgp_server_id("KR", None), "KR");

    // 3. Builtin servers table check
    let servers = get_builtin_sgp_servers();
    assert!(servers.contains_key("TENCENT_HN1"));
    assert!(servers.contains_key("TENCENT_BGP2"));
    assert!(servers.contains_key("EUW"));
    assert!(servers.contains_key("NA1"));
    assert!(servers.contains_key("KR"));

    // 4. Server URL resolution
    let hn1_cfg = resolve_server_config("TENCENT_HN1").expect("TENCENT_HN1 config");
    assert_eq!(hn1_cfg.match_history, "https://hn1-k8s-sgp.lol.qq.com:21019");
    assert!(hn1_cfg.is_tencent);

    let euw_cfg = resolve_server_config("EUW").expect("EUW config");
    assert_eq!(euw_cfg.match_history, "https://euc1-red.pp.sgp.pvp.net");
    assert_eq!(euw_cfg.common, "https://euw-red.lol.sgp.pvp.net");
    assert_eq!(euw_cfg.region_path_param, Some("EUW1".to_string()));

    // 5. Dynamic fallback for unlisted Tencent shard (e.g. HN2)
    let hn2_cfg = resolve_server_config("TENCENT_HN2").expect("TENCENT_HN2 dynamic config");
    assert_eq!(hn2_cfg.match_history, "https://hn2-k8s-sgp.lol.qq.com:21019");

    // 6. Sub-ID resolution
    assert_eq!(resolve_sub_id("EUW", &euw_cfg, None), "EUW1");
    assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, Some("HN1")), "HN1");
    assert_eq!(resolve_sub_id("TENCENT_HN1", &hn1_cfg, None), "HN1");
}

#[tokio::test]
async fn test_sgp_tokens_extraction() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    // Set custom tokens in mock server
    let custom_entitlements = json!({
        "accessToken": "custom_jwt_access_token_xyz_999",
        "entitlements": ["lol", "sgp"],
        "issuer": "https://auth.riotgames.com",
        "subject": "player-puuid-custom",
        "token": "custom_token_val"
    });
    server.state.set_entitlements_token(custom_entitlements).await;
    server.state.set_league_session_token("custom_jwt_session_token_abc_123").await;

    // Test token extraction
    let ent = extract_entitlements_token(&client).await.expect("entitlements token");
    assert_eq!(ent.access_token, "custom_jwt_access_token_xyz_999");
    assert_eq!(ent.subject.as_deref(), Some("player-puuid-custom"));

    let sess = extract_league_session_token(&client).await.expect("league session token");
    assert_eq!(sess, "custom_jwt_session_token_abc_123");

    let all_tokens = extract_sgp_tokens(&client).await;
    assert!(all_tokens.has_entitlements());
    assert!(all_tokens.has_league_session());
    assert_eq!(all_tokens.access_token(), Some("custom_jwt_access_token_xyz_999"));
    assert_eq!(all_tokens.session_token(), Some("custom_jwt_session_token_abc_123"));

    server.stop();
}

#[tokio::test]
async fn test_sgp_gsm_and_client_skeleton() {
    // 1. GSM Live Query model deserialization test
    let gsm_raw = json!({
        "gameName": "NORMAL_5v5",
        "reconnectDelay": 5000,
        "game": {
            "id": 12345678,
            "gameState": "in_game",
            "gameMode": "CLASSIC",
            "queueTypeName": "RANKED_SOLO_5x5",
            "teamOne": [
                {
                    "puuid": "blue-top-puuid",
                    "summonerId": 1001,
                    "teamParticipantId": 1,
                    "championId": 266, // Aatrox
                    "selectedRole": "SOLO",
                    "selectedPosition": "TOP",
                    "summonerName": "BlueTopPlayer"
                }
            ],
            "teamTwo": [
                {
                    "puuid": "red-top-puuid",
                    "summonerId": 2001,
                    "teamParticipantId": 6,
                    "championId": 875, // Sett
                    "selectedRole": "SOLO",
                    "selectedPosition": "TOP",
                    "summonerName": "RedTopPlayer"
                }
            ],
            "playerChampionSelections": [
                {
                    "puuid": "blue-top-puuid",
                    "championId": 266,
                    "spell1Id": 4,  // Flash
                    "spell2Id": 12  // Teleport
                },
                {
                    "puuid": "red-top-puuid",
                    "championId": 875,
                    "spell1Id": 4,  // Flash
                    "spell2Id": 14  // Ignite
                }
            ]
        }
    });

    let gsm: GsmLedgeRegionResponse = serde_json::from_value(gsm_raw).expect("GSM model parsing");
    assert_eq!(gsm.game_name.as_deref(), Some("NORMAL_5v5"));
    let game = gsm.game.expect("game info");
    assert_eq!(game.id, Some(12345678));
    assert_eq!(game.team_one.len(), 1);
    assert_eq!(game.team_one[0].selected_position.as_deref(), Some("TOP"));
    assert_eq!(game.player_champion_selections.len(), 2);
    assert_eq!(game.player_champion_selections[0].spell1_id, 4);
    assert_eq!(game.player_champion_selections[0].spell2_id, 12);
    assert_eq!(game.player_champion_selections[1].spell2_id, 14);

    // 2. SgpClient setup and sub-ID verification
    let tokens = SgpTokens {
        entitlements: None,
        league_session_token: Some("mock_sess".to_string()),
    };
    let sgp_client = SgpClient::with_credentials("TENCENT_HN1", Some("HN1"), tokens);
    assert_eq!(sgp_client.current_sub_id().await, "HN1");

    sgp_client.set_server("EUW", None).await;
    assert_eq!(sgp_client.current_sub_id().await, "EUW1");
}
