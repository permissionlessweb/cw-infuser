use cosmwasm_std::{Addr, Coin, HexBinary, Timestamp};
use cw_storage_plus::{Item, Map};
use sg721::RoyaltyInfoResponse;

pub const CONFIG: Item<Config> = Item::new("config");
pub const COUNT: Item<i32> = Item::new("count");
/// infusions saved to map with key of (infused_collection_addr, infusion_id )
pub const INFUSION: Map<(Addr, u64), InfusionState> = Map::new("infusion");
/// Map of the infusion id with the infused collection addr
pub const INFUSION_ID: Map<u64, (Addr, u64)> = Map::new("infusion_id");
/// New infused collection info
pub const INFUSION_INFO: Map<&Addr, InfusionInfo> = Map::new("infusion_info");
// map of index position and token id
pub const MINTABLE_TOKEN_POSITIONS: Map<u32, u32> = Map::new("mt");
pub const MINTABLE_NUM_TOKENS: Map<String, u32> = Map::new("mnt");

#[cosmwasm_schema::cw_serde]
pub struct Config {
    // Default at 0.
    pub latest_infusion_id: u64,
    pub admin: Addr,
    /// % fee from any infusion fee set to go to admin. 10 == 10% , 71 == 71%
    pub admin_fee: u64,
    /// Minimum fee that is required for creating an infusion
    pub min_creation_fee: Option<Coin>,
    /// Minimum fee that is required to be set when new infusions are being created
    pub min_infusion_fee: Option<Coin>,
    /// maximum unique infusion that can be created at once. Defaults to 2
    pub max_infusions: u64,
    /// contract global minimum nft each collection in infusion must require to burn. hard coded to 1
    pub min_per_bundle: u64,
    /// maximum nfts bundles can require
    pub max_per_bundle: u64,
    /// maximum bundles allowed per infusion
    pub max_bundles: u64,
    /// cw721-base code_id
    pub code_id: u64,
    pub code_hash: HexBinary,
}

#[cosmwasm_schema::cw_serde]
pub struct Infusion {
    /// NFT collections eligible for a specific infusion
    pub collections: Vec<NFTCollection>,
    /// Current data of the new infused collection
    pub infused_collection: InfusedCollection,
    /// Parameters of a specific infusion
    pub infusion_params: InfusionParams,
    /// Recipient of payments for an infusion
    pub payment_recipient: Option<Addr>,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusionState {
    /// NFT collections eligible for a specific infusion
    pub collections: Vec<NFTCollection>,
    /// Current data of the new infused collection
    pub infused_collection: InfusedCollection,
    /// Parameters of a specific infusion
    pub infusion_params: InfusionParamState,
    /// Recipient of payments for an infusion
    pub payment_recipient: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusionParams {
    // /// Minimum amount each collection in any infusion is required
    // pub min_per_bundle: Option<u64>,
    /// Minium amount of mint fee required for any infusion if set. Rewards will go to either infusion creator, or reward granted
    pub mint_fee: Option<Coin>,
    pub params: Option<BurnParams>,
}
#[cosmwasm_schema::cw_serde]
pub struct InfusionParamState {
    /// Minimum amount each collection in any infusion is required
    // pub min_per_bundle: u64,
    /// Minium amount of mint fee required for any infusion if set. Rewards will go to either infusion creator, or reward granted
    pub mint_fee: Option<Coin>,
    pub params: Option<BurnParams>,
}

#[cosmwasm_schema::cw_serde]
pub struct Bundle {
    pub nfts: Vec<NFT>,
}

#[cosmwasm_schema::cw_serde]
pub struct NFT {
    pub addr: Addr,
    pub token_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct NFTCollection {
    /// Contract address of collection
    pub addr: Addr,
    /// Minimum tokens required to infuse
    pub min_req: u64,
    /// Optional, maximum tokens able to be infused.
    ///  If not set, contract expects exact # of min_req per collection in bundle.
    pub max_req: Option<u64>,
    /// If set, infuser can send exact amount of tokens to consider eligibility.
    pub  payment_substitute: Option<Coin>
}

impl PartialEq<String> for NFTCollection {
    fn eq(&self, other: &String) -> bool {
        self.addr.to_string().eq(other)
    }
}

#[cosmwasm_schema::cw_serde]
pub struct InfusedCollection {
    pub sg: bool,
    pub addr: Option<String>,
    pub admin: Option<String>,
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub num_tokens: u32,
    pub royalty_info: Option<RoyaltyInfoResponse>,
    pub start_trading_time: Option<Timestamp>,
    pub explicit_content: Option<bool>,
    pub external_link: Option<String>,
}

#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct InfusionInfo {
    pub next_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct BurnParams {
    pub compatible_traits: Option<CompatibleTraits>,
}

#[cosmwasm_schema::cw_serde]
pub struct CompatibleTraits {
    pub a: String,
    pub b: String,
}

pub struct TokenPositionMapping {
    pub position: u32,
    pub token_id: u32,
}
