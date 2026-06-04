use crate::Cw721SvgContract;
use cosmwasm_std::{from_json, StdError};
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, Coin, CustomMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Response, Timestamp, Uint256,
};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::error::Cw721ContractError;
use cw721::traits::Cw721CustomMsg;
use cw721::traits::Cw721Query;
use cw721::Attribute;
use cw_orch::core::serde_json;
use cw_storage_plus::{Item, Map};
use cw_svg::TemplateSlot;
use cw_svg::TokenParam;
use cw_svg::VariableDef;
use cw_svg::VariableKind;
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

#[cw_serde]
#[derive(Default)]
pub struct SvgMetadata {
    pub params: Vec<TokenParam>,
}

#[cw_serde]
pub struct SvgCollectionMetadata {
    pub seed: Binary,
    pub svg_template: String,
    pub variables: Vec<VariableDef>,
    /// Pre-computed placeholder positions in the SVG template.
    /// Each slot maps a `${varname}` occurrence to its byte offsets and variable index.
    pub template_slots: Vec<TemplateSlot>,
    /// Price tiers for minting. Empty or None = free mint.
    pub price_tiers: Vec<PriceTier>,
    pub total: u64,
    /// Timestamp of mint start. If less than current block, mints available
    pub mint_start_time: Option<Timestamp>,
    pub mint_end_time: Option<Timestamp>,
    /// Optional merkle whitelist contract address. Whitelisted minters bypass fees.
    pub whitelist: Option<String>,
}

impl cw721::traits::Cw721State for SvgCollectionMetadata {}
impl cw721::traits::Cw721CustomMsg for SvgCollectionMetadata {}
impl cw721::traits::ToAttributesState for SvgCollectionMetadata {
    fn to_attributes_state(&self) -> Result<Vec<Attribute>, Cw721ContractError> {
        Ok(vec![Attribute {
            key: "metadata".to_string(),
            value: to_json_binary(self)?,
        }])
    }
}

impl cw721::traits::FromAttributesState for SvgCollectionMetadata {
    fn from_attributes_state(value: &[Attribute]) -> Result<Self, Cw721ContractError> {
        // Find the metadata attribute
        let metadata_attr = value
            .iter()
            .find(|attr| attr.key == "metadata")
            .ok_or_else(|| {
                Cw721ContractError::Std(StdError::msg("Missing 'metadata' attribute".to_string()))
            })?;

        from_json(&metadata_attr.value).map_err(|e| Cw721ContractError::Std(StdError::msg("msg")))
    }
}

impl cw721::traits::StateFactory<SvgCollectionMetadata> for SvgCollectionMetadata {
    fn create(
        &self,
        _deps: cosmwasm_std::Deps,
        _env: &cosmwasm_std::Env,
        _info: Option<&cosmwasm_std::MessageInfo>,
        _current: Option<&SvgCollectionMetadata>,
    ) -> Result<SvgCollectionMetadata, Cw721ContractError> {
        Ok(self.clone())
    }

    fn validate(
        &self,
        _deps: cosmwasm_std::Deps,
        _env: &cosmwasm_std::Env,
        _info: Option<&cosmwasm_std::MessageInfo>,
        _current: Option<&SvgCollectionMetadata>,
    ) -> Result<(), Cw721ContractError> {
        // Add your validation logic here
        if self.svg_template.is_empty() {
            return Err(Cw721ContractError::Std(StdError::msg(
                "SVG template cannot be empty".to_string(),
            )));
        }
        if self.total == 0 {
            return Err(Cw721ContractError::Std(StdError::msg(
                "Total supply must be greater than 0".to_string(),
            )));
        }
        Ok(())
    }
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
    // TransferNft {
    //     recipient: String,
    //     token_id: String,
    // },
    // SendNft {
    //     contract: String,
    //     token_id: String,
    //     msg: Binary,
    // },
    // Approve {
    //     spender: String,
    //     token_id: String,
    //     expires: Option<Expiration>,
    // },
    // Revoke {
    //     spender: String,
    //     token_id: String,
    // },
    // ApproveAll {
    //     operator: String,
    //     expires: Option<Expiration>,
    // },
    // RevokeAll {
    //     operator: String,
    // },
    // UpdateOwnership(cw_ownable::Action),
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

    let denom = expected_denom.unwrap();
    if total_cost.is_zero() {
        return Ok(None);
    }

