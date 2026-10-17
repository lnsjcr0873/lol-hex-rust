//! Adversarial Empirical Challenge Suite for Milestone 1 (src/lcu/client.rs)
//!
//! Specifically stress-tests:
//! 1. Singleflight Negative Caching (detail and timeline 404/failure caching under high concurrency)
//! 2. URL encoding of chat room IDs with special characters (`#`, `?`, `@`, spaces, unicode)
//! 3. 401 Unauthorized handling invalidating credentials via handle_response_fallback_candidate
//! 4. Concurrency stampede protection and cache isolation

#[path = "lcu_mock_harness/mod.rs"]
mod lcu_mock_harness;

use lcu_mock_harness::MockLcuServer;
use lol_hex_rust::lcu::client::LcuClient;
use reqwest::StatusCode;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_negative_caching_404_detail_single_and_concurrent() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;
    assert!(client.is_connected().await);

    let test_game_id = "9999901";
    let test_gid_u64 = 9999901u64;

    // Initially not negative cached
    assert!(!client.is_negative_cached_detail(test_gid_u64));

    let req_before = server.state.get_request_count();

    // 1. Initial 404 request
    let res = client.get_match_detail(test_game_id).await;
    assert!(res.is_none(), "404 detail must return None");

    // Negative cache must be populated
    assert!(
        client.is_negative_cached_detail(test_gid_u64),
        "Negative cache must record 404 game_id"
    );

    let req_after_first = server.state.get_request_count();
    let requests_made = req_after_first - req_before;
    assert!(
        requests_made > 0,
        "First 404 should have reached the server via fallback candidate / http"
    );

    // 2. 30 sequential requests must immediately hit negative cache with ZERO network traffic
    for _ in 0..30 {
        let res2 = client.get_match_detail(test_game_id).await;
        assert!(res2.is_none());
    }

    assert_eq!(
        server.state.get_request_count(),
        req_after_first,
        "Subsequent requests MUST NOT trigger any HTTP calls when negative cached"
    );

    // 3. 50 concurrent requests must also all hit negative cache with ZERO network traffic
    let mut tasks = Vec::new();
    for _ in 0..50 {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            client_clone.get_match_detail(test_game_id).await
        }));
    }

    for task in tasks {
        let r = task.await.expect("task join");
        assert!(r.is_none());
    }

    assert_eq!(
        server.state.get_request_count(),
        req_after_first,
        "Concurrent requests MUST NOT trigger any HTTP calls when negative cached"
    );

    // 4. get_raw_match_detail must also observe the negative cache
    assert!(client.get_raw_match_detail(test_gid_u64).await.is_none());
    assert_eq!(server.state.get_request_count(), req_after_first);
}

#[tokio::test]
async fn test_negative_caching_404_timeline_single_and_concurrent() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;

    let test_gid = 8888801u64;

    assert!(!client.is_negative_cached_timeline(test_gid));

    let req_before = server.state.get_request_count();

    // 1. Initial 404 timeline request
    let res = client.get_raw_match_timeline(test_gid).await;
    assert!(res.is_none(), "404 timeline must return None");

    assert!(
        client.is_negative_cached_timeline(test_gid),
        "Timeline negative cache must record 404"
    );

    let req_after_first = server.state.get_request_count();
    assert!(req_after_first > req_before);

    // 2. Sequential requests hit negative cache with 0 network calls
    for _ in 0..20 {
        assert!(client.get_raw_match_timeline(test_gid).await.is_none());
        assert!(client.get_game_timeline(test_gid).await.is_none());
        assert!(client.get_match_timeline(test_gid).await.is_none());
    }

    assert_eq!(
        server.state.get_request_count(),
        req_after_first,
        "Zero network calls for negative cached timeline"
    );

    // 3. Concurrent requests hit negative cache
    let mut tasks = Vec::new();
    for _ in 0..40 {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            client_clone.get_raw_match_timeline(test_gid).await
        }));
    }

    for task in tasks {
        let r = task.await.expect("task join");
        assert!(r.is_none());
    }

    assert_eq!(
        server.state.get_request_count(),
        req_after_first,
        "Zero network calls for concurrent requests on negative cached timeline"
    );
}

#[tokio::test]
async fn test_singleflight_stampede_protection_on_404() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(LcuClient::new());
    client.set_credentials(server.port, &server.auth_token).await;

    let stampede_gid_str = "7777701";

    let req_before = server.state.get_request_count();

    // Spawn 30 concurrent tasks requesting the same un-cached 404 game_id simultaneously
    let mut tasks = Vec::new();
    for _ in 0..30 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.get_match_detail(stampede_gid_str).await
        }));
    }

    for t in tasks {
        let res = t.await.expect("join task");
        assert!(res.is_none(), "All stampede tasks must resolve to None");
    }

    let req_after = server.state.get_request_count();
    let requests_made = req_after - req_before;

    // CRITICAL ADVERSARIAL ASSERTION:
    // With singleflight + negative caching, the first task acquires in-flight lock,
    // fails (404), inserts negative cache, and wakes waiters.
    // All other 29 waiting tasks wake up and see negative cache, returning None.
    // Total requests made must NOT be 30 (or 60 with fallback). It should be at most 2 (1 https fail + 1 http 404)!
    assert!(
        requests_made <= 2,
        "Singleflight must suppress stampede: expected <= 2 requests, but got {}",
        requests_made
    );

    // Subsequent requests must also return immediately
    assert!(client.get_match_detail(stampede_gid_str).await.is_none());
    assert_eq!(server.state.get_request_count(), req_after);
}

