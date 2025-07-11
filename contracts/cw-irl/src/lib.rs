pub mod contract;
mod error;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

pub use crate::error::ContractError;

#[cw_serde]
pub struct InstantiateMsg {
    pub event_ticket_label: String,
    pub event_metadata: EventMetadata,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    MintTickets { data: Vec<MintTicketObject> },
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}

#[cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct CreateEventTickets {
    pub sender: String,
    pub subdenom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MintTicketObject {
    pub amount: u128,
    /// address of ephemeral ticket account
    pub ticket: String,
}
#[cosmwasm_schema::cw_serde]
pub struct OsmosisMintObject {
    pub sender: String,
    pub amount: Coin,
    pub mint_to_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct EventMetadata {
    pub description: String,
    pub denom_units: String,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}
