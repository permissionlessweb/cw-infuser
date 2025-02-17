use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfusionsResponse, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Bundle, Config, InfusedCollection, Infusion, InfusionParamState, InfusionState, NFTCollection,
    TokenPositionMapping, CONFIG, INFUSION, INFUSION_ID, INFUSION_INFO, MINTABLE_NUM_TOKENS,
    MINTABLE_TOKEN_POSITIONS, NFT,
};
use cosmwasm_schema::serde::Serialize;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, instantiate2_address, to_json_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Deps, DepsMut, Empty, Env, Event, HexBinary, MessageInfo, Order, QueryRequest, Reply, Response,
    StdError, StdResult, Storage, SubMsg, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
use cw721_base::{ExecuteMsg as Cw721ExecuteMessage, InstantiateMsg as Cw721InstantiateMsg};
use cw_controllers::AdminError;
use rand_core::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use semver::Version;
use sg721::{CollectionInfo, InstantiateMsg as Sg721InitMsg, RoyaltyInfoResponse};
use sha2::{Digest, Sha256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};
use url::Url;

const INFUSION_COLLECTION_INIT_MSG_ID: u64 = 21;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-infuser";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.max_infusions.is_some_and(|f| f > 2u64) {
        return Err(ContractError::MaxInfusionErrror);
    }
    if msg.max_bundles.is_some_and(|f| f > 5u64) {
        return Err(ContractError::MaxBundleError);
    }
    if msg.min_per_bundle.is_some_and(|f| f < 1u64) {
        return Err(ContractError::MaxInfusionErrror);
    }
    if msg.max_per_bundle.is_some_and(|f| f > 10u64) {
        return Err(ContractError::MaxInfusionErrror);
    }
    if !(msg.admin_fee <= 100u64) {
        return Err(ContractError::Std(StdError::generic_err(
            "admin fee incorrect. Must be less than or 100%",
        )));
    }
    if msg
        .min_creation_fee
        .clone()
        .is_some_and(|f| f.amount.u128() == 0u128)
    {
        return Err(ContractError::Std(StdError::generic_err(
            "admin fee incorrect. Must be less than 100%",
        )));
    }

    // let hardcode_fee = coin(350, "ustars");
    // if info.funds.iter().find(|&e| e == &hardcode_fee).is_some() {
    //     let base_fee: CosmosMsg<Empty> = CosmosMsg::Bank(BankMsg::Send {
    //         to_address: "stars1ampqmqrmuc03d7828qqw296q9ygnt5quf778hv".into(),
    //         amount: vec![hardcode_fee],
    //     });
    //     fee_msg.push(base_fee);
    // } else {
    //     return Err(ContractError::RequirednfusionFeeError);
    // }

    // get checksum of cw721
    let cw721_checksum = deps.querier.query_wasm_code_info(msg.cw721_code_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            min_per_bundle: msg.min_per_bundle.unwrap_or(1),
            max_per_bundle: msg.max_per_bundle.unwrap_or(10u64),
            code_id: msg.cw721_code_id,
            code_hash: cw721_checksum.checksum,
            latest_infusion_id: 0,
            admin: info.sender,
            max_infusions: msg.max_infusions.unwrap_or(2u64),
            max_bundles: msg.max_bundles.unwrap_or(5),
            admin_fee: msg.admin_fee,
            min_creation_fee: msg.min_creation_fee,
            min_infusion_fee: msg.min_infusion_fee,
        },
    )?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateInfusion { infusions } => {
            execute_create_infusion(deps, info.clone(), env, infusions)
        }
        ExecuteMsg::Infuse {
            infusion_id,
            bundle,
        } => execute_infuse_bundle(deps, env, info, infusion_id, bundle),
        ExecuteMsg::UpdateConfig {} => update_config(deps, info),
        ExecuteMsg::EndInfusion { id } => execute_end_infusion(deps, info, id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Infusion { addr, id } => to_json_binary(&query_infusion(deps, addr, id)?),
        QueryMsg::InfusionById { id } => to_json_binary(&query_infusion_by_id(deps, id)?),
        QueryMsg::Infusions { addr, index } => to_json_binary(&query_infusions(deps, addr, index)?),
        QueryMsg::IsInBundle {
            collection_addr,
            infusion_id,
        } => to_json_binary(&query_if_is_in_bundle(deps, collection_addr, infusion_id)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INFUSION_COLLECTION_INIT_MSG_ID => Ok(Response::new()),
        _ => Err(ContractError::Unauthorized {}),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Only the admin should be able to call this
    if config.admin != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }

    // todo: update configs

    Ok(Response::new())
}

