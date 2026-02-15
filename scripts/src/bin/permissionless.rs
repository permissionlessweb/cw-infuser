use clap::Parser;
use cosmwasm_std::coin;
use cw721_svg::msg::{VariableDef, VariableKind};
use cw_infuser_scripts::deploy::svg::Cw721Svg;
use cw_infuser_scripts::MOROCCO_1;
use cw_orch::daemon::{Daemon, TxSender};
use cw_orch::prelude::CwOrchInstantiate;
use cw_svg::{InstantiateMsg, PriceTier};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Minimal SVG template that visualizes attractor params as a geometric constellation.
/// The real fractal rendering happens off-chain; this gives each token a displayable on-chain preview.
const FRACTAL_SVG_TEMPLATE: &str = include_str!("../../../packages/cw-svg/svgs/fractal.svg");

/// Deploy fractal attractor SVG NFTs using on-chain Range variables.
///
/// Fractal params (a–h, base_range) are generated at mint time via
/// `VariableKind::Range` — no pre-generated pool needed. The zooms
/// variable still uses `VariableKind::Options` since it's a complex
/// JSON structure.
///
/// These params feed an off-chain fractal visualizer on permissionless.money.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Initial seed for deterministic RNG (used for contract seed + zoom generation)
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Number of tokens
    #[arg(long, default_value_t = 256)]
    supply: u64,

    /// Number of unique zoom sets to pre-generate
    #[arg(long, default_value_t = 64)]
    zoom_variants: u64,

    /// Output directory for JSON files
    #[arg(long, default_value = "scripts/json")]
    out_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct Zoom {
    cx: f64,
    cy: f64,
    scale: f64,
}

fn gen_float(rng: &mut impl rand::Rng, min: f64, max: f64, precision: u32) -> f64 {
    let raw: f64 = rng.gen_range(min..=max);
    let factor = 10f64.powi(precision as i32);
    (raw * factor).round() / factor
}

fn gen_zoom_set(rng: &mut impl rand::Rng) -> Vec<Zoom> {
    let num_zooms = rng.gen_range(3u32..=5);
    let mut zooms = vec![Zoom {
        cx: 0.0,
        cy: 0.0,
        scale: 1.0,
    }];

    for i in 1..num_zooms {
        let progress = i as f64 / (num_zooms - 1) as f64;
        zooms.push(Zoom {
            cx: gen_float(rng, -4.0, 4.0, 1),
            cy: gen_float(rng, -4.0, 4.0, 1),
            scale: gen_float(rng, 1.0 + progress, 1.0 + progress * 5.0, 1),
        });
    }

    zooms
}

/// Build variable definitions using Range for numeric params and Options for zooms.
fn build_variable_defs(zoom_variants: &[Vec<Zoom>]) -> Vec<VariableDef> {
    let range_vars = vec![
        ("a", "-2.5", "2.5", 2u32),
        ("b", "-2.5", "2.5", 2),
        ("c", "-2.5", "2.5", 2),
        ("d", "-2.5", "2.5", 2),
        ("s", "-2.0", "2.0", 1),
        ("f", "-2.0", "2.0", 1),
        ("g", "-2.0", "2.0", 1),
        ("h", "-2.0", "2.0", 1),
        ("base_range", "10", "80", 0),
    ];

    let mut defs: Vec<VariableDef> = range_vars
        .into_iter()
        .map(|(name, min, max, precision)| VariableDef {
            name: name.to_string(),
            kind: VariableKind::Range {
                min: min.to_string(),
                max: max.to_string(),
                precision,
            },
        })
        .collect();

    // Zooms remain as Options since they're complex JSON
    let zoom_options: Vec<String> = zoom_variants
        .iter()
        .map(|z| serde_json::to_string(z).unwrap())
        .collect();

    defs.push(VariableDef {
        name: "zooms".to_string(),
        kind: VariableKind::Options(zoom_options),
    });

    defs
}

pub fn main() -> anyhow::Result<()> {
    use rand::{rngs::StdRng, SeedableRng};

    let args = Args::parse();
    let mut rng = StdRng::seed_from_u64(args.seed);

    // Pre-generate zoom variants (complex JSON — still uses Options)
    let zoom_variants: Vec<Vec<Zoom>> = (0..args.zoom_variants)
        .map(|_| gen_zoom_set(&mut rng))
        .collect();

    // Build variable definitions: Range for numeric params, Options for zooms
    let variable_defs = build_variable_defs(&zoom_variants);

    // Ensure output dir exists
    fs::create_dir_all(&args.out_dir)?;

    // Write VariableDef JSON (for reference / debugging)
    let json_path = args.out_dir.join("fractal_variables.json");
    let json = serde_json::to_string_pretty(&variable_defs)?;
    fs::write(&json_path, &json)?;
    println!("VariableDefs JSON written to {}", json_path.display());

    // Summary
    println!("\n--- Summary ---");
    for vd in &variable_defs {
        match &vd.kind {
            VariableKind::Range {
                min,
                max,
                precision,
            } => {
                println!(
                    "  ${{{}}}: Range [{}, {}] precision={}",
                    vd.name, min, max, precision
                );
            }
            VariableKind::Options(opts) => {
                println!("  ${{{}}}: {} pre-generated options", vd.name, opts.len());
            }
        }
    }

    let chain = Daemon::builder(MOROCCO_1).build()?;
    let svg = Cw721Svg::new(chain.clone());
    let _res = svg.instantiate(
        &InstantiateMsg {
            name: "Permissionless Fractals".into(),
            symbol: "FRACTAL".into(),
            svg_template: FRACTAL_SVG_TEMPLATE.to_string(),
            variables: variable_defs,
            total: args.supply,
            seed: blake3::hash(&args.seed.to_le_bytes()).as_bytes().into(),
            owner: Some(chain.sender().address().to_string()),
            mint_start_time: None,
            mint_end_time: None,
            price_tiers: vec![
                PriceTier {
                    until_count: 1000,
                    price: coin(100_000_000, "uthiol"),
                },
                PriceTier {
                    until_count: 5000,
                    price: coin(500_000_000, "uthiol"),
                },
                PriceTier {
                    until_count: args.supply,
                    price: coin(1000_000_000, "uthiol"),
                },
            ],
            payment_address: None,
            whitelist: None,
        },
        Some(&chain.sender().address()),
        None,
    )?;

    Ok(())
}
