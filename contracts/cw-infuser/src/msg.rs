use crate::contract::CwInfuser;

use crate::state::{Bundle, DefaultInfusionParams, Infusion};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(CwInfuser, CwInfuserExecuteMsg, CwInfuserQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct CwInfuserInstantiateMsg {
    pub default_infusion_params: DefaultInfusionParams,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum CwInfuserExecuteMsg {
    UpdateConfig {},
    /// Increment count by 1
    CreateInfusion {
        collections: Vec<Infusion>,
    },
    Infuse {
        infusion_id: u64,
        bundle: Vec<Bundle>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MyAppMigrateMsg {}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
pub enum CwInfuserQueryMsg {
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
pub struct ConfigResponse {
    pub infusion_params: DefaultInfusionParams,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusionsResponse {
    pub infusions: Vec<Infusion>,
}
