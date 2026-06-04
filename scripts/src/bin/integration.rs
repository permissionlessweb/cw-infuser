// Integration deployment script for cw-svg + cw-infuser.
//
// Usage:
//   cargo run -p cw-infuser-scripts --bin integration -- --network local
//   cargo run -p cw-infuser-scripts --bin integration -- --network mainnet
// with single integration:
//   cargo run -p cw-infuser-scripts --bin integration -- --network mainnet --single

use anyhow::{anyhow, Result};
use clap::Parser;
use cosmwasm_std::Decimal;
use cw_infuser_scripts::suite::svg::load_svg_init_msg;
use cw_infuser_scripts::suite::whitelist::load_terp_warrior_mtree;
use cw_infuser_scripts::suite::{CwSvgSuite, CwSvgSuiteDeployData};
use cw_orch::daemon::networks::{TERP_LOCALNET, TERP_MAINNET};
use cw_orch::daemon::DaemonBuilder;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Runtime;
use shit_scripts::{CwShitstrapSuite, CwShitstrapSuiteDeployData, ShitInitMsg, UncheckedDenom};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network: local, mainnet
    #[clap(short, long, default_value = "mainnet")]
    network: String,

    #[clap(long)]
    single: bool,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dotenv::dotenv().ok();
    env_logger::init();

    let network = match args.network.as_str() {
        "local" => TERP_LOCALNET.to_owned(),
        "mainnet" => TERP_MAINNET.to_owned(),
        other => return Err(anyhow!("Unknown network: {}", other)),
    };

    match args.single {
        true => workflow_single(network.clone().into())?,
        false => workflow(network.into())?,
    }

    Ok(())
}

fn workflow_single(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .build()?;
    let sender = chain.sender_addr();

    let data = deploy_data_single(sender.clone(), &network.chain_id)?;
    let shit = shit_deploy_data_single(sender);

    let mut suite = CwSvgSuite::deploy_on(chain.clone(), data)?;
    let shit = CwShitstrapSuite::deploy_on(chain.clone(), shit)?;
    suite.shit = shit;

    // Print addresses in a format the shell script can parse
    println!("CONTRACT_ADDR:cw_svg_minter={}", suite.minter.addr_str()?);
    println!("CONTRACT_ADDR:cw721_svg={}", suite.cwsvg.addr_str()?);
    println!(
        "CONTRACT_ADDR:cw_shitstrap_factory={}",
        suite.shit.factory.addr_str()?
    );
    // Infuser is only instantiated when infusions are enabled
    if let Ok(addr) = suite.infuser.addr_str() {
        println!("CONTRACT_ADDR:cw_infuser={}", addr);
    } else {
        println!("SKIP: cw_infuser not instantiated");
    }

    Ok(())
}

fn workflow(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::new(network.clone())
        .handle(rt.handle())
        .build()?;

    let sender = chain.sender_addr();
    let data = deploy_data(sender.clone(), &network.chain_id)?;
    let mut suite = CwSvgSuite::deploy_on(chain.clone(), data)?;
    suite.shit = CwShitstrapSuite::deploy_on(chain.clone(), shit_deploy_data(sender))?;
    // Print addresses in a format the shell script can parse
    println!("CONTRACT_ADDR:cw_svg_minter={}", suite.minter.addr_str()?);
    println!("CONTRACT_ADDR:cw721_svg={}", suite.cwsvg.addr_str()?);
    println!(
        "CONTRACT_ADDR:cw_shitstrap_factory={}",
        suite.shit.factory.addr_str()?
    );
    // Infuser is only instantiated when infusions are enabled
    if let Ok(addr) = suite.infuser.addr_str() {
        println!("CONTRACT_ADDR:cw_infuser={}", addr);
    } else {
        println!("SKIP: cw_infuser not instantiated");
    }

    Ok(())
}

fn deploy_data_single(sender: Addr, _chain_id: &str) -> Result<Option<CwSvgSuiteDeployData>> {
    let mut terp = load_svg_init_msg("scripts/svgs/interchain/terp/init.json")?;
    let mut mt = load_terp_warrior_mtree("data/mtree-init.json")?;
    terp.creator = Some(sender.to_string());
    // Set admin to the deploying address at runtime rather than baking it into the JSON
    mt.admins = vec![sender.to_string()];
    Ok(Some(CwSvgSuiteDeployData {
        svg: vec![(terp, Some(mt))],
        infuse: None,
        admin: Some(sender.clone()),
        infuse_coins: vec![],
        shit: None,
    }))
}

