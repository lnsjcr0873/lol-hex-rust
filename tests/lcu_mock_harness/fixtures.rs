use serde_json::{json, Value};

/// Canned champ-select session representing a 5v5 ranked match with 1 local player and 4 anonymous teammates
pub fn canned_champ_select_session(chat_room_id: &str, local_puuid: &str, local_cell_id: i64) -> Value {
    let mut my_team = Vec::new();
    for i in 0..5 {
        let cell_id = i as i64;
        let is_local = cell_id == local_cell_id;
        let puuid = if is_local { local_puuid.to_string() } else { format!("anon-puuid-{}", i) };
        let name = if is_local { "LocalPlayer".to_string() } else { format!("召唤师{}", i + 1) };
        my_team.push(json!({
            "cellId": cell_id,
            "championId": if is_local { 157 } else { 0 }, // Yasuo for local
            "championPickIntent": 0,
            "assignedPosition": match i {
                0 => "top",
                1 => "jungle",
                2 => "middle",
                3 => "bottom",
                _ => "utility",
            },
            "spell1Id": 4,
            "spell2Id": 14,
            "puuid": puuid,
            "summonerId": if is_local { 10001 } else { 0 },
            "displayName": name,
            "team": 1,
            "isLocalPlayer": is_local
        }));
    }

    json!({
        "gameId": 9876543210i64,
        "chatDetails": {
            "chatRoomId": chat_room_id,
            "chatRoomName": chat_room_id,
            "multiUserChatId": chat_room_id
        },
        "hasSimulatedInput": false,
        "isCustomGame": false,
        "myTeam": my_team,
        "theirTeam": [],
        "benchChampions": [],
        "benchEnabled": false,
        "rerollsRemaining": 2
    })
}

/// Canned local summoner info
pub fn canned_current_summoner(puuid: &str, display_name: &str) -> Value {
    json!({
        "accountId": 2000001,
        "displayName": display_name,
        "gameName": display_name,
        "internalName": display_name,
        "percentCompleteForNextLevel": 45,
        "profileIconId": 548,
        "puuid": puuid,
        "summonerId": 10001,
        "summonerLevel": 320,
        "tagLine": "CN",
        "unnamed": false,
        "xpSinceLastLevel": 1200,
        "xpUntilNextLevel": 2500
    })
}

/// Canned chat conversation participants (real unmasked summoners)
pub fn canned_chat_participants(chat_room_id: &str, players: &[(&str, &str)]) -> Vec<Value> {
    players
        .iter()
        .map(|(puuid, name)| {
            json!({
                "activePlatform": "league",
                "cid": chat_room_id,
                "game_name": name,
                "game_tag": "CN",
                "id": format!("{}-{}", puuid, chat_room_id),
                "name": name,
                "puuid": puuid,
                "role": "member"
            })
        })
        .collect()
}

/// Canned ranked stats JSON
pub fn canned_ranked_stats(tier: &str, lp: i32, wins: i32, losses: i32) -> Value {
    json!({
        "queues": [
            {
                "queueType": "RANKED_SOLO_5x5",
                "tier": tier,
                "division": "I",
                "leaguePoints": lp,
                "wins": wins,
                "losses": losses,
                "isHotStreak": wins > losses
            },
            {
                "queueType": "RANKED_FLEX_SR",
                "tier": "PLATINUM",
                "division": "II",
                "leaguePoints": 50,
                "wins": 25,
                "losses": 15,
                "isHotStreak": false
            }
        ]
    })
}

