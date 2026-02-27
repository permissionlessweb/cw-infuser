pub mod commands;
mod error;
pub mod msg;
pub mod state;

#[cfg(feature = "interface")]
pub mod interface;

pub use crate::error::ContractError;
use crate::msg::SvgMetadata;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{Deps, DepsMut, Empty, Env, MessageInfo};
use cw721::{
    error::Cw721ContractError,
    traits::{Contains, Cw721CustomMsg, Cw721State, StateFactory},
    EmptyOptionalCollectionExtension, EmptyOptionalCollectionExtensionMsg,
};

const CONTRACT_NAME: &str = "crates.io:cw721-svg";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Trait impls required by Cw721Extensions ──────────────────────────────────

impl Cw721State for SvgMetadata {}
impl Cw721CustomMsg for SvgMetadata {}

impl Contains for SvgMetadata {
    fn contains(&self, _other: &Self) -> bool {
        true
    }
}

impl StateFactory<SvgMetadata> for SvgMetadata {
    fn create(
        &self,
        _deps: Deps,
        _env: &Env,
        _info: Option<&MessageInfo>,
        _current: Option<&SvgMetadata>,
    ) -> Result<SvgMetadata, Cw721ContractError> {
        Ok(self.clone())
    }
    fn validate(
        &self,
        _deps: Deps,
        _env: &Env,
        _info: Option<&MessageInfo>,
        _current: Option<&SvgMetadata>,
    ) -> Result<(), Cw721ContractError> {
        Ok(())
    }
}

// ── Contract type alias ───────────────────────────────────────────────────────

pub type Cw721SvgContract<'a> = cw721::extension::Cw721Extensions<
    'a,
    SvgMetadata,                         // TNftExtension
    SvgMetadata,                         // TNftExtensionMsg
    EmptyOptionalCollectionExtension,    // TCollectionExtension
    EmptyOptionalCollectionExtensionMsg, // TCollectionExtensionMsg
    Empty,                               // TExtensionMsg
    Empty,                               // TExtensionQueryMsg
    Empty,                               // TCustomResponseMsg
>;

// ── Entry points ──────────────────────────────────────────────────────────────

pub mod entry {
    use super::*;
    use crate::commands::*;
    use crate::msg::{
        ConfigResponse, MigrateMsg, MintConfig, SvgTemplateResponse, SvgTokenUriResponse,
    };
    use crate::state::{
        MAX_SVG_SIZE, MAX_TOTAL_SUPPLY, MINT_CONFIG, SVG_TEMPLATE, TEMPLATE_SLOTS, VARIABLES,
        WHITELIST,
    };
    use cosmwasm_std::{
        entry_point, to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response,
        StdError, StdResult, Timestamp,
    };
    use cw721::msg::{
        Cw721ExecuteMsg as BaseExecuteMsg, Cw721InstantiateMsg, Cw721QueryMsg as BaseQueryMsg,
    };
    use cw721::traits::{Cw721Execute, Cw721Query};

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: crate::msg::InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        if msg.svg_template.len() > MAX_SVG_SIZE {
            return Err(ContractError::SvgTemplateTooLarge {
                max: MAX_SVG_SIZE,
                got: msg.svg_template.len(),
            });
        }

        if msg.total > MAX_TOTAL_SUPPLY {
            return Err(ContractError::TotalSupplyTooHigh {
                max: MAX_TOTAL_SUPPLY,
                got: msg.total,
            });
        }

        Cw721SvgContract::default()
            .instantiate(
                deps.branch(),
                &env,
                &info,
                Cw721InstantiateMsg::<EmptyOptionalCollectionExtensionMsg> {
                    name: msg.name.clone(),
                    symbol: msg.symbol.clone(),
                    minter: Some(env.contract.address.to_string()),
                    creator: Some(info.sender.to_string()),
                    collection_info_extension: None,
                    withdraw_address: None,
                },
            )
            .map_err(|e| ContractError::Std(StdError::generic_err(e.to_string())))?;

        // Compute mint start time
        let mint_start_time = match msg.mint_start_time {
            Some(secs) => Timestamp::from_seconds(env.block.time.seconds() + secs),
            None => env.block.time,
        };

        // Compute mint end time (cutoff)
        let mint_end_time = msg
            .mint_end_time
            .map(|secs| Timestamp::from_seconds(env.block.time.seconds() + secs));

        let owner = msg.owner.unwrap_or_else(|| info.sender.to_string());

        // Resolve payment address (defaults to owner)
        let payment_address = deps
            .api
            .addr_validate(msg.payment_address.as_deref().unwrap_or(&owner))?;

        // Store mint config
        let config = MintConfig {
            seed: msg.seed,
            mint_count: 0,
            total: msg.total,
            paused: false,
            mint_start_time,
            mint_end_time,
            price_tiers: msg.price_tiers,
            payment_address,
        };
        MINT_CONFIG.save(deps.storage, &config)?;

