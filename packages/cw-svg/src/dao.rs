use crate::compute_template_slots;
use cosmwasm_std::Binary;
use cw721_svg::msg::{InstantiateMsg, VariableDef, VariableKind};

/// Reference SVG with default colors (Black theme) filled in
// pub const YIN_YANG_SVG_DEFAULT: &str = include_str!("../svgs/dao-default.svg");
pub const YIN_YANG_SVG_TEMPLATE: &str = include_str!("../svgs/dao.svg");

pub fn dao_variables() -> Vec<VariableDef> {
    vec![
        VariableDef {
            name: "color_yin".to_string(),
            kind: VariableKind::Options(vec![
                "rgb(64,38,14)".to_string(),    // Brown
                "rgb(34,34,37)".to_string(),    // Black
                "rgb(177,182,180)".to_string(), // Polar
                "rgb(0,0,0)".to_string(),       // Panda
            ]),
        },
        VariableDef {
            name: "color_yang".to_string(),
            kind: VariableKind::Options(vec![
                "rgb(141,93,51)".to_string(),   // Brown
                "rgb(56,56,64)".to_string(),    // Black
                "rgb(208,229,226)".to_string(), // Polar
                "rgb(221,221,222)".to_string(), // Panda
            ]),
        },
    ]
}

pub fn dao_instantiate_msg(
    name: impl Into<String>,
    symbol: impl Into<String>,
    total: u64,
    seed: Binary,
) -> InstantiateMsg {
    InstantiateMsg {
        name: name.into(),
        symbol: symbol.into(),
        svg_template: YIN_YANG_SVG_TEMPLATE.to_string(),
        variables: dao_variables(),
        total,
        seed,
        owner: None,
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![],
        payment_address: None,
        whitelist: None,
        template_slots: compute_template_slots(YIN_YANG_SVG_TEMPLATE, &dao_variables()),
    }
}
