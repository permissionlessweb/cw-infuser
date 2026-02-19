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
use cw721_svg::msg::{InstantiateMsg, PriceTier, RgbRange, TemplateSlot, VariableDef, VariableKind};
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

    /// Seed string (will be blake3-hashed). If omitted, prompted interactively.
    #[arg(long)]
    seed: Option<String>,

    /// Path to a JSON file containing variable definitions (skips interactive prompts).
    /// Format: [{"name":"color_yin","kind":{"options":["red","blue"]}}]
    #[arg(long)]
    vars_json: Option<String>,

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
    println!("  [1] Options      — pick from a list of values");
    println!("  [2] Range        — random decimal in [min, max]");
    println!("  [3] Rgb          — random rgb(R,G,B) color");
    println!("  [4] Rgb Styled   — random shade within defined color ranges");
    let choice = prompt("  Type (1, 2, 3, or 4): ");

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
        "3" => VariableKind::Rgb,
        "4" => {
            println!("  Define color ranges. Each range constrains R, G, B channels.");
            println!("  Enter ranges one at a time. Empty r_min to finish.");
            let mut ranges = Vec::new();
            loop {
                println!("  --- Range {} ---", ranges.len() + 1);
                let r_min = prompt("    r_min (0-255, empty to finish): ");
                if r_min.is_empty() {
                    break;
                }
                let r_max = prompt("    r_max (0-255): ");
                let g_min = prompt("    g_min (0-255): ");
                let g_max = prompt("    g_max (0-255): ");
                let b_min = prompt("    b_min (0-255): ");
                let b_max = prompt("    b_max (0-255): ");
                ranges.push(RgbRange {
                    r_min: r_min.parse().context("invalid r_min")?,
                    r_max: r_max.parse().context("invalid r_max")?,
                    g_min: g_min.parse().context("invalid g_min")?,
                    g_max: g_max.parse().context("invalid g_max")?,
                    b_min: b_min.parse().context("invalid b_min")?,
                    b_max: b_max.parse().context("invalid b_max")?,
                });
            }
            if ranges.is_empty() {
                return Err(anyhow!("Rgb Styled requires at least one range for '{}'", name));
            }
            VariableKind::RgbStyled(ranges)
        }
        _ => return Err(anyhow!("Invalid choice '{}', expected 1, 2, 3, or 4", choice)),
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

    // 3. Define variables (from JSON file or interactively)
    let variables: Vec<VariableDef> = if let Some(vars_path) = &args.vars_json {
        let vars_str = fs::read_to_string(vars_path)
            .with_context(|| format!("Failed to read vars JSON: {}", vars_path))?;
        let vars: Vec<VariableDef> = serde_json::from_str(&vars_str)
            .with_context(|| format!("Failed to parse vars JSON from {}", vars_path))?;
        // Verify all template variables are defined
        for name in &var_names {
            if !vars.iter().any(|v| v.name == *name) {
                return Err(anyhow!(
                    "Template variable '{}' not found in vars JSON",
                    name
                ));
            }
        }
        println!("Loaded {} variable definition(s) from {}", vars.len(), vars_path);
        vars
    } else {
        let mut vars = Vec::new();
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for ph in &placeholders {
            *counts.entry(&ph.name).or_default() += 1;
        }
        for name in &var_names {
            let count = *counts.get(name.as_str()).unwrap_or(&0);
            let var_def = define_variable(name, count)?;
            vars.push(var_def);
        }
        vars
    };

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

    let seed_input = args
        .seed
        .unwrap_or_else(|| prompt("Seed (any string, will be hashed): "));
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
