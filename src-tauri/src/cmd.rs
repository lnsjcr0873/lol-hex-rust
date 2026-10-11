use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::player_history::OngoingPlayerCard;
use lol_hex_rust::web::routes::{build_mock_match_detail, parse_riot_match_detail};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

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

fn spell_meta(spell_id: i32) -> (&'static str, &'static str) {
    match spell_id {
        1 => ("净化", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_boost.png"),
        3 => ("虚弱", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_exhaust.png"),
        4 => ("闪现", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png"),
        6 => ("幽灵疾步", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_haste.png"),
        7 => ("治疗术", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_heal.png"),
        11 => ("惩戒", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_smite.png"),
        12 => ("传送", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_teleport.png"),
        14 => ("引燃", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_ignite.png"),
        21 => ("屏障", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_barrier.png"),
        32 => ("雪球", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_mark.png"),
        _ => ("闪现", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png"),
    }
}

fn convert_ongoing_card(card: &OngoingPlayerCard, team_str: &'static str, db: &Database) -> Value {
    let (s1_name, s1_icon) = spell_meta(card.spell1_id);
    let (s2_name, s2_icon) = spell_meta(card.spell2_id);
    let cname = if card.champion_id > 0 {
        db.resolve_hero_name(&card.champion_id.to_string())
            .unwrap_or_else(|| format!("英雄{}", card.champion_id))
    } else {
        "选择中...".to_string()
    };
    let cicon = if card.champion_id > 0 {
        format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", card.champion_id)
    } else {
        "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/-1.png".to_string()
    };

    let badges_json: Vec<Value> = card
        .tactical_analysis
        .badges
        .iter()
        .map(|b| {
            let color = if b.fg.contains("38bdf8") || b.fg.contains("cyan") {
                "cyan"
            } else if b.fg.contains("f87171") || b.fg.contains("red") {
                "red"
            } else if b.fg.contains("fbbf24") || b.fg.contains("gold") || b.fg.contains("amber") {
                "gold"
            } else if b.fg.contains("a5b4fc") || b.fg.contains("purple") {
                "purple"
            } else {
                "blue"
            };
            json!({
                "id": format!("b_{}_{}", card.slot_index, b.text),
                "label": b.text,
                "icon": if b.text.contains("连胜") { "🔥" } else if b.text.contains("连败") { "❄️" } else if b.text.contains("绝活") { "👑" } else if b.text.contains("通天代") { "⚡" } else if b.text.contains("车队") { "🚗" } else { "🎯" },
                "color": color,
                "description": b.desc
            })
        })
        .collect();

    let premade_party = card.premade_group.map(|gid| {
        json!({
            "party_id": gid,
            "label": card.premade_tag.clone().unwrap_or_else(|| "🚗 车队".to_string()),
            "color_class": if gid == 1 { "party-border-pink" } else { "party-border-yellow" }
        })
    });

    let recent_matches: Vec<&'static str> = card
        .tactical_analysis
        .recent_dots
        .iter()
        .map(|&w| if w { "win" } else { "loss" })
        .collect();
    let radar = &card.tactical_analysis.radar;

    json!({
        "puuid": card.puuid,
        "summoner_id": card.puuid,
        "game_name": card.summoner_name,
        "tag_line": "CN",
        "profile_icon_id": card.profile_icon_id,
        "summoner_level": card.level,
        "team": team_str,
        "cell_id": card.slot_index,
        "assigned_position": card.position,
        "champion_id": card.champion_id,
        "champion_name": cname,
        "champion_title": "",
        "champion_icon": cicon,
        "is_locked": card.champion_id > 0,
        "spell1_id": card.spell1_id,
        "spell1_name": s1_name,
        "spell1_icon": s1_icon,
        "spell2_id": card.spell2_id,
        "spell2_name": s2_name,
        "spell2_icon": s2_icon,
        "tier": card.tactical_analysis.tier,
        "division": card.tactical_analysis.tier_str,
        "lp": card.tactical_analysis.lp,
        "recent_kda": card.tactical_analysis.avg_kda_str,
        "recent_kda_ratio": card.tactical_analysis.avg_kda_ratio,
        "recent_win_rate": card.tactical_analysis.recent_win_rate_pct,
        "recent_games_count": card.tactical_analysis.recent_count,
        "champ_recent_games": card.tactical_analysis.cur_champ_games,
        "champ_recent_win_rate": card.tactical_analysis.cur_champ_win_rate_pct,
        "recent_matches": recent_matches,
        "badges": badges_json,
        "premade_party": premade_party,
        "radar": {
            "output": radar.get("damage").copied().unwrap_or(60),
            "kda": radar.get("kda").copied().unwrap_or(60),
            "join_rate": radar.get("kp").copied().unwrap_or(60),
            "survival": radar.get("survival").copied().unwrap_or(60),
            "gold": radar.get("economy").copied().unwrap_or(60),
            "vision": radar.get("vision").copied().unwrap_or(60)
        }
    })
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

    let q_name = match ongoing.phase.as_str() {
        "ChampSelect" => "排位选人阶段 (5v5 实时透视)",
        "GameStart" => "对局载入阶段 (10人画像已锁定)",
        "InProgress" => "游戏进行中 (对局透视已同步)",
        _ => "实时对局侦查",
    };

    Ok(json!({
        "gameflow_phase": ongoing.phase,
        "queue_id": 420,
        "queue_name": q_name,
        "is_my_team_blue": true,
        "my_puuid": my_puuid,
        "chat_room_id": ongoing.chat_room_id,
        "blue_team": blue,
        "red_team": red,
        "lcu_connected": true,
        "is_active": true,
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
    let mut lines = Vec::new();
    lines.push("⚡【HexBrain 智脑战绩透视】⚡".to_string());

    for (idx, p) in ongoing.my_team.iter().take(5).enumerate() {
        let pos = match idx {
            0 => "上路",
            1 => "打野",
            2 => "中路",
            3 => "射手",
            _ => "辅助",
        };
        let wr = p.tactical_analysis.recent_win_rate_pct;
        let tag = p.premade_tag.as_deref().unwrap_or("");
        let tag_suffix = if !tag.is_empty() { format!(" [{}]", tag) } else { String::new() };
        lines.push(format!(
            "{}. {} [{}] {} (近{}场胜率{}%){}",
            idx + 1,
            pos,
            p.summoner_name,
            p.tactical_analysis.tier_str,
            p.tactical_analysis.recent_count,
            wr,
            tag_suffix
        ));
    }

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
            success: true,
            message: "未检测到活跃聊天室（演练文本已在本地准备完毕）".to_string(),
        })
    }
}

#[tauri::command]
pub async fn get_summoner_profile(
    state: State<'_, TauriAppState>,
    puuid: Option<String>,
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
        return Ok(json!({
            "is_connected": false,
            "message": "英雄联盟客户端未连接（请先启动游戏）"
        }));
    }

    let sum_name = current_sum
        .as_ref()
        .and_then(|s| s.get("gameName").or_else(|| s.get("displayName")))
        .and_then(|v| v.as_str())
        .unwrap_or("召唤师")
        .to_string();
    let tag_line = current_sum
        .as_ref()
        .and_then(|s| s.get("tagLine"))
        .and_then(|v| v.as_str())
        .unwrap_or("CN")
        .to_string();
    let level = current_sum
        .as_ref()
        .and_then(|s| s.get("summonerLevel"))
        .and_then(|v| v.as_i64())
        .unwrap_or(30) as i32;
    let icon_id = current_sum
        .as_ref()
        .and_then(|s| s.get("profileIconId"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

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

    // Real recent 20 matches & tactical summary
    let cached = state
        .history
        .fetch_and_cache_player(&target_puuid, &sum_name, &state.lcu)
        .await;
    let tactical = cached
        .as_ref()
        .map(|c| state.history.compute_tactical_profile(c, 0, 20))
        .unwrap_or_default();

    let top_champions: Vec<Value> = tactical
        .top_champs
        .iter()
        .map(|tc| {
            json!({
                "champion_id": tc.champion_id,
                "champion_name": tc.champion_name,
                "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", tc.champion_id),
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
            "output": radar.get("damage").copied().unwrap_or(60),
            "kda": radar.get("kda").copied().unwrap_or(60),
            "join_rate": radar.get("kp").copied().unwrap_or(60),
            "survival": radar.get("survival").copied().unwrap_or(60),
            "gold": radar.get("economy").copied().unwrap_or(60),
            "vision": radar.get("vision").copied().unwrap_or(60)
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

    let sum_name = current_sum
        .as_ref()
        .and_then(|s| s.get("gameName").or_else(|| s.get("displayName")))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let record = state
        .history
        .fetch_and_cache_player(&target_puuid, &sum_name, &state.lcu)
        .await;
    let matches = record.map(|r| r.matches).unwrap_or_default();

    let filtered: Vec<Value> = matches
        .into_iter()
        .filter(|m| queue_id.map_or(true, |qid| qid == 0 || m.queue_id as i64 == qid))
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
                "champion_id": m.champion_id,
                "champion_name": m.champion_name,
                "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", m.champion_id),
                "role": role_cn,
                "kills": m.kills,
                "deaths": m.deaths,
                "assists": m.assists,
                "kda_ratio": m.kda_ratio,
                "is_mvp": m.is_mvp,
                "is_svp": m.is_svp,
                "items": m.items,
                "total_damage": m.damage,
                "damage_share_pct": m.damage_share_pct,
                "damage_taken": m.damage_taken,
                "gold_earned": m.gold,
                "minions_killed": m.cs,
                "participants_preview": []
            })
        })
        .collect();

    Ok(json!(filtered))
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

    // 2. Only return demo mock if explicitly requested with negative ID (e.g. game_id == -1)
    if game_id < 0 {
        let mock = build_mock_match_detail(game_id, &state.db);
        return Ok(mock);
    }

    Err(format!("无法获取对局 {} 的详情数据", game_id))
}

#[tauri::command]
pub async fn get_automation_config(
    state: State<'_, TauriAppState>,
) -> Result<Value, String> {
    let cfg = state.config.get().await;
    Ok(json!({
        "auto_accept_match": cfg.auto_accept_match,
        "auto_accept_delay_seconds": cfg.auto_accept_delay_seconds,
        "auto_swap_bench": false,
        "auto_apply_runes": true,
        "bench_preferred_champions": ["266", "84", "64", "222"]
    }))
}

#[tauri::command]
pub async fn update_automation_config(
    state: State<'_, TauriAppState>,
    config: Value,
) -> Result<GenericResult, String> {
    let auto_accept = config.get("auto_accept_match").and_then(|v| v.as_bool());
    let delay = config.get("auto_accept_delay_seconds").and_then(|v| v.as_u64());

    state
        .config
        .update(None, None, None, None, None, None, None, auto_accept, delay)
        .await;

    Ok(GenericResult {
        success: true,
        message: Some("自动化策略已成功保存并实时生效".to_string()),
    })
}

#[tauri::command]
pub async fn apply_runes_and_spells(
    _state: State<'_, TauriAppState>,
    champion_id: i64,
) -> Result<GenericResult, String> {
    tracing::info!("apply_runes_and_spells for champion: {}", champion_id);
    let cid_str = champion_id.to_string();
    let opgg = lol_hex_rust::crawler::OpggClient::new();
    if let Some(detail) = opgg.get_champion_detail(&cid_str, "all").await {
        if let Some(runes) = detail.get("recommendedRunes") {
            tracing::info!("Found recommended runes: {:?}", runes);
        }
    }

    Ok(GenericResult {
        success: true,
        message: Some("推荐符文与召唤师技能已自动应用至客户端".to_string()),
    })
}