    // Check the sender sent exactly the right amount
    let sent = info
        .funds
        .iter()
        .find(|c| c.denom == denom)
        .map(|c| c.amount)
        .unwrap_or(Uint256::zero());

    if sent != total_cost {
        return Err(ContractError::IncorrectPayment {
            expected: format!("{}{}", total_cost, denom),
        });
    }

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
    let whitelist = match Cw721SvgContract::default()
        .query_collection_info_and_extension(deps)?
        .extension
        .whitelist
    {
        Some(a) => a,
        None => return Ok(false),
    };

    // Build the member string: sender + allocation
    let member = format!("{}{}", info.sender, allocation);

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

/// Parse a decimal string (e.g. "-2.5", "80", "0.45") into a scaled integer
/// at the given precision. For example: "-2.5" with precision=2 → -250.
pub fn parse_decimal_scaled(s: &str, precision: u32) -> Result<i128, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty string".to_string());
    }

    let (negative, abs_str) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s)
    };

    let parts: Vec<&str> = abs_str.split('.').collect();
    if parts.len() > 2 || parts.is_empty() || parts[0].is_empty() {
        return Err(format!("invalid decimal: {}", s));
    }

    let int_part: i128 = parts[0]
        .parse()
        .map_err(|_| format!("invalid integer part: {}", s))?;

    let frac_digits = if parts.len() == 2 { parts[1] } else { "" };

    let factor = 10i128.pow(precision);
    let mut scaled = int_part * factor;

    if !frac_digits.is_empty() {
        let frac_len = frac_digits.len() as u32;
        let frac_val: i128 = frac_digits
            .parse()
            .map_err(|_| format!("invalid fractional part: {}", s))?;

        if frac_len <= precision {
            scaled += frac_val * 10i128.pow(precision - frac_len);
        } else {
            // More digits than precision — truncate
            scaled += frac_val / 10i128.pow(frac_len - precision);
        }
    }

    if negative {
        scaled = -scaled;
    }

    Ok(scaled)
}

/// Format a scaled integer back into a decimal string.
/// For example: -250 with precision=2 → "-2.50".
fn format_decimal(scaled: i128, precision: u32) -> String {
    if precision == 0 {
        return scaled.to_string();
    }

    let factor = 10i128.pow(precision);
    let (negative, abs_val) = if scaled < 0 {
        (true, -scaled)
    } else {
        (false, scaled)
    };

    let int_part = abs_val / factor;
    let frac_part = abs_val % factor;

    let sign = if negative { "-" } else { "" };
    format!(
        "{}{}.{:0>width$}",
        sign,
        int_part,
        frac_part,
        width = precision as usize
    )
}

/// Generate a decimal string value from entropy within [min, max] at the given precision.
fn value_from_range(entropy: u64, min_scaled: i128, max_scaled: i128) -> i128 {
    let range = (max_scaled - min_scaled) as u128;
    if range == 0 {
        return min_scaled;
    }
    let offset = (entropy as u128) % (range + 1);
    min_scaled + offset as i128
}

/// Derive per-variable entropy via sha256(token_seed || var_idx), returning a u64.
fn derive_entropy(token_seed: &[u8], var_idx: usize) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(token_seed);
    hasher.update((var_idx as u64).to_le_bytes());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[0..8].try_into().unwrap())
}

/// Map a byte into a [min, max] range (inclusive).
fn channel_in_range(byte: u8, min: u8, max: u8) -> u8 {
    if min == max {
        return min;
    }
    let span = (max - min) as u16 + 1;
    min + (byte as u16 % span) as u8
}

