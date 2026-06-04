use crate::state::SvgCollectionMetadata;
use crate::{ContractError, Cw721SvgContract, SvgMetadata};

use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CustomMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Response, Uint256,
};

use cosmwasm_schema::{cw_serde, QueryResponses};

use cw721::msg::CollectionInfoMsg;
use cw721::traits::Cw721Query;
use cw721::traits::{Cw721CustomMsg, Cw721Execute};
use cw721::Attribute;
use cw_storage_plus::{Item, Map};
use cw_svg::TemplateSlot;
use cw_svg::TokenParam;
use cw_svg::VariableDef;

use sha2::{Digest, Sha256};

// 0.043 MB limit
pub const MAX_SVG_SIZE: usize = 42 * 1024;
pub const MAX_TOTAL_SUPPLY: u64 = 10_000;
pub const PAUSED: Item<()> = Item::new("svg_paused");
pub const SVG_TEMPLATE: Item<String> = Item::new("svg_template");
pub const VARIABLES: Item<Vec<VariableDef>> = Item::new("variables");
pub const TEMPLATE_SLOTS: Item<Vec<TemplateSlot>> = Item::new("template_slots");
// TODO: utilize cw721 state trait to implement this somehow
pub const MINTER_ADDRS: Map<&Addr, u32> = Map::new("minter_addrs");
pub const WL_MINTER_ADDRS: Map<&Addr, u32> = Map::new("wl_minter_addrs");

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
    proof_hashes: Vec<String>,
    allocation: u32,
) -> Result<Response, ContractError> {
    match PAUSED.may_load(deps.storage)? {
        Some(_) => return Err(ContractError::MintingPaused {}),
        None => {}
    }
    let c = Cw721SvgContract::default();
    let config = c
        .query_collection_info_and_extension(deps.as_ref())?
        .extension;
    println!("queried collection info and extension");
    let mint_count = c.query_num_tokens(deps.storage)?.count;
    println!("queried num tokens");

    if env.block.time < config.mint_start_time.unwrap_or_default() {
        return Err(ContractError::MintingNotStarted {});
    }
    println!("checked start");

    if let Some(end_time) = config.mint_end_time {
        if env.block.time > end_time {
            return Err(ContractError::MintingEnded {});
        }
    }
    println!("checked end");
    if mint_count + amount > config.total {
        return Err(ContractError::CannotMintMoreThanTotal {});
    }

    println!("checked total");
    // Check whitelist — if verified, bypass payment

    let is_whitelisted = match !proof_hashes.is_empty() {
        true => {
            println!("checking proofs");
            let res = verify_whitelist(deps.as_ref(), &info, proof_hashes, allocation, amount)?;
            println!("whitelist verified: {}", res);
            res
        }
        false => {
            println!("skipped checking proofs");
            false
        }
    };
    // Validate payment (skipped for whitelisted minters)
    let payment_msg = if is_whitelisted {
        None
    } else {
        validate_and_build_payment(
            c.config.withdraw_address.load(deps.storage)?,
            &config,
            &info,
            mint_count,
            amount,
        )?
    };
    println!("pay validated");

    let variables = VARIABLES.load(deps.storage)?;
    let mut events = vec![];
    for i in 0..amount {
        let tid = mint_count + i;
        let token_id = tid.to_string();
        println!("set tokenid");
        // Generate per-token entropy: sha256(tid || seed || block_height)
        let mut hasher = Sha256::new();
        hasher.update(tid.to_le_bytes());
        hasher.update(config.seed.as_slice());
        hasher.update(env.block.height.to_le_bytes());
        let token_seed = hasher.finalize();
        println!("set token_seed");
        // Resolve each variable using entropy
        let params: Vec<TokenParam> = variables
            .iter()
            .enumerate()
            .map(|(var_idx, var_def)| {
                cw_svg::resolve_variable(&token_seed, var_idx, var_def, derive_entropy)
            })
            .collect();
        let extension = SvgMetadata { params };
        println!("resolved ext and vars");

        let token = cw721::state::NftInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            token_uri: None,
            extension,
        };
        println!("resolved nft info");

        Cw721SvgContract::default().config.nft_info.update(
            deps.storage,
            &token_id,
            |old| match old {
                Some(_) => Err(ContractError::Base(
                    cw721::error::Cw721ContractError::Claimed {},
                )),
                None => Ok(token),
            },
        )?;
        println!("saved nft");
        Cw721SvgContract::default()
            .config
            .increment_tokens(deps.storage)?;
        println!("incremented");

        events.push(
            Event::new("mint")
                .add_attribute("token_id", &token_id)
                .add_attribute("owner", info.sender.to_string()),
        );
    }

    // Track per-address mint count
    let prev_count = MINTER_ADDRS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(0);
    MINTER_ADDRS.save(deps.storage, &info.sender, &(prev_count + amount as u32))?;
    // Track whitelist mint count if applicable
    if is_whitelisted {
        let prev_wl = WL_MINTER_ADDRS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or(0);
        WL_MINTER_ADDRS.save(deps.storage, &info.sender, &(prev_wl + amount as u32))?;
    }

    let mut res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender.to_string())
        .add_attribute("amount", amount.to_string());

    if let Some(bank_msg) = payment_msg {
        res = res.add_message(bank_msg);
    }

    for event in events {
        res = res.add_event(event);
    }

    Ok(res)
}

