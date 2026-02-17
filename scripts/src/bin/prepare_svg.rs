/// Helper binary that prepares a cw721-svg InstantiateMsg JSON from an SVG template file.
///
/// Usage:
///   cargo run -p cw-infuser-scripts --bin prepare_svg -- --svg path/to/template.svg
///
/// The tool:
///   1. Loads the SVG file and scans for ${varname} placeholders
///   2. Walks you through defining each variable (Options or Range)
///   3. Outputs a complete InstantiateMsg JSON ready for on-chain instantiation
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cw721_svg::msg::{InstantiateMsg, PriceTier, TemplateSlot, VariableDef, VariableKind};
use serde_json;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Prepare a cw721-svg InstantiateMsg JSON from an SVG template"
)]
struct Args {
    /// Path to the SVG template file containing ${varname} placeholders
    #[arg(long)]
    svg: String,

    /// Collection name
    #[arg(long)]
    name: Option<String>,

    /// Collection symbol
    #[arg(long)]
    symbol: Option<String>,

    /// Total supply
    #[arg(long)]
    total: Option<u64>,

    /// Output file (defaults to stdout)
    #[arg(long, short)]
    output: Option<String>,
}

/// Discovered placeholder in the template
struct Placeholder {
    name: String,
    start: usize,
    end: usize,
}

/// Scan an SVG template for ${varname} placeholders.
/// Returns unique variable names (in order of first appearance) and all slot positions.
fn scan_placeholders(template: &str) -> Result<(Vec<String>, Vec<Placeholder>)> {
    let bytes = template.as_bytes();
    let len = bytes.len();
    let mut placeholders = Vec::new();
    let mut seen_order: Vec<String> = Vec::new();
    let mut i = 0;

    while i < len.saturating_sub(1) {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i;
            let name_start = i + 2;
            let mut j = name_start;
            while j < len && bytes[j] != b'}' {
                j += 1;
            }
            if j >= len {
                return Err(anyhow!(
                    "Unclosed placeholder at byte offset {} (near: '{}...')",
                    start,
                    &template[start..std::cmp::min(start + 30, len)]
                ));
            }
            let name = template[name_start..j].to_string();
            let end = j + 1;

            if !seen_order.contains(&name) {
                seen_order.push(name.clone());
            }

            placeholders.push(Placeholder { name, start, end });
            i = end;
        } else {
            i += 1;
        }
    }

    Ok((seen_order, placeholders))
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn define_variable(name: &str, occurrence_count: usize) -> Result<VariableDef> {
    println!();
    println!("━━━ Variable: ${{{name}}} ({occurrence_count} occurrence(s)) ━━━");
    println!("  [1] Options  — pick from a list of values");
    println!("  [2] Range    — random decimal in [min, max]");
    let choice = prompt("  Type (1 or 2): ");

    let kind = match choice.as_str() {
        "1" => {
            println!("  Enter options one per line. Empty line to finish:");
            let mut opts = Vec::new();
            loop {
                let val = prompt("    > ");
                if val.is_empty() {
                    break;
                }
                opts.push(val);
            }
            if opts.is_empty() {
                return Err(anyhow!("Options list must not be empty for '{}'", name));
            }
            VariableKind::Options(opts)
        }
        "2" => {
            let min = prompt("  min: ");
            let max = prompt("  max: ");
            let prec = prompt("  precision (decimal places, default 0): ");
            let precision = if prec.is_empty() {
                0
            } else {
                prec.parse().context("invalid precision")?
            };
            VariableKind::Range {
                min,
                max,
                precision,
            }
        }
        _ => return Err(anyhow!("Invalid choice '{}', expected 1 or 2", choice)),
    };

    Ok(VariableDef {
        name: name.to_string(),
        kind,
    })
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Load SVG template
    let template =
        fs::read_to_string(&args.svg).with_context(|| format!("Failed to read: {}", args.svg))?;

    println!("Loaded SVG template: {} bytes", template.len());

    // 2. Scan for placeholders
    let (var_names, placeholders) = scan_placeholders(&template)?;

    if var_names.is_empty() {
        println!("No ${{...}} placeholders found in template.");
        println!("The SVG will be stored as-is with no variable substitution.");
    } else {
        println!("Found {} unique variable(s):", var_names.len());
        // Count occurrences per variable
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for ph in &placeholders {
            *counts.entry(&ph.name).or_default() += 1;
        }
        for name in &var_names {
            let count = counts.get(name.as_str()).unwrap_or(&0);
            println!("  ${{{name}}}  — {count} occurrence(s)");
        }
    }

    // 3. Define each variable interactively
    let mut variables = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for ph in &placeholders {
        *counts.entry(&ph.name).or_default() += 1;
    }

    for name in &var_names {
        let count = *counts.get(name.as_str()).unwrap_or(&0);
        let var_def = define_variable(name, count)?;
        variables.push(var_def);
    }

    // 4. Build template_slots
    let var_index: BTreeMap<&str, u16> = var_names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i as u16))
        .collect();

    let template_slots: Vec<TemplateSlot> = placeholders
        .iter()
        .map(|ph| TemplateSlot {
            start: ph.start as u32,
            end: ph.end as u32,
            var_idx: var_index[ph.name.as_str()],
        })
        .collect();

    // 5. Collect remaining fields
    let name = args.name.unwrap_or_else(|| prompt("Collection name: "));
    let symbol = args.symbol.unwrap_or_else(|| prompt("Collection symbol: "));
    let total = match args.total {
        Some(t) => t,
        None => {
            let t = prompt("Total supply: ");
            t.parse().context("invalid total supply")?
        }
    };

    let seed_input = prompt("Seed (any string, will be hashed): ");
    let seed_bytes = blake3::hash(seed_input.as_bytes());
    let seed = cosmwasm_std::Binary::from(seed_bytes.as_bytes().as_slice());

    // 6. Build InstantiateMsg
    let msg = InstantiateMsg {
        name,
        symbol,
        svg_template: template,
        variables,
        total,
        seed,
        owner: None,
        mint_start_time: None,
        mint_end_time: None,
        price_tiers: vec![],
        payment_address: None,
        whitelist: None,
        template_slots,
    };

    // 7. Output JSON
    let json = serde_json::to_string_pretty(&msg)?;

    match args.output {
        Some(path) => {
            fs::write(&path, &json).with_context(|| format!("Failed to write: {}", path))?;
            println!("\nInstantiateMsg written to: {path}");
        }
        None => {
            println!("\n{json}");
        }
    }

    Ok(())
}
