use crate::error::{AnyOfErr, ContractError};
use crate::msg::{ExecuteMsg, InfusionsResponse, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    Bundle, BundleBlend, BundleType, Config, InfusedCollection, Infusion, InfusionParamState,
    InfusionState, NFTCollection, TokenPositionMapping, UpdatingConfig, CONFIG, INFUSION,
    INFUSION_ID, INFUSION_INFO, MINTABLE_NUM_TOKENS, MINTABLE_TOKEN_VECTORS, MINT_COUNT, NFT,
};
use cosmwasm_schema::serde::Serialize;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, instantiate2_address, to_json_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    Decimal, Deps, DepsMut, Empty, Env, Event, HexBinary, MessageInfo, QuerierWrapper,
    QueryRequest, Reply, Response, StdError, StdResult, Storage, SubMsg, Uint128, WasmMsg,
    WasmQuery,
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
    if !(msg.owner_fee <= Decimal::one()) {
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

    // admin is either sender or manually set.
    let mut contract_owner = info.sender;
    if let Some(ad) = msg.contract_owner {
        contract_owner = deps.api.addr_validate(&ad)?;
    }

    // get checksum of cw721
    let cw721_checksum = deps.querier.query_wasm_code_info(msg.cw721_code_id)?;
    CONFIG.save(
        deps.storage,
        &Config {
            contract_owner,
            min_per_bundle: msg.min_per_bundle.unwrap_or(1),
            max_per_bundle: msg.max_per_bundle.unwrap_or(10u64),
            code_id: msg.cw721_code_id,
            code_hash: cw721_checksum.checksum,
            latest_infusion_id: 0,
            max_infusions: msg.max_infusions.unwrap_or(2u64),
            max_bundles: msg.max_bundles.unwrap_or(5),
            owner_fee: msg.owner_fee,
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
        ExecuteMsg::UpdateConfig { config } => update_config(deps, info, config),
        ExecuteMsg::EndInfusion { id } => execute_end_infusion(deps, info, id),
        ExecuteMsg::UpdateInfusionBaseUri { id, base_uri } => {
            update_infused_base_uri(deps, info, id, base_uri)
        }
        ExecuteMsg::UpdateInfusionsEligibleCollections {
            id,
            to_add,
            to_remove,
        } => update_infusion_eligible_collections(deps, info, id, to_add, to_remove),
        ExecuteMsg::UpdateInfusionMintFee { id, mint_fee } => {
            update_infusion_mint_fee(deps, info, id, mint_fee)
        }
        ExecuteMsg::UpdateInfusionBundleType { id, bundle_type } => {
            update_infusion_bundle_type(deps, info, id, bundle_type)
        }
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

/// Update the infused collec
fn update_infusion_eligible_collections(
    deps: DepsMut,
    msg: MessageInfo,
    id: u64,
    to_add: Vec<NFTCollection>,
    to_remove: Vec<NFTCollection>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let key = INFUSION_ID.load(deps.storage, id)?;
    let mut infusion = INFUSION.load(deps.storage, key.clone())?;

    if infusion.owner != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }

    for nft in to_remove {
        // Iterate through the collections to find and remove the NFT
        for i in 0..infusion.collections.len() {
            if infusion.collections[i] == nft {
                infusion.collections.remove(i);
                break;
            }
        }
    }
    // ensure new eligible collection params
    let collections = validate_eligible_collection_list(
        deps.querier,
        &cfg,
        &infusion.infusion_params.bundle_type,
        &infusion.collections,
        &to_add,
        true,
    )?;
    infusion.collections = collections;

    INFUSION.save(deps.storage, key, &infusion)?;
    Ok(Response::new())
}

/// Update the baseuri used for infused collection metadata
fn update_infused_base_uri(
    deps: DepsMut,
    msg: MessageInfo,
    id: u64,
    base_uri: String,
) -> Result<Response, ContractError> {
    let key = INFUSION_ID.load(deps.storage, id)?;
    let mut infusion = INFUSION.load(deps.storage, key.clone())?;
    if infusion.owner != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }
    infusion.infused_collection.base_uri = base_uri;
    INFUSION.save(deps.storage, key, &infusion)?;

    Ok(Response::new())
}

/// Update the mint fee for an infusion
fn update_infusion_bundle_type(
    deps: DepsMut,
    msg: MessageInfo,
    id: u64,
    bt: BundleType,
) -> Result<Response, ContractError> {
    let key = INFUSION_ID.load(deps.storage, id)?;
    let mut infusion = INFUSION.load(deps.storage, key.clone())?;
    if infusion.owner != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }
    match &bt {
        BundleType::AllOf {} => {}
        BundleType::AnyOf { addrs } => {
            let mut unique = vec![];
            for col in infusion.collections.iter() {
                unique.push(col.addr.clone());
            }
            for addr in addrs {
                if !unique.contains(&addr) {
                    return Err(ContractError::AnyOfConfigError {
                        err: AnyOfErr::Uneligible,
                    });
                }
            }
        }
        BundleType::AnyOfBlend { blends: _ } => return Err(ContractError::UnImplemented),
    }

    infusion.infusion_params.bundle_type = bt;
    INFUSION.save(deps.storage, key, &infusion)?;

    Ok(Response::new())
}