pub fn execute_pause(
    deps: DepsMut,
    info: MessageInfo,
    pause: bool,
) -> Result<Response, ContractError> {
    match Cw721SvgContract::default()
        .query_creator_ownership(deps.storage)?
        .owner
    {
        Some(ow) => match ow == info.sender {
            true => {
                match PAUSED.may_load(deps.storage)? {
                    Some(_) => PAUSED.remove(deps.storage),
                    None => PAUSED.save(deps.storage, &())?,
                }
                Ok(Response::new()
                    .add_attribute("action", "pause")
                    .add_attribute("paused", pause.to_string()))
            }
            false => return Err(ContractError::Unauthorized {}),
        },
        None => return Err(ContractError::Unauthorized {}),
    }
}

/// Find the price per token for the current mint count from the tier list.
/// Returns None if no tiers are configured (free mint).
pub fn current_tier_price(tiers: &[PriceTier], mint_count: u64) -> Option<&Coin> {
    tiers
        .iter()
        .find(|tier| mint_count < tier.until_count)
        .map(|tier| &tier.price)
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

/// Query message for the merkle whitelist contract.
/// Must match the whitelist contract's QueryMsg::HasMember variant.
#[cw_serde]
pub enum WhitelistHasMemberMsg {
    HasMember {
        member: String,
        proof_hashes: Vec<String>,
    },
}

#[cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum SvgExecuteMsgExt {
    Mint(cw_svg::MintMsg),
    Pause {
        pause: bool,
    },
    /// Owner-only: set or clear the merkle whitelist contract address
    UpdateWhitelist {
        /// Set to Some(addr) to enable whitelist, None to disable
        address: Option<String>,
    },
}

#[cw_ownable::cw_ownable_query]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cw_serde]
#[derive(QueryResponses)]
pub enum SvgQueryMsgExt {
    #[returns(SvgTokenUriResponse)]
    SvgTokenUri { token_id: String },
    /// Returns a preview SVG with random placeholder values filled in.
    #[returns(SvgTokenUriResponse)]
    SvgPlaceholder { seed: Option<String> },
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
    #[returns(PriceTier)]
    CurrentPriceTier {},
}

impl CustomMsg for SvgQueryMsgExt {}
impl Cw721CustomMsg for SvgQueryMsgExt {}

#[cw_serde]
pub struct MigrateMsg {}

