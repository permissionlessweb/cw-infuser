use clap::{arg, command, Parser};
use cw_infuser::msg::ExecuteMsgFns;
use cw_infuser::state::Bundle;
use cw_orch::core::serde_json;
use cw_orch::daemon::TxSender as _;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::ELGAFAR_1;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// create infusion json message
    #[arg(short, long)]
    id: u64,
    /// optional recipient of infusions. defaults to sender
    #[arg(short, long)]
    bundles: String,
}

// cargo run -- --infusion-id <ID> --infusion-json '{}'
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();
    let chain = Daemon::builder(ELGAFAR_1).build()?;
    let bundles: Vec<Bundle> = serde_json::from_str(&args.bundles)?;
    let infuser = CwInfuser::new(chain.clone());
    infuser.upload()?;
    infuser.infuse(bundles, args.id)?;

    Ok(())
}
