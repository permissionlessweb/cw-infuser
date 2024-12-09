use cw_infuser::msg::InstantiateMsg;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::{ELGAFAR_1, STARGAZE_1};

const CONTRACT_MIGRATION_OWNER: &str = "";
const CW721_CODE_ID: u64 = 69;

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let chain = Daemon::builder(ELGAFAR_1).build()?;
    // chain.authz_granter(CONTRACT_MIGRATION_OWNER);

    let infuser = CwInfuser::new(chain.clone());
    infuser.upload()?;

    let chain = Daemon::builder(ELGAFAR_1).build()?;

    let infuser = CwInfuser::new(chain.clone());
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