/// Response from the whitelist HasMember query
#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
pub struct SvgTokenUriResponse {
    pub svg: String,
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

/// Validate that the sent funds match the required payment for `amount` tokens.
/// Returns the total cost as a BankMsg to forward to the payment address.
fn validate_and_build_payment(
    payment_address: String,
    config: &SvgCollectionMetadata,
    info: &MessageInfo,
    mint_count: u64,
    amount: u64,
) -> Result<Option<BankMsg>, ContractError> {
    // If no price tiers, mint is free — require no funds
    if config.price_tiers.is_empty() {
        if !info.funds.is_empty() {
            return Err(ContractError::IncorrectPayment {
                expected: "free mint (no funds)".into(),
            });
        }
        return Ok(None);
    }

    // Calculate total cost across all tokens being minted.
    // Each token might span different tiers.
    let mut total_cost: Uint256 = Uint256::zero();
    let mut expected_denom: Option<String> = None;

    println!("checking payment");
    for i in 0..amount {
        let count_at = mint_count + i;
        let price = current_tier_price(&config.price_tiers, count_at)
            .ok_or(ContractError::CannotMintMoreThanTotal {})?;

        // All tiers must use the same denom
        match &expected_denom {
            Some(d) => {
                if d != &price.denom {
                    return Err(ContractError::IncorrectPayment {
                        expected: format!("all tiers must use denom {}", d),
                    });
                }
            }
            None => expected_denom = Some(price.denom.clone()),
        }

        total_cost += price.amount;
    }
    println!("checked payment");
    let denom = expected_denom.unwrap();
    println!("unwrapped denom");
    if total_cost.is_zero() {
        return Ok(None);
    }
    println!("checking funds");

    // Check the sender sent exactly the right amount
    let sent = info
        .funds
        .iter()
        .find(|c| c.denom == denom)
        .map(|c| c.amount)
        .unwrap_or(Uint256::zero());

    if sent != total_cost {
        return Err(ContractError::IncorrectPayment {
            expected: format!("{}{} got {}", total_cost, denom, sent),
        });
    }
    println!("checked funds");

    Ok(Some(BankMsg::Send {
        to_address: payment_address,
        amount: vec![Coin {
            denom,
            amount: total_cost,
        }],
    }))
}

/// Verify that the sender is on the merkle whitelist and has not exceeded their mint allocation.
/// Returns true if whitelisted (fees should be bypassed), false if no whitelist check was performed.
///
/// When `allocation` is Some, the merkle leaf is `hash(sender || allocation)` and the
/// address may mint up to `allocation` tokens via whitelist. When None, the leaf is
/// just `hash(sender)` and the whitelist contract's `per_address_limit` is used.
fn verify_whitelist(
    deps: Deps,
    info: &MessageInfo,
    proof_hashes: Vec<String>,
    allocation: u32,
    mint_amount: u64,
) -> Result<bool, ContractError> {
    println!("queriing whitelist");
    let whitelist = match Cw721SvgContract::default()
        .query_collection_info_and_extension(deps)?
        .extension
        .whitelist
    {
        Some(a) => a,
        None => {
            println!("no whitelist");
            return Ok(false)}
    };

    // Build the member string: sender + allocation
    println!("checking membership");
    let member = format!("{}{}", info.sender.to_string(), allocation);
    println!("{:#?}", member);
    let res: HasMemberResponse = deps.querier.query_wasm_smart(
        whitelist.clone(),
        &WhitelistHasMemberMsg::HasMember {
            member,
            proof_hashes,
        },
    )?;

    if !res.has_member {
        return Err(ContractError::NotWhitelisted {
            addr: info.sender.to_string(),
        });
    }

    // Determine max mints allowed for this address
    // Enforce per-address whitelist mint limit
    let current_wl_count = whitelist_mint_count(deps, &info.sender);
    if current_wl_count + mint_amount as u32 > allocation {
        return Err(ContractError::MaxPerAddressLimitExceeded {});
    }

    Ok(true)
}

/// Get the total mint count for an address.
pub fn mint_count(deps: Deps, addr: &Addr) -> u32 {
    MINTER_ADDRS
        .may_load(deps.storage, addr)
        .unwrap_or(None)
        .unwrap_or(0)
}

/// Get the whitelist mint count for an address.
pub fn whitelist_mint_count(deps: Deps, addr: &Addr) -> u32 {
    WL_MINTER_ADDRS
        .may_load(deps.storage, addr)
        .unwrap_or(None)
        .unwrap_or(0)
}

/// Derive per-variable entropy via sha256(token_seed || var_idx), returning a u64.
fn derive_entropy(token_seed: &[u8], var_idx: usize) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(token_seed);
    hasher.update((var_idx as u64).to_le_bytes());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[0..8].try_into().unwrap())
}

pub fn query_svg_token_uri(
    deps: cosmwasm_std::Deps,
    token_id: String,
) -> Result<String, ContractError> {
    let template = SVG_TEMPLATE.load(deps.storage)?;
    let slots = TEMPLATE_SLOTS.load(deps.storage)?;
    let token = Cw721SvgContract::default()
        .config
        .nft_info
        .load(deps.storage, &token_id)?;

    let mut result = String::with_capacity(template.len());
    let mut cursor = 0usize;

    for slot in &slots {
        let start = slot.start as usize;
        let end = slot.end as usize;
        result.push_str(&template[cursor..start]);
        result.push_str(&token.extension.params[slot.var_idx as usize].value);
        cursor = end;
    }
    result.push_str(&template[cursor..]);

    Ok(result)
}

