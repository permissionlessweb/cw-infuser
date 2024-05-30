use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub default_infusion_params: DefaultInfusionParams,
    pub latest_infusion_id: Option<u64>,
}

#[cw_serde]
pub struct Infusion {
    pub collections: Vec<NFTCollection>,
    pub infused_collection: InfusedCollection,
    pub infusion_params: InfusionParams,
    pub infusion_id: u64,
}


#[cw_serde]
pub struct DefaultInfusionParams {
    pub min_required: u64,
    pub code_id: u64,
}
#[cw_serde]
pub struct InfusionParams {
    pub amount_required: u64,
    pub params: BurnParams,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const INFUSION: Map<(Addr, u64), Infusion> = Map::new("infusion");
pub const INFUSION_ID: Map<u64, (Addr, u64)> = Map::new("infusion_id");
pub const INFUSION_INFO: Map<&Addr, InfusionInfo> = Map::new("infusion_info");


#[cw_serde]
pub struct NFT {
    pub addr: Addr,
    pub token_id: u64,
}

#[cw_serde]
pub struct NFTCollection {
    pub addr: Addr,
    pub admin: Option<String>,
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub struct InfusedCollection {
    pub addr: Addr,
    pub admin: Option<String>,
    pub name: String,
    pub symbol: String,
}

#[cw_serde]
pub struct BurnParams {
    pub compatible_traits: Option<CompatibleTraits>,
}

#[cw_serde]
pub struct CompatibleTraits {
    pub a: String,
    pub b: String,
}

#[cw_serde]
pub struct Bundle {
    pub nfts: Vec<NFT>,
}
#[cw_serde]
#[derive(Default)]
pub struct InfusionInfo {
    pub next_id: u64,
}




