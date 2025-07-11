use clap::Parser;
use cosmwasm_std::Decimal;
use cw_infuser::msg::InstantiateMsg;
use cw_infuser_scripts::CwInfuser;
use cw_infuser_scripts::{ELGAFAR_1, STARGAZE_1};
use cw_orch::prelude::*;

const CONTRACT_MIGRATION_OWNER: &str = "stars1ampqmqrmuc03d7828qqw296q9ygnt5quf778hv";
const CW721_CODE_ID: u64 = 274;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// select network
    #[arg(short, long)]
    network: String,
    #[arg(long)]
    admin: Option<String>,
    #[arg(long)]
    min_creation_fee_denom: Option<String>,
    #[arg(long)]
    min_creation_fee_amount: Option<String>,
    #[arg(long)]
    min_infusion_fee_denom: Option<String>,
    #[arg(long)]
    min_infusion_fee_amount: Option<String>,
    #[arg(long)]
    min_per_bundle: Option<String>,
    #[arg(long)]
    max_infusions: Option<String>,
    #[arg(long)]
    max_bundles: Option<String>,
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

    let infuser = CwInfuser::new(chain.clone());
    infuser.upload()?;

    infuser.instantiate(
        &InstantiateMsg {
            contract_owner: None,
            min_per_bundle: None,
            max_per_bundle: None,
            max_bundles: None,
            max_infusions: None,
            cw721_code_id: CW721_CODE_ID,
            owner_fee: Decimal::new(10u128.into()),
            min_creation_fee: None,
            min_infusion_fee: None,
            wavs_public_key: None,
        },
        Some(&Addr::unchecked(CONTRACT_MIGRATION_OWNER)),
        &[],
    )?;

    Ok(())
}