pub fn query_svg_placeholder(
    deps: cosmwasm_std::Deps,
    seed: Option<String>,
) -> Result<String, ContractError> {
    let template = SVG_TEMPLATE.load(deps.storage)?;
    let slots = TEMPLATE_SLOTS.load(deps.storage)?;
    let variables = VARIABLES.load(deps.storage)?;

    let seed_bytes = seed.unwrap_or_else(|| "placeholder".to_string());
    let mut hasher = Sha256::new();
    hasher.update(seed_bytes.as_bytes());
    let token_seed = hasher.finalize();

    let params: Vec<TokenParam> = variables
        .iter()
        .enumerate()
        .map(|(var_idx, var_def)| {
            cw_svg::resolve_variable(&token_seed, var_idx, var_def, derive_entropy)
        })
        .collect();

    let mut result = String::with_capacity(template.len());
    let mut cursor = 0usize;

    for slot in &slots {
        let start = slot.start as usize;
        let end = slot.end as usize;
        result.push_str(&template[cursor..start]);
        result.push_str(&params[slot.var_idx as usize].value);
        cursor = end;
    }
    result.push_str(&template[cursor..]);

    Ok(result)
}

pub fn query_config(
    deps: cosmwasm_std::Deps,
) -> Result<cw721::msg::CollectionInfoAndExtensionResponse<SvgCollectionMetadata>, ContractError> {
    Ok(Cw721SvgContract::default().query_collection_info_and_extension(deps)?)
}

pub fn query_svg_template(deps: cosmwasm_std::Deps) -> Result<String, ContractError> {
    Ok(SVG_TEMPLATE.load(deps.storage)?)
}

pub fn execute_update_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    address: Option<String>,
) -> Result<Response, ContractError> {
    let contract = Cw721SvgContract::default();
    let mut config = contract.query_collection_info_and_extension(deps.as_ref())?;
    match contract.query_creator_ownership(deps.storage)?.owner {
        Some(own) => match &info.sender == own {
            true => {}
            false => return Err(ContractError::Unauthorized {}),
        },
        None => return Err(ContractError::Unauthorized {}),
    };

    match address {
        Some(addr) => {
            let validated = deps.api.addr_validate(&addr)?;
            config.extension.whitelist = Some(validated.to_string());
            contract.update_collection_info(
                deps,
                Some(&info),
                &env,
                CollectionInfoMsg {
                    name: Some(config.name),
                    symbol: Some(config.symbol),
                    extension: config.extension,
                },
            )?;
            Ok(Response::new()
                .add_attribute("action", "update_whitelist")
                .add_attribute("whitelist", validated.to_string()))
        }
        None => {
            config.extension.whitelist = None;
            contract.update_collection_info(
                deps,
                Some(&info),
                &env,
                CollectionInfoMsg {
                    name: Some(config.name),
                    symbol: Some(config.symbol),
                    extension: config.extension,
                },
            )?;
            Ok(Response::new()
                .add_attribute("action", "update_whitelist")
                .add_attribute("whitelist", "removed"))
        }
    }
}

pub fn query_whitelist(deps: cosmwasm_std::Deps) -> Result<Option<String>, ContractError> {
    Ok(Cw721SvgContract::default()
        .query_collection_info_and_extension(deps)?
        .extension
        .whitelist)
}

#[cfg(feature = "interface")]
pub use interface::Cw721SvgContractSuite;
#[cfg(feature = "interface")]
mod interface {
    use crate::{
        entry::{execute, instantiate, query},
        CONTRACT_NAME, *,
    };
    use cw_orch::prelude::*;

    #[cw_orch::interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id = CONTRACT_NAME)]
    pub struct Cw721SvgContractSuite;

    impl<Chain: CwEnv> Uploadable for Cw721SvgContractSuite<Chain> {
        /// Return the path to the wasm file corresponding to the contract
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path_from_crates_label(CONTRACT_NAME)
                .unwrap()
        }
        /// Returns a CosmWasm contract wrapper
        fn wrapper() -> Box<dyn MockContract<Empty>> {
            Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
        }
    }
}
