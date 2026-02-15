use crate::msg::{MintConfig, VariableDef};
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const MAX_SVG_SIZE: usize = 420 * 1024;
pub const MAX_TOTAL_SUPPLY: u64 = 10_000;

pub const SVG_TEMPLATE: Item<String> = Item::new("svg_template");
pub const VARIABLES: Item<Vec<VariableDef>> = Item::new("variables");
pub const MINT_CONFIG: Item<MintConfig> = Item::new("mint_config");
/// Optional merkle whitelist contract. Whitelisted minters bypass mint fees.
pub const WHITELIST: Item<Addr> = Item::new("whitelist");
/// Per-address total mint count (all minters).
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("minter_addrs");
/// Per-address whitelist mint count (only whitelisted minters).
pub const WL_MINTER_ADDRS: Map<&Addr, u32> = Map::new("wl_minter_addrs");
