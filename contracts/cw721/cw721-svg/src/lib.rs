pub mod contract;
pub use contract::*;

pub(crate) const CONTRACT_NAME: &str = "cw721_svg";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Contract type alias ───────────────────────────────────────────────────────

pub type InstantiateMsg = cw721::msg::Cw721InstantiateMsg<SvgCollectionMetadata>;
pub type ExecuteMsg =
    cw721::msg::Cw721ExecuteMsg<SvgMetadata, SvgCollectionMetadata, SvgExecuteMsgExt>;
pub type QueryMsg = cw721::msg::Cw721QueryMsg<SvgMetadata, SvgCollectionMetadata, SvgQueryMsgExt>;

pub type Cw721SvgContract<'a> = cw721::extension::Cw721Extensions<
    'a,
    SvgMetadata,           // TNftExtension
    SvgMetadata,           // TNftExtensionMsg
    SvgCollectionMetadata, // TCollectionExtension
    SvgCollectionMetadata, // TCollectionExtensionMsg
    SvgExecuteMsgExt,      // TExtensionMsg
    SvgQueryMsgExt,        // TExtensionQueryMsg
    cosmwasm_std::Empty,   // TCustomResponseMsg
>;

// ── Entry points ──────────────────────────────────────────────────────────────

pub mod entry {
    use super::*;
    use crate::contract::{MigrateMsg, SvgTemplateResponse, SvgTokenUriResponse};
    use crate::contract::{
        MAX_SVG_SIZE, MAX_TOTAL_SUPPLY, SVG_TEMPLATE, TEMPLATE_SLOTS, VARIABLES,
    };
    use cosmwasm_std::{
        to_json_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError,
        StdResult, Timestamp,
    };
    use cw721::msg::Cw721InstantiateMsg;
    use cw721::traits::{Cw721Execute, Cw721Query};

    #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        let mut svg = msg.collection_info_extension.clone();

        if svg.svg_template.len() > MAX_SVG_SIZE {
            return Err(ContractError::SvgTemplateTooLarge {
                max: MAX_SVG_SIZE,
                got: svg.svg_template.len(),
            });
        }

        if svg.total > MAX_TOTAL_SUPPLY {
            return Err(ContractError::TotalSupplyTooHigh {
                max: MAX_TOTAL_SUPPLY,
                got: svg.total,
            });
        }
        let owner = msg
            .creator
            .clone()
            .unwrap_or_else(|| info.sender.to_string());
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&owner))?;

        // Compute mint start time
        let mint_start_time = match svg.mint_start_time {
            Some(secs) => Timestamp::from_seconds(env.block.time.seconds() + secs.seconds()),
            None => env.block.time,
        };
        svg.mint_start_time = Some(mint_start_time);

        // Compute mint end time (cutoff)
        svg.mint_end_time = svg
            .mint_end_time
            .map(|secs| Timestamp::from_seconds(env.block.time.seconds() + secs.seconds()));

        // Resolve payment address (defaults to owner)
        let witdraw_addr = deps
            .api
            .addr_validate(msg.withdraw_address.as_deref().unwrap_or(&owner))?;
        // Validate price tiers: each cutoff must be strictly higher than the previous
        if svg
            .price_tiers
            .windows(2)
            .any(|w| w[1].until_count <= w[0].until_count)
        {
            return Err(ContractError::PricingTierError {});
        };

        // Validate variable definitions
        validate_variables(&svg.variables)?;

        validate_template_slots(&svg.svg_template, &svg.variables, &svg.template_slots)?;
        SVG_TEMPLATE.save(deps.storage, &svg.svg_template)?;
        VARIABLES.save(deps.storage, &svg.variables)?;
        TEMPLATE_SLOTS.save(deps.storage, &svg.template_slots)?;

        Cw721SvgContract::default()
            .instantiate(
                deps.branch(),
                &env,
                &info,
                Cw721InstantiateMsg::<SvgCollectionMetadata> {
                    name: msg.name.clone(),
                    symbol: msg.symbol.clone(),
                    minter: Some(env.contract.address.to_string()),
                    creator: Some(info.sender.to_string()),
                    collection_info_extension: svg,
                    withdraw_address: Some(witdraw_addr.into_string()),
                },
            )
            .map_err(|e| ContractError::Std(StdError::msg(e.to_string())))?;

        Ok(Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("contract", env.contract.address.to_string()))
    }

    #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::UpdateExtension { msg } => match msg {
                SvgExecuteMsgExt::Mint(m) => {
                    execute_mint(deps, env, info, m.amnt, m.proof_hashes, m.alloc)
                }
                SvgExecuteMsgExt::Pause { pause } => execute_pause(deps, info, pause),
                SvgExecuteMsgExt::UpdateWhitelist { address } => {
                    execute_update_whitelist(deps, info, address)
                }
            },
            ExecuteMsg::Mint { .. } => return Err(ContractError::IncorrectEntrypoint),
            _ => Cw721SvgContract::default()
                .execute(deps, &env, &info, msg)
                .map_err(Into::into),
        }
    }

    #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                SvgQueryMsgExt::SvgTokenUri { token_id } => {
                    let svg = query_svg_token_uri(deps, token_id)
                        .map_err(|e| StdError::msg(e.to_string()))?;
                    to_json_binary(&SvgTokenUriResponse { svg })
                }
                SvgQueryMsgExt::SvgPlaceholder { seed } => {
                    let svg = query_svg_placeholder(deps, seed)
                        .map_err(|e| StdError::msg(e.to_string()))?;
                    to_json_binary(&SvgTokenUriResponse { svg })
                }

                SvgQueryMsgExt::SvgTemplate {} => {
                    let template =
                        query_svg_template(deps).map_err(|e| StdError::msg(e.to_string()))?;
                    to_json_binary(&SvgTemplateResponse { template })
                }
                SvgQueryMsgExt::Whitelist {} => {
                    let wl = query_whitelist(deps).map_err(|e| StdError::msg(e.to_string()))?;
                    to_json_binary(&wl)
                }
                SvgQueryMsgExt::Ownership {} => {
                    to_json_binary(&cw_ownable::get_ownership(deps.storage)?)
                }

                SvgQueryMsgExt::MintCount { address } => {
                    let addr = deps.api.addr_validate(&address)?;
                    let count = mint_count(deps, &addr);
                    to_json_binary(&MintCountResponse { address, count })
                }
                SvgQueryMsgExt::WlMintCount { address } => {
                    let addr = deps.api.addr_validate(&address)?;
                    let count = whitelist_mint_count(deps, &addr);
                    to_json_binary(&MintCountResponse { address, count })
                }
                SvgQueryMsgExt::CurrentPriceTier {} => {
                    let c = Cw721SvgContract::default();
                    let mc = c.config.token_count(deps.storage)?;
                    let pt = c
                        .query_collection_info_and_extension(deps)?
                        .extension
                        .price_tiers;

                    to_json_binary(&current_tier_price(&pt, mc))
                }
            },
            other => Cw721SvgContract::default()
                .query(deps, &env, other)
                .map_err(|e| cosmwasm_std::StdError::msg(e.to_string())),
        }
    }

    #[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
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

pub mod state {
    use crate::SvgMetadata;
    use cosmwasm_std::{Deps, Empty, Env, MessageInfo};
    use cw721::{
        error::Cw721ContractError,
        traits::{Contains, Cw721CustomMsg, Cw721State, StateFactory},
    };
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
}
