/// Parses a Cosmos SDK genesis and/or exported state JSON to produce a tiered
/// whitelist CSV.
///
/// Tiers:
///   1 — addresses with > min_sequence txs on-chain  → allocation 3
///   2 — genesis validators                          → allocation 6  (tier 1 × 2)
///   3 — current validators (in export)              → allocation 18 (tier 2 × 3)
///
/// Each address receives the highest applicable tier.
///
/// Usage:
///   cargo run -p cw-infuser-scripts --bin parse_export -- \
///       --export exported-state.json \
///       --genesis genesis.json \
///       --prefix terp \
///       -o whitelist.csv
use anyhow::{Context, Result};
use bech32::{Bech32, Hrp};
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Parse Cosmos SDK export/genesis to produce a tiered whitelist CSV"
)]
struct Args {
    /// Path to the exported state JSON (live network export).
    /// Used for: active accounts (tier 1) and current validators (tier 3).
    #[arg(long)]
    export: Option<String>,

    /// Path to the genesis JSON.
    /// Used for: genesis validators (tier 2).
    #[arg(long)]
    genesis: Option<String>,

    /// Bech32 address prefix (e.g. "terp", "cosmos", "osmo")
    #[arg(long, default_value = "terp")]
    prefix: String,

    /// Minimum sequence (tx count) to qualify for tier 1.
    /// Default 1 means any address with at least 1 tx qualifies.
    #[arg(long, default_value = "1")]
    min_sequence: u64,

    /// Output CSV file (defaults to stdout)
    #[arg(long, short)]
    output: Option<String>,
}

/// Extract (address, sequence) from a Cosmos SDK auth account JSON object.
/// Handles BaseAccount, vesting accounts, and skips ModuleAccounts.
fn extract_account_info(account: &Value) -> Option<(String, u64)> {
    let account_type = account.get("@type")?.as_str()?;

    // Skip module accounts — these are system addresses
    if account_type.contains("ModuleAccount") {
        return None;
    }

    // Navigate to the base account fields depending on account type
    let (address, sequence_str) = if let Some(bva) = account.get("base_vesting_account") {
        let ba = bva.get("base_account")?;
        (ba.get("address")?.as_str()?, ba.get("sequence")?.as_str()?)
    } else if account_type.contains("BaseAccount") {
        (
            account.get("address")?.as_str()?,
            account.get("sequence")?.as_str()?,
        )
    } else if let Some(ba) = account.get("base_account") {
        (ba.get("address")?.as_str()?, ba.get("sequence")?.as_str()?)
    } else {
        // Unknown account type — try top-level fields
        (
            account.get("address")?.as_str()?,
            account.get("sequence")?.as_str()?,
        )
    };

    let sequence: u64 = sequence_str.parse().ok()?;
    Some((address.to_string(), sequence))
}