pub fn execute_end_infusion(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let key = INFUSION_ID.load(deps.storage, id)?;
    let mut infusion = INFUSION.load(deps.storage, key.clone())?;
    if infusion.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if !infusion.enabled {
        return Err(ContractError::InfusionIsEnded {});
    } else {
        infusion.enabled = false;
        INFUSION.save(deps.storage, key, &infusion)?;
    }

    Ok(Response::new())
}
pub fn execute_create_infusion(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    infusions: Vec<Infusion>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<SubMsg> = Vec::new();
    let mut fee_msgs: Vec<CosmosMsg<Empty>> = Vec::new();
    let mut attrs = vec![];

    if config.max_infusions < infusions.len() as u64 {
        return Err(ContractError::MaxInfusionsError {});
    }

    let collection_checksum = config.code_hash.clone();
    let salt1 = generate_instantiate_salt2(&collection_checksum, env.block.height);

    // loop through each infusion
    for infusion in infusions {
        // assert creation fees
        if let Some(creation_fee) = config.min_creation_fee.clone() {
            if info.funds.iter().find(|&e| e == &creation_fee).is_some() {
                let base_fee = CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: vec![creation_fee],
                });
                fee_msgs.push(base_fee);
            } else {
                return Err(ContractError::RequirednfusionFeeError);
            }
        }

        // assert fees being set
        if let Some(mf) = infusion.infusion_params.mint_fee.clone() {
            if !mf.amount.is_zero() {
                if !(config
                    .min_infusion_fee
                    .clone()
                    .is_some_and(|f| f.amount > mf.amount))
                {
                } else {
                    return Err(ContractError::InfusionFeeLessThanMinimumRequired {
                        min: config
                            .min_infusion_fee
                            .expect("should never be empty if errors"),
                    });
                }
            } else {
                return Err(ContractError::InfusionFeeCannotbeZero);
            }
        }

        // checks min_per_bundle
        if config.max_bundles < infusion.collections.len() as u64 {
            return Err(ContractError::TooManyCollectionsInInfusion {
                have: infusion.collections.len() as u64,
                max: config.max_bundles,
            });
        }

        // checks for any unique collections
        let mut unique_collections = Vec::new();
        for col in infusion.collections.iter() {
            if unique_collections.contains(&col.addr) {
                return Err(ContractError::DuplicateCollectionInInfusion);
            } else {
                // check if addr is cw721 collection
                let _res: cw721::ContractInfoResponse = deps
                    .querier
                    .query_wasm_smart(col.addr.clone(), &cw721::Cw721QueryMsg::ContractInfo {})
                    .map_err(|_| {
                        return ContractError::AddrIsNotNFTCol {
                            addr: col.addr.to_string(),
                        };
                    })?;

                // checks # of nft required per bundle
                if col.addr.to_string().is_empty() {
                    return Err(ContractError::BundleCollectionContractEmpty {});
                }
                if col.min_req < config.min_per_bundle
                    || col.min_req > config.max_per_bundle
                    || col.max_req.is_some_and(|a| config.max_per_bundle < a)
                    || col.max_req.is_some_and(|a| config.min_per_bundle > a)
                {
                    return Err(ContractError::BadBundle {
                        have: col.min_req,
                        min: config.min_per_bundle,
                        max: config.max_per_bundle,
                    });
                }
                unique_collections.push(col.addr.clone());
            }
        }

        // sanitize base token uri
        let mut base_token_uri = infusion.infused_collection.base_uri.trim().to_string();
        // Token URI must be a valid URL (ipfs, https, etc.)
        let parsed_token_uri =
            Url::parse(&base_token_uri).map_err(|_| ContractError::InvalidBaseTokenURI {})?;
        base_token_uri = parsed_token_uri.to_string();

        // predict the infused collection contract address
        let infusion_addr = match instantiate2_address(
            collection_checksum.as_slice(),
            &deps.api.addr_canonicalize(env.contract.address.as_str())?,
            salt1.as_slice(),
        ) {
            Ok(addr) => addr,
            Err(err) => return Err(ContractError::from(err)),
        };

        let infusion_collection_addr_human = deps.api.addr_humanize(&infusion_addr)?;
        // get the global infusion id
        let infusion_id: u64 = config.latest_infusion_id + 1;
        config.latest_infusion_id = infusion_id;

        // sets infuser contract as admin if no admin specified (not sure if we want this)
        let admin = Some(
            infusion
                .infused_collection
                .admin
                .unwrap_or(env.contract.address.to_string()),
        );

        // select if sg or vanilla cw721
        let init_msg = match infusion.infused_collection.sg {
            false => to_json_binary(&Cw721InstantiateMsg {
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                minter: env.contract.address.to_string(),
            })?,
            true => to_json_binary(&Sg721InitMsg {
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                minter: env.contract.address.to_string(), // this contract
                collection_info: CollectionInfo::<RoyaltyInfoResponse> {
                    creator: admin.clone().unwrap_or(info.sender.to_string()),
                    description: "Infused Collection".into(),
                    image: base_token_uri.clone(),
                    external_link: infusion.infused_collection.external_link.clone(),
                    explicit_content: infusion.infused_collection.explicit_content.clone(),
                    start_trading_time: infusion.infused_collection.start_trading_time.clone(),
                    royalty_info: infusion.infused_collection.royalty_info.clone(),
                },
            })?,
        };

        let init_infusion = WasmMsg::Instantiate2 {
            admin: admin.clone(),
            code_id: config.code_id,
            msg: init_msg,
            funds: vec![],
            label: "Infused without permission".to_string()
                + infusion.infused_collection.name.as_ref()
                + &env.block.height.to_string(),
            salt: salt1.clone(),
        };

        let infusion_collection_submsg =
            SubMsg::<Empty>::reply_on_success(init_infusion, INFUSION_COLLECTION_INIT_MSG_ID);

        let token_ids = random_token_list(
            &env,
            deps.api.addr_validate(
                &infusion
                    .infused_collection
                    .addr
                    .unwrap_or(info.sender.to_string()),
            )?,
            (1..=infusion.infused_collection.num_tokens).collect::<Vec<u32>>(),
        )?;

        // Save mintable token ids map
        let mut token_position = 1;
        for token_id in token_ids {
            MINTABLE_TOKEN_POSITIONS.save(deps.storage, token_position, &token_id)?;
            token_position += 1;
        }

        let infusion_config = InfusionState {
            collections: infusion.collections,
            infused_collection: InfusedCollection {
                addr: Some(infusion_collection_addr_human.to_string()),
                admin: admin.clone(),
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                base_uri: infusion.infused_collection.base_uri,
                num_tokens: infusion.infused_collection.num_tokens,
                sg: infusion.infused_collection.sg,
                royalty_info: infusion.infused_collection.royalty_info,
                start_trading_time: infusion.infused_collection.start_trading_time,
                explicit_content: infusion.infused_collection.explicit_content,
                external_link: infusion.infused_collection.external_link,
            },
            infusion_params: InfusionParamState {
                mint_fee: infusion.infusion_params.mint_fee,
                params: infusion.infusion_params.params,
            },
            payment_recipient: infusion.payment_recipient.unwrap_or(info.sender.clone()),
            owner: infusion.owner.unwrap_or(info.sender.clone()),
            enabled: true,
        };

        // saves the infusion bundle to state with (infused_collection, id)
        let key = (infusion_collection_addr_human.clone(), infusion_id);
        INFUSION.save(deps.storage, key.clone(), &infusion_config)?;
        INFUSION_ID.save(deps.storage, infusion_id, &key)?;
        MINTABLE_NUM_TOKENS.save(
            deps.storage,
            infusion_collection_addr_human.to_string(),
            &infusion.infused_collection.num_tokens,
        )?;
        CONFIG.save(deps.storage, &config)?;

        msgs.push(infusion_collection_submsg);
        attrs.push(Attribute::new("infusion-id", infusion_id.to_string()));
    }

    Ok(Response::new()
        .add_submessages(msgs)
        .add_messages(fee_msgs)
        .add_attributes(attrs))
}