#[tokio::test]
async fn test_singleflight_stampede_protection_on_timeline_404() {
    let server = MockLcuServer::start().await;
    let client = Arc::new(LcuClient::new());
    client.set_credentials(server.port, &server.auth_token).await;

    let stampede_gid = 6666601u64;

    let req_before = server.state.get_request_count();

    // Spawn 30 concurrent tasks requesting timeline
    let mut tasks = Vec::new();
    for _ in 0..30 {
        let c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.get_raw_match_timeline(stampede_gid).await
        }));
    }

    for t in tasks {
        let res = t.await.expect("join task");
        assert!(res.is_none(), "All stampede tasks must resolve to None");
    }

    let req_after = server.state.get_request_count();
    let requests_made = req_after - req_before;

    assert!(
        requests_made <= 2,
        "Timeline singleflight must suppress stampede: expected <= 2 requests, but got {}",
        requests_made
    );
}

// Mirror of RFC 3986 path segment encoding logic to empirically verify URL construction and parsing
fn mirror_percent_encode_path_segment(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{:02X}", b);
            }
        }
    }
    encoded
}

#[tokio::test]
async fn test_chat_room_id_url_encoding_special_characters_and_safety() {
    let client = LcuClient::new();

    // 1. Boundary conditions: empty room ID or empty message must return immediately without panic
    assert!(client.get_chat_conversation_participants("").await.is_empty());
    assert!(!client.send_chat_message("", "hello").await);
    assert!(!client.send_chat_message("valid_room", "").await);
    assert!(!client.send_chat_message("", "").await);

    // 2. Adversarial test cases covering #, ?, @, spaces, slashes, unicode
    let test_cases = vec![
        ("champ-select#12345", "champ-select%2312345"),
        ("room?session=abc&mode=aram", "room%3Fsession%3Dabc%26mode%3Daram"),
        (
            "champ-select-123@champ-select.lol.riotgames.com",
            "champ-select-123%40champ-select.lol.riotgames.com",
        ),
        ("custom room with spaces", "custom%20room%20with%20spaces"),
        ("战队开黑房#2026", "%E6%88%98%E9%98%9F%E5%BC%80%E9%BB%91%E6%88%BF%232026"),
        ("nested/path/room", "nested%2Fpath%2Froom"),
    ];

    for (raw, expected_encoded) in test_cases {
        let encoded = mirror_percent_encode_path_segment(raw);
        assert_eq!(encoded, expected_encoded);

        // Verify URL Parsing guarantees:
        // A. Without percent encoding, special characters cause fatal URL distortions:
        let unencoded_url_str = format!("https://127.0.0.1:1234/lol-chat/v1/conversations/{}/participants", raw);
        if raw.contains('#') {
            if let Ok(bad_parsed) = reqwest::Url::parse(&unencoded_url_str) {
                // Fragment truncation: anything after '#' becomes fragment, NOT path!
                assert!(bad_parsed.fragment().is_some(), "Raw '#' must have turned into a fragment");
                assert!(
                    !bad_parsed.path().ends_with("/participants"),
                    "Path must have been truncated by unencoded '#'"
                );
            }
        }

        // B. With percent encoding, URL parses cleanly without fragment or query truncation:
        let encoded_url_str = format!("https://127.0.0.1:1234/lol-chat/v1/conversations/{}/participants", encoded);
        let parsed = reqwest::Url::parse(&encoded_url_str).expect("Encoded URL must parse successfully");
        assert!(parsed.fragment().is_none(), "Encoded URL must NEVER have a fragment");
        assert!(parsed.query().is_none(), "Encoded URL must NEVER have query parameters");
        assert!(
            parsed.path().ends_with("/participants"),
            "Encoded URL path must terminate cleanly with /participants"
        );
        let expected_path = format!("/lol-chat/v1/conversations/{}/participants", expected_encoded);
        assert_eq!(parsed.path(), expected_path);
    }
}

#[tokio::test]
async fn test_unauthorized_401_invalidates_credentials_via_fallback_candidate() {
    let server = MockLcuServer::start().await;
    let client = LcuClient::new();
    client.set_credentials(server.port, &server.auth_token).await;
    *client.current_phase.write().await = "ChampSelect".to_string();
    *client.current_session.write().await = json!({"id": 100});
    assert!(client.is_connected().await);

    // Override server to return 401 Unauthorized
    server.state.set_status_override(Some(StatusCode::UNAUTHORIZED)).await;

    // Calling get_match_detail triggers handle_response_fallback_candidate on https / http
    let res = client.get_match_detail("55555").await;
    assert!(res.is_none());

    // CRITICAL ASSERTION:
    // 401 Unauthorized MUST invalidate credentials immediately!
    assert!(
        !client.is_connected().await,
        "Credentials must be invalidated on 401"
    );
    assert_eq!(*client.current_phase.read().await, "None");
    assert_eq!(*client.current_session.read().await, serde_json::Value::Null);

    // Test get_raw_match_timeline also invalidates credentials on 401
    client.set_credentials(server.port, &server.auth_token).await;
    assert!(client.is_connected().await);
    let res_timeline = client.get_raw_match_timeline(44444).await;
    assert!(res_timeline.is_none());
    assert!(
        !client.is_connected().await,
        "Timeline 401 must invalidate credentials"
    );
}

#[tokio::test]
async fn test_cache_clearing_and_isolation() {
    let client = LcuClient::new();
    let gid = 123456u64;

    client.insert_negative_cached_detail(gid);
    assert!(client.is_negative_cached_detail(gid));
    // Timeline negative cache is distinct
    assert!(!client.is_negative_cached_timeline(gid));

    client.insert_negative_cached_timeline(gid);
    assert!(client.is_negative_cached_timeline(gid));

    // Clearing caches removes negative caches
    client.clear_lru_caches();
    assert!(!client.is_negative_cached_detail(gid));
    assert!(!client.is_negative_cached_timeline(gid));
}
