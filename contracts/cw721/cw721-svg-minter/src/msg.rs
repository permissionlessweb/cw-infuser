use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_ownable::cw_ownable_execute;
use cw721_svg::msg::InstantiateMsg as SvgInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    /// Optional owner. If set, only the owner may create collections.
    pub owner: Option<String>,
    /// Code ID of the cw721-svg contract to instantiate.
    pub svg_code_id: u64,
}

#[cw_ownable_execute]
#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Create a new cw721-svg collection.
    /// Uses instantiate2 for deterministic, anti-collision address prediction.
    CreateSvgCollection {
        instantiate_msg: SvgInstantiateMsg,
        label: String,
    },

    /// Owner-only: update the cw721-svg code ID used for new collections.
    UpdateCodeId { code_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Returns a paginated list of all svg collections.
    #[returns(Vec<crate::state::SvgCollection>)]
    ListSvgCollections {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Returns a paginated list of all svg collections in reverse.
    #[returns(Vec<crate::state::SvgCollection>)]
    ListSvgCollectionsReverse {
        start_before: Option<String>,
        limit: Option<u32>,
    },

    /// Returns a paginated list of svg collections created by a specific address.
    #[returns(Vec<crate::state::SvgCollection>)]
    ListSvgCollectionsByCreator {
        creator: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Returns a paginated list of svg collections by creator in reverse.
    #[returns(Vec<crate::state::SvgCollection>)]
    ListSvgCollectionsByCreatorReverse {
        creator: String,
        start_before: Option<String>,
        limit: Option<u32>,
    },

    /// Returns ownership info.
    #[returns(::cw_ownable::Ownership<::cosmwasm_std::Addr>)]
    Ownership {},

    /// Returns the cw721-svg code ID currently used to instantiate collections.
    #[returns(::std::primitive::u64)]
    CodeId {},
}
