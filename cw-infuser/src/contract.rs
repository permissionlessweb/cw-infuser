#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, instantiate2_address, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Empty, Env, HexBinary, MessageInfo, QueryRequest, Reply, Response, StdError,
    StdResult, Storage, SubMsg, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
use cw721_base::{ExecuteMsg as Cw721ExecuteMessage, InstantiateMsg as Cw721InstantiateMsg};
use cw_controllers::AdminError;
use sg721::{CollectionInfo, InstantiateMsg as Sg721InitMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfusionsResponse, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Bundle, Config, InfusedCollection, Infusion, InfusionInfo, NFTCollection, CONFIG, INFUSION,
    INFUSION_ID, INFUSION_INFO, NFT,
};
use cosmwasm_schema::serde::Serialize;

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
        ExecuteMsg::CreateInfusion {
            collections,
            payment_recipient,
        } => execute_create_infusion(
            deps,
            info.clone(),
            env,
            collections,
            payment_recipient.unwrap_or(info.sender.clone()),
        ),
        ExecuteMsg::Infuse {
            infusion_id,
            bundle,
        } => execute_infuse_bundle(deps, info, infusion_id, bundle),
        ExecuteMsg::UpdateConfig {} => update_config(deps, info),
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

pub fn execute_create_infusion(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    infusions: Vec<Infusion>,
    payment_recipient: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<SubMsg> = Vec::new();
    let mut fee_msgs: Vec<CosmosMsg<Empty>> = Vec::new();

    if infusions.len() > config.max_infusions.try_into().unwrap() {
        return Err(ContractError::TooManyInfusions {});
    }

    let collection_checksum = config.code_hash;
    let salt1 = generate_instantiate_salt2(&collection_checksum);

    // loop through each infusion
    for infusion in infusions {
        // checks min_per_bundle
        if config.max_bundles < infusion.collections.len().try_into().unwrap() {
            return Err(ContractError::NotEnoughNFTsInBundle {
                have: infusion.collections.len().try_into().unwrap(),
                min: config.max_bundles,
                max: config.max_bundles,
            });
        }
        // checks # of nft required per bundle
        let required = infusion.infusion_params.min_per_bundle;
        if config.min_per_bundle > required || config.max_per_bundle < required {
            return Err(ContractError::BadBundle {
                have: required,
                min: config.min_per_bundle,
                max: config.max_per_bundle,
            });
        }

        // assert fees
        if let Some(creation_fee) = config.min_creation_fee.clone() {
            if info.funds.iter().find(|&e| e == &creation_fee).is_some() {
                let base_fee = CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: vec![creation_fee],
                });
                fee_msgs.push(base_fee);
            } else {
                return Err(ContractError::CreateInfusionFeeError);
            }
        }
        if let Some(fee) = infusion.infusion_params.mint_fee.clone() {
            if !fee.amount.is_zero() {
                if !(config
                    .min_infusion_fee
                    .clone()
                    .is_some_and(|f| f.amount > fee.amount))
                {
                } else {
                    return Err(ContractError::CreateInfusionFeeError);
                }
            } else {
                return Err(ContractError::CreateInfusionFeeError);
            }
        }

        // predict the contract address
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
        let infusion_id: u64 = CONFIG
            .update(deps.storage, |mut c| -> StdResult<_> {
                c.latest_infusion_id += 1u64;
                Ok(c)
            })?
            .latest_infusion_id;

        // sets infuser contract as admin if no admin specified (not sure if we want this)
        let admin = Some(
            infusion
                .infused_collection
                .admin
                .unwrap_or(env.contract.address.to_string()),
        );

        // cw721-collection
        let _init_msg = Cw721InstantiateMsg {
            name: infusion.infused_collection.name.clone(),
            symbol: infusion.infused_collection.symbol.clone(),
            minter: env.contract.address.to_string(),
        };

        // sg721-collection
        let sg_init_msg = Sg721InitMsg {
            name: infusion.infused_collection.name.clone(),
            symbol: infusion.infused_collection.symbol.clone(),
            minter: env.contract.address.to_string(),
            collection_info: CollectionInfo {
                creator: admin.clone().unwrap(),
                description: "Infused Collection".into(),
                image: infusion.infused_collection.base_uri.clone(),
                external_link: None,
                explicit_content: None,
                start_trading_time: None,
                royalty_info: None,
            },
        };

        let init_infusion = WasmMsg::Instantiate2 {
            admin: admin.clone(),
            code_id: config.code_id,
            msg: to_json_binary(&sg_init_msg)?,
            funds: vec![],
            label: "Infused collection".to_string() + infusion.infused_collection.name.as_ref() + &env.block.height.to_string(),
            salt: salt1.clone(),
        };

        let infusion_collection_submsg =
            SubMsg::<Empty>::reply_on_success(init_infusion, INFUSION_COLLECTION_INIT_MSG_ID);

        // gets the next id for an address
        let id = get_next_id(deps.storage, info.sender.clone())?;

        let infusion_config = Infusion {
            collections: infusion.collections,
            infused_collection: InfusedCollection {
                addr: Some(infusion_collection_addr_human.to_string()),
                admin: admin.clone(),
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                base_uri: infusion.infused_collection.base_uri,
            },
            infusion_params: infusion.infusion_params,
            payment_recipient: payment_recipient.clone(),
        };

        // saves the infusion bundle to state with (infused_collection, id)
        let key = (infusion_collection_addr_human, id);
        INFUSION.save(deps.storage, key.clone(), &infusion_config)?;
        INFUSION_ID.save(deps.storage, infusion_id, &key)?;

        msgs.push(infusion_collection_submsg)
    }

    Ok(Response::new().add_submessages(msgs).add_messages(fee_msgs))
}