fn execute_infuse_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    infusion_id: u64,
    bundle: Vec<Bundle>,
) -> Result<Response, ContractError> {
    let res = Response::new();
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    let config = CONFIG.load(deps.storage)?;
    let key = INFUSION_ID.load(deps.storage, infusion_id)?;
    let infusion = INFUSION.load(deps.storage, key)?;

    if !infusion.enabled {
        return Err(ContractError::InfusionIsEnded {});
    }

    let mut funds = info.funds.clone();

    // first, any fee parameters are validated
    if let Some(fee) = infusion.infusion_params.mint_fee.clone() {
        let mut fee_error = None;
        funds
            .iter_mut()
            .filter(|a| a.denom == fee.denom)
            .for_each(|a| {
                if a.amount < fee.amount {
                    fee_error = Some(ContractError::FeeNotAccepted {});
                } else {
                    a.amount -= fee.amount;
                }
            });
        if let Some(e) = fee_error {
            return Err(e);
        } else {
            // split fees between admin and infusion owner
            let contract_fee = fee.amount.u128() as u64 * config.admin_fee / 100;
            let remaining_fee_amount = fee.amount.u128() as u64 * (100 - config.admin_fee) / 100;

            if contract_fee != 0 {
                let base_fee = CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: vec![coin(contract_fee.into(), fee.denom.clone())],
                });
                msgs.push(base_fee);
            }

            // remaining fee to infusion owner
            let fee_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: infusion.payment_recipient.to_string(),
                amount: vec![coin(remaining_fee_amount.into(), fee.denom.clone())],
            });

            msgs.push(fee_msg);
        }
    }

    // check lens
    if bundle.is_empty() {
        return Err(ContractError::EmptyBundle);
    }

    let sender = info.sender.clone();
    // for each nft collection bundle sent to infuse
    for bundle in bundle {
        // assert ownership
        is_nft_owner(deps.as_ref(), sender.clone(), bundle.nfts.clone())?;
        // add each burn nft & mint infused token to response
        msgs.extend(burn_bundle(
            &deps,
            env.clone(),
            bundle.nfts,
            &sender,
            &infusion,
            &funds,
        )?)
    }

    Ok(res.add_messages(msgs))
}

