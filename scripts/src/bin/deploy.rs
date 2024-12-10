use clap::Parser;
use cw_infuser::msg::InstantiateMsg;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::{ELGAFAR_1, STARGAZE_1};

const CONTRACT_MIGRATION_OWNER: &str = "stars1ampqmqrmuc03d7828qqw296q9ygnt5quf778hv";
const CW721_CODE_ID: u64 = 274; 

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// create infusion json message
    #[arg(short, long)]
    network: String,
}

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();

    let network = match args.network.as_str() {
        "testnet" => ELGAFAR_1,
        "mainnet" => STARGAZE_1,
        _ => panic!(),
    };
    let chain = Daemon::builder(network.clone()).build()?;
    // chain.authz_granter(CONTRACT_MIGRATION_OWNER);

    let mut infuser = CwInfuser::new(chain.clone());
    infuser.upload()?;

    infuser.instantiate(
        &InstantiateMsg {
            admin: None,
            min_per_bundle: None,
            max_per_bundle: None,
            max_bundles: None,
            max_infusions: None,
            cw721_code_id: CW721_CODE_ID,
        },
        Some(&Addr::unchecked(CONTRACT_MIGRATION_OWNER)),
        None,
    )?;

    Ok(())
}
