use anyhow::Result;
use clap::Parser;
use rs_merkle::MerkleTree;
use serde::Serialize;
use std::fs;
use std::io::Write;
use whitelist_mtree::tests::{hasher::SortingBlake3Hasher, test_helpers::hash_and_build_tree};

#[derive(Parser, Debug)]
#[command(
    name = "merkle",
    about = "Generate BLAKE3 sorted merkle tree root and proofs from a list of addresses"
)]
struct Args {
    /// Path to CSV/text file with addresses (one per line)
    #[arg(short, long)]
    input: String,

    /// Generate individual proofs for each address
    #[arg(long, default_value_t = false)]
    proofs: bool,

    /// Input contains per-address allocations in CSV format: address,allocation
    /// The merkle leaf becomes hash(address || allocation) so the proof covers both.
    #[arg(long, default_value_t = false)]
    with_allocations: bool,

    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Serialize)]
struct Output {
    root: String,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    proofs: Option<Vec<ProofEntry>>,
}

#[derive(Serialize)]
struct ProofEntry {
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    allocation: Option<u32>,
    proof: Vec<String>,
}

struct ParsedEntry {
    address: String,
    allocation: Option<u32>,
    /// The leaf string fed into the tree: address or address+allocation
    leaf: String,
}

fn parse_input(path: &str, with_allocations: bool) -> Result<Vec<ParsedEntry>> {
    let data = fs::read_to_string(path)?;
    let mut lines: Vec<&str> = data
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    // Skip header if the first line looks like one
    if let Some(first) = lines.first() {
        if first.to_lowercase().contains("address") {
            lines.remove(0);
        }
    }

    let entries: Result<Vec<ParsedEntry>> = lines
        .into_iter()
        .map(|line| {
            if with_allocations {
                let parts: Vec<&str> = line.splitn(2, ',').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Expected 'address,allocation' format but got: {}", line);
                }
                let address = parts[0].trim().to_string();
                let allocation: u32 = parts[1].trim().parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid allocation '{}' for address {}",
                        parts[1].trim(),
                        address
                    )
                })?;
                let leaf = format!("{}{}", address, allocation);
                Ok(ParsedEntry {
                    address,
                    allocation: Some(allocation),
                    leaf,
                })
            } else {
                let address = line.to_string();
                Ok(ParsedEntry {
                    leaf: address.clone(),
                    address,
                    allocation: None,
                })
            }
        })
        .collect();

    entries
}

fn main() -> Result<()> {
    let args = Args::parse();

    let entries = parse_input(&args.input, args.with_allocations)?;
    if entries.is_empty() {
        anyhow::bail!("No addresses found in {}", args.input);
    }

    let leaves: Vec<String> = entries.iter().map(|e| e.leaf.clone()).collect();
    let tree: MerkleTree<SortingBlake3Hasher> = hash_and_build_tree(&leaves);
    let root = tree
        .root_hex()
        .ok_or_else(|| anyhow::anyhow!("Failed to compute merkle root"))?;

    let proofs = if args.proofs {
        let proof_entries: Vec<ProofEntry> = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let proof = tree.proof(&[i]);
                ProofEntry {
                    address: entry.address.clone(),
                    allocation: entry.allocation,
                    proof: proof.proof_hashes_hex(),
                }
            })
            .collect();
        Some(proof_entries)
    } else {
        None
    };

    let output = Output {
        root,
        count: entries.len(),
        proofs,
    };

    let json = serde_json::to_string_pretty(&output)?;

    match args.output {
        Some(path) => {
            let mut file = fs::File::create(&path)?;
            file.write_all(json.as_bytes())?;
            eprintln!("Written to {}", path);
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}