/// Extract validator operator addresses from app_state.staking.validators
fn extract_validators(state: &Value) -> Vec<String> {
    state
        .pointer("/app_state/staking/validators")
        .and_then(|v| v.as_array())
        .map(|validators| {
            validators
                .iter()
                .filter_map(|v| v.get("operator_address")?.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Convert a valoper address to an account address by swapping the bech32 prefix.
/// e.g. terpvaloper1abc... → terp1abc...
fn valoper_to_account(valoper: &str, account_prefix: &str) -> Result<String> {
    let (_, data) = bech32::decode(valoper)
        .with_context(|| format!("failed to decode bech32 address: {}", valoper))?;
    let hrp = Hrp::parse(account_prefix)?;
    let encoded = bech32::encode::<Bech32>(hrp, &data)?;
    Ok(encoded)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.export.is_none() && args.genesis.is_none() {
        anyhow::bail!("at least one of --export or --genesis must be provided");
    }

    // Track address → tier (highest wins)
    let mut tiers: HashMap<String, u8> = HashMap::new();

    // --- Parse export file for active accounts (tier 1) and current validators (tier 3) ---
    if let Some(export_path) = &args.export {
        eprintln!("Loading export file: {}", export_path);
        let raw = fs::read_to_string(export_path)
            .with_context(|| format!("failed to read: {}", export_path))?;
        let export: Value =
            serde_json::from_str(&raw).with_context(|| "failed to parse export JSON")?;

        // Tier 1: active accounts (sequence >= min_sequence)
        if let Some(accounts) = export.pointer("/app_state/auth/accounts").and_then(|v| v.as_array()) {
            let mut active_count = 0u64;
            for account in accounts {
                if let Some((address, sequence)) = extract_account_info(account) {
                    if sequence >= args.min_sequence {
                        tiers.entry(address).or_insert(1);
                        active_count += 1;
                    }
                }
            }
            eprintln!("  Tier 1 (active accounts): {} addresses", active_count);
        }

        // Tier 3: current validators
        let current_validators = extract_validators(&export);
        let mut val_count = 0u64;
        for valoper in &current_validators {
            match valoper_to_account(valoper, &args.prefix) {
                Ok(account_addr) => {
                    tiers.insert(account_addr, 3);
                    val_count += 1;
                }
                Err(e) => eprintln!("  warning: skipping validator {}: {}", valoper, e),
            }
        }
        eprintln!("  Tier 3 (current validators): {} addresses", val_count);
    }

    // --- Parse genesis file for genesis validators (tier 2) ---
    if let Some(genesis_path) = &args.genesis {
        eprintln!("Loading genesis file: {}", genesis_path);
        let raw = fs::read_to_string(genesis_path)
            .with_context(|| format!("failed to read: {}", genesis_path))?;
        let genesis: Value =
            serde_json::from_str(&raw).with_context(|| "failed to parse genesis JSON")?;

        let genesis_validators = extract_validators(&genesis);
        let mut gen_val_count = 0u64;
        for valoper in &genesis_validators {
            match valoper_to_account(valoper, &args.prefix) {
                Ok(account_addr) => {
                    let current = tiers.get(&account_addr).copied().unwrap_or(0);
                    // Only upgrade to tier 2 if not already tier 3
                    if current < 2 {
                        tiers.insert(account_addr, 2);
                    }
                    gen_val_count += 1;
                }
                Err(e) => eprintln!("  warning: skipping genesis validator {}: {}", valoper, e),
            }
        }
        eprintln!("  Tier 2 (genesis validators): {} addresses", gen_val_count);
    }

    // --- Build sorted output ---
    let allocation_for_tier = |tier: u8| -> u32 {
        match tier {
            1 => 3,
            2 => 6,
            3 => 18,
            _ => 0,
        }
    };

    let mut rows: Vec<(String, u8, u32)> = tiers
        .into_iter()
        .map(|(addr, tier)| {
            let alloc = allocation_for_tier(tier);
            (addr, tier, alloc)
        })
        .collect();
    rows.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    // --- Summary ---
    let tier_counts: HashMap<u8, usize> = rows.iter().fold(HashMap::new(), |mut m, (_, t, _)| {
        *m.entry(*t).or_default() += 1;
        m
    });
    eprintln!("\nSummary:");
    for tier in 1..=3u8 {
        let count = tier_counts.get(&tier).unwrap_or(&0);
        eprintln!(
            "  Tier {}: {} addresses (allocation: {})",
            tier,
            count,
            allocation_for_tier(tier)
        );
    }
    eprintln!("  Total: {} addresses", rows.len());

    // --- Write CSV ---
    let csv_content = {
        let mut buf = Vec::new();
        writeln!(buf, "address,tier,allocation")?;
        for (addr, tier, alloc) in &rows {
            writeln!(buf, "{},{},{}", addr, tier, alloc)?;
        }
        buf
    };

    match args.output {
        Some(path) => {
            fs::write(&path, &csv_content)
                .with_context(|| format!("failed to write: {}", path))?;
            eprintln!("\nCSV written to: {}", path);
        }
        None => {
            std::io::stdout().write_all(&csv_content)?;
        }
    }

    Ok(())
}
