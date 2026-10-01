use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetadata {
    pub total_costs: HashMap<String, i64>,
    pub recipes: HashMap<String, Vec<String>>,
    pub component_prices: HashMap<String, i64>,
    pub component_names: HashMap<String, String>,
}

lazy_static! {
    pub static ref RECIPE_META: RecipeMetadata = {
        let raw = include_str!("../../data/recipe_metadata.json");
        serde_json::from_str(raw).expect("Failed to parse embedded recipe_metadata.json")
    };
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
    let total_cost = *RECIPE_META
        .total_costs
        .get(&tid)
        .or_else(|| RECIPE_META.total_costs.get(raw_tid))
        .unwrap_or(&3000);

    let cur_gold_int = if current_gold.is_nan() || current_gold.is_infinite() {
        0
    } else {
        (current_gold.max(0.0)) as i64
    };

    if gold_calc_mode == "FULL_COST" {
        let diff = total_cost - cur_gold_int;
        let is_afford = cur_gold_int >= total_cost;
        let status_text = if is_afford {
            format!("💰 {}/{}g [可全款购买]", cur_gold_int, total_cost)
        } else {
            format!("💰 {}/{}g (差{}g)", cur_gold_int, total_cost, diff.max(0))
        };

        return ItemNetCostResult {
            target_item_id: if !raw_tid.is_empty() { raw_tid.to_string() } else { tid },
            total_cost,
            net_cost: total_cost,
            deducted_gold: 0,
            matched_components: Vec::new(),
            has_all_components: false,
            is_affordable: is_afford,
            remaining_diff: diff.max(0),
            status_text,
            calc_mode: "FULL_COST".to_string(),
        };
    }

    // Mode: NET_DEDUCT / NET_REMAINING
    let mut available_owned: Vec<String> = owned_item_ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "0" && s != &tid && s != raw_tid && s != "None" && s != "undefined")
        .collect();

    let mut matched_components = Vec::new();
    let mut total_deducted: i64 = 0;

    let recipe = RECIPE_META
        .recipes
        .get(&tid)
        .or_else(|| RECIPE_META.recipes.get(raw_tid));

    if let Some(recipe_items) = recipe {
        let mut matched_direct_indices = std::collections::HashSet::new();

        // Pass 1: Direct components
        for (idx, req_cid) in recipe_items.iter().enumerate() {
            if let Some(pos) = available_owned.iter().position(|x| x == req_cid) {
                available_owned.remove(pos);
                matched_direct_indices.insert(idx);
                let price = *RECIPE_META.component_prices.get(req_cid).unwrap_or(&800);
                total_deducted += price;
                let name = RECIPE_META
                    .component_names
                    .get(req_cid)
                    .cloned()
                    .unwrap_or_else(|| format!("散件#{}", req_cid));

                matched_components.push(MatchedComponent {
                    item_id: req_cid.clone(),
                    name,
                    price,
                    slot: "direct".to_string(),
                });
            }
        }

        // Pass 2: Nested sub-components for unmatched slots
        for (idx, req_cid) in recipe_items.iter().enumerate() {
            if matched_direct_indices.contains(&idx) {
                continue;
            }
            if let Some(sub_recipe) = RECIPE_META.recipes.get(req_cid) {
                for sub_cid in sub_recipe {
                    if let Some(pos) = available_owned.iter().position(|x| x == sub_cid) {
                        available_owned.remove(pos);
                        let price = *RECIPE_META.component_prices.get(sub_cid).unwrap_or(&400);
                        total_deducted += price;
                        let name = RECIPE_META
                            .component_names
                            .get(sub_cid)
                            .cloned()
                            .unwrap_or_else(|| format!("小件#{}", sub_cid));

                        matched_components.push(MatchedComponent {
                            item_id: sub_cid.clone(),
                            name,
                            price,
                            slot: "nested".to_string(),
                        });
                    }
                }
            }
        }
    }

    let net_cost = (total_cost - total_deducted).max(0);
    let diff = net_cost - cur_gold_int;
    let is_afford = cur_gold_int >= net_cost;

    let status_text = if !matched_components.is_empty() {
        if is_afford {
            format!(
                "⚡ 散件已抵扣{}g，差价仅需{}g [立即升级]",
                total_deducted, net_cost
            )
        } else {
            format!(
                "⚡ 散件已抵扣{}g，升级还差{}g",
                total_deducted, diff
            )
        }
    } else if is_afford {
        format!("💰 可直接购买 ({}g)", net_cost)
    } else {
        format!("💰 还差{}g (总价{}g)", diff, total_cost)
    };

    let recipe_len = recipe.map(|r| r.len()).unwrap_or(0);
    let has_all = recipe_len > 0 && matched_components.len() == recipe_len;

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
