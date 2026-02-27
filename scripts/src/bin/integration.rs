// Integration deployment script for cw-svg + cw-infuser.
//
// Usage:
//   cargo run -p cw-infuser-scripts --bin integration -- --network local
//   cargo run -p cw-infuser-scripts --bin integration -- --network mainnet

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cw_infuser_scripts::suite::svg::load_svg_init_msg;
use cw_infuser_scripts::suite::{CwSvgSuite, CwSvgSuiteDeployData};
use cw_infuser_scripts::{LOCAL_TERP, MOROCCO_1};
use cw_orch::daemon::DaemonBuilder;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Runtime;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network: local, mainnet
    #[clap(short, long, default_value = "mainnet")]
    network: String,

    /// Skip Docker spinup (chain already running)
    #[clap(long)]
    skip_docker: bool,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dotenv::dotenv().ok();
    env_logger::init();

    let network = match args.network.as_str() {
        "local" => LOCAL_TERP.to_owned(),
        "mainnet" => MOROCCO_1.to_owned(),
        other => return Err(anyhow!("Unknown network: {}", other)),
    };

    if !args.skip_docker && args.network == "local" {
        // When invoked from local-test-env.sh, Docker is already running.
        // Only spinup if running standalone.
        println!("Note: pass --skip-docker if chain is already running.");
    }

    workflow(network.into())?;
    Ok(())
}

fn workflow(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .build()?;

    let sender = chain.sender_addr();
    let data = deploy_data(sender.clone(), &network.chain_id)?;
    let suite = CwSvgSuite::deploy_on(chain.clone(), data)?;

    // Print addresses in a format the shell script can parse
    println!("CONTRACT_ADDR:cw_svg_minter={}", suite.cwsvgminter.addr_str()?);
    println!("CONTRACT_ADDR:cw721_svg={}", suite.cwsvg.addr_str()?);
    // Infuser is only instantiated when infusions are enabled
    if let Ok(addr) = suite.infuser.addr_str() {
        println!("CONTRACT_ADDR:cw_infusion_minter={}", addr);
    } else {
        println!("SKIP: cw_infusion_minter not instantiated");
    }

    Ok(())
}

fn deploy_data(sender: Addr, _chain_id: &str) -> Result<Option<CwSvgSuiteDeployData>> {
    let mut svg = Vec::new();
    let mut akt = load_svg_init_msg("scripts/svgs/interchain/akash/init.json")?;
    let mut bcna = load_svg_init_msg("scripts/svgs/interchain/bcna/init.json")?;
    let mut btc = load_svg_init_msg("scripts/svgs/interchain/bitcoin/init.json")?;
    let mut btsg = load_svg_init_msg("scripts/svgs/interchain/bitsong/init.json")?;
    let mut atom = load_svg_init_msg("scripts/svgs/interchain/cosmos/init.json")?;
    let mut dao = load_svg_init_msg("scripts/svgs/interchain/dao/init.json")?;
    let mut eth = load_svg_init_msg("scripts/svgs/interchain/eth/init.json")?;
    let mut monero = load_svg_init_msg("scripts/svgs/interchain/monero/init.json")?;
    let mut um = load_svg_init_msg("scripts/svgs/interchain/penumbra/init.json")?;
    let mut terp = load_svg_init_msg("scripts/svgs/interchain/terp/init.json")?;
    let mut zec = load_svg_init_msg("scripts/svgs/interchain/zec/init.json")?;
    terp.owner = Some(sender.to_string());
    dao.owner = Some(sender.to_string());
    atom.owner = Some(sender.to_string());
    btc.owner = Some(sender.to_string());
    akt.owner = Some(sender.to_string());
    um.owner = Some(sender.to_string());
    btsg.owner = Some(sender.to_string());
    bcna.owner = Some(sender.to_string());
    monero.owner = Some(sender.to_string());
    eth.owner = Some(sender.to_string());
    zec.owner = Some(sender.to_string());

    svg.extend(vec![
        terp, dao, atom, btc, akt, um, btsg, bcna, monero, eth, zec,
    ]);
    Ok(Some(CwSvgSuiteDeployData {
        svg,
        infuse: None,
        admin: Some(sender),
        infuse_coins: vec![],
    }))
}
// fn spinup() -> Result<()> {
//     println!("Building localterp image...");
//     run_sh_command(
//         "docker buildx build --target localterp -t terpnetwork/terp-core:localterp --load .",
//     )?;
//     println!("Starting localterp container...");
//     run_sh_command(
//         "docker run --rm -it -p 26657:26657 -p 1317:1317 -p 8545:8545 terpnetwork/terp-core:localterp"
//     )?;
//     println!("Container started successfully.");
//     Ok(())
// }

// fn run_sh_command(cmd: &str) -> Result<()> {
//     let mut parts = shlex::Shlex::new(cmd);
//     let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
//     let args: Vec<String> = parts.collect();

//     let status = Command::new(&program)
//         .args(&args)
//         .status()
//         .with_context(|| format!("Failed to execute: {}", cmd))?;

//     if !status.success() {
//         return Err(anyhow!(
//             "Command failed: {} (exit code: {:?})",
//             cmd,
//             status.code()
//         ));
//     }

//     Ok(())
// }