/// Update the mint fee for an infusion
fn update_infusion_mint_fee(
    deps: DepsMut,
    msg: MessageInfo,
    id: u64,
    mint_fee: Option<Coin>,
) -> Result<Response, ContractError> {
    let key = INFUSION_ID.load(deps.storage, id)?;
    let mut infusion = INFUSION.load(deps.storage, key.clone())?;
    if infusion.owner != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }
    infusion.infusion_params.mint_fee = mint_fee;
    INFUSION.save(deps.storage, key, &infusion)?;

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
    let mut cfg = CONFIG.load(deps.storage)?;
    let mut msgs: Vec<SubMsg> = Vec::new();
    let mut fee_msgs: Vec<CosmosMsg<Empty>> = Vec::new();
    let mut attrs = vec![];

    if cfg.max_infusions < infusions.len() as u64 {
        return Err(ContractError::MaxInfusionsError {});
    }

    let collection_checksum = cfg.code_hash.clone();
    let salt1 = generate_instantiate_salt2(&collection_checksum, env.block.height);

    // loop through each infusion
    for infusion in infusions {
        // assert description length
        if infusion.description.is_some_and(|a| a.len() > 512) {
            return Err(ContractError::InfusionDescriptionLengthError {});
        }
        // assert creation fees
        if let Some(creation_fee) = cfg.min_creation_fee.clone() {
            if info.sender.to_string() == cfg.contract_owner {
                // skip over creation fees for admin
            } else {
                if info.funds.iter().find(|&e| e == &creation_fee).is_some() {
                    let base_fee = CosmosMsg::Bank(BankMsg::Send {
                        to_address: cfg.contract_owner.to_string(),
                        amount: vec![creation_fee],
                    });
                    fee_msgs.push(base_fee);
                } else {
                    return Err(ContractError::RequirednfusionFeeError { fee: creation_fee });
                }
            }
        }

        // assert fees being set
        if let Some(mf) = infusion.infusion_params.mint_fee.clone() {
            if !mf.amount.is_zero() {
                if !(cfg
                    .min_infusion_fee
                    .clone()
                    .is_some_and(|f| f.amount > mf.amount))
                {
                } else {
                    return Err(ContractError::InfusionFeeLessThanMinimumRequired {
                        min: cfg
                            .min_infusion_fee
                            .expect("should never be empty if errors"),
                    });
                }
            } else {
                return Err(ContractError::InfusionFeeCannotbeZero);
            }
        }

        validate_eligible_collection_list(
            deps.querier,
            &cfg,
            &infusion.infusion_params.bundle_type,
            &vec![],
            &infusion.collections,
            false,
        )?;
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
        let infusion_id: u64 = cfg.latest_infusion_id + 1;
        cfg.latest_infusion_id = infusion_id;

        // sets msg sender as infusion admin for infused collection if not specified
        let infusion_admin = infusion
            .infused_collection
            .admin
            .unwrap_or(info.sender.to_string());

        // select if sg or vanilla cw721
        let init_msg = match infusion.infused_collection.sg {
            false => to_json_binary(&Cw721InstantiateMsg {
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                minter: env.contract.address.to_string(), // this contract
            })?,
            true => to_json_binary(&Sg721InitMsg {
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                minter: env.contract.address.to_string(), // this contract
                collection_info: CollectionInfo::<RoyaltyInfoResponse> {
                    creator: infusion_admin.clone(),
                    description: "Infused Collection: ".to_owned()
                        + &infusion.infused_collection.description,
                    image: infusion.infused_collection.image.clone(),
                    external_link: infusion.infused_collection.external_link.clone(),
                    explicit_content: infusion.infused_collection.explicit_content.clone(),
                    start_trading_time: infusion.infused_collection.start_trading_time.clone(),
                    royalty_info: infusion.infused_collection.royalty_info.clone(),
                },
            })?,
        };

        let init_infusion = WasmMsg::Instantiate2 {
            admin: Some(infusion_admin.clone()),
            code_id: cfg.code_id,
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

        // Save the updated vector
        MINTABLE_TOKEN_VECTORS.save(deps.storage, infusion_id, &token_ids)?;

        let infusion_config = InfusionState {
            collections: infusion.collections,
            infused_collection: InfusedCollection {
                addr: Some(infusion_collection_addr_human.to_string()),
                admin: Some(infusion_admin),
                name: infusion.infused_collection.name.clone(),
                symbol: infusion.infused_collection.symbol.clone(),
                base_uri: base_token_uri,
                num_tokens: infusion.infused_collection.num_tokens,
                sg: infusion.infused_collection.sg,
                royalty_info: infusion.infused_collection.royalty_info,
                start_trading_time: infusion.infused_collection.start_trading_time,
                explicit_content: infusion.infused_collection.explicit_content,
                external_link: infusion.infused_collection.external_link,
                image: infusion.infused_collection.image,
                description: infusion.infused_collection.description,
            },
            infusion_params: InfusionParamState {
                mint_fee: infusion.infusion_params.mint_fee,
                params: infusion.infusion_params.params,
                bundle_type: infusion.infusion_params.bundle_type,
            },
            payment_recipient: infusion.payment_recipient.unwrap_or(info.sender.clone()),
            owner: infusion.owner.unwrap_or(info.sender.clone()),
            enabled: true,
        };

        // saves the infusion bundle to state with (infused_collection, id)
        let key = (infusion_collection_addr_human.clone(), infusion_id);
        INFUSION.save(deps.storage, key.clone(), &infusion_config)?;
        INFUSION_ID.save(deps.storage, infusion_id, &key)?;
        // contribute to contract randomness
        let mc = MINT_COUNT.load(deps.storage).unwrap_or_default();
        MINT_COUNT.save(deps.storage, &(mc + 1u64))?;
        MINTABLE_NUM_TOKENS.save(
            deps.storage,
            infusion_collection_addr_human.to_string(),
            &infusion.infused_collection.num_tokens,
        )?;
        CONFIG.save(deps.storage, &cfg)?;

        msgs.push(infusion_collection_submsg);
        attrs.push(Attribute::new("infusion-id", infusion_id.to_string()));
    }

    Ok(Response::new()
        .add_submessages(msgs)
        .add_messages(fee_msgs)
        .add_attributes(attrs))
}

