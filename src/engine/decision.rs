use crate::config::RecommendationConfig;
use crate::data::loader::Database;
use crate::data::recipe_data::calculate_item_net_cost;
use crate::engine::archetype::ArchetypeEngine;
use crate::engine::counter::CounterItemEngine;
use crate::engine::synergy::HexItemSynergyEngine;
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn calc_wilson_lower_bound(p: f64, n: i64, z: f64) -> f64 {
    if p.is_nan() || p.is_infinite() || z.is_nan() || z.is_infinite() {
        return 0.5;
    }
    if n <= 0 {
        return p.clamp(0.0, 1.0);
    }
    let p = p.clamp(0.0, 1.0);
    let n_f = n as f64;
    let z2 = z * z;
    let denominator = 1.0 + z2 / n_f;
    let centre = p + z2 / (2.0 * n_f);
    let mut discriminant = (p * (1.0 - p) / n_f) + (z2 / (4.0 * n_f * n_f));
    if discriminant < 0.0 || discriminant.is_nan() {
        discriminant = 0.0;
    }
    let spread = z * discriminant.sqrt();
    let lower = (centre - spread) / denominator;
    if lower.is_nan() {
        p
    } else {
        lower.clamp(0.0, 1.0)
    }
}

pub struct DecisionEngine;

impl DecisionEngine {
    pub fn get_meta_info(db: &Database) -> Value {
        json!({
            "patch": db.meta.report_patch,
            "reportDate": db.meta.report_date,
            "totalMatches": db.meta.sample_scope.snapshot_matches,
            "eligibleMatches": db.meta.sample_scope.eligible_matches,
            "regions": db.meta.sample_scope.platforms,
            "heroCount": db.heroes_map.len(),
            "augmentCount": db.augments_map.len()
        })
    }

