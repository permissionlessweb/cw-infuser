use clap::{arg, command, Parser};
use cw_infuser::msg::ExecuteMsgFns;
use cw_infuser::state::Infusion;
use cw_orch::core::serde_json;
use cw_orch::daemon::TxSender;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::ELGAFAR_1;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// create infusion json message
    #[arg(short, long)]
    infusion: String,
    /// optional recipient of infusions. defaults to sender
    #[arg(short, long)]
    treasury: Option<String>,
}

// cargo run -- --infusion-id <ID> --infusion-json '{}'
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();
    let chain = Daemon::builder(ELGAFAR_1).build()?;
    // chain.authz_granter(CONTRACT_MIGRATION_OWNER);
    let infusions: Vec<Infusion> = serde_json::from_str(&args.infusion)?;
    // require flags to be infusion id and json string of infusion

    let infuser = CwInfuser::new(chain.clone());
    infuser.upload()?;

    infuser.create_infusion(
        infusions,
        Some(Addr::unchecked(
            args.treasury
                .unwrap_or(chain.sender().address().to_string()),
        )),
    )?;

    Ok(())
}