/// Performs various validations on an infusions eligilbe collections being set. If triggered via config update,
/// we validate any possible conficts from existing store values with ones to_add.
fn validate_eligible_collection_list(
    query: QuerierWrapper,
    cfg: &Config,
    bundle_type: &BundleType,
    existing: &Vec<NFTCollection>,
    to_add: &Vec<NFTCollection>,
    updating_config: bool,
) -> Result<Vec<NFTCollection>, ContractError> {
    // checks min_per_bundle
    if cfg.max_bundles < to_add.len() as u64 {
        return Err(ContractError::TooManyCollectionsInInfusion {
            have: to_add.len() as u64,
            max: cfg.max_bundles,
        });
    }

    let mut validate_nfts = to_add.clone();
    if updating_config {
        // prevent duplicates,
        if !to_add.is_empty() {
            let elig = existing;
            let mut elig_to_save = elig.clone();

            for elig in elig_to_save.iter_mut() {
                if let Some(new_elig) = to_add.iter().find(|n| n.addr == elig.addr) {
                    *elig = new_elig.clone();
                }
            }

            for new_elig in to_add {
                if !elig_to_save.iter().any(|e| e.addr == new_elig.addr) {
                    elig_to_save.push(new_elig.clone());
                }
            }

            validate_nfts = elig_to_save;
        }
    }

    // checks for any unique collections
    let mut unique = Vec::new();
    for col in validate_nfts.iter() {
        if unique.contains(&col.addr) {
            return Err(ContractError::DuplicateCollectionInInfusion);
        }
        // check if addr is cw721 collection
        let _res: cw721::ContractInfoResponse = query
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
        if col.min_req < cfg.min_per_bundle
            || col.min_req > cfg.max_per_bundle
            || col.max_req.is_some_and(|a| cfg.max_per_bundle < a)
            || col.max_req.is_some_and(|a| cfg.min_per_bundle > a)
        {
            return Err(ContractError::BadBundle {
                have: col.min_req,
                min: cfg.min_per_bundle,
                max: cfg.max_per_bundle,
            });
        }

        unique.push(col.addr.clone());
    }

    let unique_len = unique.len();
    match bundle_type {
        BundleType::AnyOf { addrs } => {
            if addrs.len() == 0 {
                return Err(ContractError::AnyOfConfigError {
                    err: AnyOfErr::Empty,
                });
            }
            for bc_addr in addrs.iter() {
                if !unique.contains(&bc_addr) {
                    return Err(ContractError::AnyOfConfigError {
                        err: AnyOfErr::Uneligible,
                    });
                }
            }
        }
        BundleType::AnyOfBlend { blends } => {
            for blend in blends.iter() {
                let b_len = blend.blend_nfts.len();

                if b_len == 0 || b_len > unique_len {
                    return Err(ContractError::AnyOfConfigError {
                        err: AnyOfErr::Empty,
                    });
                }
                let mut unique_blend = Vec::new();
                for nfts in blend.blend_nfts.iter() {
                    if nfts.min_req == 0 && !nfts.payment_substitute {
                        return Err(ContractError::AnyOfBlendConfigError);
                    }
                    if !unique.contains(&nfts.addr) {
                        return Err(ContractError::DuplicateCollectionInInfusion);
                    }
                    if unique_blend.contains(&nfts.addr) {
                        return Err(ContractError::DuplicateCollectionInInfusion);
                    } else {
                        unique_blend.push(nfts.addr.clone());
                    }
                }
            }
        }
        BundleType::AllOf {} => {}
    }
    return Ok(validate_nfts.to_vec());
}