        // Store whitelist if provided
        if let Some(wl) = msg.whitelist {
            let wl_addr = deps.api.addr_validate(&wl)?;
            WHITELIST.save(deps.storage, &wl_addr)?;
        }

        // Validate variable definitions
        validate_variables(&msg.variables)?;

        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

        validate_template_slots(&msg.svg_template, &msg.variables, &msg.template_slots)?;
        SVG_TEMPLATE.save(deps.storage, &msg.svg_template)?;
        VARIABLES.save(deps.storage, &msg.variables)?;
        TEMPLATE_SLOTS.save(deps.storage, &msg.template_slots)?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("contract", env.contract.address.to_string()))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Mint {
                amount,
                proof_hashes,
                allocation,
            } => execute_mint(deps, env, info, amount, proof_hashes, allocation),
            ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
            ExecuteMsg::UpdateWhitelist { address } => {
                execute_update_whitelist(deps, info, address)
            }
            ExecuteMsg::UpdateOwnership(action) => {
                let res = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
                Ok(Response::new().add_attributes(res.into_attributes()))
            }
            // Delegate standard cw721 messages to base
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::TransferNft {
                        recipient,
                        token_id,
                    },
                )
                .map_err(ContractError::Base),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg: send_msg,
            } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::SendNft {
                        contract,
                        token_id,
                        msg: send_msg,
                    },
                )
                .map_err(ContractError::Base),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::Approve {
                        spender,
                        token_id,
                        expires,
                    },
                )
                .map_err(ContractError::Base),
            ExecuteMsg::Revoke { spender, token_id } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::Revoke {
                        spender,
                        token_id,
                    },
                )
                .map_err(ContractError::Base),
            ExecuteMsg::ApproveAll { operator, expires } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::ApproveAll {
                        operator,
                        expires,
                    },
                )
                .map_err(ContractError::Base),
            ExecuteMsg::RevokeAll { operator } => Cw721SvgContract::default()
                .execute(
                    deps,
                    &env,
                    &info,
                    BaseExecuteMsg::<SvgMetadata, EmptyOptionalCollectionExtensionMsg, Empty>::RevokeAll {
                        operator,
                    },
                )
                .map_err(ContractError::Base),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::SvgTokenUri { token_id } => {
                let svg = query_svg_token_uri(deps, token_id)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTokenUriResponse { svg })
            }
            QueryMsg::SvgPlaceholder { seed } => {
                let svg = query_svg_placeholder(deps, seed)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTokenUriResponse { svg })
            }
            QueryMsg::Config {} => {
                let config = query_config(deps)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                to_json_binary(&ConfigResponse { config })
            }
            QueryMsg::SvgTemplate {} => {
                let template = query_svg_template(deps)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTemplateResponse { template })
            }
            QueryMsg::Whitelist {} => {
                let wl = query_whitelist(deps)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                to_json_binary(&wl)
            }
            QueryMsg::Minter {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
            QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
            // Delegate standard cw721 queries to base
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::OwnerOf {
                        token_id,
                        include_expired,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::Approval {
                        token_id,
                        spender,
                        include_expired,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::Approvals {
                        token_id,
                        include_expired,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::AllOperators {
                        owner,
                        include_expired,
                        start_after,
                        limit,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::NumTokens {} => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::NumTokens {},
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::ContractInfo {} => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::GetCollectionInfoAndExtension {},
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::NftInfo { token_id } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::NftInfo {
                        token_id,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::AllNftInfo {
                        token_id,
                        include_expired,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::Tokens {
                        owner,
                        start_after,
                        limit,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::AllTokens { start_after, limit } => Cw721SvgContract::default()
                .query(
                    deps,
                    &env,
                    BaseQueryMsg::<SvgMetadata, EmptyOptionalCollectionExtension, Empty>::AllTokens {
                        start_after,
                        limit,
                    },
                )
                .map_err(|e| StdError::generic_err(e.to_string())),
            QueryMsg::MintCount { address } => {
                let addr = deps.api.addr_validate(&address)?;
                let count = mint_count(deps, &addr);
                to_json_binary(&crate::msg::MintCountResponse { address, count })
            }
            QueryMsg::WlMintCount { address } => {
                let addr = deps.api.addr_validate(&address)?;
                let count = whitelist_mint_count(deps, &addr);
                to_json_binary(&crate::msg::MintCountResponse { address, count })
            }
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
        let prev_version = cw2::get_contract_version(deps.storage)?;
        let res = Response::new();
        let event = Event::new("migrate")
            .add_attribute("from_name", prev_version.contract)
            .add_attribute("from_version", prev_version.version)
            .add_attribute("to_name", CONTRACT_NAME)
            .add_attribute("to_version", CONTRACT_VERSION);
        Ok(res.add_event(event))
    }
}
