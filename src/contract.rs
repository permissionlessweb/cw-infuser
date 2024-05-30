#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    instantiate2_address, to_json_binary, Addr, Binary, Checksum, Coin, CosmosMsg, Deps, DepsMut,
    Empty, Env, MessageInfo, QueryRequest, Reply, Response, StdResult, Storage, SubMsg,
    SubMsgResult, WasmMsg, WasmQuery,
};
use cw721_base::{ExecuteMsg as Cw721ExecuteMessage, InstantiateMsg as Cw721InstantiateMsg};
use serde::Serialize;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{self, ConfigResponse, ExecuteMsg, InfusionsResponse, InstantiateMsg, QueryMsg};
use crate::state::{
    Bundle, Config, InfusedCollection, Infusion, InfusionInfo, NFTCollection, CONFIG, INFUSION, INFUSION_ID, INFUSION_INFO, NFT
};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-infuser";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/
const INFUSION_COLLECTION_INIT_MSG_ID: u64 = 21;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let res = Response::new();
    let InstantiateMsg {
        default_infusion_params,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            default_infusion_params,
            latest_infusion_id: None,
        },
    )?;

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Infuse {
            bundle,
            infusion_id,
        } => execute_infuse_bundle(deps, info, infusion_id, bundle),
        ExecuteMsg::UpdateConfig {} => todo!(),
        ExecuteMsg::CreateInfusion { collections } => {
            execute_create_infusion(deps, info, env, collections)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Infusion { addr, id } => to_json_binary(&query_infusion(deps, addr, id)?),
        QueryMsg::InfusionById { id } => to_json_binary(&query_infusion_by_id(deps, id)?),
        QueryMsg::Infusions { addr } => to_json_binary(&query_infusions(deps, addr)?),
        QueryMsg::IsInBundle { collection_addr } => {
            to_json_binary(&query_infusions(deps, collection_addr)?)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg {
        Reply {
            id: INFUSION_COLLECTION_INIT_MSG_ID,
            result,
            payload,
            gas_used,
        } => handle_reply(deps, result, payload, gas_used),
        _ => Err(ContractError::UnexpectedReply {}),
    }
}

pub fn execute_create_infusion(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    collections: Vec<Infusion>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<SubMsg> = Vec::new();

    let collection_checksum = deps
        .querier
        .query_wasm_code_info(config.default_infusion_params.code_id.clone())?
        .checksum;
    let salt1 = generate_instantiate_salt2(&collection_checksum);

    for bundle in collections {
        let infusion_addr = match instantiate2_address(
            &collection_checksum.as_slice(),
            &deps.api.addr_canonicalize(env.contract.address.as_str())?,
            salt1.as_slice(),
        ) {
            Ok(addr) => addr,
            Err(err) => return Err(ContractError::from(err)),
        };
        let infusion_collection_addr_human = deps.api.addr_humanize(&infusion_addr)?;

        let infusion_id: u64 = CONFIG
            .update(deps.storage, |mut c| -> StdResult<_> {
                c.latest_infusion_id = c.latest_infusion_id.map_or(Some(0), |id| Some(id + 1));
                Ok(c)
            })?
            .latest_infusion_id
            .unwrap();

            let admin = Some(bundle.infused_collection.admin.unwrap_or(env.contract.address.to_string()));

        let infusion_config = Infusion {
            collections: bundle.collections,
            infused_collection: InfusedCollection {
                addr: infusion_collection_addr_human,
                admin: admin.clone(),
                name: bundle.infused_collection.name.clone(),
                symbol: bundle.infused_collection.symbol.to_string(),
            },
            infusion_params: bundle.infusion_params,
            infusion_id,
        };

        let init_msg = Cw721InstantiateMsg {
            name: bundle.infused_collection.name.clone(),
            symbol: bundle.infused_collection.symbol,
            minter: env.contract.address.to_string(),
        };

        let init_infusion = WasmMsg::Instantiate2 {
            admin: admin.clone(),
            code_id: config.default_infusion_params.code_id,
            msg: to_json_binary(&init_msg)?,
            funds: vec![],
            label: "Infused collection".to_string() + bundle.infused_collection.name.as_ref(),
            salt: salt1.clone(),
        };

        let infusion_collection_submsg =
            SubMsg::<Empty>::reply_on_success(init_infusion, INFUSION_COLLECTION_INIT_MSG_ID);

        // gets the next id for an address
        let id = get_next_id(deps.storage, info.sender.clone())?;

        let key = (info.sender.clone(), id);
        // saves the infusion bundle to state for query by id for each address
        INFUSION.save(deps.storage, key.clone(), &infusion_config)?;
        INFUSION_ID.save(deps.storage, infusion_id, &key)?;

        msgs.push(infusion_collection_submsg)
    }

    Ok(Response::new().add_submessages(msgs))
}

pub fn handle_reply(
    _deps: DepsMut,
    _result: SubMsgResult,
    _payload: Binary,
    gas_used: u64,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("gas_used", gas_used.to_string()))
}