/// Canned 20 match history for a given PUUID
pub fn canned_match_history(
    puuid: &str,
    count: usize,
    win_ratio: f64,
    champ_id: i32,
    teammates: &[&str],
) -> Value {
    let mut games = Vec::new();
    let wins_target = (count as f64 * win_ratio).round() as usize;

    for i in 0..count {
        let game_id = 1_000_000_000i64 + (i as i64);
        let win = i < wins_target;
        let kills = if win { 8 } else { 3 };
        let deaths = if win { 2 } else { 7 };
        let assists = if win { 10 } else { 4 };

        let mut participants = Vec::new();
        let mut participant_identities = Vec::new();

        // Slot 1: target player
        participant_identities.push(json!({
            "participantId": 1,
            "player": {
                "puuid": puuid,
                "summonerName": "TargetPlayer",
                "gameName": "TargetPlayer"
            }
        }));
        participants.push(json!({
            "participantId": 1,
            "teamId": 100,
            "championId": champ_id,
            "spell1Id": 4,
            "spell2Id": 14,
            "stats": {
                "win": win,
                "kills": kills,
                "deaths": deaths,
                "assists": assists,
                "totalDamageDealtToChampions": if win { 28000 } else { 14000 },
                "goldEarned": if win { 14500 } else { 9200 },
                "totalMinionsKilled": 180,
                "item0": 3031,
                "item1": 3046,
                "item2": 3006,
                "item3": 0,
                "item4": 0,
                "item5": 0,
                "item6": 3340
            }
        }));

        // Teammates in slots 2..=5
        for (idx, tm_puuid) in teammates.iter().take(4).enumerate() {
            let pid = (idx + 2) as i64;
            participant_identities.push(json!({
                "participantId": pid,
                "player": {
                    "puuid": tm_puuid,
                    "summonerName": format!("Teammate{}", idx + 1),
                    "gameName": format!("Teammate{}", idx + 1)
                }
            }));
            participants.push(json!({
                "participantId": pid,
                "teamId": 100,
                "championId": 100 + (idx as i32),
                "spell1Id": 4,
                "spell2Id": 12,
                "stats": {
                    "win": win,
                    "kills": 4,
                    "deaths": 4,
                    "assists": 6,
                    "totalDamageDealtToChampions": 18000,
                    "goldEarned": 11000,
                    "totalMinionsKilled": 140,
                    "item0": 0, "item1": 0, "item2": 0, "item3": 0, "item4": 0, "item5": 0, "item6": 0
                }
            }));
        }

        // Fill up remaining slots to make 10 players
        while participant_identities.len() < 10 {
            let pid = (participant_identities.len() + 1) as i64;
            let team_id = if pid <= 5 { 100 } else { 200 };
            participant_identities.push(json!({
                "participantId": pid,
                "player": {
                    "puuid": format!("enemy-puuid-{}", pid),
                    "summonerName": format!("Opponent{}", pid),
                    "gameName": format!("Opponent{}", pid)
                }
            }));
            participants.push(json!({
                "participantId": pid,
                "teamId": team_id,
                "championId": 200 + (pid as i32),
                "spell1Id": 4,
                "spell2Id": 14,
                "stats": {
                    "win": if team_id == 100 { win } else { !win },
                    "kills": 2,
                    "deaths": 5,
                    "assists": 3,
                    "totalDamageDealtToChampions": 12000,
                    "goldEarned": 8000,
                    "totalMinionsKilled": 100,
                    "item0": 0, "item1": 0, "item2": 0, "item3": 0, "item4": 0, "item5": 0, "item6": 0
                }
            }));
        }

        games.push(json!({
            "gameId": game_id,
            "gameCreation": 1712000000000i64 + (i as i64 * 3600000),
            "gameDuration": 1800,
            "queueId": 420,
            "gameMode": "CLASSIC",
            "participantIdentities": participant_identities,
            "participants": participants
        }));
    }

    json!({
        "games": {
            "games": games
        }
    })
}