fn execute_infuse_bundle(
    deps: DepsMut,
    info: MessageInfo,
    infusion_id: u64,
    bundle: Vec<Bundle>,
) -> Result<Response, ContractError> {
    let res = Response::new();
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    let config = CONFIG.load(deps.storage)?;
    let key = INFUSION_ID.load(deps.storage, infusion_id)?;
    let infusion = INFUSION.load(deps.storage, key)?;

    // add optional fee required for minting
    if let Some(fee) = infusion.infusion_params.mint_fee.clone() {
        // must be first token in tx & more than mint fee
        if info.funds.iter().find(|&e| e == &fee).is_none() {
            return Err(ContractError::FeeNotAccepted {});
        } else {
            // split fees between admin and infusion owner
            let base_fee_amount = fee.amount.u128() as u64 * config.admin_fee / 100;
            let remaining_fee_amount = fee.amount.u128() as u64 * (100 - config.admin_fee) / 100;

            if base_fee_amount != 0 {
                let base_fee = CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: vec![coin(base_fee_amount.into(), fee.denom.clone())],
                });
                msgs.push(base_fee);
            }
            // remaining fee to infusion owner
            let fee_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: infusion.payment_recipient.to_string(),
                amount: vec![coin(remaining_fee_amount.into(), fee.denom.clone())],
            });
            msgs.extend(vec![fee_msg]);
        }
    }

    // check lens
    if bundle.is_empty() {
        return Err(ContractError::BundleNotAccepted {});
    }
    // for each nft collection bundle sent to infuse
    for bundle in bundle {
        let sender = info.sender.clone();
        // assert ownership
        is_nft_owner(deps.as_ref(), sender.clone(), bundle.nfts.clone())?;
        // add each burn nft & mint infused token to response
        msgs.extend(burn_bundle(deps.storage, sender, bundle.nfts, &infusion)?)
    }

    Ok(res.add_messages(msgs))
}

// burns all nft bundles
fn burn_bundle(
    storage: &mut dyn Storage,
    sender: Addr,
    nfts: Vec<NFT>,
    infusion: &Infusion,
) -> Result<Vec<CosmosMsg>, ContractError> {
    // confirm bundle is in current infusion, and expected amount sent
    check_bundles(nfts.clone(), infusion.collections.clone())?;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    for nft in nfts {
        let token_id = nft.token_id;
        let address = nft.addr;
        let message = Cw721ExecuteMsg::Burn {
            token_id: token_id.to_string(),
        };
        let msg = into_cosmos_msg(message, address, None)?;
        messages.push(msg);
    }

    // increment tokens
    let token_id = get_next_id(
        storage,
        Addr::unchecked(
            infusion
                .infused_collection
                .addr
                .clone()
                .expect("no infused colection"),
        ),
    )?;

    // mint_msg
    let mint_msg = Cw721ExecuteMessage::<Empty, Empty>::Mint {
        token_id: token_id.to_string(),
        owner: sender.to_string(),
        token_uri: Some(infusion.infused_collection.base_uri.clone() + "/" + &token_id.to_string()),
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
    messages.push(msg);
    Ok(messages)
}

fn check_bundles(bundle: Vec<NFT>, collections: Vec<NFTCollection>) -> Result<(), ContractError> {
    // verify correct # of nft's provided, filter  collection from collections map
    // verify that the bundle is include in infusion
    for col in &collections {
        let matching_nfts: Vec<_> = bundle.iter().filter(|b| b.addr == col.addr).collect();
        if matching_nfts.is_empty() {
            return Err(ContractError::BundleNotAccepted);
        }
        if matching_nfts.len() as u64 != col.min_req {
            return Err(ContractError::NotEnoughNFTsInBundle {
                have: matching_nfts.len().try_into().unwrap(),
                min: col.min_req,
                max: col.min_req,
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
/// TODO: will prob need hook or query to collection to confirm accurate
fn get_next_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, ContractError> {
    let token_id = INFUSION_INFO
        .update::<_, ContractError>(storage, &addr, |x| match x {
            Some(mut info) => {
                info.next_id += 1;
                Ok(info)
            }
            None => Ok(InfusionInfo::default()),
        })?
        .next_id;
    Ok(token_id)
}

pub fn get_current_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, ContractError> {
    let token_id = INFUSION_INFO.load(storage, &addr)?.next_id;
    Ok(token_id)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn query_infusion(deps: Deps, addr: Addr, id: u64) -> StdResult<Infusion> {
    let infusion = INFUSION.load(deps.storage, (addr, id))?;
    Ok(infusion)
}
pub fn query_infusion_by_id(deps: Deps, id: u64) -> StdResult<Infusion> {
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

/// Generates the value used with instantiate2, via a hash of the infusers checksum.
pub const SALT_POSTFIX: &[u8] = b"infusion";
pub fn generate_instantiate_salt2(checksum: &HexBinary) -> Binary {
    let checksum_hash = <sha2::Sha256 as sha2::Digest>::digest(checksum.to_string());
    let mut hash = checksum_hash.to_vec();
    hash.extend(SALT_POSTFIX);
    Binary(hash.to_vec())
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg(test)]
mod tests {}
