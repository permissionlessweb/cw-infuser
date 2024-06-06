use crate::{
    contract::{CwInfuser, CwInfuserResult},
    msg::CwInfuserInstantiateMsg,
    state::{Config, CONFIG},
};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: CwInfuser,
    msg: CwInfuserInstantiateMsg,
) -> CwInfuserResult {
    let CwInfuserInstantiateMsg {
        default_infusion_params,
    } = msg;

    CONFIG.save(
        deps.storage,
        &Config {
            default_infusion_params,
            latest_infusion_id: None,
        },
    )?;
    Ok(Response::new())
}
