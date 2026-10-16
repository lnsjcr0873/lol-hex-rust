use crate::crawler::OpggClient;
use crate::data::loader::Database;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

pub const ORDERED_POSITIONS: [&str; 5] = ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"];
pub const SHORT_POS_NAMES_CN: [&str; 5] = ["上单", "打野", "中单", "下路", "辅助"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DraftSlot {
    pub cell_id: i64,
    pub champion_id: String,
    pub champion_name: String,
    pub position: String,
    pub position_cn: String,
    pub is_local_player: bool,
    #[serde(default)]
    pub is_locked: bool,
    pub win_rate: f64,
    pub tier: i32,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RankedDraftState {
    pub is_draft_active: bool,
    pub my_team: Vec<DraftSlot>,
    pub their_team: Vec<DraftSlot>,
    pub decision_card: Value,
}

fn resolve_position(
    manual_pos: Option<&String>,
    auto_pos: Option<&str>,
    cid: &str,
    opgg: Option<&OpggClient>,
    db: Option<&Database>,
    fallback_idx: usize,
) -> String {
    manual_pos
        .cloned()
        .or_else(|| auto_pos.map(|s| s.to_string()))
        .or_else(|| {
            if !cid.is_empty() {
                opgg.and_then(|op| op.get_primary_position(cid))
                    .or_else(|| {
                        db.and_then(|database| {
                            let resolved_id = database.resolve_hero_id(cid).unwrap_or_else(|| cid.to_string());
                            database.heroes_map.get(&resolved_id).and_then(|s| {
                                s.roles.first().and_then(|r| match r.to_lowercase().as_str() {
                                    "top" => Some("TOP".to_string()),
                                    "jungle" | "jg" => Some("JUNGLE".to_string()),
                                    "mid" | "middle" => Some("MID".to_string()),
                                    "bot" | "bottom" | "adc" => Some("ADC".to_string()),
                                    "support" | "sup" | "utility" => Some("SUPPORT".to_string()),
                                    _ => None,
                                })
                            })
                        })
                    })
            } else {
                None
            }
        })
        .unwrap_or_else(|| ORDERED_POSITIONS[fallback_idx % 5].to_string())
}

pub struct RankedDraftManager {
    pub state: RankedDraftState,
    pub manual_positions: HashMap<String, String>,
}

impl Default for RankedDraftManager {
    fn default() -> Self {
        Self {
            state: RankedDraftState::default(),
            manual_positions: HashMap::new(),
        }
    }
}

impl RankedDraftManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_slot_position(&mut self, team: &str, slot_index: usize, position: &str) {
        let key = format!("{}_{}", team, slot_index);
        self.manual_positions.insert(key, position.to_uppercase());
    }

    pub fn reset_manual_positions(&mut self) {
        self.manual_positions.clear();
    }

    pub fn parse_session(&mut self, session: &Value) {
        self.parse_session_with_db(session, None);
    }

    pub fn parse_session_with_db(&mut self, session: &Value, db: Option<&Database>) {
        self.parse_session_with_db_and_opgg(session, db, None);
    }

    pub fn parse_session_with_db_and_opgg(
        &mut self,
        session: &Value,
        db: Option<&Database>,
        opgg: Option<&crate::crawler::OpggClient>,
    ) {
        let my_team_arr = session.get("myTeam").and_then(|v| v.as_array());
        let their_team_arr = session.get("theirTeam").and_then(|v| v.as_array());
        let local_cell_id = session.get("localPlayerCellId").and_then(|v| v.as_i64()).unwrap_or(-1);

        let my_team_slice = match my_team_arr {
            Some(arr) if !arr.is_empty() => arr,
            _ => {
                self.state.is_draft_active = false;
                self.state.my_team.clear();
                self.state.their_team.clear();
                return;
            }
        };

        self.state.is_draft_active = true;
        self.state.my_team.clear();
        self.state.their_team.clear();

        for (idx, m) in my_team_slice.iter().enumerate() {
            let cell_id = m.get("cellId").and_then(|v| v.as_i64()).unwrap_or(0);
            let raw_cid = m.get("championId").or_else(|| m.get("championPickIntent"))
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => "".to_string(),
                })
                .unwrap_or_default();
            let cid = if raw_cid == "0" || raw_cid == "-1" { String::new() } else { raw_cid };

            let auto_pos = m.get("assignedPosition").and_then(|v| v.as_str()).and_then(|s| {
                match s.to_lowercase().as_str() {
                    "top" => Some("TOP"),
                    "jungle" => Some("JUNGLE"),
                    "middle" | "mid" => Some("MID"),
                    "bottom" | "bot" => Some("ADC"),
                    "utility" | "support" => Some("SUPPORT"),
                    _ => None,
                }
            });

            let key = format!("my_{}", idx);
            let pos = resolve_position(self.manual_positions.get(&key), auto_pos, &cid, opgg, db, idx);

            let pos_idx = ORDERED_POSITIONS.iter().position(|&p| p == pos).unwrap_or(idx % 5);

            let (cname, cavatar) = if let Some(database) = db {
                if !cid.is_empty() {
                    let resolved_id = database.resolve_hero_id(&cid).unwrap_or_else(|| cid.clone());
                    let name = database.resolve_hero_name(&resolved_id)
                        .unwrap_or_else(|| format!("队友{}", idx + 1));
                    let summary = database.heroes_map.get(&resolved_id);
                    let avatar = summary.map(|s| s.avatar_url.clone()).unwrap_or_default();
                    (name, avatar)
                } else {
                    (format!("队友{}", idx + 1), String::new())
                }
            } else {
                (format!("队友{}", idx + 1), String::new())
            };

            // Ranked ladder win rate and tier strictly from OP.GG (neutral 50.0% / T3 if loading)
            let mut cwr = 0.500;
            let mut ctier = 3;
            if let Some(op) = opgg {
                if !cid.is_empty() {
                    if let Some((op_wr, op_tier)) = op.get_cached_stats(&cid, &pos) {
                        cwr = op_wr;
                        ctier = op_tier;
                    }
                }
            }

            let is_locked = if !cid.is_empty() {
                session.get("actions")
                    .and_then(|a| a.as_array())
                    .map(|rounds| {
                        rounds.iter().any(|round| {
                            round.as_array().map(|actions| {
                                actions.iter().any(|act| {
                                    act.get("actorCellId").and_then(|c| c.as_i64()) == Some(cell_id)
                                        && act.get("type").and_then(|t| t.as_str()) == Some("pick")
                                        && act.get("completed").and_then(|c| c.as_bool()) == Some(true)
                                })
                            }).unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            } else {
                false
            };

            self.state.my_team.push(DraftSlot {
                cell_id,
                champion_id: cid,
                champion_name: cname,
                position: pos,
                position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                is_local_player: local_cell_id >= 0 && cell_id == local_cell_id,
                is_locked,
                win_rate: cwr,
                tier: ctier,
                avatar_url: cavatar,
            });
        }

        if let Some(arr) = their_team_arr {
            for (idx, m) in arr.iter().enumerate() {
                let cell_id = m.get("cellId").and_then(|v| v.as_i64()).unwrap_or(0);
                let raw_cid = m.get("championId").or_else(|| m.get("championPickIntent"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => "".to_string(),
                    })
                    .unwrap_or_default();
                let cid = if raw_cid == "0" || raw_cid == "-1" { String::new() } else { raw_cid };

                let auto_pos = m.get("assignedPosition").and_then(|v| v.as_str()).and_then(|s| {
                    match s.to_lowercase().as_str() {
                        "top" => Some("TOP"),
                        "jungle" => Some("JUNGLE"),
                        "middle" | "mid" => Some("MID"),
                        "bottom" | "bot" => Some("ADC"),
                        "utility" | "support" => Some("SUPPORT"),
                        _ => None,
                    }
                });

                let key = format!("their_{}", idx);
                let pos = resolve_position(self.manual_positions.get(&key), auto_pos, &cid, opgg, db, idx);

                let pos_idx = ORDERED_POSITIONS.iter().position(|&p| p == pos).unwrap_or(idx % 5);

                let (cname, cavatar) = if let Some(database) = db {
                    if !cid.is_empty() {
                        let resolved_id = database.resolve_hero_id(&cid).unwrap_or_else(|| cid.clone());
                        let name = database.resolve_hero_name(&resolved_id)
                            .unwrap_or_else(|| format!("对手{}", idx + 1));
                        let summary = database.heroes_map.get(&resolved_id);
                        let avatar = summary.map(|s| s.avatar_url.clone()).unwrap_or_default();
                        (name, avatar)
                    } else {
                        (format!("对手{}", idx + 1), String::new())
                    }
                } else {
                    (format!("对手{}", idx + 1), String::new())
                };

                // Ranked ladder win rate and tier strictly from OP.GG (neutral 50.0% / T3 if loading)
                let mut cwr = 0.500;
                let mut ctier = 3;
                if let Some(op) = opgg {
                    if !cid.is_empty() {
                        if let Some((op_wr, op_tier)) = op.get_cached_stats(&cid, &pos) {
                            cwr = op_wr;
                            ctier = op_tier;
                        }
                    }
                }

                let is_their_locked = !cid.is_empty();

                self.state.their_team.push(DraftSlot {
                    cell_id,
                    champion_id: cid,
                    champion_name: cname,
                    position: pos,
                    position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                    is_local_player: false,
                    is_locked: is_their_locked,
                    win_rate: cwr,
                    tier: ctier,
                    avatar_url: cavatar,
                });
            }
        }

        let local_slot = self.state.my_team.iter().find(|s| s.is_local_player);
        let mut runes_val = json!(null);
        let mut spells_val = json!({
            "spell1Id": 4, // Flash
            "spell2Id": 14 // Ignite
        });
        let mut skill_order = String::new();

        if let Some(local) = local_slot {
            if !local.champion_id.is_empty() {
                if let Some(op) = opgg {
                    if let Some(cached_detail) = op.get_cached_champion_detail(&local.champion_id, &local.position) {
                        if let Some(runes) = cached_detail.get("recommendedRunes") {
                            if !runes.is_null() {
                                runes_val = runes.clone();
                            }
                        }
                        if let Some(spells_arr) = cached_detail.get("summonerSpells").and_then(|v| v.as_array()) {
                            if let Some(first_pair) = spells_arr.first().and_then(|v| v.as_array()) {
                                if first_pair.len() >= 2 {
                                    if let (Some(s1), Some(s2)) = (first_pair[0].as_i64(), first_pair[1].as_i64()) {
                                        spells_val = json!({
                                            "spell1Id": s1,
                                            "spell2Id": s2
                                        });
                                    }
                                }
                            }
                        }
                        if let Some(sk) = cached_detail.get("skillOrder").and_then(|v| v.as_str()) {
                            skill_order = sk.to_string();
                        }
                    }
                }
            }
        }

        if runes_val.is_null() {
            runes_val = json!({
                "name": "HexBrain: 推荐符文",
                "primaryStyleId": 8000,
                "subStyleId": 8200,
                "selectedPerkIds": [8010, 9111, 9105, 8299, 8224, 8234, 5008, 5008, 5001]
            });
        }

        self.state.decision_card = json!({
            "status": "ready",
            "recommendedRunes": runes_val,
            "recommendedSpells": spells_val,
            "skillOrder": skill_order
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_draft_empty_session_handling() {
        let mut mgr = RankedDraftManager::new();

        // 1. Completely empty object
        mgr.parse_session(&json!({}));
        assert!(!mgr.state.is_draft_active);
        assert!(mgr.state.my_team.is_empty());
        assert!(mgr.state.their_team.is_empty());

        // 2. myTeam is empty array
        mgr.parse_session(&json!({
            "myTeam": [],
            "theirTeam": []
        }));
        assert!(!mgr.state.is_draft_active);
        assert!(mgr.state.my_team.is_empty());
    }

    #[test]
    fn test_draft_sentinel_champion_ids_and_local_cell() {
        let mut mgr = RankedDraftManager::new();

        let session = json!({
            "myTeam": [
                { "cellId": 0, "championId": 0, "assignedPosition": "top" },
                { "cellId": 1, "championId": -1, "assignedPosition": "jungle" },
                { "cellId": 2, "championId": "103", "assignedPosition": "middle" }
            ],
            "theirTeam": [
                { "cellId": 5, "championId": "-1", "assignedPosition": "utility" }
            ],
            "localPlayerCellId": 1
        });

        mgr.parse_session(&session);
        assert!(mgr.state.is_draft_active);
        assert_eq!(mgr.state.my_team.len(), 3);

        // championId 0 and -1 should be treated as empty
        assert_eq!(mgr.state.my_team[0].champion_id, "");
        assert!(!mgr.state.my_team[0].is_local_player);

        assert_eq!(mgr.state.my_team[1].champion_id, "");
        assert!(mgr.state.my_team[1].is_local_player);

        assert_eq!(mgr.state.my_team[2].champion_id, "103");
        assert!(!mgr.state.my_team[2].is_local_player);

        // Opponent sentinel
        assert_eq!(mgr.state.their_team[0].champion_id, "");
    }

    #[test]
    fn test_manual_positions_override_and_reset() {
        let mut mgr = RankedDraftManager::new();

        let session = json!({
            "myTeam": [
                { "cellId": 0, "championId": 1, "assignedPosition": "top" }
            ]
        });

        mgr.parse_session(&session);
        assert_eq!(mgr.state.my_team[0].position, "TOP");

        // Manually set slot 0 to MID
        mgr.set_slot_position("my", 0, "mid");
        mgr.parse_session(&session);
        assert_eq!(mgr.state.my_team[0].position, "MID");
        assert_eq!(mgr.state.my_team[0].position_cn, "中单");

        // Reset manual positions
        mgr.reset_manual_positions();
        mgr.parse_session(&session);
        assert_eq!(mgr.state.my_team[0].position, "TOP");
    }

    #[test]
    fn test_draft_decision_card_with_cached_runes() {
        let mut mgr = RankedDraftManager::new();
        let opgg = crate::crawler::OpggClient::new();

        let session = json!({
            "myTeam": [
                { "cellId": 0, "championId": 103, "assignedPosition": "middle" }
            ],
            "theirTeam": [],
            "localPlayerCellId": 0
        });

        mgr.parse_session_with_db_and_opgg(&session, None, Some(&opgg));
        assert!(mgr.state.is_draft_active);
        let dc = &mgr.state.decision_card;
        assert_eq!(dc["status"], "ready");
        assert!(dc["recommendedRunes"].is_object());
        assert_eq!(dc["recommendedSpells"]["spell1Id"], 4);
    }
}