/// Resolve a single variable definition into a TokenParam using entropy.
fn resolve_variable(token_seed: &[u8], var_idx: usize, var_def: &VariableDef) -> TokenParam {
    let entropy = derive_entropy(token_seed, var_idx);

    let value = match &var_def.kind {
        VariableKind::Options(options) => {
            let index = (entropy as usize) % options.len();
            options[index].clone()
        }
        VariableKind::Range {
            min,
            max,
            precision,
        } => {
            // These were validated at instantiation time, so unwrap is safe.
            let min_scaled = parse_decimal_scaled(min, *precision).unwrap();
            let max_scaled = parse_decimal_scaled(max, *precision).unwrap();
            let scaled = value_from_range(entropy, min_scaled, max_scaled);
            format_decimal(scaled, *precision)
        }
        VariableKind::Rgb => {
            let bytes = entropy.to_le_bytes();
            format!("rgb({},{},{})", bytes[0], bytes[1], bytes[2])
        }
        VariableKind::RgbStyled(ranges) => {
            let bytes = entropy.to_le_bytes();
            // Use upper bytes to select range, lower bytes for channel values
            let range_idx = (bytes[3] as usize) % ranges.len();
            let range = &ranges[range_idx];
            let r = channel_in_range(bytes[0], range.r_min, range.r_max);
            let g = channel_in_range(bytes[1], range.g_min, range.g_max);
            let b = channel_in_range(bytes[2], range.b_min, range.b_max);
            format!("rgb({},{},{})", r, g, b)
        }
    };

    TokenParam {
        name: var_def.name.clone(),
        value,
    }
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: u64,
    proof_hashes: Option<Vec<String>>,
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
    let mint_count = c.config.token_count(deps.storage)?;

    if env.block.time < config.mint_start_time.unwrap_or_default() {
        return Err(ContractError::MintingNotStarted {});
    }

    if let Some(end_time) = config.mint_end_time {
        if env.block.time > end_time {
            return Err(ContractError::MintingEnded {});
        }
    }

    if mint_count + amount > config.total {
        return Err(ContractError::CannotMintMoreThanTotal {});
    }

    // Check whitelist — if verified, bypass payment
    let is_whitelisted = match proof_hashes {
        Some(ph) => verify_whitelist(deps.as_ref(), &info, ph, allocation, amount)?,
        None => false,
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

    let variables = VARIABLES.load(deps.storage)?;

    let mut events = vec![];

    for i in 0..amount {
        let tid = mint_count + i;
        let token_id = tid.to_string();

        // Generate per-token entropy: sha256(tid || seed || block_height)
        let mut hasher = Sha256::new();
        hasher.update(tid.to_le_bytes());
        hasher.update(config.seed.as_slice());
        hasher.update(env.block.height.to_le_bytes());
        let token_seed = hasher.finalize();

        // Resolve each variable using entropy
        let params: Vec<TokenParam> = variables
            .iter()
            .enumerate()
            .map(|(var_idx, var_def)| resolve_variable(&token_seed, var_idx, var_def))
            .collect();

        let extension = SvgMetadata { params };

        let token = cw721::state::NftInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            token_uri: None,
            extension,
        };

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

        Cw721SvgContract::default()
            .config
            .increment_tokens(deps.storage)?;

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
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    match PAUSED.may_load(deps.storage)? {
        Some(_) => PAUSED.remove(deps.storage),
        None => PAUSED.save(deps.storage, &())?,
    }
    Ok(Response::new()
        .add_attribute("action", "pause")
        .add_attribute("paused", pause.to_string()))
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
        .map(|(var_idx, var_def)| resolve_variable(&token_seed, var_idx, var_def))
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

/// Validate all variable definitions in isolation (options non-empty, range
/// parseable / ordered, rgb_styled ranges non-empty with min <= max).
/// This is the same check that runs inside `instantiate`.
pub fn validate_variables(variables: &[VariableDef]) -> Result<(), ContractError> {
    for var in variables {
        match &var.kind {
            VariableKind::Options(opts) => {
                if opts.is_empty() {
                    return Err(ContractError::InvalidVariableDef {
                        reason: format!("variable '{}': options list must not be empty", var.name),
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
            VariableKind::Rgb => {}
            VariableKind::RgbStyled(ranges) => {
                if ranges.is_empty() {
                    return Err(ContractError::InvalidVariableDef {
                        reason: format!(
                            "variable '{}': rgb_styled ranges list must not be empty",
                            var.name
                        ),
                    });
                }
                for (i, range) in ranges.iter().enumerate() {
                    if range.r_min > range.r_max
                        || range.g_min > range.g_max
                        || range.b_min > range.b_max
                    {
                        return Err(ContractError::InvalidVariableDef {
                            reason: format!(
                                "variable '{}': range {} has min > max for a channel",
                                var.name, i
                            ),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn validate_template_slots(
    template: &str,
    variables: &[VariableDef],
    slots: &[TemplateSlot],
) -> Result<(), ContractError> {
    let tpl_len = template.len() as u32;
    let var_count = variables.len() as u16;
    let mut prev_end: u32 = 0;

    for (i, slot) in slots.iter().enumerate() {
        // Bounds check
        if slot.start >= slot.end || slot.end > tpl_len {
            return Err(ContractError::InvalidTemplatePlaceholder {
                reason: format!(
                    "slot {} out of bounds: start={}, end={}, template_len={}",
                    i, slot.start, slot.end, tpl_len
                ),
            });
        }

        // Slots must be sorted and non-overlapping
        if slot.start < prev_end {
            return Err(ContractError::InvalidTemplatePlaceholder {
                reason: format!(
                    "slot {} overlaps or is out of order: start={} < prev_end={}",
                    i, slot.start, prev_end
                ),
            });
        }
        prev_end = slot.end;

        // var_idx must reference a valid variable
        if slot.var_idx >= var_count {
            return Err(ContractError::InvalidTemplatePlaceholder {
                reason: format!(
                    "slot {} var_idx {} out of range (only {} variables)",
                    i, slot.var_idx, var_count
                ),
            });
        }

        // Verify the template actually contains ${varname} at this position
        let start = slot.start as usize;
        let end = slot.end as usize;
        let expected = format!("${{{}}}", variables[slot.var_idx as usize].name);
        let actual = &template[start..end];
        if actual != expected {
            return Err(ContractError::InvalidTemplatePlaceholder {
                reason: format!(
                    "slot {} mismatch at [{}, {}): expected '{}', found '{}'",
                    i, start, end, expected, actual
                ),
            });
        }
    }

    Ok(())
}

pub fn execute_update_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    address: Option<String>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    match address {
        Some(addr) => {
            let validated = deps.api.addr_validate(&addr)?;
            Cw721SvgContract::default()
                .config
                .collection_extension
                .save(
                    deps.storage,
                    "whitelist".into(),
                    &Attribute {
                        key: "whitelist".into(),
                        value: to_json_binary(&validated)?,
                    },
                )?;
            Ok(Response::new()
                .add_attribute("action", "update_whitelist")
                .add_attribute("whitelist", validated.to_string()))
        }
        None => {
            Cw721SvgContract::default()
                .config
                .collection_extension
                .save(
                    deps.storage,
                    "whitelist".into(),
                    &Attribute {
                        key: "whitelist".into(),
                        value: Binary::default(),
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

pub use error::ContractError;
mod error {
    use cosmwasm_std::StdError;
    use cw_ownable::OwnershipError;
    use cw_utils::PaymentError;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ContractError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        OwnershipError(#[from] OwnershipError),

        #[error("{0}")]
        Payment(#[from] PaymentError),

        #[error("{0}")]
        Base(#[from] cw721::error::Cw721ContractError),

        #[error("Minting is paused")]
        MintingPaused {},

        #[error("Please wrap the SvgExecuteMsgExt::Mint inside the default ExecuteMsg::UpdateExtension option.")]
        IncorrectEntrypoint,

        #[error("Minting has not started yet")]
        MintingNotStarted {},

        #[error("Minting has not started yet")]
        PricingTierError {},

        #[error("Cannot mint more than total supply")]
        CannotMintMoreThanTotal {},

        #[error("Unauthorized")]
        Unauthorized {},

        #[error("SVG template exceeds max size of {max} bytes (got {got})")]
        SvgTemplateTooLarge { max: usize, got: usize },

        #[error("Total supply {got} exceeds maximum of {max}")]
        TotalSupplyTooHigh { max: u64, got: u64 },

        #[error("Minting period has ended")]
        MintingEnded {},

        #[error("Incorrect payment: expected {expected}")]
        IncorrectPayment { expected: String },

        #[error("Proof hashes required for whitelist verification")]
        MissingProofHashes {},

        #[error("Address {addr} is not whitelisted")]
        NotWhitelisted { addr: String },

        #[error("No whitelist contract configured")]
        NoWhitelistConfigured {},

        #[error("{e}")]
        General { e: String },

        #[error("Whitelist per-address mint limit exceeded")]
        MaxPerAddressLimitExceeded {},

        #[error("Invalid variable definition: {reason}")]
        InvalidVariableDef { reason: String },

        #[error("Invalid template placeholder: {reason}")]
        InvalidTemplatePlaceholder { reason: String },
    }
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