/// Creates the msgs that split any fees between the contract owner and an infusion owner, if configured.
fn form_feesplit_helper(
    owner_fee: Decimal,
    owner: String,
    payment_recipient: String,
    fee: Coin,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut msgs = vec![];
    // split fees between contract owner and infusion owner
    let dev_fee = fee.amount * owner_fee;
    let remaining_fee_amount = fee.amount * (Decimal::one() - owner_fee);

    if dev_fee != Uint128::zero() {
        let base_fee = CosmosMsg::Bank(BankMsg::Send {
            to_address: owner,
            amount: vec![coin(dev_fee.into(), fee.denom.clone())],
        });
        msgs.push(base_fee);
    }

    // remaining fee to infusion owner
    let fee_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: payment_recipient.to_string(),
        amount: vec![coin(remaining_fee_amount.into(), fee.denom.clone())],
    });

    msgs.push(fee_msg);
    Ok(msgs)
}
// Infuse bundles. Burns nfts in eligilbe bundles
fn execute_infuse_bundle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    infusion_id: u64,
    bundle: Vec<Bundle>,
) -> Result<Response, ContractError> {
    let res = Response::new();
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    let cfg = CONFIG.load(deps.storage)?;
    let key = INFUSION_ID.load(deps.storage, infusion_id)?;
    let infusion = INFUSION.load(deps.storage, key)?;

    if !infusion.enabled {
        return Err(ContractError::InfusionIsEnded {});
    }

    let mut funds = info.funds.clone();

    // first, any fee parameters are validated
    if let Some(fee) = infusion.infusion_params.mint_fee.clone() {
        if info.sender == infusion.owner {
            // infusion owner omitted from fee payment
        } else {
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
                let fee_msgs = form_feesplit_helper(
                    cfg.owner_fee,
                    cfg.contract_owner.to_string(),
                    infusion.payment_recipient.to_string(),
                    fee,
                )?;
                msgs.extend(fee_msgs);
            }
        }
    }

    // // check lens
    if bundle.is_empty() {
        return Err(ContractError::EmptyBundle);
    }

    let sender = info.sender.clone();
    // for each nft collection bundle sent to infuse
    for bundle in bundle {
        // assert ownership
        is_nft_owner(deps.as_ref(), sender.clone(), bundle.nfts.clone())?;
        // add each burn nft & mint infused token to response
        let burn = burn_bundle(
            &deps,
            env.clone(),
            &cfg,
            bundle.nfts,
            &sender,
            &infusion,
            infusion_id,
            &funds,
        )?;
        println!("burn: {:#?}", burn);
        MINT_COUNT.save(deps.storage, &burn.1)?;
        msgs.extend(burn.0);
    }

    Ok(res.add_messages(msgs))
}

