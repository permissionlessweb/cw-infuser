use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Bundle, DefaultInfusionParams, Infusion};

#[cw_serde]
pub struct InstantiateMsg {
    pub default_infusion_params: DefaultInfusionParams,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateInfusion {
        collections: Vec<Infusion>,
    },
    Infuse {
        infusion_id: u64,
        bundle: Vec<Bundle>,
    },
    UpdateConfig {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Infusion)]
    Infusion { addr: Addr, id: u64 },
    #[returns(Infusion)]
    InfusionById { id: u64 },
    #[returns(InfusionsResponse)]
    Infusions { addr: Addr },
    #[returns(bool)]
    IsInBundle { collection_addr: Addr },
}

#[cw_serde]
pub struct InfusedCollectionParams {
    pub code_id: u64,
    pub name: String,
    pub symbol: String,
    pub admin: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub infusion_params: DefaultInfusionParams,
}
#[cw_serde]
pub struct InfusionsResponse {
    pub infusions: Vec<Infusion>,
}
