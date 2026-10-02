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

        if my_team_arr.is_none() {
            self.state.is_draft_active = false;
            return;
        }

        self.state.is_draft_active = true;
        self.state.my_team.clear();
        self.state.their_team.clear();

        if let Some(arr) = my_team_arr {
            for (idx, m) in arr.iter().enumerate() {
                let cell_id = m.get("cellId").and_then(|v| v.as_i64()).unwrap_or(0);
                let raw_cid = m.get("championId").or_else(|| m.get("championPickIntent"))
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => "".to_string(),
                    })
                    .unwrap_or_default();
                let cid = if raw_cid == "0" { String::new() } else { raw_cid };

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
                let pos = self.manual_positions.get(&key)
                    .cloned()
                    .or_else(|| auto_pos.map(|s| s.to_string()))
                    .unwrap_or_else(|| ORDERED_POSITIONS[idx % 5].to_string());

                let pos_idx = ORDERED_POSITIONS.iter().position(|&p| p == pos).unwrap_or(idx % 5);

                let (cname, cavatar, mut cwr, mut ctier) = if let Some(database) = db {
                    if !cid.is_empty() {
                        let resolved_id = database.resolve_hero_id(&cid).unwrap_or_else(|| cid.clone());
                        let name = database.resolve_hero_name(&resolved_id)
                            .unwrap_or_else(|| format!("队友{}", idx + 1));
                        let summary = database.heroes_map.get(&resolved_id);
                        let avatar = summary.map(|s| s.avatar_url.clone()).unwrap_or_default();
                        let wr = summary.map(|s| s.win_rate).unwrap_or(0.505);
                        let tier = summary.and_then(|s| s.tier).unwrap_or(2);
                        (name, avatar, wr, tier)
                    } else {
                        (format!("队友{}", idx + 1), String::new(), 0.505, 2)
                    }
                } else {
                    (format!("队友{}", idx + 1), String::new(), 0.505, 2)
                };

                if let Some(op) = opgg {
                    if !cid.is_empty() {
                        if let Some((op_wr, op_tier)) = op.get_cached_stats(&cid, &pos) {
                            cwr = op_wr;
                            ctier = op_tier;
                        }
                    }
                }

                self.state.my_team.push(DraftSlot {
                    cell_id,
                    champion_id: cid,
                    champion_name: cname,
                    position: pos,
                    position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                    is_local_player: cell_id == local_cell_id,
                    win_rate: cwr,
                    tier: ctier,
                    avatar_url: cavatar,
                });
            }
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
                let cid = if raw_cid == "0" { String::new() } else { raw_cid };

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
                let pos = self.manual_positions.get(&key)
                    .cloned()
                    .or_else(|| auto_pos.map(|s| s.to_string()))
                    .unwrap_or_else(|| ORDERED_POSITIONS[idx % 5].to_string());

                let pos_idx = ORDERED_POSITIONS.iter().position(|&p| p == pos).unwrap_or(idx % 5);

                let (cname, cavatar, mut cwr, mut ctier) = if let Some(database) = db {
                    if !cid.is_empty() {
                        let resolved_id = database.resolve_hero_id(&cid).unwrap_or_else(|| cid.clone());
                        let name = database.resolve_hero_name(&resolved_id)
                            .unwrap_or_else(|| format!("对手{}", idx + 1));
                        let summary = database.heroes_map.get(&resolved_id);
                        let avatar = summary.map(|s| s.avatar_url.clone()).unwrap_or_default();
                        let wr = summary.map(|s| s.win_rate).unwrap_or(0.495);
                        let tier = summary.and_then(|s| s.tier).unwrap_or(3);
                        (name, avatar, wr, tier)
                    } else {
                        (format!("对手{}", idx + 1), String::new(), 0.495, 3)
                    }
                } else {
                    (format!("对手{}", idx + 1), String::new(), 0.495, 3)
                };

                if let Some(op) = opgg {
                    if !cid.is_empty() {
                        if let Some((op_wr, op_tier)) = op.get_cached_stats(&cid, &pos) {
                            cwr = op_wr;
                            ctier = op_tier;
                        }
                    }
                }

                self.state.their_team.push(DraftSlot {
                    cell_id,
                    champion_id: cid,
                    champion_name: cname,
                    position: pos,
                    position_cn: SHORT_POS_NAMES_CN[pos_idx].to_string(),
                    is_local_player: false,
                    win_rate: cwr,
                    tier: ctier,
                    avatar_url: cavatar,
                });
            }
        }

        self.state.decision_card = json!({
            "status": "ready",
            "recommendedRunes": {
                "name": "HexBrain: 推荐符文",
                "primaryStyleId": 8000,
                "subStyleId": 8200,
                "selectedPerkIds": [8010, 9111, 9105, 8299, 8224, 8234, 5008, 5008, 5001]
            },
            "recommendedSpells": {
                "spell1Id": 4, // Flash
                "spell2Id": 14 // Ignite
            }
        });
    }
}