/// checks all bundles nfts, determines how many nfts to mint,
/// returns msgs to burn, mint nfts, & transfer any fee substitute funds to their destination.
fn burn_bundle(
    deps: &DepsMut,
    env: Env,
    cfg: &Config,
    nfts: Vec<NFT>,
    sender: &Addr,
    infusion: &InfusionState,
    infusion_id: u64,
    funds: &Vec<Coin>,
) -> Result<(Vec<CosmosMsg>, u64), ContractError> {
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    // confirm bundle is in current infusion, and expected amount sent
    let mint_num = check_bundles(
        cfg,
        infusion.infusion_params.bundle_type.clone(),
        nfts.clone(),
        infusion.collections.clone(),
        &funds,
        infusion.payment_recipient.to_string(),
    )?;

    println!("mint_num: {:#?}", mint_num);
    for nft in nfts {
        msgs.push(into_cosmos_msg(
            Cw721ExecuteMsg::Burn {
                token_id: nft.token_id.to_string(),
            },
            nft.addr,
            None,
        )?);
    }

    let mut mc = MINT_COUNT.load(deps.storage)?;

    for i in 0..mint_num.1 {
        mc = mc + i + 1;
        // increment tokens
        let token_id = get_next_id(
            &deps,
            env.clone(),
            Addr::unchecked(
                infusion
                    .infused_collection
                    .addr
                    .clone()
                    .expect("no infused collection"),
            ),
            sender.clone(),
            mc,
            infusion_id,
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
                .expect("no infused collection"),
            None,
        )?;
        msgs.push(msg);
    }

    Ok((msgs, mc))
}

