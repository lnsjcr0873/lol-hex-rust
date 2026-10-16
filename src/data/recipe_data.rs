use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetadata {
    pub total_costs: HashMap<String, i64>,
    pub recipes: HashMap<String, Vec<String>>,
    pub component_prices: HashMap<String, i64>,
    pub component_names: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummaryEntry {
    #[serde(default)]
    pub item_id: String,
    #[serde(default)]
    pub item_name: String,
    #[serde(default)]
    pub cdn_icon_url: Option<String>,
}

lazy_static! {
    pub static ref RECIPE_META: RecipeMetadata = {
        let raw = include_str!("../../data/recipe_metadata.json");
        serde_json::from_str(raw).expect("Failed to parse embedded recipe_metadata.json")
    };
    pub static ref ITEMS_SUMMARY: HashMap<String, ItemSummaryEntry> = {
        let raw = include_str!("../../data/items_summary.json");
        serde_json::from_str(raw).unwrap_or_default()
    };
}

pub fn resolve_item_name(item_id: &str) -> String {
    let clean_id = normalize_item_id(item_id);
    if let Some(entry) = ITEMS_SUMMARY.get(&clean_id) {
        if !entry.item_name.is_empty() {
            return entry.item_name.clone();
        }
    }
    if let Some(name) = RECIPE_META.component_names.get(&clean_id) {
        return name.clone();
    }
    match clean_id.as_str() {
        "1054" => "多兰之盾".to_string(),
        "1055" => "多兰之刃".to_string(),
        "1056" => "多兰之戒".to_string(),
        "1082" => "黑暗封印".to_string(),
        "1083" => "萃取".to_string(),
        "2003" => "生命药水".to_string(),
        "2031" => "复用型药水".to_string(),
        "2033" => "腐败药水".to_string(),
        "2055" => "控制守卫".to_string(),
        "3006" => "狂战士胫甲".to_string(),
        "3009" => "轻灵之靴".to_string(),
        "3020" => "法师之靴".to_string(),
        "3047" => "铁板靴".to_string(),
        "3111" => "水银之靴".to_string(),
        "3117" => "疾行之靴".to_string(),
        "3158" => "明朗之靴".to_string(),
        "3010" => "共生草鞋".to_string(),
        "3865" => "云游图标".to_string(),
        "3866" => "符文罗盘".to_string(),
        "3867" => "世界地图".to_string(),
        "3869" => "星界异界".to_string(),
        "3870" => "圆梦使者".to_string(),
        "3871" => "扎兹沙克的溃口".to_string(),
        "3876" => "血鸣".to_string(),
        "3877" => "至尊极冰碎片".to_string(),
        "1101" => "风行幼体".to_string(),
        "1102" => "踏苔幼体".to_string(),
        "1103" => "焰爪幼体".to_string(),
        _ => {
            if clean_id.is_empty() {
                "未知装备".to_string()
            } else {
                format!("装备#{}", clean_id)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MatchedComponent {
    pub item_id: String,
    pub name: String,
    pub price: i64,
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ItemNetCostResult {
    pub target_item_id: String,
    pub total_cost: i64,
    pub net_cost: i64,
    pub deducted_gold: i64,
    pub matched_components: Vec<MatchedComponent>,
    pub has_all_components: bool,
    pub is_affordable: bool,
    pub remaining_diff: i64,
    pub status_text: String,
    pub calc_mode: String,
}

pub fn normalize_item_id(item_id: &str) -> String {
    let tid = item_id.trim();
    if tid.is_empty() {
        return "".to_string();
    }
    if RECIPE_META.total_costs.contains_key(tid) || RECIPE_META.recipes.contains_key(tid) {
        return tid.to_string();
    }
    let digits: String = tid.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && (RECIPE_META.total_costs.contains_key(&digits) || RECIPE_META.recipes.contains_key(&digits)) {
        return digits;
    }
    for pfx_len in [2, 3] {
        if tid.len() > pfx_len {
            let sub = &tid[pfx_len..];
            if RECIPE_META.total_costs.contains_key(sub) {
                return sub.to_string();
            }
        }
    }
    tid.to_string()
}

pub fn calculate_item_net_cost(
    target_item_id: &str,
    owned_item_ids: &[String],
    current_gold: f64,
    gold_calc_mode: &str,
) -> ItemNetCostResult {
    let raw_tid = target_item_id.trim();
    let tid = normalize_item_id(raw_tid);
    let total_cost = (*RECIPE_META
        .total_costs
        .get(&tid)
        .or_else(|| RECIPE_META.total_costs.get(raw_tid))
        .unwrap_or(&0)).max(0);

    let cur_gold_int = if current_gold.is_nan() || current_gold.is_infinite() {
        0
    } else {
        (current_gold.max(0.0)) as i64
    };

    if gold_calc_mode == "FULL_COST" {
        let diff = (total_cost - cur_gold_int).max(0);
        let is_afford = cur_gold_int >= total_cost && total_cost > 0;
        let status_text = if total_cost == 0 {
            format!("💰 {}g [价格未收录]", cur_gold_int)
        } else if is_afford {
            format!("💰 {}/{}g [可全款购买]", cur_gold_int, total_cost)
        } else {
            format!("💰 {}/{}g (差{}g)", cur_gold_int, total_cost, diff)
        };

        return ItemNetCostResult {
            target_item_id: if !raw_tid.is_empty() { raw_tid.to_string() } else { tid },
            total_cost,
            net_cost: total_cost,
            deducted_gold: 0,
            matched_components: Vec::new(),
            has_all_components: false,
            is_affordable: is_afford,
            remaining_diff: diff,
            status_text,
            calc_mode: "FULL_COST".to_string(),
        };
    }

    // Mode: NET_DEDUCT / NET_REMAINING
    let mut available_owned: Vec<String> = owned_item_ids
        .iter()
        .map(|s| normalize_item_id(s.trim()))
        .filter(|norm| {
            !norm.is_empty() && norm != "0" && norm != &tid && norm != raw_tid && norm != "None" && norm != "undefined"
        })
        .collect();

    let mut matched_components = Vec::new();
    let mut total_deducted: i64 = 0;

    fn satisfy_component(
        req_item_id: &str,
        available_owned: &mut Vec<String>,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<String>,
    ) -> (Vec<MatchedComponent>, bool) {
        if depth > max_depth || visited.contains(req_item_id) {
            return (Vec::new(), false);
        }

        // Direct match in player inventory
        if let Some(pos) = available_owned.iter().position(|x| x == req_item_id) {
            available_owned.remove(pos);
            let price = RECIPE_META
                .component_prices
                .get(req_item_id)
                .or_else(|| RECIPE_META.total_costs.get(req_item_id))
                .copied()
                .unwrap_or(0)
                .max(0);
            let name = RECIPE_META
                .component_names
                .get(req_item_id)
                .cloned()
                .unwrap_or_else(|| format!("散件#{}", req_item_id));

            return (
                vec![MatchedComponent {
                    item_id: req_item_id.to_string(),
                    name,
                    price,
                    slot: if depth == 1 { "direct".to_string() } else { "nested".to_string() },
                }],
                true,
            );
        }

        // Recurse into sub-components for multi-tier composite recipes
        if let Some(sub_recipe) = RECIPE_META.recipes.get(req_item_id) {
            if !sub_recipe.is_empty() {
                visited.insert(req_item_id.to_string());
                let mut sub_matches = Vec::new();
                let mut all_sub_satisfied = true;
                for sub_id in sub_recipe {
                    let (found, satisfied) = satisfy_component(sub_id, available_owned, depth + 1, max_depth, visited);
                    if !satisfied {
                        all_sub_satisfied = false;
                    }
                    sub_matches.extend(found);
                }
                visited.remove(req_item_id);
                if !sub_matches.is_empty() {
                    return (sub_matches, all_sub_satisfied);
                }
            }
        }

        (Vec::new(), false)
    }

    let recipe = RECIPE_META
        .recipes
        .get(&tid)
        .or_else(|| RECIPE_META.recipes.get(raw_tid));

    let mut all_satisfied = false;
    if let Some(recipe_items) = recipe {
        if !recipe_items.is_empty() {
            all_satisfied = true;
            let mut visited = HashSet::new();
            visited.insert(tid.clone());
            if !raw_tid.is_empty() {
                visited.insert(raw_tid.to_string());
            }
            for req_cid in recipe_items {
                let (matches, satisfied) = satisfy_component(req_cid, &mut available_owned, 1, 4, &mut visited);
                if !satisfied {
                    all_satisfied = false;
                }
                for m in matches {
                    total_deducted += m.price;
                    matched_components.push(m);
                }
            }
        }
    }

    let net_cost = (total_cost - total_deducted).max(0);
    let diff = net_cost - cur_gold_int;
    let is_afford = cur_gold_int >= net_cost && total_cost > 0;

    let status_text = if total_cost == 0 {
        format!("💰 当前{}g [价格未收录]", cur_gold_int)
    } else if !matched_components.is_empty() {
        if is_afford {
            format!(
                "⚡ 散件已抵扣{}g，差价仅需{}g [立即升级]",
                total_deducted, net_cost
            )
        } else {
            format!(
                "⚡ 散件已抵扣{}g，升级还差{}g",
                total_deducted, diff.max(0)
            )
        }
    } else if is_afford {
        format!("💰 可直接购买 ({}g)", net_cost)
    } else {
        format!("💰 还差{}g (总价{}g)", diff.max(0), total_cost)
    };

    let recipe_len = recipe.map(|r| r.len()).unwrap_or(0);
    let has_all = recipe_len > 0 && all_satisfied;

    ItemNetCostResult {
        target_item_id: if !raw_tid.is_empty() { raw_tid.to_string() } else { tid },
        total_cost,
        net_cost,
        deducted_gold: total_deducted,
        matched_components,
        has_all_components: has_all,
        is_affordable: is_afford,
        remaining_diff: diff.max(0),
        status_text,
        calc_mode: gold_calc_mode.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_item_net_cost_no_negative_diff() {
        // Infinity and NaN current_gold handling
        let res_nan = calculate_item_net_cost("3089", &[], f64::NAN, "FULL_COST");
        assert_eq!(res_nan.remaining_diff, res_nan.total_cost);
        assert!(!res_nan.status_text.contains("NaN"));

        let res_inf = calculate_item_net_cost("3089", &[], f64::INFINITY, "FULL_COST");
        assert_eq!(res_inf.remaining_diff, res_inf.total_cost);

        let res_neg = calculate_item_net_cost("3089", &[], -999.0, "NET_DEDUCT");
        assert_eq!(res_neg.remaining_diff, res_neg.net_cost);
        assert!(!res_neg.status_text.contains("-"));

        // Finished item in inventory does not self-deduct
        let res_self = calculate_item_net_cost("3089", &["3089".to_string(), "Item_3089".to_string()], 500.0, "NET_DEDUCT");
        assert_eq!(res_self.deducted_gold, 0);

        // Unknown item ID
        let res_unknown = calculate_item_net_cost("99999999", &[], 1000.0, "NET_DEDUCT");
        assert_eq!(res_unknown.total_cost, 0);
        assert_eq!(res_unknown.net_cost, 0);
        assert_eq!(res_unknown.remaining_diff, 0);
    }
}

