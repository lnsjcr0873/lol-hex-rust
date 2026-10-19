use lol_hex_rust::web::routes::{
    build_chat_broadcast_lines, build_mock_match_detail, build_mock_match_timeline,
    convert_ongoing_card, parse_riot_match_detail,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State};

use crate::TauriAppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatBroadcastResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericResult {
    pub success: bool,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn get_ongoing_match(
    state: State<'_, TauriAppState>,
    force_mock: Option<bool>,
) -> Result<Value, String> {
    // 1. If explicit demo/mock mode requested
    if force_mock == Some(true) {
        let mock = state.history.get_mock_ongoing_match(&state.db);
        let blue: Vec<Value> = mock.blue_team.iter().map(|c| convert_ongoing_card(c, "blue", &state.db)).collect();
        let red: Vec<Value> = mock.red_team.iter().map(|c| convert_ongoing_card(c, "red", &state.db)).collect();
        return Ok(json!({
            "gameflow_phase": "ChampSelect",
            "queue_id": 420,
            "queue_name": "排位赛 单双排 (5v5 演练模拟)",
            "is_my_team_blue": true,
            "my_puuid": "mock_p_0",
            "chat_room_id": "mock_room_1",
            "blue_team": blue,
            "red_team": red,
            "lcu_connected": false,
            "is_active": true,
            "is_mock": true,
            "message": "当前处于【模拟演练模式】，用于预览 10 人透视排版"
        }));
    }

    // 2. Real LCU connection inspection
    let phase = state.lcu.get_gameflow_phase().await;

    // LCU completely disconnected or not logged in
    if phase.is_empty() || phase == "None" {
        return Ok(json!({
            "gameflow_phase": "None",
            "queue_id": 0,
            "queue_name": "等待英雄联盟启动",
            "is_my_team_blue": true,
            "my_puuid": "",
            "blue_team": [],
            "red_team": [],
            "lcu_connected": false,
            "is_active": false,
            "is_mock": false,
            "message": "英雄联盟客户端未连接（请先启动游戏并登录）"
        }));
    }

    // LCU connected but in lobby / matchmaking / idle
    if phase != "ChampSelect" && phase != "GameStart" && phase != "InProgress" {
        let phase_cn = match phase.as_str() {
            "Lobby" => "房间组队中",
            "Matchmaking" => "匹配寻找对局中",
            "ReadyCheck" => "对局就绪等待确认",
            "WaitingForStats" | "PreEndOfGame" | "EndOfGame" => "对局结算中",
            _ => "大厅待命",
        };
        return Ok(json!({
            "gameflow_phase": phase,
            "queue_id": 0,
            "queue_name": format!("客户端在线 · {}", phase_cn),
            "is_my_team_blue": true,
            "my_puuid": "",
            "blue_team": [],
            "red_team": [],
            "lcu_connected": true,
            "is_active": false,
            "is_mock": false,
            "message": format!("客户端已连接，当前状态【{}】，进入英雄选择后将自动穿透侦查", phase_cn)
        }));
    }

    // 3. Active Real Match: ChampSelect / GameStart / InProgress
    let ongoing = state.history.get_ongoing_match(&state.lcu, &state.db).await;
    let blue: Vec<Value> = ongoing.blue_team.iter().map(|c| convert_ongoing_card(c, "blue", &state.db)).collect();
    let red: Vec<Value> = ongoing.red_team.iter().map(|c| convert_ongoing_card(c, "red", &state.db)).collect();

    // Record game ID if present in session for persistent history tracking
    {
        let sess = state.lcu.current_session.read().await;
        if let Some(gid) = sess.get("gameData").and_then(|g| g.get("gameId")).and_then(|v| v.as_i64()) {
            state.history.record_game_id(gid);
        } else if let Some(gid) = sess.get("gameId").and_then(|v| v.as_i64()) {
            state.history.record_game_id(gid);
        }
    }

    let cur_sum = state.lcu.get_current_summoner().await;
    let my_puuid = cur_sum.and_then(|s| s.get("puuid").and_then(|v| v.as_str()).map(|v| v.to_string())).unwrap_or_default();

    let q_name = if !ongoing.queue_name.is_empty() {
        ongoing.queue_name.clone()
    } else {
        match ongoing.phase.as_str() {
            "ChampSelect" => "排位选人阶段".to_string(),
            "GameStart" => "对局载入阶段".to_string(),
            "InProgress" => "游戏进行中".to_string(),
            _ => "实时对局侦查".to_string(),
        }
    };

    Ok(json!({
        "gameflow_phase": ongoing.phase,
        "queue_id": ongoing.queue_id,
        "queue_name": q_name,
        "is_my_team_blue": ongoing.is_my_team_blue,
        "my_puuid": my_puuid,
        "chat_room_id": ongoing.chat_room_id,
        "blue_team": blue,
        "red_team": red,
        "lcu_connected": true,
        "is_active": ongoing.is_active,
        "is_mock": ongoing.is_mock,
        "message": ongoing.message
    }))
}

#[tauri::command]
pub async fn send_chat_records(
    state: State<'_, TauriAppState>,
    room_id: Option<String>,
    format: Option<String>,
) -> Result<ChatBroadcastResponse, String> {
    tracing::info!("send_chat_records requested, room: {:?}, format: {:?}", room_id, format);

    let mut target_room = room_id.unwrap_or_default();
    if target_room.is_empty() {
        let session = state.lcu.get_champ_select_session().await;
        if let Some(cd) = session.get("chatDetails") {
            target_room = cd
                .get("multiUserChatId")
                .or_else(|| cd.get("chatRoomId"))
                .or_else(|| cd.get("chatRoomName"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }
    }

    if target_room.is_empty() {
        let convs = state.lcu.get_conversations().await;
        for c in &convs {
            let ctype = c.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if ctype == "championSelect" {
                if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
                    target_room = id.to_string();
                    break;
                }
            }
        }
    }

    let ongoing = state.history.get_ongoing_match(&state.lcu, &state.db).await;
    let lines = build_chat_broadcast_lines(&ongoing.my_team, format.as_deref());

    if !target_room.is_empty() {
        let mut any_ok = false;
        for line in lines {
            if state.lcu.send_chat_message(&target_room, &line).await {
                any_ok = true;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        }

        Ok(ChatBroadcastResponse {
            success: any_ok,
            message: if any_ok {
                "战绩与车队画像已一键广播至英雄选择聊天室！".to_string()
            } else {
                "发送失败：请确认已处于选人聊天房间".to_string()
            },
        })
    } else {
        Ok(ChatBroadcastResponse {
            success: false,
            message: "未检测到活跃选人聊天室，请在英雄选择阶段重试".to_string(),
        })
    }
}

#[tauri::command]
pub async fn get_summoner_profile(
    state: State<'_, TauriAppState>,
    puuid: Option<String>,
) -> Result<Value, String> {
    let current_sum = state.lcu.get_current_summoner().await;

    let mut target_puuid = if let Some(ref p) = puuid {
        p.clone()
    } else if let Some(ref cur) = current_sum {
        cur.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string()
    } else {
        String::new()
    };

    if target_puuid.is_empty() {
        return Ok(json!({
            "is_connected": false,
            "message": "英雄联盟客户端未连接（请先启动游戏）"
        }));
    }

    let target_sum = if let Some(ref p) = puuid {
        if let Some(ref cur) = current_sum {
            if cur.get("puuid").and_then(|v| v.as_str()) == Some(p.as_str()) {
                current_sum.clone()
            } else if let Some(by_puuid) = state.lcu.get_summoner_by_puuid(p).await {
                Some(by_puuid)
            } else {
                state.lcu.get_summoner_by_name(p).await
            }
        } else if let Some(by_puuid) = state.lcu.get_summoner_by_puuid(p).await {
            Some(by_puuid)
        } else {
            state.lcu.get_summoner_by_name(p).await
        }
    } else {
        current_sum.clone()
    };

    if let Some(ref s) = target_sum {
        if let Some(real_puuid) = s.get("puuid").and_then(|v| v.as_str()) {
            if !real_puuid.is_empty() {
                target_puuid = real_puuid.to_string();
            }
        }
    }

    let mut sum_name = target_sum
        .as_ref()
        .and_then(|s| s.get("gameName").or_else(|| s.get("displayName")).or_else(|| s.get("summonerName")))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut tag_line = target_sum
        .as_ref()
        .and_then(|s| s.get("tagLine"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut level = target_sum
        .as_ref()
        .and_then(|s| s.get("summonerLevel"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let mut icon_id = target_sum
        .as_ref()
        .and_then(|s| s.get("profileIconId"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    if sum_name.is_empty() {
        if let Some(cached) = state.history.get_player_data_fresh(&target_puuid, std::time::Duration::from_secs(600)).await {
            sum_name = cached.summoner_name.clone();
            if let Some(tag) = cached.raw_summoner.get("tagLine").and_then(|v| v.as_str()) {
                tag_line = tag.to_string();
            }
            if let Some(lvl) = cached.raw_summoner.get("summonerLevel").and_then(|v| v.as_i64()) {
                level = lvl as i32;
            }
            if let Some(icon) = cached.raw_summoner.get("profileIconId").and_then(|v| v.as_i64()) {
                icon_id = icon as i32;
            }
        }
    }
    if sum_name.is_empty() {
        sum_name = "召唤师".to_string();
    }

    // Real ranked stats
    let ranked = state.lcu.get_player_ranked_stats(&target_puuid).await;
    let mut tier_solo = "UNRANKED".to_string();
    let mut div_solo = "".to_string();
    let mut lp_solo = 0;
    let mut wins_solo = 0;
    let mut losses_solo = 0;
    let mut tier_flex = "UNRANKED".to_string();
    let mut div_flex = "".to_string();
    let mut lp_flex = 0;
    let mut wins_flex = 0;
    let mut losses_flex = 0;

    if let Some(ref r) = ranked {
        if let Some(qm) = r.get("queueMap").and_then(|q| q.as_object()) {
            if let Some(solo) = qm.get("RANKED_SOLO_5x5") {
                tier_solo = solo.get("tier").and_then(|v| v.as_str()).unwrap_or("UNRANKED").to_string();
                div_solo = solo.get("division").and_then(|v| v.as_str()).unwrap_or("").to_string();
                lp_solo = solo.get("leaguePoints").and_then(|v| v.as_i64()).unwrap_or(0);
                wins_solo = solo.get("wins").and_then(|v| v.as_i64()).unwrap_or(0);
                losses_solo = solo.get("losses").and_then(|v| v.as_i64()).unwrap_or(0);
            }
            if let Some(flex) = qm.get("RANKED_FLEX_SR") {
                tier_flex = flex.get("tier").and_then(|v| v.as_str()).unwrap_or("UNRANKED").to_string();
                div_flex = flex.get("division").and_then(|v| v.as_str()).unwrap_or("").to_string();
                lp_flex = flex.get("leaguePoints").and_then(|v| v.as_i64()).unwrap_or(0);
                wins_flex = flex.get("wins").and_then(|v| v.as_i64()).unwrap_or(0);
                losses_flex = flex.get("losses").and_then(|v| v.as_i64()).unwrap_or(0);
            }
        }
    }

    let wr_solo = if wins_solo + losses_solo > 0 {
        ((wins_solo as f64 / (wins_solo + losses_solo) as f64) * 100.0).round() as i32
    } else {
        0
    };
    let wr_flex = if wins_flex + losses_flex > 0 {
        ((wins_flex as f64 / (wins_flex + losses_flex) as f64) * 100.0).round() as i32
    } else {
        0
    };

    // Real recent matches & tactical summary
    let sample_limit = state.history.sample_count.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let end_idx = sample_limit.saturating_sub(1);
    let cached = state
        .history
        .fetch_and_cache_player(&target_puuid, &sum_name, &state.lcu, 0, end_idx)
        .await;
    let tactical = cached
        .as_ref()
        .map(|c| state.history.compute_tactical_profile(c, 0, sample_limit))
        .unwrap_or_default();

    let top_champions: Vec<Value> = tactical
        .top_champs
        .iter()
        .map(|tc| {
            json!({
                "champion_id": tc.champion_id,
                "champion_name": tc.champion_name,
                "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", tc.champion_id),
                "games": tc.games,
                "win_rate": tc.win_rate_pct,
                "kda": format!("{:.1}", tc.kda)
            })
        })
        .collect();

    let radar = &tactical.radar;

    Ok(json!({
        "is_connected": true,
        "puuid": target_puuid,
        "game_name": sum_name,
        "tag_line": tag_line,
        "profile_icon_id": icon_id,
        "summoner_level": level,
        "tier_solo": tier_solo,
        "division_solo": div_solo,
        "lp_solo": lp_solo,
        "wins_solo": wins_solo,
        "losses_solo": losses_solo,
        "win_rate_solo": wr_solo,
        "tier_flex": tier_flex,
        "division_flex": div_flex,
        "lp_flex": lp_flex,
        "wins_flex": wins_flex,
        "losses_flex": losses_flex,
        "win_rate_flex": wr_flex,
        "recent_radar": {
            "output": radar.get("damage").copied().unwrap_or(0),
            "kda": radar.get("kda").copied().unwrap_or(0),
            "join_rate": radar.get("kp").copied().unwrap_or(0),
            "survival": radar.get("survival").copied().unwrap_or(0),
            "gold": radar.get("economy").copied().unwrap_or(0),
            "vision": radar.get("vision").copied().unwrap_or(0)
        },
        "recent_summary": {
            "total_games": tactical.recent_count,
            "win_rate": tactical.recent_win_rate_pct,
            "avg_kda": tactical.avg_kda_str,
            "avg_kda_ratio": tactical.avg_kda_ratio,
            "top_champions": top_champions
        }
    }))
}

#[tauri::command]
pub async fn get_match_history(
    state: State<'_, TauriAppState>,
    puuid: Option<String>,
    queue_id: Option<i64>,
    beg_index: Option<usize>,
    end_index: Option<usize>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<Value, String> {
    let current_sum = state.lcu.get_current_summoner().await;

    let target_puuid = if let Some(ref p) = puuid {
        p.clone()
    } else if let Some(ref cur) = current_sum {
        cur.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string()
    } else {
        String::new()
    };

    if target_puuid.is_empty() {
        return Ok(json!([]));
    }

    let sum_name = if let Some(ref cur) = current_sum {
        if cur.get("puuid").and_then(|v| v.as_str()) == Some(target_puuid.as_str()) {
            cur.get("gameName").or_else(|| cur.get("displayName")).and_then(|v| v.as_str()).unwrap_or("").to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let (resolved_beg, resolved_end) = match (beg_index, end_index) {
        (Some(b), Some(e)) => (b, e.max(b)),
        (Some(b), None) => {
            let count = page_size.unwrap_or(20).clamp(1, 200);
            (b, b.saturating_add(count.saturating_sub(1)))
        }
        (None, Some(e)) => (0, e),
        (None, None) => {
            let p = page.unwrap_or(1).max(1);
            let ps = page_size.unwrap_or(20).clamp(1, 200);
            let b = (p - 1).saturating_mul(ps);
            let e = b.saturating_add(ps.saturating_sub(1));
            (b, e)
        }
    };
    let resolved_end = resolved_end.min(resolved_beg.saturating_add(199));

    let full_record = match state
        .history
        .fetch_and_cache_player(&target_puuid, &sum_name, &state.lcu, resolved_beg, resolved_end)
        .await
    {
        Some(r) => Some(r),
        None => {
            state.history.get_player_data_fresh(&target_puuid, std::time::Duration::from_secs(600)).await
        }
    };
    let all_matches = full_record.map(|r| r.matches).unwrap_or_default();
    let queue_matches: Vec<_> = all_matches
        .into_iter()
        .filter(|m| queue_id.map_or(true, |qid| qid == 0 || m.queue_id as i64 == qid))
        .collect();

    let paginated_matches = lol_hex_rust::web::routes::paginate_matches(&queue_matches, resolved_beg, resolved_end);

    let filtered: Vec<Value> = paginated_matches
        .into_iter()
        .map(|m| {
            let role_cn = match m.queue_id {
                450 => "乱斗",
                _ => "对局",
            };
            json!({
                "game_id": m.game_id,
                "game_creation": m.game_creation,
                "game_duration_seconds": m.game_duration,
                "game_mode": m.mode_cn,
                "queue_id": m.queue_id,
                "queue_name": m.mode_cn,
                "is_win": m.win,
                "team_id": m.team_id,
                "is_blue": m.is_blue,
                "champion_id": m.champion_id,
                "champion_name": m.champion_name,
                "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", m.champion_id),
                "role": role_cn,
                "spell1_id": m.spell1_id,
                "spell2_id": m.spell2_id,
                "kills": m.kills,
                "deaths": m.deaths,
                "assists": m.assists,
                "kda_ratio": m.kda_ratio,
                "is_mvp": m.is_mvp,
                "is_svp": m.is_svp,
                "items": m.items,
                "total_damage": m.damage,
                "damage_share_pct": m.damage_share_pct,
                "physical_damage": m.physical_damage,
                "magic_damage": m.magic_damage,
                "true_damage": m.true_damage,
                "primary_rune": m.primary_rune,
                "secondary_rune": m.secondary_rune,
                "damage_taken": m.damage_taken,
                "gold_earned": m.gold,
                "minions_killed": m.cs,
                "participants_preview": m.participants_preview
            })
        })
        .collect();

    Ok(json!(filtered))
}

#[tauri::command]
pub async fn get_champion_mastery(
    state: State<'_, TauriAppState>,
    puuid: String,
) -> Result<Value, String> {
    let target = if puuid == "me" || puuid == "current" {
        String::new()
    } else {
        puuid
    };
    if let Some(masteries) = state.lcu.get_champion_mastery(&target).await {
        return Ok(masteries);
    }
    Ok(json!([]))
}

#[tauri::command]
pub async fn get_player_challenges(
    state: State<'_, TauriAppState>,
    puuid: Option<String>,
) -> Result<Value, String> {
    let p = puuid.unwrap_or_default();
    let target = if p == "me" || p == "current" {
        String::new()
    } else {
        p
    };
    if target.starts_with("mock_") {
        return Ok(json!({
            "playerChallenges": [
                { "id": 505001, "currentValue": 168 },
                { "id": 510001, "currentValue": 145 },
                { "id": 510011, "currentValue": 38 },
                { "id": 504003, "currentValue": 12 },
                { "id": 504002, "currentValue": 88 },
                { "id": 504004, "currentValue": 45 }
            ]
        }));
    }
    if let Some(challenges) = state.lcu.get_player_challenges(&target).await {
        return Ok(challenges);
    }
    Ok(json!({ "playerChallenges": [] }))
}

#[tauri::command]
pub async fn get_match_detail(
    state: State<'_, TauriAppState>,
    game_id: i64,
) -> Result<Value, String> {
    // 1. Try real LCU endpoint
    if let Some(raw) = state.lcu.get_match_detail(&game_id.to_string()).await {
        if let Some(parsed) = parse_riot_match_detail(&raw, &state.db) {
            return Ok(parsed);
        }
    }

    // 2. Demo / mock fallback for mock IDs
    if game_id <= 0 || game_id == 99991234 || game_id == 100200300 {
        let mock = build_mock_match_detail(game_id, &state.db);
        return Ok(mock);
    }

    Err(format!("无法获取对局 {} 的详情数据", game_id))
}

#[tauri::command]
pub async fn get_match_timeline(
    state: State<'_, TauriAppState>,
    game_id: u64,
) -> Result<Value, String> {
    // 1. Try real LCU endpoint
    if let Some(timeline) = state.lcu.get_game_timeline(game_id).await {
        return Ok(timeline);
    }

    // 2. Demo / mock fallback only for explicit mock IDs (e.g. 0 or 99991234)
    if game_id == 0 || game_id == 99991234 || game_id == 100200300 {
        return Ok(build_mock_match_timeline(game_id));
    }

    Err(format!("无法获取对局 {} 的时间线数据", game_id))
}

#[tauri::command]
pub async fn get_raw_match_history(
    state: State<'_, TauriAppState>,
    puuid: Option<String>,
    queue_id: Option<i32>,
    beg_index: Option<usize>,
    end_index: Option<usize>,
) -> Result<Value, String> {
    let current_sum = state.lcu.get_current_summoner().await;

    let target_puuid = if let Some(ref p) = puuid {
        if p == "me" || p == "current" {
            current_sum
                .as_ref()
                .and_then(|s| s.get("puuid"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            p.clone()
        }
    } else if let Some(ref cur) = current_sum {
        cur.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string()
    } else {
        String::new()
    };

    if target_puuid.is_empty() {
        return Ok(json!({
            "accountId": 0,
            "games": {
                "games": []
            }
        }));
    }

    let b_idx = beg_index.unwrap_or(0);
    let e_idx = end_index.unwrap_or_else(|| b_idx.saturating_add(19)).max(b_idx);
    let e_idx = e_idx.min(b_idx.saturating_add(199));

    let mut raw_history = match state.lcu.get_raw_match_history(&target_puuid, b_idx, e_idx).await {
        Some(hist) => hist,
        None => {
            return Err("无法从客户端获取原始战绩数据".to_string());
        }
    };

    // If queue_id filter is requested (and > 0), filter the games array in place
    if let Some(qid) = queue_id {
        if qid > 0 {
            let filter_fn = |arr: &mut Vec<Value>| {
                arr.retain(|game| {
                    game.get("queueId").and_then(|q| q.as_i64()).map(|q| q == qid as i64).unwrap_or(false)
                });
            };

            if let Some(g_val) = raw_history.get_mut("games") {
                if let Some(arr) = g_val.as_array_mut() {
                    filter_fn(arr);
                } else if let Some(arr) = g_val.get_mut("games").and_then(|g| g.as_array_mut()) {
                    filter_fn(arr);
                }
            } else if let Some(arr) = raw_history.as_array_mut() {
                filter_fn(arr);
            }
        }
    }

    Ok(raw_history)
}

#[tauri::command]
pub async fn get_raw_match_timeline(
    state: State<'_, TauriAppState>,
    game_id: u64,
) -> Result<Value, String> {
    // 1. Try real LCU endpoint (with LRU cache and concurrency protection)
    if let Some(timeline) = state.lcu.get_raw_match_timeline(game_id).await {
        return Ok(timeline);
    }

    // 2. Demo / mock fallback only for explicit mock IDs (e.g. 0, 99991234, 100200300)
    if game_id == 0 || game_id == 99991234 || game_id == 100200300 {
        return Ok(build_mock_match_timeline(game_id));
    }

    Err(format!("无法获取对局 {} 的原始时间线数据", game_id))
}

#[tauri::command]
pub async fn get_raw_match_detail(
    state: State<'_, TauriAppState>,
    game_id: u64,
) -> Result<Value, String> {
    if game_id == 0 {
        return Err("Invalid game_id: 0".to_string());
    }

    // 1. Try real LCU endpoint (with LRU cache in LCU client)
    if let Some(detail) = state.lcu.get_raw_match_detail(game_id).await {
        return Ok(detail);
    }

    // 2. Demo / mock fallback only for explicit mock IDs (e.g. 0, 99991234, 100200300)
    if game_id == 99991234 || game_id == 100200300 {
        return Ok(build_mock_match_detail(game_id as i64, &state.db));
    }

    Err(format!("无法获取对局 {} 的原始详情数据", game_id))
}

#[tauri::command]
pub async fn get_automation_config(
    state: State<'_, TauriAppState>,
) -> Result<Value, String> {
    let cfg = state.config.get().await;
    Ok(json!({
        "auto_accept_match": cfg.auto_accept_match,
        "auto_accept_delay_seconds": cfg.auto_accept_delay_seconds,
        "auto_swap_bench": cfg.auto_swap_bench,
        "auto_apply_runes": cfg.auto_apply_runes,
        "bench_preferred_champions": cfg.bench_preferred_champions
    }))
}

#[tauri::command]
pub async fn update_automation_config(
    state: State<'_, TauriAppState>,
    config: Value,
) -> Result<GenericResult, String> {
    let auto_accept = config.get("auto_accept_match").and_then(|v| v.as_bool());
    let delay = config.get("auto_accept_delay_seconds")
        .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|u| u as f64)))
        .map(|d| (d.round() as u64).clamp(0, lol_hex_rust::config::MAX_AUTO_ACCEPT_DELAY_SECONDS));
    let auto_swap = config.get("auto_swap_bench").and_then(|v| v.as_bool());
    let auto_runes = config.get("auto_apply_runes").and_then(|v| v.as_bool());
    let bench_champs = config.get("bench_preferred_champions").and_then(|v| v.as_array()).map(|arr| {
        let mut seen = std::collections::HashSet::new();
        arr.iter().filter_map(|x| {
            let s = x.as_str().map(|s| s.trim().to_string()).or_else(|| x.as_i64().map(|n| n.to_string()))?;
            if !s.is_empty() && seen.insert(s.clone()) {
                Some(s)
            } else {
                None
            }
        }).collect()
    });

    state
        .config
        .update_automation(auto_accept, delay, auto_swap, auto_runes, bench_champs)
        .await;

    Ok(GenericResult {
        success: true,
        message: Some("自动化策略已成功保存并实时生效".to_string()),
    })
}

pub async fn do_apply_runes_and_spells(
    lcu: &lol_hex_rust::lcu::LcuClient,
    db: &lol_hex_rust::data::loader::Database,
    champion_id: i64,
) -> (bool, String) {
    let session = lcu.get_champ_select_session().await;
    let mut detected_pos = "all".to_string();
    if let Some(my_arr) = session.get("myTeam").and_then(|v| v.as_array()) {
        let local_cell = session.get("localPlayerCellId").and_then(|v| v.as_i64());
        for m in my_arr {
            let cell = m.get("cellId").and_then(|v| v.as_i64());
            if cell == local_cell {
                if let Some(assigned) = m.get("assignedPosition").and_then(|v| v.as_str()) {
                    if !assigned.is_empty() {
                        detected_pos = assigned.to_string();
                        break;
                    }
                }
            }
        }
    }
    lcu.do_apply_runes_and_spells(db, champion_id, &detected_pos).await
}

#[tauri::command]
pub async fn apply_runes_and_spells(
    state: State<'_, TauriAppState>,
    champion_id: i64,
) -> Result<GenericResult, String> {
    tracing::info!("apply_runes_and_spells for champion: {}", champion_id);
    let (success, message) = do_apply_runes_and_spells(&state.lcu, &state.db, champion_id).await;
    Ok(GenericResult {
        success,
        message: Some(message),
    })
}

#[tauri::command]
pub async fn search_summoner(
    state: State<'_, TauriAppState>,
    name: String,
) -> Result<Value, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("搜索内容不能为空".to_string());
    }

    // 1. Try search by summoner name
    if let Some(mut sum) = state.lcu.get_summoner_by_name(trimmed).await {
        if sum.get("tagLine").and_then(|v| v.as_str()).map_or(true, |t| t.is_empty()) {
            if let Some(tag) = trimmed.split('#').nth(1) {
                if !tag.trim().is_empty() {
                    sum["tagLine"] = json!(tag.trim());
                }
            }
        }
        return Ok(sum);
    }

    // 2. Try lookup by puuid
    if let Some(sum) = state.lcu.get_summoner_by_puuid(trimmed).await {
        return Ok(sum);
    }

    // 3. Fallback / mock search support (only for exact keywords or explicit mock prefixes)
    if trimmed == "演练"
        || trimmed == "测试"
        || trimmed.eq_ignore_ascii_case("mock")
        || trimmed.eq_ignore_ascii_case("demo")
        || trimmed.starts_with("mock:")
        || trimmed.starts_with("mock_")
    {
        return Ok(json!({
            "puuid": "mock_demo_summoner_search",
            "gameName": trimmed,
            "tagLine": "DEMO",
            "displayName": trimmed,
            "profileIconId": 548,
            "summonerLevel": 188
        }));
    }

    Err(format!("未查询到召唤师 '{}' 的信息", trimmed))
}

#[tauri::command]
pub async fn toggle_ongoing_overlay(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("ongoing-overlay") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            let _ = window.set_focus();
            Ok(true)
        }
    } else {
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "ongoing-overlay",
            tauri::WebviewUrl::App("index.html?window=overlay".into()),
        )
        .title("HexBrain 实时对局悬浮窗")
        .inner_size(1120.0, 720.0)
        .min_inner_size(800.0, 500.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

        window.show().map_err(|e| e.to_string())?;
        let _ = window.set_focus();
        Ok(true)
    }
}

#[tauri::command]
pub async fn set_ongoing_overlay_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("ongoing-overlay") {
        if visible {
            window.show().map_err(|e| e.to_string())?;
            let _ = window.set_focus();
        } else {
            window.hide().map_err(|e| e.to_string())?;
        }
        Ok(())
    } else if visible {
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "ongoing-overlay",
            tauri::WebviewUrl::App("index.html?window=overlay".into()),
        )
        .title("HexBrain 实时对局悬浮窗")
        .inner_size(1120.0, 720.0)
        .min_inner_size(800.0, 500.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

        window.show().map_err(|e| e.to_string())?;
        let _ = window.set_focus();
        Ok(())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub async fn set_ongoing_overlay_always_on_top(
    app: AppHandle,
    always_on_top: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("ongoing-overlay") {
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn is_ongoing_overlay_visible(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("ongoing-overlay") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

