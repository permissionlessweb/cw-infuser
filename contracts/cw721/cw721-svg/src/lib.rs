pub mod commands;
mod error;
pub mod msg;
pub mod state;

#[cfg(feature = "interface")]
pub mod interface;

pub use crate::error::ContractError;
use crate::msg::SvgMetadata;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::Empty;

const CONTRACT_NAME: &str = "crates.io:cw721-svg";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw721SvgContract<'a> = cw721_base::Cw721Contract<'a, SvgMetadata, Empty, Empty, Empty>;

pub mod entry {
    use super::*;
    use crate::commands::*;
    use crate::msg::{
        ConfigResponse, MintConfig, SvgTemplateResponse, SvgTokenUriResponse, VariableKind,
    };
    use crate::state::{
        MAX_SVG_SIZE, MAX_TOTAL_SUPPLY, MINT_CONFIG, SVG_TEMPLATE, TEMPLATE_SLOTS, VARIABLES,
        WHITELIST,
    };
    use cosmwasm_std::{
        entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
        Timestamp,
    };
    use cw721_base::msg::InstantiateMsg as Cw721InstantiateMsg;

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

        Cw721SvgContract::default().instantiate(
            deps.branch(),
            env.clone(),
            info.clone(),
            Cw721InstantiateMsg {
                name: msg.name,
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            },
        )?;

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
        for var in &msg.variables {
            match &var.kind {
                VariableKind::Options(opts) => {
                    if opts.is_empty() {
                        return Err(ContractError::InvalidVariableDef {
                            reason: format!(
                                "variable '{}': options list must not be empty",
                                var.name
                            ),
                        });
                    }
                }
                VariableKind::Range {
                    min,
                    max,
                    precision,
                } => {
                    if *precision > 18 {
                        return Err(ContractError::InvalidVariableDef {
                            reason: format!(
                                "variable '{}': precision {} exceeds maximum of 18",
                                var.name, precision
                            ),
                        });
                    }
                    let min_scaled = parse_decimal_scaled(min, *precision).map_err(|e| {
                        ContractError::InvalidVariableDef {
                            reason: format!("variable '{}' min: {}", var.name, e),
                        }
                    })?;
                    let max_scaled = parse_decimal_scaled(max, *precision).map_err(|e| {
                        ContractError::InvalidVariableDef {
                            reason: format!("variable '{}' max: {}", var.name, e),
                        }
                    })?;
                    if min_scaled > max_scaled {
                        return Err(ContractError::InvalidVariableDef {
                            reason: format!(
                                "variable '{}': min ({}) must be <= max ({})",
                                var.name, min, max
                            ),
                        });
                    }
                }
            }
        }

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
            } => Ok(Cw721SvgContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::TransferNft {
                    recipient,
                    token_id,
                },
            )?),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg: send_msg,
            } => Ok(Cw721SvgContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::SendNft {
                    contract,
                    token_id,
                    msg: send_msg,
                },
            )?),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Ok(Cw721SvgContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::Approve {
                    spender,
                    token_id,
                    expires,
                },
            )?),
            ExecuteMsg::Revoke { spender, token_id } => Ok(Cw721SvgContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::Revoke { spender, token_id },
            )?),
            ExecuteMsg::ApproveAll { operator, expires } => Ok(Cw721SvgContract::default()
                .execute(
                    deps,
                    env,
                    info,
                    cw721_base::ExecuteMsg::ApproveAll { operator, expires },
                )?),
            ExecuteMsg::RevokeAll { operator } => Ok(Cw721SvgContract::default().execute(
                deps,
                env,
                info,
                cw721_base::ExecuteMsg::RevokeAll { operator },
            )?),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::SvgTokenUri { token_id } => {
                let svg = query_svg_token_uri(deps, token_id)
                    .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTokenUriResponse { svg })
            }
            QueryMsg::SvgPlaceholder { seed } => {
                let svg = query_svg_placeholder(deps, seed)
                    .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTokenUriResponse { svg })
            }
            QueryMsg::Config {} => {
                let config = query_config(deps)
                    .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                to_json_binary(&ConfigResponse { config })
            }
            QueryMsg::SvgTemplate {} => {
                let template = query_svg_template(deps)
                    .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                to_json_binary(&SvgTemplateResponse { template })
            }
            QueryMsg::Whitelist {} => {
                let wl = query_whitelist(deps)
                    .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
                to_json_binary(&wl)
            }
            QueryMsg::Minter {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
            QueryMsg::Ownership {} => to_json_binary(&cw_ownable::get_ownership(deps.storage)?),
            // Delegate standard cw721 queries to base
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::OwnerOf {
                    token_id,
                    include_expired,
                },
            ),
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::Approval {
                    token_id,
                    spender,
                    include_expired,
                },
            ),
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::Approvals {
                    token_id,
                    include_expired,
                },
            ),
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::AllOperators {
                    owner,
                    include_expired,
                    start_after,
                    limit,
                },
            ),
            QueryMsg::NumTokens {} => {
                Cw721SvgContract::default().query(deps, env, cw721_base::QueryMsg::NumTokens {})
            }
            QueryMsg::ContractInfo {} => {
                Cw721SvgContract::default().query(deps, env, cw721_base::QueryMsg::ContractInfo {})
            }
            QueryMsg::NftInfo { token_id } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::NftInfo { token_id },
            ),
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::AllNftInfo {
                    token_id,
                    include_expired,
                },
            ),
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::Tokens {
                    owner,
                    start_after,
                    limit,
                },
            ),
            QueryMsg::AllTokens { start_after, limit } => Cw721SvgContract::default().query(
                deps,
                env,
                cw721_base::QueryMsg::AllTokens { start_after, limit },
            ),
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
}