/// Canned 10-player match detail matching Riot LCU API contract
pub fn canned_match_detail_classic(game_id: i64, blue_win: bool) -> Value {
    let mut participant_identities = Vec::new();
    let mut participants = Vec::new();

    // 5 Blue players (teamId 100), 5 Red players (teamId 200)
    for i in 1..=10 {
        let is_blue = i <= 5;
        let team_id = if is_blue { 100 } else { 200 };
        let win = if is_blue { blue_win } else { !blue_win };
        let puuid = format!("puuid-player-{}", i);
        let name = format!("召唤师{}", i);

        // Make player 1 highest damage on blue, player 6 highest damage on red
        let damage = if i == 1 { 35000 } else if i == 6 { 32000 } else { 15000 + (i * 1000) };
        let kills = if i == 1 { 12 } else if i == 6 { 10 } else { 3 };
        let deaths = if i == 1 { 2 } else if i == 6 { 3 } else { 5 };
        let assists = if i == 1 { 8 } else if i == 6 { 6 } else { 4 };

        participant_identities.push(json!({
            "participantId": i,
            "player": {
                "puuid": puuid,
                "summonerName": name,
                "gameName": name
            }
        }));

        participants.push(json!({
            "participantId": i,
            "teamId": team_id,
            "championId": 100 + i,
            "spell1Id": 4,
            "spell2Id": 14,
            "stats": {
                "win": win,
                "kills": kills,
                "deaths": deaths,
                "assists": assists,
                "totalDamageDealtToChampions": damage,
                "goldEarned": 12000 + (i * 500),
                "totalMinionsKilled": 150 + (i * 10),
                "item0": 3031,
                "item1": 3046,
                "item2": 3006,
                "item3": 0,
                "item4": 0,
                "item5": 0,
                "item6": 3340
            }
        }));
    }

    json!({
        "gameId": game_id,
        "gameCreationDate": "2026-04-12T10:00:00Z",
        "gameDuration": 1800,
        "gameMode": "CLASSIC",
        "queueId": 420,
        "participantIdentities": participant_identities,
        "participants": participants,
        "teams": [
            {
                "teamId": 100,
                "win": if blue_win { "Win" } else { "Fail" },
                "towerKills": if blue_win { 9 } else { 2 },
                "dragonKills": if blue_win { 3 } else { 1 },
                "baronKills": if blue_win { 1 } else { 0 }
            },
            {
                "teamId": 200,
                "win": if !blue_win { "Win" } else { "Fail" },
                "towerKills": if !blue_win { 9 } else { 2 },
                "dragonKills": if !blue_win { 3 } else { 1 },
                "baronKills": if !blue_win { 1 } else { 0 }
            }
        ]
    })
}

/// Canned ARAM match detail (queue 450)
pub fn canned_match_detail_aram(game_id: i64) -> Value {
    let mut val = canned_match_detail_classic(game_id, true);
    val["queueId"] = json!(450);
    val["gameMode"] = json!("ARAM");
    val
}

/// Canned Arena match detail (queue 1700, 4 pairs / 8 players)
pub fn canned_match_detail_arena(game_id: i64) -> Value {
    let mut participant_identities = Vec::new();
    let mut participants = Vec::new();

    let team_ids = [100, 100, 200, 200, 300, 300, 400, 400];
    for (idx, &tid) in team_ids.iter().enumerate() {
        let i = (idx + 1) as i64;
        let puuid = format!("arena-puuid-{}", i);
        let name = format!("斗魂勇士{}", i);

        participant_identities.push(json!({
            "participantId": i,
            "player": {
                "puuid": puuid,
                "summonerName": name,
                "gameName": name
            }
        }));

        participants.push(json!({
            "participantId": i,
            "teamId": tid,
            "championId": 200 + (i as i32),
            "spell1Id": 4,
            "spell2Id": 14,
            "stats": {
                "win": tid == 100,
                "kills": 8,
                "deaths": 4,
                "assists": 6,
                "totalDamageDealtToChampions": 25000,
                "goldEarned": 15000,
                "totalMinionsKilled": 0,
                "item0": 3031, "item1": 0, "item2": 0, "item3": 0, "item4": 0, "item5": 0, "item6": 0
            }
        }));
    }

    json!({
        "gameId": game_id,
        "gameDuration": 900,
        "gameMode": "CHERRY",
        "queueId": 1700,
        "participantIdentities": participant_identities,
        "participants": participants
    })
}
