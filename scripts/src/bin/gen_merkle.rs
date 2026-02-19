/// Reads a tiered whitelist CSV (from parse_export) and generates a merkle tree
/// with per-address allocation proofs.
///
/// Each leaf is `blake3::hash(format!("{}{}", address, allocation))`, matching
/// the whitelist-merkletree contract's allocation-based verification.
///
/// Outputs a JSON file with:
///   - merkle_root: hex-encoded root hash
///   - total_addresses: count
///   - tier_summary: per-tier counts
///   - accounts: map of address → { tier, allocation, proof_hashes }
///
/// Usage:
///   cargo run -p cw-infuser-scripts --bin gen_merkle -- \
///       --csv whitelist.csv \
///       -o merkle.json
use anyhow::{Context, Result};
use clap::Parser;
use rs_merkle::{Hasher, MerkleTree};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about = "Generate a merkle tree from a tiered whitelist CSV")]
struct Args {
    /// Path to the whitelist CSV (address,tier,allocation)
    #[arg(long)]
    csv: String,

    /// Output JSON file (defaults to stdout)
    #[arg(long, short)]
    output: Option<String>,
}

/// Blake3 hasher that sorts left/right before concatenation.
/// Must match the on-chain whitelist-merkletree contract's verification logic.
#[derive(Clone)]
struct SortingBlake3Hasher;

impl Hasher for SortingBlake3Hasher {
    type Hash = [u8; 32];

    fn concat_and_hash(left: &Self::Hash, right: Option<&Self::Hash>) -> Self::Hash {
        match right {
            Some(right_node) => {
                let mut both = [left, right_node];
                both.sort_unstable();

                let mut concatenated: Vec<u8> = both[0].to_vec();
                concatenated.extend_from_slice(both[1]);

                Self::hash(&concatenated)
            }
            None => *left,
        }
    }

    fn hash(data: &[u8]) -> Self::Hash {
        *blake3::hash(data).as_bytes()
    }

    fn hash_size() -> usize {
        blake3::OUT_LEN
    }
}

#[derive(Debug)]
struct CsvRow {
    address: String,
    tier: u8,
    allocation: u32,
}

#[derive(Serialize)]
struct AccountProof {
    tier: u8,
    allocation: u32,
    proof_hashes: Vec<String>,
}

#[derive(Serialize)]
struct TierSummary {
    count: usize,
    allocation: u32,
}

#[derive(Serialize)]
struct MerkleOutput {
    merkle_root: String,
    total_addresses: usize,
    tier_summary: BTreeMap<String, TierSummary>,
    accounts: BTreeMap<String, AccountProof>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Read CSV
    let mut reader = csv::Reader::from_path(&args.csv)
        .with_context(|| format!("failed to open CSV: {}", args.csv))?;

    let mut rows: Vec<CsvRow> = Vec::new();
    for result in reader.records() {
        let record = result.with_context(|| "failed to read CSV record")?;
        let address = record
            .get(0)
            .context("missing address column")?
            .to_string();
        let tier: u8 = record
            .get(1)
            .context("missing tier column")?
            .parse()
            .context("invalid tier")?;
        let allocation: u32 = record
            .get(2)
            .context("missing allocation column")?
            .parse()
            .context("invalid allocation")?;

        rows.push(CsvRow {
            address,
            tier,
            allocation,
        });
    }

    if rows.is_empty() {
        anyhow::bail!("CSV is empty — no addresses to process");
    }

    eprintln!("Loaded {} addresses from CSV", rows.len());

    // 2. Build leaves: blake3::hash(format!("{}{}", address, allocation))
    let leaves: Vec<[u8; 32]> = rows
        .iter()
        .map(|row| {
            let leaf_input = format!("{}{}", row.address, row.allocation);
            *blake3::hash(leaf_input.as_bytes()).as_bytes()
        })
        .collect();

    // 3. Build merkle tree
    let tree = MerkleTree::<SortingBlake3Hasher>::from_leaves(&leaves);
    let root = tree.root_hex().context("empty tree — no root")?;

    eprintln!("Merkle root: {}", root);

    // 4. Generate proofs for each address
    let mut accounts: BTreeMap<String, AccountProof> = BTreeMap::new();
    for (i, row) in rows.iter().enumerate() {
        let proof = tree.proof(&[i]);
        let proof_hashes: Vec<String> = proof.proof_hashes_hex();

        accounts.insert(
            row.address.clone(),
            AccountProof {
                tier: row.tier,
                allocation: row.allocation,
                proof_hashes,
            },
        );
    }

    // 5. Build tier summary
    let mut tier_summary: BTreeMap<String, TierSummary> = BTreeMap::new();
    for row in &rows {
        let entry = tier_summary
            .entry(format!("tier_{}", row.tier))
            .or_insert(TierSummary {
                count: 0,
                allocation: row.allocation,
            });
        entry.count += 1;
    }

    // 6. Output
    let output = MerkleOutput {
        merkle_root: root.clone(),
        total_addresses: rows.len(),
        tier_summary,
        accounts,
    };

    let json = serde_json::to_string_pretty(&output)?;

    match args.output {
        Some(path) => {
            fs::write(&path, &json).with_context(|| format!("failed to write: {}", path))?;
            eprintln!("Merkle tree JSON written to: {}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    eprintln!("\nSummary:");
    for (tier, summary) in &output.tier_summary {
        eprintln!(
            "  {}: {} addresses (allocation: {})",
            tier, summary.count, summary.allocation
        );
    }
    eprintln!("  Total: {} addresses", output.total_addresses);

    Ok(())
}