// burns all nft bundles
fn burn_bundle(
    deps: &DepsMut,
    env: Env,
    nfts: Vec<NFT>,
    sender: &Addr,
    infusion: &InfusionState,
    funds: &Vec<Coin>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    // confirm bundle is in current infusion, and expected amount sent
    check_bundles(nfts.clone(), infusion.collections.clone(), &funds)?;
    for nft in nfts {
        msgs.push(into_cosmos_msg(
            Cw721ExecuteMsg::Burn {
                token_id: nft.token_id.to_string(),
            },
            nft.addr,
            None,
        )?);
    }

    // increment tokens
    let token_id = get_next_id(
        deps,
        env.clone(),
        Addr::unchecked(
            infusion
                .infused_collection
                .addr
                .clone()
                .expect("no infused colection"),
        ),
        sender.clone(),
    )?;

    // mint_msg
    let mint_msg: Cw721ExecuteMessage<Empty, Empty> = Cw721ExecuteMessage::Mint {
        token_id: token_id.token_id.to_string(),
        owner: sender.to_string(),
        token_uri: Some(format!(
            "{}/{}",
            infusion.infused_collection.base_uri.clone(),
            token_id.token_id.to_string()
        )),
        extension: Empty {},
    };

    let msg = into_cosmos_msg(
        mint_msg,
        infusion
            .infused_collection
            .addr
            .clone()
            .expect("no infused colection"),
        None,
    )?;
    msgs.push(msg);
    Ok(msgs)
}