pub fn execute_infuse_bundle(
    deps: DepsMut,
    info: MessageInfo,
    infusion_id: u64,
    bundle: Vec<Bundle>,
) -> Result<Response, ContractError> {
    let res = Response::new();
    let mut msgs: Vec<CosmosMsg> = Vec::new();

    for bundle in bundle {
        let sender = info.sender.clone();
        // confirms ownership for each nft in bundle
        is_nft_owner(deps.as_ref(), sender.clone(), bundle.nfts.clone())?;

        // burns nfts in each bundle, mint infused token also
        let messages = burn_bundle(deps.storage, sender, bundle.nfts, infusion_id)?;
        // add msgs to response
        msgs.extend(messages)
    }

    Ok(res.add_messages(msgs))
}

fn check_bundles(
    storage: &mut dyn Storage,
    id: u64,
    bundle: Vec<NFT>,
) -> Result<(), ContractError> {
    // get the InfusionConfig
    let key = INFUSION_ID.load(storage, id)?;
    let infusion = INFUSION.load(storage, key)?;
    // verify that the bundle is include in i

    for nft in &infusion.collections {
        if !bundle.iter().any(|b| nft.addr == b.addr.clone()) {
            return Err(ContractError::BundleNotAccepted);
        }
    }

    Ok(())
}
// burns all nft bundles
fn burn_bundle(
    storage: &mut dyn Storage,
    sender: Addr,
    nfts: Vec<NFT>,
    id: u64,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let _config = CONFIG.load(storage)?;
    let key = INFUSION_ID.load(storage, id)?;
    let infusion = INFUSION.load(storage, key)?;

    // confirm bundle is in current infusion
    check_bundles(storage, id, nfts.clone())?;

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
    let token_id = get_next_id(storage, infusion.infused_collection.addr.clone())?;

    // mint_msg
    let mint_msg = Cw721ExecuteMessage::<Empty, Empty>::Mint {
        token_id: token_id.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Empty {},
    };

    let msg = into_cosmos_msg(mint_msg, infusion.infused_collection.addr, None)?;

    messages.push(msg);

    Ok(messages)
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        infusion_params: state.default_infusion_params,
    };

    Ok(resp)
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

pub fn query_infusions(deps: Deps, addr: Addr) -> StdResult<InfusionsResponse> {
    let mut infusions = vec![];
    let current_id = INFUSION_INFO.load(deps.storage, &addr.clone())?.next_id;

    for i in 0..=current_id {
        let id = i as u64;
        // return the response for each
        let state = INFUSION.load(deps.storage, (addr.clone(), id))?;
        infusions.push(state);
    }

    Ok(msg::InfusionsResponse {
        infusions: infusions,
    })
}

// confirm ownership
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

        if owner_response.owner != sender.to_string() {
            return Err(ContractError::SenderNotOwner {});
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

pub const SALT_POSTFIX: &[u8] = b"infusion";
pub fn generate_instantiate_salt2(checksum: &Checksum) -> Binary {
    let account_id_hash = <sha2::Sha256 as sha2::Digest>::digest(checksum.to_string());
    let mut hash = account_id_hash.to_vec();
    hash.extend(SALT_POSTFIX);
    Binary::new(hash.to_vec())
}

fn get_next_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, ContractError> {
    let token_id = INFUSION_INFO
        .update::<_, crate::error::ContractError>(storage, &addr, |x| match x {
            Some(mut info) => {
                info.next_id += 1;
                Ok(info)
            }
            None => Ok(InfusionInfo::default()),
        })?
        .next_id;
    Ok(token_id)
}
fn get_current_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, ContractError> {
    let token_id = INFUSION_INFO.load(storage, &addr)?.next_id;
    Ok(token_id)
}

#[cfg(test)]
mod tests {}
