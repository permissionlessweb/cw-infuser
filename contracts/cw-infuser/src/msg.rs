use crate::state::*;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal};
use cw_infusions::{
    bundles::{Bundle, BundleType},
    state::{EligibleNFTCollection, Infusion, InfusionState},
    wavs::{WavsBundle, WavsRecordResponse},
};

#[cw_serde]
pub struct InstantiateMsg {
    /// owner of this contract
    pub contract_owner: Option<String>,
    /// Fee from each infusion payment, if required. Goes to contract owner during any infusion. Set to 0.0 to disable.
    pub owner_fee: Decimal,
    /// Minimum fee that is required for creating an infusion
    pub min_creation_fee: Option<Coin>,
    /// Minimum fee that is required to be set when infusions occur
    pub min_infusion_fee: Option<Coin>,
    /// Minimum tokens required for any infusions eligible collections
    pub min_per_bundle: Option<u64>,
    /// Maximim tokens required for any infusions eligible collections
    pub max_per_bundle: Option<u64>,
    /// Maximum bundles any infusion is able to require
    pub max_bundles: Option<u64>,
    /// Maximum infusions that may be created at once
    pub max_infusions: Option<u64>,
    /// Code-ID of the cw721-collection
    pub cw721_code_id: u64,
    /// Optional key wavs operators make use of, if support is enabled
    pub wavs_public_key: Option<String>,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    CreateInfusion {
        infusions: Vec<Infusion>,
    },
    WavsEntryPoint {
        infusions: Vec<WavsBundle>,
    },
    Infuse {
        id: u64,
        bundle: Vec<Bundle>,
    },
    EndInfusion {
        id: u64,
    },
    UpdateInfusionBaseUri {
        id: u64,
        base_uri: String,
    },
    UpdateConfig {
        config: UpdatingConfig,
    },
    UpdateInfusionsEligibleCollections {
        id: u64,
        to_add: Vec<EligibleNFTCollection>,
        to_remove: Vec<EligibleNFTCollection>,
    },
    UpdateInfusionMintFee {
        id: u64,
        mint_fee: Option<Coin>,
    },
    UpdateInfusionBundleType {
        id: u64,
        bundle_type: BundleType,
    },
    Shuffle {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    /// returns an infusion for a given infusion owner & infusion id.
    #[returns(Infusion)]
    Infusion { addr: Addr, id: u64 },
    /// returns an infusion for a given infusion id.
    #[returns(InfusionState)]
    InfusionById { id: u64 },
    /// returns all infusions owned by a given address
    /// defaults to 30 entries from a given index point of the infusion map.
    /// TODO: optimize pagination
    #[returns(InfusionsResponse)]
    Infusions { addr: Addr, index: u64 },
    #[returns(bool)]
    IsInBundle {
        collection_addr: Addr,
        infusion_id: u64,
    },
    #[returns(Vec<WavsRecordResponse>)]
    WavsRecord {
        nfts: Vec<String>,
        burner: Option<Addr>,
    },
    // #[returns(Vec<WavsEligibleRes>)]
    // WavsEligibleCollection { nfts: Vec<String> },
}

#[cosmwasm_schema::cw_serde]
pub struct CountResponse {
    pub count: i32,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusedCollectionParams {
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub admin: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusionsResponse {
    pub infusions: Vec<InfusionState>,
}

#[cw_serde]
pub struct MigrateMsg {}