fn check_bundles(
    bundle: Vec<NFT>,
    elig_col: Vec<NFTCollection>,
    sent: &Vec<Coin>,
) -> Result<(), ContractError> {
    // verify correct # of nft's provided & are accepted nfts
    // verify that the bundle is included in infusions
    for c in &elig_col {
        let elig = bundle
            .iter()
            .filter(|b| b.addr == c.addr)
            .collect::<Vec<_>>();

        if let Some(ps) = c.payment_substitute.clone() {
            if !sent
                .iter()
                .any(|coin| coin.denom == ps.denom && coin.amount >= ps.amount)
            {
                let mut havea = 0u128;
                let mut haved = String::new();
                for coin in sent {
                    if coin.denom == ps.denom {
                        havea = coin.amount.into();
                        haved = coin.denom.clone();
                        break;
                    }
                }
                return Err(ContractError::PaymentSubstituteNotProvided {
                    col: c.addr.to_string(),
                    haved,
                    havea: havea.to_string(),
                    wantd: ps.denom,
                    wanta: ps.amount.to_string(),
                });
            }
        } else if elig.is_empty() {
            return Err(ContractError::CollectionNotEligible {
                col: c.addr.to_string(),
            });
        // ensure minimum is met
        } else if elig.len() as u64 != c.min_req {
            // if can subsitute nft with payment, assert the correct amount has been sent.
            return Err(ContractError::BundleNotAccepted {
                have: elig.len() as u64,
                want: c.min_req,
            });
        } else if c.max_req.is_some_and(|a| elig.len() as u64 > a) {
            return Err(ContractError::BundleNotAccepted {
                have: elig.len() as u64,
                want: c.min_req,
            });
        }
    }
    Ok(())
}

pub fn into_cosmos_msg<M: Serialize, T: Into<String>>(
    message: M,
    contract_addr: T,
    funds: Option<Vec<Coin>>,
) -> StdResult<CosmosMsg> {
    let msg = to_json_binary(&message)?;
    let execute = WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg,
        funds: funds.unwrap_or_default(),
    };
    Ok(execute.into())
}

/// Get the next token id for the infused collection addr being minted
/// TODO: will prob need hook or query to collection to confirm the next token_id  is accurate
fn get_next_id(
    deps: &DepsMut,
    env: Env,
    infused_col_addr: Addr,
    sender: Addr,
) -> Result<TokenPositionMapping, ContractError> {
    let mintable_num_tokens =
        MINTABLE_NUM_TOKENS.load(deps.storage, infused_col_addr.to_string())?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    let mintable_token_mapping =
        random_mintable_token_mapping(deps.as_ref(), env, sender.clone(), mintable_num_tokens)?;

    Ok(mintable_token_mapping)
}