    pub fn get_heroes_list(
        db: &Database,
        role: Option<&str>,
        search: Option<&str>,
        tier: Option<i32>,
        sort_by: &str,
    ) -> Vec<Value> {
        let q = search.map(|s| s.trim().to_lowercase()).unwrap_or_default();
        let target_role = role.map(|r| {
            let r_lower = r.trim().to_lowercase();
            match r_lower.as_str() {
                "marksman" | "射手" => "射手",
                "mage" | "法师" => "法师",
                "fighter" | "战士" => "战士",
                "tank" | "坦克" => "坦克",
                "assassin" | "刺客" => "刺客",
                "support" | "辅助" => "辅助",
                _ => r.trim(),
            }
        });

        let mut results = Vec::new();

        for h in &db.heroes_summary {
            let htier = h.tier.unwrap_or(5);

            if let Some(tr) = target_role {
                if !h.roles.iter().any(|r| r == tr) && !h.role_tags.iter().any(|r| r == tr) {
                    continue;
                }
            }
            if let Some(t) = tier {
                if htier != t {
                    continue;
                }
            }
            if !q.is_empty() {
                let matches_name = h.name.to_lowercase().contains(&q);
                let matches_id = h.id == q;
                let matches_terms = h.search_terms.iter().any(|t| t.to_lowercase().contains(&q));
                if !matches_name && !matches_id && !matches_terms {
                    continue;
                }
            }

            results.push(json!({
                "id": h.id,
                "name": h.name,
                "tier": htier,
                "winRate": h.win_rate,
                "pickRate": h.pick_rate,
                "banRate": h.ban_rate,
                "games": h.games,
                "kda": h.kda,
                "roles": h.roles,
                "avatarUrl": h.avatar_url,
                "topAugments": h.top_augments.iter().take(3).collect::<Vec<_>>()
            }));
        }

        match sort_by {
            "winRate" => results.sort_by(|a, b| {
                let wa = a.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let wb = b.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
            }),
            "pickRate" => results.sort_by(|a, b| {
                let pa = a.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let pb = b.get("pickRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
            }),
            "games" => results.sort_by(|a, b| {
                let ga = a.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                let gb = b.get("games").and_then(|v| v.as_i64()).unwrap_or(0);
                gb.cmp(&ga)
            }),
            _ => results.sort_by(|a, b| {
                let ta = a.get("tier").and_then(|v| v.as_i64()).unwrap_or(99);
                let tb = b.get("tier").and_then(|v| v.as_i64()).unwrap_or(99);
                if ta != tb {
                    ta.cmp(&tb)
                } else {
                    let wa = a.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let wb = b.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
                }
            }),
        }

        results
    }

    pub fn calculate_reroll_roi(tier: i32, win_rate: f64) -> Value {
        let avg_pool_wr = 0.500;

        if tier == 1 || win_rate >= 0.53 {
            json!({
                "action": "LOCK",
                "label": "👑 绝对核心·锁死勿换",
                "color": "emerald",
                "desc": format!("胜率高达 {:.1}%，版本绝对 T1，极高胜率期望，切勿重摇或换人！", win_rate * 100.0)
            })
        } else if tier == 2 || win_rate >= 0.505 {
            json!({
                "action": "KEEP",
                "label": "✅ 强势英雄·建议保留",
                "color": "blue",
                "desc": format!("胜率稳健 ({:.1}%)，除非阵容极度缺乏关键前排/控制，否则推荐首发。", win_rate * 100.0)
            })
        } else if tier == 3 {
            json!({
                "action": "OPTIONAL",
                "label": "⚖️ 中规中矩·视阵容调配",
                "color": "yellow",
                "desc": format!("胜率持平大盘 ({:.1}%)，可根据己方 AP/AD 平衡与队友骰子池灵活置换。", win_rate * 100.0)
            })
        } else if tier == 4 {
            json!({
                "action": "REROLL",
                "label": "🎲 建议重摇·期望更高",
                "color": "orange",
                "desc": format!("胜率偏低 ({:.1}%)，摇骰子获取 T1/T2 英雄的期望提升达到 +{:.1}%。", win_rate * 100.0, (avg_pool_wr - win_rate) * 100.0)
            })
        } else {
            json!({
                "action": "MUST_REROLL",
                "label": "🚫 极弱底谷·必须重随",
                "color": "rose",
                "desc": format!("胜率垫底 ({:.1}%)，严重拖累全队胜率，强烈建议骰掉或从待选席更换！", win_rate * 100.0)
            })
        }
    }

    pub fn get_hero_decision_profile(db: &Database, hero_id: &str) -> Option<Value> {
        let hid = db.resolve_hero_id(hero_id)?;
        let summary = db.heroes_map.get(&hid)?;
        let detail = db.heroes_detail.get(&hid);

        let hero_tier = summary.tier.unwrap_or(3);
        let hero_win_rate = summary.win_rate;
        let reroll_advice = Self::calculate_reroll_roi(hero_tier, hero_win_rate);

        let raw_augments = detail.map(|d| d.augments.clone()).unwrap_or(Value::Null);
        let classified_augments = Self::classify_augments(&raw_augments, hero_win_rate);

        let raw_items = detail.map(|d| d.items.clone()).unwrap_or(Value::Null);
        let items_data = Self::process_items(&raw_items);

        let terminal_trios = detail
            .map(|d| d.terminal_item_trios.clone())
            .unwrap_or_default();
        let augment_trios = detail.map(|d| d.trios.clone()).unwrap_or_default();
        let spells = detail
            .map(|d| d.summoner_spell_pairs.clone())
            .unwrap_or_default();
        let strong_against = detail.map(|d| d.strong_against.clone()).unwrap_or_default();
        let weak_against = detail.map(|d| d.weak_against.clone()).unwrap_or_default();
        let teammates = detail
            .map(|d| d.teammate_synergies.clone())
            .unwrap_or_default();

        let postmatch = db.postmatch_data.get(&hid);

        Some(json!({
            "summary": {
                "id": hid,
                "name": summary.name,
                "tier": hero_tier,
                "winRate": hero_win_rate,
                "pickRate": summary.pick_rate,
                "banRate": summary.ban_rate,
                "games": summary.games,
                "kda": summary.kda,
                "roles": summary.roles,
                "avatarUrl": summary.avatar_url,
                "rerollAdvice": reroll_advice
            },
            "augments": classified_augments,
            "augmentTrios": augment_trios,
            "items": items_data,
            "terminalTrios": terminal_trios,
            "summonerSpells": spells,
            "relationships": {
                "strongAgainst": strong_against,
                "weakAgainst": weak_against,
                "teammates": teammates
            },
            "postmatch": postmatch
        }))
    }

    fn classify_augments(raw_augments: &Value, hero_win_rate: f64) -> Value {
        let mut aug_list = Vec::new();
        if let Some(arr) = raw_augments.as_array() {
            aug_list.extend(arr.clone());
        } else if let Some(obj) = raw_augments.as_object() {
            for key in ["sTier", "aTier", "bTier", "traps"] {
                if let Some(arr) = obj.get(key).and_then(|v| v.as_array()) {
                    aug_list.extend(arr.clone());
                }
            }
        }

        let mut s_tier = Vec::new();
        let mut a_tier = Vec::new();
        let mut b_tier = Vec::new();
        let mut traps = Vec::new();

        for a in aug_list {
            let delta = a.get("deltaWinRate")
                .and_then(|v| v.as_f64())
                .unwrap_or_else(|| {
                    a.get("winRate")
                        .and_then(|v| v.as_f64())
                        .map(|wr| wr - hero_win_rate)
                        .unwrap_or(0.0)
                });
            let games = a.get("games").and_then(|v| v.as_i64()).unwrap_or(0);

            if delta < -0.005 && games >= 100 {
                traps.push(a);
            } else if delta >= 0.035 {
                s_tier.push(a);
            } else if delta >= 0.012 {
                a_tier.push(a);
            } else {
                b_tier.push(a);
            }
        }

        json!({
            "sTier": s_tier,
            "aTier": a_tier,
            "bTier": b_tier,
            "traps": traps
        })
    }

    fn process_items(raw_items: &Value) -> Value {
        let mut items_list = Vec::new();
        if let Some(arr) = raw_items.as_array() {
            items_list.extend(arr.clone());
        } else if let Some(obj) = raw_items.as_object() {
            if let Some(all) = obj.get("allItems").and_then(|v| v.as_array()) {
                items_list.extend(all.clone());
            }
            if let Some(core) = obj.get("coreItems").and_then(|v| v.as_array()) {
                items_list.extend(core.clone());
            }
        }

        json!({
            "allItems": items_list
        })
    }

    pub fn compare_augments(
        db: &Database,
        hero_id: &str,
        augment_ids: &[String],
        current_stage: Option<i32>,
        existing_augments: &[String],
    ) -> Value {
        let hid = match db.resolve_hero_id(hero_id) {
            Some(id) => id,
            None => {
                return json!({ "error": format!("未找到该英雄数据: {}", hero_id) });
            }
        };

        let hero_summary = match db.heroes_map.get(&hid) {
            Some(s) => s,
            None => {
                return json!({ "error": format!("未找到该英雄数据: {}", hid) });
            }
        };

        let hero_win_rate = hero_summary.win_rate;
        let mut candidates = Vec::new();

        let hero_detail = db.heroes_detail.get(&hid);
        let hero_aug_list: Vec<Value> = hero_detail
            .map(|d| {
                if let Some(arr) = d.augments.as_array() {
                    arr.clone()
                } else if let Some(obj) = d.augments.as_object() {
                    obj.get("allAugments")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default();

        let augment_trios = hero_detail.map(|d| d.trios.clone()).unwrap_or_default();

        for aid in augment_ids {
            let resolved_id = db.resolve_augment_id(aid).unwrap_or_else(|| aid.clone());
            let global_stat = db.augments_map.get(&resolved_id);

            // Try to find champion-specific augment stats
            let hero_specific_stat = hero_aug_list.iter().find(|ha| {
                let hid_match = ha.get("id").or_else(|| ha.get("augmentId"))
                    .map(|v| match v {
                        Value::String(s) => s == &resolved_id || s == aid,
                        Value::Number(n) => n.to_string() == resolved_id || n.to_string() == *aid,
                        _ => false,
                    })
                    .unwrap_or(false);
                let name_match = ha.get("name").or_else(|| ha.get("augmentName"))
                    .and_then(|v| v.as_str())
                    .map(|n| n == aid || n == &resolved_id)
                    .unwrap_or(false);
                hid_match || name_match
            });

            let name = hero_specific_stat
                .and_then(|ha| ha.get("name").or_else(|| ha.get("augmentName")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| global_stat.map(|a| a.name.clone()))
                .unwrap_or_else(|| format!("强化#{}", aid));

            let rarity = hero_specific_stat
                .and_then(|ha| ha.get("rarity"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| global_stat.map(|a| a.rarity.clone()))
                .unwrap_or_else(|| "白银".to_string());

            let pair_wr = hero_specific_stat
                .and_then(|ha| ha.get("winRate").or_else(|| ha.get("pairWinRate")))
                .and_then(|v| v.as_f64())
                .or_else(|| global_stat.map(|a| a.win_rate))
                .unwrap_or(hero_win_rate);

            let delta = hero_specific_stat
                .and_then(|ha| ha.get("deltaWinRate"))
                .and_then(|v| v.as_f64())
                .unwrap_or(pair_wr - hero_win_rate);

            let games = hero_specific_stat
                .and_then(|ha| ha.get("games"))
                .and_then(|v| v.as_i64())
                .or_else(|| global_stat.map(|a| a.games))
                .unwrap_or(0);

            let hex_score = hero_specific_stat
                .and_then(|ha| ha.get("hexScore"))
                .and_then(|v| v.as_f64())
                .or_else(|| global_stat.map(|a| a.hex_score))
                .unwrap_or(50.0);

            let description = hero_specific_stat
                .and_then(|ha| ha.get("description").or_else(|| ha.get("augmentDescription")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| global_stat.map(|a| a.description.clone()))
                .unwrap_or_default();

            let cdn_icon_url = hero_specific_stat
                .and_then(|ha| ha.get("iconUrl").or_else(|| ha.get("augmentIconUrl")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| global_stat.map(|a| a.cdn_icon_url.clone()))
                .unwrap_or_default();

            let wilson_wr = calc_wilson_lower_bound(pair_wr, games, 1.96);
            let mut score = 50.0 + (delta * 400.0) + (wilson_wr - 0.5) * 50.0;
            if games < 100 {
                score -= 10.0;
            }

            // Check augment trio synergy with existing augments
            let mut trio_synergy_found: Option<(String, f64)> = None;
            if !existing_augments.is_empty() {
                for trio in &augment_trios {
                    let mut trio_names: Vec<String> = Vec::new();
                    if let Some(names) = trio.get("augmentNames").and_then(|v| v.as_array()) {
                        trio_names = names.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                    }
                    let mut trio_ids: Vec<String> = Vec::new();
                    if let Some(ids) = trio.get("augmentIds").and_then(|v| v.as_array()) {
                        trio_ids = ids.iter().filter_map(|x| match x {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        }).collect();
                    }

                    let matches_cand = trio_names.iter().any(|n| n == &name)
                        || trio_ids.iter().any(|id| id == &resolved_id || id == aid);

                    if matches_cand {
                        for ex in existing_augments {
                            let ex_resolved = db.resolve_augment_id(ex).unwrap_or_else(|| ex.to_string());
                            let ex_name = db.augments_map.get(&ex_resolved).map(|a| a.name.clone()).unwrap_or_else(|| ex.to_string());
                            let matches_ex = trio_names.iter().any(|n| n == ex || n == &ex_name)
                                || trio_ids.iter().any(|id| id == ex || id == &ex_resolved);
                            if matches_ex {
                                let trio_wr = trio.get("winRate").and_then(|v| v.as_f64()).unwrap_or(pair_wr);
                                trio_synergy_found = Some((ex_name, trio_wr));
                                break;
                            }
                        }
                    }
                    if trio_synergy_found.is_some() {
                        break;
                    }
                }
            }

            if trio_synergy_found.is_some() {
                score += 15.0;
            }

            let is_destiny = delta >= 0.035;
            let is_trap = delta < -0.005;

            let (tier_badge, badge_label, badge_color) = if is_trap {
                ("TRAP", "🚫 避雷陷阱 (负收益)", "rose")
            } else if is_destiny {
                ("S", "👑 英雄天命 S 级", "emerald")
            } else if delta >= 0.012 {
                ("A", "✨ 强势 A 级优选", "blue")
            } else {
                ("B", "⚖️ 常规 B 级可用", "slate")
            };

            let synergy_partner = trio_synergy_found.as_ref().map(|(n, _)| n.clone());
            let synergy_wr = trio_synergy_found.as_ref().map(|(_, wr)| *wr);
            let synergy_note = trio_synergy_found.as_ref().map(|(n, wr)| {
                format!("与已选强化[{}]存在高胜率组合联动 ({:.1}%)", n, wr * 100.0)
            });

            candidates.push(json!({
                "augmentId": resolved_id,
                "name": name,
                "rarity": rarity,
                "pairWinRate": pair_wr,
                "deltaWinRate": delta,
                "games": games,
                "hexScore": hex_score,
                "wilsonWinRate": (wilson_wr * 1000.0).round() / 1000.0,
                "score": (score * 10.0).round() / 10.0,
                "tierBadge": tier_badge,
                "badgeLabel": badge_label,
                "badgeColor": badge_color,
                "isDestiny": is_destiny,
                "isTrap": is_trap,
                "hasTrioSynergy": trio_synergy_found.is_some(),
                "synergyPartner": synergy_partner,
                "synergyWinRate": synergy_wr,
                "synergyNote": synergy_note,
                "description": description,
                "cdnIconUrl": cdn_icon_url
            }));
        }

        candidates.sort_by(|a, b| {
            let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });

        let best_candidate = candidates.first().cloned();
        let all_traps = candidates
            .iter()
            .all(|c| c.get("deltaWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0) < 0.0);

        let rationale = if let Some(ref best) = best_candidate {
            let bname = best.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let bdelta = best.get("deltaWinRate").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0;
            let syn_note = best.get("synergyNote").and_then(|v| v.as_str());

            if all_traps {
                format!("🚨 全陷阱警告：当前 3 个候选强化对 {} 均为负收益！建议重掷！若无骰子勉强选【{}】。", hero_summary.name, bname)
            } else if let Some(sn) = syn_note {
                format!("✨ 推荐【{}】（{}），契合度与实战容错率极佳 (+{:.2}%)。", bname, sn, bdelta)
            } else if best.get("isDestiny").and_then(|v| v.as_bool()).unwrap_or(false) {
                format!("👑 首选【{}】！带来高达 +{:.2}% 净胜率暴增，是当前最强质变核心强化！", bname, bdelta)
            } else {
                format!("✨ 推荐【{}】，契合度与实战容错率极佳的强势优选 (+{:.2}%)。", bname, bdelta)
            }
        } else {
            "暂无可对比强化".to_string()
        };

        let reroll_decision = if all_traps {
            json!({
                "action": "REROLL",
                "label": "🎲 强烈建议消耗 1 颗骰子重摇",
                "desc": format!("当前 3 个强化对 {} 契合度偏低，重随摇出 T1 天命强化的期望概率极高！", hero_summary.name)
            })
        } else if let Some(ref b) = best_candidate {
            if b.get("isDestiny").and_then(|v| v.as_bool()).unwrap_or(false) {
                json!({
                    "action": "LOCK",
                    "label": "👑 天命已至 · 立即锁定",
                    "desc": "首选强化收益爆炸，建议立即锁定保留骰子！"
                })
            } else {
                json!({
                    "action": "ACCEPT",
                    "label": "⚖️ 收益稳健 · 可以采纳",
                    "desc": "首选强化胜率稳定，可按当局节奏决定是否保留骰子。"
                })
            }
        } else {
            json!({
                "action": "OPTIONAL",
                "label": "无建议",
                "desc": "暂无可供对比的海克斯候选强化。"
            })
        };

        json!({
            "hero": {
                "id": hid,
                "name": hero_summary.name,
                "winRate": hero_win_rate,
                "avatarUrl": hero_summary.avatar_url
            },
            "currentStage": current_stage,
            "allTrapsAlert": all_traps,
            "bestChoiceId": best_candidate.as_ref().and_then(|b| b.get("augmentId").cloned()),
            "recommendationRationale": rationale,
            "rerollDecision": reroll_decision,
            "candidates": candidates
        })
    }

    pub fn search_augments_for_hero(db: &Database, hero_id: &str, query: &str) -> Vec<Value> {
        let q = query.trim().to_lowercase();
        let hid = db.resolve_hero_id(hero_id).unwrap_or_else(|| hero_id.to_string());
        let hero_detail = db.heroes_detail.get(&hid);

        let mut results = Vec::new();

        for (aid, aug) in &db.augments_map {
            let name_match = aug.name.to_lowercase().contains(&q);
            let id_match = aid == &q;
            let desc_match = aug.description.to_lowercase().contains(&q);

            if q.is_empty() || name_match || id_match || desc_match {
                // If hero detail has champion-specific augment stats, prefer them
                let (wr, games) = if let Some(detail) = hero_detail {
                    let mut found = None;
                    if let Some(arr) = detail.augments.as_array() {
                        for item in arr {
                            let match_id = item.get("id").and_then(|v| v.as_str()) == Some(aid.as_str());
                            let match_name = item.get("name").and_then(|v| v.as_str()) == Some(aug.name.as_str());
                            if match_id || match_name {
                                let w = item.get("winRate").and_then(|v| v.as_f64()).unwrap_or(aug.win_rate);
                                let g = item.get("games").and_then(|v| v.as_i64()).unwrap_or(aug.games);
                                found = Some((w, g));
                                break;
                            }
                        }
                    }
                    found.unwrap_or((aug.win_rate, aug.games))
                } else {
                    (aug.win_rate, aug.games)
                };

                results.push(json!({
                    "id": aid,
                    "name": aug.name,
                    "rarity": aug.rarity,
                    "winRate": wr,
                    "games": games,
                    "description": aug.description,
                    "cdnIconUrl": aug.cdn_icon_url
                }));
            }
        }

        results.sort_by(|a, b| {
            let wa = a.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let wb = b.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.0);
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });

        results.into_iter().take(30).collect()
    }

    pub fn get_item_recommendation(
        db: &Database,
        hero_id: &str,
        current_items: &[String],
        gold: f64,
        level: i32,
        existing_augments: &[String],
        enemy_team: &[Value],
        cfg: &RecommendationConfig,
    ) -> Value {
        let hid = db.resolve_hero_id(hero_id).unwrap_or_else(|| hero_id.to_string());
        let hero_detail = db.heroes_detail.get(&hid);

        let mut candidate_items = Vec::new();
        if let Some(d) = hero_detail {
            if let Some(arr) = d.items.as_array() {
                candidate_items.extend(arr.clone());
            } else if let Some(obj) = d.items.as_object() {
                if let Some(all) = obj.get("allItems").and_then(|v| v.as_array()) {
                    candidate_items.extend(all.clone());
                }
            }
        }

        // Initialize tacticalScore
        for it in &mut candidate_items {
            let wr = it.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let hex_score = it.get("hexScore").and_then(|v| v.as_f64()).unwrap_or(50.0);
            it["tacticalScore"] = Value::from((hex_score * 0.5 + wr * 50.0).round());
        }

        let owned_set: HashSet<String> = current_items.iter().cloned().collect();

        // 1. Archetype Engine
        let active_aug_values: Vec<Value> = existing_augments
            .iter()
            .map(|a| json!({ "name": a }))
            .collect();

        let archetype_info = ArchetypeEngine::detect_archetype(
            &hid,
            current_items,
            &active_aug_values,
            &candidate_items,
            cfg.resolve_conflicts,
        );

        // 2. Counter Item Engine
        let threat_profile = CounterItemEngine::analyze_enemy_threats(enemy_team);
        let urgent_counter_ids = CounterItemEngine::get_urgent_counter_ids(&threat_profile, &archetype_info);

        let filtered_items = ArchetypeEngine::filter_and_boost_by_archetype(
            &candidate_items,
            &archetype_info,
            &urgent_counter_ids,
            cfg.archetype_strictness,
            cfg.resolve_conflicts,
        );

        let countered_items = CounterItemEngine::apply_counter_recommendations(
            &filtered_items,
            &threat_profile,
            &archetype_info,
            &owned_set,
            cfg.counter_intensity,
            cfg.resolve_conflicts,
        );

        // 3. Hex-Item Synergy Engine
        let build_path = HexItemSynergyEngine::rerank_items_with_synergy(
            &countered_items,
            &active_aug_values,
            &owned_set,
            &hid,
            cfg.hex_synergy_weight,
            cfg.resolve_conflicts,
        );

        // Next item recommendation
        let next_item = build_path.iter().find(|it| {
            let iid = it.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
            !owned_set.contains(iid)
        }).cloned();

        let gold_info = next_item.as_ref().map(|item| {
            let iid = item.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
            calculate_item_net_cost(iid, current_items, gold, &cfg.gold_calc_mode)
        });

        let terminal_trios = hero_detail
            .map(|d| d.terminal_item_trios.clone())
            .unwrap_or_default();
        let recommended_trio = ArchetypeEngine::align_terminal_trio_with_archetype(
            &terminal_trios,
            &archetype_info,
            &owned_set,
            &countered_items,
            cfg.resolve_conflicts,
            &cfg.trio_mode,
        );

        json!({
            "heroId": hid,
            "gold": gold,
            "level": level,
            "archetype": archetype_info,
            "threatProfile": threat_profile,
            "nextItem": next_item,
            "goldInfo": gold_info,
            "recommendedTrio": recommended_trio,
            "terminalTrios": terminal_trios,
            "buildPath": build_path.into_iter().take(6).collect::<Vec<_>>()
        })
    }

    pub fn analyze_team_synergy(db: &Database, hero_ids: &[String]) -> Value {
        let mut resolved_heroes = Vec::new();
        for id in hero_ids {
            if let Some(hid) = db.resolve_hero_id(id) {
                if let Some(h) = db.heroes_map.get(&hid) {
                    resolved_heroes.push(h);
                }
            }
        }

        let mut ap_count = 0;
        let mut ad_count = 0;
        let mut tank_count = 0;

        for h in &resolved_heroes {
            for role in &h.roles {
                match role.as_str() {
                    "法师" => ap_count += 1,
                    "射手" | "刺客" => ad_count += 1,
                    "坦克" => tank_count += 1,
                    _ => {}
                }
            }
        }

        let is_balanced = ap_count >= 1 && ad_count >= 1 && tank_count >= 1;
        let advice = if is_balanced {
            "阵容攻守兼备，前后排结构合理"
        } else if tank_count == 0 {
            "缺乏前排承伤核心，建议通过骰子补足重装坦克"
        } else if ap_count == 0 {
            "纯菜刀队易被敌方叠护甲克制，建议补足 AP 法系输出"
        } else {
            "阵容整体偏软，注意前中期抱团"
        };

        json!({
            "heroCount": resolved_heroes.len(),
            "apCount": ap_count,
            "adCount": ad_count,
            "tankCount": tank_count,
            "isBalanced": is_balanced,
            "synergyAdvice": advice
        })
    }
}
