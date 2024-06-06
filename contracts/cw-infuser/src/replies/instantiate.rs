use crate::contract::{CwInfuser, CwInfuserResult};

use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply};

pub fn instantiate_reply(_deps: DepsMut, _env: Env, app: CwInfuser, _reply: Reply) -> CwInfuserResult {
    Ok(app.response("instantiate_reply"))
}

pub fn create_infusion_reply(_deps: DepsMut, _env: Env, app: CwInfuser, _reply: Reply) -> CwInfuserResult {
    Ok(app.response("create_infusion_reply"))
}
