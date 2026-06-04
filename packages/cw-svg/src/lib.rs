use cosmwasm_schema::cw_serde;

pub mod callback;
pub mod dao;
pub use callback::{MintMsg, SvgMintCallbackAction};

#[cosmwasm_schema::cw_serde]
pub struct RgbRange {
    pub r_min: u8,
    pub r_max: u8,
    pub g_min: u8,
    pub g_max: u8,
    pub b_min: u8,
    pub b_max: u8,
}

#[cw_serde]
pub enum VariableKind {
    /// Pick from a list of string values
    Options(Vec<String>),
    /// Generate a decimal string in [min, max] at given precision.
    /// `min` and `max` are decimal strings (e.g. "-2.5", "80").
    Range {
        min: String,
        max: String,
        precision: u32,
    },
    /// Generate a random `rgb(R,G,B)` color string.
    /// Each channel is independently random in 0–255.
    Rgb,
    /// Pick a random range from the list, then generate a random shade within it.
    /// Each channel is constrained to [min, max] of the selected range.
    RgbStyled(Vec<RgbRange>),
}

#[cw_serde]
pub struct VariableDef {
    pub name: String,
    pub kind: VariableKind,
}

#[cw_serde]
pub struct TokenParam {
    pub name: String,
    pub value: String,
}

#[cw_serde]
pub struct TemplateSlot {
    pub start: u32,
    pub end: u32,
    pub var_idx: u16,
}

/// Compute template slots by scanning a template for `${varname}` placeholders.
/// This is the off-chain counterpart to the on-chain `validate_template_slots`.
pub fn compute_template_slots(template: &str, variables: &[VariableDef]) -> Vec<TemplateSlot> {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut slots = Vec::new();
    let mut i = 0;

    while i < len.saturating_sub(1) {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i;
            let name_start = i + 2;
            let mut j = name_start;
            while j < len && bytes[j] != b'}' {
                j += 1;
            }
            if j >= len {
                panic!("unclosed placeholder at byte offset {}", start);
            }
            let name = &template[name_start..j];
            let end = j + 1;
            let var_idx = variables
                .iter()
                .position(|v| v.name == name)
                .unwrap_or_else(|| panic!("undefined variable '{}' in template", name));
            slots.push(TemplateSlot {
                start: start as u32,
                end: end as u32,
                var_idx: var_idx as u16,
            });
            i = end;
        } else {
            i += 1;
        }
    }

    slots
}