/// Checks all parameters of a bundle, returning the number of infused nfts to mint,
/// along with a list of ComosMsgs (contract fees,etc) to append to the response.
fn check_bundles(
    cfg: &Config,
    bundle_type: BundleType,
    bundle: Vec<NFT>,
    eligible: Vec<NFTCollection>,
    sent: &Vec<Coin>,
    payment_rec: String,
) -> Result<(Vec<CosmosMsg>, u64), ContractError> {
    let btype: i32 = bundle_type.strain();
    let mut infused_mint_count = 0u64;
    let mut msgs = Vec::new();
    let mut fee_sub_map = Vec::new();
    let mut total_bundle_map = Vec::new();
    let mut eligible_in_bundle_map = Vec::new();

    for eli in &eligible {
        // println!("eligigble debug: {:#?}", eli);
        let elig = bundle
            .iter()
            .filter(|b| b.addr == eli.addr)
            .collect::<Vec<_>>();
        let elig_len = elig.len() as u64;
        eligible_in_bundle_map.push((eli.addr).clone());

        // 1. payment substitute assertion
        if let Some(ps) = eli.payment_substitute.clone() {
            if elig_len < eli.min_req {
                match btype {
                    // allOf
                    1 => {
                        let count = check_fee_substitute(btype, &eli.addr, &ps, sent)?;
                        if count == 1 {
                            let fee_msgs = form_feesplit_helper(
                                cfg.owner_fee,
                                cfg.contract_owner.to_string(),
                                payment_rec.clone(),
                                ps,
                            )?;
                            msgs.extend(fee_msgs);
                            fee_sub_map.push(eli.addr.to_string());
                            infused_mint_count += count;
                            // println!(
                            //     "mint-count increment: allOf-substitute: {:#?}",
                            //     infused_mint_count
                            // );
                        }
                    }
                    // anyOf & anyOfBlend
                    _ => {
                        let count = check_fee_substitute(btype, &eli.addr, &ps, sent)?;
                        if count != 0u64 {
                            fee_sub_map.push(eli.addr.to_string());
                            let fee_msgs = form_feesplit_helper(
                                cfg.owner_fee,
                                cfg.contract_owner.to_string(),
                                payment_rec.clone(),
                                ps,
                            )?;
                            msgs.extend(fee_msgs);
                            infused_mint_count += count;
                            // println!(
                            //     "mint-count increment: anyOf-substitute: {:#?}",
                            //     infused_mint_count
                            // );
                        }
                    }
                }
            }
            // 2. if no ps, assert allOf bundles error if no eligible nfts were found
        } else if elig.is_empty() {
            if btype == 1 {
                return Err(ContractError::BundleCollectionNotEligilbe {
                    bun_type: btype,
                    col: eli.addr.to_string(),
                });
            }
        }

        // 4. Check for AnyOf & AnyOfBlend are satisfied, or if Allof has
        if elig_len != eli.min_req {
            match bundle_type {
                BundleType::AllOf {} => {
                    if !fee_sub_map.contains(&eli.addr.to_string()) {
                        return Err(ContractError::BundleNotAccepted {
                            have: elig_len,
                            want: eli.min_req,
                        });
                    }
                }
                BundleType::AnyOf { addrs: ref any_of } => {
                    if !fee_sub_map.contains(&eli.addr.to_string()) {
                        let mc = check_anyof_bundle_helper(
                            &fee_sub_map,
                            sent.to_vec(),
                            vec![(eli.clone(), elig_len)],
                            any_of.clone(),
                            vec![],
                        )?;
                        infused_mint_count += mc;
                        println!(
                            "infused_mint_count incremented anyOf: {:#?}",
                            infused_mint_count
                        );
                        // save to map outside of elig loop
                        total_bundle_map.push((eli.clone(), elig_len));
                    }
                }
                BundleType::AnyOfBlend { ref blends } => {
                    let mc = check_anyof_bundle_helper(
                        &fee_sub_map,
                        sent.to_vec(),
                        vec![(eli.clone(), elig_len)],
                        vec![],
                        blends.to_vec(),
                    )?;
                    infused_mint_count += mc;
                    // println!(
                    //     "infused_mint_count incremented anyOfBlend: {:#?}",
                    //     infused_mint_count
                    // );
                    total_bundle_map.push((eli.clone(), elig_len));
                }
            }
        } else {
            match bundle_type {
                BundleType::AnyOf { addrs: ref any_of } => {
                    total_bundle_map.push((eli.clone(), elig_len));
                    let mc = check_anyof_bundle_helper(
                        &fee_sub_map,
                        sent.to_vec(),
                        vec![(eli.clone(), elig_len)],
                        any_of.clone(),
                        vec![],
                    )?;
                    infused_mint_count += mc;
                    // println!(
                    //     "infused_mint_count incremented anyOf: {:#?}",
                    //     infused_mint_count
                    // );
                }
                BundleType::AnyOfBlend { ref blends } => {
                    check_anyof_bundle_helper(
                        &fee_sub_map,
                        sent.to_vec(),
                        vec![(eli.clone(), elig_len)],
                        vec![],
                        blends.to_vec(),
                    )?;
                }
                BundleType::AllOf {} => {}
            }
        }
    }

    for bun in bundle {
        if !eligible_in_bundle_map.contains(&bun.addr) {
            return Err(ContractError::NftIsNotEligible {
                col: bun.addr.to_string(),
            });
        }
    }

    match btype {
        1 => {
            if infused_mint_count != 1 {
                infused_mint_count = 1;
                // println!("allOf bundle validated. mint count set to: {:#?}", infused_mint_count);
            }
        }
        _ => {}
    }

    Ok((msgs, infused_mint_count))
}

