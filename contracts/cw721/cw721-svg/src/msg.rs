use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, Timestamp, Uint128};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Expiration,
    NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use cw_ownable::Ownership;

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

#[cw_serde]
#[derive(Default)]
pub struct SvgMetadata {
    pub params: Vec<TokenParam>,
}

/// A price tier that applies until the mint count reaches `until_count`.
/// Tiers must be sorted ascending by `until_count`.
/// Example: [{ until_count: 100, price: 1_000_000utoken }, { until_count: 500, price: 5_000_000utoken }]
/// means: first 100 mints cost 1 token, mints 101-500 cost 5 tokens.
#[cw_serde]
pub struct PriceTier {
    /// This tier applies while mint_count < until_count
    pub until_count: u64,
    /// Price per token in this tier
    pub price: Coin,
}

#[cw_serde]
pub struct MintConfig {
    pub seed: Binary,
    pub mint_count: u64,
    pub total: u64,
    pub paused: bool,
    pub mint_start_time: Timestamp,
    pub mint_end_time: Option<Timestamp>,
    /// Ordered price tiers. Empty = free mint.
    pub price_tiers: Vec<PriceTier>,
    /// Address that receives mint payments. Defaults to owner.
    pub payment_address: Addr,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub svg_template: String,
    pub variables: Vec<VariableDef>,
    pub total: u64,
    pub seed: Binary,
    pub owner: Option<String>,
    pub mint_start_time: Option<u64>,
    pub mint_end_time: Option<u64>,
    /// Price tiers for minting. Empty or None = free mint.
    pub price_tiers: Vec<PriceTier>,
    /// Address to receive mint payments. Defaults to owner/sender.
    pub payment_address: Option<String>,
    /// Optional merkle whitelist contract address. Whitelisted minters bypass fees.
    pub whitelist: Option<String>,
    /// Pre-computed placeholder positions in the SVG template.
    /// Each slot maps a `${varname}` occurrence to its byte offsets and variable index.
    pub template_slots: Vec<TemplateSlot>,
}

/// Query message for the merkle whitelist contract.
/// Must match the whitelist contract's QueryMsg::HasMember variant.
#[cw_serde]
pub enum WhitelistHasMemberMsg {
    HasMember {
        member: String,
        proof_hashes: Vec<String>,
    },
}

/// Response from the whitelist HasMember query
#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    Mint {
        amount: u64,
        /// Merkle proof hashes for whitelist verification (bypasses mint fees)
        proof_hashes: Option<Vec<String>>,
        /// Per-address mint allocation encoded in the merkle leaf.
        /// When set, the leaf is hash(sender || allocation) and this value
        /// caps how many tokens the address can mint via whitelist.
        /// When None, falls back to the whitelist contract's per_address_limit.
        allocation: Option<u32>,
    },
    Pause {
        pause: bool,
    },
    /// Owner-only: set or clear the merkle whitelist contract address
    UpdateWhitelist {
        /// Set to Some(addr) to enable whitelist, None to disable
        address: Option<String>,
    },
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
    UpdateOwnership(cw_ownable::Action),
}

#[cw_serde]
pub struct SvgTokenUriResponse {
    pub svg: String,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: MintConfig,
}

#[cw_serde]
pub struct SvgTemplateResponse {
    pub template: String,
}

#[cw_serde]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}

#[cw_ownable::cw_ownable_query]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))] 
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<SvgMetadata>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<SvgMetadata>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Ownership<Addr>)]
    Minter {},
    #[returns(SvgTokenUriResponse)]
    SvgTokenUri { token_id: String },
    /// Returns a preview SVG with random placeholder values filled in.
    #[returns(SvgTokenUriResponse)]
    SvgPlaceholder { seed: Option<String> },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(SvgTemplateResponse)]
    SvgTemplate {},
    #[returns(Option<Addr>)]
    Whitelist {},
    /// Returns the total mint count for a given address.
    #[returns(MintCountResponse)]
    MintCount { address: String },
    /// Returns the whitelist mint count for a given address.
    #[returns(MintCountResponse)]
    WlMintCount { address: String },
}
