pub mod dao;

// Re-export contract types for convenience
pub use cw721_svg::msg::{
    ConfigResponse, ExecuteMsg, HasMemberResponse, InstantiateMsg, MintConfig, MintCountResponse,
    PriceTier, QueryMsg, SvgMetadata, SvgTemplateResponse, SvgTokenUriResponse, TemplateSlot,
    TokenParam, VariableDef, VariableKind, WhitelistHasMemberMsg,
};

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