fn check_anyof_bundle_helper(
    anyof_map: &Vec<String>,
    sent: Vec<Coin>,
    elig_count: Vec<(NFTCollection, u64)>,
    anyof_list: Vec<Addr>,
    anyofblend: Vec<BundleBlend>,
) -> Result<u64, ContractError> {
    let mut mc = 0u64;
    let mut error = ContractError::UnTriggered;
    // println!("elig_count: {:#?}", elig_count);
    // println!("anyof: {:#?}", anyof_list);
    // println!("sent: {:#?}", sent);

    if !anyof_list.is_empty() {
        for any in anyof_list.iter() {
            println!("mc before: {:#?}", mc);
            for elig in elig_count.iter() {
                //  only occurs once for each entry in both
                if any == elig.0.addr {
                    if let Some(fps) = &elig.0.payment_substitute {
                        let res = sent.iter().find(|coin| coin.denom == fps.denom);
                        if let Some(fee) = res {
                            if fee.amount != fps.amount {
                                if !anyof_map.contains(&any.to_string()) {
                                    error = ContractError::PaymentSubstituteNotProvided {
                                        col: elig.0.addr.to_string(),
                                        haved: fee.denom.to_string(),
                                        havea: fee.amount.to_string(),
                                        wantd: fps.denom.to_string(),
                                        wanta: fps.amount.to_string(),
                                    };
                                    continue;
                                } else {
                                    //  feepayment substitute has been provided for this eligible collection. increment mint count.
                                    mc = mc + 1;
                                    continue;
                                }
                            } else {
                                mc = mc + 1;
                                continue;
                            }
                        }
                    } else if elig.0.min_req > elig.1 {

                        // error = ContractError::NotEnoughNFTsInBundle {
                        //     col: elig.0.addr.to_string(),
                        //     have: elig.1,
                        //     min: elig.0.min_req,
                        //     max: elig.0.max_req.unwrap_or(elig.0.min_req),
                        // };
                    } else {
                        mc = mc + 1;
                    }
                }
            }
        }
    }
    // println!("error: {:#?}", error);
    // println!("mc after: {:#?}", mc);

    if anyofblend != vec![] {
        //  todo:anyofBlendLogic
        return Err(ContractError::UnImplemented);
    }
    if error.to_string() != ContractError::UnTriggered.to_string() {
        return Err(error);
    };

    Ok(mc)
}

