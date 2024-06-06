use crate::{
    contract::{CwInfuser, CwInfuserResult},
    msg::CwInfuserExecuteMsg,
    state::{generate_instantiate_salt2, is_nft_owner, Bundle, InfusedCollection, Infusion, InfusionInfo, CONFIG, INFUSION, INFUSION_ID, INFUSION_INFO, NFT}, CwInfuserError,
};

use abstract_app::traits::AbstractResponse;
use cosmwasm_schema::serde::Serialize;
use cosmwasm_std::{instantiate2_address, to_json_binary, Addr, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Storage, SubMsg, WasmMsg};
use cw721::Cw721ExecuteMsg;
use cw721_base::{ExecuteMsg as Cw721ExecuteMessage,InstantiateMsg as Cw721InstantiateMsg};

const INFUSION_COLLECTION_INIT_MSG_ID: u64 = 21;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: CwInfuser,
    msg: CwInfuserExecuteMsg,
) -> CwInfuserResult {
    match msg {
        CwInfuserExecuteMsg::CreateInfusion { collections } => {
            execute_create_infusion(deps, info, env, collections)
        }
        CwInfuserExecuteMsg::Infuse {
            infusion_id,
            bundle,
        } => execute_infuse_bundle(deps, info, infusion_id, bundle),
        CwInfuserExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}


/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: CwInfuser) -> CwInfuserResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.response("update_config"))
}

pub fn execute_create_infusion(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    infusions: Vec<Infusion>,
) -> CwInfuserResult {
    let config = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<SubMsg> = Vec::new();

    // get cw721 code id
    let collection_checksum = deps
        .querier
        .query_wasm_code_info(config.default_infusion_params.code_id.clone())?
        .checksum;
    let salt1 = generate_instantiate_salt2(&collection_checksum);

    // loop through each infusion
    for infusion in infusions {
        // predict the contract address
        let infusion_addr = match instantiate2_address(
            &collection_checksum.as_slice(),
            &deps.api.addr_canonicalize(env.contract.address.as_str())?,
            salt1.as_slice(),
        ) {
            Ok(addr) => addr,
            Err(err) => return Err(CwInfuserError::from(err)),
        };
        let infusion_collection_addr_human = deps.api.addr_humanize(&infusion_addr)?;
        // get the global infusion id
        let infusion_id: u64 = CONFIG
            .update(deps.storage, |mut c| -> StdResult<_> {
                c.latest_infusion_id = c.latest_infusion_id.map_or(Some(0), |id| Some(id + 1));
                Ok(c)
            })?
            .latest_infusion_id
            .unwrap();
        
            let admin = Some(infusion.infused_collection.admin.unwrap_or(env.contract.address.to_string()));

        let infusion_config = Infusion {
            collections: infusion.collections,
            infused_collection: InfusedCollection {
                addr: infusion_collection_addr_human,
                admin: admin.clone(),
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.to_string(),
            },
            infusion_params: infusion.infusion_params,
            infusion_id,
        };

        let init_msg = Cw721InstantiateMsg {
            name: infusion.infused_collection.name.clone(),
            symbol: infusion.infused_collection.symbol,
            minter: env.contract.address.to_string(),
        };

        let init_infusion = WasmMsg::Instantiate2 {
            admin: admin.clone(),
            code_id: config.default_infusion_params.code_id,
            msg: to_json_binary(&init_msg)?,
            funds: vec![],
            label: "Infused collection".to_string() + infusion.infused_collection.name.as_ref(),
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


fn execute_infuse_bundle(
    deps: DepsMut,
    info: MessageInfo,
    infusion_id: u64,
    bundle: Vec<Bundle>,
) -> CwInfuserResult {
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
    println!("{:?}", msgs);

    Ok(res.add_messages(msgs))
}

// burns all nft bundles
fn burn_bundle(
    storage: &mut dyn Storage,
    sender: Addr,
    nfts: Vec<NFT>,
    id: u64,
) -> Result<Vec<CosmosMsg>, CwInfuserError> {
    let _config = CONFIG.load(storage)?;
    println!("burn bundle");
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

fn check_bundles(
    storage: &mut dyn Storage,
    id: u64,
    bundle: Vec<NFT>,
) -> Result<(), CwInfuserError> {
    // get the InfusionConfig
    let key = INFUSION_ID.load(storage, id)?;
    let infusion = INFUSION.load(storage, key)?;
    // verify that the bundle is include in i

    for nft in &infusion.collections {
        if !bundle.iter().any(|b| nft.addr == b.addr.clone()) {
            return Err(CwInfuserError::BundleNotAccepted);
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

fn get_next_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, CwInfuserError> {
    let token_id = INFUSION_INFO
        .update::<_, CwInfuserError>(storage, &addr, |x| match x {
            Some(mut info) => {
                info.next_id += 1;
                Ok(info)
            }
            None => Ok(InfusionInfo::default()),
        })?
        .next_id;
    Ok(token_id)
}

pub fn get_current_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, CwInfuserError> {
    let token_id = INFUSION_INFO.load(storage, &addr)?.next_id;
    Ok(token_id)
}