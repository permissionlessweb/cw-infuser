use crate::{
    contract::{CwInfuser, CwInfuserResult},
    msg::{ConfigResponse, CwInfuserQueryMsg, InfusionsResponse},
    state::{Config, Infusion, CONFIG, INFUSION, INFUSION_ID, INFUSION_INFO},
};

use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &CwInfuser,
    msg: CwInfuserQueryMsg,
) -> CwInfuserResult<Binary> {
    match msg {
        CwInfuserQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        CwInfuserQueryMsg::Infusion { addr, id } => {
            to_json_binary(&query_infusion(deps, addr, id)?)
        }
        CwInfuserQueryMsg::InfusionById { id } => to_json_binary(&query_infusion_by_id(deps, id)?),
        CwInfuserQueryMsg::Infusions { addr } => to_json_binary(&query_infusions(deps, addr)?),
        CwInfuserQueryMsg::IsInBundle { collection_addr } => {
            to_json_binary(&query_infusions(deps, collection_addr)?)
        }
    }
    .map_err(Into::into)
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        infusion_params: state.default_infusion_params,
    };

    Ok(resp)
}

pub fn query_infusion(deps: Deps, addr: Addr, id: u64) -> StdResult<Infusion> {
    let infusion = INFUSION.load(deps.storage, (addr, id))?;
    Ok(infusion)
}
pub fn query_infusion_by_id(deps: Deps, id: u64) -> StdResult<Infusion> {
    let infuser = INFUSION_ID.load(deps.storage, id)?;
    let infusion = INFUSION.load(deps.storage, infuser)?;
    Ok(infusion)
}

pub fn query_infusions(deps: Deps, addr: Addr) -> StdResult<InfusionsResponse> {
    let mut infusions = vec![];
    let current_id = INFUSION_INFO.load(deps.storage, &addr.clone())?.next_id;

    for i in 0..=current_id {
        let id = i as u64;
        // return the response for each
        let state = INFUSION.load(deps.storage, (addr.clone(), id))?;
        infusions.push(state);
    }

    Ok(InfusionsResponse {
        infusions: infusions,
    })
}