fn deploy_data(sender: Addr, _chain_id: &str) -> Result<Option<CwSvgSuiteDeployData>> {
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
    let mut osmo = load_svg_init_msg("scripts/svgs/interchain/osmosis/init.json")?;
    terp.creator = Some(sender.to_string());
    dao.creator = Some(sender.to_string());
    atom.creator = Some(sender.to_string());
    btc.creator = Some(sender.to_string());
    akt.creator = Some(sender.to_string());
    um.creator = Some(sender.to_string());
    btsg.creator = Some(sender.to_string());
    bcna.creator = Some(sender.to_string());
    monero.creator = Some(sender.to_string());
    eth.creator = Some(sender.to_string());
    zec.creator = Some(sender.to_string());
    osmo.creator = Some(sender.to_string());

    let mut svg = vec![(terp, None)];
    svg.extend(vec![
        (dao, None),
        // atom,
        // btc,
        // akt,
        // um,
        // btsg,
        // bcna,
        // monero,
        // eth,
        // zec,
        // osmo,
    ]);

    Ok(Some(CwSvgSuiteDeployData {
        svg,
        infuse: None,
        admin: Some(sender.clone()),
        infuse_coins: vec![],
        shit: None,
    }))
}

pub fn shit_deploy_data_single(admin: Addr) -> Option<CwShitstrapSuiteDeployData> {
    let mut dd = CwShitstrapSuiteDeployData::default();
    let mut shit = Vec::new();

    // Fixed exchange: 20 THIOL in → 1 TERP out
    // rate = 1/20 uterp per uthiol × 1e18 = 50_000_000_000_000_000
    // cutoff: 500,000 TERP (500_000_000_000 uterp)
    shit.push(ShitInitMsg {
        daos: Vec::new(),
        owner: Some(admin.to_string()),
        accepted: vec![cw_shitstrap::PossibleShit::native_denom(
            "uthiol",
            50_000_000_000_000_000u128,
        )],
        cutoff: 500_000_000_000u128.into(),
        shitmos: UncheckedDenom::Native("uterp".into()),
        title: "terp".into(),
        description: "terp".into(),
    });

    dd.admin = Some(admin.clone());
    dd.shit = shit;
    Some(dd)
}

pub fn shit_deploy_data(admin: Addr) -> Option<CwShitstrapSuiteDeployData> {
    let mut dd = CwShitstrapSuiteDeployData::default();
    let mut shit = Vec::new();

    // Spot prices @ 2026-03-02 — THIOL ≈ $0.01
    // Cutoff: uthiol (6 decimals) — 710_000_000_000u128 = 710,000 THIOL = $7,100 payout cap
    let cut = 710_000_000_000u128;

    // atom @ $1.81
    let rate = calc_rates(Decimal::from_ratio(181u128, 100u128));
    let tf = tf_denom(&admin, "atom");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "atom"));

    // btc @ $66,350
    let rate = calc_rates(Decimal::from_ratio(66350u128, 1u128));
    let tf = tf_denom(&admin, "btc");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "btc"));

    // akt @ $0.29
    let rate = calc_rates(Decimal::from_ratio(29u128, 100u128));
    let tf = tf_denom(&admin, "akt");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "akt"));

    // um @ $0.007
    let rate = calc_rates(Decimal::from_ratio(7u128, 1000u128));
    let tf = tf_denom(&admin, "um");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "um"));

    // bcna @ $0.00006686
    let rate = calc_rates(Decimal::from_ratio(6686u128, 100_000_000u128));
    let tf = tf_denom(&admin, "bcna");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "btsg"));

    // monero @ $350.76
    let rate = calc_rates(Decimal::from_ratio(35076u128, 100u128));
    let tf = tf_denom(&admin, "monero");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "monero"));

    // eth @ $1,956.12
    let rate = calc_rates(Decimal::from_ratio(195612u128, 100u128));
    let tf = tf_denom(&admin, "eth");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "eth"));

    // zec @ $215.16
    let rate = calc_rates(Decimal::from_ratio(21516u128, 100u128));
    let tf = tf_denom(&admin, "zec");
    shit.push(build_shit_init(&admin, &tf, rate, cut, "zec"));

    dd.admin = Some(admin);
    dd.shit = shit;

    Some(dd)
}

// calc_rates: USD spot price → shit_rate at 1e18 precision.
//
// THIOL target price: $0.01 → users receive (price / 0.01) = price × 100 THIOL
// per accepted token. Raw atomics are scaled by 100 to avoid Decimal × Decimal
// overflow on high-price tokens (e.g. BTC).
//
// e.g. ATOM @ $1.81   → 181_000_000_000_000_000_000   (181 THIOL per ATOM)
//      BTC  @ $66,350 → 6_635_000_000_000_000_000_000  (6,635,000 THIOL per BTC)
pub fn calc_rates(price: Decimal) -> u128 {
    price.atomics().u128() * 100
}

/// Returns the tokenfactory denom for a given creator and subdenom.
/// Format: factory/<creator>/<subdenom>
pub fn tf_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{}/{}", creator, subdenom)
}

pub fn build_shit_init(
    admin: &Addr,
    native_denom: &str,
    shit_rate: u128,
    cutoff: u128,
    title: &str,
) -> ShitInitMsg {
    ShitInitMsg {
        daos: Vec::new(),
        owner: Some(admin.to_string()),
        accepted: vec![cw_shitstrap::PossibleShit::native_denom(
            native_denom,
            shit_rate,
        )],
        cutoff: cutoff.into(),
        shitmos: UncheckedDenom::Native("uthiol".into()),
        title: title.into(),
        description: title.into(),
    }
}
