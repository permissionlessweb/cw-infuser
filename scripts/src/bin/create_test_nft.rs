use clap::{arg, command, Parser};
use cw_infuser::msg::ExecuteMsgFns;
use cw_infuser::state::Infusion;
use cw_orch::core::serde_json;
use cw_orch::daemon::TxSender;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::{ELGAFAR_1, STARGAZE_1};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// create infusion json message
    #[arg(short, long)]
    network: String,
}

// cargo run --bin create -- --infusion-json '{}' --treasury <optional>
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();
    let chain = Daemon::builder(ELGAFAR_1).build()?;

    let network = match args.network.as_str() {
        "testnet" => ELGAFAR_1,
        "mainnet" => STARGAZE_1,
        _ => panic!(),
    };

    // 
    // chain.authz_granter(CONTRACT_MIGRATION_OWNER);

    Ok(())
}