/// Checks if sent tokens contains correct payment substitute for a given elig_adddr.\
/// Returns the number of nfts to mint for a given fee substitute.\
/// Bundle types 2 & 3 return 0 if fee-sub not satisfied, bundle type 1 returns error.
fn check_fee_substitute(
    btype: i32,
    elig_adddr: &Addr,
    ps: &Coin,
    sent: &Vec<Coin>,
) -> Result<u64, ContractError> {
    // Calculate the total number of whole divisions of the sent amounts by the required amount
    let mint_count = sent
        .iter()
        .filter(|coin| coin.denom == ps.denom)
        .map(|coin| coin.amount / ps.amount)
        .sum::<Uint128>();

    // println!("mint_count:  {:#?}", mint_count);

    // If no matching denominations found, set mint_count to 0
    let has_matching_denomination = sent.iter().any(|coin| coin.denom == ps.denom);
    if !has_matching_denomination {
        match btype {
            1 => {
                // Collect all relevant coins for the error message
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
                    col: elig_adddr.to_string(),
                    haved,
                    havea: havea.to_string(),
                    wantd: ps.denom.to_string(),
                    wanta: ps.amount.to_string(),
                });
            }
            2 => return Ok(0),
            3 => return Ok(0),
            _ => return Err(ContractError::UnImplemented),
        }
    }

    // Ensure at least one whole division can be made
    if mint_count == Uint128::zero() {
        match btype {
            1 => {
                // Collect all relevant coins for the error message
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
                    col: elig_adddr.to_string(),
                    haved,
                    havea: havea.to_string(),
                    wantd: ps.denom.to_string(),
                    wanta: ps.amount.to_string(),
                });
            }
            2 => return Ok(0),
            3 => return Ok(0),
            _ => return Err(ContractError::UnImplemented),
        }
    }

    // Return the calculated mint_count
    Ok(mint_count.u128() as u64)
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
    mint_count: u64,
    infusion_id: u64,
) -> Result<TokenPositionMapping, ContractError> {
    let mintable_num_tokens =
        MINTABLE_NUM_TOKENS.load(deps.storage, infused_col_addr.to_string())?;
    if mintable_num_tokens == 0 {
        return Err(ContractError::SoldOut {});
    }

    let mintable_token_mapping = random_mintable_token_mapping(
        deps.as_ref(),
        env,
        sender.clone(),
        mintable_num_tokens,
        mint_count,
        infusion_id,
    )?;

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

pub fn random_token_list(
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
    mint_count: u64,
    infusion_id: u64,
) -> Result<TokenPositionMapping, ContractError> {
    let tx_index = if let Some(tx) = &env.transaction {
        tx.index
    } else {
        0
    };
    //generate unique SHA256 sum
    let sha256 = Sha256::digest(
        format!(
            "{}{}{}{}{}",
            sender, num_tokens, env.block.height, tx_index, mint_count
        )
        .into_bytes(),
    );
    // Cut first 16 bytes from 32 byte value
    let randomness: [u8; 16] = sha256.to_vec()[0..16].try_into().unwrap();

    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);

    let r = rng.next_u32();

    let mut rem = 50;
    if rem > num_tokens {
        rem = num_tokens;
    }

    let n = r % rem;
    let infusion_positions = MINTABLE_TOKEN_VECTORS.load(deps.storage, infusion_id)?;
    let token_id = infusion_positions[n as usize - 1];

    Ok(TokenPositionMapping {
        position: n,
        token_id,
    })
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg: MessageInfo,
    uc: UpdatingConfig,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    // Only the admin should be able to call this
    if config.contract_owner != msg.sender {
        return Err(ContractError::Admin(AdminError::NotAdmin {}));
    }

    if let Some(owner) = uc.contract_owner {
        config.contract_owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(of) = uc.owner_fee {
        config.owner_fee = of;
    }

    if let Some(cf) = uc.min_creation_fee {
        config.min_creation_fee = Some(cf);
    }

    if let Some(mif) = uc.min_infusion_fee {
        config.min_infusion_fee = Some(mif);
    }

    if let Some(mi) = uc.max_infusions {
        config.max_infusions = mi;
    }

    if let Some(mpb) = uc.min_per_bundle {
        config.min_per_bundle = mpb;
    }

    if let Some(mb) = uc.max_bundles {
        config.max_bundles = mb;
    }

    if let Some(ci) = uc.code_id {
        config.code_id = ci;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

//  source: https://github.com/public-awesome/launchpad/blob/main/contracts/minters/vending-minter/src/contract.rs#L1371
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let prev_version = cw2::get_contract_version(deps.storage)?;
    if prev_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }

    let res = Response::new();
    let version: Version = prev_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid current contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid new contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(res);
    }

    #[allow(clippy::cmp_owned)]
    if prev_version.version < "0.3.0".to_string() {
        crate::upgrades::v0_3_0::migrate_contract_owner_fee_type(deps.storage, &env, &msg)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        crate::upgrades::v0_3_0::migrate_infusions_bundle_type(deps.storage)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
    }
    #[allow(clippy::cmp_owned)]
    if prev_version.version < "0.4.0".to_string() {
        crate::upgrades::v0_4_0::patch_mint_count_v040(deps.storage)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
    }

    #[allow(clippy::cmp_owned)]
    if prev_version.version < "0.4.1".to_string() {
        let data = crate::upgrades::v0_4_0::v0410_remove_mint_count_store(deps.storage)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        crate::upgrades::v0_4_0::v0410_add_mint_count_store(deps.storage, env, data)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
    }
    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let event = Event::new("migrate")
        .add_attribute("from_name", prev_version.contract)
        .add_attribute("from_version", prev_version.version)
        .add_attribute("to_name", CONTRACT_NAME)
        .add_attribute("to_version", CONTRACT_VERSION);

    // TODO: MIGRATE u64 to Decimals in config
    Ok(res.add_event(event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_form_feesplit_helper() {
        let owner_fee = Decimal::from_str("0.1").unwrap(); // 10% fee for owner
        let owner = String::from("owner");
        let payment_recipient = String::from("recipient");
        let fee = Coin {
            denom: String::from("uthiol"),
            amount: Uint128::from(1000u128), //
        };

        let result = form_feesplit_helper(owner_fee, owner, payment_recipient, fee)
            .expect("Should not return error");

        // 2 msg: one for  devs and one for fee recipient
        assert_eq!(result.len(), 2);

        // First message should send 300 uinf to owner (30% of 1000)
        let dev_fee_msg = &result[0];
        match dev_fee_msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "owner");
                assert_eq!(amount[0].denom, "uthiol");
                assert_eq!(amount[0].amount, Uint128::from(100u128));
            }
            _ => panic!("First message should be a Bank Send message"),
        }

        // Second message should send 700 uinf to recipient (70% of 1000)
        let fee_msg = &result[1];
        match fee_msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "recipient");
                assert_eq!(amount[0].denom, "uthiol");
                assert_eq!(amount[0].amount, Uint128::from(900u128));
            }
            _ => panic!("Second message should be a Bank Send message"),
        }
    }
}
