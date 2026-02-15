use crate::error::ContractError;
use crate::msg::{
    HasMemberResponse, MintConfig, PriceTier, SvgMetadata, TokenParam, VariableDef, VariableKind,
    WhitelistHasMemberMsg,
};
use crate::state::{
    MINTER_ADDRS, MINT_CONFIG, SVG_TEMPLATE, VARIABLES, WHITELIST, WL_MINTER_ADDRS,
};
use crate::Cw721SvgContract;
use cosmwasm_std::{
    Addr, BankMsg, Coin, Deps, DepsMut, Env, Event, MessageInfo, Response, Uint128,
};
use cw721_base::state::TokenInfo;
use sha2::{Digest, Sha256};
use whitelist_mtree::msg::{ConfigResponse as WhitelistConfigRes, QueryMsg as WlistQueryMsg};

/// Find the price per token for the current mint count from the tier list.
/// Returns None if no tiers are configured (free mint).
fn current_tier_price(tiers: &[PriceTier], mint_count: u64) -> Option<&Coin> {
    tiers
        .iter()
        .find(|tier| mint_count < tier.until_count)
        .map(|tier| &tier.price)
}

/// Validate that the sent funds match the required payment for `amount` tokens.
/// Returns the total cost as a BankMsg to forward to the payment address.
fn validate_and_build_payment(
    config: &MintConfig,
    info: &MessageInfo,
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
    let mut total_cost: Uint128 = Uint128::zero();
    let mut expected_denom: Option<String> = None;

    for i in 0..amount {
        let count_at = config.mint_count + i;
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
        .unwrap_or(Uint128::zero());

    if sent != total_cost {
        return Err(ContractError::IncorrectPayment {
            expected: format!("{}{}", total_cost, denom),
        });
    }

    Ok(Some(BankMsg::Send {
        to_address: config.payment_address.to_string(),
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
    deps: &Deps,
    info: &MessageInfo,
    proof_hashes: Vec<String>,
    allocation: Option<u32>,
    mint_amount: u64,
) -> Result<bool, ContractError> {
    let whitelist = match WHITELIST.may_load(deps.storage)? {
        Some(addr) => addr,
        None => {
            // No whitelist configured — no bypass
            return Ok(false);
        }
    };

    // Build the member string: sender + allocation if provided
    let member = match allocation {
        Some(alloc) => format!("{}{}", info.sender, alloc),
        None => info.sender.to_string(),
    };

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
    let max_count: u32 = match allocation {
        Some(alloc) => alloc,
        None => {
            let wl_config: WhitelistConfigRes = deps
                .querier
                .query_wasm_smart(whitelist, &WlistQueryMsg::Config {})?;
            wl_config.per_address_limit
        }
    };

    // Enforce per-address whitelist mint limit
    let current_wl_count = whitelist_mint_count(*deps, &info.sender);
    if current_wl_count + mint_amount as u32 > max_count {
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
    allocation: Option<u32>,
) -> Result<Response, ContractError> {
    let mut config = MINT_CONFIG.load(deps.storage)?;

    if config.paused {
        return Err(ContractError::MintingPaused {});
    }

    if env.block.time < config.mint_start_time {
        return Err(ContractError::MintingNotStarted {});
    }

    if let Some(end_time) = config.mint_end_time {
        if env.block.time > end_time {
            return Err(ContractError::MintingEnded {});
        }
    }

    if config.mint_count + amount > config.total {
        return Err(ContractError::CannotMintMoreThanTotal {});
    }

    // Check whitelist — if verified, bypass payment
    let is_whitelisted = match proof_hashes {
        Some(ph) => verify_whitelist(&deps.as_ref(), &info, ph, allocation, amount)?,
        None => false,
    };

    // Validate payment (skipped for whitelisted minters)
    let payment_msg = if is_whitelisted {
        None
    } else {
        validate_and_build_payment(&config, &info, amount)?
    };

    let variables = VARIABLES.load(deps.storage)?;

    let mut events = vec![];

    for i in 0..amount {
        let tid = config.mint_count + i;
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

        let token = TokenInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            token_uri: None,
            extension,
        };

        Cw721SvgContract::default()
            .tokens
            .update(deps.storage, &token_id, |old| match old {
                Some(_) => Err(ContractError::Base(cw721_base::ContractError::Claimed {})),
                None => Ok(token),
            })?;

        Cw721SvgContract::default().increment_tokens(deps.storage)?;

        events.push(
            Event::new("mint")
                .add_attribute("token_id", &token_id)
                .add_attribute("owner", info.sender.to_string()),
        );
    }

    config.mint_count += amount;
    MINT_CONFIG.save(deps.storage, &config)?;

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

    MINT_CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        config.paused = pause;
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "pause")
        .add_attribute("paused", pause.to_string()))
}

pub fn query_svg_token_uri(
    deps: cosmwasm_std::Deps,
    token_id: String,
) -> Result<String, ContractError> {
    let template = SVG_TEMPLATE.load(deps.storage)?;
    let token = Cw721SvgContract::default()
        .tokens
        .load(deps.storage, &token_id)?;

    let mut svg = template;
    for param in &token.extension.params {
        let placeholder = format!("${{{}}}", param.name);
        svg = svg.replace(&placeholder, &param.value);
    }

    Ok(svg)
}

pub fn query_config(deps: cosmwasm_std::Deps) -> Result<crate::msg::MintConfig, ContractError> {
    Ok(MINT_CONFIG.load(deps.storage)?)
}

pub fn query_svg_template(deps: cosmwasm_std::Deps) -> Result<String, ContractError> {
    Ok(SVG_TEMPLATE.load(deps.storage)?)
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
            WHITELIST.save(deps.storage, &validated)?;
            Ok(Response::new()
                .add_attribute("action", "update_whitelist")
                .add_attribute("whitelist", validated.to_string()))
        }
        None => {
            WHITELIST.remove(deps.storage);
            Ok(Response::new()
                .add_attribute("action", "update_whitelist")
                .add_attribute("whitelist", "removed"))
        }
    }
}

pub fn query_whitelist(
    deps: cosmwasm_std::Deps,
) -> Result<Option<cosmwasm_std::Addr>, ContractError> {
    Ok(WHITELIST.may_load(deps.storage)?)
}