pub fn get_current_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, ContractError> {
    let token_id = INFUSION_INFO.load(storage, &addr)?.next_id;
    Ok(token_id)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_infusion(deps: Deps, addr: Addr, id: u64) -> StdResult<InfusionState> {
    let infusion = INFUSION.load(deps.storage, (addr, id))?;
    Ok(infusion)
}
pub fn query_infusion_by_id(deps: Deps, id: u64) -> StdResult<InfusionState> {
    let infuser = INFUSION_ID.load(deps.storage, id)?;
    let infusion = INFUSION.load(deps.storage, infuser)?;
    Ok(infusion)
}

pub fn query_infusions(deps: Deps, addr: Addr, index: u64) -> StdResult<InfusionsResponse> {
    let mut infusions = vec![];

    for i in index..=index + 30 {
        let id = i;
        // return the response for each
        let state = INFUSION.load(deps.storage, (addr.clone(), id))?;
        infusions.push(state);
    }

    Ok(InfusionsResponse { infusions })
}

pub fn query_if_is_in_bundle(deps: Deps, addr: Addr, id: u64) -> StdResult<bool> {
    let key = INFUSION_ID.load(deps.storage, id)?;
    Ok(INFUSION
        .load(deps.storage, key)?
        .collections
        .iter()
        .any(|a| a.addr == addr))
}

/// verifies all nfts defined in bundle are of ownership of the current sender
pub fn is_nft_owner(deps: Deps, sender: Addr, nfts: Vec<NFT>) -> Result<(), ContractError> {
    for nft in nfts {
        let nft_address = nft.addr;
        let token_id = nft.token_id;

        let owner_response: OwnerOfResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: nft_address.to_string(),
                msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: token_id.to_string(),
                    include_expired: None,
                })?,
            }))?;

        if owner_response.owner != sender {
            return Err(ContractError::SenderNotOwner {});
        }
    }
    Ok(())
}
/// Generates the value used with instantiate2, via a hash of the infusers checksum.
pub const SALT_POSTFIX: &[u8] = b"infusion";
pub fn generate_instantiate_salt2(checksum: &HexBinary, height: u64) -> Binary {
    let mut hash = Vec::new();
    hash.extend_from_slice(checksum.as_slice());
    hash.extend_from_slice(&height.to_be_bytes());
    let checksum_hash = <sha2::Sha256 as sha2::Digest>::digest(hash);
    let mut result = checksum_hash.to_vec();
    result.extend_from_slice(SALT_POSTFIX);
    Binary(result)
}

fn random_token_list(
    env: &Env,
    sender: Addr,
    mut tokens: Vec<u32>,
) -> Result<Vec<u32>, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };
    let sha256 = Sha256::digest(
        format!("{}{}{}{}", sender, env.block.height, tokens.len(), tx_index).into_bytes(),
    );
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    let mut shuffler = FisherYates::default();
    shuffler
        .shuffle(&mut tokens, &mut rng)
        .map_err(StdError::generic_err)?;
    Ok(tokens)
}

// Does a baby shuffle, picking a token_id from the first or last 50 mintable positions.
fn random_mintable_token_mapping(
    deps: Deps,
    env: Env,
    sender: Addr,
    num_tokens: u32,
) -> Result<TokenPositionMapping, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };
    let sha256 = Sha256::digest(
        format!("{}{}{}{}", sender, num_tokens, env.block.height, tx_index).into_bytes(),
    );
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();

    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let r = rng.next_u32();

    let order = match r % 2 {
        1 => Order::Descending,
        _ => Order::Ascending,
    };
    let mut rem = 50;
    if rem > num_tokens {
        rem = num_tokens;
    }
    let n = r % rem;
    let position = MINTABLE_TOKEN_POSITIONS
        .keys(deps.storage, None, None, order)
        .skip(n as usize)
        .take(1)
        .collect::<StdResult<Vec<_>>>()?[0];

    let token_id = MINTABLE_TOKEN_POSITIONS.load(deps.storage, position)?;
    Ok(TokenPositionMapping { position, token_id })
}

//  source: https://github.com/public-awesome/launchpad/blob/main/contracts/minters/vending-minter/src/contract.rs#L1371
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }

    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }
    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let event = Event::new("migrate")
        .add_attribute("from_name", current_version.contract)
        .add_attribute("from_version", current_version.version)
        .add_attribute("to_name", CONTRACT_NAME)
        .add_attribute("to_version", CONTRACT_VERSION);

    Ok(Response::new().add_event(event))
}

#[cfg(test)]
mod tests {}
